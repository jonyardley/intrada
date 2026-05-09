//! Process-local per-token rate limiter for `/api/mcp/*`.
//!
//! Fixed 60-second windows, 60 requests per token. Bucket key is
//! `mcp_tokens.id` (ULID) so OAuth-minted tokens and manually-created
//! PATs share the same per-token limit. Bypasses [`AuthSource::Jwt`]
//! (web/iOS UI) and [`AuthSource::Disabled`] (local dev).
//!
//! State is process-local — no Redis, no DB. On Fly with
//! `auto_stop_machines = 'suspend'` the bucket resets when the machine
//! suspends; this is fine for v1 (`specs/mcp-server.md` §238 acknowledges
//! it). If we ever pin multiple machines, "60/min" becomes "60/min/machine
//! × N" and we'd need a distributed bucket.
//!
//! Window-edge bursting (up to ~120 reqs in a 2-second sliver across the
//! window boundary) is acceptable per spec wording — it asks for 60/min,
//! not "smoothed". A sliding-window / GCRA upgrade is captured as a
//! follow-up under #578.
//!
//! [`AuthSource::Jwt`]: crate::auth::AuthSource::Jwt
//! [`AuthSource::Disabled`]: crate::auth::AuthSource::Disabled

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{Request, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::auth::{resolve_pat, AuthSource, AuthUser};
use crate::db::tokens::TOKEN_PREFIX as PAT_PREFIX;
use crate::state::AppState;

/// When the bucket map grows past this, run an opportunistic cleanup pass
/// on the next insert. At ~32 bytes per entry the ceiling is ~32 KiB —
/// we'd never actually OOM, but unbounded growth is sloppy.
const MAP_SIZE_THRESHOLD: usize = 1024;

/// Per-token request bucket. Lock contention is bounded (one lock per
/// MCP request, held for the duration of a HashMap update) so a plain
/// `std::sync::Mutex` is fine — no async locking is needed.
pub struct McpRateLimiter {
    state: Mutex<HashMap<String, (Instant, u32)>>,
    limit: u32,
    window: Duration,
}

impl McpRateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            limit,
            window,
        }
    }

    /// Production setting: 60 requests per 60 seconds, per token.
    pub fn production() -> Self {
        Self::new(60, Duration::from_secs(60))
    }

    /// Record a request against `token_id` and decide whether to allow it.
    /// `Ok(())` = allow, `Err(retry_after_seconds)` = reject with 429.
    pub fn check(&self, token_id: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut state = self.state.lock().expect("rate-limit state poisoned");

        if let Some((window_start, count)) = state.get_mut(token_id) {
            if now.duration_since(*window_start) >= self.window {
                *window_start = now;
                *count = 1;
                return Ok(());
            }
            if *count < self.limit {
                *count += 1;
                return Ok(());
            }
            let retry_after = self
                .window
                .saturating_sub(now.duration_since(*window_start))
                .as_secs()
                // Never advertise 0 — clients then retry instantly and
                // amplify the burst we're trying to suppress.
                .max(1);
            return Err(retry_after);
        }

        // New entry. Run sampled cleanup before insert if the map has
        // grown past the threshold — keeps it bounded without a reaper task.
        if state.len() >= MAP_SIZE_THRESHOLD {
            let cutoff = self.window * 2;
            state.retain(|_, (start, _)| now.duration_since(*start) < cutoff);
        }
        state.insert(token_id.to_string(), (now, 1));
        Ok(())
    }
}

/// Middleware applied to the `/api/mcp` nest. Eagerly resolves PATs so
/// the bucket can be checked before the handler runs. Note that handlers
/// will still re-resolve via the `AuthUser` extractor — at 60 req/min/token
/// the duplicate `lookup_by_hash` is negligible vs. the plumbing cost of
/// caching the extraction result in `request.extensions`.
///
/// **Known limitation**: any bearer that starts with `intrada_pat_` triggers
/// one `lookup_by_hash` call here, even if the token is malformed or unknown.
/// A high-rate flood of bogus tokens will burn DB lookups (and hash CPU)
/// without consuming any bucket. The bucket-keyed-on-resolved-`token_id`
/// design protects legitimate tokens from being starved by such a flood,
/// but does not bound the flood's cost itself. A separate per-IP layer
/// covering both this surface and unauthenticated `/oauth/register` is
/// captured as a follow-up under #578.
pub async fn mcp_rate_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // CORS preflight: pass through without consuming the bucket. The
    // outer CORS layer is what actually responds to OPTIONS, but
    // short-circuiting here is cheap insurance against a future config
    // change that lets preflights reach this layer.
    if req.method() == Method::OPTIONS {
        return next.run(req).await;
    }

    let bearer = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    // Only PAT-shaped tokens trigger the rate limit. JWTs (web/iOS UI),
    // missing/invalid auth, and Disabled-mode requests pass straight through —
    // the handler-level `AuthUser` extractor produces the right 200/401.
    if let Some(token) = bearer.filter(|t| t.starts_with(PAT_PREFIX)) {
        if let Ok(AuthUser {
            source: AuthSource::Pat { token_id },
            ..
        }) = resolve_pat(&state, token).await
        {
            if let Err(retry_after) = state.rate_limiter.check(&token_id) {
                // info!, not warn! — rate-limit hits are expected operational
                // signal, not errors. We want them greppable in logs without
                // tripping Sentry's default ERROR-only event capture.
                tracing::info!(token_id = %token_id, "mcp rate limit exceeded");
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(header::RETRY_AFTER, retry_after.to_string())],
                    Json(json!({"error": "rate limit exceeded"})),
                )
                    .into_response();
            }
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_allows_up_to_limit_then_returns_retry_after() {
        let limiter = McpRateLimiter::new(2, Duration::from_secs(60));

        assert!(limiter.check("tok").is_ok());
        assert!(limiter.check("tok").is_ok());
        let err = limiter
            .check("tok")
            .expect_err("3rd call should be rejected");
        assert!(
            (1..=60).contains(&err),
            "retry_after should be in (0, window]; got {err}"
        );
    }

    #[test]
    fn check_resets_after_window() {
        let limiter = McpRateLimiter::new(2, Duration::from_millis(50));

        assert!(limiter.check("tok").is_ok());
        assert!(limiter.check("tok").is_ok());
        assert!(limiter.check("tok").is_err());
        std::thread::sleep(Duration::from_millis(60));
        assert!(
            limiter.check("tok").is_ok(),
            "bucket should reset after window"
        );
    }

    #[test]
    fn separate_tokens_have_separate_buckets() {
        let limiter = McpRateLimiter::new(1, Duration::from_secs(60));

        assert!(limiter.check("tok_a").is_ok());
        assert!(limiter.check("tok_b").is_ok());
        assert!(limiter.check("tok_a").is_err());
        assert!(limiter.check("tok_b").is_err());
    }

    #[test]
    fn cleanup_drops_stale_entries_when_map_grows() {
        // Limit large so we never reject; window short so entries become
        // stale fast. Insert MAP_SIZE_THRESHOLD entries to trip cleanup.
        let limiter = McpRateLimiter::new(1000, Duration::from_millis(10));
        for i in 0..MAP_SIZE_THRESHOLD {
            let _ = limiter.check(&format!("tok_{i}"));
        }
        // Wait past 2× window so all entries are eligible for cleanup.
        std::thread::sleep(Duration::from_millis(25));
        // One more insert triggers the retain pass.
        let _ = limiter.check("trigger");

        let state = limiter.state.lock().unwrap();
        assert!(
            state.len() <= 1,
            "stale entries should have been reaped; got {}",
            state.len()
        );
    }
}

//! Process-local rate limiters for `/api/mcp/*` and OAuth endpoints.
//!
//! Two flavours:
//! - [`McpRateLimiter`] — per-token fixed-window bucket (60 req/min/token).
//!   Applied to `/api/mcp` by `mcp_rate_limit`. Bypasses JWT + Disabled.
//! - [`IpRateLimiter`] — per-IP fixed-window bucket. Two instances:
//!   - MCP (300 req/min/IP): early reject for bogus-PAT floods before
//!     they burn `lookup_by_hash` DB calls.
//!   - OAuth (20 req/min/IP): DoS guard on the unauthenticated DCR
//!     endpoint (`/oauth/register`) that creates a DB row per call.
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

/// Per-IP fixed-window rate limiter. Bucket key is the client IP string
/// (extracted from `Fly-Client-IP` or `X-Forwarded-For`).
///
/// Used for two purposes with different production limits:
/// - MCP IP guard (300 req/min/IP): reject bogus-PAT floods before they
///   reach the per-token bucket and burn `lookup_by_hash` DB calls.
/// - OAuth register guard (20 req/min/IP): DoS protection on the
///   unauthenticated DCR endpoint which creates a DB row per call.
pub struct IpRateLimiter {
    state: Mutex<HashMap<String, (Instant, u32)>>,
    limit: u32,
    window: Duration,
}

impl IpRateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            limit,
            window,
        }
    }

    /// MCP IP guard: 300 req/min/IP. Generous enough for multiple agents
    /// behind a shared home IP (e.g. Claude Desktop + Cursor = 2 × 60
    /// = 120 req/min) while blocking flood-level abuse.
    pub fn production_mcp() -> Self {
        Self::new(300, Duration::from_secs(60))
    }

    /// OAuth register guard: 20 req/min/IP. Tight because each call
    /// mints a new OAuth client in the DB; legitimate flows make at
    /// most 1 registration per session.
    pub fn production_oauth() -> Self {
        Self::new(20, Duration::from_secs(60))
    }

    /// Record a request from `ip` and decide whether to allow it.
    /// `Ok(())` = allow, `Err(retry_after_seconds)` = reject with 429.
    pub fn check(&self, ip: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut state = self.state.lock().expect("ip rate-limit state poisoned");

        if let Some((window_start, count)) = state.get_mut(ip) {
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
                .max(1);
            return Err(retry_after);
        }

        if state.len() >= MAP_SIZE_THRESHOLD {
            let cutoff = self.window * 2;
            state.retain(|_, (start, _)| now.duration_since(*start) < cutoff);
        }
        state.insert(ip.to_string(), (now, 1));
        Ok(())
    }
}

/// Extract the best available client IP from request headers.
///
/// Fly.io sets `Fly-Client-IP` to the real originating IP (trusted).
/// Falls back to the first entry of `X-Forwarded-For` for non-Fly
/// deployments. Returns `"unknown"` if neither header is present — rate
/// limits keyed on `"unknown"` are shared across all such requests, which
/// is intentionally conservative.
fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("fly-client-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Middleware applied to the `/api/mcp` nest. Two-stage check:
///
/// 1. **IP bucket** (`state.mcp_ip_limiter`): fast path — any request
///    from a flooding IP is rejected before hitting the DB. Catches
///    bogus-PAT floods that previously slipped through to `resolve_pat`
///    and burned `lookup_by_hash` calls.
/// 2. **Token bucket** (`state.rate_limiter`): per-PAT check after the IP
///    check passes. Handlers still re-resolve via the `AuthUser` extractor
///    — at 60 req/min/token the duplicate `lookup_by_hash` is negligible
///    vs the plumbing cost of caching the extraction result in extensions.
///
/// JWTs (web/iOS UI), missing/invalid auth, and Disabled-mode requests
/// bypass the token bucket but still pass through the IP check.
pub async fn mcp_rate_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // CORS preflight: pass through without consuming either bucket.
    if req.method() == Method::OPTIONS {
        return next.run(req).await;
    }

    // Stage 1: IP-level check — catches floods before DB involvement.
    let ip = client_ip(req.headers());
    if let Err(retry_after) = state.mcp_ip_limiter.check(&ip) {
        tracing::info!(%ip, "mcp ip rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(json!({"error": "rate limit exceeded"})),
        )
            .into_response();
    }

    // Stage 2: per-token check. Only PAT-shaped tokens are bucketed.
    let bearer = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

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

/// Middleware applied to the `/oauth/register` route. Enforces a per-IP
/// fixed-window limit (production: 20 req/min/IP) to prevent a single
/// client from flooding the DB with DCR registrations.
///
/// OPTIONS preflights are passed through without consuming the bucket.
/// All other methods — including unauthenticated POST — are subject to
/// the limit. This is intentional: OAuth registration is a one-time
/// operation per client; 20/min is generous for legitimate automated
/// registrations while blocking obvious abuse.
pub async fn oauth_ip_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if req.method() == Method::OPTIONS {
        return next.run(req).await;
    }

    let ip = client_ip(req.headers());
    if let Err(retry_after) = state.oauth_ip_limiter.check(&ip) {
        tracing::info!(%ip, "oauth register rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            Json(json!({"error": "rate limit exceeded"})),
        )
            .into_response();
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── McpRateLimiter (per-token) ────────────────────────────────────────

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

    // ── IpRateLimiter (per-IP) ────────────────────────────────────────────

    #[test]
    fn ip_limiter_allows_up_to_limit_then_rejects() {
        let limiter = IpRateLimiter::new(3, Duration::from_secs(60));

        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_ok());
        let err = limiter
            .check("1.2.3.4")
            .expect_err("4th call should be rejected");
        assert!(
            (1..=60).contains(&err),
            "retry_after should be in (0, window]; got {err}"
        );
    }

    #[test]
    fn ip_limiter_separate_ips_have_separate_buckets() {
        let limiter = IpRateLimiter::new(1, Duration::from_secs(60));

        assert!(limiter.check("1.1.1.1").is_ok());
        assert!(limiter.check("2.2.2.2").is_ok());
        assert!(limiter.check("1.1.1.1").is_err());
        assert!(limiter.check("2.2.2.2").is_err());
    }

    #[test]
    fn ip_limiter_resets_after_window() {
        let limiter = IpRateLimiter::new(1, Duration::from_millis(50));

        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_err());
        std::thread::sleep(Duration::from_millis(60));
        assert!(
            limiter.check("1.2.3.4").is_ok(),
            "bucket should reset after window"
        );
    }

    // ── client_ip helper ─────────────────────────────────────────────────

    #[test]
    fn client_ip_prefers_fly_client_ip() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("fly-client-ip", "1.2.3.4".parse().unwrap());
        headers.insert("x-forwarded-for", "5.6.7.8, 9.10.11.12".parse().unwrap());
        assert_eq!(client_ip(&headers), "1.2.3.4");
    }

    #[test]
    fn client_ip_falls_back_to_x_forwarded_for_first_entry() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "5.6.7.8, 9.10.11.12".parse().unwrap());
        assert_eq!(client_ip(&headers), "5.6.7.8");
    }

    #[test]
    fn client_ip_returns_unknown_when_no_headers() {
        let headers = axum::http::HeaderMap::new();
        assert_eq!(client_ip(&headers), "unknown");
    }
}

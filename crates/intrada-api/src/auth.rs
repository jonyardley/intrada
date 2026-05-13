use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db;
use crate::db::tokens::{PatLookup, TOKEN_PREFIX as PAT_PREFIX};
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct AuthConfig {
    pub issuer: String,
    pub decoding_keys: Arc<RwLock<Vec<DecodingKey>>>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

/// How an `AuthUser` was produced. Phase 4's `mcp_audit_log` will record the
/// `token_id` for every MCP write — surfacing it through the extractor here
/// avoids a redundant DB lookup per write handler.
#[derive(Debug, Clone)]
pub enum AuthSource {
    /// Clerk JWT validation passed.
    Jwt,
    /// MCP Personal Access Token resolved.
    Pat { token_id: String },
    /// `CLERK_ISSUER_URL` is unset and no PAT was provided. Local dev only.
    // TODO(#477 phase 4): MCP write tools must refuse to record audit-log
    // rows when source is Disabled (otherwise the audit table will record
    // anonymous-but-attributed-to-empty-user_id rows in dev). The contract
    // is captured here; enforcement lives with the audit-log impl.
    Disabled,
}

/// Extractor that yields the authenticated user's ID and how the request
/// authenticated.
///
/// Resolution order:
/// 1. `Authorization: Bearer intrada_pat_…` → MCP PAT lookup. Works whether
///    or not Clerk is configured, so MCP can authenticate against an API
///    instance running without `CLERK_ISSUER_URL` (dev / local).
/// 2. No `auth_config` (Clerk disabled) → empty `user_id` + `AuthSource::Disabled`.
/// 3. JWT validation against the configured Clerk issuer.
pub struct AuthUser {
    pub user_id: String,
    pub source: AuthSource,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let bearer = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        // PAT path runs first so it works in auth-disabled mode too.
        if let Some(token) = bearer.filter(|t| t.starts_with(PAT_PREFIX)) {
            return resolve_pat(state, token).await;
        }

        let auth_config = match &state.auth_config {
            Some(config) => config,
            None => {
                // Defensive: if anything earlier in the middleware chain ever
                // attaches a user to this per-request hub, make sure the
                // auth-disabled path doesn't inherit it.
                sentry::configure_scope(|scope| scope.set_user(None));
                return Ok(AuthUser {
                    user_id: String::new(),
                    source: AuthSource::Disabled,
                });
            }
        };

        let token = bearer.ok_or_else(|| ApiError::Unauthorized("Unauthorized".to_string()))?;

        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[&auth_config.issuer]);
        // Clerk tokens include an `aud` claim but we don't validate a specific
        // audience. jsonwebtoken v10 defaults validate_aud = true, which would
        // reject all tokens unless an audience is configured.
        validation.validate_aud = false;

        let keys = auth_config.decoding_keys.read().await;
        let mut last_error = None;
        for key in keys.iter() {
            match decode::<Claims>(token, key, &validation) {
                Ok(data) => {
                    let user_id = data.claims.sub;
                    set_sentry_user(&user_id);
                    return Ok(AuthUser {
                        user_id,
                        source: AuthSource::Jwt,
                    });
                }
                Err(e) => {
                    tracing::debug!("JWT decode error: {e}");
                    last_error = Some(e);
                    continue;
                }
            }
        }

        if let Some(e) = last_error {
            tracing::warn!(
                issuer = %auth_config.issuer,
                key_count = keys.len(),
                "JWT validation failed after trying all keys: {e}"
            );
        } else {
            tracing::warn!("JWT validation failed: no decoding keys loaded");
        }
        Err(ApiError::Unauthorized("Unauthorized".to_string()))
    }
}

/// Resolve a Bearer-PAT token to an [`AuthUser`].
///
/// Visible to the rate-limit middleware so it can attribute requests to a
/// `token_id` before the handler-level extractor runs. Reused by the
/// [`AuthUser`] `FromRequestParts` impl below.
pub(crate) async fn resolve_pat(state: &AppState, token: &str) -> Result<AuthUser, ApiError> {
    let conn = state.conn();
    let hash = db::tokens::hash_token(token);

    match db::tokens::lookup_by_hash(&conn, &hash).await {
        Ok(Some(PatLookup {
            token_id,
            user_id,
            revoked_at: None,
        })) => {
            // Best-effort `last_used_at` update — auth has already succeeded,
            // so a write failure here shouldn't deny access.
            if let Err(e) = db::tokens::mark_used(&conn, &hash).await {
                tracing::warn!(?e, "PAT mark_used update failed");
            }
            set_sentry_user(&user_id);
            Ok(AuthUser {
                user_id,
                source: AuthSource::Pat { token_id },
            })
        }
        Ok(Some(PatLookup {
            revoked_at: Some(_),
            ..
        })) => {
            // Revoked. Distinct log line so revoked-token use is grep-able
            // separately from "unknown PAT".
            tracing::info!("PAT auth rejected: token revoked");
            Err(ApiError::Unauthorized("Unauthorized".to_string()))
        }
        Ok(None) => {
            tracing::debug!("PAT auth rejected: token not found");
            Err(ApiError::Unauthorized("Unauthorized".to_string()))
        }
        Err(e) => {
            tracing::warn!(?e, "PAT lookup DB error");
            Err(ApiError::Internal("Auth DB error".into()))
        }
    }
}

fn set_sentry_user(user_id: &str) {
    // Per-request hub isolation comes from the NewSentryLayer in
    // routes/mod.rs — configuring the current scope here cannot bleed into
    // other requests.
    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            id: Some(user_id.to_string()),
            ..Default::default()
        }));
    });
}

impl AuthConfig {
    /// Re-fetch JWKS keys from the issuer and replace the cached keys.
    /// On failure, the existing keys are kept.
    pub async fn refresh_jwks(&self) {
        match fetch_jwks(&self.issuer).await {
            Ok(new_keys) => {
                let count = new_keys.len();
                let mut keys = self.decoding_keys.write().await;
                *keys = new_keys;
                tracing::info!("JWKS refreshed: {count} key(s) loaded");
            }
            Err(e) => {
                tracing::warn!("JWKS refresh failed (keeping existing keys): {e}");
            }
        }
    }
}

/// Fetch JWKS from the Clerk issuer URL and return decoding keys.
///
/// Uses `jsonwebtoken::jwk::JwkSet` to deserialize the full JWK objects
/// and `DecodingKey::from_jwk()` to properly construct keys with all
/// metadata (kty, alg, use, kid, n, e).
pub async fn fetch_jwks(
    issuer_url: &str,
) -> Result<Vec<DecodingKey>, Box<dyn std::error::Error + Send + Sync>> {
    let jwks_url = format!("{}/.well-known/jwks.json", issuer_url.trim_end_matches('/'));
    let jwk_set: JwkSet = reqwest::get(&jwks_url).await?.json().await?;

    tracing::info!("JWKS contains {} key(s)", jwk_set.keys.len());

    let mut keys = Vec::new();
    for jwk in &jwk_set.keys {
        match DecodingKey::from_jwk(jwk) {
            Ok(key) => {
                tracing::info!("Loaded JWK key successfully");
                keys.push(key);
            }
            Err(e) => {
                tracing::warn!("Skipping JWK key: {e}");
            }
        }
    }

    if keys.is_empty() {
        return Err("No valid keys found in JWKS".into());
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Db;

    async fn make_state() -> AppState {
        let db_path = std::env::temp_dir().join(format!("auth_test_{}.db", ulid::Ulid::new()));
        let db = libsql::Builder::new_local(&db_path)
            .build()
            .await
            .expect("test db build");
        let conn = db.connect().expect("test db connect");
        crate::migrations::run_migrations_direct(&conn)
            .await
            .expect("test migrations");
        AppState::new(
            Db::new(db, conn),
            "http://localhost".to_string(),
            None,
            None,
            None,
        )
    }

    #[tokio::test]
    async fn pat_resolves_to_pat_source_with_correct_token_id() {
        // Locks in the contract: a successfully-resolved PAT must produce
        // AuthSource::Pat carrying the inserted token_id (not the user_id,
        // not the hash). Phase 4's audit log depends on this.
        let state = make_state().await;
        let conn = state.conn();

        let token = "intrada_pat_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
        let hash = db::tokens::hash_token(token);
        let token_id = "01HTEST00000000000000000PAT".to_string();
        let prefix = token[..16].to_string();
        db::tokens::insert(
            &conn,
            &token_id,
            "user_42",
            "auth-test",
            &hash,
            &prefix,
            chrono::Utc::now(),
        )
        .await
        .expect("insert PAT");

        let auth_user = resolve_pat(&state, token)
            .await
            .expect("PAT should resolve");
        assert_eq!(auth_user.user_id, "user_42");
        match auth_user.source {
            AuthSource::Pat { token_id: t } => assert_eq!(t, token_id),
            other => panic!("Expected AuthSource::Pat, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn revoked_pat_resolves_to_unauthorized() {
        let state = make_state().await;
        let conn = state.conn();

        let token = "intrada_pat_b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2";
        let hash = db::tokens::hash_token(token);
        let token_id = "01HTEST_REVOKED_PAT".to_string();
        db::tokens::insert(
            &conn,
            &token_id,
            "user_42",
            "auth-test",
            &hash,
            "intrada_pat_b1c2",
            chrono::Utc::now(),
        )
        .await
        .expect("insert PAT");
        db::tokens::revoke(&conn, "user_42", &token_id)
            .await
            .expect("revoke PAT");

        let result = resolve_pat(&state, token).await;
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    /// Verify that our Claims struct correctly deserializes a Clerk JWT payload.
    /// Clerk JWTs contain numeric fields (iat, exp, nbf) alongside string
    /// fields — serde must ignore the unknown ones without erroring.
    #[test]
    fn claims_deserializes_clerk_jwt_payload() {
        let payload = r#"{"azp":"pk_test_xxx","exp":1778693283,"iat":1778693223,"iss":"https://clerk.myintrada.com","nbf":1778693213,"sid":"sess_abc","sub":"user_2abc"}"#;
        let claims: Claims = serde_json::from_str(payload).expect("should parse");
        assert_eq!(claims.sub, "user_2abc");
    }

    /// Regression test for jsonwebtoken v10 bug (#489): Clerk JWTs
    /// include a numeric `oiat` field in the header, which the library's
    /// `Header.extras: HashMap<String, String>` rejected. The patched
    /// version (PR #496) uses `HashMap<String, serde_json::Value>`.
    #[test]
    fn decode_header_accepts_non_string_extras() {
        use base64::Engine;

        // Clerk production header contains `oiat` (numeric) and `cat` (string)
        let header_json =
            r#"{"alg":"RS256","cat":"cl_abc","kid":"ins_abc","oiat":1778699272,"typ":"JWT"}"#;
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header_json);
        let payload =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"sub":"user_123"}"#);
        let sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("fake");
        let token = format!("{header}.{payload}.{sig}");

        // Before the patch this returned Err("expected a string")
        let result = jsonwebtoken::decode_header(&token);
        assert!(
            result.is_ok(),
            "decode_header should accept non-string extras: {:?}",
            result
        );
    }
}

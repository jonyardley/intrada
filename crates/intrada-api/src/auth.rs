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
    /// `CLERK_ISSUER_URL` is unset and no PAT was provided. Local dev only;
    /// MCP write tools will refuse to record audit rows for this source.
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
                    continue;
                }
            }
        }

        Err(ApiError::Unauthorized("Unauthorized".to_string()))
    }
}

async fn resolve_pat(state: &AppState, token: &str) -> Result<AuthUser, ApiError> {
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

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct AuthConfig {
    pub issuer: String,
    pub decoding_keys: Arc<Vec<DecodingKey>>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

/// Extractor that yields the authenticated user's ID.
///
/// When `AppState.auth_config` is `None` (no `CLERK_ISSUER_URL` set),
/// returns `AuthUser("")` — matching the migration default and preserving
/// existing test behavior.
pub struct AuthUser(pub String);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_config = match &state.auth_config {
            Some(config) => config,
            None => return Ok(AuthUser(String::new())),
        };

        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Unauthorized".to_string()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::Unauthorized("Unauthorized".to_string()))?;

        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[&auth_config.issuer]);
        // Clerk tokens include an `aud` claim but we don't validate a specific
        // audience. jsonwebtoken v10 defaults validate_aud = true, which would
        // reject all tokens unless an audience is configured.
        validation.validate_aud = false;

        let mut last_err = None;
        for key in auth_config.decoding_keys.iter() {
            // Wrap decode in catch_unwind to guard against panics in
            // the underlying crypto library when processing malformed tokens.
            let token_owned = token.to_owned();
            let key_clone = key.clone();
            let validation_clone = validation.clone();
            let result = std::panic::catch_unwind(move || {
                decode::<Claims>(&token_owned, &key_clone, &validation_clone)
            });
            match result {
                Ok(Ok(data)) => return Ok(AuthUser(data.claims.sub)),
                Ok(Err(e)) => {
                    tracing::debug!("JWT decode error with key: {e}");
                    last_err = Some(format!("{e}"));
                    continue;
                }
                Err(_) => {
                    tracing::warn!("JWT decode panicked — treating as invalid token");
                    last_err = Some("decode panicked".to_string());
                    continue;
                }
            }
        }

        if let Some(err) = last_err {
            tracing::warn!("All JWT keys failed. Last error: {err}");
        }
        Err(ApiError::Unauthorized("Unauthorized".to_string()))
    }
}

/// Fetch JWKS from the Clerk issuer URL and return decoding keys.
///
/// Uses `jsonwebtoken::jwk::JwkSet` to deserialize the full JWK objects
/// and `DecodingKey::from_jwk()` to properly construct keys with all
/// metadata (kty, alg, use, kid, n, e).
pub async fn fetch_jwks(issuer_url: &str) -> Result<Vec<DecodingKey>, Box<dyn std::error::Error>> {
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

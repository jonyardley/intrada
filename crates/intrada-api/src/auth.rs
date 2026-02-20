use axum::extract::FromRequestParts;
use axum::http::request::Parts;
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
        // audience. jsonwebtoken v9 defaults validate_aud = true, which would
        // reject all tokens unless an audience is configured.
        validation.validate_aud = false;

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
                Ok(Err(_)) => continue,
                Err(_) => {
                    tracing::warn!("JWT decode panicked — treating as invalid token");
                    continue;
                }
            }
        }

        Err(ApiError::Unauthorized("Unauthorized".to_string()))
    }
}

#[derive(Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Deserialize)]
struct JwkKey {
    n: String,
    e: String,
}

/// Fetch JWKS from the Clerk issuer URL and return decoding keys.
pub async fn fetch_jwks(issuer_url: &str) -> Result<Vec<DecodingKey>, Box<dyn std::error::Error>> {
    let jwks_url = format!("{}/.well-known/jwks.json", issuer_url.trim_end_matches('/'));
    let resp: JwksResponse = reqwest::get(&jwks_url).await?.json().await?;

    let keys: Vec<DecodingKey> = resp
        .keys
        .iter()
        .filter_map(|k| DecodingKey::from_rsa_components(&k.n, &k.e).ok())
        .collect();

    if keys.is_empty() {
        return Err("No valid RSA keys found in JWKS".into());
    }

    Ok(keys)
}

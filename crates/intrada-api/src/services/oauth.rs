//! OAuth 2.1 protocol logic.
//!
//! Implements:
//! - **DCR (RFC 7591)** — `register_client`. Generates a client_id and
//!   (optional) client_secret for an MCP client.
//! - **Authorization (RFC 6749)** — `validate_authorize_request` +
//!   `mint_auth_code`. The authorize endpoint splits into "validate
//!   params + look up client" (synchronous) and "mint code after user
//!   consent" (called from /oauth/finalize once the user has approved).
//! - **Token exchange (RFC 6749 + 7636 PKCE)** — `exchange_code_for_token`.
//!   Verifies PKCE, mints an `intrada_pat_*` access token in the
//!   `mcp_tokens` table so the existing auth extractor handles it like
//!   any other PAT.
//!
//! Lifetime: OAuth-minted tokens have no expiry — they live in
//! `mcp_tokens` and the user revokes them from the same UI as
//! manually-created PATs. Refresh tokens aren't issued today (we'd
//! need them if we shorten access-token lifetime in the future).

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use libsql::Connection;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db;
use crate::db::oauth::{OAuthClient, OAuthCode};
use crate::error::ApiError;

const CLIENT_ID_PREFIX: &str = "intrada_client_";

/// Auth codes live for 10 minutes — long enough for a user to confirm
/// the consent dialog and the agent to round-trip /token, short enough
/// that an intercepted code is unusable after a coffee break.
const AUTH_CODE_TTL_MINUTES: i64 = 10;

// ── DCR ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterClientRequest {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    /// Per RFC 7591 §2 — clients may declare additional metadata. We
    /// accept and ignore unknown fields rather than reject (forward
    /// compat with new fields).
    #[serde(flatten)]
    #[serde(default)]
    pub _extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RegisterClientResponse {
    pub client_id: String,
    /// Returned ONCE on registration. Public clients (the typical MCP
    /// case — claude.ai talking from a browser) don't get a secret;
    /// PKCE provides authentication. We issue a secret only on request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
}

pub async fn register_client(
    conn: &Connection,
    req: RegisterClientRequest,
) -> Result<RegisterClientResponse, ApiError> {
    if req.client_name.trim().is_empty() {
        return Err(ApiError::Validation("client_name cannot be empty".into()));
    }
    if req.redirect_uris.is_empty() {
        return Err(ApiError::Validation(
            "redirect_uris must contain at least one URI".into(),
        ));
    }
    for uri in &req.redirect_uris {
        if !uri.starts_with("https://") && !uri.starts_with("http://") {
            return Err(ApiError::Validation(format!(
                "redirect_uri must be http(s): {uri}"
            )));
        }
    }

    let client_id = format!("{CLIENT_ID_PREFIX}{}", random_hex_32());
    // Public clients only — PKCE-protected, no secret. Adding secrets
    // would mean teaching DCR clients to handle them; not worth the
    // complexity for v1 since PKCE is the actual auth in OAuth 2.1
    // public-client flows.
    let client_secret_hash: Option<&str> = None;

    let redirect_uris_json = serde_json::to_string(&req.redirect_uris)
        .map_err(|e| ApiError::Internal(format!("serialize redirect_uris: {e}")))?;

    db::oauth::insert_client(
        conn,
        &client_id,
        client_secret_hash,
        &req.client_name,
        &redirect_uris_json,
        Utc::now(),
    )
    .await?;

    Ok(RegisterClientResponse {
        client_id,
        client_secret: None,
        client_name: req.client_name,
        redirect_uris: req.redirect_uris,
    })
}

// ── Authorize ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizeParams {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub scope: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// Validate the authorize request and return the registered client.
/// Caller (the route handler) decides what to do with the result —
/// typically: redirect the user to the consent page if validation
/// passes, or render an error if not.
pub async fn validate_authorize_request(
    conn: &Connection,
    params: &AuthorizeParams,
) -> Result<OAuthClient, ApiError> {
    if params.response_type != "code" {
        return Err(ApiError::Validation(format!(
            "Unsupported response_type: {} (only \"code\" is supported)",
            params.response_type
        )));
    }
    if params.code_challenge_method != "S256" {
        return Err(ApiError::Validation(format!(
            "Unsupported code_challenge_method: {} (only \"S256\" is supported)",
            params.code_challenge_method
        )));
    }
    if params.code_challenge.is_empty() {
        return Err(ApiError::Validation(
            "code_challenge is required (PKCE)".into(),
        ));
    }

    let client = db::oauth::get_client_by_id(conn, &params.client_id)
        .await?
        .ok_or_else(|| ApiError::Validation(format!("Unknown client_id: {}", params.client_id)))?;

    let registered: Vec<String> =
        serde_json::from_str(&client.redirect_uris_json).map_err(|e| {
            ApiError::Internal(format!(
                "redirect_uris not valid JSON for {}: {e}",
                client.client_id
            ))
        })?;
    if !registered.contains(&params.redirect_uri) {
        return Err(ApiError::Validation(format!(
            "redirect_uri {:?} not registered for client {}",
            params.redirect_uri, client.client_id
        )));
    }

    Ok(client)
}

/// Mint an auth code. Returns the raw code (the caller redirects the
/// user's browser to `redirect_uri?code=<this>&state=<original-state>`).
/// The hash, not the code itself, lands in the DB so an attacker with
/// DB read can't replay codes.
pub async fn mint_auth_code(
    conn: &Connection,
    user_id: &str,
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    code_challenge_method: &str,
    scope: Option<String>,
) -> Result<String, ApiError> {
    let code = random_hex_32();
    let code_hash = sha256_hex(code.as_bytes());
    let now = Utc::now();
    let expires_at = now + Duration::minutes(AUTH_CODE_TTL_MINUTES);

    db::oauth::insert_code(
        conn,
        &OAuthCode {
            code_hash,
            client_id: client_id.to_string(),
            user_id: user_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            code_challenge: code_challenge.to_string(),
            code_challenge_method: code_challenge_method.to_string(),
            scope,
            expires_at,
            created_at: now,
        },
    )
    .await?;

    Ok(code)
}

// ── Token exchange ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    /// Per RFC — `0` means "no expiration declared" (we don't expire
    /// OAuth-minted tokens for v1). Some clients require this field.
    pub expires_in: u64,
    /// Echo the requested scope so the client knows what was granted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

pub async fn exchange_code_for_token(
    conn: &Connection,
    req: TokenRequest,
) -> Result<TokenResponse, ApiError> {
    if req.grant_type != "authorization_code" {
        return Err(ApiError::Validation(format!(
            "Unsupported grant_type: {} (only \"authorization_code\" is supported)",
            req.grant_type
        )));
    }

    let code_hash = sha256_hex(req.code.as_bytes());
    let code = db::oauth::consume_code_by_hash(conn, &code_hash)
        .await?
        .ok_or_else(|| ApiError::Validation("Invalid or already-used authorization code".into()))?;

    if code.expires_at < Utc::now() {
        return Err(ApiError::Validation(
            "Authorization code expired (10-minute lifetime)".into(),
        ));
    }
    if code.client_id != req.client_id {
        return Err(ApiError::Validation(
            "client_id does not match the issued code".into(),
        ));
    }
    if code.redirect_uri != req.redirect_uri {
        return Err(ApiError::Validation(
            "redirect_uri does not match the issued code".into(),
        ));
    }

    // PKCE: verify code_challenge == base64url-no-pad(SHA256(code_verifier))
    let computed_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(req.code_verifier.as_bytes()));
    if computed_challenge != code.code_challenge {
        return Err(ApiError::Validation(
            "code_verifier does not match code_challenge".into(),
        ));
    }

    // Look up the registered client name so the minted token's display
    // name surfaces it ("OAuth: Claude" rather than the opaque client_id).
    let client = db::oauth::get_client_by_id(conn, &code.client_id)
        .await?
        .ok_or_else(|| ApiError::Internal("Client vanished mid-exchange".into()))?;

    let token_name = format!("OAuth: {}", client.client_name);
    let created = crate::services::tokens::create_token(conn, &code.user_id, &token_name).await?;

    Ok(TokenResponse {
        access_token: created.token,
        token_type: "Bearer",
        expires_in: 0,
        scope: code.scope,
    })
}

// ── Helpers ────────────────────────────────────────────────────────────

fn random_hex_32() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex_encode(&bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_encode(&digest)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verification_round_trip() {
        // RFC 7636 §4.6 example values.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        assert_eq!(
            expected_challenge,
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }
}

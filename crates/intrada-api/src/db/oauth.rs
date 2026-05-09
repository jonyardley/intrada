//! OAuth 2.1 + DCR table operations.
//!
//! Two tables:
//! - `oauth_clients` — DCR-registered MCP clients. Holds the public
//!   `client_id`, optional hashed `client_secret`, the registered
//!   `redirect_uris` (JSON array), and metadata.
//! - `oauth_codes` — short-lived authorization codes minted at the
//!   `/oauth/authorize` finalize step and exchanged for an access token
//!   at `/oauth/token`. Stored as a hash (the raw code is one-time and
//!   never persisted).
//!
//! Code expiry is application-checked rather than DB-enforced (libsql HTTP
//! doesn't support background jobs); a periodic cleanup of expired codes
//! is a nice-to-have but not strictly required since the lookup checks
//! `expires_at`.

use chrono::{DateTime, Utc};
use libsql::Connection;

use super::col;
use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct OAuthClient {
    pub client_id: String,
    /// Hex SHA-256 of the secret. `None` for public clients (PKCE-only).
    pub client_secret_hash: Option<String>,
    pub client_name: String,
    /// JSON array of redirect URIs. Stored as TEXT; parsed by the service
    /// layer (which handles validation against the requested redirect_uri).
    pub redirect_uris_json: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OAuthCode {
    pub code_hash: String,
    pub client_id: String,
    pub user_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub scope: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ── oauth_clients ──────────────────────────────────────────────────────

pub async fn insert_client(
    conn: &Connection,
    client_id: &str,
    client_secret_hash: Option<&str>,
    client_name: &str,
    redirect_uris_json: &str,
    created_at: DateTime<Utc>,
) -> Result<(), ApiError> {
    conn.execute(
        "INSERT INTO oauth_clients (client_id, client_secret_hash, client_name, redirect_uris, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        libsql::params![
            client_id,
            client_secret_hash,
            client_name,
            redirect_uris_json,
            created_at.to_rfc3339()
        ],
    )
    .await?;
    Ok(())
}

pub async fn get_client_by_id(
    conn: &Connection,
    client_id: &str,
) -> Result<Option<OAuthClient>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT client_id, client_secret_hash, client_name, redirect_uris, created_at
             FROM oauth_clients WHERE client_id = ?1",
            libsql::params![client_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let client_id: String = col!(row, 0)?;
            let client_secret_hash: Option<String> = col!(row, 1)?;
            let client_name: String = col!(row, 2)?;
            let redirect_uris_json: String = col!(row, 3)?;
            let created_at_str: String = col!(row, 4)?;
            let created_at: DateTime<Utc> = created_at_str
                .parse()
                .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
            Ok(Some(OAuthClient {
                client_id,
                client_secret_hash,
                client_name,
                redirect_uris_json,
                created_at,
            }))
        }
        None => Ok(None),
    }
}

// ── oauth_codes ────────────────────────────────────────────────────────

pub async fn insert_code(conn: &Connection, code: &OAuthCode) -> Result<(), ApiError> {
    conn.execute(
        "INSERT INTO oauth_codes (code_hash, client_id, user_id, redirect_uri, code_challenge, code_challenge_method, scope, expires_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        libsql::params![
            code.code_hash.clone(),
            code.client_id.clone(),
            code.user_id.clone(),
            code.redirect_uri.clone(),
            code.code_challenge.clone(),
            code.code_challenge_method.clone(),
            code.scope.clone(),
            code.expires_at.to_rfc3339(),
            code.created_at.to_rfc3339(),
        ],
    )
    .await?;
    Ok(())
}

/// Read-and-delete by hash. The service layer enforces expiry at the
/// caller — we don't filter on `expires_at` here because the auth code
/// is single-use either way and the caller wants to distinguish
/// "expired" from "unknown" for clearer error messaging.
pub async fn consume_code_by_hash(
    conn: &Connection,
    code_hash: &str,
) -> Result<Option<OAuthCode>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT code_hash, client_id, user_id, redirect_uri, code_challenge, code_challenge_method, scope, expires_at, created_at
             FROM oauth_codes WHERE code_hash = ?1",
            libsql::params![code_hash],
        )
        .await?;

    let row = match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(r) => r,
        None => return Ok(None),
    };

    let code_hash: String = col!(row, 0)?;
    let client_id: String = col!(row, 1)?;
    let user_id: String = col!(row, 2)?;
    let redirect_uri: String = col!(row, 3)?;
    let code_challenge: String = col!(row, 4)?;
    let code_challenge_method: String = col!(row, 5)?;
    let scope: Option<String> = col!(row, 6)?;
    let expires_at_str: String = col!(row, 7)?;
    let created_at_str: String = col!(row, 8)?;
    let expires_at: DateTime<Utc> = expires_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid expires_at: {e}")))?;
    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;

    // Single-use: delete the row immediately on read so a replay attack
    // (intercepted code re-submitted) can't double-spend even if the
    // first exchange is still in flight.
    conn.execute(
        "DELETE FROM oauth_codes WHERE code_hash = ?1",
        libsql::params![code_hash.clone()],
    )
    .await?;

    Ok(Some(OAuthCode {
        code_hash,
        client_id,
        user_id,
        redirect_uri,
        code_challenge,
        code_challenge_method,
        scope,
        expires_at,
        created_at,
    }))
}

/// Best-effort cleanup of expired codes. Not strictly required because
/// `consume_code_by_hash` checks expiry, but keeps the table from
/// growing unbounded if codes are minted but never exchanged.
pub async fn delete_expired_codes(conn: &Connection) -> Result<u64, ApiError> {
    let now = Utc::now().to_rfc3339();
    let n = conn
        .execute(
            "DELETE FROM oauth_codes WHERE expires_at < ?1",
            libsql::params![now],
        )
        .await?;
    Ok(n)
}

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::col;
use crate::error::ApiError;

/// PAT bearer-token prefix. Single source of truth: the auth extractor
/// filters incoming bearers by this prefix, and the service uses it when
/// generating new tokens.
pub const TOKEN_PREFIX: &str = "intrada_pat_";

/// Request payload for `POST /api/account/tokens`.
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
}

/// Response from `POST /api/account/tokens`. The `token` field is the only
/// place the full PAT is ever returned — list/get endpoints expose only the
/// prefix.
#[derive(Debug, Serialize)]
pub struct CreatedTokenResponse {
    pub id: String,
    pub name: String,
    pub token: String,
    pub prefix: String,
    pub created_at: DateTime<Utc>,
}

/// Public list view of an MCP PAT. Never includes the full token or the
/// hash — only the prefix (for visual identification) and metadata.
#[derive(Debug, Serialize)]
pub struct TokenListItem {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Hex-encoded SHA-256 of the full bearer token. PAT bytes have ~256 bits of
/// entropy, so a fast unsalted hash is appropriate (bcrypt/argon2 are for
/// low-entropy passwords).
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    })
}

pub async fn insert(
    conn: &Connection,
    id: &str,
    user_id: &str,
    name: &str,
    hash: &str,
    prefix: &str,
    created_at: DateTime<Utc>,
) -> Result<(), ApiError> {
    conn.execute(
        "INSERT INTO mcp_tokens (id, user_id, name, hash, prefix, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        libsql::params![id, user_id, name, hash, prefix, created_at.to_rfc3339()],
    )
    .await?;
    Ok(())
}

pub async fn list(conn: &Connection, user_id: &str) -> Result<Vec<TokenListItem>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, name, prefix, last_used_at, created_at, revoked_at
             FROM mcp_tokens WHERE user_id = ?1 ORDER BY created_at DESC",
            libsql::params![user_id],
        )
        .await?;

    let mut items = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let id: String = col!(row, 0)?;
        let name: String = col!(row, 1)?;
        let prefix: String = col!(row, 2)?;
        let last_used_at: Option<String> = col!(row, 3)?;
        let created_at_str: String = col!(row, 4)?;
        let revoked_at: Option<String> = col!(row, 5)?;

        items.push(TokenListItem {
            id,
            name,
            prefix,
            last_used_at: parse_dt_opt(last_used_at)?,
            created_at: parse_dt(&created_at_str)?,
            revoked_at: parse_dt_opt(revoked_at)?,
        });
    }
    Ok(items)
}

/// Mark the token as revoked. Returns `true` if a non-revoked row was
/// updated, `false` if no matching row exists or it was already revoked.
pub async fn revoke(conn: &Connection, user_id: &str, token_id: &str) -> Result<bool, ApiError> {
    let now = Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE mcp_tokens SET revoked_at = ?1
             WHERE id = ?2 AND user_id = ?3 AND revoked_at IS NULL",
            libsql::params![now, token_id, user_id],
        )
        .await?;
    Ok(rows > 0)
}

/// Result of a `lookup_by_hash` call. `revoked_at` is returned raw rather
/// than baking the revocation check into the lookup so the auth extractor
/// can log revoked-token use distinctly from missing-token use; and the
/// Phase 4 audit log will need `token_id` to record which token initiated
/// each MCP write.
pub struct PatLookup {
    pub token_id: String,
    pub user_id: String,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn lookup_by_hash(conn: &Connection, hash: &str) -> Result<Option<PatLookup>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, user_id, revoked_at FROM mcp_tokens WHERE hash = ?1",
            libsql::params![hash],
        )
        .await?;
    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let token_id: String = col!(row, 0)?;
            let user_id: String = col!(row, 1)?;
            let revoked_at: Option<String> = col!(row, 2)?;
            Ok(Some(PatLookup {
                token_id,
                user_id,
                revoked_at: parse_dt_opt(revoked_at)?,
            }))
        }
        None => Ok(None),
    }
}

/// Revoke all non-revoked tokens with the given name for a user.
/// Returns the number of tokens revoked.
pub async fn revoke_by_name(conn: &Connection, user_id: &str, name: &str) -> Result<u64, ApiError> {
    let now = Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE mcp_tokens SET revoked_at = ?1
             WHERE user_id = ?2 AND name = ?3 AND revoked_at IS NULL",
            libsql::params![now, user_id, name],
        )
        .await?;
    Ok(rows)
}

/// Update `last_used_at` to now. Best-effort; auth extractor calls this
/// after a successful PAT resolution but ignores errors.
pub async fn mark_used(conn: &Connection, hash: &str) -> Result<(), ApiError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE mcp_tokens SET last_used_at = ?1 WHERE hash = ?2",
        libsql::params![now, hash],
    )
    .await?;
    Ok(())
}

fn parse_dt(s: &str) -> Result<DateTime<Utc>, ApiError> {
    s.parse()
        .map_err(|e| ApiError::Internal(format!("Invalid timestamp {s:?}: {e}")))
}

fn parse_dt_opt(s: Option<String>) -> Result<Option<DateTime<Utc>>, ApiError> {
    s.map(|s| parse_dt(&s)).transpose()
}

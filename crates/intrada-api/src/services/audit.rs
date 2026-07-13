//! Audit recording for MCP write tools.
//!
//! Every successful MCP write by a real user is recorded:
//! - `AuthSource::Pat { token_id }`: row written with the resolved token id.
//! - `AuthSource::Jwt`: row written with `token_id = NULL` (#528). The
//!   browser/iOS session is the auth mechanism; no PAT exists to attribute
//!   the write to, but the write still appears in the user's audit view
//!   labelled "(web app)".
//! - `AuthSource::Disabled`: skipped — dev-mode has no real user id.
//!
//! After migration 0062-0066, `mcp_audit_log.token_id` is nullable,
//! so JWT rows are schema-valid.

use chrono::Utc;
use libsql::Connection;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::auth::AuthSource;
use crate::db;
use crate::error::ApiError;

/// Hex SHA-256 of the JSON-stringified arguments. Stored to prove "this
/// agent invoked tool X with args Y" without persisting the args
/// themselves.
pub fn args_hash(args: &Value) -> String {
    let json = serde_json::to_string(args).unwrap_or_default();
    let digest = Sha256::digest(json.as_bytes());
    digest.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Record a successful MCP write. Called after every successful single-write
/// tool call. No-op for `AuthSource::Disabled` (local dev — no real user).
///
/// Errors are logged but not propagated — a failed audit row must not roll
/// back a write the user already saw succeed.
pub async fn record_mcp_write(
    conn: &Connection,
    source: &AuthSource,
    user_id: &str,
    tool: &str,
    args: &Value,
) {
    // Resolve the token_id (None for JWT-authenticated calls).
    let token_id: Option<String> = match source {
        AuthSource::Pat { token_id } => Some(token_id.clone()),
        AuthSource::Jwt => None,
        // Disabled = local dev with no real user — skip entirely.
        AuthSource::Disabled => return,
    };

    let id = ulid::Ulid::gen().to_string();
    let hash = args_hash(args);
    if let Err(e) = db::audit::insert(
        conn,
        &id,
        token_id.as_deref(),
        user_id,
        tool,
        &hash,
        Utc::now(),
    )
    .await
    {
        tracing::warn!(?e, tool, "audit log write failed; continuing");
    }
}

pub async fn list_audit(
    conn: &Connection,
    user_id: &str,
    limit: u32,
) -> Result<Vec<db::audit::AuditLogEntry>, ApiError> {
    db::audit::list(conn, user_id, limit).await
}

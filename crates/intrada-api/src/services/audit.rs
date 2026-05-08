//! Audit recording for MCP write tools.
//!
//! Only `AuthSource::Pat { token_id }` writes are recorded — the audit
//! row's `token_id` column is non-nullable and the table's purpose is
//! to attribute MCP-issued mutations to a specific PAT.
//!
//! - **Jwt**: writes via `/api/mcp` from a Clerk-authenticated browser
//!   session don't go through the audit log; the user's regular session
//!   activity is the audit trail.
//! - **Disabled**: dev-mode writes have no token to attribute to.
//!
//! See `auth.rs::AuthSource::Disabled` for the `// TODO(#477 phase 4)`
//! marker that originally signposted this contract.

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

/// Record a successful PAT write. No-op for non-PAT auth sources.
/// Errors are logged but not propagated — failing to record an audit row
/// shouldn't break a successful write that the user already saw succeed.
pub async fn record_pat_write(
    conn: &Connection,
    source: &AuthSource,
    user_id: &str,
    tool: &str,
    args: &Value,
) {
    let token_id = match source {
        AuthSource::Pat { token_id } => token_id.clone(),
        AuthSource::Jwt | AuthSource::Disabled => return,
    };

    let id = ulid::Ulid::new().to_string();
    let hash = args_hash(args);
    if let Err(e) = db::audit::insert(conn, &id, &token_id, user_id, tool, &hash, Utc::now()).await
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

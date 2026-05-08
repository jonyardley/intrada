//! `mcp_audit_log` table operations.
//!
//! One row per successful MCP write. Visible to the user in account
//! settings (Phase 4b). Stores `args_hash` (not the args themselves) —
//! the audit trail proves *what* tool was called by *which* token at
//! *what* time, without leaking the contents of the user's library
//! into a more sensitive table.

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::Serialize;

use super::col;
use crate::error::ApiError;

/// Public list view of an audit-log row.
#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub token_id: String,
    pub tool: String,
    pub args_hash: String,
    pub created_at: DateTime<Utc>,
}

pub async fn insert(
    conn: &Connection,
    id: &str,
    token_id: &str,
    user_id: &str,
    tool: &str,
    args_hash: &str,
    created_at: DateTime<Utc>,
) -> Result<(), ApiError> {
    conn.execute(
        "INSERT INTO mcp_audit_log (id, token_id, user_id, tool, args_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        libsql::params![
            id,
            token_id,
            user_id,
            tool,
            args_hash,
            created_at.to_rfc3339()
        ],
    )
    .await?;
    Ok(())
}

/// Newest-first listing for the audit-log view. `limit` is the page size;
/// callers cap it (we hard-cap at the SQL layer too as a defence).
pub async fn list(
    conn: &Connection,
    user_id: &str,
    limit: u32,
) -> Result<Vec<AuditLogEntry>, ApiError> {
    let limit = limit.min(500);
    let mut rows = conn
        .query(
            "SELECT id, token_id, tool, args_hash, created_at
             FROM mcp_audit_log
             WHERE user_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
            libsql::params![user_id, limit as i64],
        )
        .await?;

    let mut entries = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let id: String = col!(row, 0)?;
        let token_id: String = col!(row, 1)?;
        let tool: String = col!(row, 2)?;
        let args_hash: String = col!(row, 3)?;
        let created_at_str: String = col!(row, 4)?;
        let created_at: DateTime<Utc> = created_at_str
            .parse()
            .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
        entries.push(AuditLogEntry {
            id,
            token_id,
            tool,
            args_hash,
            created_at,
        });
    }
    Ok(entries)
}

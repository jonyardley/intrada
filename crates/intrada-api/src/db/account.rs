use chrono::Utc;
use libsql::Connection;

use super::col;
use crate::error::ApiError;

/// User preferences read by the client and surfaced in the Settings sheet.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AccountPreferences {
    pub default_focus_minutes: u32,
    pub default_rep_count: u32,
}

/// Fallback when no row exists yet.
pub const DEFAULT_FOCUS_MINUTES: u32 = 15;
pub const DEFAULT_REP_COUNT: u32 = 10;

pub async fn get_preferences(
    conn: &Connection,
    user_id: &str,
) -> Result<AccountPreferences, ApiError> {
    let mut rows = conn
        .query(
            "SELECT default_focus_minutes, default_rep_count FROM user_preferences WHERE user_id = ?1",
            libsql::params![user_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let focus: Option<i64> = col!(row, 0)?;
            let reps: Option<i64> = col!(row, 1)?;
            Ok(AccountPreferences {
                default_focus_minutes: focus.map(|v| v as u32).unwrap_or(DEFAULT_FOCUS_MINUTES),
                default_rep_count: reps.map(|v| v as u32).unwrap_or(DEFAULT_REP_COUNT),
            })
        }
        None => Ok(AccountPreferences {
            default_focus_minutes: DEFAULT_FOCUS_MINUTES,
            default_rep_count: DEFAULT_REP_COUNT,
        }),
    }
}

pub async fn upsert_preferences(
    conn: &Connection,
    user_id: &str,
    prefs: &AccountPreferences,
) -> Result<AccountPreferences, ApiError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO user_preferences (user_id, default_focus_minutes, default_rep_count, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(user_id) DO UPDATE SET
             default_focus_minutes = excluded.default_focus_minutes,
             default_rep_count = excluded.default_rep_count,
             updated_at = excluded.updated_at",
        libsql::params![
            user_id,
            prefs.default_focus_minutes as i64,
            prefs.default_rep_count as i64,
            now.as_str(),
        ],
    )
    .await?;
    Ok(prefs.clone())
}

/// Delete every user-scoped row across the schema.
///
/// Sequential statements (no transaction) — Turso's HTTP layer doesn't
/// reliably support multi-statement transactions across the same
/// connection, and child tables already follow this pattern in
/// `delete_session` / `delete_set` / `delete_lesson`. The DELETE endpoint
/// is idempotent, so a partial failure can be retried safely.
pub async fn delete_all_user_data(conn: &Connection, user_id: &str) -> Result<(), ApiError> {
    // Child tables first (joined via parent's user_id) so we don't orphan
    // them if the parent delete races.
    conn.execute(
        "DELETE FROM setlist_entries WHERE session_id IN (SELECT id FROM sessions WHERE user_id = ?1)",
        libsql::params![user_id],
    )
    .await?;
    conn.execute(
        "DELETE FROM routine_entries WHERE routine_id IN (SELECT id FROM routines WHERE user_id = ?1)",
        libsql::params![user_id],
    )
    .await?;

    for sql in [
        "DELETE FROM lesson_photos WHERE user_id = ?1",
        "DELETE FROM lessons WHERE user_id = ?1",
        "DELETE FROM sessions WHERE user_id = ?1",
        "DELETE FROM items WHERE user_id = ?1",
        "DELETE FROM routines WHERE user_id = ?1",
        "DELETE FROM user_preferences WHERE user_id = ?1",
    ] {
        conn.execute(sql, libsql::params![user_id]).await?;
    }

    Ok(())
}

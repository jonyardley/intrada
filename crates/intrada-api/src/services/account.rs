use libsql::Connection;

use crate::clerk::ClerkClient;
use crate::db;
use crate::db::account::AccountPreferences;
use crate::error::ApiError;
use crate::storage::R2Client;

pub async fn get_preferences(
    conn: &Connection,
    user_id: &str,
) -> Result<AccountPreferences, ApiError> {
    db::account::get_preferences(conn, user_id).await
}

pub async fn put_preferences(
    conn: &Connection,
    user_id: &str,
    input: &AccountPreferences,
) -> Result<AccountPreferences, ApiError> {
    if input.default_focus_minutes == 0 || input.default_focus_minutes > 600 {
        return Err(ApiError::Validation(
            "default_focus_minutes must be 1..=600".to_string(),
        ));
    }
    if input.default_rep_count == 0 || input.default_rep_count > 999 {
        return Err(ApiError::Validation(
            "default_rep_count must be 1..=999".to_string(),
        ));
    }
    db::account::upsert_preferences(conn, user_id, input).await
}

pub async fn delete_account(
    conn: &Connection,
    r2: Option<&R2Client>,
    clerk: Option<&ClerkClient>,
    user_id: &str,
) -> Result<(), ApiError> {
    // Refuse empty user_id outright — auth-disabled mode (no
    // CLERK_ISSUER_URL) yields `AuthUser("")`, which would otherwise
    // turn into an R2 prefix `/` and a Clerk DELETE /v1/users/ —
    // both blast-radius hazards.
    if user_id.is_empty() {
        return Err(ApiError::Unauthorized("Unauthorized".to_string()));
    }

    // 1. DB rows. Hard fail — if data delete doesn't succeed, the user
    //    can re-run.
    db::account::delete_all_user_data(conn, user_id).await?;

    // 2. R2 photo blobs. Best-effort: log but don't fail. The DB
    //    `lesson_photos` rows are already gone, so the blobs are
    //    orphaned-but-private (keys include user_id; bucket has no public
    //    listing).
    if let Some(r2) = r2 {
        if let Err(err) = r2.delete_user_photos(user_id).await {
            tracing::warn!(?err, %user_id, "R2 photo cleanup failed during account delete");
        }
    }

    // 3. Clerk user record. Best-effort: log but don't fail. 404 from
    //    Clerk is treated as success (idempotent retry).
    if let Some(clerk) = clerk {
        if let Err(err) = clerk.delete_user(user_id).await {
            tracing::warn!(?err, %user_id, "Clerk user delete failed during account delete");
        }
    }

    Ok(())
}

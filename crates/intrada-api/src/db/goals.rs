use chrono::{DateTime, Utc};
use libsql::Connection;

use intrada_core::domain::goal::{Goal, GoalItem, GoalPhoto, GoalStatus};
use intrada_core::domain::item::ItemKind;
use intrada_core::domain::types::{CreateGoal, UpdateGoal, UpdateGoalItem};

use super::{col, item_kind_from_str, item_kind_to_str};
use crate::error::ApiError;
use crate::storage::R2Client;

const SELECT_COLUMNS: &str =
    "id, user_id, title, date, notes, deadline, status, completed_at, created_at, updated_at, target_confidence";

const PHOTO_SELECT_COLUMNS: &str = "id, goal_id, user_id, storage_key, created_at";

const GOAL_ITEMS_SELECT_COLUMNS: &str =
    "goal_id, item_id, item_title, item_type, target_date, target_confidence";

fn goal_status_from_str(s: &str) -> GoalStatus {
    match s {
        "completed" => GoalStatus::Completed,
        _ => GoalStatus::Active,
    }
}

fn goal_status_to_str(status: &GoalStatus) -> &'static str {
    match status {
        GoalStatus::Active => "active",
        GoalStatus::Completed => "completed",
    }
}

fn row_to_goal(
    row: &libsql::Row,
    items: Vec<GoalItem>,
    photos: Vec<GoalPhoto>,
) -> Result<Goal, ApiError> {
    let id: String = col!(row, 0)?;
    let _user_id: String = col!(row, 1)?;
    let title: Option<String> = col!(row, 2)?;
    let date: String = col!(row, 3)?;
    let notes: Option<String> = col!(row, 4)?;
    let deadline: Option<String> = col!(row, 5)?;
    let status_str: String = col!(row, 6)?;
    let completed_at_str: Option<String> = col!(row, 7)?;
    let created_at_str: String = col!(row, 8)?;
    let updated_at_str: String = col!(row, 9)?;
    let target_confidence: Option<i64> = col!(row, 10)?;
    let target_confidence: Option<u8> = target_confidence.map(|n| n as u8);

    let status = goal_status_from_str(&status_str);

    let completed_at: Option<DateTime<Utc>> = completed_at_str
        .map(|s| {
            s.parse()
                .map_err(|e| ApiError::Internal(format!("Invalid completed_at: {e}")))
        })
        .transpose()?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
    let updated_at: DateTime<Utc> = updated_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid updated_at: {e}")))?;

    Ok(Goal {
        id,
        title,
        date,
        notes,
        deadline,
        status,
        completed_at,
        items,
        photos,
        created_at,
        updated_at,
        target_confidence,
    })
}

fn row_to_photo(row: &libsql::Row, r2: &R2Client) -> Result<GoalPhoto, ApiError> {
    let id: String = col!(row, 0)?;
    let _goal_id: String = col!(row, 1)?;
    let _user_id: String = col!(row, 2)?;
    let storage_key: String = col!(row, 3)?;
    let created_at_str: String = col!(row, 4)?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;

    let url = r2.public_url(&storage_key);

    Ok(GoalPhoto {
        id,
        url,
        created_at,
    })
}

fn row_to_goal_item(row: &libsql::Row) -> Result<GoalItem, ApiError> {
    let _goal_id: String = col!(row, 0)?;
    let item_id: String = col!(row, 1)?;
    let item_title: String = col!(row, 2)?;
    let item_type_str: String = col!(row, 3)?;
    let item_type: ItemKind = item_kind_from_str(&item_type_str)?;
    let target_date: Option<String> = col!(row, 4)?;
    let target_confidence: Option<i64> = col!(row, 5)?;
    let target_confidence: Option<u8> = target_confidence.map(|n| n as u8);

    Ok(GoalItem {
        item_id,
        item_title,
        item_type,
        target_date,
        target_confidence,
    })
}

async fn list_photos_for_goal(
    conn: &Connection,
    goal_id: &str,
    r2: Option<&R2Client>,
) -> Result<Vec<GoalPhoto>, ApiError> {
    let r2 = match r2 {
        Some(r2) => r2,
        None => return Ok(Vec::new()),
    };

    let mut rows = conn
        .query(
            &format!(
                "SELECT {PHOTO_SELECT_COLUMNS} FROM goal_photos WHERE goal_id = ?1 ORDER BY created_at ASC"
            ),
            libsql::params![goal_id],
        )
        .await?;

    let mut photos = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        photos.push(row_to_photo(&row, r2)?);
    }
    Ok(photos)
}

pub async fn list_items_for_goal(
    conn: &Connection,
    goal_id: &str,
) -> Result<Vec<GoalItem>, ApiError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {GOAL_ITEMS_SELECT_COLUMNS} FROM goal_items WHERE goal_id = ?1 ORDER BY item_title ASC"
            ),
            libsql::params![goal_id],
        )
        .await?;

    let mut items = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        items.push(row_to_goal_item(&row)?);
    }
    Ok(items)
}

pub async fn list_goals(
    conn: &Connection,
    user_id: &str,
    status_filter: Option<&str>,
    r2: Option<&R2Client>,
) -> Result<Vec<Goal>, ApiError> {
    let filter = status_filter.unwrap_or("active");

    let sql = match filter {
        "completed" => format!(
            "SELECT {SELECT_COLUMNS} FROM goals WHERE user_id = ?1 AND status = 'completed' ORDER BY completed_at DESC"
        ),
        "all" => format!(
            "SELECT {SELECT_COLUMNS} FROM goals WHERE user_id = ?1 ORDER BY CASE WHEN deadline IS NULL THEN 1 ELSE 0 END, deadline ASC, created_at DESC"
        ),
        // "active" and anything else
        _ => format!(
            "SELECT {SELECT_COLUMNS} FROM goals WHERE user_id = ?1 AND status = 'active' ORDER BY CASE WHEN deadline IS NULL THEN 1 ELSE 0 END, deadline ASC, created_at DESC"
        ),
    };

    let mut rows = conn.query(&sql, libsql::params![user_id]).await?;

    let mut goals = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let id: String = col!(row, 0)?;
        let items = list_items_for_goal(conn, &id).await?;
        let photos = list_photos_for_goal(conn, &id, r2).await?;
        goals.push(row_to_goal(&row, items, photos)?);
    }
    Ok(goals)
}

pub async fn get_goal(
    conn: &Connection,
    id: &str,
    user_id: &str,
    r2: Option<&R2Client>,
) -> Result<Option<Goal>, ApiError> {
    let mut rows = conn
        .query(
            &format!("SELECT {SELECT_COLUMNS} FROM goals WHERE id = ?1 AND user_id = ?2"),
            libsql::params![id, user_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let items = list_items_for_goal(conn, id).await?;
            let photos = list_photos_for_goal(conn, id, r2).await?;
            Ok(Some(row_to_goal(&row, items, photos)?))
        }
        None => Ok(None),
    }
}

pub async fn insert_goal(
    conn: &Connection,
    user_id: &str,
    input: &CreateGoal,
    r2: Option<&R2Client>,
) -> Result<Goal, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let target_confidence_val: Option<i64> = input.target_confidence.map(|c| c as i64);
    conn.execute(
        "INSERT INTO goals (id, user_id, title, date, notes, deadline, status, completed_at, created_at, updated_at, target_confidence) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        libsql::params![
            id.as_str(),
            user_id,
            input.title.as_deref(),
            input.date.as_str(),
            input.notes.as_deref(),
            input.deadline.as_deref(),
            "active",
            Option::<String>::None,
            now_str.as_str(),
            now_str.as_str(),
            target_confidence_val
        ],
    )
    .await?;

    get_goal(conn, &id, user_id, r2)
        .await?
        .ok_or_else(|| ApiError::Internal("Failed to read back inserted goal".into()))
}

pub async fn update_goal(
    conn: &Connection,
    id: &str,
    user_id: &str,
    input: &UpdateGoal,
    r2: Option<&R2Client>,
) -> Result<Option<Goal>, ApiError> {
    let current = match get_goal(conn, id, user_id, r2).await? {
        Some(g) => g,
        None => return Ok(None),
    };

    let title: Option<&str> = match &input.title {
        None => current.title.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let date = input.date.as_deref().unwrap_or(&current.date);

    let notes: Option<&str> = match &input.notes {
        None => current.notes.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let deadline: Option<&str> = match &input.deadline {
        None => current.deadline.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let status = input.status.as_ref().unwrap_or(&current.status);
    let status_str = goal_status_to_str(status);

    // When status changes to "completed", set completed_at to now if not already set.
    let completed_at_str: Option<String> = match status {
        GoalStatus::Completed => {
            let existing = current.completed_at.map(|dt| dt.to_rfc3339());
            Some(existing.unwrap_or_else(|| Utc::now().to_rfc3339()))
        }
        GoalStatus::Active => None,
    };

    let target_confidence: Option<i64> = match &input.target_confidence {
        None => current.target_confidence.map(|c| c as i64),
        Some(opt) => opt.map(|c| c as i64),
    };

    let now = Utc::now();
    let now_str = now.to_rfc3339();

    conn.execute(
        "UPDATE goals SET title = ?1, date = ?2, notes = ?3, deadline = ?4, status = ?5, completed_at = ?6, updated_at = ?7, target_confidence = ?8 WHERE id = ?9 AND user_id = ?10",
        libsql::params![title, date, notes, deadline, status_str, completed_at_str.as_deref(), now_str.as_str(), target_confidence, id, user_id],
    )
    .await?;

    get_goal(conn, id, user_id, r2).await
}

pub async fn delete_goal(conn: &Connection, id: &str, user_id: &str) -> Result<bool, ApiError> {
    // Explicit child-row delete — FK cascade is disabled (Turso compatibility),
    // so application code owns the cleanup.
    conn.execute(
        "DELETE FROM goal_photos WHERE goal_id = ?1 AND user_id = ?2",
        libsql::params![id, user_id],
    )
    .await?;

    conn.execute(
        "DELETE FROM goal_items WHERE goal_id = ?1 AND user_id = ?2",
        libsql::params![id, user_id],
    )
    .await?;

    let rows_affected = conn
        .execute(
            "DELETE FROM goals WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    Ok(rows_affected > 0)
}

// ── Photo DB operations ────────────────────────────────────────────────

pub async fn insert_goal_photo(
    conn: &Connection,
    goal_id: &str,
    user_id: &str,
    storage_key: &str,
    r2: &R2Client,
) -> Result<GoalPhoto, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    // Plain INSERT — no goal existence check. Turso's remote HTTP
    // connections have cross-connection read-after-write inconsistency
    // (and with multi-machine Fly.io, cross-machine too). Any read of
    // the goals table — SELECT, INSERT...SELECT WHERE EXISTS, or FK
    // constraint check — can fail to see a just-created goal.
    //
    // Security: user_id comes from the JWT and is stored on the photo
    // row, so ownership is enforced at read time.
    conn.execute(
        "INSERT INTO goal_photos (id, goal_id, user_id, storage_key, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        libsql::params![
            id.as_str(),
            goal_id,
            user_id,
            storage_key,
            now_str.as_str()
        ],
    )
    .await?;

    Ok(GoalPhoto {
        id,
        url: r2.public_url(storage_key),
        created_at: now,
    })
}

pub async fn get_goal_photo_storage_key(
    conn: &Connection,
    photo_id: &str,
    goal_id: &str,
    user_id: &str,
) -> Result<Option<String>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT storage_key FROM goal_photos WHERE id = ?1 AND goal_id = ?2 AND user_id = ?3",
            libsql::params![photo_id, goal_id, user_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let key: String = col!(row, 0)?;
            Ok(Some(key))
        }
        None => Ok(None),
    }
}

pub async fn delete_goal_photo(
    conn: &Connection,
    photo_id: &str,
    goal_id: &str,
    user_id: &str,
) -> Result<bool, ApiError> {
    let rows_affected = conn
        .execute(
            "DELETE FROM goal_photos WHERE id = ?1 AND goal_id = ?2 AND user_id = ?3",
            libsql::params![photo_id, goal_id, user_id],
        )
        .await?;

    Ok(rows_affected > 0)
}

pub async fn list_photo_storage_keys(
    conn: &Connection,
    goal_id: &str,
    user_id: &str,
) -> Result<Vec<String>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT storage_key FROM goal_photos WHERE goal_id = ?1 AND user_id = ?2",
            libsql::params![goal_id, user_id],
        )
        .await?;

    let mut keys = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let key: String = col!(row, 0)?;
        keys.push(key);
    }
    Ok(keys)
}

// ── Goal item DB operations ────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn insert_goal_item(
    conn: &Connection,
    goal_id: &str,
    user_id: &str,
    item_id: &str,
    item_title: &str,
    item_type: &ItemKind,
    target_date: Option<&str>,
    target_confidence: Option<u8>,
) -> Result<(), ApiError> {
    let now = chrono::Utc::now().to_rfc3339();
    let target_confidence_val: Option<i64> = target_confidence.map(|c| c as i64);
    conn.execute(
        "INSERT OR IGNORE INTO goal_items (goal_id, item_id, user_id, item_title, item_type, created_at, target_date, target_confidence) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        libsql::params![
            goal_id,
            item_id,
            user_id,
            item_title,
            item_kind_to_str(item_type),
            now,
            target_date,
            target_confidence_val
        ],
    )
    .await?;
    Ok(())
}

pub async fn update_goal_item(
    conn: &Connection,
    goal_id: &str,
    item_id: &str,
    user_id: &str,
    input: &UpdateGoalItem,
) -> Result<bool, ApiError> {
    let current = list_items_for_goal(conn, goal_id)
        .await?
        .into_iter()
        .find(|i| i.item_id == item_id);
    let Some(current) = current else {
        return Ok(false);
    };

    let target_date: Option<String> = match &input.target_date {
        None => current.target_date.clone(),
        Some(opt) => opt.clone(),
    };
    let target_confidence: Option<i64> = match &input.target_confidence {
        None => current.target_confidence.map(|c| c as i64),
        Some(opt) => opt.map(|c| c as i64),
    };

    let rows_affected = conn
        .execute(
            "UPDATE goal_items SET target_date = ?1, target_confidence = ?2 WHERE goal_id = ?3 AND item_id = ?4 AND user_id = ?5",
            libsql::params![target_date, target_confidence, goal_id, item_id, user_id],
        )
        .await?;
    Ok(rows_affected > 0)
}

pub async fn delete_goal_item(
    conn: &Connection,
    goal_id: &str,
    item_id: &str,
    user_id: &str,
) -> Result<bool, ApiError> {
    let rows_affected = conn
        .execute(
            "DELETE FROM goal_items WHERE goal_id = ?1 AND item_id = ?2 AND user_id = ?3",
            libsql::params![goal_id, item_id, user_id],
        )
        .await?;

    Ok(rows_affected > 0)
}

use chrono::{DateTime, Utc};
use libsql::Connection;

use intrada_core::domain::lesson::{Lesson, LessonPhoto};
use intrada_core::domain::types::{CreateLesson, UpdateLesson};

use super::col;
use crate::error::ApiError;
use crate::storage::R2Client;

const SELECT_COLUMNS: &str = "id, user_id, date, notes, created_at, updated_at";

const PHOTO_SELECT_COLUMNS: &str = "id, lesson_id, user_id, storage_key, created_at";

fn row_to_lesson(row: &libsql::Row, photos: Vec<LessonPhoto>) -> Result<Lesson, ApiError> {
    let id: String = col!(row, 0)?;
    let _user_id: String = col!(row, 1)?;
    let date: String = col!(row, 2)?;
    let notes: Option<String> = col!(row, 3)?;
    let created_at_str: String = col!(row, 4)?;
    let updated_at_str: String = col!(row, 5)?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
    let updated_at: DateTime<Utc> = updated_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid updated_at: {e}")))?;

    Ok(Lesson {
        id,
        date,
        notes,
        photos,
        created_at,
        updated_at,
    })
}

fn row_to_photo(row: &libsql::Row, r2: &R2Client) -> Result<LessonPhoto, ApiError> {
    let id: String = col!(row, 0)?;
    let _lesson_id: String = col!(row, 1)?;
    let _user_id: String = col!(row, 2)?;
    let storage_key: String = col!(row, 3)?;
    let created_at_str: String = col!(row, 4)?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;

    let url = r2.public_url(&storage_key);

    Ok(LessonPhoto {
        id,
        url,
        created_at,
    })
}

async fn list_photos_for_lesson(
    conn: &Connection,
    lesson_id: &str,
    r2: &R2Client,
) -> Result<Vec<LessonPhoto>, ApiError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {PHOTO_SELECT_COLUMNS} FROM lesson_photos WHERE lesson_id = ?1 ORDER BY created_at ASC"
            ),
            libsql::params![lesson_id],
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

pub async fn list_lessons(
    conn: &Connection,
    user_id: &str,
    r2: &R2Client,
) -> Result<Vec<Lesson>, ApiError> {
    let mut rows = conn
        .query(
            &format!("SELECT {SELECT_COLUMNS} FROM lessons WHERE user_id = ?1 ORDER BY date DESC"),
            libsql::params![user_id],
        )
        .await?;

    let mut lessons = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let id: String = col!(row, 0)?;
        let photos = list_photos_for_lesson(conn, &id, r2).await?;
        lessons.push(row_to_lesson(&row, photos)?);
    }
    Ok(lessons)
}

pub async fn get_lesson(
    conn: &Connection,
    id: &str,
    user_id: &str,
    r2: &R2Client,
) -> Result<Option<Lesson>, ApiError> {
    let mut rows = conn
        .query(
            &format!("SELECT {SELECT_COLUMNS} FROM lessons WHERE id = ?1 AND user_id = ?2"),
            libsql::params![id, user_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let photos = list_photos_for_lesson(conn, id, r2).await?;
            Ok(Some(row_to_lesson(&row, photos)?))
        }
        None => Ok(None),
    }
}

pub async fn insert_lesson(
    conn: &Connection,
    user_id: &str,
    input: &CreateLesson,
    r2: &R2Client,
) -> Result<Lesson, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    conn.execute(
        "INSERT INTO lessons (id, user_id, date, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        libsql::params![
            id.as_str(),
            user_id,
            input.date.as_str(),
            input.notes.as_deref(),
            now_str.as_str(),
            now_str.as_str()
        ],
    )
    .await?;

    get_lesson(conn, &id, user_id, r2)
        .await?
        .ok_or_else(|| ApiError::Internal("Failed to read back inserted lesson".into()))
}

pub async fn update_lesson(
    conn: &Connection,
    id: &str,
    user_id: &str,
    input: &UpdateLesson,
    r2: &R2Client,
) -> Result<Option<Lesson>, ApiError> {
    let current = match get_lesson(conn, id, user_id, r2).await? {
        Some(l) => l,
        None => return Ok(None),
    };

    let date = input.date.as_deref().unwrap_or(&current.date);

    let notes: Option<&str> = match &input.notes {
        None => current.notes.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let now = Utc::now();
    let now_str = now.to_rfc3339();

    conn.execute(
        "UPDATE lessons SET date = ?1, notes = ?2, updated_at = ?3 WHERE id = ?4 AND user_id = ?5",
        libsql::params![date, notes, now_str.as_str(), id, user_id],
    )
    .await?;

    get_lesson(conn, id, user_id, r2).await
}

pub async fn delete_lesson(conn: &Connection, id: &str, user_id: &str) -> Result<bool, ApiError> {
    // Explicit child-row delete — FK cascade is disabled (Turso compatibility),
    // so application code owns the cleanup.
    conn.execute(
        "DELETE FROM lesson_photos WHERE lesson_id = ?1 AND user_id = ?2",
        libsql::params![id, user_id],
    )
    .await?;

    let rows_affected = conn
        .execute(
            "DELETE FROM lessons WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    Ok(rows_affected > 0)
}

// ── Photo DB operations ────────────────────────────────────────────────

pub async fn insert_lesson_photo(
    conn: &Connection,
    lesson_id: &str,
    user_id: &str,
    storage_key: &str,
    r2: &R2Client,
) -> Result<LessonPhoto, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    // Plain INSERT — no lesson existence check. Turso's remote HTTP
    // connections have cross-connection read-after-write inconsistency
    // (and with multi-machine Fly.io, cross-machine too). Any read of
    // the lessons table — SELECT, INSERT...SELECT WHERE EXISTS, or FK
    // constraint check — can fail to see a just-created lesson.
    //
    // Security: user_id comes from the JWT and is stored on the photo
    // row, so ownership is enforced at read time.
    conn.execute(
        "INSERT INTO lesson_photos (id, lesson_id, user_id, storage_key, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        libsql::params![
            id.as_str(),
            lesson_id,
            user_id,
            storage_key,
            now_str.as_str()
        ],
    )
    .await?;

    Ok(LessonPhoto {
        id,
        url: r2.public_url(storage_key),
        created_at: now,
    })
}

pub async fn get_lesson_photo_storage_key(
    conn: &Connection,
    photo_id: &str,
    lesson_id: &str,
    user_id: &str,
) -> Result<Option<String>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT storage_key FROM lesson_photos WHERE id = ?1 AND lesson_id = ?2 AND user_id = ?3",
            libsql::params![photo_id, lesson_id, user_id],
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

pub async fn delete_lesson_photo(
    conn: &Connection,
    photo_id: &str,
    lesson_id: &str,
    user_id: &str,
) -> Result<bool, ApiError> {
    let rows_affected = conn
        .execute(
            "DELETE FROM lesson_photos WHERE id = ?1 AND lesson_id = ?2 AND user_id = ?3",
            libsql::params![photo_id, lesson_id, user_id],
        )
        .await?;

    Ok(rows_affected > 0)
}

pub async fn list_photo_storage_keys(
    conn: &Connection,
    lesson_id: &str,
    user_id: &str,
) -> Result<Vec<String>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT storage_key FROM lesson_photos WHERE lesson_id = ?1 AND user_id = ?2",
            libsql::params![lesson_id, user_id],
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

use chrono::{DateTime, Utc};
use libsql::Connection;

use intrada_core::domain::exercise::Exercise;
use intrada_core::domain::types::{CreateExercise, Tempo, UpdateExercise};

use crate::error::ApiError;

/// Helper: reconstruct Tempo from flattened columns.
fn tempo_from_row(marking: Option<String>, bpm: Option<i64>) -> Option<Tempo> {
    Tempo::from_parts(marking, bpm.map(|v| v as u16))
}

/// Helper: serialize tags Vec to JSON string for storage.
fn tags_to_json(tags: &[String]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
}

/// Helper: deserialize tags from JSON string.
fn tags_from_json(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}

/// Helper: parse a row from the exercises table into an Exercise.
fn row_to_exercise(row: &libsql::Row) -> Result<Exercise, ApiError> {
    let id: String = row.get(0).map_err(|e| ApiError::Internal(e.to_string()))?;
    let title: String = row.get(1).map_err(|e| ApiError::Internal(e.to_string()))?;
    let composer: Option<String> = row.get(2).map_err(|e| ApiError::Internal(e.to_string()))?;
    let category: Option<String> = row.get(3).map_err(|e| ApiError::Internal(e.to_string()))?;
    let key: Option<String> = row.get(4).map_err(|e| ApiError::Internal(e.to_string()))?;
    let tempo_marking: Option<String> =
        row.get(5).map_err(|e| ApiError::Internal(e.to_string()))?;
    let tempo_bpm: Option<i64> = row.get(6).map_err(|e| ApiError::Internal(e.to_string()))?;
    let notes: Option<String> = row.get(7).map_err(|e| ApiError::Internal(e.to_string()))?;
    let tags_json: String = row.get(8).map_err(|e| ApiError::Internal(e.to_string()))?;
    let created_at_str: String = row.get(9).map_err(|e| ApiError::Internal(e.to_string()))?;
    let updated_at_str: String = row.get(10).map_err(|e| ApiError::Internal(e.to_string()))?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
    let updated_at: DateTime<Utc> = updated_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid updated_at: {e}")))?;

    Ok(Exercise {
        id,
        title,
        composer,
        category,
        key,
        tempo: tempo_from_row(tempo_marking, tempo_bpm),
        notes,
        tags: tags_from_json(&tags_json),
        created_at,
        updated_at,
    })
}

const SELECT_COLUMNS: &str =
    "id, title, composer, category, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at";

pub async fn list_exercises(conn: &Connection) -> Result<Vec<Exercise>, ApiError> {
    let mut rows = conn
        .query(
            &format!("SELECT {SELECT_COLUMNS} FROM exercises ORDER BY created_at DESC"),
            (),
        )
        .await?;

    let mut exercises = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        exercises.push(row_to_exercise(&row)?);
    }
    Ok(exercises)
}

pub async fn get_exercise(conn: &Connection, id: &str) -> Result<Option<Exercise>, ApiError> {
    let mut rows = conn
        .query(
            &format!("SELECT {SELECT_COLUMNS} FROM exercises WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => Ok(Some(row_to_exercise(&row)?)),
        None => Ok(None),
    }
}

pub async fn insert_exercise(
    conn: &Connection,
    input: &CreateExercise,
) -> Result<Exercise, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let (tempo_marking, tempo_bpm) = match &input.tempo {
        Some(t) => (t.marking.clone(), t.bpm.map(|b| b as i64)),
        None => (None, None),
    };

    let tags_json = tags_to_json(&input.tags);

    conn.execute(
        "INSERT INTO exercises (id, title, composer, category, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        libsql::params![
            id.as_str(),
            input.title.as_str(),
            input.composer.as_deref(),
            input.category.as_deref(),
            input.key.as_deref(),
            tempo_marking.as_deref(),
            tempo_bpm,
            input.notes.as_deref(),
            tags_json.as_str(),
            now_str.as_str(),
            now_str.as_str()
        ],
    )
    .await?;

    Ok(Exercise {
        id,
        title: input.title.clone(),
        composer: input.composer.clone(),
        category: input.category.clone(),
        key: input.key.clone(),
        tempo: input.tempo.clone(),
        notes: input.notes.clone(),
        tags: input.tags.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_exercise(
    conn: &Connection,
    id: &str,
    input: &UpdateExercise,
) -> Result<Option<Exercise>, ApiError> {
    // Fetch current row
    let current = match get_exercise(conn, id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    // Apply three-state update semantics
    let title = input.title.as_ref().unwrap_or(&current.title);

    let composer = match &input.composer {
        None => current.composer.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let category = match &input.category {
        None => current.category.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let key = match &input.key {
        None => current.key.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let tempo = match &input.tempo {
        None => current.tempo.clone(),
        Some(opt) => opt.clone(),
    };

    let notes = match &input.notes {
        None => current.notes.as_deref(),
        Some(opt) => opt.as_deref(),
    };

    let tags = input.tags.as_ref().unwrap_or(&current.tags);

    let (tempo_marking, tempo_bpm) = match &tempo {
        Some(t) => (t.marking.clone(), t.bpm.map(|b| b as i64)),
        None => (None, None),
    };

    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let tags_json = tags_to_json(tags);

    conn.execute(
        "UPDATE exercises SET title = ?1, composer = ?2, category = ?3, key_signature = ?4, tempo_marking = ?5, tempo_bpm = ?6, notes = ?7, tags = ?8, updated_at = ?9 WHERE id = ?10",
        libsql::params![
            title.as_str(),
            composer,
            category,
            key,
            tempo_marking.as_deref(),
            tempo_bpm,
            notes,
            tags_json.as_str(),
            now_str.as_str(),
            id
        ],
    )
    .await?;

    Ok(Some(Exercise {
        id: id.to_string(),
        title: title.to_string(),
        composer: composer.map(|s| s.to_string()),
        category: category.map(|s| s.to_string()),
        key: key.map(|s| s.to_string()),
        tempo,
        notes: notes.map(|s| s.to_string()),
        tags: tags.clone(),
        created_at: current.created_at,
        updated_at: now,
    }))
}

pub async fn delete_exercise(conn: &Connection, id: &str) -> Result<bool, ApiError> {
    let rows_affected = conn
        .execute("DELETE FROM exercises WHERE id = ?1", libsql::params![id])
        .await?;

    Ok(rows_affected > 0)
}

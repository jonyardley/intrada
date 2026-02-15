use chrono::Utc;
use sqlx::{PgPool, Row};

use intrada_core::{CreateExercise, Exercise, Tempo, UpdateExercise};

use crate::error::ApiError;

pub async fn insert_exercise(pool: &PgPool, input: &CreateExercise) -> Result<Exercise, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let (tempo_marking, tempo_bpm) = flatten_tempo(&input.tempo);

    sqlx::query(
        "INSERT INTO exercises (id, title, composer, category, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.composer)
    .bind(&input.category)
    .bind(&input.key)
    .bind(&tempo_marking)
    .bind(tempo_bpm)
    .bind(&input.notes)
    .bind(&input.tags)
    .bind(now)
    .bind(now)
    .execute(pool)
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

pub async fn list_exercises(pool: &PgPool) -> Result<Vec<Exercise>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, title, composer, category, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at
         FROM exercises ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(row_to_exercise).collect())
}

pub async fn get_exercise(pool: &PgPool, id: &str) -> Result<Option<Exercise>, ApiError> {
    let row = sqlx::query(
        "SELECT id, title, composer, category, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at
         FROM exercises WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(row_to_exercise))
}

pub async fn update_exercise(
    pool: &PgPool,
    id: &str,
    input: &UpdateExercise,
) -> Result<Option<Exercise>, ApiError> {
    let existing = get_exercise(pool, id).await?;
    let Some(mut exercise) = existing else {
        return Ok(None);
    };

    if let Some(ref title) = input.title {
        exercise.title = title.clone();
    }
    if let Some(ref composer) = input.composer {
        exercise.composer = composer.clone();
    }
    if let Some(ref category) = input.category {
        exercise.category = category.clone();
    }
    if let Some(ref key) = input.key {
        exercise.key = key.clone();
    }
    if let Some(ref tempo) = input.tempo {
        exercise.tempo = tempo.clone();
    }
    if let Some(ref notes) = input.notes {
        exercise.notes = notes.clone();
    }
    if let Some(ref tags) = input.tags {
        exercise.tags = tags.clone();
    }
    exercise.updated_at = Utc::now();

    let (tempo_marking, tempo_bpm) = flatten_tempo(&exercise.tempo);

    sqlx::query(
        "UPDATE exercises
         SET title = $2, composer = $3, category = $4, key = $5, tempo_marking = $6, tempo_bpm = $7,
             notes = $8, tags = $9, updated_at = $10
         WHERE id = $1",
    )
    .bind(&exercise.id)
    .bind(&exercise.title)
    .bind(&exercise.composer)
    .bind(&exercise.category)
    .bind(&exercise.key)
    .bind(&tempo_marking)
    .bind(tempo_bpm)
    .bind(&exercise.notes)
    .bind(&exercise.tags)
    .bind(exercise.updated_at)
    .execute(pool)
    .await?;

    Ok(Some(exercise))
}

pub async fn delete_exercise(pool: &PgPool, id: &str) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM exercises WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

fn row_to_exercise(row: sqlx::postgres::PgRow) -> Exercise {
    let tempo_marking: Option<String> = row.get("tempo_marking");
    let tempo_bpm: Option<i16> = row.get("tempo_bpm");
    let tags: Vec<String> = row.get("tags");

    Exercise {
        id: row.get("id"),
        title: row.get("title"),
        composer: row.get("composer"),
        category: row.get("category"),
        key: row.get("key"),
        tempo: Tempo::from_parts(tempo_marking, tempo_bpm.map(|v| v as u16)),
        notes: row.get("notes"),
        tags,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn flatten_tempo(tempo: &Option<Tempo>) -> (Option<String>, Option<i16>) {
    match tempo {
        Some(t) => (t.marking.clone(), t.bpm.map(|b| b as i16)),
        None => (None, None),
    }
}

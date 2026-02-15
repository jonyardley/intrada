use chrono::Utc;
use sqlx::{PgPool, Row};

use intrada_core::{CreatePiece, Piece, Tempo, UpdatePiece};

use crate::error::ApiError;

pub async fn insert_piece(pool: &PgPool, input: &CreatePiece) -> Result<Piece, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let (tempo_marking, tempo_bpm) = flatten_tempo(&input.tempo);

    sqlx::query(
        "INSERT INTO pieces (id, title, composer, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.composer)
    .bind(&input.key)
    .bind(&tempo_marking)
    .bind(tempo_bpm)
    .bind(&input.notes)
    .bind(&input.tags)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Piece {
        id,
        title: input.title.clone(),
        composer: input.composer.clone(),
        key: input.key.clone(),
        tempo: input.tempo.clone(),
        notes: input.notes.clone(),
        tags: input.tags.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub async fn list_pieces(pool: &PgPool) -> Result<Vec<Piece>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, title, composer, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at
         FROM pieces ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(row_to_piece).collect())
}

pub async fn get_piece(pool: &PgPool, id: &str) -> Result<Option<Piece>, ApiError> {
    let row = sqlx::query(
        "SELECT id, title, composer, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at
         FROM pieces WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(row_to_piece))
}

pub async fn update_piece(
    pool: &PgPool,
    id: &str,
    input: &UpdatePiece,
) -> Result<Option<Piece>, ApiError> {
    let existing = get_piece(pool, id).await?;
    let Some(mut piece) = existing else {
        return Ok(None);
    };

    if let Some(ref title) = input.title {
        piece.title = title.clone();
    }
    if let Some(ref composer) = input.composer {
        piece.composer = composer.clone();
    }
    if let Some(ref key) = input.key {
        piece.key = key.clone();
    }
    if let Some(ref tempo) = input.tempo {
        piece.tempo = tempo.clone();
    }
    if let Some(ref notes) = input.notes {
        piece.notes = notes.clone();
    }
    if let Some(ref tags) = input.tags {
        piece.tags = tags.clone();
    }
    piece.updated_at = Utc::now();

    let (tempo_marking, tempo_bpm) = flatten_tempo(&piece.tempo);

    sqlx::query(
        "UPDATE pieces
         SET title = $2, composer = $3, key = $4, tempo_marking = $5, tempo_bpm = $6,
             notes = $7, tags = $8, updated_at = $9
         WHERE id = $1",
    )
    .bind(&piece.id)
    .bind(&piece.title)
    .bind(&piece.composer)
    .bind(&piece.key)
    .bind(&tempo_marking)
    .bind(tempo_bpm)
    .bind(&piece.notes)
    .bind(&piece.tags)
    .bind(piece.updated_at)
    .execute(pool)
    .await?;

    Ok(Some(piece))
}

pub async fn delete_piece(pool: &PgPool, id: &str) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM pieces WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

fn row_to_piece(row: sqlx::postgres::PgRow) -> Piece {
    let tempo_marking: Option<String> = row.get("tempo_marking");
    let tempo_bpm: Option<i16> = row.get("tempo_bpm");
    let tags: Vec<String> = row.get("tags");

    Piece {
        id: row.get("id"),
        title: row.get("title"),
        composer: row.get("composer"),
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

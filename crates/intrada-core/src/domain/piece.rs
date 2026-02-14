use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use super::types::{CreatePiece, Tempo, UpdatePiece};
use crate::app::{Effect, Event, StorageEffect};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Piece {
    pub id: String,
    pub title: String,
    pub composer: String,
    pub key: Option<String>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PieceEvent {
    // User actions
    Add(CreatePiece),
    Update { id: String, input: UpdatePiece },
    Delete { id: String },
    AddTags { id: String, tags: Vec<String> },
    RemoveTags { id: String, tags: Vec<String> },

    // Storage responses
    Saved(Piece),
    Updated(Piece),
    Deleted { id: String },
}

pub fn handle_piece_event(event: PieceEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        PieceEvent::Add(input) => {
            if let Err(e) = validation::validate_create_piece(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let piece = Piece {
                id: ulid::Ulid::new().to_string(),
                title: input.title,
                composer: input.composer,
                key: input.key,
                tempo: input.tempo,
                notes: input.notes,
                tags: input.tags,
                created_at: now,
                updated_at: now,
            };

            model.pieces.push(piece.clone());
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::SavePiece(piece)).into(),
                crux_core::render::render(),
            ])
        }
        PieceEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_piece(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(piece) = model.pieces.iter_mut().find(|p| p.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if let Some(title) = input.title {
                piece.title = title;
            }
            if let Some(composer) = input.composer {
                piece.composer = composer;
            }
            if let Some(key) = input.key {
                piece.key = key;
            }
            if let Some(tempo) = input.tempo {
                piece.tempo = tempo;
            }
            if let Some(notes) = input.notes {
                piece.notes = notes;
            }
            if let Some(tags) = input.tags {
                piece.tags = tags;
            }
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            Command::all([
                Command::notify_shell(StorageEffect::UpdatePiece(piece)).into(),
                crux_core::render::render(),
            ])
        }
        PieceEvent::Delete { id } => {
            let len_before = model.pieces.len();
            model.pieces.retain(|p| p.id != id);
            if model.pieces.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            }
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::DeleteItem { id }).into(),
                crux_core::render::render(),
            ])
        }
        PieceEvent::AddTags { id, tags } => {
            if let Err(e) = validation::validate_tags(&tags) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(piece) = model.pieces.iter_mut().find(|p| p.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            for tag in tags {
                let tag_lower = tag.to_lowercase();
                if !piece.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                    piece.tags.push(tag);
                }
            }
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            Command::all([
                Command::notify_shell(StorageEffect::UpdatePiece(piece)).into(),
                crux_core::render::render(),
            ])
        }
        PieceEvent::RemoveTags { id, tags } => {
            let Some(piece) = model.pieces.iter_mut().find(|p| p.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
            piece
                .tags
                .retain(|t| !tags_lower.contains(&t.to_lowercase()));
            piece.updated_at = chrono::Utc::now();
            model.last_error = None;

            let piece = piece.clone();
            Command::all([
                Command::notify_shell(StorageEffect::UpdatePiece(piece)).into(),
                crux_core::render::render(),
            ])
        }
        // Storage confirmation events — model already updated optimistically
        PieceEvent::Saved(_) | PieceEvent::Updated(_) | PieceEvent::Deleted { .. } => {
            Command::done()
        }
    }
}

use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::types::{CreateItem, Tempo, UpdateItem};
use crate::app::{Effect, Event};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

/// Discriminates between a piece (repertoire) and an exercise (technique drill).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Piece,
    Exercise,
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Piece => write!(f, "piece"),
            ItemKind::Exercise => write!(f, "exercise"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub kind: ItemKind,
    pub composer: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ItemEvent {
    Add(CreateItem),
    Update { id: String, input: UpdateItem },
    Delete { id: String },
    AddTags { id: String, tags: Vec<String> },
    RemoveTags { id: String, tags: Vec<String> },
}

pub fn handle_item_event(event: ItemEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        ItemEvent::Add(input) => {
            if let Err(e) = validation::validate_create_item(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let item = Item {
                id: ulid::Ulid::new().to_string(),
                title: input.title,
                kind: input.kind,
                composer: input.composer,
                key: input.key,
                tempo: input.tempo,
                notes: input.notes,
                tags: input.tags,
                created_at: now,
                updated_at: now,
            };

            model.items.push(item.clone());
            model.last_error = None;

            Command::all([
                crate::http::create_item(&model.api_base_url, &item),
                crux_core::render::render(),
            ])
        }
        ItemEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_item(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if let Some(title) = input.title {
                item.title = title;
            }
            if let Some(composer) = input.composer {
                item.composer = composer;
            }
            if let Some(key) = input.key {
                item.key = key;
            }
            if let Some(tempo) = input.tempo {
                item.tempo = tempo;
            }
            if let Some(notes) = input.notes {
                item.notes = notes;
            }
            if let Some(tags) = input.tags {
                item.tags = tags;
            }
            item.updated_at = chrono::Utc::now();
            model.last_error = None;

            let item = item.clone();
            Command::all([
                crate::http::update_item(&model.api_base_url, &item),
                crux_core::render::render(),
            ])
        }
        ItemEvent::Delete { id } => {
            let len_before = model.items.len();
            model.items.retain(|i| i.id != id);
            if model.items.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            }
            model.last_error = None;

            Command::all([
                crate::http::delete_item(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }
        ItemEvent::AddTags { id, tags } => {
            if let Err(e) = validation::validate_tags(&tags) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            for tag in tags {
                let tag_lower = tag.to_lowercase();
                if !item.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                    item.tags.push(tag);
                }
            }
            item.updated_at = chrono::Utc::now();
            model.last_error = None;

            let item = item.clone();
            Command::all([
                crate::http::update_item(&model.api_base_url, &item),
                crux_core::render::render(),
            ])
        }
        ItemEvent::RemoveTags { id, tags } => {
            let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
            item.tags
                .retain(|t| !tags_lower.contains(&t.to_lowercase()));
            item.updated_at = chrono::Utc::now();
            model.last_error = None;

            let item = item.clone();
            Command::all([
                crate::http::update_item(&model.api_base_url, &item),
                crux_core::render::render(),
            ])
        }
    }
}

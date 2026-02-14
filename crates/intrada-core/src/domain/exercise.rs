use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use super::types::{CreateExercise, Tempo, UpdateExercise};
use crate::app::{Effect, Event, StorageEffect};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub title: String,
    pub composer: Option<String>,
    pub category: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ExerciseEvent {
    // User actions
    Add(CreateExercise),
    Update { id: String, input: UpdateExercise },
    Delete { id: String },
    AddTags { id: String, tags: Vec<String> },
    RemoveTags { id: String, tags: Vec<String> },

    // Storage responses
    Saved(Exercise),
    Updated(Exercise),
    Deleted { id: String },
}

pub fn handle_exercise_event(
    event: ExerciseEvent,
    model: &mut Model,
) -> Command<Effect, Event> {
    match event {
        ExerciseEvent::Add(input) => {
            if let Err(e) = validation::validate_create_exercise(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let exercise = Exercise {
                id: ulid::Ulid::new().to_string(),
                title: input.title,
                composer: input.composer,
                category: input.category,
                key: input.key,
                tempo: input.tempo,
                notes: input.notes,
                tags: input.tags,
                created_at: now,
                updated_at: now,
            };

            model.exercises.push(exercise.clone());
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::SaveExercise(exercise)).into(),
                crux_core::render::render(),
            ])
        }
        ExerciseEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_exercise(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(exercise) = model.exercises.iter_mut().find(|e| e.id == id) else {
                model.last_error = Some(
                    LibraryError::NotFound { id }.to_string(),
                );
                return crux_core::render::render();
            };

            if let Some(title) = input.title {
                exercise.title = title;
            }
            if let Some(composer) = input.composer {
                exercise.composer = composer;
            }
            if let Some(category) = input.category {
                exercise.category = category;
            }
            if let Some(key) = input.key {
                exercise.key = key;
            }
            if let Some(tempo) = input.tempo {
                exercise.tempo = tempo;
            }
            if let Some(notes) = input.notes {
                exercise.notes = notes;
            }
            if let Some(tags) = input.tags {
                exercise.tags = tags;
            }
            exercise.updated_at = chrono::Utc::now();
            model.last_error = None;

            let exercise = exercise.clone();
            Command::all([
                Command::notify_shell(StorageEffect::UpdateExercise(exercise)).into(),
                crux_core::render::render(),
            ])
        }
        ExerciseEvent::Delete { id } => {
            let len_before = model.exercises.len();
            model.exercises.retain(|e| e.id != id);
            if model.exercises.len() == len_before {
                model.last_error = Some(
                    LibraryError::NotFound { id }.to_string(),
                );
                return crux_core::render::render();
            }
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::DeleteItem { id }).into(),
                crux_core::render::render(),
            ])
        }
        ExerciseEvent::AddTags { id, tags } => {
            if let Err(e) = validation::validate_tags(&tags) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(exercise) = model.exercises.iter_mut().find(|e| e.id == id) else {
                model.last_error = Some(
                    LibraryError::NotFound { id }.to_string(),
                );
                return crux_core::render::render();
            };

            for tag in tags {
                let tag_lower = tag.to_lowercase();
                if !exercise.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                    exercise.tags.push(tag);
                }
            }
            exercise.updated_at = chrono::Utc::now();
            model.last_error = None;

            let exercise = exercise.clone();
            Command::all([
                Command::notify_shell(StorageEffect::UpdateExercise(exercise)).into(),
                crux_core::render::render(),
            ])
        }
        ExerciseEvent::RemoveTags { id, tags } => {
            let Some(exercise) = model.exercises.iter_mut().find(|e| e.id == id) else {
                model.last_error = Some(
                    LibraryError::NotFound { id }.to_string(),
                );
                return crux_core::render::render();
            };

            let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
            exercise.tags.retain(|t| !tags_lower.contains(&t.to_lowercase()));
            exercise.updated_at = chrono::Utc::now();
            model.last_error = None;

            let exercise = exercise.clone();
            Command::all([
                Command::notify_shell(StorageEffect::UpdateExercise(exercise)).into(),
                crux_core::render::render(),
            ])
        }
        // Storage confirmation events
        ExerciseEvent::Saved(_) | ExerciseEvent::Updated(_) | ExerciseEvent::Deleted { .. } => {
            Command::done()
        }
    }
}

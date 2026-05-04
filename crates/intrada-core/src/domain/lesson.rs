use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use super::types::{CreateLesson, UpdateLesson};
use crate::app::{Effect, Event};
use crate::model::Model;
use crate::validation;

/// A record of a single teaching session.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Lesson {
    pub id: String,
    pub date: String,
    pub notes: Option<String>,
    pub photos: Vec<LessonPhoto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Photo metadata for a lesson. Binary data lives in R2; core only sees metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LessonPhoto {
    pub id: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LessonEvent {
    FetchLessons,
    FetchLesson { id: String },
    Add(CreateLesson),
    Update { id: String, input: UpdateLesson },
    Delete { id: String },
}

pub fn handle_lesson_event(event: LessonEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        LessonEvent::FetchLessons => crate::http::fetch_lessons(&model.api_base_url),
        LessonEvent::FetchLesson { id } => crate::http::fetch_lesson(&model.api_base_url, &id),
        LessonEvent::Add(input) => {
            if let Err(e) = validation::validate_create_lesson(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let lesson = Lesson {
                id: ulid::Ulid::new().to_string(),
                date: input.date.clone(),
                notes: input.notes.clone(),
                photos: Vec::new(),
                created_at: now,
                updated_at: now,
            };

            model.lessons.push(lesson);
            model.last_error = None;

            Command::all([
                crate::http::create_lesson(&model.api_base_url, &input),
                crux_core::render::render(),
            ])
        }
        LessonEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_lesson(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(lesson) = model.lessons.iter_mut().find(|l| l.id == id) else {
                model.last_error = Some(format!("Lesson not found: {id}"));
                return crux_core::render::render();
            };

            if let Some(date) = &input.date {
                lesson.date = date.clone();
            }
            if let Some(notes) = &input.notes {
                lesson.notes = notes.clone();
            }
            lesson.updated_at = chrono::Utc::now();
            model.last_error = None;

            let lesson_id = id.clone();
            Command::all([
                crate::http::update_lesson(&model.api_base_url, &lesson_id, &input),
                crux_core::render::render(),
            ])
        }
        LessonEvent::Delete { id } => {
            let len_before = model.lessons.len();
            model.lessons.retain(|l| l.id != id);
            if model.lessons.len() == len_before {
                model.last_error = Some(format!("Lesson not found: {id}"));
                return crux_core::render::render();
            }
            model.last_error = None;

            Command::all([
                crate::http::delete_lesson(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }
    }
}

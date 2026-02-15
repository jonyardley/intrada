use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use super::types::{LogSession, UpdateSession};
use crate::app::{Effect, Event, StorageEffect};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub item_id: String,
    pub duration_minutes: u32,
    pub started_at: DateTime<Utc>,
    pub logged_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SessionEvent {
    Log(LogSession),
    Update { id: String, input: UpdateSession },
    Delete { id: String },
}

pub fn handle_session_event(event: SessionEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        SessionEvent::Log(input) => {
            if let Err(e) = validation::validate_log_session(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let now = chrono::Utc::now();
            let duration = chrono::Duration::minutes(i64::from(input.duration_minutes));
            let started_at = now - duration;

            let session = Session {
                id: ulid::Ulid::new().to_string(),
                item_id: input.item_id,
                duration_minutes: input.duration_minutes,
                started_at,
                logged_at: now,
                notes: input.notes,
            };

            model.sessions.push(session.clone());
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::SaveSession(session)).into(),
                crux_core::render::render(),
            ])
        }
        SessionEvent::Update { id, input } => {
            if let Err(e) = validation::validate_update_session(&input) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(session) = model.sessions.iter_mut().find(|s| s.id == id) else {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            };

            if let Some(duration_minutes) = input.duration_minutes {
                session.duration_minutes = duration_minutes;
            }
            if let Some(notes) = input.notes {
                session.notes = notes;
            }

            model.last_error = None;
            let session = session.clone();

            Command::all([
                Command::notify_shell(StorageEffect::UpdateSession(session)).into(),
                crux_core::render::render(),
            ])
        }
        SessionEvent::Delete { id } => {
            let len_before = model.sessions.len();
            model.sessions.retain(|s| s.id != id);

            if model.sessions.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id }.to_string());
                return crux_core::render::render();
            }

            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::DeleteSession { id }).into(),
                crux_core::render::render(),
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Intrada;
    use crux_core::App;

    fn log_session(model: &mut Model, item_id: &str, duration: u32, notes: Option<&str>) {
        let app = Intrada;
        let _cmd = app.update(
            Event::Session(SessionEvent::Log(LogSession {
                item_id: item_id.to_string(),
                duration_minutes: duration,
                notes: notes.map(|s| s.to_string()),
            })),
            model,
        );
    }

    #[test]
    fn test_log_session_valid() {
        let mut model = Model::default();
        log_session(&mut model, "item1", 30, Some("Good practice"));

        assert!(model.last_error.is_none());
        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].item_id, "item1");
        assert_eq!(model.sessions[0].duration_minutes, 30);
        assert_eq!(model.sessions[0].notes, Some("Good practice".to_string()));
        assert!(!model.sessions[0].id.is_empty());
    }

    #[test]
    fn test_log_session_without_notes() {
        let mut model = Model::default();
        log_session(&mut model, "item1", 15, None);

        assert!(model.last_error.is_none());
        assert_eq!(model.sessions.len(), 1);
        assert!(model.sessions[0].notes.is_none());
    }

    #[test]
    fn test_log_session_duration_zero() {
        let mut model = Model::default();
        log_session(&mut model, "item1", 0, None);

        assert!(model.last_error.is_some());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_log_session_duration_too_high() {
        let mut model = Model::default();
        log_session(&mut model, "item1", 1441, None);

        assert!(model.last_error.is_some());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_log_session_duration_at_boundaries() {
        let mut model = Model::default();

        log_session(&mut model, "item1", 1, None);
        assert!(model.last_error.is_none());
        assert_eq!(model.sessions.len(), 1);

        log_session(&mut model, "item2", 1440, None);
        assert!(model.last_error.is_none());
        assert_eq!(model.sessions.len(), 2);
    }

    #[test]
    fn test_log_session_empty_item_id() {
        let mut model = Model::default();
        log_session(&mut model, "", 30, None);

        assert!(model.last_error.is_some());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_log_session_notes_too_long() {
        let mut model = Model::default();
        let long_notes = "x".repeat(5001);
        log_session(&mut model, "item1", 30, Some(&long_notes));

        assert!(model.last_error.is_some());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_log_session_started_at_before_logged_at() {
        let mut model = Model::default();
        log_session(&mut model, "item1", 45, None);

        let session = &model.sessions[0];
        assert!(session.started_at < session.logged_at);
        let diff = session.logged_at - session.started_at;
        assert_eq!(diff.num_minutes(), 45);
    }

    #[test]
    fn test_update_session_duration() {
        let app = Intrada;
        let mut model = Model::default();
        log_session(&mut model, "item1", 30, None);
        let session_id = model.sessions[0].id.clone();

        let _cmd = app.update(
            Event::Session(SessionEvent::Update {
                id: session_id.clone(),
                input: UpdateSession {
                    duration_minutes: Some(45),
                    notes: None,
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.sessions[0].duration_minutes, 45);
    }

    #[test]
    fn test_update_session_notes() {
        let app = Intrada;
        let mut model = Model::default();
        log_session(&mut model, "item1", 30, None);
        let session_id = model.sessions[0].id.clone();

        let _cmd = app.update(
            Event::Session(SessionEvent::Update {
                id: session_id.clone(),
                input: UpdateSession {
                    duration_minutes: None,
                    notes: Some(Some("Added notes".to_string())),
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.sessions[0].notes, Some("Added notes".to_string()));
    }

    #[test]
    fn test_update_session_clear_notes() {
        let app = Intrada;
        let mut model = Model::default();
        log_session(&mut model, "item1", 30, Some("Old notes"));
        let session_id = model.sessions[0].id.clone();

        let _cmd = app.update(
            Event::Session(SessionEvent::Update {
                id: session_id.clone(),
                input: UpdateSession {
                    duration_minutes: None,
                    notes: Some(None),
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert!(model.sessions[0].notes.is_none());
    }

    #[test]
    fn test_update_session_invalid_duration() {
        let app = Intrada;
        let mut model = Model::default();
        log_session(&mut model, "item1", 30, None);
        let session_id = model.sessions[0].id.clone();

        let _cmd = app.update(
            Event::Session(SessionEvent::Update {
                id: session_id.clone(),
                input: UpdateSession {
                    duration_minutes: Some(0),
                    notes: None,
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_some());
        // Duration should remain unchanged
        assert_eq!(model.sessions[0].duration_minutes, 30);
    }

    #[test]
    fn test_update_session_not_found() {
        let app = Intrada;
        let mut model = Model::default();

        let _cmd = app.update(
            Event::Session(SessionEvent::Update {
                id: "nonexistent".to_string(),
                input: UpdateSession {
                    duration_minutes: Some(45),
                    notes: None,
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_some());
        assert!(model.last_error.as_ref().unwrap().contains("not found"));
    }

    #[test]
    fn test_delete_session() {
        let app = Intrada;
        let mut model = Model::default();
        log_session(&mut model, "item1", 30, None);
        let session_id = model.sessions[0].id.clone();

        let _cmd = app.update(
            Event::Session(SessionEvent::Delete {
                id: session_id.clone(),
            }),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_delete_session_not_found() {
        let app = Intrada;
        let mut model = Model::default();

        let _cmd = app.update(
            Event::Session(SessionEvent::Delete {
                id: "nonexistent".to_string(),
            }),
            &mut model,
        );

        assert!(model.last_error.is_some());
        assert!(model.last_error.as_ref().unwrap().contains("not found"));
    }
}

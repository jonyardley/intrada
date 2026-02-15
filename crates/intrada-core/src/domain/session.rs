use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event, StorageEffect};
use crate::domain::exercise::Exercise;
use crate::domain::piece::Piece;
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;

// ── Enums ──────────────────────────────────────────────────────────────

/// Completion status of a single setlist entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EntryStatus {
    /// Item was practised and time was recorded.
    Completed,
    /// Item was explicitly skipped (duration_secs = 0).
    Skipped,
    /// Session ended early before reaching this item (duration_secs = 0).
    NotAttempted,
}

/// Whether the session ran to completion or was ended early.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CompletionStatus {
    /// All items in the setlist were addressed (completed or skipped).
    Completed,
    /// Session was ended before all items were reached.
    EndedEarly,
}

// ── Domain Types ───────────────────────────────────────────────────────

/// An individual item within a session's setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetlistEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    pub position: usize,
    pub duration_secs: u64,
    pub status: EntryStatus,
    pub notes: Option<String>,
}

/// A completed practice session (persisted to localStorage).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PracticeSession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub session_notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_secs: u64,
    pub completion_status: CompletionStatus,
}

// ── Transient State Types ──────────────────────────────────────────────

/// State during setlist assembly (Building phase).
#[derive(Debug, Clone)]
pub struct BuildingSession {
    pub entries: Vec<SetlistEntry>,
}

/// State during active practice (Active phase).
/// Persisted to `intrada:session-in-progress` for crash recovery.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ActiveSession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub current_index: usize,
    pub current_item_started_at: DateTime<Utc>,
    pub session_started_at: DateTime<Utc>,
}

/// State during post-session review (Summary phase).
#[derive(Debug, Clone)]
pub struct SummarySession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub session_started_at: DateTime<Utc>,
    pub session_ended_at: DateTime<Utc>,
    pub session_notes: Option<String>,
    pub completion_status: CompletionStatus,
}

/// The lifecycle state of a session in the core Model.
#[derive(Debug, Clone, Default)]
pub enum SessionStatus {
    #[default]
    Idle,
    Building(BuildingSession),
    Active(ActiveSession),
    Summary(SummarySession),
}

// ── Events ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SessionEvent {
    // === Building Phase ===
    StartBuilding,
    AddToSetlist {
        item_id: String,
    },
    AddNewItemToSetlist {
        title: String,
        item_type: String,
    },
    RemoveFromSetlist {
        entry_id: String,
    },
    ReorderSetlist {
        entry_id: String,
        new_position: usize,
    },
    StartSession {
        now: DateTime<Utc>,
    },
    CancelBuilding,

    // === Active Phase ===
    NextItem {
        now: DateTime<Utc>,
    },
    SkipItem {
        now: DateTime<Utc>,
    },
    AddItemMidSession {
        item_id: String,
    },
    AddNewItemMidSession {
        title: String,
        item_type: String,
    },
    FinishSession {
        now: DateTime<Utc>,
    },
    EndSessionEarly {
        now: DateTime<Utc>,
    },

    // === Summary Phase ===
    UpdateEntryNotes {
        entry_id: String,
        notes: Option<String>,
    },
    UpdateSessionNotes {
        notes: Option<String>,
    },
    SaveSession {
        now: DateTime<Utc>,
    },
    DiscardSession,

    // === Recovery ===
    RecoverSession {
        session: ActiveSession,
    },

    // === History ===
    DeleteSession {
        id: String,
    },
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Format seconds into a human-readable duration string.
pub fn format_duration_display(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

/// Look up a library item by ID and return (title, type_string).
fn find_item_in_model(model: &Model, item_id: &str) -> Option<(String, String)> {
    if let Some(piece) = model.pieces.iter().find(|p| p.id == item_id) {
        return Some((piece.title.clone(), "piece".to_string()));
    }
    if let Some(exercise) = model.exercises.iter().find(|e| e.id == item_id) {
        return Some((exercise.title.clone(), "exercise".to_string()));
    }
    None
}

/// Create a new SetlistEntry from a library item lookup.
fn create_entry(item_id: &str, item_title: &str, item_type: &str, position: usize) -> SetlistEntry {
    SetlistEntry {
        id: ulid::Ulid::new().to_string(),
        item_id: item_id.to_string(),
        item_title: item_title.to_string(),
        item_type: item_type.to_string(),
        position,
        duration_secs: 0,
        status: EntryStatus::NotAttempted,
        notes: None,
    }
}

/// Re-index entry positions after a mutation.
fn reindex_entries(entries: &mut [SetlistEntry]) {
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.position = i;
    }
}

/// Create a minimal Piece from title-only input.
fn create_piece_from_title(title: &str) -> Piece {
    let now = Utc::now();
    Piece {
        id: ulid::Ulid::new().to_string(),
        title: title.to_string(),
        composer: String::new(),
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: now,
        updated_at: now,
    }
}

/// Create a minimal Exercise from title-only input.
fn create_exercise_from_title(title: &str) -> Exercise {
    let now = Utc::now();
    Exercise {
        id: ulid::Ulid::new().to_string(),
        title: title.to_string(),
        composer: None,
        category: None,
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: now,
        updated_at: now,
    }
}

/// Transition from Active to Summary, computing final duration for the current item.
fn transition_to_summary(
    active: &mut ActiveSession,
    now: DateTime<Utc>,
    completion_status: CompletionStatus,
) -> SummarySession {
    // Record duration for current item
    let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.duration_secs = elapsed;
        entry.status = EntryStatus::Completed;
    }

    // Mark remaining items as NotAttempted if ending early
    if completion_status == CompletionStatus::EndedEarly {
        for entry in active.entries.iter_mut().skip(active.current_index + 1) {
            entry.status = EntryStatus::NotAttempted;
            entry.duration_secs = 0;
        }
    }

    SummarySession {
        id: active.id.clone(),
        entries: active.entries.clone(),
        session_started_at: active.session_started_at,
        session_ended_at: now,
        session_notes: None,
        completion_status,
    }
}

// ── Event Handler ──────────────────────────────────────────────────────

pub fn handle_session_event(event: SessionEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        // ── Building Phase ─────────────────────────────────────────
        SessionEvent::StartBuilding => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error = Some("A session is already in progress".to_string());
                return crux_core::render::render();
            }
            model.session_status = SessionStatus::Building(BuildingSession { entries: vec![] });
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::AddToSetlist { item_id } => {
            if !matches!(model.session_status, SessionStatus::Building(_)) {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            }

            let Some((title, item_type)) = find_item_in_model(model, &item_id) else {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            };

            let SessionStatus::Building(ref mut building) = model.session_status else {
                unreachable!()
            };
            let position = building.entries.len();
            let entry = create_entry(&item_id, &title, &item_type, position);
            building.entries.push(entry);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::AddNewItemToSetlist { title, item_type } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            if title.is_empty() || title.len() > validation::MAX_TITLE {
                model.last_error = Some(format!(
                    "Title must be between 1 and {} characters",
                    validation::MAX_TITLE
                ));
                return crux_core::render::render();
            }

            let (new_item_id, storage_effect) = match item_type.as_str() {
                "piece" => {
                    let piece = create_piece_from_title(&title);
                    let id = piece.id.clone();
                    model.pieces.push(piece.clone());
                    (id, StorageEffect::SavePiece(piece))
                }
                "exercise" => {
                    let exercise = create_exercise_from_title(&title);
                    let id = exercise.id.clone();
                    model.exercises.push(exercise.clone());
                    (id, StorageEffect::SaveExercise(exercise))
                }
                _ => {
                    model.last_error = Some("Item type must be 'piece' or 'exercise'".to_string());
                    return crux_core::render::render();
                }
            };

            let position = building.entries.len();
            let entry = create_entry(&new_item_id, &title, &item_type, position);
            building.entries.push(entry);
            model.last_error = None;

            Command::all([
                Command::notify_shell(storage_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::RemoveFromSetlist { entry_id } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let len_before = building.entries.len();
            building.entries.retain(|e| e.id != entry_id);

            if building.entries.len() == len_before {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            }

            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::ReorderSetlist {
            entry_id,
            new_position,
        } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let Some(current_index) = building.entries.iter().position(|e| e.id == entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            };

            if new_position >= building.entries.len() {
                model.last_error = Some(format!(
                    "Invalid position: {new_position} (max: {})",
                    building.entries.len() - 1
                ));
                return crux_core::render::render();
            }

            let entry = building.entries.remove(current_index);
            building.entries.insert(new_position, entry);
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::StartSession { now } => {
            let SessionStatus::Building(ref building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_setlist_not_empty(&building.entries) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let active = ActiveSession {
                id: ulid::Ulid::new().to_string(),
                entries: building.entries.clone(),
                current_index: 0,
                current_item_started_at: now,
                session_started_at: now,
            };

            let save_effect = StorageEffect::SaveSessionInProgress(active.clone());
            model.session_status = SessionStatus::Active(active);
            model.last_error = None;

            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::CancelBuilding => {
            if !matches!(model.session_status, SessionStatus::Building(_)) {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            }
            model.session_status = SessionStatus::Idle;
            model.last_error = None;
            crux_core::render::render()
        }

        // ── Active Phase ───────────────────────────────────────────
        SessionEvent::NextItem { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;

            if let Some(entry) = active.entries.get_mut(active.current_index) {
                entry.duration_secs = elapsed;
                entry.status = EntryStatus::Completed;
            }

            // If this was the last item, transition to Summary
            if active.current_index >= active.entries.len() - 1 {
                let summary = transition_to_summary(active, now, CompletionStatus::Completed);
                model.session_status = SessionStatus::Summary(summary);
                model.last_error = None;
                return crux_core::render::render();
            }

            active.current_index += 1;
            active.current_item_started_at = now;
            model.last_error = None;

            let save_effect = StorageEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::SkipItem { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            if let Some(entry) = active.entries.get_mut(active.current_index) {
                entry.duration_secs = 0;
                entry.status = EntryStatus::Skipped;
            }

            // If this was the last item, transition to Summary
            if active.current_index >= active.entries.len() - 1 {
                let summary = SummarySession {
                    id: active.id.clone(),
                    entries: active.entries.clone(),
                    session_started_at: active.session_started_at,
                    session_ended_at: now,
                    session_notes: None,
                    completion_status: CompletionStatus::Completed,
                };
                model.session_status = SessionStatus::Summary(summary);
                model.last_error = None;
                return crux_core::render::render();
            }

            active.current_index += 1;
            active.current_item_started_at = now;
            model.last_error = None;

            let save_effect = StorageEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::AddItemMidSession { item_id } => {
            if !matches!(model.session_status, SessionStatus::Active(_)) {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            }

            let Some((title, item_type)) = find_item_in_model(model, &item_id) else {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            };

            let SessionStatus::Active(ref mut active) = model.session_status else {
                unreachable!()
            };
            let position = active.entries.len();
            let entry = create_entry(&item_id, &title, &item_type, position);
            active.entries.push(entry);
            model.last_error = None;

            let save_effect = StorageEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::AddNewItemMidSession { title, item_type } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            if title.is_empty() || title.len() > validation::MAX_TITLE {
                model.last_error = Some(format!(
                    "Title must be between 1 and {} characters",
                    validation::MAX_TITLE
                ));
                return crux_core::render::render();
            }

            let (new_item_id, storage_effect) = match item_type.as_str() {
                "piece" => {
                    let piece = create_piece_from_title(&title);
                    let id = piece.id.clone();
                    model.pieces.push(piece.clone());
                    (id, StorageEffect::SavePiece(piece))
                }
                "exercise" => {
                    let exercise = create_exercise_from_title(&title);
                    let id = exercise.id.clone();
                    model.exercises.push(exercise.clone());
                    (id, StorageEffect::SaveExercise(exercise))
                }
                _ => {
                    model.last_error = Some("Item type must be 'piece' or 'exercise'".to_string());
                    return crux_core::render::render();
                }
            };

            let position = active.entries.len();
            let entry = create_entry(&new_item_id, &title, &item_type, position);
            active.entries.push(entry);
            model.last_error = None;

            let save_effect_session = StorageEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(storage_effect).into(),
                Command::notify_shell(save_effect_session).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::FinishSession { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            let summary = transition_to_summary(active, now, CompletionStatus::Completed);
            model.session_status = SessionStatus::Summary(summary);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::EndSessionEarly { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            let summary = transition_to_summary(active, now, CompletionStatus::EndedEarly);
            model.session_status = SessionStatus::Summary(summary);
            model.last_error = None;
            crux_core::render::render()
        }

        // ── Summary Phase ──────────────────────────────────────────
        SessionEvent::UpdateEntryNotes { entry_id, notes } => {
            let SessionStatus::Summary(ref mut summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_entry_notes(&notes) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(entry) = summary.entries.iter_mut().find(|e| e.id == entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found"));
                return crux_core::render::render();
            };

            entry.notes = notes;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateSessionNotes { notes } => {
            let SessionStatus::Summary(ref mut summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_session_notes(&notes) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            summary.session_notes = notes;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SaveSession { now } => {
            let SessionStatus::Summary(ref summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };

            let total_duration_secs: u64 = summary.entries.iter().map(|e| e.duration_secs).sum();

            let practice_session = PracticeSession {
                id: summary.id.clone(),
                entries: summary.entries.clone(),
                session_notes: summary.session_notes.clone(),
                started_at: summary.session_started_at,
                completed_at: now,
                total_duration_secs,
                completion_status: summary.completion_status.clone(),
            };

            model.sessions.push(practice_session.clone());
            model.session_status = SessionStatus::Idle;
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::SavePracticeSession(practice_session)).into(),
                Command::notify_shell(StorageEffect::ClearSessionInProgress).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::DiscardSession => {
            if !matches!(model.session_status, SessionStatus::Summary(_)) {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            }

            model.session_status = SessionStatus::Idle;
            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::ClearSessionInProgress).into(),
                crux_core::render::render(),
            ])
        }

        // ── Recovery ───────────────────────────────────────────────
        SessionEvent::RecoverSession { session } => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error =
                    Some("Cannot recover: a session is already in progress".to_string());
                return crux_core::render::render();
            }

            model.session_status = SessionStatus::Active(session);
            model.last_error = None;
            crux_core::render::render()
        }

        // ── History ────────────────────────────────────────────────
        SessionEvent::DeleteSession { id } => {
            let len_before = model.sessions.len();
            model.sessions.retain(|s| s.id != id);

            if model.sessions.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id: id.clone() }.to_string());
                return crux_core::render::render();
            }

            model.last_error = None;

            Command::all([
                Command::notify_shell(StorageEffect::DeletePracticeSession { id }).into(),
                crux_core::render::render(),
            ])
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Intrada;
    use crux_core::App;

    fn model_with_library() -> Model {
        let now = Utc::now();
        Model {
            pieces: vec![
                Piece {
                    id: "piece-1".to_string(),
                    title: "Moonlight Sonata".to_string(),
                    composer: "Beethoven".to_string(),
                    key: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                },
                Piece {
                    id: "piece-2".to_string(),
                    title: "Clair de Lune".to_string(),
                    composer: "Debussy".to_string(),
                    key: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                },
            ],
            exercises: vec![Exercise {
                id: "exercise-1".to_string(),
                title: "C Major Scale".to_string(),
                composer: None,
                category: Some("Scales".to_string()),
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
            }],
            ..Default::default()
        }
    }

    fn update(model: &mut Model, event: Event) {
        let app = Intrada;
        let _cmd = app.update(event, model);
    }

    // --- Building Phase Tests ---

    #[test]
    fn test_start_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Building(_)));
    }

    #[test]
    fn test_start_building_when_already_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_add_to_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 1);
            assert_eq!(b.entries[0].item_title, "Moonlight Sonata");
            assert_eq!(b.entries[0].item_type, "piece");
            assert_eq!(b.entries[0].position, 0);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_add_to_setlist_item_not_found() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "nonexistent".to_string(),
            }),
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_add_duplicate_items_to_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 2);
            // Each entry has a unique ID
            assert_ne!(b.entries[0].id, b.entries[1].id);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_remove_from_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RemoveFromSetlist { entry_id }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 1);
            assert_eq!(b.entries[0].item_title, "Clair de Lune");
            assert_eq!(b.entries[0].position, 0); // Re-indexed
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_reorder_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "exercise-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[2].id.clone() // exercise-1 at position 2
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::ReorderSetlist {
                entry_id,
                new_position: 0,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries[0].item_title, "C Major Scale");
            assert_eq!(b.entries[1].item_title, "Moonlight Sonata");
            assert_eq!(b.entries[2].item_title, "Clair de Lune");
            // Verify positions are re-indexed
            assert_eq!(b.entries[0].position, 0);
            assert_eq!(b.entries[1].position, 1);
            assert_eq!(b.entries[2].position, 2);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_start_session_empty_setlist() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        assert!(model.last_error.is_some());
        assert!(matches!(model.session_status, SessionStatus::Building(_)));
    }

    #[test]
    fn test_start_session_with_items() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.current_index, 0);
            assert_eq!(active.entries.len(), 1);
            assert_eq!(active.session_started_at, now);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_cancel_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(&mut model, Event::Session(SessionEvent::CancelBuilding));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
    }

    // --- Active Phase Tests ---

    fn model_with_active_session(item_count: usize) -> (Model, DateTime<Utc>) {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        let items = ["piece-1", "piece-2", "exercise-1"];
        for i in 0..item_count.min(3) {
            update(
                &mut model,
                Event::Session(SessionEvent::AddToSetlist {
                    item_id: items[i].to_string(),
                }),
            );
        }

        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );
        (model, now)
    }

    #[test]
    fn test_next_item() {
        let (mut model, start) = model_with_active_session(3);
        let now = start + chrono::Duration::seconds(30);

        update(&mut model, Event::Session(SessionEvent::NextItem { now }));

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.current_index, 1);
            assert_eq!(active.entries[0].duration_secs, 30);
            assert_eq!(active.entries[0].status, EntryStatus::Completed);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_next_item_on_last_transitions_to_summary() {
        let (mut model, start) = model_with_active_session(1);
        let now = start + chrono::Duration::seconds(60);

        update(&mut model, Event::Session(SessionEvent::NextItem { now }));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Summary(_)));
    }

    #[test]
    fn test_finish_session() {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(30);
        let t2 = t1 + chrono::Duration::seconds(45);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].duration_secs, 30);
            assert_eq!(summary.entries[1].duration_secs, 45);
            assert_eq!(summary.completion_status, CompletionStatus::Completed);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_end_session_early() {
        let (mut model, start) = model_with_active_session(3);
        let t1 = start + chrono::Duration::seconds(30);
        let t2 = t1 + chrono::Duration::seconds(20);

        // Complete first item
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        // End early on second item
        update(
            &mut model,
            Event::Session(SessionEvent::EndSessionEarly { now: t2 }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].duration_secs, 30);
            assert_eq!(summary.entries[0].status, EntryStatus::Completed);
            assert_eq!(summary.entries[1].duration_secs, 20);
            assert_eq!(summary.entries[1].status, EntryStatus::Completed);
            assert_eq!(summary.entries[2].duration_secs, 0);
            assert_eq!(summary.entries[2].status, EntryStatus::NotAttempted);
            assert_eq!(summary.completion_status, CompletionStatus::EndedEarly);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_skip_item() {
        let (mut model, start) = model_with_active_session(3);
        let now = start + chrono::Duration::seconds(10);

        update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.current_index, 1);
            assert_eq!(active.entries[0].duration_secs, 0);
            assert_eq!(active.entries[0].status, EntryStatus::Skipped);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_skip_last_item_transitions_to_summary() {
        let (mut model, start) = model_with_active_session(1);
        let now = start + chrono::Duration::seconds(10);

        update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].status, EntryStatus::Skipped);
            assert_eq!(summary.entries[0].duration_secs, 0);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_add_item_mid_session() {
        let (mut model, _start) = model_with_active_session(2);

        update(
            &mut model,
            Event::Session(SessionEvent::AddItemMidSession {
                item_id: "exercise-1".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.entries.len(), 3);
            assert_eq!(active.entries[2].item_title, "C Major Scale");
            assert_eq!(active.current_index, 0); // Timer not interrupted
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_add_new_item_mid_session() {
        let (mut model, _start) = model_with_active_session(2);

        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemMidSession {
                title: "New Scale".to_string(),
                item_type: "exercise".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.entries.len(), 3);
            assert_eq!(active.entries[2].item_title, "New Scale");
        } else {
            panic!("Expected Active state");
        }
        // Verify item was added to library
        assert_eq!(model.exercises.len(), 2); // original + new
    }

    // --- Summary Phase Tests ---

    fn model_with_summary() -> Model {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(30);
        let t2 = t1 + chrono::Duration::seconds(45);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );
        model
    }

    #[test]
    fn test_update_entry_notes() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id,
                notes: Some("Needs more practice".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].notes, Some("Needs more practice".to_string()));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_notes_too_long() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id,
                notes: Some("x".repeat(5001)),
            }),
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_update_session_notes() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionNotes {
                notes: Some("Great practice today".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.session_notes, Some("Great practice today".to_string()));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_save_session() {
        let mut model = model_with_summary();

        let now = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now }),
        );

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].total_duration_secs, 75); // 30 + 45
        assert_eq!(
            model.sessions[0].completion_status,
            CompletionStatus::Completed
        );
    }

    #[test]
    fn test_discard_session() {
        let mut model = model_with_summary();

        update(&mut model, Event::Session(SessionEvent::DiscardSession));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert!(model.sessions.is_empty());
    }

    // --- Recovery Tests ---

    #[test]
    fn test_recover_session() {
        let mut model = model_with_library();
        let now = Utc::now();

        let active = ActiveSession {
            id: "recovered-session".to_string(),
            entries: vec![create_entry("piece-1", "Moonlight Sonata", "piece", 0)],
            current_index: 0,
            current_item_started_at: now,
            session_started_at: now,
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RecoverSession { session: active }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.id, "recovered-session");
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_recover_session_when_not_idle() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        let now = Utc::now();
        let active = ActiveSession {
            id: "recovered".to_string(),
            entries: vec![],
            current_index: 0,
            current_item_started_at: now,
            session_started_at: now,
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RecoverSession { session: active }),
        );

        assert!(model.last_error.is_some());
    }

    // --- Delete Session Test ---

    #[test]
    fn test_delete_session() {
        let mut model = model_with_summary();
        let now = Utc::now();

        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now }),
        );

        let session_id = model.sessions[0].id.clone();
        update(
            &mut model,
            Event::Session(SessionEvent::DeleteSession { id: session_id }),
        );

        assert!(model.last_error.is_none());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_delete_session_not_found() {
        let mut model = model_with_library();

        update(
            &mut model,
            Event::Session(SessionEvent::DeleteSession {
                id: "nonexistent".to_string(),
            }),
        );

        assert!(model.last_error.is_some());
    }

    // --- Edge Case Tests ---

    #[test]
    fn test_all_items_skipped() {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(5);
        let t2 = t1 + chrono::Duration::seconds(5);

        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t2 }),
        );

        // Should be in summary state
        assert!(matches!(model.session_status, SessionStatus::Summary(_)));

        // Save it
        let save_time = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: save_time }),
        );

        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].total_duration_secs, 0);
    }

    #[test]
    fn test_single_item_setlist() {
        let (mut model, start) = model_with_active_session(1);
        let now = start + chrono::Duration::seconds(120);

        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries.len(), 1);
            assert_eq!(s.entries[0].duration_secs, 120);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_complete_lifecycle() {
        let mut model = model_with_library();
        let t0 = Utc::now();

        // 1. Start building
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        // 2. Add 3 items
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "exercise-1".to_string(),
            }),
        );

        // 3. Start session
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now: t0 }),
        );

        // 4. Practice first item for 30s, then Next
        let t1 = t0 + chrono::Duration::seconds(30);
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );

        // 5. Skip second item
        let t2 = t1 + chrono::Duration::seconds(5);
        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t2 }),
        );

        // 6. Finish on third item after 60s
        let t3 = t2 + chrono::Duration::seconds(60);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t3 }),
        );

        // 7. Add notes
        let entry_id_0 = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary");
        };
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id: entry_id_0,
                notes: Some("Good tempo control".to_string()),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionNotes {
                notes: Some("Focused session".to_string()),
            }),
        );

        // 8. Save
        let t_save = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: t_save }),
        );

        // Verify final state
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert_eq!(model.sessions.len(), 1);

        let session = &model.sessions[0];
        assert_eq!(session.entries.len(), 3);
        assert_eq!(session.entries[0].duration_secs, 30);
        assert_eq!(session.entries[0].status, EntryStatus::Completed);
        assert_eq!(
            session.entries[0].notes,
            Some("Good tempo control".to_string())
        );
        assert_eq!(session.entries[1].duration_secs, 0);
        assert_eq!(session.entries[1].status, EntryStatus::Skipped);
        assert_eq!(session.entries[2].duration_secs, 60);
        assert_eq!(session.entries[2].status, EntryStatus::Completed);
        assert_eq!(session.session_notes, Some("Focused session".to_string()));
        assert_eq!(session.total_duration_secs, 90); // 30 + 0 + 60
        assert_eq!(session.completion_status, CompletionStatus::Completed);
    }

    // --- format_duration_display Tests ---

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration_display(0), "0s");
        assert_eq!(format_duration_display(45), "45s");
        assert_eq!(format_duration_display(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration_display(60), "1m 0s");
        assert_eq!(format_duration_display(90), "1m 30s");
        assert_eq!(format_duration_display(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration_display(3600), "1h 0m 0s");
        assert_eq!(format_duration_display(3661), "1h 1m 1s");
        assert_eq!(format_duration_display(7200), "2h 0m 0s");
    }

    // --- SessionsData Serialization Test ---

    #[test]
    fn test_sessions_data_serialization() {
        use crate::domain::types::SessionsData;

        let data = SessionsData { sessions: vec![] };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: SessionsData = serde_json::from_str(&json).unwrap();
        assert!(parsed.sessions.is_empty());
    }

    // --- AddNewItemToSetlist Tests ---

    #[test]
    fn test_add_new_item_to_setlist_piece() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemToSetlist {
                title: "New Piece".to_string(),
                item_type: "piece".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        // Verify new item in library
        assert_eq!(model.pieces.len(), 3);
        assert_eq!(model.pieces[2].title, "New Piece");
        // Verify in setlist
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 1);
            assert_eq!(b.entries[0].item_title, "New Piece");
            assert_eq!(b.entries[0].item_type, "piece");
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_add_new_item_to_setlist_exercise() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemToSetlist {
                title: "New Exercise".to_string(),
                item_type: "exercise".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.exercises.len(), 2);
    }

    #[test]
    fn test_add_new_item_invalid_type() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemToSetlist {
                title: "Something".to_string(),
                item_type: "invalid".to_string(),
            }),
        );

        assert!(model.last_error.is_some());
    }
}

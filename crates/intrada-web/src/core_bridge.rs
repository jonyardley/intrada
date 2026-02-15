use std::cell::RefCell;

use crux_core::Core;
use leptos::prelude::{RwSignal, Set};

use intrada_core::{Effect, Event, Intrada, LibraryData, SessionsData, StorageEffect, ViewModel};

use crate::data::create_stub_data;

pub const STORAGE_KEY: &str = "intrada:library";
pub const SESSIONS_KEY: &str = "intrada:sessions";

thread_local! {
    static LIBRARY: RefCell<LibraryData> = RefCell::new(LibraryData::default());
    static SESSIONS: RefCell<SessionsData> = RefCell::new(SessionsData::default());
}

fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn load_from_local_storage() -> LibraryData {
    let Some(storage) = get_local_storage() else {
        return LibraryData::default();
    };

    match storage.get_item(STORAGE_KEY) {
        Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
        _ => LibraryData::default(),
    }
}

pub fn save_to_local_storage(data: &LibraryData) {
    let Some(storage) = get_local_storage() else {
        return;
    };

    match serde_json::to_string(data) {
        Ok(json) => {
            if storage.set_item(STORAGE_KEY, &json).is_err() {
                web_sys::console::warn_1(
                    &"intrada: localStorage write failed (storage may be full)".into(),
                );
            }
        }
        Err(e) => {
            web_sys::console::warn_1(
                &format!("intrada: failed to serialise library data: {e}").into(),
            );
        }
    }
}

fn load_sessions_from_local_storage() -> SessionsData {
    let Some(storage) = get_local_storage() else {
        return SessionsData::default();
    };

    match storage.get_item(SESSIONS_KEY) {
        Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
        _ => SessionsData::default(),
    }
}

pub fn save_sessions_to_local_storage(data: &SessionsData) {
    let Some(storage) = get_local_storage() else {
        return;
    };

    match serde_json::to_string(data) {
        Ok(json) => {
            if storage.set_item(SESSIONS_KEY, &json).is_err() {
                web_sys::console::warn_1(
                    &"intrada: localStorage write failed for sessions (storage may be full)".into(),
                );
            }
        }
        Err(e) => {
            web_sys::console::warn_1(
                &format!("intrada: failed to serialise sessions data: {e}").into(),
            );
        }
    }
}

/// Load sessions data from localStorage.
pub fn load_sessions_data() -> Vec<intrada_core::Session> {
    let data = load_sessions_from_local_storage();
    SESSIONS.with(|s| *s.borrow_mut() = data.clone());
    data.sessions
}

/// Load library data from localStorage (or seed with stub data on first run).
///
/// Called by `App()` during initialisation, mirroring the CLI shell's `load_data()`.
pub fn load_library_data() -> (Vec<intrada_core::Piece>, Vec<intrada_core::Exercise>) {
    let mut data = load_from_local_storage();

    // If localStorage was empty, seed with stub data
    if data.pieces.is_empty() && data.exercises.is_empty() {
        let (pieces, exercises) = create_stub_data();
        data.pieces = pieces;
        data.exercises = exercises;
        save_to_local_storage(&data);
    }

    LIBRARY.with(|lib| *lib.borrow_mut() = data.clone());
    (data.pieces, data.exercises)
}

/// Process effects returned by the Crux core.
pub fn process_effects(
    core: &Core<Intrada>,
    effects: Vec<Effect>,
    view_model: &RwSignal<ViewModel>,
) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {}
            Effect::Storage(boxed_request) => match &boxed_request.operation {
                StorageEffect::LoadAll => {
                    let (pieces, exercises) = load_library_data();
                    let inner_effects = core.process_event(Event::DataLoaded { pieces, exercises });
                    process_effects(core, inner_effects, view_model);
                }
                StorageEffect::SavePiece(piece) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        data.pieces.push(piece.clone());
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::SaveExercise(exercise) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        data.exercises.push(exercise.clone());
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::UpdatePiece(piece) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        if let Some(existing) = data.pieces.iter_mut().find(|p| p.id == piece.id) {
                            *existing = piece.clone();
                        }
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::UpdateExercise(exercise) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        if let Some(existing) =
                            data.exercises.iter_mut().find(|e| e.id == exercise.id)
                        {
                            *existing = exercise.clone();
                        }
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::DeleteItem { id } => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        data.pieces.retain(|p| p.id != *id);
                        data.exercises.retain(|e| e.id != *id);
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::LoadSessions => {
                    let sessions = load_sessions_data();
                    let inner_effects = core.process_event(Event::SessionsLoaded { sessions });
                    process_effects(core, inner_effects, view_model);
                }
                StorageEffect::SaveSession(session) => {
                    SESSIONS.with(|s| {
                        let mut data = s.borrow_mut();
                        data.sessions.push(session.clone());
                        save_sessions_to_local_storage(&data);
                    });
                }
                StorageEffect::UpdateSession(session) => {
                    SESSIONS.with(|s| {
                        let mut data = s.borrow_mut();
                        if let Some(existing) =
                            data.sessions.iter_mut().find(|sess| sess.id == session.id)
                        {
                            *existing = session.clone();
                        }
                        save_sessions_to_local_storage(&data);
                    });
                }
                StorageEffect::DeleteSession { id } => {
                    SESSIONS.with(|s| {
                        let mut data = s.borrow_mut();
                        data.sessions.retain(|sess| sess.id != *id);
                        save_sessions_to_local_storage(&data);
                    });
                }
            },
        }
    }
    view_model.set(core.view());
}

#[cfg(test)]
mod tests {
    use crux_core::Core;
    use intrada_core::{
        CreateExercise, CreatePiece, Effect, Event, ExerciseEvent, Intrada, LogSession, Piece,
        PieceEvent, SessionEvent, StorageEffect, UpdateSession,
    };

    /// Extract storage effects from a Vec<Effect>, skipping Render effects.
    fn storage_effects(effects: Vec<Effect>) -> Vec<StorageEffect> {
        effects
            .into_iter()
            .filter_map(|e| match e {
                Effect::Storage(boxed_req) => Some(boxed_req.operation.clone()),
                Effect::Render(_) => None,
            })
            .collect()
    }

    /// Create a core loaded with seed data so events can reference existing items.
    fn loaded_core() -> (Core<Intrada>, String) {
        let core = Core::<Intrada>::new();
        let now = chrono::Utc::now();
        let piece = Piece {
            id: "piece-1".to_string(),
            title: "Test Piece".to_string(),
            composer: "Test Composer".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        let _effects = core.process_event(Event::DataLoaded {
            pieces: vec![piece],
            exercises: vec![],
        });
        let _effects = core.process_event(Event::SessionsLoaded { sessions: vec![] });
        (core, "piece-1".to_string())
    }

    #[test]
    fn test_add_piece_produces_save_piece_effect() {
        let core = Core::<Intrada>::new();
        // Load empty data first
        let _ = core.process_event(Event::DataLoaded {
            pieces: vec![],
            exercises: vec![],
        });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let effects = core.process_event(Event::Piece(PieceEvent::Add(CreatePiece {
            title: "Moonlight Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::SavePiece(p) if p.title == "Moonlight Sonata")),
            "Expected SavePiece effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_add_exercise_produces_save_exercise_effect() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::DataLoaded {
            pieces: vec![],
            exercises: vec![],
        });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let effects = core.process_event(Event::Exercise(ExerciseEvent::Add(CreateExercise {
            title: "C Major Scale".to_string(),
            composer: None,
            category: Some("Scales".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let storage = storage_effects(effects);
        assert!(
            storage.iter().any(
                |e| matches!(e, StorageEffect::SaveExercise(ex) if ex.title == "C Major Scale")
            ),
            "Expected SaveExercise effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_delete_item_produces_delete_effect() {
        let (core, piece_id) = loaded_core();

        let effects = core.process_event(Event::Piece(PieceEvent::Delete {
            id: piece_id.clone(),
        }));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::DeleteItem { id } if id == &piece_id)),
            "Expected DeleteItem effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_log_session_produces_save_session_effect() {
        let (core, piece_id) = loaded_core();

        let effects = core.process_event(Event::Session(SessionEvent::Log(LogSession {
            item_id: piece_id,
            duration_minutes: 30,
            notes: Some("Good practice".to_string()),
        })));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::SaveSession(s) if s.duration_minutes == 30)),
            "Expected SaveSession effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_update_session_produces_update_session_effect() {
        let (core, piece_id) = loaded_core();

        // First log a session
        let effects = core.process_event(Event::Session(SessionEvent::Log(LogSession {
            item_id: piece_id,
            duration_minutes: 30,
            notes: None,
        })));
        let session_id = storage_effects(effects)
            .into_iter()
            .find_map(|e| match e {
                StorageEffect::SaveSession(s) => Some(s.id),
                _ => None,
            })
            .expect("Should have SaveSession effect");

        // Now update it
        let effects = core.process_event(Event::Session(SessionEvent::Update {
            id: session_id,
            input: UpdateSession {
                duration_minutes: Some(45),
                notes: None,
            },
        }));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::UpdateSession(s) if s.duration_minutes == 45)),
            "Expected UpdateSession effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_delete_session_produces_delete_session_effect() {
        let (core, piece_id) = loaded_core();

        // Log a session first
        let effects = core.process_event(Event::Session(SessionEvent::Log(LogSession {
            item_id: piece_id,
            duration_minutes: 20,
            notes: None,
        })));
        let session_id = storage_effects(effects)
            .into_iter()
            .find_map(|e| match e {
                StorageEffect::SaveSession(s) => Some(s.id),
                _ => None,
            })
            .expect("Should have SaveSession effect");

        // Delete it
        let effects = core.process_event(Event::Session(SessionEvent::Delete {
            id: session_id.clone(),
        }));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::DeleteSession { id } if id == &session_id)),
            "Expected DeleteSession effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_view_reflects_added_piece() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::DataLoaded {
            pieces: vec![],
            exercises: vec![],
        });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let vm_before = core.view();
        assert!(vm_before.items.is_empty());

        let _ = core.process_event(Event::Piece(PieceEvent::Add(CreatePiece {
            title: "Clair de Lune".to_string(),
            composer: "Debussy".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let vm_after = core.view();
        assert_eq!(vm_after.items.len(), 1);
        assert_eq!(vm_after.items[0].title, "Clair de Lune");
    }

    #[test]
    fn test_view_shows_error_on_validation_failure() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::DataLoaded {
            pieces: vec![],
            exercises: vec![],
        });

        // Empty title should trigger validation error
        let _ = core.process_event(Event::Piece(PieceEvent::Add(CreatePiece {
            title: "".to_string(),
            composer: "Someone".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let vm = core.view();
        assert!(vm.error.is_some(), "Expected validation error in ViewModel");
    }
}

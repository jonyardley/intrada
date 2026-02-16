use crux_core::Core;
use leptos::prelude::{RwSignal, Set};
use wasm_bindgen_futures::spawn_local;

use intrada_core::{Effect, Event, Intrada, StorageEffect, ViewModel};

use crate::api_client;
use crate::types::{IsLoading, IsSubmitting, SharedCore};

pub const SESSION_IN_PROGRESS_KEY: &str = "intrada:session-in-progress";

fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn save_session_in_progress(session: &intrada_core::ActiveSession) {
    let Some(storage) = get_local_storage() else {
        return;
    };

    match serde_json::to_string(session) {
        Ok(json) => {
            let _ = storage.set_item(SESSION_IN_PROGRESS_KEY, &json);
        }
        Err(e) => {
            web_sys::console::warn_1(
                &format!("intrada: failed to serialise session-in-progress: {e}").into(),
            );
        }
    }
}

fn clear_session_in_progress() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.remove_item(SESSION_IN_PROGRESS_KEY);
    }
}

/// Load the in-progress session from localStorage (for crash recovery).
pub fn load_session_in_progress() -> Option<intrada_core::ActiveSession> {
    let storage = get_local_storage()?;
    let json = storage.get_item(SESSION_IN_PROGRESS_KEY).ok()??;
    serde_json::from_str(&json).ok()
}

/// Fetch library and session data from the API on app startup.
///
/// Spawns two async tasks (library + sessions) that call the API and
/// dispatch the results into the Crux core.
pub fn fetch_initial_data(
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
) {
    // Fetch library data (pieces + exercises)
    {
        let core = leptos::prelude::expect_context::<SharedCore>();
        let vm = *view_model;
        let loading = *is_loading;
        let submitting = *is_submitting;
        spawn_local(async move {
            loading.set(true);
            let pieces_result = api_client::fetch_pieces().await;
            let exercises_result = api_client::fetch_exercises().await;

            match (pieces_result, exercises_result) {
                (Ok(pieces), Ok(exercises)) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
                    process_effects(&core_ref, effects, &vm, &loading, &submitting);
                }
                (Err(e), _) | (_, Err(e)) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::LoadFailed(e.to_user_message()));
                    process_effects(&core_ref, effects, &vm, &loading, &submitting);
                }
            }
            loading.set(false);
        });
    }

    // Fetch sessions
    {
        let core = leptos::prelude::expect_context::<SharedCore>();
        let vm = *view_model;
        let loading = *is_loading;
        let submitting = *is_submitting;
        spawn_local(async move {
            match api_client::fetch_sessions().await {
                Ok(sessions) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::SessionsLoaded { sessions });
                    process_effects(&core_ref, effects, &vm, &loading, &submitting);
                }
                Err(e) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::LoadFailed(e.to_user_message()));
                    process_effects(&core_ref, effects, &vm, &loading, &submitting);
                }
            }
        });
    }
}

/// Process effects returned by the Crux core.
///
/// HTTP-backed effects use `spawn_local()` to run async tasks.
/// Session-in-progress effects remain localStorage-based (FR-008).
pub fn process_effects(
    core: &Core<Intrada>,
    effects: Vec<Effect>,
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {}
            Effect::Storage(boxed_request) => match &boxed_request.operation {
                // ---- Load operations: spawn async HTTP fetch ----
                StorageEffect::LoadAll => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    spawn_local(async move {
                        loading.set(true);
                        let pieces_result = api_client::fetch_pieces().await;
                        let exercises_result = api_client::fetch_exercises().await;

                        match (pieces_result, exercises_result) {
                            (Ok(pieces), Ok(exercises)) => {
                                let core_ref = core.borrow();
                                let effects =
                                    core_ref.process_event(Event::DataLoaded { pieces, exercises });
                                process_effects(&core_ref, effects, &vm, &loading, &submitting);
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                let core_ref = core.borrow();
                                let effects =
                                    core_ref.process_event(Event::LoadFailed(e.to_user_message()));
                                process_effects(&core_ref, effects, &vm, &loading, &submitting);
                            }
                        }
                        loading.set(false);
                    });
                }

                StorageEffect::LoadSessions => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    spawn_local(async move {
                        loading.set(true);
                        match api_client::fetch_sessions().await {
                            Ok(sessions) => {
                                let core_ref = core.borrow();
                                let effects =
                                    core_ref.process_event(Event::SessionsLoaded { sessions });
                                process_effects(&core_ref, effects, &vm, &loading, &submitting);
                            }
                            Err(e) => {
                                let core_ref = core.borrow();
                                let effects =
                                    core_ref.process_event(Event::LoadFailed(e.to_user_message()));
                                process_effects(&core_ref, effects, &vm, &loading, &submitting);
                            }
                        }
                        loading.set(false);
                    });
                }

                // ---- Library write operations ----
                StorageEffect::SavePiece(piece) => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let create = intrada_core::CreatePiece {
                        title: piece.title.clone(),
                        composer: piece.composer.clone(),
                        key: piece.key.clone(),
                        tempo: piece.tempo.clone(),
                        notes: piece.notes.clone(),
                        tags: piece.tags.clone(),
                    };
                    submitting.set(true);
                    spawn_local(async move {
                        match api_client::create_piece(&create).await {
                            Ok(_) => refresh_library(core, vm, loading, submitting).await,
                            Err(e) => {
                                report_error(&core, &vm, &loading, &submitting, e);
                            }
                        }
                    });
                }

                StorageEffect::SaveExercise(exercise) => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let create = intrada_core::CreateExercise {
                        title: exercise.title.clone(),
                        composer: exercise.composer.clone(),
                        category: exercise.category.clone(),
                        key: exercise.key.clone(),
                        tempo: exercise.tempo.clone(),
                        notes: exercise.notes.clone(),
                        tags: exercise.tags.clone(),
                    };
                    submitting.set(true);
                    spawn_local(async move {
                        match api_client::create_exercise(&create).await {
                            Ok(_) => refresh_library(core, vm, loading, submitting).await,
                            Err(e) => {
                                report_error(&core, &vm, &loading, &submitting, e);
                            }
                        }
                    });
                }

                StorageEffect::UpdatePiece(piece) => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let piece_id = piece.id.clone();
                    let update = intrada_core::UpdatePiece {
                        title: Some(piece.title.clone()),
                        composer: Some(piece.composer.clone()),
                        key: Some(piece.key.clone()),
                        tempo: Some(piece.tempo.clone()),
                        notes: Some(piece.notes.clone()),
                        tags: Some(piece.tags.clone()),
                    };
                    submitting.set(true);
                    spawn_local(async move {
                        match api_client::update_piece(&piece_id, &update).await {
                            Ok(_) => refresh_library(core, vm, loading, submitting).await,
                            Err(e) => {
                                report_error(&core, &vm, &loading, &submitting, e);
                            }
                        }
                    });
                }

                StorageEffect::UpdateExercise(exercise) => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let exercise_id = exercise.id.clone();
                    let update = intrada_core::UpdateExercise {
                        title: Some(exercise.title.clone()),
                        composer: Some(exercise.composer.clone()),
                        category: Some(exercise.category.clone()),
                        key: Some(exercise.key.clone()),
                        tempo: Some(exercise.tempo.clone()),
                        notes: Some(exercise.notes.clone()),
                        tags: Some(exercise.tags.clone()),
                    };
                    submitting.set(true);
                    spawn_local(async move {
                        match api_client::update_exercise(&exercise_id, &update).await {
                            Ok(_) => refresh_library(core, vm, loading, submitting).await,
                            Err(e) => {
                                report_error(&core, &vm, &loading, &submitting, e);
                            }
                        }
                    });
                }

                StorageEffect::DeleteItem { id } => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let item_id = id.clone();
                    submitting.set(true);
                    spawn_local(async move {
                        // Try deleting as piece first, then as exercise
                        // (DeleteItem doesn't carry the item type)
                        let piece_result = api_client::delete_piece(&item_id).await;
                        if piece_result.is_err() {
                            if let Err(e) = api_client::delete_exercise(&item_id).await {
                                report_error(&core, &vm, &loading, &submitting, e);
                                return;
                            }
                        }
                        refresh_library(core, vm, loading, submitting).await;
                    });
                }

                // ---- Session write operations ----
                StorageEffect::SavePracticeSession(session) => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let session_data = session.clone();
                    // Clear in-progress from localStorage immediately (FR-008)
                    clear_session_in_progress();
                    submitting.set(true);
                    spawn_local(async move {
                        match api_client::create_session(&session_data).await {
                            Ok(_) => refresh_sessions(core, vm, loading, submitting).await,
                            Err(e) => {
                                report_error(&core, &vm, &loading, &submitting, e);
                            }
                        }
                    });
                }

                StorageEffect::DeletePracticeSession { id } => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    let session_id = id.clone();
                    submitting.set(true);
                    spawn_local(async move {
                        match api_client::delete_session(&session_id).await {
                            Ok(_) => refresh_sessions(core, vm, loading, submitting).await,
                            Err(e) => {
                                report_error(&core, &vm, &loading, &submitting, e);
                            }
                        }
                    });
                }

                // ---- Session-in-progress: localStorage only (FR-008) ----
                StorageEffect::SaveSessionInProgress(session) => {
                    save_session_in_progress(session);
                }
                StorageEffect::ClearSessionInProgress => {
                    clear_session_in_progress();
                }
            },
        }
    }
    view_model.set(core.view());
}

/// Refresh library data from API after a mutation (refresh-after-mutate pattern).
async fn refresh_library(
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
    is_loading: IsLoading,
    is_submitting: IsSubmitting,
) {
    let pieces_result = api_client::fetch_pieces().await;
    let exercises_result = api_client::fetch_exercises().await;

    match (pieces_result, exercises_result) {
        (Ok(pieces), Ok(exercises)) => {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
        (Err(e), _) | (_, Err(e)) => {
            report_error(&core, &view_model, &is_loading, &is_submitting, e);
        }
    }

    is_submitting.set(false);
}

/// Refresh sessions data from API after a mutation.
async fn refresh_sessions(
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
    is_loading: IsLoading,
    is_submitting: IsSubmitting,
) {
    match api_client::fetch_sessions().await {
        Ok(sessions) => {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::SessionsLoaded { sessions });
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
        Err(e) => {
            report_error(&core, &view_model, &is_loading, &is_submitting, e);
        }
    }

    is_submitting.set(false);
}

/// Report an API error to the core via LoadFailed event.
fn report_error(
    core: &SharedCore,
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
    error: api_client::ApiError,
) {
    let core_ref = core.borrow();
    let effects = core_ref.process_event(Event::LoadFailed(error.to_user_message()));
    process_effects(&core_ref, effects, view_model, is_loading, is_submitting);
    is_submitting.set(false);
}

#[cfg(test)]
mod tests {
    use crux_core::Core;
    use intrada_core::{
        CreateExercise, CreatePiece, Effect, Event, ExerciseEvent, Intrada, Piece, PieceEvent,
        StorageEffect,
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
    fn test_session_building_and_start() {
        use intrada_core::SessionEvent;

        let (core, piece_id) = loaded_core();

        let effects = core.process_event(Event::Session(SessionEvent::StartBuilding));
        let storage = storage_effects(effects);
        assert!(
            storage.is_empty(),
            "Expected no storage effects for StartBuilding"
        );

        let effects = core.process_event(Event::Session(SessionEvent::AddToSetlist {
            item_id: piece_id,
        }));
        let storage = storage_effects(effects);
        assert!(
            storage.is_empty(),
            "Expected no storage effects for AddToSetlist"
        );

        let now = chrono::Utc::now();
        let effects = core.process_event(Event::Session(SessionEvent::StartSession { now }));
        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::SaveSessionInProgress(_))),
            "Expected SaveSessionInProgress effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_delete_session_produces_delete_practice_session_effect() {
        use intrada_core::SessionEvent;

        let (core, piece_id) = loaded_core();

        let _ = core.process_event(Event::Session(SessionEvent::StartBuilding));
        let _ = core.process_event(Event::Session(SessionEvent::AddToSetlist {
            item_id: piece_id,
        }));
        let now = chrono::Utc::now();
        let _ = core.process_event(Event::Session(SessionEvent::StartSession { now }));
        let later = now + chrono::Duration::minutes(10);
        let _ = core.process_event(Event::Session(SessionEvent::FinishSession { now: later }));

        let save_now = later + chrono::Duration::seconds(5);
        let effects =
            core.process_event(Event::Session(SessionEvent::SaveSession { now: save_now }));
        let storage = storage_effects(effects);
        let session_id = storage.iter().find_map(|e| match e {
            StorageEffect::SavePracticeSession(s) => Some(s.id.clone()),
            _ => None,
        });
        assert!(session_id.is_some(), "Expected SavePracticeSession effect");

        let effects = core.process_event(Event::Session(SessionEvent::DeleteSession {
            id: session_id.unwrap(),
        }));
        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, StorageEffect::DeletePracticeSession { .. })),
            "Expected DeletePracticeSession effect, got: {storage:?}"
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

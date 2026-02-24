use crux_core::Core;
use leptos::prelude::{RwSignal, Set};
use std::future::Future;
use wasm_bindgen_futures::spawn_local;

use intrada_core::domain::goal::{Goal, GoalStatus};
use intrada_core::{AppEffect, Effect, Event, Intrada, Routine, ViewModel};

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
    // Fetch library data (items)
    {
        let core = leptos::prelude::expect_context::<SharedCore>();
        let vm = *view_model;
        let loading = *is_loading;
        let submitting = *is_submitting;
        spawn_local(async move {
            loading.set(true);
            match api_client::fetch_items().await {
                Ok(items) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::DataLoaded { items });
                    process_effects(&core_ref, effects, &vm, &loading, &submitting);
                }
                Err(e) => {
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

    // Fetch routines
    {
        let core = leptos::prelude::expect_context::<SharedCore>();
        let vm = *view_model;
        let loading = *is_loading;
        let submitting = *is_submitting;
        spawn_local(async move {
            match api_client::fetch_routines().await {
                Ok(routines) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::RoutinesLoaded { routines });
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

    // Fetch goals
    {
        let core = leptos::prelude::expect_context::<SharedCore>();
        let vm = *view_model;
        let loading = *is_loading;
        let submitting = *is_submitting;
        spawn_local(async move {
            match api_client::fetch_goals().await {
                Ok(goals) => {
                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::GoalsLoaded { goals });
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
            Effect::App(boxed_request) => match &boxed_request.operation {
                // ---- Load operations: spawn async HTTP fetch ----
                AppEffect::LoadAll => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    spawn_local(async move {
                        loading.set(true);
                        match api_client::fetch_items().await {
                            Ok(items) => {
                                let core_ref = core.borrow();
                                let effects = core_ref.process_event(Event::DataLoaded { items });
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

                AppEffect::LoadSessions => {
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
                AppEffect::SaveItem(item) => {
                    let create = intrada_core::CreateItem {
                        title: item.title.clone(),
                        kind: item.kind.clone(),
                        composer: item.composer.clone(),
                        category: item.category.clone(),
                        key: item.key.clone(),
                        tempo: item.tempo.clone(),
                        notes: item.notes.clone(),
                        tags: item.tags.clone(),
                    };
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::create_item(&create).await },
                        RefreshKind::Library,
                    );
                }

                AppEffect::UpdateItem(item) => {
                    let item_id = item.id.clone();
                    let update = intrada_core::UpdateItem {
                        title: Some(item.title.clone()),
                        composer: Some(item.composer.clone()),
                        category: Some(item.category.clone()),
                        key: Some(item.key.clone()),
                        tempo: Some(item.tempo.clone()),
                        notes: Some(item.notes.clone()),
                        tags: Some(item.tags.clone()),
                    };
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::update_item(&item_id, &update).await },
                        RefreshKind::Library,
                    );
                }

                AppEffect::DeleteItem { id } => {
                    let item_id = id.clone();
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::delete_item(&item_id).await },
                        RefreshKind::Library,
                    );
                }

                // ---- Session write operations ----
                AppEffect::SavePracticeSession(session) => {
                    let session_data = session.clone();
                    // Clear in-progress from localStorage immediately (FR-008)
                    clear_session_in_progress();
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::create_session(&session_data).await },
                        RefreshKind::Sessions,
                    );
                }

                AppEffect::DeletePracticeSession { id } => {
                    let session_id = id.clone();
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::delete_session(&session_id).await },
                        RefreshKind::Sessions,
                    );
                }

                // ---- Session-in-progress: localStorage only (FR-008) ----
                AppEffect::SaveSessionInProgress(session) => {
                    save_session_in_progress(session);
                }
                AppEffect::ClearSessionInProgress => {
                    clear_session_in_progress();
                }

                // ---- Routine operations ----
                AppEffect::SaveRoutine(routine) => {
                    let create = build_create_routine_request(routine);
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::create_routine(&create).await },
                        RefreshKind::Routines,
                    );
                }

                AppEffect::UpdateRoutine(routine) => {
                    let routine_id = routine.id.clone();
                    let update = build_update_routine_request(routine);
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::update_routine(&routine_id, &update).await },
                        RefreshKind::Routines,
                    );
                }

                AppEffect::DeleteRoutine { id } => {
                    let routine_id = id.clone();
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::delete_routine(&routine_id).await },
                        RefreshKind::Routines,
                    );
                }

                // ---- Goal operations ----
                AppEffect::SaveGoal(goal) => {
                    let create = intrada_core::domain::types::CreateGoal {
                        title: goal.title.clone(),
                        kind: goal.kind.clone(),
                        deadline: goal.deadline,
                    };
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::create_goal(&create).await },
                        RefreshKind::Goals,
                    );
                }

                AppEffect::UpdateGoal(goal) => {
                    let goal_id = goal.id.clone();
                    let update = build_update_goal_request(goal);
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::update_goal(&goal_id, &update).await },
                        RefreshKind::Goals,
                    );
                }

                AppEffect::DeleteGoal { id } => {
                    let goal_id = id.clone();
                    spawn_mutate(
                        view_model,
                        is_loading,
                        is_submitting,
                        async move { api_client::delete_goal(&goal_id).await },
                        RefreshKind::Goals,
                    );
                }

                AppEffect::LoadGoals => {
                    let core = leptos::prelude::expect_context::<SharedCore>();
                    let vm = *view_model;
                    let loading = *is_loading;
                    let submitting = *is_submitting;
                    spawn_local(async move {
                        loading.set(true);
                        match api_client::fetch_goals().await {
                            Ok(goals) => {
                                let core_ref = core.borrow();
                                let effects = core_ref.process_event(Event::GoalsLoaded { goals });
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
            },
        }
    }
    view_model.set(core.view());
}

/// Which data to re-fetch from the API after a write operation.
#[derive(Clone, Copy)]
enum RefreshKind {
    Library,
    Sessions,
    Routines,
    Goals,
}

/// Spawn a mutating API call followed by a data refresh.
///
/// Encapsulates the "refresh-after-mutate" pattern used by all write operations:
/// set submitting → run API call → report error (if any) → re-fetch from API.
fn spawn_mutate<T, Fut>(
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
    api_call: Fut,
    kind: RefreshKind,
) where
    T: 'static,
    Fut: Future<Output = Result<T, api_client::ApiError>> + 'static,
{
    let core = leptos::prelude::expect_context::<SharedCore>();
    let vm = *view_model;
    let loading = *is_loading;
    let submitting = *is_submitting;
    submitting.set(true);
    spawn_local(async move {
        if let Err(e) = api_call.await {
            report_error(&core, &vm, &loading, &submitting, e);
        }
        match kind {
            RefreshKind::Library => refresh_library(core, vm, loading, submitting).await,
            RefreshKind::Sessions => refresh_sessions(core, vm, loading, submitting).await,
            RefreshKind::Routines => refresh_routines(core, vm, loading, submitting).await,
            RefreshKind::Goals => refresh_goals(core, vm, loading, submitting).await,
        }
    });
}

/// Refresh library data from API after a mutation (refresh-after-mutate pattern).
async fn refresh_library(
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
    is_loading: IsLoading,
    is_submitting: IsSubmitting,
) {
    match api_client::fetch_items().await {
        Ok(items) => {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::DataLoaded { items });
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
        Err(e) => {
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

/// Refresh routines data from API after a mutation.
async fn refresh_routines(
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
    is_loading: IsLoading,
    is_submitting: IsSubmitting,
) {
    match api_client::fetch_routines().await {
        Ok(routines) => {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::RoutinesLoaded { routines });
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
        Err(e) => {
            report_error(&core, &view_model, &is_loading, &is_submitting, e);
        }
    }

    is_submitting.set(false);
}

/// Refresh goals data from API after a mutation.
async fn refresh_goals(
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
    is_loading: IsLoading,
    is_submitting: IsSubmitting,
) {
    match api_client::fetch_goals().await {
        Ok(goals) => {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::GoalsLoaded { goals });
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
        Err(e) => {
            report_error(&core, &view_model, &is_loading, &is_submitting, e);
        }
    }

    is_submitting.set(false);
}

/// Build an UpdateGoalApiRequest from a domain Goal.
///
/// Converts the core Goal (with typed GoalStatus enum) into the API's string-based
/// update DTO that includes title, status, and deadline.
fn build_update_goal_request(goal: &Goal) -> api_client::UpdateGoalApiRequest {
    let status_str = match goal.status {
        GoalStatus::Active => "active",
        GoalStatus::Completed => "completed",
        GoalStatus::Archived => "archived",
    };
    api_client::UpdateGoalApiRequest {
        title: Some(goal.title.clone()),
        status: Some(status_str.to_string()),
        deadline: Some(goal.deadline),
    }
}

/// Build a CreateRoutineApiRequest from a domain Routine.
fn build_create_routine_request(routine: &Routine) -> api_client::CreateRoutineApiRequest {
    api_client::CreateRoutineApiRequest {
        name: routine.name.clone(),
        entries: routine
            .entries
            .iter()
            .map(|e| api_client::CreateRoutineEntryApiRequest {
                item_id: e.item_id.clone(),
                item_title: e.item_title.clone(),
                item_type: e.item_type.clone(),
            })
            .collect(),
    }
}

/// Build an UpdateRoutineApiRequest from a domain Routine.
fn build_update_routine_request(routine: &Routine) -> api_client::UpdateRoutineApiRequest {
    api_client::UpdateRoutineApiRequest {
        name: routine.name.clone(),
        entries: routine
            .entries
            .iter()
            .map(|e| api_client::CreateRoutineEntryApiRequest {
                item_id: e.item_id.clone(),
                item_title: e.item_title.clone(),
                item_type: e.item_type.clone(),
            })
            .collect(),
    }
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
    use intrada_core::{AppEffect, CreateItem, Effect, Event, Intrada, Item, ItemEvent, ItemKind};

    /// Extract storage effects from a Vec<Effect>, skipping Render effects.
    fn storage_effects(effects: Vec<Effect>) -> Vec<AppEffect> {
        effects
            .into_iter()
            .filter_map(|e| match e {
                Effect::App(boxed_req) => Some(boxed_req.operation.clone()),
                Effect::Render(_) => None,
            })
            .collect()
    }

    /// Create a core loaded with seed data so events can reference existing items.
    fn loaded_core() -> (Core<Intrada>, String) {
        let core = Core::<Intrada>::new();
        let now = chrono::Utc::now();
        let item = Item {
            id: "item-1".to_string(),
            title: "Test Piece".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Test Composer".to_string()),
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        let _effects = core.process_event(Event::DataLoaded { items: vec![item] });
        let _effects = core.process_event(Event::SessionsLoaded { sessions: vec![] });
        (core, "item-1".to_string())
    }

    #[test]
    fn test_add_piece_produces_save_item_effect() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::DataLoaded { items: vec![] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let effects = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, AppEffect::SaveItem(i) if i.title == "Moonlight Sonata")),
            "Expected SaveItem effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_add_exercise_produces_save_item_effect() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::DataLoaded { items: vec![] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let effects = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "C Major Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            category: Some("Scales".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, AppEffect::SaveItem(i) if i.title == "C Major Scale")),
            "Expected SaveItem effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_delete_item_produces_delete_effect() {
        let (core, item_id) = loaded_core();

        let effects = core.process_event(Event::Item(ItemEvent::Delete {
            id: item_id.clone(),
        }));

        let storage = storage_effects(effects);
        assert!(
            storage
                .iter()
                .any(|e| matches!(e, AppEffect::DeleteItem { id } if id == &item_id)),
            "Expected DeleteItem effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_session_building_and_start() {
        use intrada_core::SessionEvent;

        let (core, item_id) = loaded_core();

        let effects = core.process_event(Event::Session(SessionEvent::StartBuilding));
        let storage = storage_effects(effects);
        assert!(
            storage.is_empty(),
            "Expected no storage effects for StartBuilding"
        );

        let effects = core.process_event(Event::Session(SessionEvent::AddToSetlist { item_id }));
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
                .any(|e| matches!(e, AppEffect::SaveSessionInProgress(_))),
            "Expected SaveSessionInProgress effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_delete_session_produces_delete_practice_session_effect() {
        use intrada_core::SessionEvent;

        let (core, item_id) = loaded_core();

        let _ = core.process_event(Event::Session(SessionEvent::StartBuilding));
        let _ = core.process_event(Event::Session(SessionEvent::AddToSetlist { item_id }));
        let now = chrono::Utc::now();
        let _ = core.process_event(Event::Session(SessionEvent::StartSession { now }));
        let later = now + chrono::Duration::minutes(10);
        let _ = core.process_event(Event::Session(SessionEvent::FinishSession { now: later }));

        let save_now = later + chrono::Duration::seconds(5);
        let effects =
            core.process_event(Event::Session(SessionEvent::SaveSession { now: save_now }));
        let storage = storage_effects(effects);
        let session_id = storage.iter().find_map(|e| match e {
            AppEffect::SavePracticeSession(s) => Some(s.id.clone()),
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
                .any(|e| matches!(e, AppEffect::DeletePracticeSession { .. })),
            "Expected DeletePracticeSession effect, got: {storage:?}"
        );
    }

    #[test]
    fn test_view_reflects_added_item() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::DataLoaded { items: vec![] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let vm_before = core.view();
        assert!(vm_before.items.is_empty());

        let _ = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "Clair de Lune".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            category: None,
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
        let _ = core.process_event(Event::DataLoaded { items: vec![] });

        let _ = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Someone".to_string()),
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let vm = core.view();
        assert!(vm.error.is_some(), "Expected validation error in ViewModel");
    }
}

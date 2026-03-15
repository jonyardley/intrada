//! Bridge between the Crux core and the web shell.
//!
//! Processes effects returned by the core, executing HTTP requests via gloo-net
//! and localStorage operations for crash recovery.

use crux_core::Core;
use leptos::prelude::{RwSignal, Set};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

use intrada_core::{AppEffect, Effect, Event, Intrada, ViewModel};

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

// ── Core initialisation ──────────────────────────────────────────────────

/// Initialise the Crux core and fetch all data from the API.
///
/// Sends `Event::StartApp` with the API base URL, which makes the core produce
/// HTTP fetch effects for items, sessions, and routines. A shared
/// counter keeps `is_loading` true until ALL fetches complete.
pub fn init_core(
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
) {
    is_loading.set(true);

    let core = leptos::prelude::expect_context::<SharedCore>();

    // Send Init and capture effects, then drop the borrow so spawn_local
    // closures can re-borrow later.
    let effects = {
        let core_ref = core.borrow();
        let effects = core_ref.process_event(Event::StartApp {
            api_base_url: api_client::API_BASE_URL.to_string(),
        });
        view_model.set(core_ref.view());
        effects
    };

    // Count HTTP effects for the loading counter.
    let http_count = effects
        .iter()
        .filter(|e| matches!(e, Effect::Http(_)))
        .count() as u32;
    if http_count == 0 {
        is_loading.set(false);
        return;
    }

    let remaining = Rc::new(Cell::new(http_count));
    let loading_signal = *is_loading;

    for effect in effects {
        match effect {
            Effect::Render(_) => {}
            Effect::Http(mut request) => {
                let core_handle = leptos::prelude::expect_context::<SharedCore>();
                let vm = *view_model;
                let loading = *is_loading;
                let submitting = *is_submitting;
                let remaining = Rc::clone(&remaining);
                spawn_local(async move {
                    let result = execute_http(&request.operation).await;
                    let core_ref = core_handle.borrow();
                    match core_ref.resolve(&mut request, result) {
                        Ok(new_effects) => {
                            process_effects_inner(new_effects, &vm, &loading, &submitting);
                            vm.set(core_ref.view());
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("resolve error: {e:?}").into());
                        }
                    }
                    let n = remaining.get().saturating_sub(1);
                    remaining.set(n);
                    if n == 0 {
                        loading_signal.set(false);
                    }
                });
            }
            Effect::App(request) => {
                handle_app_effect(&request.operation);
            }
        }
    }
}

// ── Effect processing ────────────────────────────────────────────────────

/// Process effects returned by the Crux core.
///
/// Called by views after `core.process_event(...)`. Spawns async tasks for
/// HTTP effects and handles localStorage operations synchronously.
///
/// Automatically sets `is_submitting` when HTTP effects are present and
/// the app is not in the initial loading state.
pub fn process_effects(
    core: &Core<Intrada>,
    effects: Vec<Effect>,
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
) {
    // If HTTP effects are present and we're past initial load, mark submitting.
    let has_http = effects.iter().any(|e| matches!(e, Effect::Http(_)));
    if has_http && !is_loading.get_untracked() {
        is_submitting.set(true);
    }

    process_effects_inner(effects, view_model, is_loading, is_submitting);
    view_model.set(core.view());
}

/// Internal effect processor. Spawns async tasks for HTTP effects.
///
/// Does NOT call `view_model.set(core.view())` — callers are responsible
/// for updating the view model after this returns.
fn process_effects_inner(
    effects: Vec<Effect>,
    view_model: &RwSignal<ViewModel>,
    is_loading: &IsLoading,
    is_submitting: &IsSubmitting,
) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {}
            Effect::Http(mut request) => {
                let core = leptos::prelude::expect_context::<SharedCore>();
                let vm = *view_model;
                let loading = *is_loading;
                let submitting = *is_submitting;
                spawn_local(async move {
                    let result = execute_http(&request.operation).await;
                    let core_ref = core.borrow();
                    match core_ref.resolve(&mut request, result) {
                        Ok(new_effects) => {
                            let has_more_http =
                                new_effects.iter().any(|e| matches!(e, Effect::Http(_)));
                            process_effects_inner(new_effects, &vm, &loading, &submitting);
                            vm.set(core_ref.view());
                            // Clear submitting once the HTTP chain ends
                            // (but not if we're still in initial loading).
                            if !has_more_http && !loading.get_untracked() {
                                submitting.set(false);
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("resolve error: {e:?}").into());
                            if !loading.get_untracked() {
                                submitting.set(false);
                            }
                        }
                    }
                });
            }
            Effect::App(request) => {
                handle_app_effect(&request.operation);
            }
        }
    }
}

/// Handle a non-HTTP shell effect (localStorage only).
fn handle_app_effect(effect: &AppEffect) {
    match effect {
        AppEffect::SaveSessionInProgress(session) => save_session_in_progress(session),
        AppEffect::ClearSessionInProgress => clear_session_in_progress(),
    }
}

// ── HTTP execution ───────────────────────────────────────────────────────

/// Execute an HTTP request and return a `HttpResult` for `core.resolve()`.
async fn execute_http(request: &intrada_core::HttpRequest) -> intrada_core::HttpResult {
    match send_with_retry(request).await {
        Ok(response) => intrada_core::HttpResult::Ok(response),
        Err(error) => intrada_core::HttpResult::Err(error),
    }
}

/// Send an HTTP request, retrying once with a fresh Clerk token on 401.
async fn send_with_retry(
    request: &intrada_core::HttpRequest,
) -> Result<intrada_core::HttpResponse, intrada_core::HttpError> {
    let response = send_once(request).await?;
    if response.status == 401 {
        // Retry with fresh auth token
        return send_once(request).await;
    }
    Ok(response)
}

/// Send a single HTTP request via gloo-net with Clerk auth.
async fn send_once(
    request: &intrada_core::HttpRequest,
) -> Result<intrada_core::HttpResponse, intrada_core::HttpError> {
    let mut builder = match request.method.as_str() {
        "GET" => gloo_net::http::Request::get(&request.url),
        "POST" => gloo_net::http::Request::post(&request.url),
        "PUT" => gloo_net::http::Request::put(&request.url),
        "DELETE" => gloo_net::http::Request::delete(&request.url),
        "PATCH" => gloo_net::http::Request::patch(&request.url),
        other => {
            return Err(intrada_core::HttpError::Io(format!(
                "unsupported HTTP method: {other}"
            )))
        }
    };

    // Forward headers from core, skipping Content-Type (let .json() set it).
    for header in &request.headers {
        if !header.name.eq_ignore_ascii_case("content-type") {
            builder = builder.header(&header.name, &header.value);
        }
    }

    // Add Clerk auth header.
    if let Some(auth) = api_client::auth_header_value().await {
        builder = builder.header("Authorization", &auth);
    }

    // Send with or without JSON body.
    let gloo_response = if !request.body.is_empty() {
        let json: serde_json::Value = serde_json::from_slice(&request.body)
            .map_err(|e| intrada_core::HttpError::Io(e.to_string()))?;
        builder
            .json(&json)
            .map_err(|e| intrada_core::HttpError::Io(e.to_string()))?
            .send()
            .await
            .map_err(|e| intrada_core::HttpError::Io(e.to_string()))?
    } else {
        builder
            .send()
            .await
            .map_err(|e| intrada_core::HttpError::Io(e.to_string()))?
    };

    let status = gloo_response.status();
    let body_bytes = gloo_response
        .binary()
        .await
        .map_err(|e| intrada_core::HttpError::Io(e.to_string()))?;

    Ok(intrada_core::HttpResponse {
        status,
        headers: vec![],
        body: body_bytes,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crux_core::Core;
    use intrada_core::{
        AppEffect, CreateItem, Effect, Event, HttpRequest, Intrada, Item, ItemEvent, ItemKind,
    };

    /// Extract HTTP request operations from a list of effects.
    fn http_effects(effects: &[Effect]) -> Vec<&HttpRequest> {
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::Http(req) => Some(&req.operation),
                _ => None,
            })
            .collect()
    }

    /// Extract app (localStorage) effects from a list of effects.
    fn app_effects(effects: &[Effect]) -> Vec<&AppEffect> {
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::App(req) => Some(&req.operation),
                _ => None,
            })
            .collect()
    }

    /// Create a core with API URL set and seed data loaded.
    fn loaded_core() -> (Core<Intrada>, String) {
        let core = Core::<Intrada>::new();
        // Set API base URL via Init (ignore the 4 HTTP fetch effects)
        let _ = core.process_event(Event::StartApp {
            api_base_url: "http://localhost:3001".to_string(),
        });
        let now = chrono::Utc::now();
        let item = Item {
            id: "item-1".to_string(),
            title: "Test Piece".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Test Composer".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        let _ = core.process_event(Event::DataLoaded { items: vec![item] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });
        (core, "item-1".to_string())
    }

    #[test]
    fn test_add_piece_produces_http_post() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::StartApp {
            api_base_url: "http://localhost:3001".to_string(),
        });
        let _ = core.process_event(Event::DataLoaded { items: vec![] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let effects = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let http = http_effects(&effects);
        assert!(
            http.iter()
                .any(|r| r.method == "POST" && r.url.contains("/api/items")),
            "Expected HTTP POST to /api/items, got: {http:?}"
        );
    }

    #[test]
    fn test_add_exercise_produces_http_post() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::StartApp {
            api_base_url: "http://localhost:3001".to_string(),
        });
        let _ = core.process_event(Event::DataLoaded { items: vec![] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let effects = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "C Major Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let http = http_effects(&effects);
        assert!(
            http.iter()
                .any(|r| r.method == "POST" && r.url.contains("/api/items")),
            "Expected HTTP POST to /api/items, got: {http:?}"
        );
    }

    #[test]
    fn test_delete_item_produces_http_delete() {
        let (core, item_id) = loaded_core();

        let effects = core.process_event(Event::Item(ItemEvent::Delete {
            id: item_id.clone(),
        }));

        let http = http_effects(&effects);
        assert!(
            http.iter().any(|r| r.method == "DELETE"
                && r.url.contains("/api/items/")
                && r.url.contains(&item_id)),
            "Expected HTTP DELETE to /api/items/{item_id}, got: {http:?}"
        );
    }

    #[test]
    fn test_session_building_and_start() {
        use intrada_core::SessionEvent;

        let (core, item_id) = loaded_core();

        let effects = core.process_event(Event::Session(SessionEvent::StartBuilding));
        let app = app_effects(&effects);
        assert!(app.is_empty(), "Expected no app effects for StartBuilding");

        let effects = core.process_event(Event::Session(SessionEvent::AddToSetlist { item_id }));
        let app = app_effects(&effects);
        assert!(app.is_empty(), "Expected no app effects for AddToSetlist");

        let now = chrono::Utc::now();
        let effects = core.process_event(Event::Session(SessionEvent::StartSession { now }));
        let app = app_effects(&effects);
        assert!(
            app.iter()
                .any(|e| matches!(e, AppEffect::SaveSessionInProgress(_))),
            "Expected SaveSessionInProgress effect, got: {app:?}"
        );
    }

    #[test]
    fn test_save_session_produces_http_post_and_clear() {
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

        let http = http_effects(&effects);
        assert!(
            http.iter()
                .any(|r| r.method == "POST" && r.url.contains("/api/sessions")),
            "Expected HTTP POST to /api/sessions, got: {http:?}"
        );

        let app = app_effects(&effects);
        assert!(
            app.iter()
                .any(|e| matches!(e, AppEffect::ClearSessionInProgress)),
            "Expected ClearSessionInProgress effect, got: {app:?}"
        );
    }

    #[test]
    fn test_delete_session_produces_http_delete() {
        use intrada_core::SessionEvent;

        let (core, item_id) = loaded_core();

        let _ = core.process_event(Event::Session(SessionEvent::StartBuilding));
        let _ = core.process_event(Event::Session(SessionEvent::AddToSetlist { item_id }));
        let now = chrono::Utc::now();
        let _ = core.process_event(Event::Session(SessionEvent::StartSession { now }));
        let later = now + chrono::Duration::minutes(10);
        let _ = core.process_event(Event::Session(SessionEvent::FinishSession { now: later }));

        let save_now = later + chrono::Duration::seconds(5);
        let save_effects =
            core.process_event(Event::Session(SessionEvent::SaveSession { now: save_now }));

        // Extract session ID from the HTTP POST body
        let session_id = http_effects(&save_effects)
            .iter()
            .find(|r| r.method == "POST" && r.url.contains("/api/sessions"))
            .and_then(|r| serde_json::from_slice::<intrada_core::PracticeSession>(&r.body).ok())
            .map(|s| s.id)
            .expect("Expected session POST with deserializable body");

        let effects = core.process_event(Event::Session(SessionEvent::DeleteSession {
            id: session_id.clone(),
        }));

        let http = http_effects(&effects);
        assert!(
            http.iter().any(|r| r.method == "DELETE"
                && r.url.contains("/api/sessions/")
                && r.url.contains(&session_id)),
            "Expected HTTP DELETE to /api/sessions/{session_id}, got: {http:?}"
        );
    }

    #[test]
    fn test_view_reflects_added_item() {
        let core = Core::<Intrada>::new();
        let _ = core.process_event(Event::StartApp {
            api_base_url: "http://localhost:3001".to_string(),
        });
        let _ = core.process_event(Event::DataLoaded { items: vec![] });
        let _ = core.process_event(Event::SessionsLoaded { sessions: vec![] });

        let vm_before = core.view();
        assert!(vm_before.items.is_empty());

        let _ = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "Clair de Lune".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
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
        let _ = core.process_event(Event::StartApp {
            api_base_url: "http://localhost:3001".to_string(),
        });
        let _ = core.process_event(Event::DataLoaded { items: vec![] });

        let _ = core.process_event(Event::Item(ItemEvent::Add(CreateItem {
            title: "".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Someone".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })));

        let vm = core.view();
        assert!(vm.error.is_some(), "Expected validation error in ViewModel");
    }

    #[test]
    fn test_init_produces_three_http_fetches() {
        let core = Core::<Intrada>::new();
        let effects = core.process_event(Event::StartApp {
            api_base_url: "http://localhost:3001".to_string(),
        });

        let http = http_effects(&effects);
        assert_eq!(http.len(), 3, "Expected 3 HTTP GET fetches from Init");
        assert!(
            http.iter().all(|r| r.method == "GET"),
            "All Init HTTP effects should be GET requests"
        );
    }
}

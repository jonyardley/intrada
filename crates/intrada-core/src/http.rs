//! HTTP request builders for the intrada API.
//!
//! All request construction and response parsing happens here in the core.
//! The shell executes the raw HTTP requests and feeds responses back;
//! auth headers are added by the shell when processing `Effect::Http`.

use crux_core::Command;

use crate::app::{Effect, Event};
use crate::domain::item::Item;
use crate::domain::session::PracticeSession;
use crate::domain::types::{CreateItem, CreateRoutineRequest, UpdateItem, UpdateRoutineRequest};

type Http = crux_http::command::Http<Effect, Event>;

// ── Fetch operations ────────────────────────────────────────────────────

pub fn fetch_items(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/items"))
        .expect_json::<Vec<Item>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => Event::DataLoaded {
                items: response.body().cloned().unwrap_or_default(),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load items: {e}")),
        })
}

pub fn fetch_sessions(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/sessions"))
        .expect_json::<Vec<PracticeSession>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => Event::SessionsLoaded {
                sessions: response.body().cloned().unwrap_or_default(),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load sessions: {e}")),
        })
}

pub fn fetch_routines(api_base_url: &str) -> Command<Effect, Event> {
    use crate::domain::routine::Routine;

    Http::get(format!("{api_base_url}/api/routines"))
        .expect_json::<Vec<Routine>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => Event::RoutinesLoaded {
                routines: response.body().cloned().unwrap_or_default(),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load routines: {e}")),
        })
}

// ── Item operations ─────────────────────────────────────────────────────

pub fn create_item(api_base_url: &str, item: &Item) -> Command<Effect, Event> {
    let create = CreateItem {
        title: item.title.clone(),
        kind: item.kind.clone(),
        composer: item.composer.clone(),
        key: item.key.clone(),
        tempo: item.tempo.clone(),
        notes: item.notes.clone(),
        tags: item.tags.clone(),
    };
    Http::post(format!("{api_base_url}/api/items"))
        .body_json(&create)
        .expect("serialize CreateItem")
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::RefetchItems,
            Err(e) => Event::LoadFailed(format!("Failed to save item: {e}")),
        })
}

pub fn update_item(api_base_url: &str, item: &Item) -> Command<Effect, Event> {
    let update = UpdateItem {
        title: Some(item.title.clone()),
        composer: Some(item.composer.clone()),
        key: Some(item.key.clone()),
        tempo: Some(item.tempo.clone()),
        notes: Some(item.notes.clone()),
        tags: Some(item.tags.clone()),
    };
    Http::put(format!("{api_base_url}/api/items/{}", item.id))
        .body_json(&update)
        .expect("serialize UpdateItem")
        .expect_json::<Item>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(item) => Event::ItemUpdated { item },
                None => Event::LoadFailed("update_item: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update item: {e}")),
        })
}

pub fn delete_item(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/items/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete item: {e}")),
        })
}

// ── Session operations ──────────────────────────────────────────────────

pub fn create_session(api_base_url: &str, session: &PracticeSession) -> Command<Effect, Event> {
    Http::post(format!("{api_base_url}/api/sessions"))
        .body_json(session)
        .expect("serialize PracticeSession")
        .build()
        .then_send(|result| match result {
            // Don't re-fetch: SaveSession already pushed the session into the
            // model optimistically and rebuilt practice_summaries. Re-fetching
            // could overwrite the optimistic data with a stale server response
            // before the write is visible, causing #247 (practice data not
            // updating after session save).
            Ok(_) => Event::SessionSaved,
            Err(e) => Event::LoadFailed(format!("Failed to save session: {e}")),
        })
}

pub fn delete_session(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/sessions/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete session: {e}")),
        })
}

// ── Routine operations ──────────────────────────────────────────────────

pub fn create_routine(
    api_base_url: &str,
    routine: &crate::domain::routine::Routine,
) -> Command<Effect, Event> {
    let create = CreateRoutineRequest::from_routine(routine);
    Http::post(format!("{api_base_url}/api/routines"))
        .body_json(&create)
        .expect("serialize CreateRoutineRequest")
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::RefetchRoutines,
            Err(e) => Event::LoadFailed(format!("Failed to save routine: {e}")),
        })
}

pub fn update_routine(
    api_base_url: &str,
    routine: &crate::domain::routine::Routine,
) -> Command<Effect, Event> {
    use crate::domain::routine::Routine;

    let update = UpdateRoutineRequest::from_routine(routine);
    Http::put(format!("{api_base_url}/api/routines/{}", routine.id))
        .body_json(&update)
        .expect("serialize UpdateRoutineRequest")
        .expect_json::<Routine>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(routine) => Event::RoutineUpdated { routine },
                None => Event::LoadFailed("update_routine: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update routine: {e}")),
        })
}

pub fn delete_routine(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/routines/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete routine: {e}")),
        })
}

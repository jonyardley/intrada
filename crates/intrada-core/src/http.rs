//! HTTP request builders for the intrada API.
//!
//! All request construction and response parsing happens here in the core.
//! The shell executes the raw HTTP requests and feeds responses back;
//! auth headers are added by the shell when processing `Effect::Http`.

use crux_core::Command;

use crate::app::{Effect, Event};
use crate::domain::item::Item;
use crate::domain::lesson::Lesson;
use crate::domain::session::PracticeSession;
use crate::domain::types::{
    CreateItem, CreateLesson, CreateSetRequest, UpdateItem, UpdateLesson, UpdateSetRequest,
};

type Http = crux_http::command::Http<Effect, Event>;

// ── Fetch operations ────────────────────────────────────────────────────

pub fn fetch_items(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/items"))
        .expect_json::<Vec<Item>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(items) => Event::DataLoaded { items },
                None => Event::LoadFailed("Failed to parse items response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load items: {e}")),
        })
}

pub fn fetch_sessions(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/sessions"))
        .expect_json::<Vec<PracticeSession>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(sessions) => Event::SessionsLoaded { sessions },
                None => Event::LoadFailed("Failed to parse sessions response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load sessions: {e}")),
        })
}

pub fn fetch_sets(api_base_url: &str) -> Command<Effect, Event> {
    use crate::domain::set::Set;

    Http::get(format!("{api_base_url}/api/sets"))
        .expect_json::<Vec<Set>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(sets) => Event::SetsLoaded { sets },
                None => Event::LoadFailed("Failed to parse sets response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load sets: {e}")),
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

// ── Set operations ──────────────────────────────────────────────────

pub fn create_set(api_base_url: &str, set: &crate::domain::set::Set) -> Command<Effect, Event> {
    let create = CreateSetRequest::from_set(set);
    Http::post(format!("{api_base_url}/api/sets"))
        .body_json(&create)
        .expect("serialize CreateSetRequest")
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::RefetchSets,
            Err(e) => Event::LoadFailed(format!("Failed to save set: {e}")),
        })
}

pub fn update_set(api_base_url: &str, set: &crate::domain::set::Set) -> Command<Effect, Event> {
    use crate::domain::set::Set;

    let update = UpdateSetRequest::from_set(set);
    Http::put(format!("{api_base_url}/api/sets/{}", set.id))
        .body_json(&update)
        .expect("serialize UpdateSetRequest")
        .expect_json::<Set>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(set) => Event::SetUpdated { set },
                None => Event::LoadFailed("update_set: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update set: {e}")),
        })
}

pub fn delete_set(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/sets/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete set: {e}")),
        })
}

// ── Lesson operations ──────────────────────────────────────────────────

pub fn fetch_lessons(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/lessons"))
        .expect_json::<Vec<Lesson>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(lessons) => Event::LessonsLoaded { lessons },
                None => Event::LoadFailed("Failed to parse lessons response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load lessons: {e}")),
        })
}

pub fn fetch_lesson(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/lessons/{id}"))
        .expect_json::<Lesson>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(lesson) => Event::LessonLoaded { lesson },
                None => Event::LoadFailed("Failed to parse lesson response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load lesson: {e}")),
        })
}

pub fn create_lesson(api_base_url: &str, input: &CreateLesson) -> Command<Effect, Event> {
    Http::post(format!("{api_base_url}/api/lessons"))
        .body_json(input)
        .expect("serialize CreateLesson")
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::RefetchLessons,
            Err(e) => Event::LoadFailed(format!("Failed to save lesson: {e}")),
        })
}

pub fn update_lesson(api_base_url: &str, id: &str, input: &UpdateLesson) -> Command<Effect, Event> {
    Http::put(format!("{api_base_url}/api/lessons/{id}"))
        .body_json(input)
        .expect("serialize UpdateLesson")
        .expect_json::<Lesson>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(lesson) => Event::LessonLoaded { lesson },
                None => Event::LoadFailed("update_lesson: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update lesson: {e}")),
        })
}

pub fn delete_lesson(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/lessons/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete lesson: {e}")),
        })
}

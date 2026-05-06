// The `#[effect]` macro generates an enum with large variant size differences
// (Request<HttpRequest> vs Request<RenderOperation>); we can't Box through the macro.
#![allow(clippy::large_enum_variant)]

use crux_core::capability::Operation;
use crux_core::macros::effect;
use crux_core::render::RenderOperation;
use crux_core::{App, Command};
use crux_http::HttpRequest;
use serde::{Deserialize, Serialize};

use crate::analytics::compute_analytics;
#[cfg(test)]
use crate::domain::item::ItemKind;
use crate::domain::item::{handle_item_event, Item, ItemEvent};
use crate::domain::lesson::{handle_lesson_event, Lesson, LessonEvent};
use crate::domain::session::{
    handle_session_event, ActiveSession, PracticeSession, SessionEvent, SessionStatus,
};
#[cfg(test)]
use crate::domain::session::{CompletionStatus, EntryStatus, SetlistEntry};
use crate::domain::set::{handle_set_event, Set, SetEvent};
use crate::domain::types::ListQuery;
use crate::http;
use crate::model::{
    build_active_session_view, build_summary_view, entry_to_view, lesson_to_view, session_to_view,
    BuildingSetlistView, ItemPracticeSummary, LibraryItemView, Model, SessionStatusView, ViewModel,
};

/// Root Crux application for the music practice library.
#[derive(Default)]
pub struct Intrada;

/// All events the application can process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Event {
    // ── Lifecycle ────────────────────────────────────────────────────
    /// Shell provides the API base URL on startup.
    /// Named `StartApp` (not `Init`) to avoid Swift keyword collision.
    StartApp {
        api_base_url: String,
    },
    /// Fetch all data from the API (items, sessions, sets).
    FetchAll,
    /// Re-fetch a single resource kind after a mutation (refresh-after-mutate).
    RefetchItems,
    RefetchSessions,
    RefetchSets,
    RefetchLessons,

    // ── Domain ──────────────────────────────────────────────────────
    Item(ItemEvent),
    Session(SessionEvent),
    Set(SetEvent),
    Lesson(LessonEvent),

    // ── Data loaded callbacks ───────────────────────────────────────
    DataLoaded {
        items: Vec<Item>,
    },
    SessionsLoaded {
        sessions: Vec<PracticeSession>,
    },
    SetsLoaded {
        sets: Vec<Set>,
    },
    LessonsLoaded {
        lessons: Vec<Lesson>,
    },
    LessonLoaded {
        lesson: Lesson,
    },

    // ── Write-confirmation callbacks ────────────────────────────────
    /// Server confirmed an item update — replace the optimistic copy.
    ItemUpdated {
        item: Item,
    },
    /// Server confirmed a set update — replace the optimistic copy.
    SetUpdated {
        set: Set,
    },
    /// Server confirmed a delete — model already updated optimistically.
    DeleteConfirmed,
    /// Server confirmed session creation — model already updated optimistically.
    SessionSaved,

    // ── Error handling ──────────────────────────────────────────────
    LoadFailed(String),
    ClearError,
    SetQuery(Option<ListQuery>),
}

/// Side effects the core requests from shells.
///
/// The `#[effect]` attribute macro from `crux_core` generates the
/// `From<Request<Op>>` impls plus `impl crux_core::Effect`. Source variants
/// hold **operation types** (e.g. `RenderOperation`, `HttpRequest`); the macro
/// wraps each in `Request<Op>` in the compiled enum.
///
/// HTTP API calls go through `Http` (crux_http). The shell executes the raw
/// HTTP request and feeds the response back; all request construction and
/// response parsing happens in the core (see `http.rs`).
#[effect]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    /// Shell-only side effects that are NOT HTTP (localStorage only).
    App(AppEffect),
}

/// Non-HTTP side-effect operations handled by the shell.
///
/// After the crux_http migration, only localStorage crash-recovery operations
/// remain here. All API calls now go through the `Http` effect variant.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AppEffect {
    /// Persist the active session to localStorage for crash recovery (FR-008).
    SaveSessionInProgress(ActiveSession),
    /// Clear the active session from localStorage.
    ClearSessionInProgress,
}

impl Operation for AppEffect {
    type Output = ();
}

// Note: `impl crux_core::Effect` and `From<Request<Op>>` impls are generated
// by the `#[effect]` macro.

impl App for Intrada {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            // ── Lifecycle ────────────────────────────────────────────
            Event::StartApp { api_base_url } => {
                model.api_base_url = api_base_url;
                // Immediately fetch all data
                Command::all([
                    http::fetch_items(&model.api_base_url),
                    http::fetch_sessions(&model.api_base_url),
                    http::fetch_sets(&model.api_base_url),
                ])
            }
            Event::FetchAll => Command::all([
                http::fetch_items(&model.api_base_url),
                http::fetch_sessions(&model.api_base_url),
                http::fetch_sets(&model.api_base_url),
            ]),
            Event::RefetchItems => http::fetch_items(&model.api_base_url),
            Event::RefetchSessions => http::fetch_sessions(&model.api_base_url),
            Event::RefetchSets => http::fetch_sets(&model.api_base_url),
            Event::RefetchLessons => http::fetch_lessons(&model.api_base_url),

            // ── Domain handlers ──────────────────────────────────────
            Event::Item(item_event) => handle_item_event(item_event, model),
            Event::Session(session_event) => handle_session_event(session_event, model),
            Event::Set(set_event) => handle_set_event(set_event, model),
            Event::Lesson(lesson_event) => handle_lesson_event(lesson_event, model),

            // ── Data loaded callbacks ────────────────────────────────
            Event::DataLoaded { items } => {
                model.items = items;
                model.last_error = None;
                crux_core::render::render()
            }
            Event::SessionsLoaded { sessions } => {
                model.sessions = sessions;
                model.practice_summaries = build_practice_summaries(&model.sessions);
                crux_core::render::render()
            }
            Event::SetsLoaded { sets } => {
                model.sets = sets;
                crux_core::render::render()
            }
            Event::LessonsLoaded { lessons } => {
                model.lessons = lessons;
                model.last_error = None;
                crux_core::render::render()
            }
            Event::LessonLoaded { lesson } => {
                model.current_lesson = Some(lesson);
                model.last_error = None;
                crux_core::render::render()
            }

            // ── Write-confirmation callbacks ─────────────────────────
            Event::ItemUpdated { item } => {
                if let Some(existing) = model.items.iter_mut().find(|i| i.id == item.id) {
                    *existing = item;
                }
                crux_core::render::render()
            }
            Event::SetUpdated { set } => {
                if let Some(existing) = model.sets.iter_mut().find(|r| r.id == set.id) {
                    *existing = set;
                }
                crux_core::render::render()
            }
            Event::DeleteConfirmed | Event::SessionSaved => {
                // Model was already updated optimistically; no action needed.
                Command::done()
            }

            // ── Error handling ───────────────────────────────────────
            Event::LoadFailed(msg) => {
                model.last_error = Some(msg);
                crux_core::render::render()
            }
            Event::ClearError => {
                model.last_error = None;
                crux_core::render::render()
            }
            Event::SetQuery(query) => {
                model.active_query = query;
                crux_core::render::render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let mut items: Vec<LibraryItemView> = Vec::new();

        for item in &model.items {
            let practice = model.practice_summaries.get(&item.id).cloned();
            let subtitle = item.composer.clone().unwrap_or_default();
            let latest_achieved_tempo = practice.as_ref().and_then(|p| p.latest_tempo);
            items.push(LibraryItemView {
                id: item.id.clone(),
                item_type: item.kind.clone(),
                title: item.title.clone(),
                subtitle,
                key: item.key.clone(),
                tempo: item
                    .tempo
                    .as_ref()
                    .map(|t| t.format_display())
                    .filter(|s| !s.is_empty()),
                notes: item.notes.clone(),
                tags: item.tags.clone(),
                created_at: item.created_at.to_rfc3339(),
                updated_at: item.updated_at.to_rfc3339(),
                practice,
                latest_achieved_tempo,
            });
        }

        // Apply active query filter
        if let Some(ref query) = model.active_query {
            items = apply_query_filter(items, query);
        }

        // Sort by created_at descending (newest first)
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Build completed session views sorted newest-first
        let mut sessions: Vec<_> = model.sessions.iter().map(session_to_view).collect();
        sessions.sort_by(|a, b| b.finished_at.cmp(&a.finished_at));

        // Build active session / building / summary views from session_status
        let (active_session, building_setlist, summary) = match &model.session_status {
            SessionStatus::Idle => (None, None, None),
            SessionStatus::Building(building) => {
                let entries: Vec<_> = building.entries.iter().map(entry_to_view).collect();
                let item_count = entries.len();
                (
                    None,
                    Some(BuildingSetlistView {
                        entries,
                        item_count,
                        session_intention: building.session_intention.clone(),
                        target_duration_mins: building.target_duration_mins,
                    }),
                    None,
                )
            }
            SessionStatus::Active(active) => (Some(build_active_session_view(active)), None, None),
            SessionStatus::Summary(summary_session) => {
                (None, None, Some(build_summary_view(summary_session)))
            }
        };

        let session_status = match &model.session_status {
            SessionStatus::Idle => SessionStatusView::Idle,
            SessionStatus::Building(_) => SessionStatusView::Building,
            SessionStatus::Active(_) => SessionStatusView::Active,
            SessionStatus::Summary(_) => SessionStatusView::Summary,
        };

        // Compute analytics from session data.
        // Note: Uses Utc::now() which makes view() impure. This is a pragmatic
        // tradeoff — the date only changes once/day and caching analytics in the
        // Model would require plumbing a clock through the event system. All
        // computation functions accept `today` as a parameter for testability.
        let analytics = if model.sessions.is_empty() {
            None
        } else {
            let today = chrono::Utc::now().date_naive();
            Some(compute_analytics(&model.sessions, &model.items, today))
        };

        // Build set views
        let sets = model
            .sets
            .iter()
            .map(|r| {
                use crate::model::{SetEntryView, SetView};
                SetView {
                    id: r.id.clone(),
                    name: r.name.clone(),
                    entry_count: r.entries.len(),
                    entries: r
                        .entries
                        .iter()
                        .map(|e| SetEntryView {
                            id: e.id.clone(),
                            item_id: e.item_id.clone(),
                            item_title: e.item_title.clone(),
                            item_type: e.item_type.clone(),
                            position: e.position,
                        })
                        .collect(),
                }
            })
            .collect();

        // Build lesson views sorted by date descending
        let mut lessons: Vec<_> = model.lessons.iter().map(lesson_to_view).collect();
        lessons.sort_by(|a, b| b.date.cmp(&a.date));

        let current_lesson = model.current_lesson.as_ref().map(lesson_to_view);

        ViewModel {
            items,
            sessions,
            active_session,
            building_setlist,
            summary,
            session_status,
            error: model.last_error.clone(),
            analytics,
            sets,
            lessons,
            current_lesson,
        }
    }
}

/// Build practice summaries for all items in a single pass over sessions.
///
/// Returns a map keyed by item_id. Called once when sessions change,
/// replacing the old per-item O(M×E) scan that ran on every render.
pub(crate) fn build_practice_summaries(
    sessions: &[PracticeSession],
) -> std::collections::HashMap<String, ItemPracticeSummary> {
    use crate::model::{ScoreHistoryEntry, TempoHistoryEntry};
    use std::collections::HashMap;

    let mut acc: HashMap<String, (usize, u64, Vec<ScoreHistoryEntry>, Vec<TempoHistoryEntry>)> =
        HashMap::new();

    for session in sessions {
        for entry in &session.entries {
            let record = acc
                .entry(entry.item_id.clone())
                .or_insert_with(|| (0, 0, Vec::new(), Vec::new()));
            record.0 += 1;
            record.1 += entry.duration_secs;

            if let Some(score) = entry.score {
                record.2.push(ScoreHistoryEntry {
                    session_date: session.started_at.to_rfc3339(),
                    score,
                    session_id: session.id.clone(),
                });
            }

            if let Some(tempo) = entry.achieved_tempo {
                record.3.push(TempoHistoryEntry {
                    session_date: session.started_at.to_rfc3339(),
                    tempo,
                    session_id: session.id.clone(),
                });
            }
        }
    }

    acc.into_iter()
        .map(
            |(item_id, (session_count, total_secs, mut score_history, mut tempo_history))| {
                score_history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
                let latest_score = score_history.first().map(|e| e.score);

                tempo_history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
                let latest_tempo = tempo_history.first().map(|e| e.tempo);

                (
                    item_id,
                    ItemPracticeSummary {
                        session_count,
                        total_minutes: (total_secs / 60) as u32,
                        latest_score,
                        score_history,
                        latest_tempo,
                        tempo_history,
                    },
                )
            },
        )
        .collect()
}

fn apply_query_filter(items: Vec<LibraryItemView>, query: &ListQuery) -> Vec<LibraryItemView> {
    items
        .into_iter()
        .filter(|item| {
            // Filter by item type
            if let Some(ref item_type) = query.item_type {
                if item.item_type != *item_type {
                    return false;
                }
            }

            // Filter by key
            if let Some(ref key) = query.key {
                if item.key.as_deref() != Some(key.as_str()) {
                    return false;
                }
            }

            // Filter by tags (all must match, case-insensitive)
            if !query.tags.is_empty() {
                for tag in &query.tags {
                    let tag_lower = tag.to_lowercase();
                    if !item.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                        return false;
                    }
                }
            }

            // Filter by text search (case-insensitive substring match)
            if let Some(ref text) = query.text {
                let text_lower = text.to_lowercase();
                let matches = item.title.to_lowercase().contains(&text_lower)
                    || item.subtitle.to_lowercase().contains(&text_lower)
                    || item
                        .notes
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&text_lower))
                    || item
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&text_lower));
                if !matches {
                    return false;
                }
            }

            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_loaded_populates_model() {
        let app = Intrada;
        let mut model = Model::test_default();

        let now = chrono::Utc::now();
        let items = vec![
            Item {
                id: "piece1".to_string(),
                title: "Clair de Lune".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Debussy".to_string()),
                key: Some("Db Major".to_string()),
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
            },
            Item {
                id: "ex1".to_string(),
                title: "C Major Scale".to_string(),
                kind: ItemKind::Exercise,
                composer: None,
                key: Some("C Major".to_string()),
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
            },
        ];

        let _cmd = app.update(Event::DataLoaded { items }, &mut model);

        assert_eq!(model.items.len(), 2);
        assert_eq!(model.items[0].title, "Clair de Lune");
        assert_eq!(model.items[1].title, "C Major Scale");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_clear_error() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("some error".to_string()),
            ..Default::default()
        };

        let _cmd = app.update(Event::ClearError, &mut model);

        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_view_empty_model() {
        let app = Intrada;
        let model = Model::test_default();
        let vm = app.view(&model);

        assert!(vm.items.is_empty());
        assert_eq!(vm.items.len(), 0);
        assert!(vm.error.is_none());
        assert_eq!(vm.session_status, SessionStatusView::Idle);
    }

    #[test]
    fn test_view_with_items() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            items: vec![
                Item {
                    id: "p1".to_string(),
                    title: "Sonata".to_string(),
                    kind: ItemKind::Piece,
                    composer: Some("Beethoven".to_string()),
                    key: None,
                    tempo: Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(132),
                    }),
                    notes: None,
                    tags: vec!["classical".to_string()],
                    created_at: now,
                    updated_at: now,
                },
                Item {
                    id: "e1".to_string(),
                    title: "Scales".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                },
            ],
            ..Default::default()
        };

        let vm = app.view(&model);

        assert_eq!(vm.items.len(), 2);

        // Check piece
        let piece_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        assert_eq!(piece_view.item_type, ItemKind::Piece);
        assert_eq!(piece_view.title, "Sonata");
        assert_eq!(piece_view.subtitle, "Beethoven");
        assert_eq!(piece_view.tempo, Some("Allegro (132 BPM)".to_string()));
        assert_eq!(piece_view.tags, vec!["classical".to_string()]);

        // Check exercise
        let ex_view = vm.items.iter().find(|i| i.id == "e1").unwrap();
        assert_eq!(ex_view.item_type, ItemKind::Exercise);
        assert_eq!(ex_view.title, "Scales");
        assert_eq!(ex_view.subtitle, "");
    }

    #[test]
    fn test_view_shows_error() {
        let app = Intrada;
        let model = Model {
            last_error: Some("Something went wrong".to_string()),
            ..Default::default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.error, Some("Something went wrong".to_string()));
    }

    // --- Query filtering in core ---

    #[test]
    fn test_set_query_filters_by_type() {
        let app = Intrada;
        let mut model = Model::test_default();
        let now = chrono::Utc::now();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });
        model.items.push(Item {
            id: "e1".to_string(),
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        // No filter — both items
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 2);

        // Filter to pieces only
        let _cmd = app.update(
            Event::SetQuery(Some(ListQuery {
                item_type: Some(ItemKind::Piece),
                ..Default::default()
            })),
            &mut model,
        );
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].item_type, ItemKind::Piece);

        // Clear filter
        let _cmd = app.update(Event::SetQuery(None), &mut model);
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 2);
    }

    #[test]
    fn test_set_query_filters_by_text() {
        let app = Intrada;
        let mut model = Model::test_default();
        let now = chrono::Utc::now();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });
        model.items.push(Item {
            id: "p2".to_string(),
            title: "Clair de Lune".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        model.active_query = Some(ListQuery {
            text: Some("beethoven".to_string()),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Moonlight Sonata");
    }

    #[test]
    fn test_set_query_filters_by_tags() {
        let app = Intrada;
        let mut model = Model::test_default();
        let now = chrono::Utc::now();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["classical".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
        });
        model.items.push(Item {
            id: "p2".to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["romantic".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
        });

        model.active_query = Some(ListQuery {
            tags: vec!["classical".to_string()],
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Sonata");
    }

    // --- T042: Unicode handling in core ---

    #[test]
    fn test_unicode_in_item_add() {
        let app = Intrada;
        let mut model = Model::test_default();

        let _cmd = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "Ménuet en Sol".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Dvořák".to_string()),
                key: Some("ré mineur".to_string()),
                tempo: None,
                notes: Some("Pièce très jolie — «superbe»".to_string()),
                tags: vec!["日本語タグ".to_string()],
            })),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Ménuet en Sol");
        assert_eq!(model.items[0].composer, Some("Dvořák".to_string()));
        assert_eq!(model.items[0].key, Some("ré mineur".to_string()));
        assert_eq!(
            model.items[0].notes,
            Some("Pièce très jolie — «superbe»".to_string())
        );
        assert_eq!(model.items[0].tags, vec!["日本語タグ".to_string()]);

        // Verify ViewModel preserves Unicode
        let vm = app.view(&model);
        assert_eq!(vm.items[0].title, "Ménuet en Sol");
        assert_eq!(vm.items[0].subtitle, "Dvořák");
    }

    // --- T045: Performance benchmark ---

    #[test]
    fn test_performance_10k_items() {
        let app = Intrada;
        let mut model = Model::test_default();
        let now = chrono::Utc::now();

        // Populate 10,000 items (5k pieces + 5k exercises)
        let start = std::time::Instant::now();
        for i in 0..5000 {
            model.items.push(Item {
                id: format!("p{i:05}"),
                title: format!("Piece {i}"),
                kind: ItemKind::Piece,
                composer: Some(format!("Composer {}", i % 100)),
                key: if i % 3 == 0 {
                    Some("C Major".to_string())
                } else {
                    None
                },
                tempo: if i % 5 == 0 {
                    Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(120),
                    })
                } else {
                    None
                },
                notes: if i % 7 == 0 {
                    Some(format!("Notes for piece {i}"))
                } else {
                    None
                },
                tags: vec![format!("tag{}", i % 10)],
                created_at: now,
                updated_at: now,
            });
        }
        for i in 0..5000 {
            model.items.push(Item {
                id: format!("e{i:05}"),
                title: format!("Exercise {i}"),
                kind: ItemKind::Exercise,
                composer: None,
                key: if i % 4 == 0 {
                    Some("G Major".to_string())
                } else {
                    None
                },
                tempo: None,
                notes: None,
                tags: vec![format!("etag{}", i % 10)],
                created_at: now,
                updated_at: now,
            });
        }
        let populate_time = start.elapsed();
        assert!(
            populate_time.as_millis() < 100,
            "Populating 10k items took {}ms (target: <100ms)",
            populate_time.as_millis()
        );

        // Populate 500 sessions with 5 entries each (2,500 entries total)
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };
        let start = std::time::Instant::now();
        for s in 0..500u32 {
            let entries: Vec<SetlistEntry> = (0..5u32)
                .map(|e| {
                    let item_idx = ((s * 5 + e) % 10_000) as usize;
                    let (item_id, item_title, item_type) = if item_idx < 5000 {
                        (
                            format!("p{item_idx:05}"),
                            format!("Piece {item_idx}"),
                            ItemKind::Piece,
                        )
                    } else {
                        let idx = item_idx - 5000;
                        (
                            format!("e{idx:05}"),
                            format!("Exercise {idx}"),
                            ItemKind::Exercise,
                        )
                    };
                    SetlistEntry {
                        id: format!("se{s:04}_{e}"),
                        item_id,
                        item_title,
                        item_type,
                        position: e as usize,
                        duration_secs: 300,
                        status: EntryStatus::Completed,
                        notes: None,
                        score: if e % 2 == 0 { Some(3) } else { None },
                        intention: None,
                        rep_target: None,
                        rep_count: None,
                        rep_target_reached: None,
                        rep_history: None,
                        planned_duration_secs: None,
                        achieved_tempo: if e % 3 == 0 { Some(120) } else { None },
                    }
                })
                .collect();
            model.sessions.push(PracticeSession {
                id: format!("sess{s:04}"),
                started_at: now - chrono::Duration::hours(s as i64 + 1),
                completed_at: now - chrono::Duration::hours(s as i64),
                total_duration_secs: 1500,
                completion_status: CompletionStatus::Completed,
                session_notes: None,
                session_intention: None,
                entries,
            });
        }
        model.practice_summaries = build_practice_summaries(&model.sessions);
        let session_populate_time = start.elapsed();
        assert!(
            session_populate_time.as_millis() < 200,
            "Populating 500 sessions + cache took {}ms (target: <200ms)",
            session_populate_time.as_millis()
        );

        // Benchmark: view() with 10k items + 500 sessions
        let start = std::time::Instant::now();
        let vm = app.view(&model);
        let view_time = start.elapsed();
        assert_eq!(vm.items.len(), 10_000);
        assert!(
            view_time.as_millis() < 200,
            "view() with 10k items took {}ms (target: <200ms)",
            view_time.as_millis()
        );

        // Benchmark: add one more item with 10k existing
        let start = std::time::Instant::now();
        let _cmd = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "New Piece".to_string(),
                kind: ItemKind::Piece,
                composer: Some("New Composer".to_string()),
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
            })),
            &mut model,
        );
        let add_time = start.elapsed();
        assert_eq!(model.items.len(), 10_001);
        assert!(
            add_time.as_millis() < 100,
            "Adding item with 10k existing took {}ms (target: <100ms)",
            add_time.as_millis()
        );

        // Benchmark: delete item with 10k existing
        let start = std::time::Instant::now();
        let _cmd = app.update(
            Event::Item(ItemEvent::Delete {
                id: "p00042".to_string(),
            }),
            &mut model,
        );
        let delete_time = start.elapsed();
        assert_eq!(model.items.len(), 10_000);
        assert!(
            delete_time.as_millis() < 100,
            "Deleting item with 10k existing took {}ms (target: <100ms)",
            delete_time.as_millis()
        );
    }

    // --- Practice summary with new setlist sessions ---

    #[test]
    fn test_view_practice_summary_with_setlist_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::test_default();

        let p1 = Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        let p2 = Item {
            id: "p2".to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        model.items = vec![p1, p2];

        // Create a completed session with two entries
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::minutes(60),
            completed_at: now,
            total_duration_secs: 2700,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries: vec![
                SetlistEntry {
                    id: "e1".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 0,
                    duration_secs: 1800, // 30 min
                    status: EntryStatus::Completed,
                    notes: None,
                    score: None,
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                },
                SetlistEntry {
                    id: "e2".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 1,
                    duration_secs: 900, // 15 min
                    status: EntryStatus::Completed,
                    notes: None,
                    score: None,
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                },
            ],
        });
        model.practice_summaries = build_practice_summaries(&model.sessions);

        let vm = app.view(&model);
        let p1_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let p2_view = vm.items.iter().find(|i| i.id == "p2").unwrap();

        // p1 has 2 entries totalling 45 minutes, no scores, no tempo
        assert_eq!(
            p1_view.practice,
            Some(ItemPracticeSummary {
                session_count: 2,
                total_minutes: 45,
                latest_score: None,
                score_history: vec![],
                latest_tempo: None,
                tempo_history: vec![],
            })
        );
        // p2 has no entries
        assert_eq!(p2_view.practice, None);
    }

    // ── Score history tests (T019) ────────────────────────────────────

    #[test]
    fn test_score_history_multiple_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::test_default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // Session 1: older, score 3
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(2),
            completed_at: now - chrono::Duration::hours(1),
            total_duration_secs: 3600,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries: vec![SetlistEntry {
                id: "e1".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 1800,
                status: EntryStatus::Completed,
                notes: None,
                score: Some(3),
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
            }],
        });

        // Session 2: newer, score 5
        model.sessions.push(PracticeSession {
            id: "sess2".to_string(),
            started_at: now - chrono::Duration::minutes(30),
            completed_at: now,
            total_duration_secs: 1800,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries: vec![SetlistEntry {
                id: "e2".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 900,
                status: EntryStatus::Completed,
                notes: None,
                score: Some(5),
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
            }],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        // latest_score should be from the newer session
        assert_eq!(practice.latest_score, Some(5));
        assert_eq!(practice.score_history.len(), 2);
        // First entry = most recent (score 5)
        assert_eq!(practice.score_history[0].score, 5);
        assert_eq!(practice.score_history[0].session_id, "sess2");
        // Second entry = older (score 3)
        assert_eq!(practice.score_history[1].score, 3);
        assert_eq!(practice.score_history[1].session_id, "sess1");
    }

    #[test]
    fn test_score_history_no_scored_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::test_default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // Session with no score
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(1),
            completed_at: now,
            total_duration_secs: 1800,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries: vec![SetlistEntry {
                id: "e1".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 1800,
                status: EntryStatus::Completed,
                notes: None,
                score: None,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
            }],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        assert_eq!(practice.latest_score, None);
        assert!(practice.score_history.is_empty());
    }

    #[test]
    fn test_score_history_item_multiple_times_in_one_session() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::test_default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // Single session with the same item twice (different scores)
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(1),
            completed_at: now,
            total_duration_secs: 3600,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries: vec![
                SetlistEntry {
                    id: "e1".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 0,
                    duration_secs: 1800,
                    status: EntryStatus::Completed,
                    notes: None,
                    score: Some(2),
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                },
                SetlistEntry {
                    id: "e2".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 1,
                    duration_secs: 1800,
                    status: EntryStatus::Completed,
                    notes: None,
                    score: Some(4),
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                },
            ],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        // Both entries from the same session should appear in score_history
        assert_eq!(practice.score_history.len(), 2);
        // Both have the same session_id
        assert!(practice
            .score_history
            .iter()
            .all(|e| e.session_id == "sess1"));
    }

    #[test]
    fn test_score_history_skipped_entries_excluded() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::test_default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // A skipped entry won't have a score (scores only set on completed entries)
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(1),
            completed_at: now,
            total_duration_secs: 600,
            completion_status: CompletionStatus::EndedEarly,
            session_notes: None,
            session_intention: None,
            entries: vec![SetlistEntry {
                id: "e1".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 600,
                status: EntryStatus::Skipped,
                notes: None,
                score: None, // Skipped entries never have scores
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
            }],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        assert_eq!(practice.latest_score, None);
        assert!(practice.score_history.is_empty());
    }

    // --- Lifecycle events ---

    #[test]
    fn test_start_app_sets_api_base_url() {
        let app = Intrada;
        let mut model = Model::default();
        assert!(model.api_base_url.is_empty());

        let _cmd = app.update(
            Event::StartApp {
                api_base_url: "https://api.example.com".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.api_base_url, "https://api.example.com");
    }

    // --- Data loaded callbacks ---

    fn make_session(
        id: &str,
        item_id: &str,
        score: Option<u8>,
        tempo: Option<u16>,
    ) -> PracticeSession {
        let now = chrono::Utc::now();
        PracticeSession {
            id: id.to_string(),
            started_at: now,
            completed_at: now,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries: vec![SetlistEntry {
                id: format!("{id}-e1"),
                item_id: item_id.to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 300,
                status: EntryStatus::Completed,
                notes: None,
                score,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: tempo,
            }],
        }
    }

    #[test]
    fn test_sessions_loaded_populates_model_and_summaries() {
        let app = Intrada;
        let mut model = Model::test_default();

        let sessions = vec![make_session("s1", "item-1", Some(4), Some(120))];
        let _cmd = app.update(Event::SessionsLoaded { sessions }, &mut model);

        assert_eq!(model.sessions.len(), 1);
        let summary = model.practice_summaries.get("item-1");
        assert!(summary.is_some());
        let summary = summary.unwrap();
        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.total_minutes, 5);
        assert_eq!(summary.latest_score, Some(4));
        assert_eq!(summary.latest_tempo, Some(120));
    }

    #[test]
    fn test_sets_loaded_populates_model() {
        use crate::domain::set::{Set, SetEntry};

        let app = Intrada;
        let mut model = Model::test_default();
        let now = chrono::Utc::now();

        let sets = vec![Set {
            id: "r1".to_string(),
            name: "Warm-up".to_string(),
            entries: vec![SetEntry {
                id: "re1".to_string(),
                item_id: "item-1".to_string(),
                item_title: "Scales".to_string(),
                item_type: ItemKind::Exercise,
                position: 0,
            }],
            created_at: now,
            updated_at: now,
        }];

        let _cmd = app.update(Event::SetsLoaded { sets }, &mut model);

        assert_eq!(model.sets.len(), 1);
        assert_eq!(model.sets[0].name, "Warm-up");
    }

    // --- Write-confirmation callbacks ---

    #[test]
    fn test_item_updated_replaces_existing() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Old Title".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Composer".to_string()),
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
            }],
            ..Model::test_default()
        };

        let updated = Item {
            id: "p1".to_string(),
            title: "New Title".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Composer".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };

        let _cmd = app.update(Event::ItemUpdated { item: updated }, &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "New Title");
    }

    #[test]
    fn test_item_updated_ignores_unknown_id() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Original".to_string(),
                kind: ItemKind::Piece,
                composer: None,
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
            }],
            ..Model::test_default()
        };

        let unknown = Item {
            id: "unknown".to_string(),
            title: "Ghost".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };

        let _cmd = app.update(Event::ItemUpdated { item: unknown }, &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Original");
    }

    #[test]
    fn test_set_updated_replaces_existing() {
        use crate::domain::set::Set;

        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            sets: vec![Set {
                id: "r1".to_string(),
                name: "Old Set".to_string(),
                entries: vec![],
                created_at: now,
                updated_at: now,
            }],
            ..Model::test_default()
        };

        let updated = Set {
            id: "r1".to_string(),
            name: "Renamed Set".to_string(),
            entries: vec![],
            created_at: now,
            updated_at: now,
        };

        let _cmd = app.update(Event::SetUpdated { set: updated }, &mut model);

        assert_eq!(model.sets[0].name, "Renamed Set");
    }

    #[test]
    fn test_delete_confirmed_is_noop() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.items.push(Item {
            id: "p1".to_string(),
            title: "Still Here".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let _cmd = app.update(Event::DeleteConfirmed, &mut model);

        // Model unchanged — optimistic delete already happened
        assert_eq!(model.items.len(), 1);
    }

    // --- Error handling ---

    #[test]
    fn test_load_failed_sets_error() {
        let app = Intrada;
        let mut model = Model::test_default();

        let _cmd = app.update(
            Event::LoadFailed("Connection refused".to_string()),
            &mut model,
        );

        assert_eq!(model.last_error, Some("Connection refused".to_string()));
    }

    #[test]
    fn test_data_loaded_clears_previous_error() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("Old error".to_string()),
            ..Model::test_default()
        };

        let _cmd = app.update(Event::DataLoaded { items: vec![] }, &mut model);

        assert!(model.last_error.is_none());
    }

    // --- View: session status mapping ---

    #[test]
    fn test_view_session_status_building() {
        use crate::domain::session::BuildingSession;

        let app = Intrada;
        let model = Model {
            session_status: SessionStatus::Building(BuildingSession {
                entries: vec![],
                session_intention: Some("Focus on dynamics".to_string()),
                target_duration_mins: None,
            }),
            ..Model::test_default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.session_status, SessionStatusView::Building);
        assert!(vm.building_setlist.is_some());
        assert!(vm.active_session.is_none());
        assert!(vm.summary.is_none());
        let setlist = vm.building_setlist.unwrap();
        assert_eq!(
            setlist.session_intention,
            Some("Focus on dynamics".to_string())
        );
    }

    // --- View: sets ---

    #[test]
    fn test_view_renders_sets() {
        use crate::domain::set::{Set, SetEntry};

        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            sets: vec![Set {
                id: "r1".to_string(),
                name: "Morning Warm-up".to_string(),
                entries: vec![
                    SetEntry {
                        id: "re1".to_string(),
                        item_id: "item-1".to_string(),
                        item_title: "Scales".to_string(),
                        item_type: ItemKind::Exercise,
                        position: 0,
                    },
                    SetEntry {
                        id: "re2".to_string(),
                        item_id: "item-2".to_string(),
                        item_title: "Arpeggios".to_string(),
                        item_type: ItemKind::Exercise,
                        position: 1,
                    },
                ],
                created_at: now,
                updated_at: now,
            }],
            ..Model::test_default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.sets.len(), 1);
        assert_eq!(vm.sets[0].name, "Morning Warm-up");
        assert_eq!(vm.sets[0].entry_count, 2);
        assert_eq!(vm.sets[0].entries[0].item_title, "Scales");
        assert_eq!(vm.sets[0].entries[1].item_title, "Arpeggios");
    }

    // --- Practice summaries edge cases ---

    #[test]
    fn test_practice_summaries_empty_sessions() {
        let summaries = build_practice_summaries(&[]);
        assert!(summaries.is_empty());
    }

    #[test]
    fn test_practice_summaries_entry_without_score_or_tempo() {
        let sessions = vec![{
            let mut s = make_session("s1", "item-1", None, None);
            s.entries[0].duration_secs = 180;
            s
        }];

        let summaries = build_practice_summaries(&sessions);
        let summary = &summaries["item-1"];
        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.total_minutes, 3);
        assert!(summary.latest_score.is_none());
        assert!(summary.latest_tempo.is_none());
        assert!(summary.score_history.is_empty());
        assert!(summary.tempo_history.is_empty());
    }

    #[test]
    fn test_view_empty_sessions() {
        let app = Intrada;
        let model = Model::test_default();
        let vm = app.view(&model);
        assert!(vm.sessions.is_empty());
    }

    #[test]
    fn test_tempo_format_display() {
        use crate::domain::types::Tempo;

        // None tempo — map returns None
        let none_tempo: Option<Tempo> = None;
        assert_eq!(none_tempo.as_ref().map(|t| t.format_display()), None);

        // Both None — empty string
        let tempo = Tempo {
            marking: None,
            bpm: None,
        };
        assert_eq!(tempo.format_display(), "");

        // Marking only
        let tempo = Tempo {
            marking: Some("Adagio".to_string()),
            bpm: None,
        };
        assert_eq!(tempo.format_display(), "Adagio");

        // BPM only
        let tempo = Tempo {
            marking: None,
            bpm: Some(120),
        };
        assert_eq!(tempo.format_display(), "120 BPM");

        // Both
        let tempo = Tempo {
            marking: Some("Allegro".to_string()),
            bpm: Some(132),
        };
        assert_eq!(tempo.format_display(), "Allegro (132 BPM)");
    }
}

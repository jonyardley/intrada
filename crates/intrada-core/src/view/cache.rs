use crate::analytics::{
    analytics_from_changes, compute_score_changes, AnalyticsView, LastPractisedView, LocalClock,
};
use crate::domain::types::{LibrarySort, SortDirection, SortField};
use crate::model::{LibraryItemView, Model, PracticeSessionView};
use crate::practice_weeks::PracticeWeekView;
use crate::suggestion::SuggestedSession;
use crate::view::library::{
    build_library_item_views, sort_library_items, RECENTLY_PRACTISED_LIMIT,
};
use crate::view::session::{session_to_view, variation_labels};

/// Everything the projections read. The local day and the offset are here
/// because staleness, Up next, the week strip and Progress read
/// `LocalClock::today` (#1694).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProjectionKey {
    items: u64,
    sessions: u64,
    summaries: u64,
    clock: LocalClock,
    sort: LibrarySort,
}

impl ProjectionKey {
    pub(crate) fn of(model: &Model, clock: LocalClock) -> Self {
        ProjectionKey {
            items: model.items.revision(),
            sessions: model.sessions.revision(),
            summaries: model.practice_summaries.revision(),
            clock,
            sort: model.active_sort,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Projections {
    pub(crate) key: ProjectionKey,
    /// In `model.items` order; `sorted` holds the order the Library shows.
    pub(crate) library: Vec<LibraryItemView>,
    pub(crate) sorted: Vec<usize>,
    pub(crate) available_tags: Vec<String>,
    pub(crate) available_composers: Vec<String>,
    pub(crate) up_next: Option<SuggestedSession>,
    pub(crate) has_priorities: bool,
    pub(crate) recently_practised_ids: Vec<String>,
    pub(crate) sessions: Vec<PracticeSessionView>,
    pub(crate) practice_weeks: Vec<PracticeWeekView>,
    pub(crate) analytics: Option<AnalyticsView>,
    pub(crate) last_practised: Option<LastPractisedView>,
}

impl Projections {
    pub(crate) fn rows(&self) -> impl Iterator<Item = &LibraryItemView> {
        self.sorted.iter().map(|&i| &self.library[i])
    }
}

/// Which of the sections the shell holds apart from the ViewModel a refresh
/// changed (#1801).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Changed {
    pub(crate) library: bool,
    pub(crate) history: bool,
    pub(crate) weeks: bool,
}

/// Called at the end of every `update`, since `view` cannot store.
pub(crate) fn refresh(model: &mut Model, now: chrono::DateTime<chrono::Utc>) -> Changed {
    let clock = LocalClock::from_now(now, model.utc_offset_minutes);
    let key = ProjectionKey::of(model, clock);
    if model.projections.as_ref().is_some_and(|p| p.key == key) {
        return Changed::default();
    }
    let built = build(model, clock);
    let changed = match &model.projections {
        Some(old) => Changed {
            library: !old.rows().eq(built.rows()),
            history: old.sessions != built.sessions,
            weeks: old.practice_weeks != built.practice_weeks,
        },
        None => Changed {
            library: !built.library.is_empty(),
            history: !built.sessions.is_empty(),
            weeks: true,
        },
    };
    model.projections = Some(built);
    changed
}

pub(crate) fn build(model: &Model, clock: LocalClock) -> Projections {
    let item_index: std::collections::HashMap<&str, &crate::domain::item::Item> =
        model.items.iter().map(|i| (i.id.as_str(), i)).collect();

    let library = build_library_item_views(model, &item_index);

    // Computed before the filter so the vocabulary stays stable as the
    // filter narrows (#851).
    let available_tags = vocabulary(library.iter().flat_map(|i| &i.tags));
    let available_composers = vocabulary(library.iter().map(|i| &i.subtitle));

    // Derived before the filter: a narrowed library must not hide the
    // suggestion the Practice tab leads with (#1082).
    let up_next = crate::suggestion::compute_up_next(&library, clock);

    // Pre-filter for the same reason as `up_next`: a narrowed library must
    // not hide the "Practise your priorities" button (#981).
    let has_priorities = library.iter().any(|i| i.priority);

    let mut recently_practised: Vec<LibraryItemView> = library
        .iter()
        .filter(|i| {
            i.practice
                .as_ref()
                .is_some_and(|p| p.last_practiced_at.is_some())
        })
        .cloned()
        .collect();
    sort_library_items(
        &mut recently_practised,
        &LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Descending,
        },
    );
    let recently_practised_ids = recently_practised
        .into_iter()
        .take(RECENTLY_PRACTISED_LIMIT)
        .map(|i| i.id)
        .collect();

    let sorted = crate::view::library::sorted_order(&library, &model.active_sort);

    let labels = variation_labels(&model.items);
    let mut finished: Vec<_> = model.sessions.iter().collect();
    finished.sort_by_key(|s| std::cmp::Reverse(s.completed_at));
    let sessions = finished
        .into_iter()
        .map(|s| session_to_view(s, &labels))
        .collect();

    let practice_weeks = crate::practice_weeks::compute_practice_weeks(&model.sessions, clock);

    let score_changes = compute_score_changes(&model.sessions, clock);
    let (analytics, last_practised) = if model.sessions.is_empty() {
        (None, None)
    } else {
        (
            Some(analytics_from_changes(
                &score_changes,
                &model.sessions,
                &model.items,
                &model.practice_summaries,
                &library,
                clock,
            )),
            crate::analytics::compute_last_practised(&model.sessions, clock),
        )
    };

    Projections {
        key: ProjectionKey::of(model, clock),
        library,
        sorted,
        available_tags,
        available_composers,
        up_next,
        has_priorities,
        recently_practised_ids,
        sessions,
        practice_weeks,
        analytics,
        last_practised,
    }
}

fn vocabulary<S: AsRef<str>>(values: impl IntoIterator<Item = S>) -> Vec<String> {
    let mut words = crate::validation::distinct_ignoring_case(values);
    words.sort_by_key(|w| w.to_lowercase());
    words
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Event, Intrada};
    use crate::domain::item::{ItemEvent, ItemKind};
    use crate::domain::session::{SessionEvent, TempoReading};
    use crate::domain::types::{CreateItem, ListQuery, UpdateItem};
    use crate::persistence::PersistenceOutput;
    use crate::view::{build_view_at, rendered_at, Rendered};
    use chrono::{DateTime, Duration, Utc};
    use crux_core::App;

    fn send(model: &mut Model, event: Event) {
        let _ = Intrada.update(event, model);
    }

    fn sampled() -> Model {
        let mut model = Model::default();
        send(&mut model, Event::LoadSampleData);
        model
    }

    fn fresh_view(model: &mut Model, now: DateTime<Utc>) -> Rendered {
        let kept = model.projections.take();
        let view = rendered_at(model, now);
        model.projections = kept;
        view
    }

    /// The cache was refilled by the last `update` and renders what a build
    /// from scratch would.
    fn assert_current(model: &mut Model, what: &str) {
        let now = Utc::now();
        let clock = LocalClock::from_now(now, model.utc_offset_minutes);
        assert_eq!(
            model.projections.as_ref().map(|p| p.key),
            Some(ProjectionKey::of(model, clock)),
            "{what}: update left the cache behind"
        );
        let cached = rendered_at(model, now);
        assert_eq!(cached, fresh_view(model, now), "{what}");
    }

    fn library_allocation(model: &Model) -> *const LibraryItemView {
        model
            .projections
            .as_ref()
            .expect("update fills the cache")
            .library
            .as_ptr()
    }

    fn create(title: &str) -> CreateItem {
        CreateItem {
            title: title.to_string(),
            kind: ItemKind::Piece,
            composer: Some("Satie".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec!["new".to_string()],
            photo_id: None,
            variant_labels: Vec::new(),
        }
    }

    #[test]
    fn every_change_to_the_library_or_the_history_reaches_the_screens() {
        let mut model = sampled();
        let first_id = model.items[0].id.clone();
        let last_id = model.items.last().expect("sample items").id.clone();
        let now = Utc::now();

        let steps: Vec<(&str, Event)> = vec![
            (
                "an item added",
                Event::Item(ItemEvent::Add(create("Gymnopédie"))),
            ),
            (
                "an item renamed",
                Event::Item(ItemEvent::Update {
                    id: first_id,
                    input: UpdateItem {
                        title: Some("Renamed".to_string()),
                        ..Default::default()
                    },
                }),
            ),
            (
                "an item deleted",
                Event::Item(ItemEvent::Delete { id: last_id }),
            ),
            (
                "the sort changed",
                Event::SetSort(crate::domain::types::LibrarySort {
                    field: crate::domain::types::SortField::Title,
                    direction: crate::domain::types::SortDirection::Ascending,
                }),
            ),
            (
                "the store loaded fewer items",
                Event::StoreLoaded(PersistenceOutput::Items(
                    crate::sample::sample_items().into_iter().take(3).collect(),
                )),
            ),
            (
                "the store loaded no sessions",
                Event::SessionsStoreLoaded(PersistenceOutput::Sessions(vec![])),
            ),
            ("the sample data loaded", Event::LoadSampleData),
        ];

        for (what, event) in steps {
            let wrote = matches!(event, Event::Item(_));
            let before = rendered_at(&model, now);
            send(&mut model, event);
            assert_ne!(
                before,
                fresh_view(&mut model, now),
                "{what} changes nothing"
            );
            assert_current(&mut model, what);
            if wrote {
                // A load landing over an unconfirmed edit is dropped (#2067).
                send(&mut model, Event::StoreWritten(PersistenceOutput::Ack));
            }
        }
    }

    #[test]
    fn a_saved_practice_reaches_the_history_and_progress() {
        let mut model = sampled();
        let item_id = model.items[0].id.clone();
        let start = Utc::now() - Duration::minutes(5);
        let sessions_before = crate::view::rendered(&model).sessions.len();

        for event in [
            Event::Session(SessionEvent::StartBuilding),
            Event::Session(SessionEvent::AddToSetlist { item_id }),
            Event::Session(SessionEvent::StartSession { now: start }),
            Event::Session(SessionEvent::EndSessionEarly {
                now: start + Duration::seconds(90),
                reading: TempoReading::silent(),
            }),
            Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
            Event::SessionStoreWritten(PersistenceOutput::Ack),
        ] {
            send(&mut model, event);
        }

        assert_eq!(model.sessions.len(), sessions_before + 1);
        assert_current(&mut model, "a practice saved");
    }

    #[test]
    fn a_new_utc_offset_rebuilds() {
        let mut model = sampled();
        let kept = library_allocation(&model);
        send(&mut model, Event::SetUtcOffset { minutes: 600 });
        assert_ne!(library_allocation(&model), kept);
        assert_current(&mut model, "the offset changed");
    }

    #[test]
    fn taps_that_touch_neither_the_library_nor_the_history_keep_the_cache() {
        let mut model = sampled();
        let kept = library_allocation(&model);

        send(
            &mut model,
            Event::SetQuery(Some(ListQuery {
                item_type: Some(ItemKind::Exercise),
                ..Default::default()
            })),
        );
        send(&mut model, Event::ClearError);
        send(&mut model, Event::ClearNotice);

        assert_eq!(library_allocation(&model), kept);
        assert_current(&mut model, "a filter and a dismissal");
    }

    #[test]
    fn taps_during_a_practice_keep_the_cache() {
        let mut model = sampled();
        let ids: Vec<String> = model.items.iter().take(2).map(|i| i.id.clone()).collect();
        let start = Utc::now();
        send(&mut model, Event::Session(SessionEvent::StartBuilding));
        for item_id in ids {
            send(
                &mut model,
                Event::Session(SessionEvent::AddToSetlist { item_id }),
            );
        }
        send(
            &mut model,
            Event::Session(SessionEvent::StartSession { now: start }),
        );
        let kept = library_allocation(&model);

        send(
            &mut model,
            Event::Session(SessionEvent::RepGotIt { now: start }),
        );
        send(
            &mut model,
            Event::Session(SessionEvent::NextItem {
                now: start + Duration::seconds(30),
                next_item_started_at: start + Duration::seconds(30),
                reading: TempoReading::silent(),
            }),
        );

        assert!(matches!(
            model.session_status,
            crate::domain::session::SessionStatus::Active(_)
        ));
        assert_eq!(library_allocation(&model), kept);
        assert_current(&mut model, "taps mid-practice");
    }

    #[test]
    fn a_render_after_midnight_with_no_event_since_reads_the_new_day() {
        let mut model = sampled();
        let now = Utc::now();
        refresh(&mut model, now);
        let tomorrow = now + Duration::days(1);

        assert_ne!(
            fresh_view(&mut model, now).practice_weeks,
            fresh_view(&mut model, tomorrow).practice_weeks,
            "the sample must tell the two days apart"
        );
        assert_eq!(
            rendered_at(&model, tomorrow),
            fresh_view(&mut model, tomorrow)
        );
    }

    #[test]
    fn items_hold_the_whole_library_and_visible_ids_follow_the_filter() {
        let mut model = sampled();
        let by_title = crate::domain::types::LibrarySort {
            field: crate::domain::types::SortField::Title,
            direction: crate::domain::types::SortDirection::Ascending,
        };
        send(&mut model, Event::SetSort(by_title));
        send(
            &mut model,
            Event::SetQuery(Some(ListQuery {
                item_type: Some(ItemKind::Piece),
                ..Default::default()
            })),
        );

        let view = crate::view::rendered(&model);
        assert_eq!(view.items.len(), model.items.len());
        let mut by_title_rows = view.items.clone();
        sort_library_items(&mut by_title_rows, &by_title);
        assert_eq!(view.items, by_title_rows);
        let pieces: Vec<&str> = view
            .items
            .iter()
            .filter(|i| i.item_type == ItemKind::Piece)
            .map(|i| i.id.as_str())
            .collect();
        assert!(view.items.len() > pieces.len());
        assert_eq!(view.visible_ids, pieces);
    }

    #[test]
    fn the_view_model_round_trips_on_ffi_bincode_wire() {
        let model = sampled();
        crate::domain::types::assert_round_trips(build_view_at(&model, Utc::now()));
    }

    // ── Sections sent to the shell (#1801) ──

    #[derive(Default)]
    struct Sent {
        library: Option<Vec<LibraryItemView>>,
        history: Option<Vec<PracticeSessionView>>,
        weeks: Option<Vec<PracticeWeekView>>,
        library_before_render: bool,
    }

    fn sent(model: &mut Model, event: Event) -> Sent {
        use crate::app::{AppEffect, Effect};
        let mut command = Intrada.update(event, model);
        let mut out = Sent::default();
        let mut rendered = false;
        for effect in command.effects() {
            match effect {
                Effect::Render(_) => rendered = true,
                Effect::App(request) => match request.operation {
                    AppEffect::LibraryChanged(rows) => {
                        out.library_before_render = !rendered;
                        out.library = Some(rows);
                    }
                    AppEffect::HistoryChanged(rows) => out.history = Some(rows),
                    AppEffect::WeeksChanged(weeks) => out.weeks = Some(weeks),
                    _ => {}
                },
                _ => {}
            }
        }
        out
    }

    fn practice(item_ids: Vec<String>, start: DateTime<Utc>) -> Vec<Event> {
        let mut events = vec![Event::Session(SessionEvent::StartBuilding)];
        events.extend(
            item_ids
                .into_iter()
                .map(|item_id| Event::Session(SessionEvent::AddToSetlist { item_id })),
        );
        events.extend([
            Event::Session(SessionEvent::StartSession { now: start }),
            Event::Session(SessionEvent::RepGotIt { now: start }),
            Event::Session(SessionEvent::NextItem {
                now: start + Duration::seconds(30),
                next_item_started_at: start + Duration::seconds(30),
                reading: TempoReading::silent(),
            }),
        ]);
        events
    }

    #[test]
    fn loading_the_library_sends_every_row_in_the_library_order_before_the_render() {
        let mut model = Model::default();
        let out = sent(&mut model, Event::LoadSampleData);

        let rows = out.library.expect("a first load sends the rows");
        assert_eq!(rows.len(), model.items.len());
        let mut by_sort = rows.clone();
        sort_library_items(&mut by_sort, &model.active_sort);
        assert_eq!(rows, by_sort);
        assert!(out.library_before_render);
        let history = out.history.expect("a first load sends the history");
        assert_eq!(history.len(), model.sessions.len());
        assert!(out
            .weeks
            .is_some_and(|w| w.iter().flat_map(|w| &w.days).any(|d| d.is_today)));
    }

    #[test]
    fn a_search_and_a_practice_under_way_send_neither_section() {
        let mut model = sampled();
        let mut steps = vec![sent(
            &mut model,
            Event::SetQuery(Some(ListQuery {
                item_type: Some(ItemKind::Exercise),
                ..Default::default()
            })),
        )];
        let ids = model.items.iter().take(2).map(|i| i.id.clone()).collect();
        for event in practice(ids, Utc::now()) {
            steps.push(sent(&mut model, event));
        }

        assert!(matches!(
            model.session_status,
            crate::domain::session::SessionStatus::Active(_)
        ));
        for (i, out) in steps.iter().enumerate() {
            assert!(out.library.is_none(), "step {i} sent the library");
            assert!(out.history.is_none(), "step {i} sent the history");
            assert!(out.weeks.is_none(), "step {i} sent the weeks");
        }
    }

    #[test]
    fn the_first_refresh_of_a_new_day_sends_the_weeks() {
        let mut model = sampled();
        let now = Utc::now();
        refresh(&mut model, now);
        assert!(refresh(&mut model, now + Duration::days(1)).weeks);
    }

    #[test]
    fn an_edit_sends_the_library_and_not_the_history() {
        let mut model = sampled();
        let id = model.items[0].id.clone();
        let out = sent(
            &mut model,
            Event::Item(ItemEvent::Update {
                id: id.clone(),
                input: UpdateItem {
                    title: Some("Renamed".to_string()),
                    ..Default::default()
                },
            }),
        );

        let rows = out.library.expect("an edit sends the library");
        assert_eq!(
            rows.iter().find(|r| r.id == id).map(|r| r.title.as_str()),
            Some("Renamed")
        );
        assert!(out.history.is_none());
        assert!(out.weeks.is_none());
    }

    #[test]
    fn a_saved_practice_sends_the_history_with_it() {
        let mut model = sampled();
        let item_id = model.items[0].id.clone();
        let before = model.sessions.len();
        let start = Utc::now() - Duration::minutes(5);

        let (mut history, mut weeks) = (None, None);
        for event in [
            Event::Session(SessionEvent::StartBuilding),
            Event::Session(SessionEvent::AddToSetlist { item_id }),
            Event::Session(SessionEvent::StartSession { now: start }),
            Event::Session(SessionEvent::EndSessionEarly {
                now: start + Duration::seconds(90),
                reading: TempoReading::silent(),
            }),
            Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
            Event::SessionStoreWritten(PersistenceOutput::Ack),
        ] {
            let out = sent(&mut model, event);
            history = out.history.or(history);
            weeks = out.weeks.or(weeks);
        }

        assert_eq!(history.map(|h| h.len()), Some(before + 1));
        assert!(weeks.is_some(), "the week strip marks the new practice");
    }

    /// 200 items and 100 sessions of six entries, the size step 1 measured (#1801).
    fn a_musicians_library() -> Model {
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry, VariationPlay,
        };
        let mut model = Model::default();
        let samples = crate::sample::sample_items();
        let now = Utc::now();
        model.items = (0..200)
            .map(|i| {
                let mut item = samples[i % samples.len()].clone();
                item.id = format!("fx{i:03}");
                item.title = format!("{} {i}", item.title);
                item.notes = Some(format!("Notes on the fingering in bar {i}"));
                item.tags = vec!["scales".into(), format!("term{}", i % 3), "exam".into()];
                item
            })
            .collect::<Vec<_>>()
            .into();
        let sessions: Vec<PracticeSession> = (0..100u32)
            .map(|s| PracticeSession {
                id: format!("sess{s:03}"),
                started_at: now - Duration::hours(i64::from(s) * 20 + 1),
                completed_at: now - Duration::hours(i64::from(s) * 20),
                total_duration_secs: 1800,
                completion_status: CompletionStatus::Completed,
                session_notes: None,
                entries: (0..6u32)
                    .map(|e| {
                        let item = &model.items[((s * 6 + e) % 200) as usize];
                        SetlistEntry {
                            id: format!("se{s:03}_{e}"),
                            item_id: item.id.clone(),
                            item_title: item.title.clone(),
                            item_type: item.kind.clone(),
                            position: e as usize,
                            duration_secs: 300,
                            status: EntryStatus::Completed,
                            plays: vec![VariationPlay {
                                id: format!("se{s:03}_{e}-play"),
                                seconds: 300,
                                score: Some(3),
                                ..VariationPlay::fixture()
                            }],
                            ..SetlistEntry::fixture()
                        }
                    })
                    .collect(),
                session_score: None,
            })
            .collect();
        model.practice_summaries = crate::view::library::build_practice_summaries(&sessions).into();
        model.sessions = sessions.into();
        refresh(&mut model, now);
        model
    }

    #[test]
    fn the_screen_state_sent_on_each_practice_tap_stays_under_20_kb() {
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
        let mut model = a_musicians_library();
        let ids = model.items.iter().take(6).map(|i| i.id.clone()).collect();
        let start = Utc::now();

        let mut sizes = Vec::new();
        for event in practice(ids, start) {
            send(&mut model, event);
            let mut bytes = Vec::new();
            BincodeFfiFormat::serialize(&mut bytes, &build_view_at(&model, start)).expect("encode");
            sizes.push(bytes.len());
        }
        assert!(
            sizes.iter().all(|&n| n < 20_000),
            "bytes per tap: {sizes:?}"
        );
    }
}

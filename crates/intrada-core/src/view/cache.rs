use crate::analytics::{compute_analytics, AnalyticsView, LastPractisedView, LocalClock};
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
    pub(crate) recently_practised: Vec<LibraryItemView>,
    pub(crate) sessions: Vec<PracticeSessionView>,
    pub(crate) practice_weeks: Vec<PracticeWeekView>,
    pub(crate) analytics: Option<AnalyticsView>,
    pub(crate) last_practised: Option<LastPractisedView>,
}

/// Called at the end of every `update`, since `view` cannot store.
pub(crate) fn refresh(model: &mut Model, now: chrono::DateTime<chrono::Utc>) {
    let clock = LocalClock::from_now(now, model.utc_offset_minutes);
    let key = ProjectionKey::of(model, clock);
    if model.projections.as_ref().is_some_and(|p| p.key == key) {
        return;
    }
    model.projections = Some(build(model, clock));
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
    recently_practised.truncate(RECENTLY_PRACTISED_LIMIT);

    let sorted = crate::view::library::sorted_order(&library, &model.active_sort);

    let labels = variation_labels(&model.items);
    let mut finished: Vec<_> = model.sessions.iter().collect();
    finished.sort_by_key(|s| std::cmp::Reverse(s.completed_at));
    let sessions = finished
        .into_iter()
        .map(|s| session_to_view(s, &labels))
        .collect();

    let practice_weeks = crate::practice_weeks::compute_practice_weeks(&model.sessions, clock);

    let (analytics, last_practised) = if model.sessions.is_empty() {
        (None, None)
    } else {
        (
            Some(compute_analytics(
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
        recently_practised,
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
    use crate::model::ViewModel;
    use crate::persistence::PersistenceOutput;
    use crate::view::build_view_at;
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

    fn fresh_view(model: &mut Model, now: DateTime<Utc>) -> ViewModel {
        let kept = model.projections.take();
        let view = build_view_at(model, now);
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
        let cached = build_view_at(model, now);
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
            let before = build_view_at(&model, now);
            send(&mut model, event);
            assert_ne!(
                before,
                fresh_view(&mut model, now),
                "{what} changes nothing"
            );
            assert_current(&mut model, what);
        }
    }

    #[test]
    fn a_saved_practice_reaches_the_history_and_progress() {
        let mut model = sampled();
        let item_id = model.items[0].id.clone();
        let start = Utc::now() - Duration::minutes(5);
        let sessions_before = build_view_at(&model, Utc::now()).sessions.len();

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
            build_view_at(&model, tomorrow),
            fresh_view(&mut model, tomorrow)
        );
    }

    #[test]
    fn visible_ids_follow_the_filter_and_the_sort() {
        let mut model = sampled();
        send(
            &mut model,
            Event::SetSort(crate::domain::types::LibrarySort {
                field: crate::domain::types::SortField::Title,
                direction: crate::domain::types::SortDirection::Ascending,
            }),
        );
        send(
            &mut model,
            Event::SetQuery(Some(ListQuery {
                item_type: Some(ItemKind::Piece),
                ..Default::default()
            })),
        );

        let view = build_view_at(&model, Utc::now());
        let pieces: Vec<&str> = view.items.iter().map(|i| i.id.as_str()).collect();
        assert!(view.all_items.len() > pieces.len());
        assert_eq!(view.visible_ids, pieces);
    }

    #[test]
    fn recently_practised_ids_name_the_recently_practised_rows() {
        let model = sampled();
        let view = build_view_at(&model, Utc::now());
        assert!(!view.recently_practised.is_empty());
        let rows: Vec<&str> = view
            .recently_practised
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        assert_eq!(view.recently_practised_ids, rows);
    }

    #[test]
    fn the_view_model_round_trips_on_ffi_bincode_wire() {
        let model = sampled();
        crate::domain::types::assert_round_trips(build_view_at(&model, Utc::now()));
    }
}

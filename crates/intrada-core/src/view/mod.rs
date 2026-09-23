use crate::analytics::compute_analytics;
use crate::domain::item::ItemKind;
use crate::domain::profile::build_profile_view;
use crate::domain::session::SessionStatus;
use crate::domain::types::{LibrarySort, SortDirection, SortField};
use crate::model::{BuildingSetlistView, LimitsView, Model, PhotoRecognitionView, ViewModel};
use crate::view::library::{
    apply_query_filter, build_library_item_views, sort_library_items, RECENTLY_PRACTISED_LIMIT,
};
use crate::view::session::{
    build_active_session_view, build_blocks, build_summary_view, entry_to_view, session_to_view,
};

pub mod library;
pub mod session;

use crate::app::Intrada;

impl Intrada {
    pub(crate) fn build_view(&self, model: &Model) -> ViewModel {
        use std::collections::HashMap;

        let item_index: HashMap<&str, &crate::domain::item::Item> =
            model.items.iter().map(|i| (i.id.as_str(), i)).collect();

        let mut items = build_library_item_views(model, &item_index);

        // Computed before the filter so the vocabulary stays stable as the
        // filter narrows (#851).
        let available_tags = vocabulary(items.iter().flat_map(|i| &i.tags));
        let available_composers = vocabulary(items.iter().map(|i| &i.subtitle));

        // view() reads the clock: the local day and the greeting's hour (#1694).
        let now = chrono::Utc::now();
        let clock = crate::analytics::LocalClock::from_now(now, model.utc_offset_minutes);

        // Derived before the filter: a narrowed library must not hide the
        // suggestion the Practice tab leads with (#1082).
        let up_next = crate::suggestion::compute_up_next(&items, clock);

        // Pre-filter for the same reason as `up_next`: a narrowed library must
        // not hide the "Practise your priorities" button (#981).
        let has_priorities = items.iter().any(|i| i.priority);

        // Cloned pre-filter for pickers that curate their own subset (#1484).
        let all_items = items.clone();

        let mut recently_practised = all_items.clone();
        recently_practised.retain(|i| {
            i.practice
                .as_ref()
                .is_some_and(|p| p.last_practiced_at.is_some())
        });
        sort_library_items(
            &mut recently_practised,
            &LibrarySort {
                field: SortField::LastPracticed,
                direction: SortDirection::Descending,
            },
        );
        recently_practised.truncate(RECENTLY_PRACTISED_LIMIT);

        if let Some(ref query) = model.active_query {
            items = apply_query_filter(items, query);
        }

        sort_library_items(&mut items, &model.active_sort);

        // Counted after the filter so the subtitle describes the visible set.
        let visible_pieces = items
            .iter()
            .filter(|i| i.item_type == ItemKind::Piece)
            .count();
        let visible_exercises = items
            .iter()
            .filter(|i| i.item_type == ItemKind::Exercise)
            .count();

        let labels = crate::view::session::variation_labels(&model.items);

        let mut sessions: Vec<_> = model
            .sessions
            .iter()
            .map(|s| session_to_view(s, &labels))
            .collect();
        sessions.sort_by(|a, b| b.finished_at.cmp(&a.finished_at));

        let (active_session, building_setlist, summary) = match &model.session_status {
            SessionStatus::Idle => (None, None, None),
            SessionStatus::Building(building) => {
                let entries: Vec<_> = building
                    .entries
                    .iter()
                    .map(|e| entry_to_view(e, &labels))
                    .collect();
                let item_count = entries.len();
                let blocks = build_blocks(&entries);
                let block_count = blocks.len();
                let planned_total_secs: u64 = building
                    .entries
                    .iter()
                    .filter_map(|e| e.planned_duration_secs)
                    .map(u64::from)
                    .sum();
                let (total_duration_display, total_duration_summary) = if planned_total_secs > 0 {
                    (
                        Some(crate::view::session::format_duration_display(
                            planned_total_secs,
                        )),
                        Some(crate::view::session::format_planned_duration(
                            planned_total_secs,
                        )),
                    )
                } else {
                    (None, None)
                };
                (
                    None,
                    Some(BuildingSetlistView {
                        entries,
                        item_count,
                        blocks,
                        block_count,
                        total_duration_display,
                        total_duration_summary,
                    }),
                    None,
                )
            }
            SessionStatus::Active(active) => {
                let current_entry = active.current_entry();
                // Looks up in all_items, not the filtered items: a Library
                // search must not empty the variation picker (#1484).
                let current_variations = all_items
                    .iter()
                    .find(|i| i.id == current_entry.item_id)
                    .map_or(&[][..], |i| i.variants.as_slice());
                (
                    Some(build_active_session_view(
                        active,
                        &item_index,
                        &labels,
                        current_variations,
                    )),
                    None,
                    None,
                )
            }
            SessionStatus::Summary(summary_session) => (
                None,
                None,
                Some(build_summary_view(summary_session, &labels)),
            ),
        };

        let practice_weeks = crate::practice_weeks::compute_practice_weeks(&model.sessions, clock);

        let (analytics, last_practised) = if model.sessions.is_empty() {
            (None, None)
        } else {
            (
                Some(compute_analytics(
                    &model.sessions,
                    &model.items,
                    &model.practice_summaries,
                    &all_items,
                    clock,
                )),
                crate::analytics::compute_last_practised(&model.sessions, clock),
            )
        };

        ViewModel {
            items,
            all_items,
            recently_practised,
            active_query: model.active_query.clone(),
            active_sort: model.active_sort,
            visible_pieces,
            visible_exercises,
            available_tags,
            available_composers,
            sessions,
            practice_weeks,
            active_session,
            building_setlist,
            summary,
            error: model.last_error.clone(),
            error_target: model.last_error_target.clone(),
            error_seq: model.error_seq,
            notice: model.last_notice.clone(),
            notice_seq: model.notice_seq,
            analytics,
            last_practised,
            profile: build_profile_view(&model.profile, clock.hour_of(now)),
            up_next,
            has_priorities,
            photo_recognition: photo_recognition_view(&model.photo_recognition),
            limits: LimitsView::default(),
        }
    }
}

fn vocabulary<S: AsRef<str>>(values: impl IntoIterator<Item = S>) -> Vec<String> {
    let mut words = crate::validation::distinct_ignoring_case(values);
    words.sort_by_key(|w| w.to_lowercase());
    words
}

fn photo_recognition_view(state: &crate::model::PhotoRecognition) -> PhotoRecognitionView {
    use crate::model::{PhotoRecognition, PhotoRecognitionStatus};

    match state {
        PhotoRecognition::Idle => PhotoRecognitionView::default(),
        PhotoRecognition::Reading { photo_id } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Reading,
            photo_id: Some(photo_id.clone()),
            draft: None,
        },
        PhotoRecognition::Ready { photo_id, draft } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Ready,
            photo_id: Some(photo_id.clone()),
            draft: Some(draft.clone()),
        },
        PhotoRecognition::Unsupported { photo_id } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Unsupported,
            photo_id: Some(photo_id.clone()),
            draft: None,
        },
        PhotoRecognition::Failed { photo_id } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Failed,
            photo_id: Some(photo_id.clone()),
            draft: None,
        },
    }
}

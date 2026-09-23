use crate::analytics::LocalClock;
use crate::domain::item::ItemKind;
use crate::domain::profile::build_profile_view;
use crate::domain::session::SessionStatus;
use crate::model::{
    BuildingSetlistView, EntryVariationsView, LimitsView, Model, PhotoRecognitionView, ViewModel,
};
use crate::view::cache::ProjectionKey;
use crate::view::library::matches_query;
use crate::view::session::{
    build_active_session_view, build_blocks, build_summary_view, entry_to_view, picker_variations,
};

pub mod cache;
pub mod library;
pub mod session;

use crate::app::Intrada;

impl Intrada {
    pub(crate) fn build_view(&self, model: &Model) -> ViewModel {
        build_view_at(model, chrono::Utc::now())
    }
}

/// `now` is the render's clock: the local day and the greeting's hour (#1694).
/// After a midnight with no event since the last `update`, the cache no longer
/// matches and this builds fresh without storing (#1998).
pub(crate) fn build_view_at(model: &Model, now: chrono::DateTime<chrono::Utc>) -> ViewModel {
    let clock = LocalClock::from_now(now, model.utc_offset_minutes);
    let fresh;
    let cached = match &model.projections {
        Some(p) if p.key == ProjectionKey::of(model, clock) => p,
        _ => {
            fresh = cache::build(model, clock);
            &fresh
        }
    };

    let items: Vec<_> = cached
        .sorted
        .iter()
        .map(|&i| cached.library[i].clone())
        .collect();
    let visible: Vec<_> = items
        .iter()
        .filter(|i| {
            model
                .active_query
                .as_ref()
                .is_none_or(|q| matches_query(i, q))
        })
        .collect();

    // Counted after the filter so the subtitle describes the visible set.
    let visible_pieces = visible
        .iter()
        .filter(|i| i.item_type == ItemKind::Piece)
        .count();
    let visible_exercises = visible
        .iter()
        .filter(|i| i.item_type == ItemKind::Exercise)
        .count();
    let visible_ids = visible.iter().map(|i| i.id.clone()).collect();

    let labels = crate::view::session::variation_labels(&model.items);

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
            let entry_variations = building
                .entries
                .iter()
                .filter_map(|entry| {
                    let item = cached.library.iter().find(|i| i.id == entry.item_id)?;
                    (!item.variants.is_empty()).then(|| EntryVariationsView {
                        entry_id: entry.id.clone(),
                        variations: picker_variations(entry, &item.variants),
                    })
                })
                .collect();
            (
                None,
                Some(BuildingSetlistView {
                    entries,
                    item_count,
                    blocks,
                    total_duration_display,
                    total_duration_summary,
                    entry_variations,
                }),
                None,
            )
        }
        SessionStatus::Active(active) => {
            let current_entry = active.current_entry();
            // Looks up in the whole library, not the filtered items: a
            // Library search must not empty the variation picker (#1484).
            let current_variations = cached
                .library
                .iter()
                .find(|i| i.id == current_entry.item_id)
                .map_or(&[][..], |i| i.variants.as_slice());
            let item_index: std::collections::HashMap<&str, &crate::domain::item::Item> =
                model.items.iter().map(|i| (i.id.as_str(), i)).collect();
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
            Some(build_summary_view(
                summary_session,
                &labels,
                &cached.score_changes,
            )),
        ),
    };

    ViewModel {
        items,
        active_query: model.active_query.clone(),
        active_sort: model.active_sort,
        visible_pieces,
        visible_exercises,
        available_tags: cached.available_tags.clone(),
        available_composers: cached.available_composers.clone(),
        sessions: cached.sessions.clone(),
        practice_weeks: cached.practice_weeks.clone(),
        active_session,
        building_setlist,
        summary,
        error: model.last_error.clone(),
        error_target: model.last_error_target.clone(),
        error_seq: model.error_seq,
        notice: model.last_notice.clone(),
        notice_seq: model.notice_seq,
        analytics: cached.analytics.clone(),
        last_practised: cached.last_practised.clone(),
        profile: build_profile_view(&model.profile, clock.hour_of(now)),
        up_next: cached.up_next.clone(),
        has_priorities: cached.has_priorities,
        photo_recognition: photo_recognition_view(&model.photo_recognition),
        limits: LimitsView::default(),
        visible_ids,
        recently_practised_ids: cached.recently_practised_ids.clone(),
        shows_priorities: cached.has_priorities
            && matches!(model.session_status, SessionStatus::Idle),
    }
}

fn photo_recognition_view(state: &crate::model::PhotoRecognition) -> PhotoRecognitionView {
    use crate::model::{PhotoRecognition, PhotoRecognitionStatus};

    match state {
        PhotoRecognition::Idle => PhotoRecognitionView::default(),
        PhotoRecognition::Reading { photo_id } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Reading,
            photo_id: Some(photo_id.clone()),
            draft: None,
            read_nothing: false,
        },
        PhotoRecognition::Ready { photo_id, draft } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Ready,
            photo_id: Some(photo_id.clone()),
            draft: Some(draft.clone()),
            read_nothing: *draft == crate::recognition::PhotoDraft::default(),
        },
        PhotoRecognition::Failed { photo_id } => PhotoRecognitionView {
            status: PhotoRecognitionStatus::Failed,
            photo_id: Some(photo_id.clone()),
            draft: None,
            read_nothing: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::photo_recognition_view;
    use crate::domain::types::Tempo;
    use crate::model::PhotoRecognition;
    use crate::recognition::{DraftSource, PhotoDraft, TempoDraftField, TextDraftField};

    fn text(value: &str) -> Option<TextDraftField> {
        Some(TextDraftField {
            value: value.to_string(),
            source: DraftSource::Recognised,
            confidence: 0.9,
            weak: false,
        })
    }

    fn ready(draft: PhotoDraft) -> PhotoRecognition {
        PhotoRecognition::Ready {
            photo_id: "photo".to_string(),
            draft,
        }
    }

    #[test]
    fn a_finished_read_that_found_nothing_says_so() {
        let tempo = Some(TempoDraftField {
            value: Tempo {
                marking: None,
                bpm: Some(96),
            },
            source: DraftSource::Recognised,
            confidence: 0.9,
            weak: false,
        });
        let cases: Vec<(PhotoRecognition, bool, &str)> = vec![
            (ready(PhotoDraft::default()), true, "an empty draft"),
            (
                ready(PhotoDraft {
                    title: text("Autumn Leaves"),
                    ..Default::default()
                }),
                false,
                "a title alone",
            ),
            (
                ready(PhotoDraft {
                    composer: text("Kosma"),
                    ..Default::default()
                }),
                false,
                "a composer alone",
            ),
            (
                ready(PhotoDraft {
                    tempo,
                    ..Default::default()
                }),
                false,
                "a tempo alone",
            ),
            (
                ready(PhotoDraft {
                    chart_text: text("| Cm7 | F7 |"),
                    ..Default::default()
                }),
                false,
                "a chart alone",
            ),
            (PhotoRecognition::Idle, false, "no read"),
            (
                PhotoRecognition::Reading {
                    photo_id: "photo".to_string(),
                },
                false,
                "still reading",
            ),
            (
                PhotoRecognition::Failed {
                    photo_id: "photo".to_string(),
                },
                false,
                "a failed read is its own state",
            ),
        ];
        for (state, expected, why) in cases {
            let view = photo_recognition_view(&state);
            assert_eq!(view.read_nothing, expected, "{why}");
            crate::domain::types::assert_round_trips(view);
        }
    }
}

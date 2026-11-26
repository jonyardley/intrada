use crate::analytics::LocalClock;
use crate::domain::first_run::build_first_run_view;
use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::profile::build_profile_view;
use crate::domain::session::{
    item_reads_as, last_score, last_time, opening_way, start_score, PlayWay, SessionStatus,
};
use crate::model::{
    BuildingSetlistView, EntryVariationsView, LimitsView, Model, SetlistBlockView, ViewModel,
};
use crate::view::cache::ProjectionKey;
use crate::view::library::{library_count_line, matches_query};
use crate::view::session::{
    build_active_session_view, build_blocks, build_summary_view, entry_to_view, last_time_view,
    picker_variations, plan_tags,
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

    let visible: Vec<_> = cached
        .rows()
        .filter(|i| {
            model
                .active_query
                .as_ref()
                .is_none_or(|q| matches_query(i, q))
        })
        .collect();

    // Counted after the filter so the header line describes the visible set.
    let visible_pieces = visible
        .iter()
        .filter(|i| i.item_type == ItemKind::Piece)
        .count();
    let visible_exercises = visible
        .iter()
        .filter(|i| i.item_type == ItemKind::Exercise)
        .count();
    let visible_ids = visible.iter().map(|i| i.id.clone()).collect();

    let labels = crate::view::session::play_labels(&model.variations);

    let (active_session, building_setlist, summary) = match &model.session_status {
        SessionStatus::Idle => (None, None, None),
        SessionStatus::Building(building) => {
            let entries: Vec<_> = building
                .entries
                .iter()
                .map(|e| {
                    let mut view = entry_to_view(e, &labels);
                    view.plan_tags = plan_tags(
                        e.plan.key.map(|k| k.label()),
                        live_variation_labels(&cached.library, e).into_iter(),
                    );
                    view
                })
                .collect();
            let item_count = entries.len();
            let blocks = build_blocks(&entries);
            let planned_total_secs: u64 = building
                .entries
                .iter()
                .filter_map(|e| e.plan.duration_secs)
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
                    let item = cached.library.iter().find(|i| i.id == entry.plan.item_id)?;
                    (!item.variations.is_empty()).then(|| EntryVariationsView {
                        entry_id: entry.id.clone(),
                        variations: picker_variations(entry, &item.variations),
                    })
                })
                .collect();
            let last_times = building
                .entries
                .iter()
                // Decided 2026-10-04 (#2249, spec): an offer only while nothing is planned.
                .filter(|entry| {
                    let item_id = &entry.plan.item_id;
                    entry.plan.variation_ids.is_empty()
                        && Key::same_or_written(
                            item_reads_as(model, item_id, entry.plan.key),
                            opening_way(model, item_id).key,
                        )
                })
                .filter_map(|entry| {
                    let way = last_time(model, &entry.plan.item_id)?;
                    Some(last_time_view(entry, way, &labels))
                })
                .collect();
            let entry_keys = building
                .entries
                .iter()
                .filter_map(|entry| {
                    let item = model.items.iter().find(|i| i.id == entry.plan.item_id)?;
                    crate::view::session::entry_keys_view(entry, item)
                })
                .collect();
            let summary = builder_summary(total_duration_summary.as_deref(), &blocks, item_count);
            let start_title = match &total_duration_summary {
                Some(total) => format!("Start session \u{b7} {total}"),
                None => "Start session".to_string(),
            };
            (
                None,
                Some(BuildingSetlistView {
                    entries,
                    item_count,
                    blocks,
                    total_duration_display,
                    total_duration_summary,
                    length_mins: building.length_mins,
                    length_summary: crate::view::session::format_length_summary(
                        building.length_mins,
                        planned_total_secs,
                    ),
                    entry_variations,
                    last_times,
                    entry_keys,
                    summary,
                    start_title,
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
                .find(|i| i.id == current_entry.plan.item_id)
                .map_or(&[][..], |i| i.variations.as_slice());
            let item_index: std::collections::HashMap<&str, &crate::domain::item::Item> =
                model.items.iter().map(|i| (i.id.as_str(), i)).collect();
            let mut view = build_active_session_view(
                active,
                &item_index,
                &labels,
                current_variations,
                &model.practice_defaults,
            );
            // Offered only before the musician has chosen a way (#2249).
            let item_id = &current_entry.plan.item_id;
            let played_plain = current_entry.open_play().is_some_and(|p| {
                PlayWay {
                    key: item_reads_as(model, item_id, p.key),
                    ..p.way()
                }
                .is_same_way(&opening_way(model, item_id))
            });
            view.record.last_time = last_time(model, item_id)
                .filter(|_| played_plain && active.reflection.is_none())
                .map(|way| last_time_view(current_entry, way, &labels));
            if let (Some(sheet), Some(draft)) =
                (view.reflection.as_mut(), active.reflection.as_ref())
            {
                for row in &mut sheet.tempos {
                    if let Some(play) = current_entry.play(&row.play_id) {
                        let way = draft.answers.way_of(play);
                        row.last_score = last_score(model, item_id, &way);
                        row.start_score = start_score(model, item_id, &way);
                    }
                }
            }
            if let (Some(finish), Some(item), Some(draft)) = (
                view.record.finish.as_mut(),
                model
                    .items
                    .iter()
                    .find(|i| i.id == current_entry.plan.item_id),
                active.reflection.as_ref(),
            ) {
                crate::view::session::fill_finish_choices(
                    finish,
                    current_entry,
                    draft,
                    item,
                    !current_variations.is_empty(),
                    &labels,
                );
            }
            (Some(view), None, None)
        }
        SessionStatus::Summary(summary_session) => (
            None,
            None,
            Some(build_summary_view(
                summary_session,
                &labels,
                &model.practice_summaries,
            )),
        ),
    };

    ViewModel {
        active_query: model.active_query.clone(),
        active_sort: model.active_sort,
        library_count_line: library_count_line(
            visible_pieces,
            visible_exercises,
            model.active_query.is_some(),
        ),
        available_tags: cached.available_tags.clone(),
        available_composers: cached.available_composers.clone(),
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
        profile: build_profile_view(
            &model.profile,
            model.profile_draft.as_ref(),
            clock.hour_of(now),
        ),
        limits: LimitsView::default(),
        visible_ids,
        recently_practised_ids: cached.recently_practised_ids.clone(),
        offers_recovery: !matches!(
            model.session_status,
            SessionStatus::Active(_) | SessionStatus::Summary(_)
        ),
        practice_defaults: model.practice_defaults,
        first_run: build_first_run_view(model),
        variations: cached.variation_options.clone(),
        undo_delete: crate::domain::session::undo_offer(model),
        suggested_plan: if matches!(model.session_status, SessionStatus::Idle) {
            crate::plan::fill(
                &cached.library,
                &model.practice_defaults,
                model.plan_length(),
                clock,
            )
        } else {
            None
        },
    }
}

/// The screen state plus the two sections sent apart from it (#1801), so a
/// test reads the rows the shell would hold.
#[cfg(test)]
#[derive(Debug, PartialEq)]
pub(crate) struct Rendered {
    pub(crate) view: ViewModel,
    pub(crate) items: Vec<crate::model::LibraryItemView>,
    pub(crate) sessions: Vec<crate::model::PracticeSessionView>,
    pub(crate) practice_weeks: Vec<crate::practice_weeks::PracticeWeekView>,
}

#[cfg(test)]
impl std::ops::Deref for Rendered {
    type Target = ViewModel;
    fn deref(&self) -> &ViewModel {
        &self.view
    }
}

#[cfg(test)]
impl Intrada {
    pub(crate) fn rendered(&self, model: &Model) -> Rendered {
        rendered(model)
    }
}

#[cfg(test)]
pub(crate) fn rendered(model: &Model) -> Rendered {
    rendered_at(model, chrono::Utc::now())
}

#[cfg(test)]
pub(crate) fn rendered_at(model: &Model, now: chrono::DateTime<chrono::Utc>) -> Rendered {
    let clock = LocalClock::from_now(now, model.utc_offset_minutes);
    let fresh;
    let cached = match &model.projections {
        Some(p) if p.key == ProjectionKey::of(model, clock) => p,
        _ => {
            fresh = cache::build(model, clock);
            &fresh
        }
    };
    Rendered {
        view: build_view_at(model, now),
        items: cached.rows().cloned().collect(),
        sessions: cached.sessions.clone(),
        practice_weeks: cached.practice_weeks.clone(),
    }
}

fn live_variation_labels(
    library: &[crate::model::LibraryItemView],
    entry: &crate::domain::session::SetlistEntry,
) -> Vec<String> {
    let live = library
        .iter()
        .find(|i| i.id == entry.plan.item_id)
        .map_or(&[][..], |i| i.variations.as_slice());
    entry
        .plan
        .variation_ids
        .iter()
        .filter_map(|id| live.iter().find(|v| &v.id == id).map(|v| v.label.clone()))
        .collect()
}

fn builder_summary(total: Option<&str>, blocks: &[SetlistBlockView], items: usize) -> String {
    let plural = |n: usize, one: &str| format!("{n} {one}{}", if n == 1 { "" } else { "s" });
    let grouped = blocks.iter().any(|b| b.group_id.is_some());
    total
        .map(str::to_string)
        .into_iter()
        .chain(grouped.then(|| plural(blocks.len(), "block")))
        .chain([plural(items, "item")])
        .collect::<Vec<_>>()
        .join(" \u{b7} ")
}

#[cfg(test)]
mod tests {
    use crate::app::{Event, Intrada};
    use crate::model::Model;
    use crux_core::App;

    #[test]
    fn a_variation_says_which_items_use_it() {
        let mut model = Model::default();
        let _ = Intrada.update(Event::LoadSampleData, &mut model);
        let ids: Vec<String> = model.variations.iter().map(|v| v.id.clone()).collect();
        for item in model.items.iter_mut() {
            item.variation_ids.clear();
        }
        model.items[0].variation_ids = vec![ids[0].clone(), ids[1].clone()];
        model.items[1].variation_ids = vec![ids[1].clone()];
        let first_title = model.items[0].title.clone();

        let usage = |id: &str| {
            Intrada
                .rendered(&model)
                .variations
                .iter()
                .find(|v| v.id == id)
                .and_then(|v| v.usage.clone())
        };

        assert_eq!(usage(&ids[0]), Some(format!("On {first_title}")));
        assert_eq!(usage(&ids[1]).as_deref(), Some("On 2 items"));
        assert_eq!(usage(&ids[2]), None);
    }
}

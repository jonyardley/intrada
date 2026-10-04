use super::plays::*;
use super::*;
use crate::app::{Effect, Event};
use crate::model::Model;
use crux_core::Command;

/// The item's live named sections as (id, name), for reading its notes.
pub(crate) fn named_sections<'a>(model: &'a Model, item_id: &str) -> Vec<(&'a str, &'a str)> {
    model
        .items
        .iter()
        .find(|i| i.id == item_id)
        .map(|i| {
            i.sections
                .iter()
                .filter(|s| s.deleted_at.is_none() && !s.name.is_empty())
                .map(|s| (s.id.as_str(), s.name.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

/// A finish answer lands like a mark: on a completed entry, in the sheet's
/// hand-off after `NextItem` or on the summary.
fn answer(
    model: &mut Model,
    entry_id: &str,
    apply: impl FnOnce(&mut SetlistEntry),
) -> Command<Effect, Event> {
    let Some(entry) = entry_for_update_mut(model, entry_id) else {
        return crux_core::render::render();
    };
    if entry.status != EntryStatus::Completed {
        return crux_core::render::render();
    }
    apply(entry);
    model.last_error = None;
    persist_if_active(model)
}

pub(super) fn set_felt(
    model: &mut Model,
    entry_id: String,
    felt: Option<Felt>,
) -> Command<Effect, Event> {
    answer(model, &entry_id, |entry| entry.felt = felt)
}

pub(super) fn toggle_obstacle(
    model: &mut Model,
    entry_id: String,
    obstacle: Obstacle,
) -> Command<Effect, Event> {
    answer(model, &entry_id, |entry| {
        if let Some(at) = entry.got_in_the_way.iter().position(|o| *o == obstacle) {
            entry.got_in_the_way.remove(at);
        } else {
            entry.got_in_the_way.push(obstacle);
        }
    })
}

pub(super) fn answer_intention(
    model: &mut Model,
    entry_id: String,
    answer_given: Option<IntentionMet>,
) -> Command<Effect, Event> {
    answer(model, &entry_id, |entry| {
        if intention_met_read(entry).is_none() {
            entry.intention_met = answer_given;
        }
    })
}

/// Stores the point the entry's note offers at `span`, once. A span the note
/// offers nothing at is not a point and writes nothing.
pub(super) fn confirm_note_point(
    model: &mut Model,
    entry_id: String,
    span: NoteSpan,
) -> Command<Effect, Event> {
    let Some(entry) = entry_for_plan(model, &entry_id) else {
        return crux_core::render::render();
    };
    let sections = named_sections(model, &entry.item_id);
    let offered = note_offers(entry.notes.as_deref().unwrap_or_default(), &sections)
        .into_iter()
        .find(|p| p.span == span);
    let Some(point) = offered else {
        return crux_core::render::render();
    };
    answer(model, &entry_id, |entry| {
        let held = entry.note_points.iter().any(|p| p.span == span);
        if !held && entry.note_points.len() < validation::MAX_NOTE_POINTS {
            entry.note_points.push(point);
        }
    })
}

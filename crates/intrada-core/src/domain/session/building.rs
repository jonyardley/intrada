use super::plays::*;
use super::*;
use crate::app::{Effect, Event};
use crate::domain::item::ItemKind;
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;

pub(super) fn create_entry(
    item_id: &str,
    item_title: &str,
    item_type: ItemKind,
    position: usize,
) -> SetlistEntry {
    SetlistEntry {
        id: ulid::Ulid::generate().to_string(),
        item_id: item_id.to_string(),
        item_title: item_title.to_string(),
        item_type,
        position,
        duration_secs: 0,
        status: EntryStatus::NotAttempted,
        notes: None,
        intention: None,
        planned_duration_secs: None,
        group_id: None,
        planned_variation_ids: Vec::new(),
        planned_rep_target: None,
        plays: Vec::new(),
        segments: Vec::new(),
        focus: None,
        intention_met: None,
        felt: None,
        got_in_the_way: Vec::new(),
        note_points: Vec::new(),
    }
}

pub(super) fn reindex_entries(entries: &mut [SetlistEntry]) {
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.position = i;
    }
}

/// Partition entries into ordered units: a contiguous run sharing a `Some`
/// `group_id` is one unit (a block); every `None` entry is its own unit.
pub(super) fn into_units(entries: Vec<SetlistEntry>) -> Vec<Vec<SetlistEntry>> {
    let mut units: Vec<Vec<SetlistEntry>> = Vec::new();
    for entry in entries {
        match &entry.group_id {
            Some(g) => {
                let extends = units
                    .last()
                    .and_then(|u| u.first())
                    .and_then(|e| e.group_id.as_deref())
                    == Some(g.as_str());
                if extends {
                    units.last_mut().expect("checked above").push(entry);
                } else {
                    units.push(vec![entry]);
                }
            }
            None => units.push(vec![entry]),
        }
    }
    units
}

/// Clear the `group_id` of any block left without its anchor piece. A block
/// only means "this piece's warm-up", so when the piece goes the related
/// exercises become standalone (§7.4 dissolve).
pub(super) fn dissolve_pieceless_groups(entries: &mut [SetlistEntry]) {
    let anchored: std::collections::HashSet<&str> = entries
        .iter()
        .filter(|e| e.item_type == ItemKind::Piece)
        .filter_map(|e| e.group_id.as_deref())
        .collect();
    let orphans: std::collections::HashSet<String> = entries
        .iter()
        .filter_map(|e| e.group_id.clone())
        .filter(|g| !anchored.contains(g.as_str()))
        .collect();
    for entry in entries.iter_mut() {
        if entry
            .group_id
            .as_deref()
            .is_some_and(|g| orphans.contains(g))
        {
            entry.group_id = None;
        }
    }
}

fn fresh_building(model: &Model) -> BuildingSession {
    BuildingSession {
        entries: Vec::new(),
        length_mins: model.practice_defaults.session_length_mins,
    }
}

pub(super) fn start_building(model: &mut Model) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("A practice is already in progress".to_string());
        return crux_core::render::render();
    }
    model.session_status = SessionStatus::Building(fresh_building(model));
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn set_entry_intention(
    model: &mut Model,
    entry_id: String,
    intention: Option<String>,
) -> Command<Effect, Event> {
    let check = validation::validate_intention(&intention);
    set_planned(model, &entry_id, check, |entry| entry.intention = intention)
}

pub(super) fn set_entry_plan(
    model: &mut Model,
    entry_id: String,
    section_ids: Vec<String>,
    variation_ids: Vec<String>,
) -> Command<Effect, Event> {
    // The plan is a Building-phase thing: once practice starts, the
    // record is the plays and `SwitchPlay` is what changes it (#1739
    // decision 5).
    if !matches!(model.session_status, SessionStatus::Building(_)) {
        model.raise_error("A plan can only be set while building".to_string());
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_plan(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };

    if let Err(e) = validation::validate_entry_plan(entry, &section_ids, &variation_ids, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_plan_mut(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };
    let mut segments: Vec<Segment> = section_ids
        .into_iter()
        .map(|section_id| Segment {
            section_id,
            planned_secs: 0,
        })
        .collect();
    split_evenly(&mut segments, entry.planned_duration_secs);
    entry.segments = segments;
    entry.planned_variation_ids = variation_ids;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn set_rep_target(
    model: &mut Model,
    entry_id: String,
    target: Option<u8>,
) -> Command<Effect, Event> {
    let check = validation::validate_rep_target(&target);
    set_planned(model, &entry_id, check, |entry| {
        entry.planned_rep_target = target;
    })
}

pub(super) fn set_entry_duration(
    model: &mut Model,
    entry_id: String,
    duration_secs: Option<u32>,
) -> Command<Effect, Event> {
    let check = validation::validate_planned_duration(&duration_secs);
    set_planned(model, &entry_id, check, |entry| {
        entry.planned_duration_secs = duration_secs;
        split_evenly(&mut entry.segments, duration_secs);
    })
}

pub(super) fn set_segments(
    model: &mut Model,
    entry_id: String,
    mut segments: Vec<Segment>,
) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Building(_)) {
        model.raise_error("Segments can only be set while building".to_string());
        return crux_core::render::render();
    }
    let Some(entry) = entry_for_plan(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };
    let check = validation::validate_segment_sections(entry, &segments, model)
        .and_then(|()| rebalance(&mut segments, entry.planned_duration_secs));
    set_planned(model, &entry_id, check, |entry| entry.segments = segments)
}

pub(super) fn set_focus(
    model: &mut Model,
    entry_id: String,
    focus: Option<IntentionFocus>,
) -> Command<Effect, Event> {
    let check = match (&focus, entry_for_plan(model, &entry_id)) {
        (Some(focus), Some(entry)) => {
            let sections = live_section_ids(model, &entry.item_id);
            validate_focus(focus, &sections)
        }
        _ => Ok(()),
    };
    set_planned(model, &entry_id, check, |entry| entry.focus = focus)
}

fn live_section_ids<'a>(model: &'a Model, item_id: &str) -> Vec<&'a str> {
    model
        .items
        .iter()
        .find(|i| i.id == item_id)
        .map(|i| {
            i.sections
                .iter()
                .filter(|s| s.deleted_at.is_none())
                .map(|s| s.id.as_str())
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn apply_last_time(model: &mut Model, entry_id: String) -> Command<Effect, Event> {
    let way = entry_for_plan(model, &entry_id).and_then(|e| last_time(model, &e.item_id));
    let Some(way) = way else {
        model.raise_error("There's no last time to plan from".to_string());
        return crux_core::render::render();
    };
    set_planned(model, &entry_id, Ok(()), |entry| {
        let mut segments: Vec<Segment> = way
            .section_id
            .into_iter()
            .map(|section_id| Segment {
                section_id,
                planned_secs: 0,
            })
            .collect();
        split_evenly(&mut segments, entry.planned_duration_secs);
        entry.segments = segments;
        entry.planned_variation_ids = way.variation_ids;
    })
}

/// Kept to what is still in the library; the whole piece, plain, offers nothing.
pub(crate) fn last_time(model: &Model, item_id: &str) -> Option<PlayWay> {
    let play = model
        .sessions
        .iter()
        .filter(|s| s.entries.iter().any(|e| e.item_id == item_id))
        .max_by_key(|s| s.started_at)?
        .entries
        .iter()
        .filter(|e| e.item_id == item_id)
        .flat_map(|e| e.plays.iter())
        .rev()
        .find(|p| !p.is_incidental())?;
    let sections = live_section_ids(model, item_id);
    let live_variation = |id: &String| {
        model
            .variations
            .iter()
            .any(|v| &v.id == id && v.deleted_at.is_none())
    };
    let way = PlayWay {
        section_id: play
            .section_id
            .clone()
            .filter(|id| sections.contains(&id.as_str())),
        key: None,
        variation_ids: play
            .variation_ids
            .iter()
            .filter(|id| live_variation(id))
            .cloned()
            .collect(),
    };
    (way.section_id.is_some() || !way.variation_ids.is_empty()).then_some(way)
}

pub(super) fn set_session_length(
    model: &mut Model,
    length_mins: Option<u16>,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("A session length can only be set while building".to_string());
        return crux_core::render::render();
    };
    if let Err(e) = validation::validate_session_length(&length_mins) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }
    building.length_mins = length_mins;
    model.last_error = None;
    crux_core::render::render()
}

fn set_planned(
    model: &mut Model,
    entry_id: &str,
    check: Result<(), LibraryError>,
    apply: impl FnOnce(&mut SetlistEntry),
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("An entry can only be planned while building".to_string());
        return crux_core::render::render();
    };

    if let Err(e) = check {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };

    apply(entry);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn start_building_with(model: &mut Model, item_id: String) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("A practice is already in progress".to_string());
        return crux_core::render::render();
    }
    if !model.items.iter().any(|i| i.id == item_id) {
        model.raise_error(LibraryError::NotFound { id: item_id }.to_string());
        return crux_core::render::render();
    }
    model.session_status = SessionStatus::Building(fresh_building(model));
    handle_session_event(SessionEvent::AddToSetlist { item_id }, model)
}

pub(super) fn start_building_from_suggestion(
    model: &mut Model,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    seed_from_suggestion(model, now);
    crux_core::render::render()
}

pub(super) fn start_from_suggestion(
    model: &mut Model,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    if seed_from_suggestion(model, now) {
        start_session(model, now)
    } else {
        crux_core::render::render()
    }
}

/// False when nothing was seeded; nothing to suggest is not an error (#1082).
fn seed_from_suggestion(model: &mut Model, now: DateTime<Utc>) -> bool {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("A practice is already in progress".to_string());
        return false;
    }
    let Some(plan) = crate::view::library::derive_up_next(model, now) else {
        return false;
    };

    let mut building = fresh_building(model);
    for block in &plan.blocks {
        let group_id = ulid::Ulid::generate().to_string();
        for suggested in &block.items {
            let position = building.entries.len();
            let mut entry = create_entry(
                &suggested.item_id,
                &suggested.item_title,
                suggested.item_type.clone(),
                position,
            );
            entry.group_id = Some(group_id.clone());
            building.entries.push(entry);
        }
    }

    model.session_status = SessionStatus::Building(building);
    model.last_error = None;
    true
}

pub(super) fn start_building_with_priorities(
    model: &mut Model,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("A practice is already in progress".to_string());
        return crux_core::render::render();
    }
    // Nothing starred is not an error, for the same reason as the Up
    // next CTA: the button cannot be on screen, and a race must not
    // strand an empty builder.
    let ordered = crate::view::library::derive_priorities(model, now);
    if ordered.is_empty() {
        return crux_core::render::render();
    }

    model.session_status = SessionStatus::Building(fresh_building(model));
    // `AddToSetlist` owns block formation, deduping and idempotency
    // (#939), so seeding folds over it rather than building entries a
    // second way. Every step's command is kept: dropping all but the
    // last would silently swallow any effect it grows later.
    Command::all(
        ordered
            .into_iter()
            .map(|item_id| handle_session_event(SessionEvent::AddToSetlist { item_id }, model))
            .collect::<Vec<_>>(),
    )
}

pub(super) fn add_to_setlist(model: &mut Model, item_id: String) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Building(_)) {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    }

    // Membership is binary (the picker/sheet toggle relies on it):
    // re-adding a present item is an idempotent no-op, not a duplicate (#939).
    if let SessionStatus::Building(ref building) = model.session_status {
        if building.entries.iter().any(|e| e.item_id == item_id) {
            model.last_error = None;
            return crux_core::render::render();
        }
    }

    // Resolve the item and, for a piece, its related exercises as owned
    // tuples before taking the mutable Building borrow.
    let Some(item) = model.items.iter().find(|i| i.id == item_id) else {
        model.raise_error(LibraryError::NotFound { id: item_id }.to_string());
        return crux_core::render::render();
    };
    let piece = (item.id.clone(), item.title.clone(), item.kind.clone());
    let related: Vec<(String, String, ItemKind)> = if item.kind == ItemKind::Piece {
        item.linked_exercise_ids()
            .iter()
            .filter_map(|ex_id| {
                model
                    .items
                    .iter()
                    .find(|i| &i.id == ex_id && i.kind == ItemKind::Exercise)
                    .map(|i| (i.id.clone(), i.title.clone(), i.kind.clone()))
            })
            .collect()
    } else {
        Vec::new()
    };

    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Internal error: expected Building state".to_string());
        return crux_core::render::render();
    };

    // Skip related exercises already in the setlist; don't duplicate.
    let existing: std::collections::HashSet<String> =
        building.entries.iter().map(|e| e.item_id.clone()).collect();
    let related_to_add: Vec<(String, String, ItemKind)> = related
        .into_iter()
        .filter(|(id, _, _)| !existing.contains(id))
        .collect();

    // A block forms only when ≥1 related exercise actually comes along.
    let group_id = if related_to_add.is_empty() {
        None
    } else {
        Some(ulid::Ulid::generate().to_string())
    };

    // Related first (warm-up order), then the piece.
    for (id, title, kind) in &related_to_add {
        let position = building.entries.len();
        let mut entry = create_entry(id, title, kind.clone(), position);
        entry.group_id.clone_from(&group_id);
        building.entries.push(entry);
    }
    let position = building.entries.len();
    let mut piece_entry = create_entry(&piece.0, &piece.1, piece.2, position);
    piece_entry.group_id = group_id;
    building.entries.push(piece_entry);

    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn remove_from_setlist(model: &mut Model, entry_id: String) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let len_before = building.entries.len();
    building.entries.retain(|e| e.id != entry_id);

    if building.entries.len() == len_before {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    }

    dissolve_pieceless_groups(&mut building.entries);
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn move_unit(
    model: &mut Model,
    entry_id: &str,
    new_position: usize,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let mut units = into_units(std::mem::take(&mut building.entries));
    let Some(current) = units
        .iter()
        .position(|u| u.iter().any(|e| e.id == entry_id))
    else {
        building.entries = units.into_iter().flatten().collect();
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };

    let target = new_position.min(units.len().saturating_sub(1));
    let unit = units.remove(current);
    units.insert(target, unit);
    building.entries = units.into_iter().flatten().collect();
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn move_related(
    model: &mut Model,
    entry_id: &str,
    new_position: usize,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let Some(entry) = building.entries.iter().find(|e| e.id == entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };
    let (Some(group_id), ItemKind::Exercise) = (entry.group_id.clone(), &entry.item_type) else {
        model.raise_error("Only a related exercise moves within its block".to_string());
        return crux_core::render::render();
    };

    let related: Vec<usize> = building
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.group_id.as_deref() == Some(group_id.as_str()) && e.item_type == ItemKind::Exercise
        })
        .map(|(i, _)| i)
        .collect();
    let (Some(&destination), Some(&from)) = (
        related.get(new_position),
        related
            .iter()
            .find(|&&i| building.entries[i].id == entry_id),
    ) else {
        let msg = format!(
            "Invalid position: {new_position} (max: {})",
            related.len().saturating_sub(1)
        );
        model.raise_error(msg);
        return crux_core::render::render();
    };

    // Both indices sit inside one contiguous block, so the move cannot split it.
    let moved = building.entries.remove(from);
    building.entries.insert(destination, moved);
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn keep_only_piece(model: &mut Model, group_id: String) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let in_block = |e: &SetlistEntry| e.group_id.as_deref() == Some(group_id.as_str());
    if !building.entries.iter().any(in_block) {
        model.raise_error(format!("Block '{group_id}' not found in setlist"));
        return crux_core::render::render();
    }

    building
        .entries
        .retain(|e| !(in_block(e) && e.item_type == ItemKind::Exercise));
    // The lone piece left behind is no longer a block.
    for entry in building.entries.iter_mut().filter(|e| in_block(e)) {
        entry.group_id = None;
    }
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn ungroup_block(model: &mut Model, group_id: String) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let mut found = false;
    for entry in building
        .entries
        .iter_mut()
        .filter(|e| e.group_id.as_deref() == Some(group_id.as_str()))
    {
        entry.group_id = None;
        found = true;
    }
    if !found {
        model.raise_error(format!("Block '{group_id}' not found in setlist"));
        return crux_core::render::render();
    }
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn ungroup_all_blocks(model: &mut Model) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };
    for entry in &mut building.entries {
        entry.group_id = None;
    }
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn remove_block(model: &mut Model, group_id: String) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let len_before = building.entries.len();
    building
        .entries
        .retain(|e| e.group_id.as_deref() != Some(group_id.as_str()));
    if building.entries.len() == len_before {
        model.raise_error(format!("Block '{group_id}' not found in setlist"));
        return crux_core::render::render();
    }
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn add_exercise_to_block(
    model: &mut Model,
    group_id: String,
    item_id: String,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    // Membership is binary, same idempotency as `AddToSetlist` (#939).
    if building.entries.iter().any(|e| e.item_id == item_id) {
        model.last_error = None;
        return crux_core::render::render();
    }

    let Some(anchor_index) = building.entries.iter().position(|e| {
        e.group_id.as_deref() == Some(group_id.as_str()) && e.item_type == ItemKind::Piece
    }) else {
        model.raise_error(format!("Block '{group_id}' not found in setlist"));
        return crux_core::render::render();
    };

    let Some(item) = model.items.iter().find(|i| i.id == item_id) else {
        model.raise_error(LibraryError::NotFound { id: item_id }.to_string());
        return crux_core::render::render();
    };
    if item.kind != ItemKind::Exercise {
        model.raise_error("Only an exercise can be added to a block".to_string());
        return crux_core::render::render();
    }
    let (id, title, kind) = (item.id.clone(), item.title.clone(), item.kind.clone());

    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Internal error: expected Building state".to_string());
        return crux_core::render::render();
    };
    let mut entry = create_entry(&id, &title, kind, anchor_index);
    entry.group_id = Some(group_id);
    building.entries.insert(anchor_index, entry);
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn start_session(model: &mut Model, now: DateTime<Utc>) -> Command<Effect, Event> {
    let SessionStatus::Building(ref building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    if let Err(e) = validation::validate_entries_not_empty(&building.entries, "Setlist") {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let mut active = ActiveSession {
        id: ulid::Ulid::generate().to_string(),
        entries: building.entries.clone(),
        current_index: 0,
        current_item_started_at: now,
        session_started_at: now,
        reflection: None,
        segment: None,
    };

    if let Some(entry) = active.entries.first_mut() {
        open_first_play(entry, now);
    }
    active.segment = first_segment_clock(active.current_entry(), now);

    let persist = persist_active(&active);
    model.session_status = SessionStatus::Active(active);
    model.last_error = None;
    persist
}

pub(super) fn cancel_building(model: &mut Model) -> Command<Effect, Event> {
    // Idempotent: already-Idle cancel is a no-op success, not a silent error (#944).
    match model.session_status {
        SessionStatus::Building(_) | SessionStatus::Idle => {
            model.session_status = SessionStatus::Idle;
            model.last_error = None;
        }
        _ => {
            model.raise_error("Not in building state".to_string());
        }
    }
    crux_core::render::render()
}

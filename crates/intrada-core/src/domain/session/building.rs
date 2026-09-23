use super::plays::*;
use super::*;
use crate::app::{AppEffect, Effect, Event};
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
        planned_variation_id: None,
        planned_rep_target: None,
        plays: Vec::new(),
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

/// True when every `group_id` occupies a single contiguous run: the block
/// invariant a reorder must never break.
pub(super) fn groups_contiguous(entries: &[SetlistEntry]) -> bool {
    let mut closed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut current: Option<&str> = None;
    for entry in entries {
        let g = entry.group_id.as_deref();
        if g != current {
            if let Some(prev) = current {
                closed.insert(prev);
            }
            if let Some(g) = g {
                if closed.contains(g) {
                    return false;
                }
            }
            current = g;
        }
    }
    true
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

pub(super) fn start_building(model: &mut Model) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("A practice is already in progress".to_string());
        return crux_core::render::render();
    }
    model.session_status = SessionStatus::Building(BuildingSession::default());
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn set_entry_intention(
    model: &mut Model,
    entry_id: String,
    intention: Option<String>,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        // No-op when not in Building state
        return crux_core::render::render();
    };

    if let Err(e) = validation::validate_intention(&intention) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };

    entry.intention = intention;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn set_entry_variant(
    model: &mut Model,
    entry_id: String,
    variant_id: Option<String>,
) -> Command<Effect, Event> {
    // The plan is a Building-phase thing: once practice starts, the
    // record is the plays and `SwitchVariation` is what changes it
    // (#1739 decision 5).
    if !matches!(model.session_status, SessionStatus::Building(_)) {
        model.raise_error("A variation can only be planned while building".to_string());
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_variant(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };

    if let Err(e) = validation::validate_entry_variation(entry, &variant_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_variant_mut(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };
    entry.planned_variation_id = variant_id;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn set_rep_target(
    model: &mut Model,
    entry_id: String,
    target: Option<u8>,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        // No-op when not in Building state
        return crux_core::render::render();
    };

    if let Some(t) = target {
        if let Err(e) = validation::validate_rep_target(&Some(t)) {
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    }

    let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };

    entry.planned_rep_target = target;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn set_entry_duration(
    model: &mut Model,
    entry_id: String,
    duration_secs: Option<u32>,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        // No-op when not in Building state
        return crux_core::render::render();
    };

    if let Err(e) = validation::validate_planned_duration(&duration_secs) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };

    entry.planned_duration_secs = duration_secs;
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
    model.session_status = SessionStatus::Building(BuildingSession::default());
    handle_session_event(SessionEvent::AddToSetlist { item_id }, model)
}

pub(super) fn start_building_from_suggestion(
    model: &mut Model,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("A practice is already in progress".to_string());
        return crux_core::render::render();
    }
    // Nothing to suggest is not an error: the CTA cannot be on screen
    // in that case, and a race must not strand an empty builder.
    let Some(suggestion) = crate::view::library::derive_up_next(model, now) else {
        return crux_core::render::render();
    };

    let mut building = BuildingSession::default();
    let group_id = ulid::Ulid::generate().to_string();
    for suggested in &suggestion.items {
        let position = building.entries.len();
        let mut entry = create_entry(
            &suggested.item_id,
            &suggested.item_title,
            suggested.item_type.clone(),
            position,
        );
        entry.group_id = Some(group_id.clone());
        entry.planned_variation_id.clone_from(&suggested.variant_id);
        building.entries.push(entry);
    }

    model.session_status = SessionStatus::Building(building);
    model.last_error = None;
    crux_core::render::render()
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

    model.session_status = SessionStatus::Building(BuildingSession::default());
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
        item.linked_exercise_ids
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

pub(super) fn reorder_setlist(
    model: &mut Model,
    entry_id: String,
    new_position: usize,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let Some(current_index) = building.entries.iter().position(|e| e.id == entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found in setlist"));
        return crux_core::render::render();
    };

    if new_position >= building.entries.len() {
        let msg = format!(
            "Invalid position: {new_position} (max: {})",
            building.entries.len().saturating_sub(1)
        );
        model.raise_error(msg);
        return crux_core::render::render();
    }

    let entry = building.entries.remove(current_index);
    building.entries.insert(new_position, entry);
    if !groups_contiguous(&building.entries) {
        // Revert: the move would split a block.
        let entry = building.entries.remove(new_position);
        building.entries.insert(current_index, entry);
        model.raise_error("Can't move an item out of its block".to_string());
        return crux_core::render::render();
    }
    reindex_entries(&mut building.entries);
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn reorder_block(
    model: &mut Model,
    group_id: String,
    new_position: usize,
) -> Command<Effect, Event> {
    let SessionStatus::Building(ref mut building) = model.session_status else {
        model.raise_error("Not in building state".to_string());
        return crux_core::render::render();
    };

    let mut units = into_units(std::mem::take(&mut building.entries));
    let Some(current) = units
        .iter()
        .position(|u| u.first().and_then(|e| e.group_id.as_deref()) == Some(group_id.as_str()))
    else {
        building.entries = units.into_iter().flatten().collect();
        model.raise_error(format!("Block '{group_id}' not found in setlist"));
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
    };

    if let Some(entry) = active.entries.first_mut() {
        open_first_play(entry, &model.items, now);
    }

    let save_effect = AppEffect::SaveSessionInProgress(active.clone());
    model.session_status = SessionStatus::Active(active);
    model.last_error = None;

    Command::all([
        Command::notify_shell(save_effect).into(),
        crux_core::render::render(),
    ])
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

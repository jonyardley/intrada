use super::plays::*;
use super::*;
use crate::app::{Effect, Event};
use crate::domain::section::{BarRange, BarsInput, SectionEdit, SectionKind};
use crate::model::Model;
use chrono::{DateTime, Utc};
use crux_core::Command;

/// Changes the current entry's open play, while no sheet is up: the stamped
/// play is final once it is (#2137). `apply` says whether it changed anything.
fn with_open_play(
    model: &mut Model,
    apply: impl FnOnce(&mut Play) -> bool,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    if active.reflection.is_some() {
        return crux_core::render::render();
    }
    let index = active.current_index;
    let changed = active
        .entries
        .get_mut(index)
        .and_then(SetlistEntry::open_play_mut)
        .is_some_and(apply);
    if changed {
        persist_active(active)
    } else {
        crux_core::render::render()
    }
}

pub(super) fn went_away(model: &mut Model, at: DateTime<Utc>) -> Command<Effect, Event> {
    with_open_play(model, |play| {
        let already_away = play.away.last().is_some_and(|a| a.back_at.is_none());
        if already_away || play.away.len() >= validation::MAX_TIMES_AWAY {
            return false;
        }
        play.away.push(Away {
            left_at: at.max(play.started_at),
            back_at: None,
            left_out: false,
        });
        true
    })
}

pub(super) fn came_back(model: &mut Model, at: DateTime<Utc>) -> Command<Effect, Event> {
    with_open_play(model, |play| {
        let Some(away) = play.away.last_mut().filter(|a| a.back_at.is_none()) else {
            return false;
        };
        away.back_at = Some(at.max(away.left_at));
        true
    })
}

pub(super) fn leave_away_out(model: &mut Model) -> Command<Effect, Event> {
    with_open_play(model, |play| {
        if away_to_offer(play).is_none() {
            return false;
        }
        if let Some(away) = play.away.last_mut() {
            away.left_out = true;
        }
        true
    })
}

pub(super) fn move_to_next_segment(
    model: &mut Model,
    now: DateTime<Utc>,
    reading: TempoReading,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    if active.reflection.is_some() {
        return crux_core::render::render();
    }
    let Some(clock) = active.segment.clone() else {
        return crux_core::render::render();
    };
    let index = active.current_index;
    let Some(entry) = active.entries.get_mut(index) else {
        return crux_core::render::render();
    };
    let next_index = clock.index as usize + 1;
    let Some(next) = entry.segments.get(next_index).cloned() else {
        return crux_core::render::render();
    };
    if let Err(e) = validation::validate_play_capacity(entry) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let open = entry.open_play().cloned();
    let stamp = close_open_play(entry, now, Some(&reading));
    let way = PlayWay {
        section_id: Some(next.section_id.clone()),
        key: open.as_ref().and_then(|p| p.key),
        variation_ids: open.map(|p| p.variation_ids).unwrap_or_default(),
    };
    entry
        .plays
        .push(Play::opened(way, entry.planned_rep_target, now));
    active.segment = Some(SegmentClock {
        index: next_index as u32,
        started_at: now,
        allowance_secs: next.planned_secs.saturating_sub(clock.taken_from_next_secs),
        taken_from_next_secs: 0,
        left_out_secs: 0,
    });
    model.last_error = None;
    let SessionStatus::Active(ref active) = model.session_status else {
        return crux_core::render::render();
    };
    let persist = persist_active(active);
    report_stamp(model, stamp);
    persist
}

/// Stay is offered only while the next segment can give the time and keep a
/// minute of its own.
pub(crate) fn can_stay(entry: &SetlistEntry, clock: &SegmentClock) -> bool {
    entry
        .segments
        .get(clock.index as usize + 1)
        .is_some_and(|next| {
            let left = next.planned_secs.saturating_sub(clock.taken_from_next_secs);
            left >= STAY_SECS + validation::MIN_PLANNED_DURATION_SECS
        })
}

pub(super) fn stay_on_segment(model: &mut Model) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    if active.reflection.is_some() {
        return crux_core::render::render();
    }
    let entry = active.current_entry().clone();
    let Some(clock) = active.segment.as_mut().filter(|c| can_stay(&entry, c)) else {
        return crux_core::render::render();
    };
    clock.allowance_secs += STAY_SECS;
    clock.taken_from_next_secs += STAY_SECS;
    persist_active(active)
}

/// A nameless trouble spot, placed before the first section that starts after
/// it, through the section editor's own validation (#2245).
pub(super) fn add_trouble_spot(
    model: &mut Model,
    item_id: String,
    bars: BarRange,
) -> Command<Effect, Event> {
    let Some(item) = model.items.iter().find(|i| i.id == item_id) else {
        model.raise_error(crate::error::LibraryError::NotFound { id: item_id }.to_string());
        return crux_core::render::render();
    };
    let mut live: Vec<&crate::domain::section::ItemSection> = item
        .sections
        .iter()
        .filter(|s| s.deleted_at.is_none())
        .collect();
    live.sort_by_key(|s| s.position);
    let mut edits: Vec<SectionEdit> = live
        .iter()
        .map(|s| SectionEdit {
            id: Some(s.id.clone()),
            name: s.name.clone(),
            bars: s.bars.map_or(BarsInput::Blank, |b| BarsInput::Picked {
                first: b.first,
                last: b.last,
            }),
            kind: s.kind,
            target_bpm: s.target_bpm.map(|b| b.to_string()).unwrap_or_default(),
        })
        .collect();
    let at = live
        .iter()
        .position(|s| s.bars.is_some_and(|b| b.first > bars.first))
        .unwrap_or(live.len());
    edits.insert(
        at,
        SectionEdit {
            id: None,
            name: String::new(),
            bars: BarsInput::Picked {
                first: bars.first,
                last: bars.last,
            },
            kind: SectionKind::TroubleSpot,
            target_bpm: String::new(),
        },
    );
    crate::domain::item::update_sections(model, item_id, edits)
}

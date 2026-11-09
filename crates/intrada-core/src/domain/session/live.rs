use super::plays::*;
use super::*;
use crate::app::{Effect, Event};
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

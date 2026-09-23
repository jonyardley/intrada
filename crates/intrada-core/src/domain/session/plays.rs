use super::*;
use crate::app::{AppEffect, Effect, Event};
use crate::domain::item::Item;
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;

pub(super) fn freeze_rep_state(entry: &mut SetlistEntry) {
    if let Some(play) = entry.open_play_mut() {
        if let (Some(target), Some(count)) = (play.rep_target, play.rep_count) {
            play.rep_target_reached = Some(count >= target);
        }
    }
}

/// An entry that has just become current opens its first play, seeded from
/// the builder's plan (#1739 decisions 3 and 5), else the item's first live
/// variation, lowest position first (#1758). No live variants means
/// unattributed. Idempotent: a recovered session already carries its plays.
pub(super) fn open_first_play(entry: &mut SetlistEntry, items: &[Item], now: DateTime<Utc>) {
    if entry.plays.is_empty() {
        let variation_id = entry.planned_variation_id.clone().or_else(|| {
            items
                .iter()
                .find(|item| item.id == entry.item_id)
                .and_then(|item| {
                    item.variants
                        .iter()
                        .filter(|v| v.deleted_at.is_none())
                        .min_by_key(|v| v.position)
                        .map(|v| v.id.clone())
                })
        });
        entry.plays.push(VariationPlay::opened(
            variation_id,
            entry.planned_rep_target,
            now,
        ));
    }
}

/// Stamp the open play's seconds from its own start, so a switch mid item
/// splits the time rather than double-counting it, and its tempo from what
/// the click was doing at that instant (#1761). `None` is a close with no
/// reading, which only a skip makes.
pub(super) fn close_open_play(
    entry: &mut SetlistEntry,
    now: DateTime<Utc>,
    reading: Option<&TempoReading>,
) -> TempoStamp {
    let Some(play) = entry.open_play_mut() else {
        return TempoStamp::NothingToKeep;
    };
    play.seconds = (now - play.started_at).num_seconds().max(0) as u64;
    match reading {
        Some(reading) => stamp_tempo(play, reading),
        None => TempoStamp::NothingToKeep,
    }
}

/// What a close did with its reading. The handler that closed the play
/// reports `Unusable` as a notice, not an error: the play still closed and
/// the tap landed (#1325).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TempoStamp {
    Kept,
    NothingToKeep,
    Unusable,
}

pub(super) const UNUSABLE_TEMPO_NOTICE: &str =
    "That metronome setting doesn't give a crotchet tempo, so this play has none.";

/// A silent reading writes nothing and clears nothing, so a later close never
/// erases an earlier stamp, and a sounding one overwrites it: the later instant
/// wins. An invalid reading stamps nothing and raises no error, since a close
/// is not a user action (#944).
pub(super) fn stamp_tempo(play: &mut VariationPlay, reading: &TempoReading) -> TempoStamp {
    if !reading.click_sounding {
        return TempoStamp::NothingToKeep;
    }
    if let Some(click) = &reading.click {
        if validation::validate_click_state(click).is_err() {
            return TempoStamp::Unusable;
        }
    }
    let crotchets = reading
        .click
        .as_ref()
        .map_or(reading.bpm, |c| c.metre.crotchet_bpm(reading.bpm));
    if validation::validate_achieved_tempo(&Some(crotchets)).is_err() {
        return TempoStamp::Unusable;
    }
    play.achieved_tempo = Some(crotchets);
    play.click_pattern = reading.click.clone();
    TempoStamp::Kept
}

/// A stray tap on the picker is not practice: every terminal transition drops
/// the plays that recorded nothing and ran for under five seconds. A practised
/// entry always keeps at least one, though, so decision 3's invariant holds and
/// the item-complete sheet always has a row to mark: dropping the only play
/// would leave the shell sending a `play_id` the core has just deleted.
pub(super) fn drop_incidental_play(entry: &mut SetlistEntry) {
    if entry.plays.len() < 2 {
        return;
    }
    let opened = entry.plays[0].clone();
    entry.plays.retain(|p| !p.is_incidental());
    if entry.plays.is_empty() {
        entry.plays.push(opened);
    }
}

pub(super) fn drop_incidental_plays(entries: &mut [SetlistEntry]) {
    for entry in entries.iter_mut() {
        drop_incidental_play(entry);
    }
}

/// Whether `play` would still be in `entry.plays` after `drop_incidental_play`
/// runs (#1758). `play` must be one of `entry.plays`. Relies on
/// `PrepareReflection` having stamped the open play's real seconds with the
/// same `now` the terminal transition will use; otherwise an open play
/// always reads as incidental regardless of how long it ran.
pub(crate) fn play_would_survive_drop(entry: &SetlistEntry, play: &VariationPlay) -> bool {
    if !play.is_incidental() {
        return true;
    }
    entry.plays.iter().all(VariationPlay::is_incidental)
        && entry.plays.first().is_some_and(|first| first.id == play.id)
}

/// The first tap is what switches the counter on: it writes the target along
/// with itself, so an untouched entry keeps all four rep fields `None` and
/// banks nothing (design-principles T19). A target set in the builder is kept.
pub(super) fn record_rep(
    model: &mut Model,
    action: RepAction,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    let Some(entry) = active.entries.get_mut(active.current_index) else {
        return crux_core::render::render();
    };
    // Repetitions belong to the variation being played, so they retarget the
    // open play and reset when a switch opens the next one (#1739 decision 6).
    open_first_play(entry, &model.items, now);
    let Some(play) = entry.open_play_mut() else {
        return crux_core::render::render();
    };
    // A miss at the target steps the count back so an accidental tap can be
    // corrected and re-earned (#1507).
    if play.rep_target_reached == Some(true) && action == RepAction::Success {
        return crux_core::render::render();
    }

    let target = *play
        .rep_target
        .get_or_insert(validation::DEFAULT_REP_TARGET);
    let count = play.rep_count.unwrap_or(0);
    let new_count = match action {
        RepAction::Success => (count + 1).min(target),
        RepAction::Missed => count.saturating_sub(1),
    };
    play.rep_count = Some(new_count);
    play.rep_target_reached = Some(new_count >= target);
    let history = play.rep_history.get_or_insert_with(Vec::new);
    if history.len() < validation::MAX_REP_HISTORY {
        history.push(RepEvent { action, at: now });
    }

    model.last_error = None;
    Command::all([
        Command::notify_shell(AppEffect::SaveSessionInProgress(active.clone())).into(),
        crux_core::render::render(),
    ])
}

/// Find an entry by id in Active *or* Summary phase, so the mid-session
/// reflection sheet can write per-entry data before the summary screen.
pub(super) fn entry_for_update_mut<'a>(
    model: &'a mut Model,
    entry_id: &str,
) -> Option<&'a mut SetlistEntry> {
    match &mut model.session_status {
        SessionStatus::Active(active) => active.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Summary(summary) => summary.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Idle | SessionStatus::Building(_) => None,
    }
}

// Unlike score/notes (post-play reflection only, `entry_for_update_mut`), the
// variation tag is also a Building-phase plan ("which rung am I about to climb"),
// so its lookup spans all three phases (#1083). The immutable twin exists for
// checks that also read the library (a mutable borrow would lock the model).
pub(super) fn entry_for_variant<'a>(model: &'a Model, entry_id: &str) -> Option<&'a SetlistEntry> {
    match &model.session_status {
        SessionStatus::Building(building) => building.entries.iter().find(|e| e.id == entry_id),
        SessionStatus::Active(active) => active.entries.iter().find(|e| e.id == entry_id),
        SessionStatus::Summary(summary) => summary.entries.iter().find(|e| e.id == entry_id),
        SessionStatus::Idle => None,
    }
}

pub(super) fn entry_for_variant_mut<'a>(
    model: &'a mut Model,
    entry_id: &str,
) -> Option<&'a mut SetlistEntry> {
    match &mut model.session_status {
        SessionStatus::Building(building) => building.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Active(active) => active.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Summary(summary) => summary.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Idle => None,
    }
}

pub(super) fn transition_to_summary(
    active: &mut ActiveSession,
    items: &[Item],
    now: DateTime<Utc>,
    reading: &TempoReading,
    completion_status: CompletionStatus,
) -> (SummarySession, TempoStamp) {
    let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;
    let mut stamp = TempoStamp::NothingToKeep;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.duration_secs = elapsed;
        entry.status = EntryStatus::Completed;
        open_first_play(entry, items, active.current_item_started_at);
        stamp = close_open_play(entry, now, Some(reading));
        freeze_rep_state(entry);
    }

    if completion_status == CompletionStatus::EndedEarly {
        for entry in active.entries.iter_mut().skip(active.current_index + 1) {
            entry.status = EntryStatus::NotAttempted;
            entry.duration_secs = 0;
            entry.plays.clear();
        }
    }

    drop_incidental_plays(&mut active.entries);

    let summary = SummarySession {
        id: active.id.clone(),
        entries: active.entries.clone(),
        session_started_at: active.session_started_at,
        session_ended_at: now,
        session_notes: None,
        completion_status,
        session_score: None,
    };
    (summary, stamp)
}

pub(super) fn report_stamp(model: &mut Model, stamp: TempoStamp) {
    if stamp == TempoStamp::Unusable {
        model.raise_notice(UNUSABLE_TEMPO_NOTICE);
    }
}

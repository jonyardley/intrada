use super::*;
use crate::app::{AppEffect, Effect, Event};
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;

/// An entry that has just become current opens its first play on its first
/// segment, else the focus's section, else the whole item; plain, with no
/// key (#2246, #2249). Last time is an offer, never a start. Idempotent: a
/// recovered session already carries its plays.
pub(super) fn open_first_play(entry: &mut SetlistEntry, now: DateTime<Utc>) {
    if entry.plays.is_empty() {
        let section_id = entry
            .segments
            .first()
            .map(|s| s.section_id.clone())
            .or_else(|| entry.focus.as_ref().and_then(|f| f.section_id.clone()));
        let way = PlayWay {
            section_id,
            ..PlayWay::default()
        };
        entry
            .plays
            .push(Play::opened(way, entry.planned_rep_target, now));
    }
}

pub(super) fn first_segment_clock(
    entry: &SetlistEntry,
    now: DateTime<Utc>,
) -> Option<SegmentClock> {
    let first = entry.segments.first()?;
    (entry.segments.len() > 1 && first.planned_secs > 0).then_some(SegmentClock {
        index: 0,
        started_at: now,
        allowance_secs: first.planned_secs,
        taken_from_next_secs: 0,
    })
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
    close_play(entry, now, reading, false)
}

/// `stamped` keeps the seconds the sheet's stamp wrote: after a resume the
/// play's clock is re-anchored, and recounting would take left-out time off
/// a second time.
pub(super) fn close_play(
    entry: &mut SetlistEntry,
    now: DateTime<Utc>,
    reading: Option<&TempoReading>,
    stamped: bool,
) -> TempoStamp {
    let Some(play) = entry.open_play_mut() else {
        return TempoStamp::NothingToKeep;
    };
    if let Some(away) = play.away.last_mut().filter(|a| a.back_at.is_none()) {
        away.back_at = Some(now.max(away.left_at));
    }
    if !stamped {
        let elapsed = (now - play.started_at).num_seconds().max(0) as u64;
        play.seconds = elapsed.saturating_sub(left_out_secs(&play.away, play.started_at, now));
    }
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
pub(super) fn stamp_tempo(play: &mut Play, reading: &TempoReading) -> TempoStamp {
    if !reading.click_sounding {
        return TempoStamp::NothingToKeep;
    }
    let Some(crotchets) = crotchet_tempo(reading) else {
        return TempoStamp::Unusable;
    };
    play.achieved_tempo = Some(crotchets);
    play.click_pattern = reading.click.clone();
    TempoStamp::Kept
}

/// The reading in crotchets, `None` when the setting gives no usable tempo.
/// Says nothing about evidence: whether the click sounded is the caller's.
fn crotchet_tempo(reading: &TempoReading) -> Option<u16> {
    if let Some(click) = &reading.click {
        validation::validate_click_state(click).ok()?;
    }
    let crotchets = reading
        .click
        .as_ref()
        .map_or(reading.bpm, |c| c.metre.crotchet_bpm(reading.bpm));
    validation::validate_achieved_tempo(&Some(crotchets)).ok()?;
    Some(crotchets)
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
pub(crate) fn play_would_survive_drop(entry: &SetlistEntry, play: &Play) -> bool {
    if !play.is_incidental() {
        return true;
    }
    entry.plays.iter().all(Play::is_incidental)
        && entry.plays.first().is_some_and(|first| first.id == play.id)
}

/// The first tap is what switches the counter on: it writes the target along
/// with itself, so an untouched entry keeps its rep fields `None` and banks
/// nothing (design-principles T19). A target set in the builder is kept;
/// otherwise the musician's default is written (#1915). The count is the
/// replay of the taps still standing, so it goes on past the target (#2107)
/// and a tap the full history cannot keep does not count.
pub(super) fn record_rep(
    model: &mut Model,
    action: RepAction,
    now: DateTime<Utc>,
    reading: &TempoReading,
) -> Command<Effect, Event> {
    let default_rep_target = model.practice_defaults.rep_target;
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    let Some(entry) = active.entries.get_mut(active.current_index) else {
        return crux_core::render::render();
    };
    // Repetitions belong to the play open now, so they reset when a switch
    // opens the next one (#1739 decision 6).
    open_first_play(entry, now);
    let Some(play) = entry.open_play_mut() else {
        return crux_core::render::render();
    };
    let history = play.rep_history.as_deref().unwrap_or_default();
    if history.len() >= validation::MAX_REP_HISTORY
        || (action == RepAction::Undo && standing_taps(history).is_empty())
    {
        return crux_core::render::render();
    }

    play.rep_target.get_or_insert(default_rep_target);
    let history = play.rep_history.get_or_insert_with(Vec::new);
    history.push(RepEvent {
        action,
        at: now,
        tempo: crotchet_tempo(reading),
        click_sounding: Some(reading.click_sounding),
    });
    play.rep_count = Some(rep_count(history));

    model.last_error = None;
    persist_active(active)
}

/// The got its and misses no undo has reversed, in order.
pub(crate) fn standing_taps(history: &[RepEvent]) -> Vec<RepAction> {
    let mut standing = Vec::new();
    for event in history {
        match event.action {
            RepAction::Undo => {
                standing.pop();
            }
            tap => standing.push(tap),
        }
    }
    standing
}

/// Got it adds one, uncapped bar the byte; a miss steps back one, floor 0.
pub(crate) fn rep_count(history: &[RepEvent]) -> u8 {
    standing_taps(history)
        .into_iter()
        .fold(0u8, |count, tap| match tap {
            RepAction::Success => count.saturating_add(1),
            RepAction::Missed | RepAction::Undo => count.saturating_sub(1),
        })
}

/// A change this soon after the last, with no tap between, is the same climb
/// still moving: it replaces the last rather than adding to it.
const TEMPO_SETTLE_SECS: i64 = 2;

/// Keeps where the tempo rested on the open play (#2107). A setting that gives
/// no crotchet tempo writes nothing and says nothing, like a tap's.
pub(super) fn record_tempo_change(
    model: &mut Model,
    now: DateTime<Utc>,
    reading: &TempoReading,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    // The stamped play is final while the sheet is open (#2137).
    if active.reflection.is_some() {
        return crux_core::render::render();
    }
    let Some(tempo) = crotchet_tempo(reading) else {
        return crux_core::render::render();
    };
    let Some(entry) = active.entries.get_mut(active.current_index) else {
        return crux_core::render::render();
    };
    open_first_play(entry, now);
    let Some(play) = entry.open_play_mut() else {
        return crux_core::render::render();
    };
    let change = TempoChange {
        at: now,
        tempo,
        click_sounding: reading.click_sounding,
    };
    let last_tap_at = play
        .rep_history
        .as_ref()
        .and_then(|h| h.last())
        .map(|e| e.at);
    let full = play.tempo_changes.len() >= validation::MAX_REP_HISTORY;
    match play.tempo_changes.last_mut() {
        Some(last) if last.tempo == tempo && last.click_sounding == reading.click_sounding => {
            return crux_core::render::render();
        }
        Some(last)
            if (now - last.at).num_seconds() < TEMPO_SETTLE_SECS
                && last_tap_at.is_none_or(|tap| tap < last.at) =>
        {
            *last = change;
        }
        _ if full => return crux_core::render::render(),
        _ => play.tempo_changes.push(change),
    }
    persist_active(active)
}

pub(super) fn persist_active(active: &ActiveSession) -> Command<Effect, Event> {
    Command::all([
        Command::notify_shell(AppEffect::SaveSessionInProgress(active.clone())).into(),
        crux_core::render::render(),
    ])
}

/// A reflection written before the summary lives only in the running practice
/// until the next save, so an app killed in between loses it (#2061).
pub(super) fn persist_if_active(model: &Model) -> Command<Effect, Event> {
    match &model.session_status {
        SessionStatus::Active(active) => persist_active(active),
        _ => crux_core::render::render(),
    }
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
// plan is a Building-phase write and a switch an Active one, so the lookup
// spans all three phases (#1083). The immutable twin exists for checks that
// also read the library (a mutable borrow would lock the model).
pub(super) fn entry_for_plan<'a>(model: &'a Model, entry_id: &str) -> Option<&'a SetlistEntry> {
    match &model.session_status {
        SessionStatus::Building(building) => building.entries.iter().find(|e| e.id == entry_id),
        SessionStatus::Active(active) => active.entries.iter().find(|e| e.id == entry_id),
        SessionStatus::Summary(summary) => summary.entries.iter().find(|e| e.id == entry_id),
        SessionStatus::Idle => None,
    }
}

pub(super) fn entry_for_plan_mut<'a>(
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

/// The plays tile the item from its start, so their seconds are its time,
/// and a resume, which backdates the item by that sum, cannot skew it.
pub(super) fn item_seconds(entry: &SetlistEntry) -> u64 {
    entry
        .plays
        .iter()
        .fold(0u64, |sum, p| sum.saturating_add(p.seconds))
}

pub(super) fn transition_to_summary(
    active: &mut ActiveSession,
    now: DateTime<Utc>,
    reading: &TempoReading,
    completion_status: CompletionStatus,
    stamped: bool,
) -> (SummarySession, TempoStamp) {
    let started = active.current_item_started_at;
    let mut stamp = TempoStamp::NothingToKeep;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.status = EntryStatus::Completed;
        open_first_play(entry, started);
        stamp = close_play(entry, now, Some(reading), stamped);
        entry.duration_secs = item_seconds(entry);
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

use super::plays::*;
use super::*;
use crate::app::{AppEffect, Effect, Event};
use crate::domain::item::Item;
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;

pub(super) fn prepare_reflection(
    model: &mut Model,
    now: DateTime<Utc>,
    reading: TempoReading,
) -> Command<Effect, Event> {
    // An internal step, not a user action: last_error is left alone
    // in both branches, not raised or cleared for it (#944).
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };

    let stamp = active
        .entries
        .get_mut(active.current_index)
        .map_or(TempoStamp::NothingToKeep, |entry| {
            close_open_play(entry, now, Some(&reading))
        });
    let persist = persist_active(active);
    report_stamp(model, stamp);
    persist
}

pub(super) fn next_item(
    model: &mut Model,
    now: DateTime<Utc>,
    next_item_started_at: DateTime<Utc>,
    reading: TempoReading,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        model.raise_error("Not in active state".to_string());
        return crux_core::render::render();
    };

    if active.current_index >= active.entries.len() - 1 {
        let (summary, stamp) = transition_to_summary(
            active,
            &model.items,
            now,
            &reading,
            CompletionStatus::Completed,
        );
        return finish(model, summary, stamp);
    }

    let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;

    let mut stamp = TempoStamp::NothingToKeep;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.duration_secs = elapsed;
        entry.status = EntryStatus::Completed;
        open_first_play(entry, &model.items, active.current_item_started_at);
        stamp = close_open_play(entry, now, Some(&reading));
        freeze_rep_state(entry);
        drop_incidental_play(entry);
    }

    advance(active, &model.items, next_item_started_at);
    model.last_error = None;

    let persist = persist_active(active);
    report_stamp(model, stamp);
    persist
}

pub(super) fn skip_item(model: &mut Model, now: DateTime<Utc>) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        model.raise_error("Not in active state".to_string());
        return crux_core::render::render();
    };

    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.duration_secs = 0;
        entry.status = EntryStatus::Skipped;
        // Skipped means not practised, so the play opened when the item
        // became current goes (#1739 decision 3). A play that banked
        // repetitions or a mark first is not that play and survives,
        // exactly as rep state did before plays existed. Time alone
        // does not count here: the clock ran while they decided to skip.
        close_open_play(entry, now, None);
        freeze_rep_state(entry);
        entry.plays.retain(VariationPlay::recorded_something);
        // Only a completed entry carries a tempo, and the tempo history
        // reads plays without the entry's status (#1761 rule 7).
        for play in &mut entry.plays {
            play.achieved_tempo = None;
            play.click_pattern = None;
        }
    }

    if active.current_index >= active.entries.len() - 1 {
        drop_incidental_plays(&mut active.entries);
        let summary = SummarySession {
            id: active.id.clone(),
            entries: active.entries.clone(),
            session_started_at: active.session_started_at,
            session_ended_at: now,
            session_notes: None,
            completion_status: CompletionStatus::Completed,
            session_score: None,
        };
        return finish(model, summary, TempoStamp::NothingToKeep);
    }

    advance(active, &model.items, now);
    model.last_error = None;
    persist_active(active)
}

pub(super) fn end_session_early(
    model: &mut Model,
    now: DateTime<Utc>,
    reading: TempoReading,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref mut active) = model.session_status else {
        model.raise_error("Not in active state".to_string());
        return crux_core::render::render();
    };

    let (summary, stamp) = transition_to_summary(
        active,
        &model.items,
        now,
        &reading,
        CompletionStatus::EndedEarly,
    );
    finish(model, summary, stamp)
}

pub(super) fn switch_variation(
    model: &mut Model,
    entry_id: String,
    variation_id: Option<String>,
    now: DateTime<Utc>,
    reading: TempoReading,
) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Active(_)) {
        model.raise_error("Not in active state".to_string());
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_variant(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };

    // Switching to the variation already open writes nothing, so a
    // stray tap cannot clear the dots (#1739 decision 6). Checked
    // before capacity, or a full entry could not re-tap its own row.
    if entry.open_play().map(|p| p.variation_id.as_deref()) == Some(variation_id.as_deref()) {
        model.last_error = None;
        return crux_core::render::render();
    }

    if let Err(e) = validation::validate_entry_variation(entry, &variation_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    if let Err(e) = validation::validate_play_capacity(entry) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    let Some(entry) = active.entries.iter_mut().find(|e| e.id == entry_id) else {
        return crux_core::render::render();
    };

    let stamp = close_open_play(entry, now, Some(&reading));
    freeze_rep_state(entry);
    let rep_target = entry.planned_rep_target;
    entry
        .plays
        .push(VariationPlay::opened(variation_id, rep_target, now));

    model.last_error = None;
    let persist = persist_active(active);
    report_stamp(model, stamp);
    persist
}

pub(super) fn recover_session(
    model: &mut Model,
    session: ActiveSession,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Idle) {
        model.raise_error("Cannot recover: a practice is already in progress".to_string());
        return crux_core::render::render();
    }

    // current_entry indexes len() - 1; the recovery blob is the one
    // input nothing keeps non-empty (#1807).
    if session.entries.is_empty() {
        model.raise_error(
            "Couldn't resume · the saved practice was empty, so it's been removed.".to_string(),
        );
        return Command::all([
            Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
            crux_core::render::render(),
        ]);
    }

    // Re-anchor the running item's wall-clock timer: the blob's anchor
    // is from before the kill, so resuming hours later would otherwise
    // show that gap as elapsed practice (#962).
    // Backdated by what the plays already recorded, or a practice saved at the
    // item-complete sheet resumes with its time wiped (#2061).
    let mut session = session;
    // A corrupt count falls back to the resume instant, or every Resume tap
    // would panic on a blob that outlives the crash.
    let recorded = |secs: u64| {
        i64::try_from(secs)
            .ok()
            .and_then(chrono::Duration::try_seconds)
            .and_then(|d| now.checked_sub_signed(d))
            .unwrap_or(now)
    };
    let entry = session.entries.get_mut(session.current_index);
    let item_secs = entry.as_ref().map_or(0, |e| {
        e.plays
            .iter()
            .fold(0u64, |sum, p| sum.saturating_add(p.seconds))
    });
    session.current_item_started_at = recorded(item_secs);
    // The open play's clock is the same clock one level down: left
    // alone, its close would record the gap as practice (#1795).
    if let Some(play) = entry.and_then(SetlistEntry::open_play_mut) {
        play.started_at = recorded(play.seconds);
    }
    model.session_status = SessionStatus::Active(session);
    model.last_error = None;
    crux_core::render::render()
}

fn advance(active: &mut ActiveSession, items: &[Item], started_at: DateTime<Utc>) {
    active.current_index += 1;
    active.current_item_started_at = started_at;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        open_first_play(entry, items, started_at);
    }
}

fn finish(model: &mut Model, summary: SummarySession, stamp: TempoStamp) -> Command<Effect, Event> {
    model.session_status = SessionStatus::Summary(summary);
    model.last_error = None;
    report_stamp(model, stamp);
    crux_core::render::render()
}

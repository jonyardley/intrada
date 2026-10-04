use super::plays::*;
use super::*;
use crate::app::{AppEffect, Effect, Event};
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
    // The sheet is already open: the stamp is final, and a second one would
    // count its dwell as practice (#2137).
    if active.reflection.is_some() {
        return crux_core::render::render();
    }

    let stamp = active
        .entries
        .get_mut(active.current_index)
        .map_or(TempoStamp::NothingToKeep, |entry| {
            close_open_play(entry, now, Some(&reading))
        });
    active.reflection = Some(ReflectionDraft {
        now,
        reading,
        answers: ReflectionAnswers::default(),
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
    // After a resume the shell no longer has the stamp's instant (#2137).
    let draft = active.reflection.take();
    let stamped = draft.is_some();
    let now = draft.map_or(now, |draft| draft.now);

    if active.current_index >= active.entries.len() - 1 {
        let (summary, stamp) =
            transition_to_summary(active, now, &reading, CompletionStatus::Completed, stamped);
        return finish(model, summary, stamp);
    }

    let mut stamp = TempoStamp::NothingToKeep;
    let started = active.current_item_started_at;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.status = EntryStatus::Completed;
        open_first_play(entry, started);
        stamp = close_play(entry, now, Some(&reading), stamped);
        entry.duration_secs = item_seconds(entry);
        drop_incidental_play(entry);
    }

    advance(active, next_item_started_at);
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
        entry.plays.retain(Play::recorded_something);
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

    advance(active, now);
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

    let draft = active.reflection.take();
    let stamped = draft.is_some();
    let now = draft.map_or(now, |draft| draft.now);
    let (summary, stamp) =
        transition_to_summary(active, now, &reading, CompletionStatus::EndedEarly, stamped);
    finish(model, summary, stamp)
}

pub(super) fn switch_play(
    model: &mut Model,
    entry_id: String,
    mut way: PlayWay,
    now: DateTime<Utc>,
    reading: TempoReading,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref active) = model.session_status else {
        model.raise_error("Not in active state".to_string());
        return crux_core::render::render();
    };
    // The stamped play is final while the sheet is open (#2137).
    if active.reflection.is_some() {
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_plan(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };

    let written = model
        .items
        .iter()
        .find(|i| i.id == entry.item_id)
        .and_then(|i| i.key);
    if way
        .key
        .zip(written)
        .is_some_and(|(named, written)| named.same_key(&written))
    {
        way.key = None;
    }

    // Switching to the way already open writes nothing, so a stray tap
    // cannot clear the dots (#1739 decision 6). Checked before capacity,
    // or a full entry could not re-tap its own row.
    if entry.open_play().is_some_and(|p| p.is_played(&way)) {
        model.last_error = None;
        return crux_core::render::render();
    }

    if let Err(e) = validation::validate_play_way(entry, &way, model) {
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
    if let (Some(clock), Some(closed)) = (active.segment.as_mut(), entry.open_play()) {
        let gap = left_out_secs(&closed.away, closed.started_at.max(clock.started_at), now);
        clock.left_out_secs = clock
            .left_out_secs
            .saturating_add(u32::try_from(gap).unwrap_or(u32::MAX));
    }
    let rep_target = entry.planned_rep_target;
    entry.plays.push(Play::opened(way, rep_target, now));

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
    // Closing at the resume instant against the backdated clocks below
    // records exactly the stamped seconds (#2137).
    if let Some(draft) = session.reflection.as_mut() {
        draft.now = now;
    }
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
        let anchor = recorded(play.seconds);
        if let Some(clock) = session.segment.as_mut() {
            let into = (play.started_at - clock.started_at).max(chrono::Duration::zero());
            clock.started_at = anchor.checked_sub_signed(into).unwrap_or(anchor);
        }
        play.started_at = anchor;
        // The blob was last saved as they left, so that is when it closes.
        if let Some(away) = play.away.last_mut().filter(|a| a.back_at.is_none()) {
            away.back_at = Some(away.left_at);
        }
    }
    model.session_status = SessionStatus::Active(session);
    model.last_error = None;
    crux_core::render::render()
}

fn advance(active: &mut ActiveSession, started_at: DateTime<Utc>) {
    active.current_index += 1;
    active.current_item_started_at = started_at;
    active.reflection = None;
    active.segment = None;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        open_first_play(entry, started_at);
        active.segment = first_segment_clock(entry, started_at);
    }
}

fn finish(model: &mut Model, summary: SummarySession, stamp: TempoStamp) -> Command<Effect, Event> {
    model.session_status = SessionStatus::Summary(summary);
    model.last_error = None;
    report_stamp(model, stamp);
    crux_core::render::render()
}

pub(super) fn update_reflection_draft(
    model: &mut Model,
    mut answers: ReflectionAnswers,
) -> Command<Effect, Event> {
    let SessionStatus::Active(ref active) = model.session_status else {
        return crux_core::render::render();
    };
    let entry = active.current_entry();
    let sections = super::finish::named_sections(model, &entry.item_id);
    keep_offered_points(&mut answers, &sections);
    if active.reflection.is_none()
        || !draft_answers_valid(entry, &answers)
        || !obstacles_distinct(&answers.got_in_the_way)
    {
        return crux_core::render::render();
    }
    let SessionStatus::Active(ref mut active) = model.session_status else {
        return crux_core::render::render();
    };
    if let Some(draft) = active.reflection.as_mut() {
        draft.answers = answers;
    }
    persist_active(active)
}

/// A note edit moves its points: a span no longer offered is dropped, since
/// refusing would leave the crash-recovery copy stale (#2137).
fn keep_offered_points(answers: &mut ReflectionAnswers, sections: &[(&str, &str)]) {
    let offered: Vec<NoteSpan> = note_offers(&answers.note, sections)
        .into_iter()
        .map(|p| p.span)
        .collect();
    let mut kept: Vec<NoteSpan> = Vec::new();
    for span in &answers.note_points {
        if offered.contains(span) && !kept.contains(span) {
            kept.push(*span);
        }
    }
    answers.note_points = kept;
}

fn obstacles_distinct(obstacles: &[Obstacle]) -> bool {
    let mut seen = obstacles.to_vec();
    seen.sort_by_key(|o| *o as u8);
    seen.dedup();
    seen.len() == obstacles.len()
}

fn draft_answers_valid(entry: &SetlistEntry, answers: &ReflectionAnswers) -> bool {
    let distinct = |ids: Vec<&str>| {
        let count = ids.len();
        ids.into_iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            == count
    };
    if !distinct(answers.marks.iter().map(|m| m.play_id.as_str()).collect())
        || !distinct(answers.tempos.iter().map(|t| t.play_id.as_str()).collect())
    {
        return false;
    }
    let marks_valid = answers.marks.iter().all(|mark| {
        validation::validate_score(mark.score).is_ok()
            && validation::validate_play_belongs(entry, &mark.play_id).is_ok()
    });
    let tempos_valid = answers.tempos.iter().all(|row| {
        let crotchets = row
            .click
            .as_ref()
            .map_or(row.tempo, |c| c.metre.crotchet_bpm(row.tempo));
        row.click
            .as_ref()
            .is_none_or(|c| validation::validate_click_state(c).is_ok())
            && validation::validate_achieved_tempo(&Some(crotchets)).is_ok()
            && validation::validate_play_belongs(entry, &row.play_id).is_ok()
    });
    marks_valid
        && tempos_valid
        && validation::validate_entry_notes(&Some(answers.note.clone())).is_ok()
}

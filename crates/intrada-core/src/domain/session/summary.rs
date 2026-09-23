use super::plays::*;
use super::*;
use crate::app::{AppEffect, Effect, Event};
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;

pub(super) fn update_entry_score(
    model: &mut Model,
    entry_id: String,
    play_id: String,
    score: Option<u8>,
) -> Command<Effect, Event> {
    if let Some(s) = score {
        if !(validation::MIN_SCORE..=validation::MAX_SCORE).contains(&s) {
            return crux_core::render::render();
        }
    }

    let Some(entry) = entry_for_update_mut(model, &entry_id) else {
        return crux_core::render::render();
    };

    if entry.status != EntryStatus::Completed {
        return crux_core::render::render();
    }

    if let Err(e) = validation::validate_play_belongs(entry, &play_id) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    if let Some(play) = entry.plays.iter_mut().find(|p| p.id == play_id) {
        play.score = score;
    }
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn update_entry_tempo(
    model: &mut Model,
    entry_id: String,
    play_id: String,
    tempo: Option<u16>,
    user_set: bool,
    click: Option<ClickState>,
) -> Command<Effect, Event> {
    if let Some(ref state) = click {
        if let Err(e) = validation::validate_click_state(state) {
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    }
    let crotchets = tempo.map(|displayed| {
        click
            .as_ref()
            .map_or(displayed, |c| c.metre.crotchet_bpm(displayed))
    });
    if let Err(_e) = validation::validate_achieved_tempo(&crotchets) {
        return crux_core::render::render();
    }

    // A number nobody set is a pre-fill nobody looked at: record
    // nothing, and clear nothing, so it cannot destroy a real
    // measurement. Clearing is always honoured.
    if tempo.is_some() && !user_set {
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_update_mut(model, &entry_id) else {
        return crux_core::render::render();
    };

    if entry.status != EntryStatus::Completed {
        return crux_core::render::render();
    }

    if let Err(e) = validation::validate_play_belongs(entry, &play_id) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    if let Some(play) = entry.plays.iter_mut().find(|p| p.id == play_id) {
        play.achieved_tempo = crotchets;
        // A close is the only writer of the pattern; a set by hand
        // keeps it, and only clearing the tempo takes it away.
        if crotchets.is_none() {
            play.click_pattern = None;
        }
    }
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn update_entry_notes(
    model: &mut Model,
    entry_id: String,
    notes: Option<String>,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_entry_notes(&notes) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(entry) = entry_for_update_mut(model, &entry_id) else {
        model.raise_error(format!("Entry '{entry_id}' not found"));
        return crux_core::render::render();
    };

    entry.notes = notes;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn update_session_notes(
    model: &mut Model,
    notes: Option<String>,
) -> Command<Effect, Event> {
    let SessionStatus::Summary(ref mut summary) = model.session_status else {
        model.raise_error("Not in summary state".to_string());
        return crux_core::render::render();
    };

    if let Err(e) = validation::validate_session_notes(&notes) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    summary.session_notes = notes;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn update_session_score(model: &mut Model, score: Option<u8>) -> Command<Effect, Event> {
    if let Some(s) = score {
        if !(validation::MIN_SCORE..=validation::MAX_SCORE).contains(&s) {
            return crux_core::render::render();
        }
    }
    let SessionStatus::Summary(ref mut summary) = model.session_status else {
        model.raise_error("Not in summary state".to_string());
        return crux_core::render::render();
    };
    summary.session_score = score;
    model.last_error = None;
    crux_core::render::render()
}

pub(super) fn save_session(model: &mut Model, now: DateTime<Utc>) -> Command<Effect, Event> {
    let SessionStatus::Summary(ref summary) = model.session_status else {
        model.raise_error("Not in summary state".to_string());
        return crux_core::render::render();
    };
    if model.saving_session.is_some() {
        return Command::done();
    }

    let total_duration_secs: u64 = summary.entries.iter().map(|e| e.duration_secs).sum();

    let practice_session = PracticeSession {
        id: summary.id.clone(),
        entries: summary.entries.clone(),
        session_notes: summary.session_notes.clone(),
        started_at: summary.session_started_at,
        completed_at: now,
        total_duration_secs,
        completion_status: summary.completion_status.clone(),
        session_score: summary.session_score,
    };

    model.saving_session = Some(practice_session.clone());
    model.clear_error();
    Command::all([
        crate::persistence::save_session(practice_session),
        crux_core::render::render(),
    ])
}

pub(super) fn discard_session(model: &mut Model) -> Command<Effect, Event> {
    if !matches!(model.session_status, SessionStatus::Summary(_)) {
        model.raise_error("Not in summary state".to_string());
        return crux_core::render::render();
    }

    model.session_status = SessionStatus::Idle;
    model.last_error = None;

    Command::all([
        Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
        crux_core::render::render(),
    ])
}

pub(crate) const SAVE_FAILED: &str =
    "Couldn't save this practice. Your notes and scores are still here: tap Save session again.";

/// The store has the row, so the model says so: push it, and close the
/// summary if it is still the one being saved (#974).
pub(crate) fn save_acknowledged(model: &mut Model) -> Command<Effect, Event> {
    let Some(session) = model.saving_session.take() else {
        return Command::done();
    };
    let still_open =
        matches!(&model.session_status, SessionStatus::Summary(s) if s.id == session.id);
    model.sessions.push(session);
    model.practice_summaries = crate::view::library::build_practice_summaries(&model.sessions);
    if !still_open {
        return crux_core::render::render();
    }
    model.session_status = SessionStatus::Idle;
    Command::all([
        Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
        crux_core::render::render(),
    ])
}

/// Nothing reached the disk: keep the summary and the copy, and raise past the
/// dismiss mute since this answers the musician's own tap (#974). `None` when
/// no save was parked.
pub(crate) fn save_refused(model: &mut Model) -> Option<Command<Effect, Event>> {
    let session = model.saving_session.take()?;
    if matches!(&model.session_status, SessionStatus::Summary(s) if s.id == session.id) {
        model.raise_error(SAVE_FAILED);
    }
    Some(crux_core::render::render())
}

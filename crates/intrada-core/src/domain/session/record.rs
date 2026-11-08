//! What v0.17 adds to the record (#50): the intention's focus, time away and the finish answers. Facts and the
//! musician's own answers only; anything the app works out is read here, when
//! asked, and never stored.

use super::{Play, SetlistEntry};
use crate::domain::bars::BarRange;
use crate::domain::metre;
use crate::domain::note_patterns::{self, NotePointKind as ReadKind};
use crate::error::LibraryError;
use crate::validation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

// ── Plan ──

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FocusKind {
    Tempo,
    CleanReps,
    FromMemory,
    Evenness,
}

/// What the intention aims at, beside its free text (#2303).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct IntentionFocus {
    pub kind: FocusKind,
    /// Crotchets for `Tempo`, a count for `CleanReps`, `None` otherwise.
    pub target: Option<u16>,
}

// ── Finish answers ──

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum IntentionMet {
    Yes,
    Partly,
    NotYet,
}

/// Append, never insert: the crash-recovery blob stores the index (#1345).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Felt {
    Comfortable,
    HardWork,
    Strained,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Obstacle {
    Notes,
    Rhythm,
    Fingering,
    Memory,
    Tone,
    Tension,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum NotePointKind {
    Bars(BarRange),
    Tempo {
        bpm: u16,
    },
    Repetitions {
        count: u16,
        clean: bool,
        in_a_row: bool,
    },
}

/// A byte range of the note as written; a struct, not a tuple, so both
/// shells' generated types can name its ends.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct NoteSpan {
    pub start: u32,
    pub end: u32,
}

/// A point the musician confirmed from their note (#2307). The span lets a
/// later reader (#2316) re-read the note without touching confirmed points.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct NotePoint {
    pub kind: NotePointKind,
    pub span: NoteSpan,
}

// ── Live ──

/// Leaving and coming back on the open play (#2306). The raw times stay;
/// `left_out` takes the gap off the play's seconds as it closes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Away {
    pub left_at: DateTime<Utc>,
    /// `None` while still away.
    pub back_at: Option<DateTime<Utc>>,
    pub left_out: bool,
}

/// A screen lock for a few seconds is not time away.
pub const MIN_AWAY_SECS: i64 = 60;

// ── Away ──

/// Only gaps that began at or after `from` count: a resume re-anchors the
/// clock past the gaps its stamped seconds already left out.
pub(crate) fn left_out_secs(gaps: &[Away], from: DateTime<Utc>, to: DateTime<Utc>) -> u64 {
    gaps.iter()
        .filter(|a| a.left_out && a.left_at >= from)
        .filter_map(|a| {
            let end = a.back_at?.min(to);
            Some((end - a.left_at).num_seconds().max(0) as u64)
        })
        .sum()
}

/// The last away on the play, when it is back and long enough to offer.
pub(crate) fn away_to_offer(play: &Play) -> Option<(&Away, i64)> {
    let away = play.away.last()?;
    let secs = (away.back_at? - away.left_at).num_seconds();
    (!away.left_out && secs >= MIN_AWAY_SECS).then_some((away, secs))
}

// ── Intention ──

/// Worked out from the plays where the focus allows (a tempo), never stored:
/// met when a play reached the target with the click sounding. `None` means the finish sheet asks.
pub(crate) fn intention_met_read(entry: &SetlistEntry) -> Option<IntentionMet> {
    let focus = entry.plan.focus.as_ref()?;
    let target = focus.target?;
    if focus.kind != FocusKind::Tempo {
        return None;
    }
    entry
        .plays
        .iter()
        .any(|p| reached_with_click(p, target))
        .then_some(IntentionMet::Yes)
}

fn reached_with_click(play: &Play, target: u16) -> bool {
    let stamped = play.click_pattern.is_some() && play.achieved_tempo.is_some_and(|t| t >= target);
    let tapped = play
        .rep_history
        .iter()
        .flatten()
        .any(|tap| tap.click_sounding == Some(true) && tap.tempo.is_some_and(|t| t >= target));
    let rested = play
        .tempo_changes
        .iter()
        .any(|c| c.click_sounding && c.tempo >= target);
    stamped || tapped || rested
}

pub(crate) fn intended(entry: &SetlistEntry) -> bool {
    entry.plan.focus.is_some()
        || entry
            .plan
            .intention
            .as_deref()
            .is_some_and(|t| !t.trim().is_empty())
}

/// The sheet asks once, only when there was an intention and the plays
/// cannot answer it.
pub(crate) fn asks_intention(entry: &SetlistEntry) -> bool {
    intended(entry) && intention_met_read(entry).is_none()
}

pub(crate) fn validate_focus(focus: &IntentionFocus) -> Result<(), LibraryError> {
    let refuse = |message: &str| LibraryError::Validation {
        field: "focus".to_string(),
        message: message.to_string(),
    };
    match (focus_target_range(focus.kind), focus.target) {
        (Some((range, _)), Some(target)) if range.contains(&target) => Ok(()),
        (Some(_), _) if focus.kind == FocusKind::Tempo => {
            Err(refuse("Give a tempo the click can play"))
        }
        (Some(_), _) => Err(refuse("Give how many clean in a row")),
        (None, None) => Ok(()),
        (None, Some(_)) => Err(refuse("This focus has no number to aim for")),
    }
}

/// One range for the core and the builder's stepper, with the stepper's step.
/// A tempo is checked with the click, so it is held to what the click plays.
pub(crate) fn focus_target_range(kind: FocusKind) -> Option<(RangeInclusive<u16>, u16)> {
    match kind {
        FocusKind::Tempo => Some((
            metre::CLICK_TEMPO_MIN..=metre::CLICK_TEMPO_MAX,
            metre::CLICK_TEMPO_STEP,
        )),
        FocusKind::CleanReps => Some((1..=validation::MAX_CLEAN_REPS, 1)),
        FocusKind::FromMemory | FocusKind::Evenness => None,
    }
}

/// A tempo focus starts at the piece's own marked tempo.
pub(crate) fn starting_target(kind: FocusKind, piece_bpm: Option<u16>) -> Option<u16> {
    let (range, _) = focus_target_range(kind)?;
    let start = match kind {
        FocusKind::Tempo => piece_bpm.unwrap_or(metre::CLICK_TEMPO_DEFAULT),
        _ => *range.start(),
    };
    Some(start.clamp(*range.start(), *range.end()))
}

pub(crate) fn target_caption(focus: &IntentionFocus) -> Option<String> {
    let target = focus.target?;
    match focus.kind {
        FocusKind::Tempo => Some(format!("\u{2669} = {target}")),
        FocusKind::CleanReps => Some(format!("{target} clean in a row")),
        FocusKind::FromMemory | FocusKind::Evenness => None,
    }
}

// ── Reading a note ──

/// The points a note offers.
pub(crate) fn note_offers(note: &str) -> Vec<NotePoint> {
    let mut offers = Vec::new();
    for point in note_patterns::read_note(note) {
        let kind = match point.kind {
            ReadKind::Bars(bars) => NotePointKind::Bars(bars),
            ReadKind::Tempo { bpm } => NotePointKind::Tempo { bpm },
            ReadKind::Repetitions {
                count,
                clean,
                in_a_row,
            } => NotePointKind::Repetitions {
                count,
                clean,
                in_a_row,
            },
        };
        let (Ok(start), Ok(end)) = (
            u32::try_from(point.span.start),
            u32::try_from(point.span.end),
        ) else {
            continue;
        };
        offers.push(NotePoint {
            kind,
            span: NoteSpan { start, end },
        });
    }
    offers
}

/// "At 84" as a focus: the first tempo, else the first clean count.
pub(crate) fn suggest_focus(text: &str) -> Option<IntentionFocus> {
    let offers = note_offers(text);
    let tempo = offers.iter().find_map(|p| match p.kind {
        NotePointKind::Tempo { bpm } => Some((FocusKind::Tempo, bpm)),
        _ => None,
    });
    let clean = || {
        offers.iter().find_map(|p| match p.kind {
            NotePointKind::Repetitions {
                count, clean: true, ..
            } => Some((FocusKind::CleanReps, count)),
            _ => None,
        })
    };
    let (kind, target) = tempo.or_else(clean)?;
    let focus = IntentionFocus {
        kind,
        target: Some(target),
    };
    validate_focus(&focus).ok().map(|()| focus)
}

pub(crate) fn note_point_label(kind: &NotePointKind) -> String {
    match kind {
        NotePointKind::Bars(bars) => bars.caption(),
        NotePointKind::Tempo { bpm } => format!("\u{2669} = {bpm}"),
        NotePointKind::Repetitions {
            count,
            clean,
            in_a_row,
        } => {
            let what = if *clean { "clean" } else { "times" };
            let row = if *in_a_row { " in a row" } else { "" };
            format!("{count} {what}{row}")
        }
    }
}

pub(crate) fn focus_label(focus: &IntentionFocus) -> String {
    let target = focus.target.unwrap_or_default();
    match focus.kind {
        FocusKind::Tempo => format!("At {target}"),
        FocusKind::CleanReps => format!("{target} clean in a row"),
        FocusKind::FromMemory => "From memory".to_string(),
        FocusKind::Evenness => "Evenness".to_string(),
    }
}

pub(crate) fn felt_label(felt: Felt) -> &'static str {
    match felt {
        Felt::Comfortable => "Comfortable",
        Felt::HardWork => "Hard work",
        Felt::Strained => "Strained",
    }
}

pub(crate) const FELT_CHOICES: [Felt; 3] = [Felt::Comfortable, Felt::HardWork, Felt::Strained];

pub(crate) fn focus_kind_label(kind: FocusKind) -> &'static str {
    match kind {
        FocusKind::Tempo => "Tempo",
        FocusKind::CleanReps => "Clean in a row",
        FocusKind::FromMemory => "From memory",
        FocusKind::Evenness => "Evenness",
    }
}

pub(crate) const FOCUS_CHOICES: [FocusKind; 4] = [
    FocusKind::Tempo,
    FocusKind::CleanReps,
    FocusKind::FromMemory,
    FocusKind::Evenness,
];

pub(crate) fn obstacle_label(obstacle: Obstacle) -> &'static str {
    match obstacle {
        Obstacle::Notes => "Notes",
        Obstacle::Rhythm => "Rhythm",
        Obstacle::Fingering => "Fingering",
        Obstacle::Memory => "Memory",
        Obstacle::Tone => "Tone",
        Obstacle::Tension => "Tension",
    }
}

pub(crate) const OBSTACLE_CHOICES: [Obstacle; 6] = [
    Obstacle::Notes,
    Obstacle::Rhythm,
    Obstacle::Fingering,
    Obstacle::Memory,
    Obstacle::Tone,
    Obstacle::Tension,
];

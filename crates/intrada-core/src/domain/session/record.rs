//! What v0.17 adds to the record (`specs/practice-record-v017.md`): segments,
//! the intention's focus, time away and the finish answers. Facts and the
//! musician's own answers only; anything the app works out is read here, when
//! asked, and never stored.

use super::{Play, SetlistEntry};
use crate::domain::note_patterns::{self, NotePointKind as ReadKind};
use crate::domain::section::BarRange;
use crate::error::LibraryError;
use crate::validation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Plan ──

/// One stretch of an entry given to one section, in order (#2315).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Segment {
    pub section_id: String,
    pub planned_secs: u32,
}

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
    /// `None` is the whole piece.
    pub section_id: Option<String>,
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

/// Provisional until the design pass settles the choices (#2308); a new
/// choice is appended, never inserted, so a saved practice still reads.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Felt {
    Easy,
    Effortful,
    Tense,
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
    pub section_id: Option<String>,
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

/// The running segment of the current entry. It never stops the item: when
/// `allowance_secs` is up the view offers to move on, and ignoring it changes
/// nothing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SegmentClock {
    pub index: u32,
    pub started_at: DateTime<Utc>,
    pub allowance_secs: u32,
    /// What Stay has borrowed from the next segment so far.
    pub taken_from_next_secs: u32,
    /// Left out on plays this segment has already closed, kept here because a
    /// resume moves the timestamps it was read from.
    pub left_out_secs: u32,
}

/// A screen lock for a few seconds is not time away.
pub const MIN_AWAY_SECS: i64 = 60;
pub const STAY_SECS: u32 = 120;

// ── Segments ──

/// Fixed segments keep their time; those sent at zero share what is left in
/// whole minutes, earliest first. Without a planned time every segment is zero.
pub(crate) fn rebalance(
    segments: &mut [Segment],
    planned: Option<u32>,
) -> Result<(), LibraryError> {
    let Some(planned) = planned else {
        for segment in segments.iter_mut() {
            segment.planned_secs = 0;
        }
        return Ok(());
    };
    let refuse = || LibraryError::Validation {
        field: "segments".to_string(),
        message: format!(
            "Those minutes don't fit in this item's {} min",
            planned / 60
        ),
    };
    let min = validation::MIN_PLANNED_DURATION_SECS;
    if segments
        .iter()
        .any(|s| s.planned_secs > 0 && s.planned_secs < min)
    {
        return Err(refuse());
    }
    let fixed = segments
        .iter()
        .fold(0u32, |sum, s| sum.saturating_add(s.planned_secs));
    let free = segments.iter().filter(|s| s.planned_secs == 0).count() as u32;
    if free == 0 {
        let Some((last, rest)) = segments.split_last_mut() else {
            return Ok(());
        };
        let others = rest
            .iter()
            .fold(0u32, |sum, s| sum.saturating_add(s.planned_secs));
        last.planned_secs = planned
            .checked_sub(others)
            .filter(|secs| *secs >= min)
            .ok_or_else(refuse)?;
        return Ok(());
    }
    let remaining = planned.checked_sub(fixed).ok_or_else(refuse)?;
    let minutes = remaining / 60;
    if minutes < free {
        return Err(refuse());
    }
    let (each, extra) = (minutes / free, minutes % free);
    let mut given = 0;
    let last_free = segments.iter().rposition(|s| s.planned_secs == 0);
    for (i, segment) in segments.iter_mut().enumerate() {
        if segment.planned_secs != 0 {
            continue;
        }
        let bonus = u32::from(given < extra);
        given += 1;
        segment.planned_secs = (each + bonus) * 60;
        if Some(i) == last_free {
            segment.planned_secs += remaining % 60;
        }
    }
    Ok(())
}

pub(crate) fn split_evenly(segments: &mut [Segment], planned: Option<u32>) {
    for segment in segments.iter_mut() {
        segment.planned_secs = 0;
    }
    // A planned time under a minute per segment keeps them at zero: the
    // builder's minimum is a minute, and the plan still names the sections.
    let _ = rebalance(segments, planned);
}

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

/// Worked out from the plays where the focus allows (a tempo on a section or
/// the whole piece), never stored: met when a play of that part reached the
/// target with the click sounding. `None` means the finish sheet asks.
pub(crate) fn intention_met_read(entry: &SetlistEntry) -> Option<IntentionMet> {
    let focus = entry.focus.as_ref()?;
    let target = focus.target?;
    if focus.kind != FocusKind::Tempo {
        return None;
    }
    entry
        .plays
        .iter()
        .filter(|p| p.section_id == focus.section_id)
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
    entry.focus.is_some()
        || entry
            .intention
            .as_deref()
            .is_some_and(|t| !t.trim().is_empty())
}

/// The sheet asks once, only when there was an intention and the plays
/// cannot answer it.
pub(crate) fn asks_intention(entry: &SetlistEntry) -> bool {
    intended(entry) && intention_met_read(entry).is_none()
}

pub(crate) fn validate_focus(
    focus: &IntentionFocus,
    section_ids: &[&str],
) -> Result<(), LibraryError> {
    let refuse = |message: &str| LibraryError::Validation {
        field: "focus".to_string(),
        message: message.to_string(),
    };
    if let Some(id) = &focus.section_id {
        if !section_ids.contains(&id.as_str()) {
            return Err(refuse("That section isn't part of this item"));
        }
    }
    match (focus.kind, focus.target) {
        (FocusKind::Tempo, Some(bpm))
            if (validation::MIN_BPM..=validation::MAX_BPM).contains(&bpm) =>
        {
            Ok(())
        }
        (FocusKind::Tempo, _) => Err(refuse("Give the tempo to aim for")),
        (FocusKind::CleanReps, Some(count))
            if (1..=validation::MAX_CLEAN_REPS).contains(&count) =>
        {
            Ok(())
        }
        (FocusKind::CleanReps, _) => Err(refuse("Give how many clean in a row")),
        (FocusKind::FromMemory | FocusKind::Evenness, None) => Ok(()),
        (FocusKind::FromMemory | FocusKind::Evenness, Some(_)) => {
            Err(refuse("This focus has no number to aim for"))
        }
    }
}

// ── Reading a note ──

/// The points a note offers, each with the section named before it in the
/// note. `sections` are the item's live sections as (id, name).
pub(crate) fn note_offers(note: &str, sections: &[(&str, &str)]) -> Vec<NotePoint> {
    let names: Vec<&str> = sections.iter().map(|(_, name)| *name).collect();
    let mut section_id: Option<String> = None;
    let mut offers = Vec::new();
    for point in note_patterns::read_note(note, &names) {
        let kind = match point.kind {
            ReadKind::Section { name } => {
                section_id = sections
                    .iter()
                    .find(|(_, n)| *n == name)
                    .map(|(id, _)| id.to_string());
                continue;
            }
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
            section_id: section_id.clone(),
            span: NoteSpan { start, end },
        });
    }
    offers
}

/// "A1 at 84" as a focus: the first tempo, else the first clean count.
pub(crate) fn suggest_focus(text: &str, sections: &[(&str, &str)]) -> Option<IntentionFocus> {
    let offers = note_offers(text, sections);
    let tempo = offers.iter().find_map(|p| match p.kind {
        NotePointKind::Tempo { bpm } => Some((FocusKind::Tempo, bpm, p)),
        _ => None,
    });
    let clean = || {
        offers.iter().find_map(|p| match p.kind {
            NotePointKind::Repetitions {
                count, clean: true, ..
            } => Some((FocusKind::CleanReps, count, p)),
            _ => None,
        })
    };
    let (kind, target, point) = tempo.or_else(clean)?;
    let focus = IntentionFocus {
        kind,
        section_id: point.section_id.clone(),
        target: Some(target),
    };
    let ids: Vec<&str> = sections.iter().map(|(id, _)| *id).collect();
    validate_focus(&focus, &ids).ok().map(|()| focus)
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

pub(crate) fn focus_label(focus: &IntentionFocus, section: Option<&str>) -> String {
    let target = focus.target.unwrap_or_default();
    match (focus.kind, section) {
        (FocusKind::Tempo, Some(s)) => format!("{s} at {target}"),
        (FocusKind::Tempo, None) => format!("At {target}"),
        (FocusKind::CleanReps, Some(s)) => format!("{s}: {target} clean in a row"),
        (FocusKind::CleanReps, None) => format!("{target} clean in a row"),
        (FocusKind::FromMemory, Some(s)) => format!("{s} from memory"),
        (FocusKind::FromMemory, None) => "From memory".to_string(),
        (FocusKind::Evenness, Some(s)) => format!("Evenness in {s}"),
        (FocusKind::Evenness, None) => "Evenness".to_string(),
    }
}

pub(crate) fn felt_label(felt: Felt) -> &'static str {
    match felt {
        Felt::Easy => "Easy",
        Felt::Effortful => "Effortful",
        Felt::Tense => "Tense",
    }
}

pub(crate) const FELT_CHOICES: [Felt; 3] = [Felt::Easy, Felt::Effortful, Felt::Tense];

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

use crate::app::{Effect, Event};
use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::metre::Metre;
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

// ── Enums ──────────────────────────────────────────────────────────────

/// Completion status of a single setlist entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum EntryStatus {
    /// Item was practised and time was recorded.
    Completed,
    /// Item was explicitly skipped (duration_secs = 0).
    Skipped,
    /// Session ended early before reaching this item (duration_secs = 0).
    NotAttempted,
}

/// Whether the session ran to completion or was ended early.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum CompletionStatus {
    /// All items in the setlist were addressed (completed or skipped).
    Completed,
    /// Session was ended before all items were reached.
    EndedEarly,
}

/// A single action in the rep history sequence.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum RepAction {
    /// Failed rep: count decremented.
    Missed,
    /// Successful rep: count incremented.
    Success,
    /// Reverses the last got it or miss still standing, so a correction is
    /// never recorded as a failure (#2107). Appended last.
    Undo,
}

/// One tap on the pass counter, when it landed and what the click was doing.
/// Sequence alone cannot tell a steady ten from a hard-won one (#1367). Only a
/// tap with the click sounding is tempo evidence (T16); `None` on taps from
/// before #2107, and `tempo: None` when the setting gave no crotchet tempo.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RepEvent {
    pub action: RepAction,
    pub at: DateTime<Utc>,
    pub tempo: Option<u16>,
    pub click_sounding: Option<bool>,
}

/// Where the tempo rested during a play, in crotchets (#2107).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoChange {
    pub at: DateTime<Utc>,
    pub tempo: u16,
    pub click_sounding: bool,
}

// ── Domain Types ───────────────────────────────────────────────────────

/// One stretch of an item played one way (#1739, #2246): what was played, as
/// against the entry's plan. Switching mid item closes the open play and opens
/// another, so an entry practised in C and then D holds two.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Play {
    pub id: String,
    /// `None` is the whole piece.
    pub section_id: Option<String>,
    /// `None` is the written key, or no key.
    pub key: Option<Key>,
    /// Library variations; empty is plain.
    pub variation_ids: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub seconds: u64,
    pub rep_target: Option<u8>,
    pub rep_count: Option<u8>,
    pub rep_history: Option<Vec<RepEvent>>,
    pub tempo_changes: Vec<TempoChange>,
    pub achieved_tempo: Option<u16>,
    pub click_pattern: Option<ClickState>,
    pub score: Option<u8>,
    pub away: Vec<Away>,
}

/// How a play is played: the part of the item, the key and the variations.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayWay {
    pub section_id: Option<String>,
    pub key: Option<Key>,
    pub variation_ids: Vec<String>,
}

impl Play {
    pub fn opened(way: PlayWay, rep_target: Option<u8>, started_at: DateTime<Utc>) -> Self {
        Self {
            id: ulid::Ulid::generate().to_string(),
            section_id: way.section_id,
            key: way.key,
            variation_ids: way.variation_ids,
            started_at,
            seconds: 0,
            rep_target,
            rep_count: None,
            rep_history: None,
            tempo_changes: Vec::new(),
            achieved_tempo: None,
            click_pattern: None,
            score: None,
            away: Vec::new(),
        }
    }

    /// The whole item, no variations; any key. The piece's own score reads
    /// only these (#50 decision 6).
    pub fn is_plain_run_through(&self) -> bool {
        self.section_id.is_none() && self.variation_ids.is_empty()
    }

    /// The same section, key in either spelling, and set of variations, in any order.
    pub fn is_played(&self, way: &PlayWay) -> bool {
        let sorted = |ids: &[String]| {
            let mut ids = ids.to_vec();
            ids.sort();
            ids
        };
        let same_key = match (self.key, way.key) {
            (Some(open), Some(named)) => open.same_key(&named),
            (open, named) => open == named,
        };
        self.section_id == way.section_id
            && same_key
            && sorted(&self.variation_ids) == sorted(&way.variation_ids)
    }

    /// A mark or a banked repetition. Time alone is not a record: the clock
    /// runs whether or not the musician played anything.
    pub fn recorded_something(&self) -> bool {
        self.score.is_some() || self.rep_count.unwrap_or(0) > 0
    }

    /// A play nobody practised: no mark, no repetitions and under five seconds.
    /// A terminal transition drops these so a stray tap on the picker leaves no
    /// trace. The time bound is what separates a stray tap from a stretch that
    /// was genuinely played and simply not marked.
    pub fn is_incidental(&self) -> bool {
        !self.recorded_something() && self.seconds < validation::MIN_PLAY_SECONDS
    }
}

/// An individual item within a session's setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetlistEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
    pub duration_secs: u64,
    pub status: EntryStatus,
    pub notes: Option<String>,
    pub intention: Option<String>,
    pub planned_duration_secs: Option<u32>,
    /// Block grouping (building phase): entries pulled in alongside a piece via
    /// its related exercises share one `group_id`. A block is the contiguous run
    /// of entries with the same id; `None` = standalone.
    pub group_id: Option<String>,
    pub planned_variation_ids: Vec<String>,
    /// The repetition target set in the builder. Every play the entry opens
    /// starts from it, so a switch redraws the same number of slots.
    pub planned_rep_target: Option<u8>,
    /// What was actually practised, in order. Empty for an entry never
    /// attempted; every practised entry has at least one, a piece included
    /// (#1739 decision 3).
    pub plays: Vec<Play>,
    /// The sections the builder planned, in order, with their time (#2315).
    /// The plan, not the record: planned against played is the clearest sign
    /// of avoiding a hard part.
    pub segments: Vec<Segment>,
    pub focus: Option<IntentionFocus>,
    /// The musician's answer, only when the sheet asked and they gave one.
    pub intention_met: Option<IntentionMet>,
    pub felt: Option<Felt>,
    pub got_in_the_way: Vec<Obstacle>,
    pub note_points: Vec<NotePoint>,
}

impl SetlistEntry {
    pub fn planned_section_ids(&self) -> Vec<String> {
        self.segments.iter().map(|s| s.section_id.clone()).collect()
    }

    pub fn open_play(&self) -> Option<&Play> {
        self.plays.last()
    }

    pub fn open_play_mut(&mut self) -> Option<&mut Play> {
        self.plays.last_mut()
    }

    /// The one place several marks collapse into one (#1739 decision 9): the
    /// mean of the plain full run-throughs that carry a score, rounded to
    /// nearest (#2246). A section or a variation is not the piece.
    pub fn score_summary(&self) -> Option<u8> {
        let scored: Vec<u16> = self
            .plays
            .iter()
            .filter(|p| p.is_plain_run_through())
            .filter_map(|p| p.score)
            .map(u16::from)
            .collect();
        if scored.is_empty() {
            return None;
        }
        let total: u16 = scored.iter().sum();
        let count = scored.len() as u16;
        Some(((total * 2 + count) / (count * 2)) as u8)
    }
}

#[cfg(test)]
impl Play {
    pub(crate) fn fixture() -> Self {
        Self {
            id: "play-1".to_string(),
            section_id: None,
            key: None,
            variation_ids: Vec::new(),
            started_at: DateTime::<Utc>::from_timestamp(0, 0).expect("epoch"),
            seconds: 60,
            rep_target: None,
            rep_count: None,
            rep_history: None,
            tempo_changes: Vec::new(),
            achieved_tempo: None,
            click_pattern: None,
            score: None,
            away: Vec::new(),
        }
    }
}

#[cfg(test)]
impl SetlistEntry {
    pub(crate) fn fixture() -> Self {
        Self {
            id: "entry-1".to_string(),
            item_id: "item-1".to_string(),
            item_title: "Item".to_string(),
            item_type: ItemKind::Piece,
            position: 0,
            duration_secs: 0,
            status: EntryStatus::NotAttempted,
            notes: None,
            intention: None,
            planned_duration_secs: None,
            group_id: None,
            planned_variation_ids: Vec::new(),
            planned_rep_target: None,
            plays: Vec::new(),
            segments: Vec::new(),
            focus: None,
            intention_met: None,
            felt: None,
            got_in_the_way: Vec::new(),
            note_points: Vec::new(),
        }
    }
}

/// A completed practice session (persisted to localStorage).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeSession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub session_notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_secs: u64,
    pub completion_status: CompletionStatus,
    #[serde(default)]
    pub session_score: Option<u8>,
    /// What this record captures, stamped `CAPTURE_VERSION` on every save so
    /// later reads know which fields to trust; `None` before #2246.
    #[serde(default)]
    pub capture_version: Option<u32>,
}

/// 1: plays carry a section, a key and variations, taps their tempo (#2246).
pub const CAPTURE_VERSION: u32 = 1;

/// What the click was doing at the instant a play closed. Facts only: the
/// core rules on what they evidence (design-principles T16, #1761).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoReading {
    /// As displayed, in `click.metre.unit`; crotchets when `click` is `None`.
    pub bpm: u16,
    pub click_sounding: bool,
    pub click: Option<ClickState>,
}

#[cfg(test)]
impl TempoReading {
    pub(crate) fn silent() -> Self {
        Self {
            bpm: 120,
            click_sounding: false,
            click: None,
        }
    }
}

/// The item-complete sheet while it is open, kept in the crash-recovery copy
/// so a kill before Next reopens the sheet with its answers (#2137). `now` is
/// the instant `NextItem` closes the entry at; `reading` seeds the sheet's
/// unstamped tempo rows after a resume.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ReflectionDraft {
    pub now: DateTime<Utc>,
    pub reading: TempoReading,
    pub answers: ReflectionAnswers,
}

/// What the sheet holds before Next. The shell still sends these around
/// `NextItem` itself, so a skipped sheet writes none of them.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ReflectionAnswers {
    pub marks: Vec<DraftMark>,
    pub note: String,
    /// Only the rows set by hand; the rest reseed from their stamps.
    pub tempos: Vec<DraftTempo>,
    pub felt: Option<Felt>,
    pub got_in_the_way: Vec<Obstacle>,
    /// The spans of the note's points the musician confirmed.
    pub note_points: Vec<NoteSpan>,
    pub intention_met: Option<IntentionMet>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct DraftMark {
    pub play_id: String,
    pub score: u8,
}

/// `tempo` as displayed, in `click.metre.unit`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct DraftTempo {
    pub play_id: String,
    pub tempo: u16,
    pub click: Option<ClickState>,
}

/// What the click was set to. Facts only, like `TempoReading`: the core
/// decides what they mean. The pattern never divides the tempo; the pulse
/// keeps its rate and `sounding` gates the beats.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ClickState {
    pub metre: Metre,
    /// Bitmask over `metre.beats`, LSB = beat 1. Fixed width and cheap to mask.
    pub sounding: u16,
}

// ── Transient State Types ──────────────────────────────────────────────

/// State during setlist assembly (Building phase). No `Default`: a build
/// starts from `fresh_building`, or it drops the preferred length (#1736).
#[derive(Debug, Clone)]
pub struct BuildingSession {
    pub entries: Vec<SetlistEntry>,
    /// Today's length, copied from the preference when the build starts and
    /// never carried onto `ActiveSession` (`specs/session-length.md`).
    pub length_mins: Option<u16>,
}

/// State during active practice (Active phase).
/// Persisted for crash recovery under a shell key built from `BLOB_VERSION`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ActiveSession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub current_index: usize,
    pub current_item_started_at: DateTime<Utc>,
    pub session_started_at: DateTime<Utc>,
    /// `Some` while the item-complete sheet is open on the current entry.
    pub reflection: Option<ReflectionDraft>,
    /// `Some` while the current entry runs through two or more segments.
    pub segment: Option<SegmentClock>,
}

const RETIRED_BLOB_VERSION_MAX: u32 = 6;
const _: () = assert!(ActiveSession::BLOB_VERSION > RETIRED_BLOB_VERSION_MAX);

impl ActiveSession {
    /// The crash-recovery blob is positional bincode, so a build reads only a
    /// blob of its own shape. The shell names its storage key by this number,
    /// so a shape change bumps it here and nowhere else (#1116). Versions 1 to
    /// 6 named earlier shapes and are never reused.
    pub const BLOB_VERSION: u32 = 7;

    /// `entries` is never empty during an active session, so this indexes
    /// unconditionally rather than returning an `Option`.
    pub fn current_entry(&self) -> &SetlistEntry {
        let idx = self.current_index.min(self.entries.len() - 1);
        &self.entries[idx]
    }
}

/// State during post-session review (Summary phase).
#[derive(Debug, Clone)]
pub struct SummarySession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub session_started_at: DateTime<Utc>,
    pub session_ended_at: DateTime<Utc>,
    pub session_notes: Option<String>,
    pub completion_status: CompletionStatus,
    pub session_score: Option<u8>,
}

/// The lifecycle state of a session in the core Model.
#[derive(Debug, Clone, Default)]
pub enum SessionStatus {
    #[default]
    Idle,
    Building(BuildingSession),
    Active(ActiveSession),
    Summary(SummarySession),
}

// ── Events ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SessionEvent {
    // === Building Phase ===
    StartBuilding,
    SetEntryIntention {
        entry_id: String,
        intention: Option<String>,
    },
    /// Plan the section (at most one) and the variations an entry will be
    /// practised on; empty lists clear (#2246). Building phase only: once
    /// practice starts the record is the plays, and `SwitchPlay` changes it.
    SetEntryPlan {
        entry_id: String,
        section_ids: Vec<String>,
        variation_ids: Vec<String>,
    },
    /// Set or clear the rep target for an entry during building phase.
    /// `None` disables the counter; `Some(n)` enables it with target `n`.
    SetRepTarget {
        entry_id: String,
        target: Option<u8>,
    },
    /// Set or clear the planned duration for an entry during building phase.
    /// `None` clears the planned duration; `Some(secs)` sets it (range: 60 to 3600).
    SetEntryDuration {
        entry_id: String,
        duration_secs: Option<u32>,
    },
    /// Today's length for the session being built; `None` switches it off.
    /// Never changes the musician's preferred length.
    SetSessionLength {
        length_mins: Option<u16>,
    },
    AddToSetlist {
        item_id: String,
    },
    /// One-tap "Practise this": from Idle, start building seeded with the
    /// item (a piece brings its related exercises, as `AddToSetlist`).
    StartBuildingWith {
        item_id: String,
    },
    /// "Change it first" on the Practice hero: from Idle, start building
    /// seeded with today's plan (#1082, #57). Carries only `now`, because the
    /// core re-derives the plan rather than trusting the shell's copy of it,
    /// and the derivation is clock-dependent.
    StartBuildingFromSuggestion {
        now: DateTime<Utc>,
    },
    /// The Practice hero's Start: from Idle, seed today's plan and start
    /// playing it in one step (#57). Carries only `now`, for the same reason.
    StartFromSuggestion {
        now: DateTime<Utc>,
    },
    /// The Practice tab's "Practise your priorities" (#981): from Idle, start
    /// building seeded with every starred item, least ready first. Carries only
    /// `now`, for the same reason `StartBuildingFromSuggestion` does: the core
    /// re-derives the set rather than trusting the shell's copy, and the
    /// ordering is clock-dependent.
    StartBuildingWithPriorities {
        now: DateTime<Utc>,
    },
    RemoveFromSetlist {
        entry_id: String,
    },
    /// Move the unit holding `entry_id` (its whole block, or the entry alone)
    /// to a unit position, clamped to the end (#1957).
    MoveUnit {
        entry_id: String,
        new_position: usize,
    },
    /// Move a related exercise among its block's related exercises; the
    /// anchor piece stays last (#1957).
    MoveRelated {
        entry_id: String,
        new_position: usize,
    },
    /// Drop a block's related exercises, keeping the piece (becomes standalone).
    KeepOnlyPiece {
        group_id: String,
    },
    /// Dissolve a block: its entries stay in place but become standalone.
    UngroupBlock {
        group_id: String,
    },
    /// Dissolve every block.
    UngroupAllBlocks,
    /// Remove a whole block (piece + its related exercises).
    RemoveBlock {
        group_id: String,
    },
    /// Add one more exercise into an already-present block, positioned before
    /// the block's anchor piece. Membership is binary, same idempotency as
    /// `AddToSetlist` (#939): re-adding a present item is a no-op.
    AddExerciseToBlock {
        group_id: String,
        item_id: String,
    },
    StartSession {
        now: DateTime<Utc>,
    },
    CancelBuilding,

    // === Active Phase ===
    /// Stamp the current entry's open play with its real duration and tempo
    /// ahead of the terminal transition, so the sheet's mark control can tell
    /// a play that will survive from one about to be dropped (#1758) and its
    /// rows can prefill from the stamp (#1761). The shell must send this `now`
    /// and `reading` again on the `NextItem` that follows: see `NextItem`'s
    /// own doc for what a fresher instant there invalidates.
    PrepareReflection {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// `now` closes the finished entry and must be `PrepareReflection`'s own
    /// instant when a reflection sheet came first, or its prediction goes
    /// stale (#1758). `next_item_started_at` starts the next entry's clock
    /// and first play; it should be a fresh instant taken when the shell
    /// actually advances, or the sheet's own dwell time reads as practice on
    /// the item that follows. On the last item this finishes the session.
    NextItem {
        now: DateTime<Utc>,
        next_item_started_at: DateTime<Utc>,
        reading: TempoReading,
    },
    /// Carries no reading: a skipped entry keeps no tempo (#1761 rule 7).
    SkipItem {
        now: DateTime<Utc>,
    },
    EndSessionEarly {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// Bank a pass on the current entry, past the target too (#2107). The
    /// first tap on an untouched entry writes the target; see `record_rep`.
    RepGotIt {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// Step back a pass on the current entry (floor 0).
    RepMissed {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// Close the open play and open another played this way (#1739 decision
    /// 6, #2246). Changing nothing writes nothing, so a stray tap cannot reset
    /// the repetition counter. Active phase only.
    SwitchPlay {
        entry_id: String,
        section_id: Option<String>,
        key: Option<Key>,
        variation_ids: Vec<String>,
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// Replace the open sheet's answers, whole, and save them for crash
    /// recovery (#2137). Refused whole without an open sheet or when any
    /// answer fails validation. `last_error` is left alone, since the
    /// sheet shows its refusal on Next.
    UpdateReflectionDraft {
        answers: ReflectionAnswers,
    },

    // === Summary Phase ===
    UpdateEntryNotes {
        entry_id: String,
        notes: Option<String>,
    },
    /// `play_id` names the row of the item-complete sheet being marked, and
    /// must belong to `entry_id` (#1739).
    UpdateEntryScore {
        entry_id: String,
        play_id: String,
        score: Option<u8>,
    },
    /// The manual path for a tempo: the click's evidence lands as a play
    /// closes (#1761 rule 5). `tempo` is as displayed, in `click.metre.unit`
    /// beats per minute, and normalised to crotchets before it is stored
    /// (#1499); `click` is never stored. A number the musician did not set
    /// writes nothing, and `None` clears the tempo and its pattern. `play_id`
    /// names the row, as `UpdateEntryScore`.
    UpdateEntryTempo {
        entry_id: String,
        play_id: String,
        tempo: Option<u16>,
        user_set: bool,
        click: Option<ClickState>,
    },
    UpdateSessionNotes {
        notes: Option<String>,
    },
    SaveSession {
        now: DateTime<Utc>,
    },
    DiscardSession,

    // === Recovery ===
    RecoverSession {
        session: ActiveSession,
        now: DateTime<Utc>,
    },

    // === History ===
    UpdateSessionScore {
        score: Option<u8>,
    },

    // === Active Phase, appended (#2107) ===
    /// Reverse the last got it or miss still standing on the open play.
    RepUndo {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// The click's tempo moved. A change under two seconds after the last,
    /// with no tap between, replaces it, so a stepper climb keeps where it
    /// rested and the core needs no timer.
    TempoChanged {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    /// The shell found a practice saved by an older build, whose shape this
    /// one cannot read, and deleted it (#2246).
    RetiredSessionFound,

    // === v0.17 record (#2249), appended ===
    /// The sections an entry runs through, in order. Segments sent at zero
    /// share what the others leave of the planned time. Building phase only.
    SetSegments {
        entry_id: String,
        segments: Vec<Segment>,
    },
    SetFocus {
        entry_id: String,
        focus: Option<IntentionFocus>,
    },
    /// Plan the section and variations the item was last played on.
    ApplyLastTime {
        entry_id: String,
    },
    /// A nameless trouble spot on the item, in score order (#2245's rules).
    AddTroubleSpot {
        item_id: String,
        bars: crate::domain::section::BarRange,
    },
    MoveToNextSegment {
        now: DateTime<Utc>,
        reading: TempoReading,
    },
    StayOnSegment,
    WentAway {
        at: DateTime<Utc>,
    },
    CameBack {
        at: DateTime<Utc>,
    },
    LeaveAwayOut,
    /// The finish answers, sent after `NextItem` like the marks, so a skipped
    /// sheet writes none of them. `span` must be a point read from the note.
    ConfirmNotePoint {
        entry_id: String,
        span: NoteSpan,
    },
    SetFelt {
        entry_id: String,
        felt: Option<Felt>,
    },
    ToggleObstacle {
        entry_id: String,
        obstacle: Obstacle,
    },
    AnswerIntention {
        entry_id: String,
        answer: Option<IntentionMet>,
    },
}

mod active;
mod building;
mod finish;
mod live;
mod plays;
mod record;
#[cfg(test)]
mod record_tests;
mod summary;
#[cfg(test)]
mod tests;

pub(crate) use building::last_time;
pub(crate) use live::can_stay;
pub(crate) use record::*;
pub use record::{
    Away, Felt, FocusKind, IntentionFocus, IntentionMet, NotePoint, NotePointKind, NoteSpan,
    Obstacle, Segment, SegmentClock,
};

use plays::{record_rep, record_tempo_change};

pub(crate) const RETIRED_SESSION_NOTICE: &str =
    "A practice left open before the update couldn't be picked up again.";

pub(crate) use plays::{item_seconds, play_would_survive_drop, standing_taps};
pub(crate) use summary::{save_acknowledged, save_refused};

// ── Event Handler ──────────────────────────────────────────────────────

pub fn handle_session_event(event: SessionEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        // ── Building Phase ─────────────────────────────────────────
        SessionEvent::StartBuilding => building::start_building(model),

        SessionEvent::SetEntryIntention {
            entry_id,
            intention,
        } => building::set_entry_intention(model, entry_id, intention),

        SessionEvent::SetEntryPlan {
            entry_id,
            section_ids,
            variation_ids,
        } => building::set_entry_plan(model, entry_id, section_ids, variation_ids),

        SessionEvent::SetRepTarget { entry_id, target } => {
            building::set_rep_target(model, entry_id, target)
        }

        SessionEvent::SetEntryDuration {
            entry_id,
            duration_secs,
        } => building::set_entry_duration(model, entry_id, duration_secs),

        SessionEvent::SetSessionLength { length_mins } => {
            building::set_session_length(model, length_mins)
        }

        SessionEvent::StartBuildingWith { item_id } => {
            building::start_building_with(model, item_id)
        }

        SessionEvent::StartBuildingFromSuggestion { now } => {
            building::start_building_from_suggestion(model, now)
        }

        SessionEvent::StartFromSuggestion { now } => building::start_from_suggestion(model, now),

        SessionEvent::StartBuildingWithPriorities { now } => {
            building::start_building_with_priorities(model, now)
        }

        SessionEvent::AddToSetlist { item_id } => building::add_to_setlist(model, item_id),

        SessionEvent::RemoveFromSetlist { entry_id } => {
            building::remove_from_setlist(model, entry_id)
        }

        SessionEvent::MoveUnit {
            entry_id,
            new_position,
        } => building::move_unit(model, &entry_id, new_position),

        SessionEvent::MoveRelated {
            entry_id,
            new_position,
        } => building::move_related(model, &entry_id, new_position),

        SessionEvent::KeepOnlyPiece { group_id } => building::keep_only_piece(model, group_id),

        SessionEvent::UngroupBlock { group_id } => building::ungroup_block(model, group_id),

        SessionEvent::UngroupAllBlocks => building::ungroup_all_blocks(model),

        SessionEvent::RemoveBlock { group_id } => building::remove_block(model, group_id),

        SessionEvent::AddExerciseToBlock { group_id, item_id } => {
            building::add_exercise_to_block(model, group_id, item_id)
        }

        SessionEvent::StartSession { now } => building::start_session(model, now),

        SessionEvent::CancelBuilding => building::cancel_building(model),

        // ── Active Phase ───────────────────────────────────────────
        SessionEvent::PrepareReflection { now, reading } => {
            active::prepare_reflection(model, now, reading)
        }

        SessionEvent::NextItem {
            now,
            next_item_started_at,
            reading,
        } => active::next_item(model, now, next_item_started_at, reading),

        SessionEvent::SkipItem { now } => active::skip_item(model, now),

        SessionEvent::EndSessionEarly { now, reading } => {
            active::end_session_early(model, now, reading)
        }

        SessionEvent::RepGotIt { now, reading } => {
            record_rep(model, RepAction::Success, now, &reading)
        }

        SessionEvent::RepMissed { now, reading } => {
            record_rep(model, RepAction::Missed, now, &reading)
        }

        SessionEvent::RepUndo { now, reading } => record_rep(model, RepAction::Undo, now, &reading),

        SessionEvent::TempoChanged { now, reading } => record_tempo_change(model, now, &reading),

        SessionEvent::SwitchPlay {
            entry_id,
            section_id,
            key,
            variation_ids,
            now,
            reading,
        } => {
            let way = PlayWay {
                section_id,
                key,
                variation_ids,
            };
            active::switch_play(model, entry_id, way, now, reading)
        }

        SessionEvent::RetiredSessionFound => {
            model.raise_notice(RETIRED_SESSION_NOTICE);
            crux_core::render::render()
        }

        SessionEvent::UpdateReflectionDraft { answers } => {
            active::update_reflection_draft(model, answers)
        }

        SessionEvent::SetSegments { entry_id, segments } => {
            building::set_segments(model, entry_id, segments)
        }

        SessionEvent::SetFocus { entry_id, focus } => building::set_focus(model, entry_id, focus),

        SessionEvent::ApplyLastTime { entry_id } => building::apply_last_time(model, entry_id),

        SessionEvent::AddTroubleSpot { item_id, bars } => {
            live::add_trouble_spot(model, item_id, bars)
        }

        SessionEvent::MoveToNextSegment { now, reading } => {
            live::move_to_next_segment(model, now, reading)
        }

        SessionEvent::StayOnSegment => live::stay_on_segment(model),

        SessionEvent::WentAway { at } => live::went_away(model, at),

        SessionEvent::CameBack { at } => live::came_back(model, at),

        SessionEvent::LeaveAwayOut => live::leave_away_out(model),

        SessionEvent::ConfirmNotePoint { entry_id, span } => {
            finish::confirm_note_point(model, entry_id, span)
        }

        SessionEvent::SetFelt { entry_id, felt } => finish::set_felt(model, entry_id, felt),

        SessionEvent::ToggleObstacle { entry_id, obstacle } => {
            finish::toggle_obstacle(model, entry_id, obstacle)
        }

        SessionEvent::AnswerIntention { entry_id, answer } => {
            finish::answer_intention(model, entry_id, answer)
        }

        // ── Entry Updates (Active or Summary) ──────────────────────
        // Accepted in both phases so the mid-session reflection sheet can record
        // as the user moves on. Invariant: only Completed entries can be scored.
        SessionEvent::UpdateEntryScore {
            entry_id,
            play_id,
            score,
        } => summary::update_entry_score(model, entry_id, play_id, score),

        SessionEvent::UpdateEntryTempo {
            entry_id,
            play_id,
            tempo,
            user_set,
            click,
        } => summary::update_entry_tempo(model, entry_id, play_id, tempo, user_set, click),

        SessionEvent::UpdateEntryNotes { entry_id, notes } => {
            summary::update_entry_notes(model, entry_id, notes)
        }

        SessionEvent::UpdateSessionNotes { notes } => summary::update_session_notes(model, notes),

        SessionEvent::UpdateSessionScore { score } => summary::update_session_score(model, score),

        SessionEvent::SaveSession { now } => summary::save_session(model, now),

        SessionEvent::DiscardSession => summary::discard_session(model),

        // ── Recovery ───────────────────────────────────────────────
        SessionEvent::RecoverSession { session, now } => {
            active::recover_session(model, session, now)
        }
    }
}

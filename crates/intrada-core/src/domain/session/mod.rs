use crate::app::{Effect, Event};
use crate::domain::item::ItemKind;
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
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum RepAction {
    /// Failed rep: count decremented.
    Missed,
    /// Successful rep: count incremented.
    Success,
}

/// One tap on the pass counter and when it landed. Sequence alone cannot tell
/// a steady ten from a hard-won one (#1367).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RepEvent {
    pub action: RepAction,
    pub at: DateTime<Utc>,
}

// ── Domain Types ───────────────────────────────────────────────────────

/// One stretch of an item spent on one variation (#1739): what was played,
/// as against `SetlistEntry::planned_variation_id`, which is what was planned.
/// Switching mid item closes the open play and opens another, so an entry
/// practised in C and then D holds two.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct VariationPlay {
    pub id: String,
    /// A live variant of the entry's item; `None` means unattributed, which is
    /// what a piece, and an exercise with no variations, always records.
    pub variation_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub seconds: u64,
    pub rep_target: Option<u8>,
    pub rep_count: Option<u8>,
    pub rep_target_reached: Option<bool>,
    pub rep_history: Option<Vec<RepEvent>>,
    pub achieved_tempo: Option<u16>,
    pub click_pattern: Option<ClickState>,
    pub score: Option<u8>,
}

impl VariationPlay {
    pub fn opened(
        variation_id: Option<String>,
        rep_target: Option<u8>,
        started_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: ulid::Ulid::generate().to_string(),
            variation_id,
            started_at,
            seconds: 0,
            rep_target,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            achieved_tempo: None,
            click_pattern: None,
            score: None,
        }
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
    /// The variation the builder planned to practise; the plan, not the record
    /// (#1739 decision 5). `None` = no plan. Dropped server-side until the sync
    /// engine, like `group_id` (invariant 6 scoped).
    pub planned_variation_id: Option<String>,
    /// The repetition target set in the builder, a plan on the same footing as
    /// `planned_variation_id`. Every play the entry opens starts from it, so a
    /// switch redraws the same number of slots for the new variation.
    pub planned_rep_target: Option<u8>,
    /// What was actually practised, in order. Empty for an entry never
    /// attempted; every practised entry has at least one, a piece included
    /// (#1739 decision 3).
    pub plays: Vec<VariationPlay>,
}

impl SetlistEntry {
    pub fn open_play(&self) -> Option<&VariationPlay> {
        self.plays.last()
    }

    pub fn open_play_mut(&mut self) -> Option<&mut VariationPlay> {
        self.plays.last_mut()
    }

    /// The one place several marks collapse into one (#1739 decision 9): the
    /// mean of the plays that carry a score, rounded to nearest. Per-variation
    /// history reads the plays and never this.
    pub fn score_summary(&self) -> Option<u8> {
        let scored: Vec<u16> = self
            .plays
            .iter()
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
impl VariationPlay {
    pub(crate) fn fixture() -> Self {
        Self {
            id: "play-1".to_string(),
            variation_id: None,
            started_at: DateTime::<Utc>::from_timestamp(0, 0).expect("epoch"),
            seconds: 60,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            achieved_tempo: None,
            click_pattern: None,
            score: None,
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
            planned_variation_id: None,
            planned_rep_target: None,
            plays: Vec::new(),
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
}

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

/// State during setlist assembly (Building phase).
#[derive(Debug, Clone, Default)]
pub struct BuildingSession {
    pub entries: Vec<SetlistEntry>,
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
}

const RETIRED_BLOB_VERSION_MAX: u32 = 3;
const _: () = assert!(ActiveSession::BLOB_VERSION > RETIRED_BLOB_VERSION_MAX);

impl ActiveSession {
    /// The crash-recovery blob is positional bincode, so a build reads only a
    /// blob of its own shape. The shell names its storage key by this number,
    /// so a shape change bumps it here and nowhere else (#1116). Versions 1 to
    /// 3 named earlier shapes and are never reused.
    pub const BLOB_VERSION: u32 = 4;

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
    /// Plan which variation an entry will be practised on; `None` clears
    /// (#1083, narrowed by #1739 decision 5). Building phase only: once
    /// practice starts the record is the plays, and `SwitchVariation` is what
    /// changes it. The variation must be a live variant of the entry's item.
    /// Local-first only until sync.
    SetEntryVariant {
        entry_id: String,
        variant_id: Option<String>,
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
    AddToSetlist {
        item_id: String,
    },
    /// One-tap "Practise this": from Idle, start building seeded with the
    /// item (a piece brings its related exercises, as `AddToSetlist`).
    StartBuildingWith {
        item_id: String,
    },
    /// The Up next card's CTA: from Idle, start building seeded with the
    /// suggested block (#1082). Carries only `now`, because the core
    /// re-derives the suggestion rather than trusting the shell's copy of it,
    /// and the derivation is clock-dependent.
    StartBuildingFromSuggestion {
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
    ReorderSetlist {
        entry_id: String,
        new_position: usize,
    },
    /// Move a whole block to a new unit-position (blocks + standalone items,
    /// in order). Keeps the block contiguous.
    ReorderBlock {
        group_id: String,
        new_position: usize,
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
    /// Bank a pass on the current entry (capped at target). The first tap on
    /// an untouched entry writes the target too; see `record_rep`.
    RepGotIt {
        now: DateTime<Utc>,
    },
    /// Step back a pass on the current entry (floor 0).
    RepMissed {
        now: DateTime<Utc>,
    },
    /// Close the open play and open another on `variation_id` (#1739 decision
    /// 6). Switching to the variation already open writes nothing, so a stray
    /// tap cannot reset the repetition counter. Active phase only.
    SwitchVariation {
        entry_id: String,
        variation_id: Option<String>,
        now: DateTime<Utc>,
        reading: TempoReading,
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
}

mod active;
mod building;
mod plays;
mod summary;
#[cfg(test)]
mod tests;

use plays::record_rep;

pub(crate) use plays::play_would_survive_drop;
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

        SessionEvent::SetEntryVariant {
            entry_id,
            variant_id,
        } => building::set_entry_variant(model, entry_id, variant_id),

        SessionEvent::SetRepTarget { entry_id, target } => {
            building::set_rep_target(model, entry_id, target)
        }

        SessionEvent::SetEntryDuration {
            entry_id,
            duration_secs,
        } => building::set_entry_duration(model, entry_id, duration_secs),

        SessionEvent::StartBuildingWith { item_id } => {
            building::start_building_with(model, item_id)
        }

        SessionEvent::StartBuildingFromSuggestion { now } => {
            building::start_building_from_suggestion(model, now)
        }

        SessionEvent::StartBuildingWithPriorities { now } => {
            building::start_building_with_priorities(model, now)
        }

        SessionEvent::AddToSetlist { item_id } => building::add_to_setlist(model, item_id),

        SessionEvent::RemoveFromSetlist { entry_id } => {
            building::remove_from_setlist(model, entry_id)
        }

        SessionEvent::ReorderSetlist {
            entry_id,
            new_position,
        } => building::reorder_setlist(model, entry_id, new_position),

        SessionEvent::ReorderBlock {
            group_id,
            new_position,
        } => building::reorder_block(model, group_id, new_position),

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

        SessionEvent::RepGotIt { now } => record_rep(model, RepAction::Success, now),

        SessionEvent::RepMissed { now } => record_rep(model, RepAction::Missed, now),

        SessionEvent::SwitchVariation {
            entry_id,
            variation_id,
            now,
            reading,
        } => active::switch_variation(model, entry_id, variation_id, now, reading),

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

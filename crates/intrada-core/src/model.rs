use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::analytics::{AnalyticsView, LastPractisedView};
use crate::domain::chart::{ChordChart, ScaffoldKind};
use crate::domain::item::{Item, ItemKind, Modality};
use crate::domain::profile::{Profile, ProfileField, ProfileView};
use crate::domain::session::{
    ActiveSession, ClickState, CompletionStatus, EntryStatus, PracticeSession, RepEvent,
    SessionStatus, SetlistEntry, SummarySession, VariationPlay,
};
use crate::domain::variant::ladder_is_all_keys;
use crate::domain::Metre;
use crate::domain::{LibrarySort, ListQuery};
use crate::recognition::PhotoDraft;
use crate::suggestion::SuggestedSession;

/// Internal application state — not exposed to shells.
#[derive(Debug, Default)]
pub struct Model {
    /// The device's UTC offset in minutes (BST = 60, New York = −240/−300),
    /// reported by the shell via `SetUtcOffset` at launch and on foreground.
    /// Turns UTC instants into user-local days for analytics (#1330); 0 until
    /// the shell reports, which degrades to the old UTC day boundary.
    pub utc_offset_minutes: i32,
    pub items: Vec<Item>,
    pub sessions: Vec<PracticeSession>,
    pub session_status: SessionStatus,
    /// The finished practice whose write is with the store. Pushed into
    /// `sessions` only when the store acknowledges it; a failure hands it
    /// back to the summary so Save can be tapped again (#974).
    pub saving_session: Option<PracticeSession>,
    pub active_query: Option<ListQuery>,
    pub active_sort: LibrarySort,
    pub last_error: Option<String>,
    /// Where the current error is, when the handler that set it could say.
    /// Cleared before every event in `App::update`, so it always belongs to the
    /// error the event in hand reported. It says nothing about shell state that
    /// has changed since, which is the shell's to drop (#1595).
    pub last_error_target: Option<FormErrorTarget>,
    /// Set when the user dismisses the error banner. While true, failures
    /// routed through [`Model::surface_error`] are silently swallowed, which
    /// avoids the "dismiss, next write fails, banner reappears" loop when the
    /// underlying problem hasn't been resolved. Cleared only by a write the
    /// store acknowledges ([`Model::record_ack`]), not by an optimistic
    /// mutation, since only the store can say it has recovered (#346, #1936).
    pub error_muted: bool,
    pub practice_summaries: HashMap<String, ItemPracticeSummary>,
    /// Device data, not account data (`specs/profile.md`).
    pub profile: Profile,
    /// Bumped each time an error is raised or surfaced, never because one is
    /// still standing, so a send accepted under the banner does not read as
    /// refused (#1056, #1936).
    pub error_seq: u64,
    /// A true outcome that is not a failure, for the neutral banner (#1325).
    /// Never muted: it follows the musician's own action, like a refusal.
    pub last_notice: Option<String>,
    /// Bumped on every raise, identical text included, never because a
    /// notice is standing.
    pub notice_seq: u64,
    /// What the last `ReadPhoto` produced. Transient: the confirm surface reads
    /// it, the user edits it, and `DiscardPhotoDraft` clears it. Never written
    /// to an item without that confirmation (spec non-goal "no silent write").
    pub photo_recognition: PhotoRecognition,
}

/// Model-side recognition state. The view projection is
/// [`PhotoRecognitionView`].
#[derive(Debug, Default, Clone, PartialEq)]
pub enum PhotoRecognition {
    #[default]
    Idle,
    Reading {
        photo_id: String,
    },
    Ready {
        photo_id: String,
        draft: PhotoDraft,
    },
    /// The device has no recognition. Not an error: the photo is still saved
    /// and the user types the fields.
    Unsupported {
        photo_id: String,
    },
    Failed {
        photo_id: String,
    },
}

impl Model {
    /// Refuse the event in hand. Never muted: the musician needs to see why
    /// their own action did not happen, whatever they dismissed earlier.
    pub fn raise_error(&mut self, msg: impl Into<String>) {
        self.last_error = Some(msg.into());
        self.error_seq = self.error_seq.wrapping_add(1);
    }

    /// Surface an error from a background failure. Respects the
    /// dismiss-mute state set by [`Model::dismiss_error`]: if the user has
    /// already dismissed the banner and the store has not yet acknowledged a
    /// write, the error is silently swallowed to stop the banner re-popping.
    /// Also dedupes identical messages to avoid render storms during burst
    /// failures (#346).
    pub fn surface_error(&mut self, msg: impl Into<String>) {
        if self.error_muted {
            return;
        }
        self.error_seq = self.error_seq.wrapping_add(1);
        let msg = msg.into();
        if self.last_error.as_deref() == Some(msg.as_str()) {
            return;
        }
        self.last_error = Some(msg);
    }

    /// Leaves the dismiss mute alone: only an acknowledged write lifts it (#1936).
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// The store confirmed a write, so later failures are worth showing
    /// again. Leaves a standing banner alone.
    pub fn record_ack(&mut self) {
        self.error_muted = false;
    }

    /// User explicitly dismissed the error banner. Clears the active error
    /// and enters the mute state so subsequent background failures don't
    /// immediately re-pop the banner.
    pub fn dismiss_error(&mut self) {
        self.last_error = None;
        self.error_muted = true;
    }

    pub fn raise_notice(&mut self, msg: impl Into<String>) {
        self.last_notice = Some(msg.into());
        self.notice_seq = self.notice_seq.wrapping_add(1);
    }

    pub fn clear_notice(&mut self) {
        self.last_notice = None;
    }
}

/// Where on the create form the failure the banner names actually is (#1595).
/// Set only by the handler that reported the error, and only where the form has
/// somewhere to point; every other path leaves it `None`, which reads as the
/// banner alone.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FormErrorTarget {
    Piece {
        field: FormErrorField,
    },
    /// The chord chart as a whole, which is what a chart holding no bars at all
    /// produces: there is nothing inside it to point at.
    Chart,
    /// One bar of the chord chart, numbered from 1, and the token the parser
    /// stumbled on.
    ChartBar {
        bar_number: usize,
        token: String,
    },
    Profile {
        field: ProfileField,
    },
    /// A staged exercise by position, from 0, in the exercises the event
    /// carried. `field` is `None` for a chosen exercise, which has no field of
    /// its own. The shell reorders that list without telling the core, so the
    /// position holds only until it does (spec decision 12).
    Exercise {
        index: usize,
        field: Option<FormErrorField>,
    },
}

/// A field the add form shows. A validation failure on anything else maps to no
/// target rather than a field nobody can reach. Named apart from the shell's own
/// `FormField` view, which it would otherwise shadow inside the app module.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FormErrorField {
    Title,
    Composer,
    Tempo,
    Notes,
    Tags,
    /// The exercise's inline variation rows (#1831). Appended last: the
    /// bincode wire is positional.
    Variations,
}

/// Serializable view state sent to shells for rendering.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,
    /// `items`, unfiltered by `active_query` — for pickers that curate their
    /// own subset rather than mirroring the Library screen's filter (#1484).
    pub all_items: Vec<LibraryItemView>,
    /// Up to 5 items with practice history, most recently practised first,
    /// for the builder's "Recently practised" quick-add section (#1362).
    pub recently_practised: Vec<LibraryItemView>,
    /// Active filter, mirrored so the shell's pill reads one source of truth (#792).
    pub active_query: Option<ListQuery>,
    /// Active sort, mirrored so the shell's menu reads one source of truth.
    pub active_sort: LibrarySort,
    /// Counts of the visible (filtered) set so the subtitle describes what's
    /// actually on screen and reacts to the filter + search.
    pub visible_pieces: usize,
    pub visible_exercises: usize,
    /// Distinct tags across the whole library (case-folded dedupe, sorted) —
    /// the vocabulary for the filter sheet and the add/edit autocomplete.
    pub available_tags: Vec<String>,
    /// Distinct composers (sorted) for the add/edit autocomplete. Computed
    /// pre-filter so it stays the full vocabulary when the list is narrowed.
    pub available_composers: Vec<String>,
    pub sessions: Vec<PracticeSessionView>,
    pub active_session: Option<ActiveSessionView>,
    pub building_setlist: Option<BuildingSetlistView>,
    pub summary: Option<SummaryView>,
    pub error: Option<String>,
    /// See [`Model::last_error_target`].
    pub error_target: Option<FormErrorTarget>,
    /// See `Model::error_seq` — compare around a send instead of the message.
    pub error_seq: u64,
    /// See [`Model::last_notice`]: true, expected, not a failure (#1325).
    pub notice: Option<String>,
    /// See [`Model::notice_seq`].
    pub notice_seq: u64,
    pub analytics: Option<AnalyticsView>,
    pub last_practised: Option<LastPractisedView>,
    pub profile: ProfileView,
    /// The one suggested session on the Practice tab (#1082). `None` whenever
    /// nothing qualifies: it suggests, it never gates.
    pub up_next: Option<SuggestedSession>,
    /// Anything starred? The Practice tab's "Practise your priorities" button
    /// shows only when this is true, and like `up_next` it is derived pre-filter (#981).
    pub has_priorities: bool,
    /// What the last photographed page was read into, for the confirm surface.
    pub photo_recognition: PhotoRecognitionView,
    /// Appended last: the bincode wire is positional.
    pub limits: LimitsView,
}

/// The bounds `validation.rs` enforces, projected for the controls that offer
/// them: a control repeating the numbers keeps offering the old range once one
/// moves, and the write is then refused with nothing on screen (#1512).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LimitsView {
    pub metre_beats_min: u8,
    pub metre_beats_max: u8,
    pub metre_units: Vec<u8>,
    pub rep_target_min: u8,
    pub rep_target_max: u8,
    pub rep_target_default: u8,
    pub planned_duration_min_secs: u32,
    pub planned_duration_max_secs: u32,
}

impl Default for LimitsView {
    fn default() -> Self {
        use crate::validation;

        LimitsView {
            metre_beats_min: validation::MIN_METRE_BEATS,
            metre_beats_max: validation::MAX_METRE_BEATS,
            metre_units: validation::METRE_UNITS.to_vec(),
            rep_target_min: validation::MIN_REP_TARGET,
            rep_target_max: validation::MAX_REP_TARGET,
            rep_target_default: validation::DEFAULT_REP_TARGET,
            planned_duration_min_secs: validation::MIN_PLANNED_DURATION_SECS,
            planned_duration_max_secs: validation::MAX_PLANNED_DURATION_SECS,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PhotoRecognitionView {
    pub status: PhotoRecognitionStatus,
    /// The photo being read, so the shell can show it beside the fields.
    pub photo_id: Option<String>,
    /// Present only under `Ready`. Each field carries its own `weak`, which is
    /// what the form marks; there is deliberately no whole-draft flag beside
    /// it, since nothing would read one.
    pub draft: Option<PhotoDraft>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PhotoRecognitionStatus {
    #[default]
    Idle,
    Reading,
    Ready,
    Unsupported,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LinkedExerciseView {
    pub id: String,
    pub title: String,
    pub key: Option<String>,
    pub tempo: Option<String>,
    pub practice: Option<ItemPracticeSummary>,
    /// This exercise's latest score *on the piece it's linked from* (#1087 B2),
    /// derived from the shared session block — distinct from `practice`, which
    /// is the exercise's flat score across every context. `None` when the pair
    /// has never been practised together (or scored) in a session.
    #[serde(default)]
    pub piece_context_score: Option<u8>,
}

/// The derived scaffold curriculum for a charted piece — the read-only preview
/// (Phase A). Recomputed from the stored chart in `build_view`; the shell renders
/// it and never derives.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScaffoldPreviewView {
    /// The key the exercises are derived in (the piece's key).
    pub key: String,
    pub specs: Vec<ScaffoldSpecView>,
    /// How many specs hit the arpeggio fallback (out-of-vocab chord).
    pub fallback_total: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScaffoldSpecView {
    /// Row identity: the shell sends the ticked `kind`s back on commit and the
    /// core re-derives, so spec content never crosses the wire (#1106).
    pub kind: ScaffoldKind,
    pub title: String,
    pub rationale: String,
    pub key: String,
    /// This spec fell back to the chord's arpeggio for at least one change.
    pub fallback: bool,
    /// A matching exercise is already linked to the piece (dedup surface).
    pub already_linked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PieceRefView {
    pub id: String,
    pub title: String,
    /// Composer, when the piece has one — disambiguates identically titled
    /// pieces in provenance UI (#1049).
    pub subtitle: Option<String>,
}

/// One piece an exercise is used in, or the "On its own" bucket. A row comes
/// from the piece's `linked_exercise_ids` (an intention), from practising the
/// two together (history, resolved per session from `group_id` — #1087 B1), or
/// from both; `linked` says which (#1363).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ExerciseUsageView {
    /// `None` is the "On its own" bucket: practice with no piece in the block.
    pub piece: Option<PieceRefView>,
    /// Always `false` for "On its own".
    pub linked: bool,
    pub latest_score: Option<u8>,
    /// Distinct sessions the exercise was practised in, in this context.
    pub session_count: usize,
    /// Most recent session date for this context (RFC3339, sorts as a string).
    pub last_practiced_at: Option<String>,
    /// `true` when this row's `piece` no longer exists in the library — the
    /// piece was deleted since it was practised (#1093, decision 2a). The row
    /// is kept (its sessions are real history), but `piece.title` is the
    /// snapshot title and the shell renders the row muted + non-tappable. Always
    /// `false` for the "On its own" bucket.
    #[serde(default)]
    pub piece_removed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LibraryItemView {
    pub id: String,
    pub item_type: ItemKind,
    pub title: String,
    pub subtitle: String,
    pub key: Option<String>,
    pub modality: Option<Modality>,
    pub tempo: Option<String>,
    pub tempo_marking: Option<String>,
    pub tempo_bpm: Option<u16>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub practice: Option<ItemPracticeSummary>,
    pub latest_achieved_tempo: Option<u16>,
    pub priority: bool,
    pub linked_exercises: Vec<LinkedExerciseView>,
    /// For exercises: every piece this exercise is used in, plus the "On its
    /// own" bucket — links and practice history merged (#1363). Empty for
    /// pieces.
    #[serde(default)]
    pub used_in: Vec<ExerciseUsageView>,
    /// For a charted piece: the derived scaffold curriculum (read-only preview).
    /// `None` for exercises and un-charted pieces.
    #[serde(default)]
    pub scaffold_preview: Option<ScaffoldPreviewView>,
    /// The parsed chord chart, for pieces that have one — the shell renders the
    /// bar grid from `symbol.raw` and pre-fills the editor from it.
    #[serde(default)]
    pub chord_chart: Option<ChordChart>,
    /// The piece's time signature, which seeds the click's bar (#1499).
    pub metre: Option<Metre>,
    /// The exercise's variation ladder with per-variation practice state (#1083);
    /// empty for pieces and un-laddered exercises.
    #[serde(default)]
    pub variants: Vec<VariantView>,
    /// Every live rung of `variants` names a key, so the shell prints "keys"
    /// rather than "variations" beside the count (#1467). False for pieces and for
    /// a ladder with no rungs. No `serde(default)`: the bridge is positional
    /// bincode, where serde never reaches a default, and the attribute would
    /// read as "optional on the wire" to the next person (#846).
    pub ladder_is_keys: bool,
    #[serde(default)]
    pub photo_id: Option<String>,
    /// An exercise in several keys has no single key (#1783 decision 1): the
    /// Add and Edit forms hide the Key field once the ladder has a live
    /// rung. `true` for a piece and for an un-laddered exercise. No
    /// `serde(default)`, matching `ladder_is_keys`: the bridge is positional
    /// bincode, where a default is never read (#846).
    pub shows_key: bool,
}

/// One variation of an exercise's ladder with its derived practice state (#1083).
/// Only live (non-tombstoned) variations reach the view, in ladder order. Users
/// see "Variations"; `variant` is the core's name and never appears on screen.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct VariantView {
    pub id: String,
    pub label: String,
    pub position: usize,
    pub latest_score: Option<u8>,
    pub score_history: Vec<ScoreHistoryEntry>,
    /// Latest score has reached `SOLID_SCORE_MIN` (8 of 10).
    pub is_solid: bool,
    pub caption: String,
}

/// The saved mark in the musician's words: the one wording the Library row and
/// the picker's out-of-session fallback share (#1809).
pub(crate) fn saved_mark_caption(latest_score: Option<u8>, is_solid: bool) -> String {
    match latest_score {
        Some(score) if is_solid => format!("Solid · {score} of 10"),
        Some(score) => format!("{score} of 10"),
        None => "Not yet played".to_string(),
    }
}

/// One row of the player's variation picker, captioned by the core so a
/// variation played earlier in this item never reads as not yet played (#1784).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PickerVariationView {
    pub id: String,
    pub label: String,
    pub caption: String,
    pub is_solid: bool,
}

/// One stretch of an item spent on one variation, as the sheet and the
/// Progress screen read it (#1739). `variation_label` is resolved here,
/// tombstones included, because a session practised on a variation that has
/// since been deleted still has to say what it was.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct VariationPlayView {
    pub id: String,
    pub variation_id: Option<String>,
    pub variation_label: Option<String>,
    pub seconds: u64,
    pub duration_display: String,
    pub rep_target: Option<u8>,
    pub rep_count: Option<u8>,
    pub rep_target_reached: Option<bool>,
    pub rep_history: Option<Vec<RepEvent>>,
    pub achieved_tempo: Option<u16>,
    pub click_pattern: Option<ClickState>,
    /// `achieved_tempo` in the unit of the click that stamped it, crotchets
    /// when no click did, so the shell never converts a tempo (#1761).
    pub tempo_display: Option<u16>,
    pub score: Option<u8>,
    /// Whether marking this play now would stick: `false` means the pending
    /// terminal transition will drop it as a stray tap on the picker before a
    /// mark could ever be attached (#1758). Only accurate on the reflection
    /// sheet, after `PrepareReflection`; mid-item it reads the open play as
    /// incidental regardless of how long it has actually run.
    pub is_markable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ItemPracticeSummary {
    pub session_count: usize,
    pub total_minutes: u32,
    pub latest_score: Option<u8>,
    pub score_history: Vec<ScoreHistoryEntry>,
    pub latest_tempo: Option<u16>,
    pub tempo_trend: TempoTrendView,
    /// Most recent session date for this item (max `started_at`), independent
    /// of whether a score/tempo was recorded. `None` if never practised.
    /// RFC3339 — sorts chronologically as a string.
    pub last_practiced_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScoreHistoryEntry {
    pub session_date: String,
    pub score: u8,
    pub session_id: String,
}

/// One practised session's slot in a `TempoTrendView`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoTrendPoint {
    pub session_date: String,
    pub session_id: String,
    pub tempo: Option<u16>,
}

/// An item's measured tempo over time, as a plottable series. A session that
/// measured no tempo keeps its slot with `tempo: None`, so the chart breaks the
/// line there rather than drawing a zero or interpolating across it (#1420).
/// Every number present was measured, because the evidence contract keeps
/// unevidenced defaults out of `achieved_tempo` upstream (T16, #1422).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoTrendView {
    /// Oldest first, the reverse of `score_history`. One point per setlist
    /// entry, matching `session_count`: an item practised twice in one session
    /// contributes two points at the same date.
    pub points: Vec<TempoTrendPoint>,
    /// Two or more measured tempos; below that there is no direction to read.
    pub has_trend: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeSessionView {
    pub id: String,
    pub started_at: String,
    pub finished_at: String,
    pub total_duration_display: String,
    pub total_duration_summary: String,
    pub completion_status: CompletionStatus,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_score: Option<u8>,
    /// The card's line of what was played, e.g. "Major scales in C, G and
    /// D · Arpeggios in E♭ major" (#1785). Built here rather than the shell
    /// so a long session is cut off honestly, at a fragment boundary with
    /// "and N more", instead of a mid-word ellipsis.
    pub played_summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetlistEntryView {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
    pub duration_display: String,
    pub status: EntryStatus,
    pub notes: Option<String>,
    pub intention: Option<String>,
    pub planned_duration_secs: Option<u32>,
    pub planned_duration_display: Option<String>,
    /// The block this entry belongs to in the builder; `None` = standalone.
    pub group_id: Option<String>,
    /// The variation the builder planned to practise (#1739 decision 5).
    pub planned_variation_id: Option<String>,
    /// The repetition target set in the builder, which every play starts from.
    pub planned_rep_target: Option<u8>,
    /// What was actually practised, in order.
    pub plays: Vec<VariationPlayView>,
    /// The mean of the plays that carry a mark, rounded to nearest, and `None`
    /// when none do. The only place several marks collapse into one (#1739
    /// decision 9): per-variation history reads `plays` and never this.
    pub score_summary: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ActiveSessionView {
    pub current_item_title: String,
    pub current_item_type: ItemKind,
    pub current_position: usize,
    pub total_items: usize,
    pub started_at: String,
    /// Wall-clock anchor (RFC3339 UTC) for the *current item*. Resets to
    /// "now" on each item advance (Next / Skip). The shell derives the
    /// per-item elapsed timer from `Utc::now() - current_item_started_at`
    /// rather than incrementing a counter — survives WebView suspension /
    /// tab backgrounding without drift.
    pub current_item_started_at: String,
    pub entries: Vec<SetlistEntryView>,
    pub current_rep_target: Option<u8>,
    pub current_rep_count: Option<u8>,
    pub current_rep_target_reached: Option<bool>,
    pub current_rep_history: Option<Vec<RepEvent>>,
    /// How many slots the counter draws before the first tap. A drawn target
    /// is not a recorded one: `current_rep_target` stays `None` until a tap.
    pub current_rep_slots: u8,
    /// The variation the open play is on, so the player's picker can show what
    /// is being practised right now (#1739). `None` = unattributed.
    pub current_variation_id: Option<String>,
    pub current_variation_label: Option<String>,
    pub current_planned_duration_secs: Option<u32>,
    pub next_item_title: Option<String>,
    /// The current entry's "Aim" note, set in the builder (`EntrySettingsSheet`).
    pub current_item_intention: Option<String>,
    /// The current item's library notes (`Item.notes`), distinct from
    /// `current_item_intention`, which is the session-level "Aim".
    pub current_item_notes: Option<String>,
    /// The anchor piece's title, when the current entry is a related exercise
    /// grouped into that piece's block; `None` for a standalone entry or when
    /// the current entry *is* the anchor piece itself.
    pub current_related_piece_title: Option<String>,
    /// The current item's own declared tempo marking/bpm (from its library
    /// entry) — the practice target, distinct from `achieved_tempo` (what was
    /// actually played, logged after completion).
    pub current_item_tempo_marking: Option<String>,
    pub current_item_tempo_bpm: Option<u16>,
    /// The piece's metre, the answer the click sheet opens with (T19).
    pub current_item_metre: Option<Metre>,
    /// The current item's variations with the picker's caption for each (#1784).
    pub current_variations: Vec<PickerVariationView>,
}

/// A unit in the builder queue: a block (a piece with its related exercises) or
/// a single standalone item. `group_id == None` means standalone.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetlistBlockView {
    pub group_id: Option<String>,
    /// The block's anchor-piece title; `None` for a standalone item.
    pub piece_title: Option<String>,
    pub related_count: usize,
    pub duration_display: String,
    pub entries: Vec<SetlistEntryView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BuildingSetlistView {
    pub entries: Vec<SetlistEntryView>,
    pub item_count: usize,
    /// The same entries grouped into ordered units (blocks + standalone items).
    pub blocks: Vec<SetlistBlockView>,
    pub block_count: usize,
    /// Sum of the entries' planned durations; `None` when nothing is planned
    /// so shells can fall back to counts-only copy.
    pub total_duration_display: Option<String>,
    pub total_duration_summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SummaryView {
    pub total_duration_display: String,
    pub completion_status: CompletionStatus,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_score: Option<u8>,
}

// ── Test fixtures ─────────────────────────────────────────────────────
// One place per many-field view type, composed with struct update, so a new
// field costs one edit rather than one per call site (CLAUDE.md, Testing).

#[cfg(test)]
impl LibraryItemView {
    pub(crate) fn fixture(id: &str, title: &str, item_type: ItemKind) -> Self {
        Self {
            id: id.to_string(),
            item_type,
            title: title.to_string(),
            subtitle: String::new(),
            key: None,
            modality: None,
            tempo: None,
            tempo_marking: None,
            tempo_bpm: None,
            notes: None,
            tags: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
            practice: None,
            latest_achieved_tempo: None,
            priority: false,
            linked_exercises: Vec::new(),
            used_in: Vec::new(),
            scaffold_preview: None,
            chord_chart: None,
            metre: None,
            variants: Vec::new(),
            ladder_is_keys: false,
            photo_id: None,
            shows_key: true,
        }
    }
}

#[cfg(test)]
impl LinkedExerciseView {
    pub(crate) fn fixture(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            key: None,
            tempo: None,
            practice: None,
            piece_context_score: None,
        }
    }
}

#[cfg(test)]
impl VariantView {
    pub(crate) fn fixture(id: &str, label: &str, position: usize) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position,
            latest_score: None,
            score_history: Vec::new(),
            is_solid: false,
            caption: saved_mark_caption(None, false),
        }
    }

    pub(crate) fn scored(self, score: u8) -> Self {
        let is_solid = score >= crate::domain::variant::SOLID_SCORE_MIN;
        Self {
            latest_score: Some(score),
            is_solid,
            caption: saved_mark_caption(Some(score), is_solid),
            ..self
        }
    }
}

#[cfg(test)]
impl ItemPracticeSummary {
    pub(crate) fn fixture() -> Self {
        Self {
            session_count: 0,
            total_minutes: 0,
            latest_score: None,
            score_history: Vec::new(),
            latest_tempo: None,
            tempo_trend: TempoTrendView::default(),
            last_practiced_at: None,
        }
    }
}

// ── View helpers ──────────────────────────────────────────────────────

/// Every variation label in the library, tombstoned ones included. A session
/// practised on a variation that has since been deleted still has to say what
/// it was, which is what the tombstone exists for (#1739).
pub type VariationLabels<'a> = HashMap<&'a str, &'a str>;

pub fn variation_labels(items: &[Item]) -> VariationLabels<'_> {
    items
        .iter()
        .flat_map(|i| i.variants.iter())
        .map(|v| (v.id.as_str(), v.label.as_str()))
        .collect()
}

pub fn play_to_view(
    play: &VariationPlay,
    entry: &SetlistEntry,
    labels: &VariationLabels,
) -> VariationPlayView {
    VariationPlayView {
        id: play.id.clone(),
        variation_id: play.variation_id.clone(),
        variation_label: play
            .variation_id
            .as_deref()
            .and_then(|id| labels.get(id).map(|l| (*l).to_string())),
        seconds: play.seconds,
        duration_display: crate::domain::session::format_duration_display(play.seconds),
        rep_target: play.rep_target,
        rep_count: play.rep_count,
        rep_target_reached: play.rep_target_reached,
        rep_history: play.rep_history.clone(),
        achieved_tempo: play.achieved_tempo,
        click_pattern: play.click_pattern.clone(),
        tempo_display: play.achieved_tempo.map(|crotchets| {
            play.click_pattern
                .as_ref()
                .map_or(crotchets, |c| c.metre.displayed_bpm(crotchets))
        }),
        score: play.score,
        is_markable: crate::domain::session::play_would_survive_drop(entry, play),
    }
}

pub fn entry_to_view(entry: &SetlistEntry, labels: &VariationLabels) -> SetlistEntryView {
    SetlistEntryView {
        id: entry.id.clone(),
        item_id: entry.item_id.clone(),
        item_title: entry.item_title.clone(),
        item_type: entry.item_type.clone(),
        position: entry.position,
        duration_display: crate::domain::session::format_duration_display(entry.duration_secs),
        status: entry.status.clone(),
        notes: entry.notes.clone(),
        intention: entry.intention.clone(),
        planned_duration_secs: entry.planned_duration_secs,
        planned_duration_display: entry
            .planned_duration_secs
            .map(|secs| crate::domain::session::format_planned_duration(u64::from(secs))),
        group_id: entry.group_id.clone(),
        planned_variation_id: entry.planned_variation_id.clone(),
        planned_rep_target: entry.planned_rep_target,
        plays: entry
            .plays
            .iter()
            .map(|p| play_to_view(p, entry, labels))
            .collect(),
        score_summary: entry.score_summary(),
    }
}

/// Project flat entry views into ordered units: a contiguous run sharing a
/// `group_id` becomes one block (related exercises first, piece last); every
/// ungrouped entry is its own standalone unit.
pub fn build_blocks(entries: &[SetlistEntryView]) -> Vec<SetlistBlockView> {
    let mut blocks: Vec<SetlistBlockView> = Vec::new();
    for entry in entries {
        let extends = match (&entry.group_id, blocks.last()) {
            (Some(g), Some(last)) => last.group_id.as_deref() == Some(g.as_str()),
            _ => false,
        };
        let is_piece = entry.item_type == ItemKind::Piece;
        if extends {
            let block = blocks.last_mut().expect("extends implies a last block");
            if is_piece {
                block.piece_title = Some(entry.item_title.clone());
            } else {
                block.related_count += 1;
            }
            block.entries.push(entry.clone());
        } else {
            let grouped = entry.group_id.is_some();
            blocks.push(SetlistBlockView {
                group_id: entry.group_id.clone(),
                piece_title: (grouped && is_piece).then(|| entry.item_title.clone()),
                related_count: usize::from(grouped && !is_piece),
                duration_display: String::new(),
                entries: vec![entry.clone()],
            });
        }
    }
    for block in &mut blocks {
        let total: u32 = block
            .entries
            .iter()
            .filter_map(|e| e.planned_duration_secs)
            .sum();
        block.duration_display = if total > 0 {
            crate::domain::session::format_planned_duration(u64::from(total))
        } else {
            "—".to_string()
        };
    }
    blocks
}

pub fn build_active_session_view(
    active: &ActiveSession,
    item_index: &HashMap<&str, &Item>,
    labels: &VariationLabels,
    current_variations: &[VariantView],
) -> ActiveSessionView {
    let safe_index = active.current_index.min(active.entries.len() - 1);
    let current = active.current_entry();
    let open = current.open_play();

    // Breadcrumb only applies to a related exercise practiced inside a block —
    // not the anchor piece itself.
    let current_related_piece_title = (current.item_type != ItemKind::Piece)
        .then_some(current.group_id.as_deref())
        .flatten()
        .and_then(|group_id| {
            active
                .entries
                .iter()
                .find(|e| e.item_type == ItemKind::Piece && e.group_id.as_deref() == Some(group_id))
                .map(|e| e.item_title.clone())
        });

    // O(1) via the caller's id→Item index (same one build_view uses for
    // linked-exercise lookups) rather than a linear scan over the library.
    let current_item_tempo = item_index
        .get(current.item_id.as_str())
        .and_then(|i| i.tempo.as_ref());

    ActiveSessionView {
        current_item_title: current.item_title.clone(),
        current_item_type: current.item_type.clone(),
        current_position: active.current_index,
        total_items: active.entries.len(),
        started_at: active.session_started_at.to_rfc3339(),
        current_item_started_at: active.current_item_started_at.to_rfc3339(),
        entries: active
            .entries
            .iter()
            .map(|e| entry_to_view(e, labels))
            .collect(),
        // Repetitions belong to the open play, so they reset when a switch
        // opens the next one (#1739 decision 6).
        current_rep_target: open.and_then(|p| p.rep_target),
        current_rep_count: open.and_then(|p| p.rep_count),
        current_rep_target_reached: open.and_then(|p| p.rep_target_reached),
        current_rep_history: open.and_then(|p| p.rep_history.clone()),
        current_rep_slots: open
            .and_then(|p| p.rep_target)
            .unwrap_or(crate::validation::DEFAULT_REP_TARGET),
        current_variation_id: open.and_then(|p| p.variation_id.clone()),
        current_variation_label: open
            .and_then(|p| p.variation_id.as_deref())
            .and_then(|id| labels.get(id).map(|l| (*l).to_string())),
        current_planned_duration_secs: current.planned_duration_secs,
        next_item_title: active
            .entries
            .get(safe_index + 1)
            .map(|e| e.item_title.clone()),
        current_item_intention: current.intention.clone(),
        current_item_notes: item_index
            .get(current.item_id.as_str())
            .and_then(|i| i.notes.clone()),
        current_related_piece_title,
        current_item_tempo_marking: current_item_tempo.and_then(|t| t.marking.clone()),
        current_item_tempo_bpm: current_item_tempo.and_then(|t| t.bpm),
        current_item_metre: item_index
            .get(current.item_id.as_str())
            .and_then(|i| i.metre.clone()),
        current_variations: picker_variations(current, current_variations),
    }
}

fn picker_variations(entry: &SetlistEntry, variants: &[VariantView]) -> Vec<PickerVariationView> {
    let playing_now = entry.open_play().and_then(|p| p.variation_id.as_deref());
    variants
        .iter()
        .map(|v| {
            let played: Vec<&VariationPlay> = entry
                .plays
                .iter()
                .filter(|p| p.variation_id.as_deref() == Some(v.id.as_str()) && !p.is_incidental())
                .collect();
            let caption = if playing_now == Some(v.id.as_str()) {
                "Playing now".to_string()
            } else if !played.is_empty() {
                let played_secs: u64 = played.iter().map(|p| p.seconds).sum();
                format!(
                    "Played this session · {}",
                    crate::domain::session::format_duration_display(played_secs)
                )
            } else {
                saved_mark_caption(v.latest_score, v.is_solid)
            };
            PickerVariationView {
                id: v.id.clone(),
                label: v.label.clone(),
                caption,
                is_solid: v.is_solid,
            }
        })
        .collect()
}

pub fn build_summary_view(summary: &SummarySession, labels: &VariationLabels) -> SummaryView {
    let total_secs: u64 = summary.entries.iter().map(|e| e.duration_secs).sum();
    SummaryView {
        total_duration_display: crate::domain::session::format_duration_display(total_secs),
        completion_status: summary.completion_status.clone(),
        notes: summary.session_notes.clone(),
        entries: summary
            .entries
            .iter()
            .map(|e| entry_to_view(e, labels))
            .collect(),
        session_score: summary.session_score,
    }
}

pub fn session_to_view(session: &PracticeSession, labels: &VariationLabels) -> PracticeSessionView {
    PracticeSessionView {
        id: session.id.clone(),
        started_at: session.started_at.to_rfc3339(),
        finished_at: session.completed_at.to_rfc3339(),
        total_duration_display: crate::domain::session::format_duration_display(
            session.total_duration_secs,
        ),
        total_duration_summary: crate::domain::session::format_duration_summary(
            session.total_duration_secs,
        ),
        completion_status: session.completion_status.clone(),
        notes: session.session_notes.clone(),
        played_summary: format_played_summary(&session.entries, labels),
        session_score: session.session_score,
        entries: session
            .entries
            .iter()
            .map(|e| entry_to_view(e, labels))
            .collect(),
    }
}

/// The line's character budget before it falls back to "and N more" (#1785):
/// the length of the mocked cut-off example, "Major scales in 7 keys · Hanon
/// No. 1 · Nocturne in E♭ and 2 more". A character count, not a measured
/// width, so it is a proxy for what fits on the card rather than a guarantee.
const PLAYED_SUMMARY_MAX_CHARS: usize = 64;

/// The most variation labels an entry names in full before collapsing to a
/// count ("Major scales in 7 keys"), as mocked for #1785.
const PLAYED_SUMMARY_SPELL_OUT_LIMIT: usize = 3;

/// The card's "what was played" line (#1785): every completed entry with
/// something genuinely practised on it, pieces named plainly and an
/// exercise's variations named when there are few, cut off with "and N more"
/// rather than a mid-word ellipsis when the whole line still will not fit.
fn format_played_summary(entries: &[SetlistEntry], labels: &VariationLabels) -> String {
    let fragments: Vec<String> = entries
        .iter()
        .filter(|entry| entry.status == EntryStatus::Completed)
        .filter_map(|entry| entry_played_fragment(entry, labels))
        .collect();
    join_played_fragments(&fragments, PLAYED_SUMMARY_MAX_CHARS)
}

/// `None` when every play on the entry is incidental (#1758): a stray tap
/// that survived only because an entry always keeps at least one play must
/// not read as something the musician set out to practise.
fn entry_played_fragment(entry: &SetlistEntry, labels: &VariationLabels) -> Option<String> {
    let played: Vec<&VariationPlay> = entry.plays.iter().filter(|p| !p.is_incidental()).collect();
    if played.is_empty() {
        return None;
    }
    let variation_labels = ordered_distinct_variation_labels(&played, labels);
    Some(match variation_labels.len() {
        0 => entry.item_title.clone(),
        n if n <= PLAYED_SUMMARY_SPELL_OUT_LIMIT => {
            format!(
                "{} in {}",
                entry.item_title,
                join_with_and(&variation_labels)
            )
        }
        n => {
            let noun = if ladder_is_all_keys(variation_labels.iter().map(String::as_str)) {
                "keys"
            } else {
                "variations"
            };
            format!("{} in {n} {noun}", entry.item_title)
        }
    })
}

/// A play's variation, first-seen order, deduplicated: switching back to a
/// key already played (rare, but possible) must not repeat it in the line.
fn ordered_distinct_variation_labels(
    plays: &[&VariationPlay],
    labels: &VariationLabels,
) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    plays
        .iter()
        .filter_map(|play| play.variation_id.as_deref().and_then(|id| labels.get(id)))
        .filter(|label| seen.insert(*label))
        .map(|label| (*label).to_string())
        .collect()
}

/// Plain English list join: "C", "C and G", "C, G and D", no Oxford comma,
/// matching how a musician would say the list aloud.
fn join_with_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// Joins fragments with " · ", dropping trailing ones and appending
/// "· and N more" until what remains fits `max_chars`: never a mid-word
/// ellipsis, and never a silently dropped count, so the line never implies
/// less was played than the truth.
fn join_played_fragments(fragments: &[String], max_chars: usize) -> String {
    let Some(first) = fragments.first() else {
        return String::new();
    };
    let full = fragments.join(" · ");
    if full.chars().count() <= max_chars {
        return full;
    }
    for shown in (1..fragments.len()).rev() {
        let candidate = fragments[..shown].join(" · ");
        let more = fragments.len() - shown;
        let with_suffix = format!("{candidate} · and {more} more");
        if with_suffix.chars().count() <= max_chars {
            return with_suffix;
        }
    }
    if fragments.len() > 1 {
        format!("{first} · and {} more", fragments.len() - 1)
    } else {
        first.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ── Projected limits (#1512) ──

    #[test]
    fn offered_metre_range_is_what_validate_metre_accepts() {
        let limits = LimitsView::default();

        for beats in limits.metre_beats_min..=limits.metre_beats_max {
            for unit in &limits.metre_units {
                let metre = Metre {
                    beats,
                    unit: *unit,
                    groups: None,
                };
                assert!(
                    crate::validation::validate_metre(&metre).is_ok(),
                    "the sheet offers {beats}/{unit}, which the core refuses"
                );
            }
        }

        for beats in [limits.metre_beats_min - 1, limits.metre_beats_max + 1] {
            let metre = Metre {
                beats,
                unit: limits.metre_units[0],
                groups: None,
            };
            assert!(
                crate::validation::validate_metre(&metre).is_err(),
                "{beats} beats is outside the offered range but the core accepts it"
            );
        }
    }

    #[test]
    fn offered_rep_targets_are_what_validate_rep_target_accepts() {
        let limits = LimitsView::default();

        for target in limits.rep_target_min..=limits.rep_target_max {
            assert!(
                crate::validation::validate_rep_target(&Some(target)).is_ok(),
                "the sheet offers {target} reps, which the core refuses"
            );
        }

        for target in [limits.rep_target_min - 1, limits.rep_target_max + 1] {
            assert!(
                crate::validation::validate_rep_target(&Some(target)).is_err(),
                "{target} reps is outside the offered range but the core accepts it"
            );
        }

        assert!(
            (limits.rep_target_min..=limits.rep_target_max).contains(&limits.rep_target_default),
            "the default the sheet opens on sits outside the range it offers"
        );
    }

    #[test]
    fn offered_planned_durations_are_whole_minutes_the_core_accepts() {
        let limits = LimitsView::default();

        for secs in [
            limits.planned_duration_min_secs,
            limits.planned_duration_max_secs,
        ] {
            assert!(
                crate::validation::validate_planned_duration(&Some(secs)).is_ok(),
                "the sheet offers {secs}s, which the core refuses"
            );
            assert_eq!(
                secs % 60,
                0,
                "the sheet steps in whole minutes, so {secs}s cannot be offered exactly"
            );
        }

        for secs in [
            limits.planned_duration_min_secs - 60,
            limits.planned_duration_max_secs + 60,
        ] {
            assert!(
                crate::validation::validate_planned_duration(&Some(secs)).is_err(),
                "{secs}s is outside the offered range but the core accepts it"
            );
        }
    }

    #[test]
    fn limits_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(LimitsView::default());
    }

    /// `ExerciseUsageView` crosses the bincode FFI bridge inside the
    /// ViewModel; guard it against the #846 silent-drop class.
    #[test]
    fn exercise_usage_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(ExerciseUsageView {
            piece: Some(PieceRefView {
                id: "P".to_string(),
                title: "Sonata".to_string(),
                subtitle: Some("Beethoven".to_string()),
            }),
            linked: true,
            latest_score: Some(6),
            session_count: 2,
            last_practiced_at: Some("2026-07-01T00:00:00+00:00".to_string()),
            piece_removed: true,
        });
    }

    /// `ScaffoldPreviewView` crosses the bincode wire inside the ViewModel
    /// (`LibraryItemView.scaffold_preview`) — guard it against the #846 drop class.
    #[test]
    fn scaffold_preview_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(ScaffoldPreviewView {
            key: "G".to_string(),
            specs: vec![ScaffoldSpecView {
                kind: ScaffoldKind::Shells,
                title: "Shells".to_string(),
                rationale: "3rd + 7th of every chord".to_string(),
                key: "G".to_string(),
                fallback: false,
                already_linked: true,
            }],
            fallback_total: 0,
        });
    }

    /// `VariantView` crosses the bincode wire inside `LibraryItemView.variants`
    /// (#1083); guard it against the #846 drop class.
    #[test]
    fn variant_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(VariantView {
            id: "v-1".to_string(),
            label: "B♭".to_string(),
            position: 2,
            latest_score: Some(8),
            score_history: vec![ScoreHistoryEntry {
                session_date: "2026-07-01T00:00:00+00:00".to_string(),
                score: 8,
                session_id: "s1".to_string(),
            }],
            is_solid: true,
            caption: "Solid · 8 of 10".to_string(),
        });
    }

    /// `LinkedExerciseView` crosses the same bincode wire; `piece_context_score`
    /// (#1087 B2) is a trailing `Option` — guard it against the #846 drop class.
    #[test]
    fn linked_exercise_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(LinkedExerciseView {
            id: "E".to_string(),
            title: "Enclosures".to_string(),
            key: Some("C".to_string()),
            tempo: Some("♩ = 120".to_string()),
            practice: None,
            piece_context_score: Some(7),
        });
    }

    fn make_item(id: &str, title: &str, kind: ItemKind) -> Item {
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        }
    }

    fn make_entry(id: &str, item_id: &str, title: &str, position: usize) -> SetlistEntry {
        SetlistEntry {
            id: id.to_string(),
            item_id: item_id.to_string(),
            item_title: title.to_string(),
            item_type: ItemKind::Piece,
            position,
            ..SetlistEntry::fixture()
        }
    }

    // ── Model error methods ────────────────────────────────────────────

    #[test]
    fn surface_error_sets_last_error() {
        let mut model = Model::default();
        model.surface_error("network down");
        assert_eq!(model.last_error.as_deref(), Some("network down"));
    }

    #[test]
    fn surface_error_dedupes_identical_messages() {
        let mut model = Model::default();
        model.surface_error("fail");
        model.surface_error("fail");
        assert_eq!(model.last_error.as_deref(), Some("fail"));
        assert_eq!(model.error_seq, 2, "a repeat is still a new failure");
    }

    #[test]
    fn raise_notice_is_a_new_raise_even_with_identical_text() {
        let mut model = Model::default();
        model.raise_notice("kept, but");
        model.raise_notice("kept, but");
        assert_eq!(model.last_notice.as_deref(), Some("kept, but"));
        assert_eq!(model.notice_seq, 2);
        assert_eq!(model.error_seq, 0, "a notice is not a refusal");
    }

    #[test]
    fn a_dismissed_error_does_not_mute_a_notice() {
        let mut model = Model::default();
        model.dismiss_error();
        model.raise_notice("kept, but");
        assert_eq!(model.last_notice.as_deref(), Some("kept, but"));
    }

    #[test]
    fn surface_error_muted_after_dismiss() {
        let mut model = Model::default();
        model.surface_error("first");
        model.dismiss_error();
        model.surface_error("second");
        assert!(model.last_error.is_none());
        assert!(model.error_muted);
    }

    #[test]
    fn surface_error_replaces_an_existing_distinct_message() {
        // A failed save must reach the banner even while a stale load failure
        // is still showing, or the user gets no feedback at all (#346).
        let mut model = Model::default();
        model.surface_error("Couldn't load your library.");
        model.surface_error("Couldn't save that piece.");
        assert_eq!(
            model.last_error.as_deref(),
            Some("Couldn't save that piece.")
        );
    }

    #[test]
    fn a_burst_of_distinct_failures_after_dismiss_stays_muted() {
        // The reproduction in #346: dismiss, then several different failures
        // against a still-broken store, none of which re-pops the banner.
        let mut model = Model::default();
        model.surface_error("Couldn't load your library.");
        model.dismiss_error();

        for msg in [
            "Couldn't access local storage.",
            "Couldn't load your library.",
            "Couldn't save that piece.",
        ] {
            model.surface_error(msg);
            assert!(model.last_error.is_none(), "still muted after: {msg}");
            assert!(model.error_muted, "the mute persists across the burst");
        }
    }

    #[test]
    fn record_ack_unmutes() {
        let mut model = Model::default();
        model.surface_error("oops");
        model.dismiss_error();
        model.record_ack();
        assert!(!model.error_muted);
        model.surface_error("new error");
        assert_eq!(model.last_error.as_deref(), Some("new error"));
    }

    #[test]
    fn clear_error_keeps_the_mute() {
        let mut model = Model::default();
        model.dismiss_error();
        model.clear_error();
        model.surface_error("still broken");
        assert!(model.last_error.is_none());
    }

    #[test]
    fn raise_error_ignores_the_mute_and_bumps_the_sequence() {
        let mut model = Model::default();
        model.dismiss_error();
        model.raise_error("Title is required");
        model.raise_error("Title is required");
        assert_eq!(model.last_error.as_deref(), Some("Title is required"));
        assert_eq!(model.error_seq, 2);
    }

    // ── entry_to_view ──────────────────────────────────────────────────

    #[test]
    fn entry_to_view_formats_duration() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.duration_secs = 125;
        let view = entry_to_view(&entry, &VariationLabels::new());
        assert_eq!(view.duration_display, "2m 5s");
    }

    #[test]
    fn entry_to_view_planned_duration_whole_minutes() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.planned_duration_secs = Some(300);
        let view = entry_to_view(&entry, &VariationLabels::new());
        assert_eq!(view.planned_duration_display.as_deref(), Some("5 min"));
    }

    #[test]
    fn entry_to_view_planned_duration_partial_minutes() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.planned_duration_secs = Some(90);
        let view = entry_to_view(&entry, &VariationLabels::new());
        assert_eq!(view.planned_duration_display.as_deref(), Some("1m 30s"));
    }

    #[test]
    fn entry_to_view_carries_planned_variation_id() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.planned_variation_id = Some("variant-1".to_string());
        let view = entry_to_view(&entry, &VariationLabels::new());
        assert_eq!(view.planned_variation_id.as_deref(), Some("variant-1"));
    }

    // ── build_active_session_view ──────────────────────────────────────

    /// The click sheet opens with the piece's metre (T19); a dropped projection
    /// would open every sheet at 4/4.
    #[test]
    fn active_session_view_carries_the_current_items_metre() {
        let metre = Metre {
            beats: 7,
            unit: 8,
            groups: Some(vec![3, 2, 2]),
        };
        let mut piece = make_item("i1", "Take Five", ItemKind::Piece);
        piece.metre = Some(metre.clone());
        let plain = make_item("i2", "Etude", ItemKind::Exercise);
        let items: HashMap<&str, &Item> = [("i1", &piece), ("i2", &plain)].into_iter().collect();
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![
                make_entry("e1", "i1", "Take Five", 0),
                make_entry("e2", "i2", "Etude", 1),
            ],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        assert_eq!(
            build_active_session_view(&active, &items, &VariationLabels::new(), &[])
                .current_item_metre,
            Some(metre)
        );
        let second = ActiveSession {
            current_index: 1,
            ..active
        };
        assert_eq!(
            build_active_session_view(&second, &items, &VariationLabels::new(), &[])
                .current_item_metre,
            None
        );
    }

    /// The resident counter draws against `current_rep_slots`, so a builder
    /// target of 7 must not render as ten slots.
    #[test]
    fn active_session_view_draws_the_builder_target_or_the_default() {
        let mut targeted = make_entry("e1", "i1", "Scale", 0);
        targeted.planned_rep_target = Some(7);
        targeted.plays = vec![VariationPlay {
            rep_target: Some(7),
            ..VariationPlay::fixture()
        }];
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![targeted, make_entry("e2", "i2", "Etude", 1)],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        assert_eq!(
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[])
                .current_rep_slots,
            7
        );

        let untouched = ActiveSession {
            current_index: 1,
            ..active
        };
        let view =
            build_active_session_view(&untouched, &HashMap::new(), &VariationLabels::new(), &[]);
        assert_eq!(view.current_rep_target, None);
        assert_eq!(
            view.current_rep_slots,
            crate::validation::DEFAULT_REP_TARGET
        );
    }

    #[test]
    fn active_session_view_next_item_title() {
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![
                make_entry("e1", "i1", "Scale", 0),
                make_entry("e2", "i2", "Etude", 1),
            ],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert_eq!(view.next_item_title.as_deref(), Some("Etude"));
    }

    #[test]
    fn active_session_view_last_item_has_no_next() {
        let active = ActiveSession {
            id: "as2".to_string(),
            entries: vec![
                make_entry("e1", "i1", "Scale", 0),
                make_entry("e2", "i2", "Etude", 1),
            ],
            current_index: 1,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert!(view.next_item_title.is_none());
    }

    #[test]
    fn active_session_view_promotes_current_item_intention() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.intention = Some("evenness".to_string());
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![entry],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert_eq!(view.current_item_intention.as_deref(), Some("evenness"));
    }

    #[test]
    fn active_session_view_related_piece_title_for_grouped_exercise() {
        let mut exercise = make_entry("e1", "i1", "Scale", 0);
        exercise.item_type = ItemKind::Exercise;
        exercise.group_id = Some("g1".to_string());
        let mut piece = make_entry("e2", "i2", "Clair de Lune", 1);
        piece.item_type = ItemKind::Piece;
        piece.group_id = Some("g1".to_string());
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![exercise, piece],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert_eq!(
            view.current_related_piece_title.as_deref(),
            Some("Clair de Lune")
        );
    }

    #[test]
    fn active_session_view_no_related_piece_title_for_standalone_entry() {
        // Explicitly Exercise + no group_id: distinct from the anchor-piece
        // test below, which covers item_type == Piece separately. make_entry's
        // default item_type is Piece, so this must override it to actually
        // exercise the "ungrouped exercise" branch rather than short-circuiting
        // on the item_type check alone.
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.item_type = ItemKind::Exercise;
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![entry],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert!(view.current_related_piece_title.is_none());
    }

    #[test]
    fn active_session_view_no_related_piece_title_when_current_is_the_anchor_piece() {
        let mut piece = make_entry("e1", "i1", "Clair de Lune", 0);
        piece.item_type = ItemKind::Piece;
        piece.group_id = Some("g1".to_string());
        let mut exercise = make_entry("e2", "i2", "Scale", 1);
        exercise.item_type = ItemKind::Exercise;
        exercise.group_id = Some("g1".to_string());
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![piece, exercise],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert!(
            view.current_related_piece_title.is_none(),
            "breadcrumb is for exercises related to a piece, not the piece itself"
        );
    }

    #[test]
    fn active_session_view_current_item_tempo_from_item_library() {
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![make_entry("e1", "i1", "Scale", 0)],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let item = Item {
            id: "i1".to_string(),
            title: "Scale".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            modality: None,
            tempo: Some(crate::domain::types::Tempo {
                marking: Some("Allegro".to_string()),
                bpm: Some(132),
            }),
            notes: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        let item_index: HashMap<&str, &Item> = HashMap::from([("i1", &item)]);
        let view = build_active_session_view(&active, &item_index, &VariationLabels::new(), &[]);
        assert_eq!(view.current_item_tempo_marking.as_deref(), Some("Allegro"));
        assert_eq!(view.current_item_tempo_bpm, Some(132));
    }

    #[test]
    fn active_session_view_current_item_tempo_none_when_item_missing() {
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![make_entry("e1", "i1", "Scale", 0)],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &[]);
        assert!(view.current_item_tempo_marking.is_none());
        assert!(view.current_item_tempo_bpm.is_none());
    }

    #[test]
    fn active_session_view_current_item_notes_from_item_library() {
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![
                make_entry("e1", "i1", "Scale", 0),
                make_entry("e2", "i2", "Etude", 1),
            ],
            current_index: 1,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let item1 = Item {
            notes: Some("Watch the thumb crossing".to_string()),
            ..make_item("i1", "Scale", ItemKind::Exercise)
        };
        let item2 = Item {
            notes: Some("Keep the bow arm relaxed".to_string()),
            ..make_item("i2", "Etude", ItemKind::Exercise)
        };
        let item_index: HashMap<&str, &Item> = HashMap::from([("i1", &item1), ("i2", &item2)]);
        let view = build_active_session_view(&active, &item_index, &VariationLabels::new(), &[]);
        assert_eq!(
            view.current_item_notes.as_deref(),
            Some("Keep the bow arm relaxed")
        );
    }

    #[test]
    fn active_session_view_current_item_notes_none_when_item_has_no_notes() {
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![make_entry("e1", "i1", "Scale", 0)],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        };
        let item = make_item("i1", "Scale", ItemKind::Exercise);
        let item_index: HashMap<&str, &Item> = HashMap::from([("i1", &item)]);
        let view = build_active_session_view(&active, &item_index, &VariationLabels::new(), &[]);
        assert!(view.current_item_notes.is_none());
    }

    // ── picker captions (#1784) ────────────────────────────────────────

    fn session_on(plays: Vec<VariationPlay>) -> ActiveSession {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.plays = plays;
        ActiveSession {
            id: "as1".to_string(),
            entries: vec![entry],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
        }
    }

    fn play_on(id: &str, variation: &str, seconds: u64) -> VariationPlay {
        VariationPlay {
            id: id.to_string(),
            variation_id: Some(variation.to_string()),
            seconds,
            ..VariationPlay::fixture()
        }
    }

    fn captions(view: &ActiveSessionView) -> Vec<(&str, &str)> {
        view.current_variations
            .iter()
            .map(|v| (v.id.as_str(), v.caption.as_str()))
            .collect()
    }

    /// Practise C, switch to G, open the picker: C must say it was played
    /// this session, not "Not yet played" off the saved mark (#1784).
    #[test]
    fn picker_caption_reads_a_variation_played_earlier_in_the_item() {
        let active = session_on(vec![play_on("p1", "c", 250), play_on("p2", "g", 0)]);
        let variants = [
            VariantView::fixture("c", "C", 0),
            VariantView::fixture("g", "G", 1),
        ];
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &variants);
        assert_eq!(
            captions(&view),
            vec![("c", "Played this session · 4m 10s"), ("g", "Playing now")]
        );
    }

    /// Time on a variation adds up across every visit to it, not the latest.
    #[test]
    fn picker_caption_adds_up_every_play_on_a_variation() {
        let active = session_on(vec![
            play_on("p1", "c", 100),
            play_on("p2", "g", 30),
            play_on("p3", "c", 200),
            play_on("p4", "d", 0),
        ]);
        let variants = [
            VariantView::fixture("c", "C", 0),
            VariantView::fixture("g", "G", 1),
            VariantView::fixture("d", "D", 2),
        ];
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &variants);
        assert_eq!(
            captions(&view),
            vec![
                ("c", "Played this session · 5m 0s"),
                ("g", "Played this session · 30s"),
                ("d", "Playing now")
            ]
        );
    }

    /// A stray tap on the picker is dropped at the terminal transition, so the
    /// caption must not count it either.
    #[test]
    fn picker_caption_ignores_a_stray_tap() {
        let active = session_on(vec![play_on("p1", "c", 3), play_on("p2", "g", 0)]);
        let variants = [
            VariantView::fixture("c", "C", 0),
            VariantView::fixture("g", "G", 1),
        ];
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &variants);
        assert_eq!(captions(&view)[0], ("c", "Not yet played"));
    }

    #[test]
    fn picker_caption_falls_back_to_the_saved_mark_or_not_yet_played() {
        let active = session_on(vec![VariationPlay::fixture()]);
        let variants = [
            VariantView::fixture("c", "C", 0).scored(8),
            VariantView::fixture("d", "D", 1).scored(5),
            VariantView::fixture("e", "E", 2),
        ];
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &variants);
        assert_eq!(
            captions(&view),
            vec![
                ("c", "Solid · 8 of 10"),
                ("d", "5 of 10"),
                ("e", "Not yet played")
            ]
        );
        let solid: Vec<bool> = view.current_variations.iter().map(|v| v.is_solid).collect();
        assert_eq!(solid, vec![true, false, false]);
    }

    /// A mark taken the instant a switch opens the next play stamps at 0
    /// seconds. Summing seconds before checking for a play must not send that
    /// past this session's record and onto the saved mark (#1784).
    #[test]
    fn picker_caption_reads_a_zero_second_scored_play_as_played_this_session() {
        let scored_at_switch = VariationPlay {
            score: Some(7),
            ..play_on("p1", "c", 0)
        };
        let active = session_on(vec![scored_at_switch, play_on("p2", "g", 0)]);
        let variants = [
            VariantView::fixture("c", "C", 0).scored(9),
            VariantView::fixture("g", "G", 1),
        ];
        let view =
            build_active_session_view(&active, &HashMap::new(), &VariationLabels::new(), &variants);
        assert_eq!(captions(&view)[0], ("c", "Played this session · 0s"));
    }

    /// `ActiveSessionView` crosses the bincode wire; `current_variations` is
    /// its trailing field (#1784), so guard it against the #846 drop class.
    #[test]
    fn active_session_view_round_trips_on_ffi_bincode_wire() {
        let active = session_on(vec![play_on("p1", "c", 250), play_on("p2", "g", 0)]);
        let variants = [VariantView::fixture("c", "C", 0)];
        let item = Item {
            notes: Some("Watch the thumb crossing".to_string()),
            ..make_item("i1", "Scale", ItemKind::Exercise)
        };
        let item_index: HashMap<&str, &Item> = HashMap::from([("i1", &item)]);
        let view =
            build_active_session_view(&active, &item_index, &VariationLabels::new(), &variants);
        assert_eq!(view.current_variations.len(), 1);
        assert_eq!(
            view.current_item_notes.as_deref(),
            Some("Watch the thumb crossing")
        );
        crate::domain::types::assert_round_trips(view);
    }

    // ── build_summary_view ─────────────────────────────────────────────

    #[test]
    fn summary_view_total_duration() {
        let mut e1 = make_entry("e1", "i1", "Scale", 0);
        e1.duration_secs = 60;
        let mut e2 = make_entry("e2", "i2", "Etude", 1);
        e2.duration_secs = 90;
        let summary = crate::domain::session::SummarySession {
            id: "sum1".to_string(),
            entries: vec![e1, e2],
            session_started_at: Utc::now(),
            session_ended_at: Utc::now(),
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
        };
        let view = build_summary_view(&summary, &VariationLabels::new());
        assert_eq!(view.total_duration_display, "2m 30s");
    }

    #[test]
    fn session_view_exposes_coarse_duration_summary() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.duration_secs = 2700;
        let session = crate::domain::session::PracticeSession {
            id: "s1".to_string(),
            entries: vec![entry],
            session_notes: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            total_duration_secs: 2700,
            completion_status: CompletionStatus::Completed,
            session_score: None,
        };
        let view = session_to_view(&session, &VariationLabels::new());
        // Precise (live-timer) form keeps seconds; the summary line drops them.
        assert_eq!(view.total_duration_display, "45m 0s");
        assert_eq!(view.total_duration_summary, "45m");
    }

    #[test]
    fn summary_view_exposes_session_score() {
        let summary = crate::domain::session::SummarySession {
            id: "sum2".to_string(),
            entries: vec![],
            session_started_at: Utc::now(),
            session_ended_at: Utc::now(),
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: Some(7),
        };
        let view = build_summary_view(&summary, &VariationLabels::new());
        assert_eq!(view.session_score, Some(7));
    }

    #[test]
    fn session_to_view_exposes_session_score() {
        let session = crate::domain::session::PracticeSession {
            id: "s2".to_string(),
            entries: vec![],
            session_notes: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            total_duration_secs: 60,
            completion_status: CompletionStatus::Completed,
            session_score: Some(5),
        };
        let view = session_to_view(&session, &VariationLabels::new());
        assert_eq!(view.session_score, Some(5));
    }

    // ── Integration: Event → model → ViewModel ────────────────────────

    fn update_model(model: &mut Model, event: crate::app::Event) {
        use crux_core::App;
        let app = crate::app::Intrada;
        let _cmd = app.update(event, model);
    }

    fn model_with_summary_state() -> Model {
        use crate::app::Event;
        use crate::domain::session::SessionEvent;

        let now = Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "piece-1".to_string(),
                title: "Test Piece".to_string(),
                kind: ItemKind::Piece,
                composer: None,
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            }],
            ..Default::default()
        };

        update_model(&mut model, Event::Session(SessionEvent::StartBuilding));
        update_model(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update_model(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );
        let t1 = now + chrono::Duration::seconds(60);
        update_model(
            &mut model,
            Event::Session(SessionEvent::NextItem {
                now: t1,
                next_item_started_at: t1,
                reading: crate::domain::session::TempoReading::silent(),
            }),
        );
        model
    }

    #[test]
    fn update_session_score_flows_to_summary_view() {
        use crate::app::Event;
        use crate::domain::session::SessionEvent;
        use crux_core::App;

        let mut model = model_with_summary_state();

        update_model(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionScore { score: Some(7) }),
        );

        let app = crate::app::Intrada;
        let vm = app.view(&model);
        let summary = vm.summary.expect("model should be in Summary state");
        assert_eq!(summary.session_score, Some(7));
    }

    // ── Plays in the view (#1739) ──────────────────────────────────────

    /// `VariationPlayView` crosses the bincode FFI wire inside every entry
    /// view; guard it against the #846 silent-drop class.
    #[test]
    fn variation_play_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(VariationPlayView {
            id: "p1".to_string(),
            variation_id: Some("v-c".to_string()),
            variation_label: Some("C".to_string()),
            seconds: 180,
            duration_display: "3m 0s".to_string(),
            rep_target: Some(10),
            rep_count: Some(4),
            rep_target_reached: Some(false),
            rep_history: Some(vec![RepEvent {
                action: crate::domain::session::RepAction::Success,
                at: Utc::now(),
            }]),
            achieved_tempo: Some(84),
            click_pattern: Some(ClickState {
                metre: crate::domain::Metre {
                    beats: 7,
                    unit: 8,
                    groups: Some(vec![3, 2, 2]),
                },
                sounding: 0b0101001,
            }),
            tempo_display: Some(168),
            score: Some(6),
            is_markable: true,
        });
    }

    /// The sheet's stepper prefills from the stamp in the unit it was played
    /// in: a quaver click at 168 stores 84 and reads back as 168 (#1761).
    #[test]
    fn a_play_stamped_by_a_quaver_click_displays_its_tempo_in_quavers() {
        let play = VariationPlay {
            achieved_tempo: Some(84),
            click_pattern: Some(ClickState {
                metre: crate::domain::Metre {
                    beats: 6,
                    unit: 8,
                    groups: None,
                },
                sounding: 0b001001,
            }),
            ..VariationPlay::fixture()
        };
        let entry = SetlistEntry {
            plays: vec![play.clone()],
            ..SetlistEntry::fixture()
        };

        let view = play_to_view(&play, &entry, &VariationLabels::new());

        assert_eq!(view.achieved_tempo, Some(84));
        assert_eq!(view.tempo_display, Some(168));
    }

    #[test]
    fn a_tempo_with_no_click_displays_in_crotchets() {
        let play = VariationPlay {
            achieved_tempo: Some(96),
            ..VariationPlay::fixture()
        };
        let entry = SetlistEntry {
            plays: vec![play.clone()],
            ..SetlistEntry::fixture()
        };

        let view = play_to_view(&play, &entry, &VariationLabels::new());

        assert_eq!(view.tempo_display, Some(96));
    }

    #[test]
    fn an_unmeasured_play_has_no_tempo_to_display() {
        let play = VariationPlay::fixture();
        let entry = SetlistEntry {
            plays: vec![play.clone()],
            ..SetlistEntry::fixture()
        };

        let view = play_to_view(&play, &entry, &VariationLabels::new());

        assert_eq!(view.tempo_display, None);
    }

    /// A variation deleted since the session was practised still has to say
    /// what it was, which is what the tombstone is for.
    #[test]
    fn a_play_resolves_the_label_of_a_deleted_variation() {
        let labels: VariationLabels = [("v-gone", "E flat")].into_iter().collect();
        let play = VariationPlay {
            variation_id: Some("v-gone".to_string()),
            ..VariationPlay::fixture()
        };
        let entry = SetlistEntry {
            plays: vec![play.clone()],
            ..SetlistEntry::fixture()
        };

        let view = play_to_view(&play, &entry, &labels);

        assert_eq!(view.variation_label.as_deref(), Some("E flat"));
    }

    #[test]
    fn an_unattributed_play_has_no_label() {
        let labels = VariationLabels::new();
        let play = VariationPlay::fixture();
        let entry = SetlistEntry {
            plays: vec![play.clone()],
            ..SetlistEntry::fixture()
        };
        let view = play_to_view(&play, &entry, &labels);

        assert_eq!(view.variation_id, None);
        assert_eq!(view.variation_label, None);
    }

    #[test]
    fn a_stray_tap_views_as_not_markable() {
        let labels = VariationLabels::new();
        let opened = VariationPlay {
            id: "play-1".to_string(),
            seconds: 60,
            ..VariationPlay::fixture()
        };
        let stray_tap = VariationPlay {
            id: "play-2".to_string(),
            seconds: 2,
            ..VariationPlay::fixture()
        };
        let entry = SetlistEntry {
            plays: vec![opened.clone(), stray_tap.clone()],
            ..SetlistEntry::fixture()
        };

        assert!(play_to_view(&opened, &entry, &labels).is_markable);
        assert!(!play_to_view(&stray_tap, &entry, &labels).is_markable);
    }

    #[test]
    fn entry_to_view_carries_the_plays_and_their_mean() {
        let labels = VariationLabels::new();
        let entry = SetlistEntry {
            plays: vec![
                VariationPlay {
                    id: "p1".to_string(),
                    score: Some(8),
                    ..VariationPlay::fixture()
                },
                VariationPlay {
                    id: "p2".to_string(),
                    score: Some(5),
                    ..VariationPlay::fixture()
                },
            ],
            ..SetlistEntry::fixture()
        };

        let view = entry_to_view(&entry, &labels);

        assert_eq!(view.plays.len(), 2);
        assert_eq!(view.score_summary, Some(7));
    }

    // ── Played summary (#1785) ─────────────────────────────────────────

    /// A play on a named variation, otherwise identical to the fixture: long
    /// enough (#1785 uses the fixture's default 60 seconds) not to read as
    /// incidental.
    fn labelled_play(id: &str, variation_id: &str) -> VariationPlay {
        VariationPlay {
            id: id.to_string(),
            variation_id: Some(variation_id.to_string()),
            ..VariationPlay::fixture()
        }
    }

    #[test]
    fn played_summary_names_a_piece_plainly() {
        let entries = vec![SetlistEntry {
            item_title: "Nocturne in E\u{266d}".to_string(),
            item_type: ItemKind::Piece,
            status: EntryStatus::Completed,
            plays: vec![VariationPlay::fixture()],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &VariationLabels::new()),
            "Nocturne in E\u{266d}"
        );
    }

    #[test]
    fn played_summary_names_the_only_variation_played() {
        let labels: VariationLabels = [("v-eb", "E\u{266d} major")].into_iter().collect();
        let entries = vec![SetlistEntry {
            item_title: "Arpeggios".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays: vec![labelled_play("p1", "v-eb")],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Arpeggios in E\u{266d} major"
        );
    }

    #[test]
    fn played_summary_spells_out_a_few_variations() {
        let labels: VariationLabels = [("v-c", "C"), ("v-g", "G"), ("v-d", "D")]
            .into_iter()
            .collect();
        let entries = vec![SetlistEntry {
            item_title: "Major scales".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays: vec![
                labelled_play("p1", "v-c"),
                labelled_play("p2", "v-g"),
                labelled_play("p3", "v-d"),
            ],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Major scales in C, G and D"
        );
    }

    #[test]
    fn played_summary_collapses_many_keys_to_a_count() {
        let keys = ["C", "D", "E", "F", "G", "A", "B"];
        let labels: VariationLabels = keys.iter().map(|k| (*k, *k)).collect();
        let plays = keys
            .iter()
            .enumerate()
            .map(|(i, k)| labelled_play(&format!("p{i}"), k))
            .collect();
        let entries = vec![SetlistEntry {
            item_title: "Major scales".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays,
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Major scales in 7 keys"
        );
    }

    #[test]
    fn played_summary_collapses_many_non_key_variations_to_a_count() {
        let ids = ["v1", "v2", "v3", "v4"];
        let names = [
            "Root position",
            "1st inversion",
            "2nd inversion",
            "3rd inversion",
        ];
        let labels: VariationLabels = ids.into_iter().zip(names).collect();
        let plays = ids
            .iter()
            .enumerate()
            .map(|(i, id)| labelled_play(&format!("p{i}"), id))
            .collect();
        let entries = vec![SetlistEntry {
            item_title: "Arpeggios".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays,
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Arpeggios in 4 variations"
        );
    }

    #[test]
    fn played_summary_ignores_an_entry_that_was_never_attempted() {
        let entries = vec![
            SetlistEntry {
                item_title: "Skipped exercise".to_string(),
                item_type: ItemKind::Exercise,
                status: EntryStatus::NotAttempted,
                plays: vec![],
                ..SetlistEntry::fixture()
            },
            SetlistEntry {
                item_title: "Nocturne in E\u{266d}".to_string(),
                item_type: ItemKind::Piece,
                status: EntryStatus::Completed,
                plays: vec![VariationPlay::fixture()],
                ..SetlistEntry::fixture()
            },
        ];

        assert_eq!(
            format_played_summary(&entries, &VariationLabels::new()),
            "Nocturne in E\u{266d}"
        );
    }

    /// `SkipItem` can leave a play behind that banked a mark or reps before
    /// the skip (`domain::session`); the card still says "Skipped", not that
    /// it was played (#1785).
    #[test]
    fn played_summary_ignores_a_skipped_entry_even_with_a_surviving_play() {
        let entries = vec![SetlistEntry {
            item_title: "Hanon No. 1".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Skipped,
            plays: vec![VariationPlay {
                score: Some(6),
                ..VariationPlay::fixture()
            }],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(format_played_summary(&entries, &VariationLabels::new()), "");
    }

    /// A stray tap on the picker leaves a play behind because an entry always
    /// keeps at least one (#1739 decision 3), but it recorded nothing and ran
    /// under `MIN_PLAY_SECONDS`: the same incidental test #1758 already uses
    /// to keep it off the mark sheet keeps it off this line too.
    #[test]
    fn played_summary_ignores_a_stray_tap_with_nothing_recorded() {
        let entries = vec![SetlistEntry {
            item_title: "Major scales".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays: vec![VariationPlay {
                seconds: 2,
                score: None,
                rep_count: None,
                ..VariationPlay::fixture()
            }],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(format_played_summary(&entries, &VariationLabels::new()), "");
    }

    /// A play with no resolvable label (no variation, or one the label map
    /// does not carry) does not count toward "how many variations": the line
    /// names only what it can actually name (#1785).
    #[test]
    fn played_summary_counts_only_plays_with_a_resolvable_label() {
        let labels: VariationLabels = [("v-c", "C")].into_iter().collect();
        let entries = vec![SetlistEntry {
            item_title: "Major scales".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays: vec![
                labelled_play("p1", "v-c"),
                VariationPlay {
                    id: "p2".to_string(),
                    variation_id: None,
                    ..VariationPlay::fixture()
                },
            ],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Major scales in C"
        );
    }

    /// A stray tap can reopen a key already played this session; the line
    /// says "C and G", not "C, C and G" (#1785).
    #[test]
    fn played_summary_deduplicates_a_repeated_variation() {
        let labels: VariationLabels = [("v-c", "C"), ("v-g", "G")].into_iter().collect();
        let entries = vec![SetlistEntry {
            item_title: "Major scales".to_string(),
            item_type: ItemKind::Exercise,
            status: EntryStatus::Completed,
            plays: vec![
                labelled_play("p1", "v-c"),
                labelled_play("p2", "v-g"),
                labelled_play("p3", "v-c"),
            ],
            ..SetlistEntry::fixture()
        }];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Major scales in C and G"
        );
    }

    fn piece_entry(title: &str) -> SetlistEntry {
        SetlistEntry {
            item_title: title.to_string(),
            item_type: ItemKind::Piece,
            status: EntryStatus::Completed,
            plays: vec![VariationPlay::fixture()],
            ..SetlistEntry::fixture()
        }
    }

    #[test]
    fn played_summary_cuts_off_with_and_n_more_never_an_ellipsis() {
        let entries: Vec<SetlistEntry> = [
            "Nocturne in E flat major",
            "Prelude in C sharp minor",
            "Waltz in A flat major",
            "Impromptu in F minor",
            "Ballade in G minor",
        ]
        .into_iter()
        .map(piece_entry)
        .collect();

        let summary = format_played_summary(&entries, &VariationLabels::new());

        assert!(!summary.contains('\u{2026}'), "no ellipsis: {summary:?}");
        assert!(
            summary.contains("Nocturne in E flat major"),
            "keeps whole fragments: {summary:?}"
        );
        assert!(
            summary.ends_with("more"),
            "names how many were cut: {summary:?}"
        );
    }

    /// Regression for a blocker in review: when even the first fragment plus
    /// a plain count would not fit, the count must still appear rather than
    /// silently disappearing (#1785). A truncated line that looks complete
    /// is worse than one that runs long.
    #[test]
    fn played_summary_still_names_the_count_when_the_first_fragment_alone_is_too_long() {
        let entries = vec![
            piece_entry("Piano Sonata No. 14 in C\u{266f} minor, Op. 27 No. 2 \"Moonlight\""),
            piece_entry("Nocturne in E\u{266d}"),
        ];

        let summary = format_played_summary(&entries, &VariationLabels::new());

        assert!(
            summary.ends_with("and 1 more"),
            "names the count: {summary:?}"
        );
    }

    /// A fragment that already ends in "...and D" must not run into the
    /// overflow's own "and", or "C, G and D and 2 more" reads as more scales
    /// (#1785 review).
    #[test]
    fn played_summary_separates_the_overflow_from_a_fragments_own_and() {
        let labels: VariationLabels = [("v-c", "C"), ("v-g", "G"), ("v-d", "D")]
            .into_iter()
            .collect();
        let entries = vec![
            SetlistEntry {
                item_title: "Major scales".to_string(),
                item_type: ItemKind::Exercise,
                status: EntryStatus::Completed,
                plays: vec![
                    labelled_play("p1", "v-c"),
                    labelled_play("p2", "v-g"),
                    labelled_play("p3", "v-d"),
                ],
                ..SetlistEntry::fixture()
            },
            piece_entry("Nocturne in E flat major, Op. 9 No. 2"),
            piece_entry("Waltz"),
        ];

        assert_eq!(
            format_played_summary(&entries, &labels),
            "Major scales in C, G and D · and 2 more"
        );
    }

    #[test]
    fn played_summary_is_empty_when_nothing_was_played() {
        assert_eq!(format_played_summary(&[], &VariationLabels::new()), "");
    }

    #[test]
    fn join_with_and_reads_as_a_musician_would_say_it() {
        assert_eq!(join_with_and(&[]), "");
        assert_eq!(join_with_and(&["C".to_string()]), "C");
        assert_eq!(
            join_with_and(&["C".to_string(), "G".to_string()]),
            "C and G"
        );
        assert_eq!(
            join_with_and(&["C".to_string(), "G".to_string(), "D".to_string()]),
            "C, G and D"
        );
    }

    #[test]
    fn join_played_fragments_keeps_a_single_long_fragment_whole() {
        let fragments = vec!["A very long single fragment that will not fit".to_string()];

        assert_eq!(join_played_fragments(&fragments, 10), fragments[0]);
    }

    /// Companion to the `format_played_summary`-level regression above, pinned
    /// directly on the join so the fallback's shape stays covered even if the
    /// caller changes.
    #[test]
    fn join_played_fragments_names_the_count_when_the_first_fragment_alone_is_too_long() {
        let fragments = vec![
            "A very long first fragment that alone exceeds the budget".to_string(),
            "Short".to_string(),
        ];

        assert_eq!(
            join_played_fragments(&fragments, 10),
            "A very long first fragment that alone exceeds the budget · and 1 more"
        );
    }

    #[test]
    fn session_to_view_builds_the_played_summary_from_its_entries() {
        let session = crate::domain::session::PracticeSession {
            id: "s1".to_string(),
            entries: vec![SetlistEntry {
                item_title: "Nocturne in E\u{266d}".to_string(),
                item_type: ItemKind::Piece,
                status: EntryStatus::Completed,
                plays: vec![VariationPlay::fixture()],
                ..SetlistEntry::fixture()
            }],
            session_notes: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            total_duration_secs: 60,
            completion_status: CompletionStatus::Completed,
            session_score: None,
        };

        let view = session_to_view(&session, &VariationLabels::new());

        assert_eq!(view.played_summary, "Nocturne in E\u{266d}");
    }
}

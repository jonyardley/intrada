use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::analytics::{AnalyticsView, LastPractisedView, ScoreChange};
use crate::domain::chart::{ChordChart, ScaffoldKind};
use crate::domain::item::{Item, ItemKind, Modality};
use crate::domain::profile::{Profile, ProfileField, ProfileView};
use crate::domain::session::{
    ClickState, CompletionStatus, EntryStatus, PracticeSession, RepEvent, SessionStatus,
};
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
    pub items: Tracked<Vec<Item>>,
    pub sessions: Tracked<Vec<PracticeSession>>,
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
    pub practice_summaries: Tracked<HashMap<String, ItemPracticeSummary>>,
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
    /// Filled at the end of `update` and only read by `view` (#1998).
    pub(crate) projections: Option<crate::view::cache::Projections>,
    pub items_sync: crate::persistence::ListSync,
    pub sessions_sync: crate::persistence::ListSync,
}

static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

/// Every mutable access takes a new revision from one process-wide counter, so
/// no handler can change what the view cache read without it seeing (#1998).
#[derive(Debug)]
pub struct Tracked<T> {
    value: T,
    revision: u64,
}

impl<T> Tracked<T> {
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

impl<T> From<T> for Tracked<T> {
    fn from(value: T) -> Self {
        Tracked {
            value,
            revision: NEXT_REVISION.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl<T: Default> Default for Tracked<T> {
    fn default() -> Self {
        T::default().into()
    }
}

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Tracked<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.revision = NEXT_REVISION.fetch_add(1, Ordering::Relaxed);
        &mut self.value
    }
}

impl<'a, T> IntoIterator for &'a Tracked<Vec<T>> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.value.iter()
    }
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

    pub fn surface_storage_error(&mut self) {
        self.surface_error("Couldn't access local storage.");
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
    /// What the last photographed page was read into, for the confirm surface.
    pub photo_recognition: PhotoRecognitionView,
    pub limits: LimitsView,
    /// Ids of the rows `active_query` leaves showing, in sort order.
    pub visible_ids: Vec<String>,
    /// Up to 5 items with practice history, most recently practised first,
    /// for the builder's "Recently practised" quick-add section (#1362).
    pub recently_practised_ids: Vec<String>,
    /// "Practise your priorities" can show: something is starred and nothing
    /// is being built, played or summarised, so the tap cannot meet the
    /// core's "already in progress" refusal (#981). Derived before the Library
    /// filter, like `up_next`.
    pub shows_priorities: bool,
}

/// The bounds `validation.rs` enforces, projected so no sheet repeats them: a
/// repeated number keeps offering the old range once a bound moves, and the
/// refused write shows nothing on screen (#1512).
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
    /// A finished read whose draft holds no field at all.
    pub read_nothing: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PhotoRecognitionStatus {
    #[default]
    Idle,
    Reading,
    Ready,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LinkedExerciseView {
    pub id: String,
    pub title: String,
    pub key: Option<String>,
    pub modality: Option<Modality>,
    pub tempo_marking: Option<String>,
    pub tempo_bpm: Option<u16>,
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
    pub tempo_marking: Option<String>,
    pub tempo_bpm: Option<u16>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub practice: Option<ItemPracticeSummary>,
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
    pub solid_variation_count: usize,
    /// The wedge the key picker lights for the stored key (#2074).
    pub key_selection: Option<crate::domain::key::KeyWheelSelection>,
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
    /// Items in the session outside this block, which it cannot take (#2075).
    pub taken_elsewhere: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BuildingSetlistView {
    pub entries: Vec<SetlistEntryView>,
    pub item_count: usize,
    /// The same entries grouped into ordered units (blocks + standalone items).
    pub blocks: Vec<SetlistBlockView>,
    /// Sum of the entries' planned durations; `None` when nothing is planned
    /// so shells can fall back to counts-only copy.
    pub total_duration_display: Option<String>,
    pub total_duration_summary: Option<String>,
    /// The variations each entry can be tagged to, for the entries whose item
    /// has any. Looked up in the whole library, so a search cannot empty it.
    pub entry_variations: Vec<EntryVariationsView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct EntryVariationsView {
    pub entry_id: String,
    pub variations: Vec<PickerVariationView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SummaryView {
    pub total_duration_display: String,
    pub completion_status: CompletionStatus,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_score: Option<u8>,
    pub completed_count: usize,
    /// The largest rise this week among this session's items, by item id.
    pub top_mover: Option<ScoreChange>,
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
            tempo_marking: None,
            tempo_bpm: None,
            notes: None,
            tags: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
            practice: None,
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
            solid_variation_count: 0,
            key_selection: None,
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
            modality: None,
            tempo_marking: None,
            tempo_bpm: None,
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
            tempo_trend: TempoTrendView::default(),
            last_practiced_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        for beats in [
            limits.metre_beats_min.saturating_sub(1),
            limits.metre_beats_max + 1,
        ] {
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

        for unit in [1, 16] {
            assert!(
                !limits.metre_units.contains(&unit),
                "the test's unoffered beat value {unit} is now offered"
            );
            let metre = Metre {
                beats: limits.metre_beats_min,
                unit,
                groups: None,
            };
            assert!(
                crate::validation::validate_metre(&metre).is_err(),
                "the core accepts a {unit} beat value the sheet never offers"
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

        for target in [
            limits.rep_target_min.saturating_sub(1),
            limits.rep_target_max + 1,
        ] {
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
            limits.planned_duration_min_secs.saturating_sub(60),
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
            modality: Some(Modality::Minor),
            tempo_marking: Some("Allegro".to_string()),
            tempo_bpm: Some(132),
            practice: None,
            piece_context_score: Some(7),
        });
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
}

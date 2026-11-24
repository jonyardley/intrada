use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::analytics::{AnalyticsView, LastPractisedView, ScoreChange};
use crate::domain::first_run::{FirstRun, FirstRunView};
use crate::domain::item::{Item, ItemKind};
use crate::domain::key::Key;
use crate::domain::metre::{ClickBarOption, ClickPresetOption, TempoBand};
use crate::domain::practice_defaults::PracticeDefaults;
use crate::domain::profile::{Profile, ProfileView};
use crate::domain::session::{
    ClickState, CompletionStatus, EntryStatus, PracticeSession, ReflectionAnswers, SessionStatus,
    TempoReading,
};
use crate::domain::Metre;
use crate::domain::{LibrarySort, ListQuery};
use crate::recognition::PhotoDraft;
use crate::suggestion::SuggestedPlan;

mod add_form;
mod library;
mod practice_summary;

pub use add_form::{FormErrorField, FormErrorTarget, PhotoRecognitionStatus, PhotoRecognitionView};
pub(crate) use library::{saved_mark_caption, solid_caption, SOLID_MIN};
pub use library::{
    ExerciseUsageView, ItemKeyView, LibraryItemView, LinkedExerciseView, PieceRefView,
    ScaffoldPreviewView, ScaffoldSpecView, VariationOptionView, VariationView,
};
pub use practice_summary::{
    ItemPracticeSummary, ScoreHistoryEntry, ScoreTrend, TempoTrendPoint, TempoTrendView,
};

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
    /// The saved session the last delete took out, for its undo (#2581).
    pub deleted_session: Option<(PracticeSession, chrono::DateTime<chrono::Utc>)>,
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
    /// Device data, not account data (#1691).
    pub profile: Profile,
    /// Device data, like the profile (#1915).
    pub practice_defaults: PracticeDefaults,
    /// Device data, never synced (#2116).
    pub first_run: FirstRun,
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
    /// The library's variations, tombstones included (#2246).
    pub variations: Tracked<Vec<crate::domain::variation::Variation>>,
    pub variations_sync: crate::persistence::ListSync,
    pub sync: crate::sync::SyncState,
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

/// Serializable view state sent to shells for rendering.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ViewModel {
    /// Active filter, mirrored so the shell's pill reads one source of truth (#792).
    pub active_query: Option<ListQuery>,
    /// Active sort, mirrored so the shell's menu reads one source of truth.
    pub active_sort: LibrarySort,
    /// The Library header's line, "2 pieces · 1 exercise", worded for the
    /// visible set so both phones show the same text (#2454).
    pub library_count_line: String,
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
    /// Today's plan on the Practice tab (#1082, #57). `None` whenever
    /// nothing qualifies: it suggests, it never gates.
    pub up_next: Option<SuggestedPlan>,
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
    /// "Show suggestion" may show once the plan is put aside: a plan waits and
    /// nothing is being built, played or summarised (#2521).
    pub shows_suggestion_restore: bool,
    /// The crash-recovery prompt may show: no session is being played or
    /// summarised, so resuming cannot replace one (#962).
    pub offers_recovery: bool,
    pub practice_defaults: PracticeDefaults,
    pub first_run: FirstRunView,
    /// The library's live variations, for the pickers (#2246).
    pub variations: Vec<VariationOptionView>,
    /// The undo banner's text while the last saved-session delete can be
    /// taken back (#2581).
    pub undo_delete: Option<String>,
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
    pub planned_duration_min_secs: u32,
    pub planned_duration_max_secs: u32,
    pub planned_duration_default_secs: u32,
    pub session_length_min_mins: u16,
    pub session_length_max_mins: u16,
    pub session_length_step_mins: u16,
    pub session_length_default_mins: u16,
    pub score_min: u8,
    pub score_max: u8,
    pub click_tempo_step: u16,
    pub click_tempo_default: u16,
    /// One per beat unit, counted in that unit.
    pub click_tempo_bands: Vec<TempoBand>,
    pub click_metre_presets: Vec<Metre>,
    pub click_bars: Vec<ClickBarOption>,
    /// The planned-time stepper's range and starting point in whole minutes,
    /// the bounds in seconds rounded inwards (#2467).
    pub planned_duration_min_mins: u16,
    pub planned_duration_max_mins: u16,
    pub planned_duration_default_mins: u16,
}

/// Seconds to the stepper's whole minutes, the lower bound rounded up and the
/// upper one down. When no whole minute fits, both sit on the lower one rather
/// than invert, since a stepper over `2...1` traps (#2467).
fn whole_minutes(min_secs: u32, max_secs: u32, default_secs: u32) -> (u16, u16, u16) {
    let mins = |m: u32| u16::try_from(m).unwrap_or(u16::MAX);
    let min = mins(min_secs.div_ceil(60));
    let max = mins(max_secs / 60).max(min);
    (min, max, mins(default_secs / 60).clamp(min, max))
}

impl Default for LimitsView {
    fn default() -> Self {
        use crate::domain::metre;
        use crate::validation;

        let (planned_duration_min_mins, planned_duration_max_mins, planned_duration_default_mins) =
            whole_minutes(
                validation::MIN_PLANNED_DURATION_SECS,
                validation::MAX_PLANNED_DURATION_SECS,
                validation::DEFAULT_PLANNED_DURATION_SECS,
            );
        LimitsView {
            metre_beats_min: validation::MIN_METRE_BEATS,
            metre_beats_max: validation::MAX_METRE_BEATS,
            metre_units: validation::METRE_UNITS.to_vec(),
            planned_duration_min_secs: validation::MIN_PLANNED_DURATION_SECS,
            planned_duration_max_secs: validation::MAX_PLANNED_DURATION_SECS,
            planned_duration_default_secs: validation::DEFAULT_PLANNED_DURATION_SECS,
            session_length_min_mins: validation::MIN_SESSION_LENGTH_MINS,
            session_length_max_mins: validation::MAX_SESSION_LENGTH_MINS,
            session_length_step_mins: validation::SESSION_LENGTH_STEP_MINS,
            session_length_default_mins: validation::DEFAULT_SESSION_LENGTH_MINS,
            score_min: validation::MIN_SCORE,
            score_max: validation::MAX_SCORE,
            click_tempo_step: metre::CLICK_TEMPO_STEP,
            click_tempo_default: metre::CLICK_TEMPO_DEFAULT,
            click_tempo_bands: metre::click_tempo_bands(),
            click_metre_presets: metre::click_metre_presets(),
            click_bars: metre::click_bars(),
            planned_duration_min_mins,
            planned_duration_max_mins,
            planned_duration_default_mins,
        }
    }
}

/// One row of a variation picker, captioned by the core so a variation played
/// earlier in this item never reads as not yet played (#1784).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PickerVariationView {
    pub id: String,
    pub label: String,
    pub caption: String,
}

/// One stretch of an item played one way, as the sheet and the Progress
/// screen read it (#1739, #2246). Labels are resolved here, tombstones
/// included, because a play on a variation since deleted still has to say
/// what it was.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlayView {
    pub id: String,
    pub key: Option<Key>,
    pub variation_ids: Vec<String>,
    /// "E♭ major · Hands separately"; `None` for the item plain, in the
    /// written key.
    pub label: Option<String>,
    pub seconds: u64,
    pub duration_display: String,
    /// "4m 0s", "92 bpm": only what this play recorded, the duration always
    /// first, since rows that show it elsewhere drop it (#2515).
    pub meta_parts: Vec<String>,
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
pub struct PracticeSessionView {
    pub id: String,
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
    /// `Today`, `Yesterday` or `Sat 30 May`, on the week strip's clock (#2053).
    pub day_label: String,
    /// "3 pieces", "2 exercises", or "5 items" when the session spans both (#2515).
    pub item_count_display: String,
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
    /// False for a block's piece: removing it alone would dissolve the block,
    /// so the block's own remove offers that instead.
    pub removable: bool,
    /// What the builder planned (#2246): any variations.
    pub planned_variation_ids: Vec<String>,
    /// "Hands separately"; `None` with no plan.
    pub planned_label: Option<String>,
    /// What was actually practised, in order.
    pub plays: Vec<PlayView>,
    /// The mean of the plays that carry a mark, rounded to nearest, and `None`
    /// when none do. The only place several marks collapse into one (#1739
    /// decision 9): per-variation history reads `plays` and never this.
    pub score_summary: Option<u8>,
    pub planned_key: Option<crate::domain::key::Key>,
    /// The builder's tags under the row: the key, then the variations still
    /// on the item (#2467). Empty outside the builder.
    pub plan_tags: Vec<String>,
}

/// The one-tap offer of how the item was last played.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LastTimeView {
    pub entry_id: String,
    pub variation_ids: Vec<String>,
    /// "Last time: Dotted".
    pub label: String,
    pub key: Option<crate::domain::key::Key>,
}

/// One row of a key picker: `key: None` is the written key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct KeyChoiceView {
    pub key: Option<crate::domain::key::Key>,
    pub label: String,
    /// The written key's name under "Written key", when the item has one.
    pub caption: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct EntryKeysView {
    pub entry_id: String,
    pub keys: Vec<KeyChoiceView>,
    /// The planned key, spelt as its row in `keys`.
    pub current: Option<crate::domain::key::Key>,
    pub current_label: String,
}

/// What the practice screen offers on the current entry beyond the play.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ActiveRecordView {
    pub away_offer: Option<AwayOfferView>,
    /// `Some` while the item-complete sheet is open.
    pub finish: Option<FinishSheetView>,
    pub last_time: Option<LastTimeView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct AwayOfferView {
    pub minutes: u32,
    /// "Away 6 minutes. Leave it out?"
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FinishSheetView {
    /// One per play, in play order.
    pub rows: Vec<FinishRowView>,
    /// What each row's key can be changed to; the variations are
    /// `current_variations`. Empty when the written key is the only choice.
    pub keys: Vec<KeyChoiceView>,
}

/// What one finish row says was played: the drafted way, else the recorded one.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FinishRowView {
    pub play_id: String,
    pub label: String,
    /// Spelt as its row in the sheet's `keys`.
    pub key: Option<crate::domain::key::Key>,
    pub variation_ids: Vec<String>,
    pub can_change: bool,
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
    /// How the open play is being played, so the player can show it (#2246).
    pub current_key: Option<Key>,
    pub current_variation_ids: Vec<String>,
    /// The open play's label; `None` for the whole item, plain.
    pub current_play_label: Option<String>,
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
    pub current_item_tempo_line: Option<String>,
    pub current_item_tempo_line_spoken: Option<String>,
    /// The beats the click starts on for this item: the musician's default,
    /// fitted to the item's metre or 4/4 (#1915).
    pub current_click_sounding: u16,
    /// The current item's variations with the picker's caption for each (#1784).
    pub current_variations: Vec<PickerVariationView>,
    /// `Some` while the item-complete sheet is open, including after a resume (#2137).
    pub reflection: Option<ReflectionView>,
    /// The bar, tempo and target flag the click reseeds to on this item: 4/4
    /// when the item declares no metre, its BPM inside the band (#2225).
    pub click_seed_metre: Metre,
    pub click_seed_bpm: u16,
    /// False when the item declares no BPM, or one the band moved (#1942).
    pub click_seed_sounds_target: bool,
    /// The seed bar's patterns: an item may carry a valid grouping the
    /// sheet's `click_bars` table does not list.
    pub click_seed_presets: Vec<ClickPresetOption>,
    /// The pattern the item's own bar opens on when chosen again, since the table may not list it.
    pub click_seed_opening: u16,
    pub record: ActiveRecordView,
}

/// The open sheet's saved answers, and the click at its stamp, which seeds
/// the rows that have no stamp of their own.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ReflectionView {
    pub answers: ReflectionAnswers,
    pub reading: TempoReading,
    /// Measured against `current_item_started_at`, it stays the stamped seconds
    /// across a resume (#2297).
    pub stopped_at: String,
    /// One per markable play, in play order (#2230).
    pub tempos: Vec<ReflectionTempoView>,
    /// The item's plays summed, so a resume cannot skew it (#2352).
    pub elapsed_secs: u64,
}

/// A sheet row's tempo as it opens, counted in `click`'s unit (#2230).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ReflectionTempoView {
    pub play_id: String,
    pub tempo: u16,
    pub click: ClickState,
    pub band: TempoBand,
    pub set_by_hand: bool,
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
    /// The planned total ("10 min"); empty when nothing is planned.
    pub duration_display: String,
    pub entries: Vec<SetlistEntryView>,
    /// The anchor piece and its related exercises, in order; `None` and empty
    /// for a standalone item.
    pub piece: Option<SetlistEntryView>,
    pub related: Vec<SetlistEntryView>,
    /// Items in the session outside this block, which it cannot take (#2075).
    pub taken_elsewhere: Vec<String>,
    /// The row's second line (#2467): "Piece · 10 min" for a standalone item,
    /// "2 related, then piece · 10 min", "Piece only" or "10 min" for a block.
    pub caption: String,
    /// A collapsed block's line: the planned total, `None` when nothing is.
    pub planned_display: Option<String>,
    /// "+2 related", or "piece only".
    pub related_label: String,
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
    pub length_mins: Option<u16>,
    /// "20 of 30 min planned", or "30 min today" before anything is planned;
    /// `None` without a length (#1736).
    pub length_summary: Option<String>,
    /// The variations each entry can be tagged to, for the entries whose item
    /// has any. Looked up in the whole library, so a search cannot empty it.
    pub entry_variations: Vec<EntryVariationsView>,
    /// For the entries whose item was last played in a key or on variations.
    pub last_times: Vec<LastTimeView>,
    /// For the entries whose item keeps keys of its own.
    pub entry_keys: Vec<EntryKeysView>,
    /// "10 min · 2 blocks · 3 items": the time only once some is planned,
    /// the blocks only once there is one (#2467).
    pub summary: String,
    /// "Start session · 10 min", or "Start session".
    pub start_title: String,
}

/// What an entry's plan can name: its item's variations.
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
    fn offered_scores_are_what_validate_score_accepts() {
        let limits = LimitsView::default();

        for score in limits.score_min..=limits.score_max {
            assert!(
                crate::validation::validate_score(score).is_ok(),
                "the selector offers a mark of {score}, which the core refuses"
            );
        }

        for score in [limits.score_min.saturating_sub(1), limits.score_max + 1] {
            assert!(
                crate::validation::validate_score(score).is_err(),
                "a mark of {score} is outside the offered range but the core accepts it"
            );
        }
    }

    #[test]
    fn offered_session_lengths_are_ones_the_core_accepts() {
        let limits = LimitsView::default();
        let accepts = |m: u16| crate::validation::validate_session_length(&Some(m)).is_ok();

        for m in [
            limits.session_length_min_mins,
            limits.session_length_max_mins,
            limits.session_length_default_mins,
            limits.session_length_min_mins + limits.session_length_step_mins,
        ] {
            assert!(
                accepts(m),
                "the stepper offers {m} min, which the core refuses"
            );
        }
        for m in [
            limits.session_length_min_mins - limits.session_length_step_mins,
            limits.session_length_max_mins + limits.session_length_step_mins,
            limits.session_length_min_mins + 1,
        ] {
            assert!(
                !accepts(m),
                "{m} min is off the stepper but the core accepts it"
            );
        }
    }

    #[test]
    fn offered_planned_durations_are_whole_minutes_the_core_accepts() {
        let limits = LimitsView::default();

        for secs in [
            limits.planned_duration_min_secs,
            limits.planned_duration_max_secs,
            limits.planned_duration_default_secs,
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

    /// A bound in seconds rounds inwards, and a range no whole minute fits
    /// collapses onto its lower bound rather than inverting (#2467).
    #[test]
    fn planned_minutes_round_inwards_from_the_bounds_in_seconds() {
        let cases = [
            ((60, 7200, 600), (1, 120, 10)),
            ((90, 7230, 630), (2, 120, 10)),
            ((90, 110, 100), (2, 2, 2)),
            ((90, 600, 100), (2, 10, 2)),
            ((60, 600, 9000), (1, 10, 10)),
        ];
        for ((min, max, default), expected) in cases {
            assert_eq!(
                whole_minutes(min, max, default),
                expected,
                "{min}s to {max}s, {default}s"
            );
        }
    }

    #[test]
    fn the_sheet_offers_the_planned_bounds_in_minutes() {
        let limits = LimitsView::default();

        assert_eq!(
            u32::from(limits.planned_duration_min_mins) * 60,
            limits.planned_duration_min_secs
        );
        assert_eq!(
            u32::from(limits.planned_duration_max_mins) * 60,
            limits.planned_duration_max_secs
        );
        assert_eq!(
            u32::from(limits.planned_duration_default_mins) * 60,
            limits.planned_duration_default_secs
        );
    }

    #[test]
    fn limits_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(LimitsView::default());
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

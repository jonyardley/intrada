use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::first_run::FirstRun;
use crate::domain::item::Item;
use crate::domain::practice_defaults::PracticeDefaults;
use crate::domain::profile::Profile;
use crate::domain::session::{PracticeSession, SessionStatus};
use crate::domain::{LibrarySort, ListQuery};

mod active_session;
mod add_form;
mod builder;
mod library;
mod practice_summary;
mod sessions;
mod summary;
mod view_model;

pub use active_session::{
    ActiveRecordView, ActiveSessionView, AwayOfferView, FinishRowView, FinishSheetView,
    PickerVariationView, ReflectionTempoView, ReflectionView,
};
pub use add_form::{FormErrorField, FormErrorTarget};
pub use builder::{
    BuildingSetlistView, EntryKeysView, EntryVariationsView, FitOffer, FitState, FitView,
    KeyChoiceView, LastTimeView, SetlistBlockView,
};
pub(crate) use library::{saved_mark_caption, solid_caption, SOLID_MIN};
pub use library::{
    ExerciseUsageView, ItemKeyView, LibraryItemView, LinkedExerciseView, PieceRefView,
    ScaffoldPreviewView, ScaffoldSpecView, VariationOptionView, VariationView,
};
pub use practice_summary::{
    ItemPracticeSummary, ScoreHistoryEntry, ScoreTrend, TempoTrendPoint, TempoTrendView,
};
pub use sessions::{PlayView, PracticeSessionView, SetlistEntryView};
pub use summary::SummaryView;
pub use view_model::{LimitsView, ViewModel};

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
    /// The profile edit sheet's unsaved values; never stored (#2519).
    pub profile_draft: Option<Profile>,
    /// Device data, like the profile (#1915).
    pub practice_defaults: PracticeDefaults,
    /// The length picked on Practice; lasts until the app closes (#3007).
    pub plan_length_mins: Option<u16>,
    /// The keys ticked on today's plan; cleared by a planned start (#3008).
    pub todays_keys: crate::plan::TodaysKeys,
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

impl Model {
    pub(crate) fn plan_length(&self) -> u16 {
        crate::validation::clamp_plan_length(
            self.plan_length_mins
                .or(self.practice_defaults.session_length_mins)
                .unwrap_or(crate::validation::DEFAULT_SESSION_LENGTH_MINS),
        )
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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

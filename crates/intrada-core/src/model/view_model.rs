use serde::{Deserialize, Serialize};

use super::{
    ActiveSessionView, BuildingSetlistView, FormErrorTarget, SummaryView, VariationOptionView,
};
use crate::analytics::{AnalyticsView, LastPractisedView};
use crate::domain::first_run::FirstRunView;
use crate::domain::metre::{ClickBarOption, TempoBand};
use crate::domain::practice_defaults::PracticeDefaults;
use crate::domain::profile::ProfileView;
use crate::domain::Metre;
use crate::domain::{LibrarySort, ListQuery};

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
    /// Distinct tags across the whole library (case-folded dedupe, sorted):
    /// the vocabulary for the filter sheet and the add/edit autocomplete.
    pub available_tags: Vec<String>,
    /// Distinct composers (sorted) for the add/edit autocomplete. Computed
    /// pre-filter so it stays the full vocabulary when the list is narrowed.
    pub available_composers: Vec<String>,
    pub active_session: Option<ActiveSessionView>,
    pub building_setlist: Option<BuildingSetlistView>,
    pub summary: Option<SummaryView>,
    pub error: Option<String>,
    /// See [`Model::last_error_target`](super::Model::last_error_target).
    pub error_target: Option<FormErrorTarget>,
    /// See `Model::error_seq`; compare around a send instead of the message.
    pub error_seq: u64,
    /// See [`Model::last_notice`](super::Model::last_notice): true, expected, not a failure (#1325).
    pub notice: Option<String>,
    /// See [`Model::notice_seq`](super::Model::notice_seq).
    pub notice_seq: u64,
    pub analytics: Option<AnalyticsView>,
    pub last_practised: Option<LastPractisedView>,
    pub profile: ProfileView,
    pub limits: LimitsView,
    /// Ids of the rows `active_query` leaves showing, in sort order.
    pub visible_ids: Vec<String>,
    /// Up to 5 items with practice history, most recently practised first,
    /// for the builder's "Recently practised" quick-add section (#1362).
    pub recently_practised_ids: Vec<String>,
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
    /// Practice's plan from time, only while nothing is being built, played
    /// or summarised and the library has items (#3007).
    pub suggested_plan: Option<crate::plan::PlanView>,
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
    /// Counted in Unicode scalars, as the core counts (#3004).
    pub play_note_max_chars: u32,
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
            play_note_max_chars: u32::try_from(validation::MAX_PLAY_NOTE).unwrap_or(u32::MAX),
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
    fn the_limits_round_trip_on_the_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(LimitsView::default());
    }

    #[test]
    fn the_offered_key_note_length_is_what_validate_play_note_accepts() {
        let max = LimitsView::default().play_note_max_chars as usize;

        assert!(crate::validation::validate_play_note(&"é".repeat(max)).is_ok());
        assert!(crate::validation::validate_play_note(&"é".repeat(max + 1)).is_err());
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
}

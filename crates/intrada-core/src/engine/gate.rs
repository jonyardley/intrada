//! Gate criteria as data (engine spec §8): a countable requirement, the judge
//! allowed to unlock it, and the progress the dots draw. Every requirement here
//! is evaluatable by `GateProgress`, which is the rule the enum is held to — a
//! variant no reader can evaluate is a gate that silently never passes.
//!
//! What the loop cannot yet judge lives in the transport half of §8 rather than
//! here: the capability check and its `Ask` verdict wait for the scoring path,
//! since with no sensor every counted gate is the user's tap.

use serde::{Deserialize, Serialize};

/// One tap against a countable criterion (decision 18). The measured verdicts
/// of the deferred scoring path land in the same shape.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Verdict {
    Clean,
    Missed,
}

/// Where a verdict came from. Machine scoring returns as a higher-weight
/// evidence class rather than a migration (spec §2, amended 4 Aug 2026).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum EvidenceSource {
    TapVerdict,
    Midi,
    Audio,
}

/// Who decides a pass. `TapVerdict` is v1's default: user-judged like
/// `SelfConfirmed`, mastery-feeding like `Machine`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Judge {
    Machine,
    TapVerdict,
    SelfConfirmed,
}

/// The sparse-click ladder — gate levels *within* click-always (decision 2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ClickLevel {
    EveryBeat,
    TwoAndFour,
    BarDownbeat,
    EveryOtherBar,
}

impl ClickLevel {
    pub fn spoken(&self) -> &'static str {
        match self {
            ClickLevel::EveryBeat => "every beat",
            ClickLevel::TwoAndFour => "beats 2 & 4",
            ClickLevel::BarDownbeat => "beat 1 of each bar",
            ClickLevel::EveryOtherBar => "beat 1, every other bar",
        }
    }

    /// One cycle of the placement, from the pulse's first body beat: the beats
    /// that sound. A bar or two bars rather than the phrase, because placement
    /// is a bar-level fact and a phrase of an odd number of bars would
    /// otherwise flip `EveryOtherBar`'s alternation on every pass.
    pub fn pattern(&self, beats_per_bar: u8) -> Vec<bool> {
        let bar = usize::from(beats_per_bar.max(1));
        match self {
            ClickLevel::EveryBeat => vec![true; bar],
            ClickLevel::TwoAndFour => (0..bar).map(|beat| beat == 1 || beat == 3).collect(),
            ClickLevel::BarDownbeat => (0..bar).map(|beat| beat == 0).collect(),
            ClickLevel::EveryOtherBar => (0..bar * 2).map(|beat| beat == 0).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Requirement {
    CleanPasses {
        count: u8,
        consecutive: bool,
    },
    /// The same criterion in every key. Which key is in front of you is the
    /// dealer's business (the traversal quota, spec §9.5); the gate counts
    /// keys banked, so it needs no key identity to be evaluatable.
    KeyCoverage {
        keys_required: u8,
        per_key_passes: u8,
        first_attempt: bool,
    },
    /// One unbroken run through consecutive keys: a stop is what the gate is
    /// about, so a miss empties it.
    Chained {
        min_keys: u8,
    },
    /// The user is the sensor, and this kind of pass may never unlock a
    /// prerequisite (decisions 3 and 13). `max_listens` is shown, not counted:
    /// nothing in the app can see a listen.
    SelfConfirmed {
        max_listens: Option<u8>,
        first_attempt: bool,
    },
}

/// A gate's tunable half. `time_ceiling_s` is the block ceiling shown
/// throughout (decision 15), not a pass condition.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct GateCriteria {
    pub id: String,
    pub node: String,
    /// Prose, display only.
    pub criterion: String,
    pub requirement: Requirement,
    pub judge: Judge,
    pub time_ceiling_s: Option<u32>,
}

/// How far through a gate this block has got. `consecutive` is carried so a
/// miss knows whether to empty the dots or leave them.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct GateProgress {
    filled: u8,
    target: u8,
    consecutive: bool,
    /// Keys banked so far. Zero `keys_required` means the gate does not count
    /// keys at all, and `filled` against `target` is the whole story.
    keys_done: u8,
    keys_required: u8,
}

impl GateProgress {
    pub fn new(requirement: &Requirement) -> Self {
        let (target, consecutive, keys_required) = match *requirement {
            Requirement::CleanPasses { count, consecutive } => (count, consecutive, 0),
            Requirement::KeyCoverage {
                keys_required,
                per_key_passes,
                first_attempt,
            } => (per_key_passes, first_attempt, keys_required.max(1)),
            Requirement::Chained { min_keys } => (min_keys, true, 0),
            Requirement::SelfConfirmed { first_attempt, .. } => (1, first_attempt, 0),
        };
        Self {
            filled: 0,
            target: target.max(1),
            consecutive,
            keys_done: 0,
            keys_required,
        }
    }

    pub fn record(&mut self, verdict: Verdict) {
        match verdict {
            Verdict::Clean => {
                self.filled = (self.filled + 1).min(self.target);
                if self.filled == self.target && self.keys_done < self.keys_required {
                    self.keys_done += 1;
                    // The last key leaves its dots full, so the glance that
                    // opens the gate still reads.
                    if self.keys_done < self.keys_required {
                        self.filled = 0;
                    }
                }
            }
            Verdict::Missed if self.consecutive => self.filled = 0,
            Verdict::Missed => {}
        }
    }

    pub fn filled(&self) -> u8 {
        self.filled
    }

    pub fn target(&self) -> u8 {
        self.target
    }

    pub fn satisfied(&self) -> bool {
        if self.keys_required > 0 {
            self.keys_done >= self.keys_required
        } else {
            self.filled >= self.target
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_passes(count: u8, consecutive: bool) -> Requirement {
        Requirement::CleanPasses { count, consecutive }
    }

    #[test]
    fn counts_clean_taps_toward_the_target() {
        let mut progress = GateProgress::new(&clean_passes(3, false));
        assert_eq!(progress.target(), 3);
        assert_eq!(progress.filled(), 0);
        assert!(!progress.satisfied());

        progress.record(Verdict::Clean);
        progress.record(Verdict::Clean);
        assert_eq!(progress.filled(), 2);
        assert!(!progress.satisfied());

        progress.record(Verdict::Clean);
        assert!(progress.satisfied());
    }

    #[test]
    fn a_miss_holds_a_non_consecutive_gate_where_it_is() {
        let mut progress = GateProgress::new(&clean_passes(3, false));
        progress.record(Verdict::Clean);
        progress.record(Verdict::Missed);
        assert_eq!(progress.filled(), 1, "the pass already banked still counts");
    }

    #[test]
    fn a_miss_empties_a_consecutive_gate() {
        let mut progress = GateProgress::new(&clean_passes(3, true));
        progress.record(Verdict::Clean);
        progress.record(Verdict::Clean);
        progress.record(Verdict::Missed);
        assert_eq!(progress.filled(), 0);
        assert!(!progress.satisfied());

        for _ in 0..3 {
            progress.record(Verdict::Clean);
        }
        assert!(progress.satisfied());
    }

    #[test]
    fn filled_never_runs_past_the_target() {
        let mut progress = GateProgress::new(&clean_passes(2, false));
        for _ in 0..5 {
            progress.record(Verdict::Clean);
        }
        assert_eq!(
            progress.filled(),
            2,
            "the dots have nowhere further to fill"
        );
    }

    // ── The other three requirements (spec §8, #1180) ──

    #[test]
    fn a_key_coverage_gate_banks_a_key_at_a_time() {
        let mut progress = GateProgress::new(&Requirement::KeyCoverage {
            keys_required: 3,
            per_key_passes: 2,
            first_attempt: false,
        });
        assert_eq!(
            (progress.filled(), progress.target()),
            (0, 2),
            "the dots draw the key in front of you, not all twelve"
        );

        progress.record(Verdict::Clean);
        progress.record(Verdict::Clean);
        assert!(
            !progress.satisfied(),
            "one key banked is not the whole coverage"
        );
        assert_eq!(progress.filled(), 0, "the dots empty for the next key");

        for _ in 0..4 {
            progress.record(Verdict::Clean);
        }
        assert!(progress.satisfied());
        assert_eq!(
            progress.filled(),
            2,
            "the last key leaves its dots full to read"
        );
    }

    #[test]
    fn a_first_attempt_key_costs_the_key_when_it_is_missed() {
        let mut progress = GateProgress::new(&Requirement::KeyCoverage {
            keys_required: 2,
            per_key_passes: 1,
            first_attempt: true,
        });
        progress.record(Verdict::Missed);
        assert_eq!(
            progress.filled(),
            0,
            "a dealt key played wrong is not a clean first attempt"
        );

        progress.record(Verdict::Clean);
        progress.record(Verdict::Clean);
        assert!(progress.satisfied());
    }

    #[test]
    fn a_chained_gate_wants_the_run_unbroken() {
        let mut progress = GateProgress::new(&Requirement::Chained { min_keys: 4 });
        for _ in 0..3 {
            progress.record(Verdict::Clean);
        }
        progress.record(Verdict::Missed);
        assert_eq!(
            progress.filled(),
            0,
            "a stop in the chain is what the gate is about"
        );

        for _ in 0..4 {
            progress.record(Verdict::Clean);
        }
        assert!(progress.satisfied());
    }

    #[test]
    fn a_self_confirmed_gate_takes_one_honest_tap() {
        let mut progress = GateProgress::new(&Requirement::SelfConfirmed {
            max_listens: Some(5),
            first_attempt: false,
        });
        assert_eq!((progress.filled(), progress.target()), (0, 1));

        progress.record(Verdict::Missed);
        assert!(!progress.satisfied(), "not yet is not a failure to record");
        progress.record(Verdict::Clean);
        assert!(progress.satisfied());
    }

    #[test]
    fn click_levels_read_as_a_musician_says_them() {
        assert_eq!(ClickLevel::TwoAndFour.spoken(), "beats 2 & 4");
        assert_eq!(ClickLevel::EveryBeat.spoken(), "every beat");
    }

    // ── Click placement ──

    #[test]
    fn every_beat_sounds_the_whole_bar() {
        assert_eq!(ClickLevel::EveryBeat.pattern(4), vec![true; 4]);
        assert_eq!(ClickLevel::EveryBeat.pattern(3), vec![true; 3]);
    }

    #[test]
    fn two_and_four_sounds_the_backbeats() {
        assert_eq!(
            ClickLevel::TwoAndFour.pattern(4),
            vec![false, true, false, true]
        );
    }

    #[test]
    fn two_and_four_drops_the_beat_a_short_bar_does_not_have() {
        assert_eq!(
            ClickLevel::TwoAndFour.pattern(3),
            vec![false, true, false],
            "a bar of 3 has no fourth beat to click"
        );
    }

    #[test]
    fn the_bar_downbeat_sounds_once_a_bar() {
        assert_eq!(
            ClickLevel::BarDownbeat.pattern(4),
            vec![true, false, false, false]
        );
    }

    #[test]
    fn every_other_bar_runs_a_two_bar_cycle() {
        let pattern = ClickLevel::EveryOtherBar.pattern(4);
        assert_eq!(
            pattern.len(),
            8,
            "the alternation is two bars long, so the cycle has to be too"
        );
        assert!(pattern[0]);
        assert!(pattern[1..].iter().all(|beat| !beat));
    }

    #[test]
    fn a_pattern_never_has_a_zero_length_cycle() {
        assert_eq!(
            ClickLevel::EveryBeat.pattern(0),
            vec![true],
            "a bar of no beats would divide by zero in the shell"
        );
    }
}

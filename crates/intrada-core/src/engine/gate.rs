//! Gate criteria as data (engine spec §8), scoped to what a tap-verdict gate
//! needs: a countable requirement, the judge allowed to unlock it, and the
//! progress the dots draw. `KeyCoverage` / `Chained` / `SelfConfirmed`
//! requirements arrive with the `content/gates.toml` parser and §9.6's
//! restructure of that file — an enum variant no reader can evaluate yet would
//! be a gate that silently never passes.

use serde::{Deserialize, Serialize};

/// One tap against a countable criterion (decision 18). The measured verdicts
/// of the deferred scoring path land in the same shape.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Missed,
}

/// Where a verdict came from. Machine scoring returns as a higher-weight
/// evidence class rather than a migration (spec §2, amended 4 Aug 2026).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceSource {
    TapVerdict,
    Midi,
    Audio,
}

/// Who decides a pass. `TapVerdict` is v1's default: user-judged like
/// `SelfConfirmed`, mastery-feeding like `Machine`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judge {
    Machine,
    TapVerdict,
    SelfConfirmed,
}

/// The sparse-click ladder — gate levels *within* click-always (decision 2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    CleanPasses { count: u8, consecutive: bool },
}

/// A gate's tunable half. `time_ceiling_s` is the block ceiling shown
/// throughout (decision 15), not a pass condition.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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
pub struct GateProgress {
    filled: u8,
    target: u8,
    consecutive: bool,
}

impl GateProgress {
    pub fn new(requirement: &Requirement) -> Self {
        match *requirement {
            Requirement::CleanPasses { count, consecutive } => Self {
                filled: 0,
                target: count.max(1),
                consecutive,
            },
        }
    }

    pub fn record(&mut self, verdict: Verdict) {
        match verdict {
            Verdict::Clean => self.filled = (self.filled + 1).min(self.target),
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
        self.filled >= self.target
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

    #[test]
    fn click_levels_read_as_a_musician_says_them() {
        assert_eq!(ClickLevel::TwoAndFour.spoken(), "beats 2 & 4");
        assert_eq!(ClickLevel::EveryBeat.spoken(), "every beat");
    }
}

//! Journey C's two in-session moments: the feel question at a block boundary
//! (C1) and the reflection at close (C2).
//!
//! Both are budget decisions, and the budget is the core's (decision 17). A
//! surface that asked whenever it felt like it would be a second answer to the
//! same question, and the one thing the measurement budget cannot survive is
//! two things believing they own it.

use serde::{Deserialize, Serialize};

use crate::engine::{BlockOrigin, BlockRecord, CoachWrites, Exit, Verdict};

/// Misses in one block that take the feel question off the table. "The budget
/// shrinks on a bad day": having just missed twice, being asked how it felt is
/// the app rubbing it in.
pub const FEEL_MISS_BUDGET: usize = 2;

/// C1, asked at most once per block and only where feel is the point. Held on
/// the model rather than the engine session: a prompt lost to a crash costs
/// nothing, because the block record it follows is already written, and putting
/// it on `EngineSession` would invalidate every crash-recovery blob for it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FeelPrompt {
    pub block_id: String,
    /// The target in the user's own words, joined where the library lives.
    pub title: String,
}

/// Whether the block that just closed earns the feel question.
///
/// Only the judgement track: a gated block ends with its GateDots, and asking
/// both would spend two of the one interruption the block is allowed. A skipped
/// block is not asked either — there is nothing it could be about.
pub fn feel_is_the_point(record: &BlockRecord) -> bool {
    record.origin == BlockOrigin::Judgement
        && record.exit != Exit::Skipped
        && misses(record) < FEEL_MISS_BUDGET
}

fn misses(record: &BlockRecord) -> usize {
    record
        .attempts
        .iter()
        .filter(|attempt| attempt.verdict == Verdict::Missed)
        .count()
}

/// C2, offered once as a session closes.
///
/// Two altitudes are silent here on purpose. Unmonitored play promised minutes
/// and nothing else, so a question at its exit would be the app going back on
/// the one thing it agreed to (decision 16). Off-piste has already asked twice
/// on the way out — the "found something?" mic and the keep-as-drill prompt —
/// and a third is past any budget.
///
/// Neither guard is load-bearing today: as the engine stands, an altitude close
/// carries its own record and no block, so the positive rule below already
/// declines both. They are here because "which altitude was this?" is the
/// question the consent gradient turns on, and a rule that holds only while
/// nobody changes what a close bundles is not a rule (the #1214 class).
pub fn reflection_is_offered(writes: &CoachWrites) -> bool {
    if !writes.unmonitored.is_empty() || !writes.wanders.is_empty() {
        return false;
    }
    !writes.blocks.is_empty() || !writes.play_throughs.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        AttemptSummary, ClickLevel, EvidenceSource, ParameterLevel, UnmonitoredRecord,
    };
    use chrono::{DateTime, Utc};

    fn attempt(verdict: Verdict) -> AttemptSummary {
        AttemptSummary {
            at: DateTime::UNIX_EPOCH,
            verdict,
            source: EvidenceSource::TapVerdictUntimed,
            cold: false,
            self_predicted: None,
            level: ParameterLevel {
                tempo_bpm: 0,
                click_level: ClickLevel::NoClick,
            },
        }
    }

    fn judgement_block() -> BlockRecord {
        BlockRecord {
            origin: BlockOrigin::Judgement,
            exit: Exit::CeilingHit,
            ..BlockRecord::fixture()
        }
    }

    #[test]
    fn a_judgement_block_is_asked_how_it_felt() {
        assert!(feel_is_the_point(&judgement_block()));
    }

    #[test]
    fn a_gated_block_ends_with_its_gate_and_is_never_also_asked() {
        for origin in [BlockOrigin::Authored, BlockOrigin::UserDrill] {
            assert!(!feel_is_the_point(&BlockRecord {
                origin,
                ..judgement_block()
            }));
        }
    }

    #[test]
    fn a_skipped_block_is_not_asked_how_it_felt() {
        assert!(!feel_is_the_point(&BlockRecord {
            exit: Exit::Skipped,
            ..judgement_block()
        }));
    }

    #[test]
    fn two_misses_in_the_block_take_the_question_off_the_table() {
        let one_miss = BlockRecord {
            attempts: vec![attempt(Verdict::Missed), attempt(Verdict::Clean)],
            ..judgement_block()
        };
        assert!(feel_is_the_point(&one_miss));

        let two_misses = BlockRecord {
            attempts: vec![
                attempt(Verdict::Missed),
                attempt(Verdict::Clean),
                attempt(Verdict::Missed),
            ],
            ..judgement_block()
        };
        assert!(!feel_is_the_point(&two_misses));
    }

    fn record_at(at: DateTime<Utc>) -> BlockRecord {
        BlockRecord {
            ended_at: at,
            ..BlockRecord::fixture()
        }
    }

    #[test]
    fn a_session_that_ran_blocks_is_offered_the_reflection() {
        assert!(reflection_is_offered(&CoachWrites {
            blocks: vec![record_at(DateTime::UNIX_EPOCH)],
            ..CoachWrites::default()
        }));
    }

    #[test]
    fn a_session_that_ran_nothing_is_not_asked_to_reflect_on_it() {
        assert!(!reflection_is_offered(&CoachWrites::default()));
    }

    /// A close carrying blocks *and* an unmonitored record is not a shape the
    /// engine writes today. It is exactly the shape the guard exists for: which
    /// altitude was running outranks what else was in the batch, because the
    /// altitude is what the user consented to.
    #[test]
    fn unmonitored_play_keeps_its_promptless_exit_whatever_else_closed_with_it() {
        assert!(!reflection_is_offered(&CoachWrites {
            blocks: vec![record_at(DateTime::UNIX_EPOCH)],
            unmonitored: vec![UnmonitoredRecord {
                id: "u1".into(),
                started_at: DateTime::UNIX_EPOCH,
                ended_at: DateTime::UNIX_EPOCH,
            }],
            ..CoachWrites::default()
        }));
    }

    #[test]
    fn off_piste_has_already_asked_twice_and_is_not_asked_again() {
        assert!(!reflection_is_offered(&CoachWrites {
            blocks: vec![record_at(DateTime::UNIX_EPOCH)],
            wanders: vec![crate::engine::WanderRecord {
                id: "w1".into(),
                started_at: DateTime::UNIX_EPOCH,
                ended_at: DateTime::UNIX_EPOCH,
                attempts: vec![],
                keep_as_drill: None,
                item_id: None,
            }],
            ..CoachWrites::default()
        }));
    }
}

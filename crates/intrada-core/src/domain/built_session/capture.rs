//! Journey C's two in-session moments: the feel question at a block boundary
//! (C1) and the reflection at close (C2).
//!
//! Both are budget decisions, and the budget is the core's (decision 17): the
//! one thing a measurement budget cannot survive is two things owning it.

use serde::{Deserialize, Serialize};

use crate::engine::{BlockOrigin, BlockRecord, CoachWrites, Exit, Verdict};

/// Misses in one block that take the feel question off the table. "The budget
/// shrinks on a bad day": having just missed twice, being asked how it felt is
/// the app rubbing it in.
pub const FEEL_MISS_BUDGET: usize = 2;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FeelPrompt {
    pub block_id: String,
    /// Joined where the library lives: the engine holds ids, never titles.
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

/// C2, offered once as a session closes. Unmonitored play promised minutes and
/// nothing else (decision 16), and off-piste has already asked twice on the way
/// out, so neither may be asked again.
///
/// Neither guard is load-bearing today — an altitude close carries no block, so
/// the positive rule already declines both — but a rule that holds only while
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

    /// Not a shape the engine writes today, and exactly the shape the guard
    /// exists for: the altitude outranks whatever else was in the batch.
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

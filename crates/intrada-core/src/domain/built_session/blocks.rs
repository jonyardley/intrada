//! A composed session, turned into the plan the drill loop runs (A6 → A7).
//!
//! The built session is a steer, not a different app: it becomes an ordinary
//! [`Plan`], and every block after that is the canonical drill loop. What
//! differs per block is only what its taps are allowed to mean, which is
//! [`BlockOrigin`]'s whole job.

use crate::domain::item::{Item, ItemKind};
use crate::engine::{
    BlockOrigin, BlockSpec, Circle, ClickLevel, ContentIndex, GateCriteria, Judge, Maturity, Mode,
    NodeState, ParameterLevel, Plan, PlannedBlock, Requirement, Stage, Why,
};

use super::compose::normalised;
use super::{BuiltBlock, BuiltSession, BuiltTarget, JournalItem, Serves, UserDrill};

/// A6's one-line, declinable suggestion. Advice, never applied on its own:
/// "a shape you can't decline isn't your session."
pub const SHAPE_ADVICE: &str = "Warm-up first, the tune at the end — that's the shape I'd suggest.";

/// What the translation needs to look a target up. The library and the drills
/// live in the model; the content index is static.
pub struct BuildContext<'a> {
    pub items: &'a [Item],
    pub user_drills: &'a [UserDrill],
    pub journal_items: &'a [JournalItem],
    pub content: &'a ContentIndex,
}

impl BuildContext<'_> {
    fn drill(&self, id: &str) -> Option<&UserDrill> {
        self.user_drills.iter().find(|drill| drill.id == id)
    }

    fn journal(&self, id: &str) -> Option<&JournalItem> {
        self.journal_items.iter().find(|journal| journal.id == id)
    }

    fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }
}

/// Node ids for the targets the authored content has never heard of. Prefixed
/// so a user drill can never land its evidence on an authored node by name
/// collision, and so the judgement track is recognisable in a stored record.
pub fn node_id(target: &BuiltTarget) -> String {
    match target {
        BuiltTarget::Node { node } => node.clone(),
        BuiltTarget::UserDrill { drill_id } => format!("drill:{drill_id}"),
        BuiltTarget::Journal { journal_id } => format!("journal:{journal_id}"),
        BuiltTarget::Piece { item_id } => format!("piece:{item_id}"),
    }
}

/// The composed session as today's plan. A block whose target has since been
/// deleted is reported in `deferred`, never dropped in silence (spec §5's rule,
/// and the same reason A2 states its price out loud).
pub fn plan_from_built(session: &BuiltSession, ctx: &BuildContext) -> Plan {
    let mut blocks = Vec::new();
    let mut deferred = Vec::new();
    for block in &session.blocks {
        match spec_for(block, ctx) {
            Some(spec) => blocks.push(PlannedBlock {
                why: Why {
                    destination: spec.destination.clone(),
                    node_state: NodeState {
                        estimate_pct: 0,
                        evidence: 0,
                        overdue_pct: 0,
                        maturity: Maturity::New,
                    },
                    placed_by: Stage::Steer,
                },
                spec,
                // The ladder never swaps material the user chose themselves:
                // an escalation into something they did not ask for would be
                // the app taking the session back.
                alternatives: Vec::new(),
                new_keys: None,
            }),
            None => deferred.push(missing_line(block)),
        }
    }
    Plan {
        blocks,
        rng_seed: 0,
        deferred,
    }
}

fn missing_line(block: &BuiltBlock) -> String {
    match &block.target {
        BuiltTarget::UserDrill { .. } => {
            "One of your drills has gone, so it is not in today's session".to_string()
        }
        BuiltTarget::Journal { .. } => {
            "One of your journal targets has gone, so it is not in today's session".to_string()
        }
        BuiltTarget::Piece { .. } => {
            "A piece you added has gone, so it is not in today's session".to_string()
        }
        BuiltTarget::Node { node } => format!("{node}: no rung of it has a click to run against"),
    }
}

fn spec_for(block: &BuiltBlock, ctx: &BuildContext) -> Option<BlockSpec> {
    match &block.target {
        BuiltTarget::Node { node } => authored_spec(node, block, ctx),
        BuiltTarget::UserDrill { drill_id } => {
            user_drill_spec(ctx.drill(drill_id)?, block, ctx, &block.target)
        }
        BuiltTarget::Journal { journal_id } => {
            let journal = ctx.journal(journal_id)?;
            Some(judgement_spec(
                &journal.name,
                ItemKind::Exercise,
                block,
                ctx,
                &block.target,
            ))
        }
        BuiltTarget::Piece { item_id } => {
            let item = ctx.item(item_id)?;
            Some(judgement_spec(
                &item.title,
                ItemKind::Piece,
                block,
                ctx,
                &block.target,
            ))
        }
    }
}

/// (a) An authored node: the same block the planner would have made, so the
/// evidence lands exactly as a prescribed block's does.
fn authored_spec(node_id: &str, block: &BuiltBlock, ctx: &BuildContext) -> Option<BlockSpec> {
    let node = ctx.content.node(node_id)?;
    let (drill, level) = crate::engine::runnable_rung(ctx.content, node_id)?;
    Some(BlockSpec {
        node: node.id.clone(),
        drill: drill.id.clone(),
        drill_title: node.title.clone(),
        section: drill.section.clone(),
        destination: drill.destination.clone(),
        kind: if drill.applied {
            ItemKind::Piece
        } else {
            ItemKind::Exercise
        },
        circle: node.circle,
        mode: node.mode,
        gate: ctx.content.gate(&drill.gate)?.clone(),
        level,
        bars: drill.bars,
        beats_per_bar: ctx.content.beats_per_bar,
        count_in_beats: ctx.content.count_in_beats,
        minutes: minutes(block, ctx),
        origin: BlockOrigin::Authored,
        serves: None,
    })
}

/// (b) A user drill. The criterion sentence is the gate; the rung is whatever
/// the sentence stated, and a sentence with no tempo is l0 — the clickless
/// acquisition block, where the tap is what bounds the attempt (decision 20).
fn user_drill_spec(
    drill: &UserDrill,
    block: &BuiltBlock,
    ctx: &BuildContext,
    target: &BuiltTarget,
) -> Option<BlockSpec> {
    let node = node_id(target);
    let level = match drill.tempo_bpm {
        Some(tempo_bpm) => ParameterLevel {
            tempo_bpm,
            click_level: ClickLevel::EveryBeat,
        },
        None => ParameterLevel {
            tempo_bpm: 0,
            click_level: ClickLevel::NoClick,
        },
    };
    let requirement = if drill.keys.len() > 1 {
        Requirement::KeyCoverage {
            keys_required: u8::try_from(drill.keys.len()).unwrap_or(u8::MAX),
            per_key_passes: drill.passes_to_open,
            first_attempt: false,
        }
    } else {
        Requirement::CleanPasses {
            count: drill.passes_to_open,
            consecutive: false,
        }
    };
    Some(BlockSpec {
        gate: GateCriteria {
            id: format!("gate:{}", drill.id),
            node: node.clone(),
            criterion: drill.criterion.clone(),
            requirement,
            judge: Judge::TapVerdict,
            time_ceiling_s: None,
        },
        node,
        drill: drill.id.clone(),
        drill_title: drill.name.clone(),
        section: None,
        destination: None,
        kind: ItemKind::Exercise,
        // A drill the user wrote is work at the keys until something says
        // otherwise; a `serves` circle is that something.
        circle: match drill.serves {
            Some(Serves::Circle(circle)) => circle,
            _ => Circle::Hands,
        },
        mode: Mode::Keys,
        level,
        bars: ctx.content.bars,
        beats_per_bar: ctx.content.beats_per_bar,
        count_in_beats: ctx.content.count_in_beats,
        minutes: minutes(block, ctx),
        origin: BlockOrigin::UserDrill,
        serves: serves_line(drill.serves.as_ref(), ctx),
    })
}

/// (c) and the Phase-B piece block: time is logged, one tap says when it is
/// done, and nothing is inferred (decisions 16 and 17). Untimed, because
/// nothing here is being counted against a clock.
fn judgement_spec(
    title: &str,
    kind: ItemKind,
    block: &BuiltBlock,
    ctx: &BuildContext,
    target: &BuiltTarget,
) -> BlockSpec {
    let node = node_id(target);
    BlockSpec {
        gate: GateCriteria {
            id: format!("gate:{node}"),
            node: node.clone(),
            criterion: "You decide when it's done.".to_string(),
            requirement: Requirement::SelfConfirmed {
                max_listens: None,
                first_attempt: false,
            },
            judge: Judge::SelfConfirmed,
            time_ceiling_s: None,
        },
        node,
        drill: String::new(),
        drill_title: title.to_string(),
        section: None,
        destination: None,
        kind,
        circle: Circle::Bridge,
        mode: Mode::Keys,
        level: ParameterLevel {
            tempo_bpm: 0,
            click_level: ClickLevel::NoClick,
        },
        bars: ctx.content.bars,
        beats_per_bar: ctx.content.beats_per_bar,
        count_in_beats: ctx.content.count_in_beats,
        minutes: minutes(block, ctx),
        origin: BlockOrigin::Judgement,
        serves: None,
    }
}

fn minutes(block: &BuiltBlock, ctx: &BuildContext) -> u16 {
    block
        .minutes
        .unwrap_or(ctx.content.planner.acquisition_minutes)
}

/// A7's line: where this drill's evidence shows in the ability picture. The
/// player's words, and never a claim the tag cannot back.
fn serves_line(serves: Option<&Serves>, ctx: &BuildContext) -> Option<String> {
    match serves? {
        Serves::Circle(Circle::Head) => Some("Adds to what you can hear".to_string()),
        Serves::Circle(Circle::Hands) => Some("Adds to what your hands know".to_string()),
        Serves::Circle(Circle::Bridge) => Some("Adds to putting the two together".to_string()),
        Serves::Node(name) => {
            let resolved = ctx
                .content
                .node(name)
                .map(|node| node.title.clone())
                .or_else(|| ctx.item(name).map(|item| item.title.clone()))
                .or_else(|| {
                    let wanted = normalised(name);
                    ctx.items
                        .iter()
                        .find(|item| normalised(&item.title) == wanted)
                        .map(|item| item.title.clone())
                })
                .unwrap_or_else(|| name.clone());
            Some(format!("Adds to {resolved}"))
        }
    }
}

/// The template shape as advice (decision 19): warm up on what is countable,
/// leave the music for the ends. Returns the blocks reordered; the caller
/// decides whether the user wanted it.
pub fn suggested_shape(blocks: &[BuiltBlock]) -> Vec<BuiltBlock> {
    let mut reordered = blocks.to_vec();
    reordered.sort_by_key(shape_rank);
    reordered
}

/// Stable within a band, so declining and re-accepting cannot shuffle the
/// blocks the shape does not care about.
fn shape_rank(block: &BuiltBlock) -> u8 {
    match block.target {
        BuiltTarget::Node { .. } | BuiltTarget::UserDrill { .. } => 0,
        BuiltTarget::Journal { .. } => 1,
        BuiltTarget::Piece { .. } => 2,
    }
}

/// `Some` only when the shape has something to offer: advice that repeats an
/// order the user already chose is noise, and it would make "Keep my order"
/// read as a refusal of nothing.
pub fn shape_advice(blocks: &[BuiltBlock]) -> Option<&'static str> {
    (suggested_shape(blocks) != blocks).then_some(SHAPE_ADVICE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::built_session::tests_support::{
        built_block, journal_item, sample_item, user_drill,
    };
    use crate::engine::Verdict;

    fn ctx<'a>(
        items: &'a [Item],
        drills: &'a [UserDrill],
        journals: &'a [JournalItem],
    ) -> BuildContext<'a> {
        BuildContext {
            items,
            user_drills: drills,
            journal_items: journals,
            content: ContentIndex::shipped(),
        }
    }

    fn session(blocks: Vec<BuiltBlock>) -> BuiltSession {
        let mut session = crate::domain::built_session::tests_support::built_session();
        session.blocks = blocks;
        session
    }

    #[test]
    fn a_user_drill_becomes_a_gated_block_on_its_own_node() {
        let drills = vec![user_drill("d1", "Stride pattern")];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &drills, &[]),
        );
        let spec = &plan.blocks[0].spec;
        assert_eq!(spec.node, "drill:d1", "its own node, never an authored one");
        assert_eq!(spec.origin, BlockOrigin::UserDrill);
        assert!(spec.origin.feeds_mastery(), "a user drill counts in full");
        assert_eq!(
            spec.gate.requirement,
            Requirement::CleanPasses {
                count: 3,
                consecutive: false
            },
            "the criterion's passes are the gate"
        );
        assert_eq!(spec.gate.criterion, drills[0].criterion);
    }

    #[test]
    fn a_drill_with_no_tempo_is_the_clickless_acquisition_block() {
        let mut drill = user_drill("d1", "Rubato shape");
        drill.tempo_bpm = None;
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &[drill], &[]),
        );
        assert!(
            plan.blocks[0].spec.level.is_untimed(),
            "a sentence with no tempo has no clock to state one against (decision 20)"
        );
    }

    #[test]
    fn a_drill_naming_several_keys_gets_a_key_coverage_gate() {
        let mut drill = user_drill("d1", "Two-fives");
        drill.keys = vec!["F".into(), "Bb".into(), "Eb".into()];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &[drill], &[]),
        );
        assert_eq!(
            plan.blocks[0].spec.gate.requirement,
            Requirement::KeyCoverage {
                keys_required: 3,
                per_key_passes: 3,
                first_attempt: false
            }
        );
    }

    #[test]
    fn a_journal_target_never_feeds_mastery() {
        let journals = vec![journal_item("j1", "Freer rubato in the intro")];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::Journal {
                    journal_id: "j1".into(),
                },
            )]),
            &ctx(&[], &[], &journals),
        );
        let spec = &plan.blocks[0].spec;
        assert_eq!(spec.origin, BlockOrigin::Judgement);
        assert!(
            !spec.origin.feeds_mastery(),
            "decision 17: qualitative data never feeds mastery"
        );
        assert_eq!(spec.drill_title, "Freer rubato in the intro");
    }

    #[test]
    fn a_piece_block_is_the_judgement_track_until_pipelines_are_authored() {
        let items = vec![sample_item("p1", "Alice in Wonderland", ItemKind::Piece)];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::Piece {
                    item_id: "p1".into(),
                },
            )]),
            &ctx(&items, &[], &[]),
        );
        let spec = &plan.blocks[0].spec;
        assert_eq!(spec.kind, ItemKind::Piece);
        assert!(
            !spec.origin.feeds_mastery(),
            "nothing can count a whole piece yet, so nothing claims to (open question 1)"
        );
    }

    #[test]
    fn a_target_that_has_been_deleted_is_reported_not_dropped_in_silence() {
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "gone".into(),
                },
            )]),
            &ctx(&[], &[], &[]),
        );
        assert!(plan.blocks.is_empty());
        assert_eq!(plan.deferred.len(), 1, "the user is told what is missing");
    }

    #[test]
    fn every_block_says_whose_session_it_is() {
        let drills = vec![user_drill("d1", "Stride pattern")];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &drills, &[]),
        );
        assert_eq!(plan.blocks[0].why.placed_by, Stage::Steer);
        assert!(
            plan.blocks[0].why_line().starts_with("You asked for"),
            "the why is the user's, not the planner's: {}",
            plan.blocks[0].why_line()
        );
    }

    #[test]
    fn the_ladder_never_swaps_material_the_user_chose() {
        let drills = vec![user_drill("d1", "Stride pattern")];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &drills, &[]),
        );
        assert!(plan.blocks[0].alternatives.is_empty());
    }

    #[test]
    fn the_serves_tag_becomes_the_line_the_boundary_card_shows() {
        let items = vec![sample_item("p1", "Alice in Wonderland", ItemKind::Piece)];
        let mut drill = user_drill("d1", "Stride pattern");
        drill.serves = Some(Serves::Node("p1".into()));
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&items, &[drill], &[]),
        );
        assert_eq!(
            plan.blocks[0].spec.serves.as_deref(),
            Some("Adds to Alice in Wonderland")
        );
    }

    #[test]
    fn a_circle_serves_tag_speaks_the_players_words() {
        let mut drill = user_drill("d1", "Ear work");
        drill.serves = Some(Serves::Circle(Circle::Head));
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &[drill], &[]),
        );
        assert_eq!(
            plan.blocks[0].spec.serves.as_deref(),
            Some("Adds to what you can hear")
        );
    }

    #[test]
    fn a_drill_with_no_serves_tag_claims_nothing() {
        let drills = vec![user_drill("d1", "Stride pattern")];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::UserDrill {
                    drill_id: "d1".into(),
                },
            )]),
            &ctx(&[], &drills, &[]),
        );
        assert_eq!(plan.blocks[0].spec.serves, None);
    }

    // ── Shape as advice ────────────────────────────────────────────────

    fn shaped(order: &[BuiltTarget]) -> Vec<BuiltBlock> {
        order
            .iter()
            .enumerate()
            .map(|(index, target)| built_block(&format!("b{index}"), target.clone()))
            .collect()
    }

    #[test]
    fn the_suggested_shape_warms_up_first_and_leaves_the_music_for_the_ends() {
        let blocks = shaped(&[
            BuiltTarget::Piece {
                item_id: "p1".into(),
            },
            BuiltTarget::Journal {
                journal_id: "j1".into(),
            },
            BuiltTarget::UserDrill {
                drill_id: "d1".into(),
            },
        ]);
        let shape = suggested_shape(&blocks);
        assert_eq!(
            shape.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
            vec!["b2", "b1", "b0"]
        );
    }

    #[test]
    fn advice_that_repeats_the_users_own_order_is_not_offered() {
        let blocks = shaped(&[
            BuiltTarget::UserDrill {
                drill_id: "d1".into(),
            },
            BuiltTarget::Piece {
                item_id: "p1".into(),
            },
        ]);
        assert_eq!(
            shape_advice(&blocks),
            None,
            "\"Keep my order\" must not be a refusal of nothing"
        );
        assert_eq!(
            shape_advice(&suggested_shape(&shaped(&[
                BuiltTarget::Piece {
                    item_id: "p1".into()
                },
                BuiltTarget::UserDrill {
                    drill_id: "d1".into()
                },
            ]))),
            None
        );
    }

    #[test]
    fn the_shape_is_stable_within_a_band() {
        let blocks = shaped(&[
            BuiltTarget::UserDrill {
                drill_id: "d1".into(),
            },
            BuiltTarget::UserDrill {
                drill_id: "d2".into(),
            },
            BuiltTarget::Piece {
                item_id: "p1".into(),
            },
        ]);
        assert_eq!(suggested_shape(&blocks), blocks);
    }

    #[test]
    fn a_judgement_block_asks_the_user_rather_than_counting() {
        let journals = vec![journal_item("j1", "Freer rubato")];
        let plan = plan_from_built(
            &session(vec![built_block(
                "b1",
                BuiltTarget::Journal {
                    journal_id: "j1".into(),
                },
            )]),
            &ctx(&[], &[], &journals),
        );
        assert_eq!(plan.blocks[0].spec.gate.judge, Judge::SelfConfirmed);
        let _ = Verdict::Clean;
    }
}

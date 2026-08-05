//! What a session was asked to run, planned from the authored content
//! (`content/gates.toml`) rather than seeded in Rust (#1180).
//!
//! Spec §5's stages, and where this stops short of them. Stage 1 resolves the
//! declared intent, stage 2 back-chains through prerequisites, stage 3 sizes a
//! block by node maturity, and stage 5's template constraints (fit the session,
//! warm up on owned material, close on music) are last and overriding. Two
//! things are honestly missing rather than faked: stage 3 reads the *seeded*
//! estimate, because the live mastery store is #1148, and with it goes the
//! `overdue` pull that puts maintenance ahead of new keys; and stage 4's grind
//! cap has nothing to bite on while a node contributes at most one block. Each
//! stage still only removes or reorders, so the why stays generable.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::content::{ContentIndex, Drill, Node};
use super::gate::{ClickLevel, GateCriteria};
#[cfg(test)]
use super::gate::{Judge, Requirement};
use crate::domain::item::ItemKind;

/// Which circle of the fluency frame a node grows: the music you can hear
/// (`Head`), what the hands can execute (`Hands`), or the overlap that is the
/// point of the whole thing (`Bridge`). Authored per node — the tags in
/// `content/nodes.md`, defined in `content/README.md` "Circle tags".
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Circle {
    Head,
    Hands,
    Bridge,
}

/// Whether the work needs the instrument. `KeysToAway` is a difficulty ladder
/// from the keys toward pure audiation, not a third place to practise.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Mode {
    Keys,
    Away,
    KeysToAway,
}

/// The rung of a parameter ladder a block is being practised at. Mastery is
/// held per `(node, parameter_level)` (spec §2), so this is state, not display.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ParameterLevel {
    pub tempo_bpm: u16,
    pub click_level: ClickLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BlockSpec {
    pub node: String,
    pub drill: String,
    pub drill_title: String,
    /// Where in the material it sits — "A section", "key of F".
    pub section: Option<String>,
    /// The in-flight tune or campaign this block serves.
    pub destination: Option<String>,
    pub kind: ItemKind,
    /// The fluency-frame tags this block inherits from its node, carried onto
    /// the record so the time-by-circle tally is free at write time (spec §4).
    pub circle: Circle,
    pub mode: Mode,
    pub gate: GateCriteria,
    pub level: ParameterLevel,
    pub bars: u16,
    pub beats_per_bar: u8,
    pub count_in_beats: u8,
    /// What the plan allowed this block, which is the ceiling shown throughout
    /// (decision 15). The gate's own `time_ceiling_s` is the content default
    /// the planner allocated from, never past it.
    pub minutes: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Plan {
    pub blocks: Vec<BlockSpec>,
    /// The seeded dealer for random-key gates, stored so any session replays
    /// in a test (spec §5).
    pub rng_seed: u64,
    /// What today could not take, in the plan's own words. An over-full
    /// campaign is sequenced and reported; silent dropping is a defect
    /// (spec §5 stage 5), and the next plan reads this.
    pub deferred: Vec<String>,
}

impl Plan {
    pub fn for_today(content: &ContentIndex, available_minutes: u16, rng_seed: u64) -> Self {
        let mut route = Vec::new();
        let mut deferred = Vec::new();
        let mut visited = BTreeSet::new();

        // Stages 1 and 2: the campaign's targets, in the order it declares
        // them, each preceded by what it depends on. A target may name the
        // drill it wants (content/intent.md states both kinds).
        for target in &content.intent.targets {
            let (node_id, wanted) = match content.drill(target) {
                Some(drill) => (drill.node.as_str(), Some(drill.id.as_str())),
                None => (target.as_str(), None),
            };
            back_chain(
                content,
                node_id,
                wanted,
                &mut visited,
                &mut route,
                &mut deferred,
            );
        }

        // Stage 5: the template, last and overriding. Fit the session first,
        // then close on music.
        let mut blocks: Vec<BlockSpec> = Vec::new();
        let mut spent = 0;
        for (node, drill) in route {
            let minutes = block_minutes(content, node, drill);
            if spent + minutes > available_minutes {
                deferred.push(format!(
                    "{}: no room in {available_minutes} minutes today",
                    drill.id
                ));
                continue;
            }
            spent += minutes;
            blocks.push(block_spec(content, node, drill, minutes));
        }
        if let Some(applied) = blocks
            .iter()
            .rposition(|block| block.kind == ItemKind::Piece)
        {
            let music = blocks.remove(applied);
            blocks.push(music);
        }

        Self {
            blocks,
            rng_seed,
            deferred,
        }
    }
}

/// Prerequisites before what needs them, each node once. A node with no
/// runnable rung yields no block: an ambition beyond what the content can
/// currently host is reported, never invented (decision 10).
fn back_chain<'c>(
    content: &'c ContentIndex,
    node_id: &str,
    wanted: Option<&str>,
    visited: &mut BTreeSet<String>,
    route: &mut Vec<(&'c Node, &'c Drill)>,
    deferred: &mut Vec<String>,
) {
    if !visited.insert(node_id.to_string()) {
        return;
    }
    let Some(node) = content.node(node_id) else {
        return;
    };
    for prerequisite in &node.prerequisites {
        back_chain(content, prerequisite, None, visited, route, deferred);
    }
    if node.drills.is_empty() {
        deferred.push(format!(
            "{node_id}: a stub, so there is no gate to pass yet"
        ));
        return;
    }
    let runnable = |drill: &&Drill| drill.level.is_some();
    let chosen = wanted
        .and_then(|id| content.drill(id))
        .filter(runnable)
        .or_else(|| {
            node.drills
                .iter()
                .filter_map(|id| content.drill(id))
                .find(runnable)
        });
    match chosen {
        Some(drill) => route.push((node, drill)),
        None => deferred.push(format!(
            "{node_id}: no rung of it has a click to run against"
        )),
    }
}

/// Stage 3: low evidence at the frontier wants one longer uninterrupted block,
/// matured material a shorter pass. Never past the gate's own ceiling.
fn block_minutes(content: &ContentIndex, node: &Node, drill: &Drill) -> u16 {
    let by_maturity = if node.estimate >= content.planner.maintenance_estimate {
        content.planner.maintenance_minutes
    } else {
        content.planner.acquisition_minutes
    };
    let wanted = drill.minutes.unwrap_or(by_maturity);
    match content
        .gate(&drill.gate)
        .and_then(|gate| gate.time_ceiling_s)
    {
        Some(ceiling_s) => wanted.min((ceiling_s / 60) as u16),
        None => wanted,
    }
}

fn block_spec(content: &ContentIndex, node: &Node, drill: &Drill, minutes: u16) -> BlockSpec {
    let gate = content
        .gate(&drill.gate)
        .expect("the parser resolved every drill's gate")
        .clone();
    BlockSpec {
        node: node.id.clone(),
        drill: drill.id.clone(),
        drill_title: node.title.clone(),
        section: drill.section.clone(),
        // Whatever the drill serves, the block is in the campaign's route, so
        // the why cites the campaign (principle 7).
        destination: Some(
            drill
                .destination
                .clone()
                .unwrap_or_else(|| content.intent.destination.clone()),
        ),
        // A drill parameterised by real music is music; the rest are exercises.
        kind: if drill.applied {
            ItemKind::Piece
        } else {
            ItemKind::Exercise
        },
        circle: node.circle,
        mode: node.mode,
        level: drill
            .level
            .expect("back-chaining dropped the unrunnable rungs"),
        gate,
        bars: drill.bars,
        beats_per_bar: content.beats_per_bar,
        count_in_beats: content.count_in_beats,
        minutes,
    }
}

#[cfg(test)]
impl Plan {
    /// A stable plan for the state-machine tests, so what they assert is the
    /// machine rather than whatever the authored content currently says.
    pub(crate) fn fixture() -> Self {
        Self {
            blocks: vec![BlockSpec {
                node: "rootless-a-b".to_string(),
                drill: "rootless-under-melody".to_string(),
                drill_title: "Rootless voicings".to_string(),
                section: Some("A section".to_string()),
                destination: Some("Strasbourg / St. Denis".to_string()),
                kind: ItemKind::Exercise,
                circle: Circle::Hands,
                mode: Mode::Keys,
                gate: GateCriteria {
                    id: "rootless-under-melody".to_string(),
                    node: "rootless-a-b".to_string(),
                    criterion: "Strasbourg A section, melody over rootless LH".to_string(),
                    requirement: Requirement::CleanPasses {
                        count: 3,
                        consecutive: false,
                    },
                    judge: Judge::TapVerdict,
                    time_ceiling_s: Some(360),
                },
                level: ParameterLevel {
                    tempo_bpm: 120,
                    click_level: ClickLevel::TwoAndFour,
                },
                bars: 8,
                beats_per_bar: 4,
                count_in_beats: 4,
                minutes: 6,
            }],
            rng_seed: 0,
            deferred: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::content::ContentIndex;

    fn plan(minutes: u16) -> Plan {
        Plan::for_today(ContentIndex::shipped(), minutes, 0)
    }

    fn drills(plan: &Plan) -> Vec<&str> {
        plan.blocks
            .iter()
            .map(|block| block.drill.as_str())
            .collect()
    }

    #[test]
    fn the_plan_is_the_route_back_chained_from_the_declared_intent() {
        assert_eq!(
            drills(&plan(20)),
            vec![
                "shells-cycle",
                "targeting-skeleton",
                "phrase-home-key",
                "rootless-under-melody",
            ],
            "shells first because both targets need it, then the targets in the \
             order intent.md declares them, and the music block last"
        );
    }

    #[test]
    fn a_stub_yields_no_block_rather_than_a_guessed_one() {
        let plan = plan(60);
        assert!(
            !plan
                .blocks
                .iter()
                .any(|block| block.node == "tune-form-memory"),
            "a declared target with no drills has no gate to pass (content/intent.md)"
        );
        assert!(
            plan.deferred
                .iter()
                .any(|note| note.contains("tune-form-memory")),
            "and it is reported rather than dropped in silence: {:?}",
            plan.deferred
        );
    }

    #[test]
    fn a_drill_with_no_click_to_run_against_is_not_scheduled() {
        let plan = plan(60);
        assert!(
            !plan
                .blocks
                .iter()
                .any(|block| block.node == "micro-transcription"),
            "the away-from-the-keys rungs have no tempo, so this loop cannot host them"
        );
        assert!(plan
            .deferred
            .iter()
            .any(|note| note.contains("micro-transcription")));
    }

    #[test]
    fn an_over_full_campaign_is_sequenced_rather_than_silently_dropped() {
        let plan = plan(12);
        let minutes: u16 = plan.blocks.iter().map(|block| block.minutes).sum();
        assert!(minutes <= 12, "{minutes} minutes planned into 12");
        assert!(
            plan.deferred
                .iter()
                .any(|note| note.contains("rootless-under-melody")),
            "what did not fit is queued for the next plan: {:?}",
            plan.deferred
        );
    }

    #[test]
    fn the_session_closes_on_music() {
        let plan = plan(20);
        assert_eq!(
            plan.blocks.last().map(|block| block.drill.as_str()),
            Some("rootless-under-melody"),
            "an applied block is reordered to last, whatever the route order was"
        );
        assert!(plan
            .blocks
            .iter()
            .any(|block| block.kind == ItemKind::Piece));
    }

    #[test]
    fn the_warm_up_is_the_material_already_owned() {
        let plan = plan(20);
        let first = plan.blocks.first().expect("a warm-up block");
        assert_eq!(
            first.node, "shells-ii-v-i",
            "0.7 seeded: maintenance, not learning"
        );
        assert_eq!(
            first.minutes, 4,
            "matured material wants a shorter pass (spec §5 stage 3)"
        );
    }

    #[test]
    fn an_acquisition_block_gets_the_longer_uninterrupted_run() {
        let block = plan(20)
            .blocks
            .into_iter()
            .find(|block| block.node == "chord-tone-targeting")
            .expect("the frontier of the campaign");
        assert_eq!(
            block.minutes, 6,
            "low evidence wants the longer block, and 6 minutes is the ceiling"
        );
    }

    #[test]
    fn every_block_names_the_destination_it_serves() {
        for block in plan(30).blocks {
            assert_eq!(
                block.destination.as_deref(),
                Some("Strasbourg / St. Denis"),
                "{} is in the campaign's route, so the why cites the campaign",
                block.drill
            );
        }
    }

    #[test]
    fn a_block_carries_the_gate_and_level_the_file_authored() {
        let block = plan(20)
            .blocks
            .into_iter()
            .find(|block| block.drill == "rootless-under-melody")
            .unwrap();

        assert_eq!(block.drill_title, "Rootless voicings");
        assert_eq!(block.section.as_deref(), Some("A section"));
        assert_eq!(
            block.level.tempo_bpm, 80,
            "the file's tempo, not a Rust one"
        );
        assert_eq!(block.level.click_level, ClickLevel::EveryBeat);
        assert_eq!(
            block.gate.requirement,
            Requirement::CleanPasses {
                count: 2,
                consecutive: false
            }
        );
        assert_eq!((block.circle, block.mode), (Circle::Hands, Mode::Keys));
        assert_eq!(block.beats_per_bar, 4);
        assert_eq!(block.count_in_beats, 4);
    }
}

//! The planner (spec §5): five ordered stages turning the authored content and
//! the live mastery track into today's session.
//!
//! Pure — the clock is a parameter and the only randomness is a seeded dealer
//! whose seed is stored on the `Plan`, so any session replays in a test. Each
//! stage may only remove or reorder candidates, never add one, which is what
//! makes every block's `why` generable by construction.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::coach::CoachState;
use super::content::{ContentIndex, Drill, Node, Traversal};
use super::gate::{ClickLevel, GateCriteria};
#[cfg(test)]
use super::gate::{Judge, Requirement};
use super::mastery::{MasteryStore, Reading};
use super::session::Rung;
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
    /// Meaningless at l0, where there is no clock to state one against: read it
    /// through [`ParameterLevel::is_untimed`] rather than on its own.
    pub tempo_bpm: u16,
    pub click_level: ClickLevel,
}

impl ParameterLevel {
    /// The acquisition rung (decision 20): no click, so no tempo, no count-in
    /// and no phrase boundary to bound an attempt on.
    pub fn is_untimed(&self) -> bool {
        self.click_level == ClickLevel::NoClick
    }
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

/// Which stage placed a block. Written by the stage itself, so the why cannot
/// drift from what actually happened.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Stage {
    Intent,
    BackChain,
    Interleave,
    GrindCap,
    Template,
    /// Not a planning stage: the ladder drew this block mid-session from the
    /// alternatives the plan carried (#1182).
    Escalation,
}

/// How well the hands know this rung, in the terms a why line says out loud.
/// Integers rather than floats, per spec §6.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct NodeState {
    pub estimate_pct: u8,
    /// Attempts beyond the prior (spec §2's `evidence`).
    pub evidence: u16,
    /// `100` is due; above it, overdue.
    pub overdue_pct: u16,
    pub maturity: Maturity,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Maturity {
    /// Nothing attempted at this rung yet.
    New,
    Acquiring,
    Maintaining,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Why {
    pub destination: Option<String>,
    pub node_state: NodeState,
    pub placed_by: Stage,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlannedBlock {
    pub spec: BlockSpec,
    pub why: Why,
    /// What the ladder's third and fourth rungs may draw on. Empty where the
    /// content offers nothing, which is why a block can still end
    /// `Exit::Escalated` rather than pretending to escalate.
    pub alternatives: Vec<Alternative>,
    /// The dealer's new-key quota for this block, where the node authors a
    /// `[traversal.<node>]` entry. `None` where it does not.
    pub new_keys: Option<u8>,
}

/// A block the ladder may swap in when the one in flight is not landing. Held
/// on the plan rather than worked out mid-block: choosing what to practise is
/// the planner's job, and the state machine only moves the cursor.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Alternative {
    pub rung: Rung,
    pub spec: BlockSpec,
    pub why: Why,
}

impl PlannedBlock {
    /// The why, said out loud. Written here rather than in the shell: the shell
    /// is a dumb pipe, and one sentence per block is what press-start renders.
    pub fn why_line(&self) -> String {
        let title = &self.spec.drill_title;
        let mut line = match self.why.placed_by {
            Stage::BackChain => format!("{title} comes first: the route runs through it"),
            Stage::Interleave | Stage::Template if self.why.node_state.overdue_pct >= 100 => {
                format!("{title} is due back")
            }
            Stage::Interleave | Stage::Template
                if self.why.node_state.maturity == Maturity::Maintaining =>
            {
                format!("A warm-up on {}, which you own", lower_first(title))
            }
            Stage::Escalation => format!("Another way into {}", lower_first(title)),
            _ => match self.why.node_state.maturity {
                Maturity::New => format!("{title} is new ground"),
                _ => format!("{title} is the frontier"),
            },
        };
        if let Some(destination) = &self.why.destination {
            line.push_str(&format!(" for {destination}"));
        }
        if let Some(keys) = self.new_keys {
            line.push_str(&format!(", {keys} new keys today"));
        }
        line.push('.');
        line
    }
}

fn lower_first(title: &str) -> String {
    let mut chars = title.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Plan {
    pub blocks: Vec<PlannedBlock>,
    /// The seeded dealer for random-key gates, stored so any session replays
    /// in a test (spec §5).
    pub rng_seed: u64,
    /// What today could not take, in the plan's own words. An over-full
    /// campaign is sequenced and reported; silent dropping is a defect
    /// (spec §5 stage 5), and the next plan reads this.
    pub deferred: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanContext {
    pub now: DateTime<Utc>,
    pub available_minutes: u16,
    pub rng_seed: u64,
}

/// Today's session. The declaration surfaces (goal, campaign, steer, length)
/// are Phase 2b, so stage 1 runs on the authored defaults.
pub fn plan(state: &CoachState, ctx: PlanContext) -> Plan {
    plan_from(ContentIndex::shipped(), &state.mastery, ctx)
}

fn plan_from(content: &ContentIndex, mastery: &MasteryStore, ctx: PlanContext) -> Plan {
    let mut deferred = Vec::new();
    let targets = resolve_intent(content, &mut deferred);
    let mut candidates = back_chain(content, &targets, &mut deferred);
    interleave(content, mastery, ctx.now, &mut candidates);
    cap_grind(content, &mut candidates, &mut deferred);
    let blocks = fit_template(content, mastery, ctx, candidates, &mut deferred);

    Plan {
        blocks,
        rng_seed: ctx.rng_seed,
        deferred,
    }
}

/// Stage 1. A target may name the rung it wants (`content/intent.md` states
/// both kinds). What nothing can count passes through as an opaque target
/// rather than being quietly turned into a block.
fn resolve_intent(content: &ContentIndex, deferred: &mut Vec<String>) -> Vec<Target> {
    for target in &content.intent.opaque_targets {
        deferred.push(format!(
            "{target}: nothing here can count it, so it stays your call"
        ));
    }
    content
        .intent
        .targets
        .iter()
        .map(|target| match content.drill(target) {
            Some(drill) => Target {
                node: drill.node.clone(),
                wanted: Some(drill.id.clone()),
            },
            None => Target {
                node: target.clone(),
                wanted: None,
            },
        })
        .collect()
}

struct Target {
    node: String,
    wanted: Option<String>,
}

/// Stage 2. Prerequisites before what needs them, each node once, down to a
/// frontier of `(node, level)` candidates. A node with no runnable rung yields
/// no block: an ambition beyond what the content can currently host is
/// reported, never invented (decision 10).
fn back_chain<'c>(
    content: &'c ContentIndex,
    targets: &[Target],
    deferred: &mut Vec<String>,
) -> Vec<Candidate<'c>> {
    let mut visited = BTreeSet::new();
    let mut candidates = Vec::new();
    for target in targets {
        walk(
            content,
            &target.node,
            target.wanted.as_deref(),
            Stage::Intent,
            &mut visited,
            &mut candidates,
            deferred,
        );
    }
    candidates
}

fn walk<'c>(
    content: &'c ContentIndex,
    node_id: &str,
    wanted: Option<&str>,
    placed_by: Stage,
    visited: &mut BTreeSet<String>,
    candidates: &mut Vec<Candidate<'c>>,
    deferred: &mut Vec<String>,
) {
    if !visited.insert(node_id.to_string()) {
        return;
    }
    let Some(node) = content.node(node_id) else {
        return;
    };
    for prerequisite in &node.prerequisites {
        walk(
            content,
            prerequisite,
            None,
            Stage::BackChain,
            visited,
            candidates,
            deferred,
        );
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
        Some(drill) => candidates.push(Candidate {
            node,
            drill,
            level: drill.level.expect("filtered to the runnable rungs"),
            minutes: 0,
            reading: Reading {
                estimate: 0.0,
                evidence: 0.0,
                overdue: 0.0,
            },
            placed_by,
        }),
        None => deferred.push(format!(
            "{node_id}: no rung of it has a click to run against"
        )),
    }
}

struct Candidate<'c> {
    node: &'c Node,
    drill: &'c Drill,
    level: ParameterLevel,
    minutes: u16,
    reading: Reading,
    placed_by: Stage,
}

impl Candidate<'_> {
    fn maturity(&self, maintenance_estimate: f32) -> Maturity {
        if self.reading.estimate >= maintenance_estimate {
            Maturity::Maintaining
        } else if self.reading.evidence >= 1.0 {
            Maturity::Acquiring
        } else {
            Maturity::New
        }
    }

    fn due_maintenance(&self, maintenance_estimate: f32) -> bool {
        self.reading.overdue >= 1.0 && self.maturity(maintenance_estimate) == Maturity::Maintaining
    }
}

/// Stage 3. Low evidence at the frontier wants one longer uninterrupted block,
/// matured material a shorter pass (lever 1), and `overdue >= 1` pulls
/// maintenance ahead of new keys — which is how journey 7 falls out rather than
/// being special-cased.
fn interleave(
    content: &ContentIndex,
    mastery: &MasteryStore,
    now: DateTime<Utc>,
    candidates: &mut [Candidate],
) {
    for candidate in candidates.iter_mut() {
        candidate.reading = mastery.reading(&candidate.node.id, candidate.level, now);
        let by_maturity = match candidate.maturity(content.planner.maintenance_estimate) {
            Maturity::Maintaining => content.planner.maintenance_minutes,
            Maturity::Acquiring | Maturity::New => content.planner.acquisition_minutes,
        };
        candidate.minutes = allowed_minutes(content, candidate.drill, by_maturity);
    }
    for candidate in candidates.iter_mut() {
        if candidate.due_maintenance(content.planner.maintenance_estimate) {
            // This stage is why the block sits where it does, so it takes the
            // why: "due back" rather than whatever put it in the route.
            candidate.placed_by = Stage::Interleave;
        }
    }
    candidates.sort_by_key(|candidate| {
        let maintenance_estimate = content.planner.maintenance_estimate;
        match (
            candidate.due_maintenance(maintenance_estimate),
            candidate.maturity(maintenance_estimate) == Maturity::Maintaining,
        ) {
            (true, _) => 0,
            (_, true) => 1,
            _ => 2,
        }
    });
}

/// Never past the gate's own ceiling: the plan allocates from it, it does not
/// overrule it.
fn allowed_minutes(content: &ContentIndex, drill: &Drill, wanted: u16) -> u16 {
    let wanted = drill.minutes.unwrap_or(wanted);
    match content
        .gate(&drill.gate)
        .and_then(|gate| gate.time_ceiling_s)
    {
        Some(ceiling_s) => wanted.min((ceiling_s / 60) as u16),
        None => wanted,
    }
}

/// Stage 4. Grind is honest work, not the session's shape: capped by blocks and
/// by minutes, and a trade is a logged debt rather than a silent skip.
fn cap_grind(content: &ContentIndex, candidates: &mut Vec<Candidate>, deferred: &mut Vec<String>) {
    let limits = &content.planner;
    let mut blocks = 0;
    let mut minutes = 0;
    let mut kept = Vec::with_capacity(candidates.len());

    for mut candidate in std::mem::take(candidates) {
        if !candidate.node.grind {
            kept.push(candidate);
            continue;
        }
        if blocks >= limits.grind_max_blocks {
            deferred.push(format!(
                "{}: the session's grind allowance was already spent",
                candidate.drill.id
            ));
            continue;
        }
        let room = limits.grind_max_minutes_per_session.saturating_sub(minutes);
        if room == 0 {
            deferred.push(format!(
                "{}: no grind minutes left in the session",
                candidate.drill.id
            ));
            continue;
        }
        if candidate.minutes > room {
            deferred.push(format!(
                "{}: trimmed to {room} minutes by the grind allowance",
                candidate.drill.id
            ));
            candidate.minutes = room;
            candidate.placed_by = Stage::GrindCap;
        }
        blocks += 1;
        minutes += candidate.minutes;
        kept.push(candidate);
    }
    *candidates = kept;
}

/// Stage 5, last and overriding. Caps and template outrank declared intent: the
/// session is cut to fit, at least one music block survives the cut, and the
/// session closes on music. What did not fit is queued and reported.
fn fit_template(
    content: &ContentIndex,
    mastery: &MasteryStore,
    ctx: PlanContext,
    candidates: Vec<Candidate>,
    deferred: &mut Vec<String>,
) -> Vec<PlannedBlock> {
    let closer = candidates
        .iter()
        .rposition(|candidate| candidate.drill.applied)
        .filter(|index| candidates[*index].minutes <= ctx.available_minutes);
    let reserved = closer.map_or(0, |index| candidates[index].minutes);

    let mut spent = 0;
    let mut blocks = Vec::new();
    let mut closing = None;
    for (index, candidate) in candidates.into_iter().enumerate() {
        if Some(index) == closer {
            closing = Some(candidate);
            continue;
        }
        if spent + candidate.minutes + reserved > ctx.available_minutes {
            deferred.push(format!(
                "{}: no room in {} minutes today",
                candidate.drill.id, ctx.available_minutes
            ));
            continue;
        }
        spent += candidate.minutes;
        blocks.push(planned(content, mastery, ctx, candidate));
    }
    if let Some(candidate) = closing {
        blocks.push(planned(content, mastery, ctx, candidate));
    }
    blocks
}

fn node_state(content: &ContentIndex, candidate: &Candidate) -> NodeState {
    NodeState {
        estimate_pct: (candidate.reading.estimate * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8,
        evidence: candidate
            .reading
            .evidence
            .round()
            .clamp(0.0, f32::from(u16::MAX)) as u16,
        overdue_pct: (candidate.reading.overdue * 100.0)
            .round()
            .clamp(0.0, f32::from(u16::MAX)) as u16,
        maturity: candidate.maturity(content.planner.maintenance_estimate),
    }
}

fn planned(
    content: &ContentIndex,
    mastery: &MasteryStore,
    ctx: PlanContext,
    candidate: Candidate,
) -> PlannedBlock {
    let new_keys = content
        .traversal
        .get(&candidate.node.id)
        .map(|quota| deal_new_keys(ctx.rng_seed, &candidate.node.id, quota));

    PlannedBlock {
        alternatives: alternatives(content, mastery, ctx.now, &candidate),
        why: Why {
            destination: destination(content, candidate.drill),
            node_state: node_state(content, &candidate),
            placed_by: candidate.placed_by,
        },
        spec: block_spec(content, &candidate),
        new_keys,
    }
}

/// What the ladder may swap to, worked out here so the state machine never has
/// to choose material: the rung below for a drill swap, and a neighbour node
/// that runs in a different mode for a mode change. Either may be absent, and
/// an absent alternative is what ends a block `Exit::Escalated`.
fn alternatives(
    content: &ContentIndex,
    mastery: &MasteryStore,
    now: DateTime<Utc>,
    candidate: &Candidate,
) -> Vec<Alternative> {
    let node = candidate.node;
    let swap = node
        .drills
        .iter()
        .filter(|id| *id != &candidate.drill.id)
        .filter_map(|id| content.drill(id))
        .find(|drill| drill.level.is_some())
        .map(|drill| (Rung::SwapDrill, node, drill));

    let mode_change = neighbours(content, node)
        .filter(|neighbour| neighbour.mode != node.mode)
        .flat_map(|neighbour| {
            neighbour
                .drills
                .iter()
                .filter_map(|id| content.drill(id))
                .map(move |drill| (neighbour, drill))
        })
        .find(|(_, drill)| drill.level.is_some())
        .map(|(neighbour, drill)| (Rung::ChangeMode, neighbour, drill));

    // The ladder's order, so a plan reads the way it will be spent.
    [mode_change, swap]
        .into_iter()
        .flatten()
        .map(|(rung, node, drill)| {
            let level = drill.level.expect("filtered to the runnable rungs");
            let swapped = Candidate {
                node,
                drill,
                level,
                minutes: candidate.minutes,
                reading: mastery.reading(&node.id, level, now),
                placed_by: Stage::Escalation,
            };
            Alternative {
                rung,
                why: Why {
                    destination: destination(content, drill),
                    node_state: node_state(content, &swapped),
                    placed_by: Stage::Escalation,
                },
                spec: block_spec(content, &swapped),
            }
        })
        .collect()
}

/// Adjacent in the graph: what this node depends on, and what depends on it.
fn neighbours<'c>(
    content: &'c ContentIndex,
    node: &'c Node,
) -> impl Iterator<Item = &'c Node> + 'c {
    node.prerequisites
        .iter()
        .filter_map(|id| content.node(id))
        .chain(
            content
                .nodes
                .values()
                .filter(|other| other.prerequisites.contains(&node.id)),
        )
}

/// The dealer for a `[traversal.<node>]` quota: how many new keys this session
/// offers, drawn from the stored seed so the same plan deals the same keys.
fn deal_new_keys(seed: u64, node: &str, quota: &Traversal) -> u8 {
    if quota.new_keys_max <= quota.new_keys_min {
        return quota.new_keys_min;
    }
    let span = u64::from(quota.new_keys_max - quota.new_keys_min) + 1;
    quota.new_keys_min + (mix(seed, node) % span) as u8
}

fn mix(seed: u64, node: &str) -> u64 {
    let mut hash = seed ^ 0xcbf2_9ce4_8422_2325;
    for byte in node.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Whatever the drill serves, the block is in the campaign's route, so the why
/// cites the campaign (principle 7).
fn destination(content: &ContentIndex, drill: &Drill) -> Option<String> {
    Some(
        drill
            .destination
            .clone()
            .unwrap_or_else(|| content.intent.destination.clone()),
    )
}

fn block_spec(content: &ContentIndex, candidate: &Candidate) -> BlockSpec {
    let (node, drill) = (candidate.node, candidate.drill);
    let gate = content
        .gate(&drill.gate)
        .expect("the parser resolved every drill's gate")
        .clone();
    BlockSpec {
        node: node.id.clone(),
        drill: drill.id.clone(),
        drill_title: node.title.clone(),
        section: drill.section.clone(),
        destination: destination(content, drill),
        // A drill parameterised by real music is music; the rest are exercises.
        kind: if drill.applied {
            ItemKind::Piece
        } else {
            ItemKind::Exercise
        },
        circle: node.circle,
        mode: node.mode,
        level: candidate.level,
        gate,
        bars: drill.bars,
        beats_per_bar: content.beats_per_bar,
        count_in_beats: content.count_in_beats,
        minutes: candidate.minutes,
    }
}

#[cfg(test)]
impl Plan {
    /// A stable plan for the state-machine tests, so what they assert is the
    /// machine rather than whatever the authored content currently says.
    pub(crate) fn fixture() -> Self {
        Self {
            blocks: vec![PlannedBlock {
                spec: BlockSpec {
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
                },
                why: Why {
                    destination: Some("Strasbourg / St. Denis".to_string()),
                    node_state: NodeState {
                        estimate_pct: 30,
                        evidence: 0,
                        overdue_pct: 0,
                        maturity: Maturity::New,
                    },
                    placed_by: Stage::Intent,
                },
                alternatives: Vec::new(),
                new_keys: None,
            }],
            rng_seed: 0,
            deferred: Vec::new(),
        }
    }

    /// The same block at l0 (decision 20). Only the rung changes: everything
    /// the clock implied — the count-in included — falls out of it.
    pub(crate) fn fixture_untimed() -> Self {
        let mut plan = Self::fixture();
        plan.blocks[0].spec.level = ParameterLevel {
            tempo_bpm: 0,
            click_level: ClickLevel::NoClick,
        };
        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::gate::Verdict;
    use crate::engine::session::Rung;
    use chrono::{TimeDelta, TimeZone};

    fn at(day: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_754_300_000, 0).unwrap() + TimeDelta::days(day)
    }

    fn context(minutes: u16) -> PlanContext {
        PlanContext {
            now: at(0),
            available_minutes: minutes,
            rng_seed: 0,
        }
    }

    fn seeded(minutes: u16) -> Plan {
        let content = ContentIndex::shipped();
        plan_from(
            content,
            &MasteryStore::seeded_from(content),
            context(minutes),
        )
    }

    fn drills(plan: &Plan) -> Vec<&str> {
        plan.blocks
            .iter()
            .map(|block| block.spec.drill.as_str())
            .collect()
    }

    fn edited(edit: impl Fn(&str) -> String) -> ContentIndex {
        ContentIndex::parse(&edit(ContentIndex::shipped_source())).expect("edited content parses")
    }

    // ── Stages 1 and 2: declared intent, back-chained ──

    #[test]
    fn the_plan_is_the_route_back_chained_from_the_declared_intent() {
        assert_eq!(
            drills(&seeded(20)),
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
        let plan = seeded(60);
        assert!(
            !plan
                .blocks
                .iter()
                .any(|block| block.spec.node == "tune-form-memory"),
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
        let plan = seeded(60);
        assert!(
            !plan
                .blocks
                .iter()
                .any(|block| block.spec.node == "micro-transcription"),
            "the away-from-the-keys rungs have no tempo, so this loop cannot host them"
        );
        assert!(plan
            .deferred
            .iter()
            .any(|note| note.contains("micro-transcription")));
    }

    #[test]
    fn a_target_nothing_can_count_is_reported_rather_than_turned_into_a_block() {
        let plan = seeded(60);

        assert!(
            plan.deferred
                .iter()
                .any(|note| note.contains("Make the last A section sing")),
            "the opaque target passes through unscored (decision 19): {:?}",
            plan.deferred
        );
    }

    // ── Stage 3: maturity sizes the block, overdue reorders the session ──

    #[test]
    fn the_warm_up_is_the_material_already_owned() {
        let first = seeded(20).blocks.first().cloned().expect("a warm-up block");

        assert_eq!(
            first.spec.node, "shells-ii-v-i",
            "0.7 seeded: maintenance, not learning"
        );
        assert_eq!(
            first.spec.minutes, 4,
            "matured material wants a shorter pass (spec §5 stage 3)"
        );
    }

    #[test]
    fn an_acquisition_block_gets_the_longer_uninterrupted_run() {
        let block = seeded(20)
            .blocks
            .into_iter()
            .find(|block| block.spec.node == "chord-tone-targeting")
            .expect("the frontier of the campaign");

        assert_eq!(
            block.spec.minutes, 6,
            "low evidence wants the longer block, and 6 minutes is the ceiling"
        );
    }

    #[test]
    fn live_evidence_outranks_the_content_seed_when_they_disagree() {
        let content = ContentIndex::shipped();
        let mut mastery = MasteryStore::seeded_from(content);
        let level = ParameterLevel {
            tempo_bpm: 100,
            click_level: ClickLevel::TwoAndFour,
        };
        for day in 0..12 {
            mastery.record("shells-ii-v-i", level, Verdict::Missed, at(day));
        }

        let plan = plan_from(content, &mastery, context(30));
        let shells = plan
            .blocks
            .iter()
            .find(|block| block.spec.node == "shells-ii-v-i")
            .expect("still in the route");

        assert_eq!(
            shells.spec.minutes, 6,
            "a seed said this was owned; the hands say it is being learned again"
        );
        assert_eq!(shells.why.node_state.maturity, Maturity::Acquiring);
    }

    #[test]
    fn overdue_maintenance_comes_ahead_of_new_keys() {
        let content = ContentIndex::shipped();
        let mut mastery = MasteryStore::seeded_from(content);
        // The last node in the route, so route order alone would leave it last.
        let phrase = ParameterLevel {
            tempo_bpm: 120,
            click_level: ClickLevel::TwoAndFour,
        };
        for day in 0..12 {
            mastery.record("phrase-transposition", phrase, Verdict::Clean, at(day));
        }

        let plan = plan_from(
            content,
            &mastery,
            PlanContext {
                now: at(60),
                ..context(30)
            },
        );
        let overdue = plan
            .blocks
            .iter()
            .position(|block| block.spec.node == "phrase-transposition")
            .expect("the overdue node");
        let new_ground = plan
            .blocks
            .iter()
            .position(|block| block.spec.node == "chord-tone-targeting")
            .expect("a node with nothing behind it");
        let owned = plan
            .blocks
            .iter()
            .position(|block| block.spec.node == "shells-ii-v-i")
            .expect("the warm-up, owned but not yet due");

        assert!(
            overdue < new_ground,
            "maintenance that is due outranks new keys (journey 7): {:?}",
            drills(&plan)
        );
        assert!(
            overdue < owned,
            "and outranks maintenance that is not due yet, which is the whole \
             difference between due and merely owned: {:?}",
            drills(&plan)
        );
        assert!(
            plan.blocks[overdue].why.node_state.overdue_pct >= 100,
            "and the why says so: {}",
            plan.blocks[overdue].why.node_state.overdue_pct
        );
        assert_eq!(
            plan.blocks[overdue].why_line(),
            "Phrase transposition is due back for Strasbourg / St. Denis.",
            "which is what the why line says out loud"
        );
    }

    // ── Stage 4: the grind cap ──

    #[test]
    fn a_grind_block_is_trimmed_to_the_sessions_grind_allowance() {
        let content = edited(|source| {
            source.replace(
                "grind_max_minutes_per_session = 10",
                "grind_max_minutes_per_session = 3",
            )
        });
        let plan = plan_from(&content, &MasteryStore::seeded_from(&content), context(30));
        let grind = plan
            .blocks
            .iter()
            .find(|block| block.spec.node == "rootless-a-b")
            .expect("the grind-tagged node");

        assert_eq!(grind.spec.minutes, 3, "capped, not dropped");
        assert!(
            plan.deferred.iter().any(|note| note.contains("trimmed")),
            "a grind trade is a logged debt, not a silent skip: {:?}",
            plan.deferred
        );
    }

    #[test]
    fn a_second_grind_block_waits_for_the_next_session() {
        let content = edited(|source| {
            source.replace(
                "mastery_estimate = 0.35\nmastery_band = \"low\"",
                "mastery_estimate = 0.35\nmastery_band = \"low\"\ngrind = true",
            )
        });
        let plan = plan_from(&content, &MasteryStore::seeded_from(&content), context(60));

        let grind_blocks = plan
            .blocks
            .iter()
            .filter(|block| {
                content
                    .node(&block.spec.node)
                    .is_some_and(|node| node.grind)
            })
            .count();
        assert_eq!(grind_blocks, 1, "grind_max_blocks = 1");
        assert!(
            plan.deferred
                .iter()
                .any(|note| note.contains("grind allowance")),
            "{:?}",
            plan.deferred
        );
    }

    // ── Stage 5: the template, last and overriding ──

    #[test]
    fn an_over_full_campaign_is_sequenced_rather_than_silently_dropped() {
        let plan = seeded(12);
        let minutes: u16 = plan.blocks.iter().map(|block| block.spec.minutes).sum();

        assert!(minutes <= 12, "{minutes} minutes planned into 12");
        assert!(
            !plan.deferred.is_empty(),
            "what did not fit is queued for the next plan"
        );
        assert!(
            plan.deferred
                .iter()
                .any(|note| note.contains("no room in 12 minutes")),
            "{:?}",
            plan.deferred
        );
    }

    #[test]
    fn the_session_closes_on_music() {
        let plan = seeded(20);

        assert_eq!(
            plan.blocks.last().map(|block| block.spec.drill.as_str()),
            Some("rootless-under-melody"),
            "an applied block is reordered to last, whatever the route order was"
        );
    }

    #[test]
    fn the_music_block_survives_a_tight_session_because_the_template_outranks_intent() {
        let plan = seeded(10);
        let minutes: u16 = plan.blocks.iter().map(|block| block.spec.minutes).sum();

        assert!(
            plan.blocks
                .iter()
                .any(|block| block.spec.kind == ItemKind::Piece),
            "at least one new-or-applied music block, even when the budget bites: {:?}",
            drills(&plan)
        );
        assert_eq!(
            plan.blocks.last().map(|block| block.spec.kind.clone()),
            Some(ItemKind::Piece)
        );
        assert!(
            minutes <= 10,
            "and its minutes are reserved out of the budget rather than added on \
             top of it: {minutes} planned into 10"
        );
    }

    // ── The why, written by the stage that placed the block ──

    #[test]
    fn every_block_carries_a_why_that_names_its_destination() {
        for block in seeded(30).blocks {
            assert_eq!(
                block.why.destination.as_deref(),
                Some("Strasbourg / St. Denis"),
                "{} is in the campaign's route, so the why cites the campaign",
                block.spec.drill
            );
            assert!(
                !block.why_line().is_empty(),
                "{} has nothing to say for itself",
                block.spec.drill
            );
        }
    }

    #[test]
    fn the_why_names_the_stage_that_placed_the_block() {
        let plan = seeded(30);
        let placed = |node: &str| {
            plan.blocks
                .iter()
                .find(|block| block.spec.node == node)
                .map(|block| block.why.placed_by)
        };

        assert_eq!(
            placed("chord-tone-targeting"),
            Some(Stage::Intent),
            "a declared target is placed by resolving the intent"
        );
        assert_eq!(
            placed("shells-ii-v-i"),
            Some(Stage::BackChain),
            "a prerequisite is placed by back-chaining"
        );
    }

    #[test]
    fn a_why_line_reads_as_a_sentence_a_musician_would_say() {
        let plan = seeded(20);

        assert_eq!(
            plan.blocks
                .iter()
                .find(|block| block.spec.node == "shells-ii-v-i")
                .map(PlannedBlock::why_line),
            Some(
                "Shell voicings comes first: the route runs through it for \
                 Strasbourg / St. Denis."
                    .to_string()
            )
        );
    }

    // ── What the ladder's third and fourth rungs may draw on (#1182) ──

    #[test]
    fn a_block_carries_a_swap_to_another_rung_of_its_own_node() {
        let block = seeded(30)
            .blocks
            .into_iter()
            .find(|block| block.spec.drill == "rootless-under-melody")
            .expect("the campaign's music block");

        let swap = block
            .alternatives
            .iter()
            .find(|alternative| alternative.rung == Rung::SwapDrill)
            .expect("somewhere for the fourth rung to go");
        assert_eq!(
            swap.spec.node, "rootless-a-b",
            "the same material, a different rung of it"
        );
        assert_ne!(swap.spec.drill, block.spec.drill);
        assert_eq!(
            swap.spec.drill, "rootless-one-key",
            "the easiest rung of the ladder below the one that is not landing"
        );
    }

    #[test]
    fn todays_content_offers_no_way_off_the_keys_so_no_mode_change_is_promised() {
        for block in seeded(30).blocks {
            assert!(
                !block
                    .alternatives
                    .iter()
                    .any(|alternative| alternative.rung == Rung::ChangeMode),
                "{}: every away-from-the-keys rung in the content is clickless, so \
                 a mode change here would be a promise the loop cannot keep",
                block.spec.drill
            );
        }
    }

    #[test]
    fn a_mode_change_is_offered_when_a_neighbour_node_can_be_run_off_the_keys() {
        // Two edits, because today's content offers no mode change at all: give
        // micro-transcription a rung with a click, and a mode that differs from
        // the node it is a prerequisite of.
        let content = edited(|source| {
            source
                .replace(
                    "judge = \"self-confirmed\"\nmax_listens = 5",
                    "clean_passes = 1\ntempo_bpm = 70\nclick_level = \"l3\"",
                )
                .replace(
                    "[nodes.micro-transcription]\ntitle = \"Micro-transcription\"\ncircle = \"head\"\nmode = \"keys-to-away\"",
                    "[nodes.micro-transcription]\ntitle = \"Micro-transcription\"\ncircle = \"head\"\nmode = \"away\"",
                )
        });
        let plan = plan_from(&content, &MasteryStore::seeded_from(&content), context(60));

        let phrase = plan
            .blocks
            .iter()
            .find(|block| block.spec.node == "phrase-transposition")
            .expect("the node whose prerequisite is micro-transcription");
        let mode_change = phrase
            .alternatives
            .iter()
            .find(|alternative| alternative.rung == Rung::ChangeMode)
            .expect("a neighbour with a different mode and a click to run against");

        assert_eq!(mode_change.spec.node, "micro-transcription");
        assert_ne!(mode_change.spec.mode, phrase.spec.mode);
    }

    // ── The seeded dealer, and replay ──

    #[test]
    fn a_traversal_quota_deals_new_keys_inside_the_range_the_content_authored() {
        let block = seeded(30)
            .blocks
            .into_iter()
            .find(|block| block.spec.node == "rootless-a-b")
            .expect("the node with a traversal quota");

        let keys = block.new_keys.expect("a dealt quota");
        assert!(
            (2..=3).contains(&keys),
            "content authors 2 to 3 new keys a session: {keys}"
        );
        assert!(
            block.why_line().contains("new keys"),
            "and the why says how many: {}",
            block.why_line()
        );
    }

    #[test]
    fn a_block_with_no_traversal_quota_deals_no_keys() {
        let block = seeded(30)
            .blocks
            .into_iter()
            .find(|block| block.spec.node == "shells-ii-v-i")
            .unwrap();

        assert_eq!(block.new_keys, None);
    }

    #[test]
    fn the_stored_seed_replays_the_session_and_a_different_seed_may_deal_otherwise() {
        let content = ContentIndex::shipped();
        let mastery = MasteryStore::seeded_from(content);
        let again = plan_from(content, &mastery, context(30));

        assert_eq!(
            plan_from(content, &mastery, context(30)),
            again,
            "the same state and context plan the same session, every time"
        );

        let dealt = |seed: u64| {
            plan_from(
                content,
                &mastery,
                PlanContext {
                    rng_seed: seed,
                    ..context(30)
                },
            )
            .blocks
            .into_iter()
            .find(|block| block.spec.node == "rootless-a-b")
            .and_then(|block| block.new_keys)
        };
        assert!(
            (0..64).any(|seed| dealt(seed) != dealt(0)),
            "the dealer is seeded, so some seed deals a different quota"
        );
    }

    #[test]
    fn a_block_carries_the_gate_and_level_the_file_authored() {
        let block = seeded(20)
            .blocks
            .into_iter()
            .find(|block| block.spec.drill == "rootless-under-melody")
            .unwrap();

        assert_eq!(block.spec.drill_title, "Rootless voicings");
        assert_eq!(block.spec.section.as_deref(), Some("A section"));
        assert_eq!(
            block.spec.level.tempo_bpm, 80,
            "the file's tempo, not a Rust one"
        );
        assert_eq!(block.spec.level.click_level, ClickLevel::EveryBeat);
        assert_eq!(
            block.spec.gate.requirement,
            Requirement::CleanPasses {
                count: 2,
                consecutive: false
            }
        );
        assert_eq!(
            (block.spec.circle, block.spec.mode),
            (Circle::Hands, Mode::Keys)
        );
        assert_eq!(block.spec.beats_per_bar, 4);
        assert_eq!(block.spec.count_in_beats, 4);
    }
}

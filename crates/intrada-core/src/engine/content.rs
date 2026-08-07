//! The authored content (`content/gates.toml`), parsed once at startup so the
//! engine does no I/O and the planner stays pure (spec §1). Validation is the
//! `validation.rs` idiom pointed at a file: the schema version must match, every
//! reference must resolve because an unreachable gate is otherwise invisible
//! (spec §8), and an unknown field is an error on every table that carries a
//! reference. The three config bags are the stated exception, for the reason
//! given above `RawDefaults`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;
use thiserror::Error;

use super::gate::{ClickLevel, GateCriteria, Judge, Requirement};
use super::plan::{Circle, Mode, ParameterLevel};
use super::session::Rung;

const SHIPPED: &str = include_str!("../../../../content/gates.toml");
const SCHEMA_VERSION: &str = "0.2";

/// Every table the file may carry. Checked by name because a mistyped section
/// is an invisible section: `[gatess.…]` would otherwise take a gate out of
/// the graph in silence. Some are authored for the reader rather than the
/// engine, which is why they are a list here and not fields below.
const SECTIONS: &[&str] = &[
    "schema_version",
    "defaults",
    "click_levels",
    "transport_tiers",
    "feedback",
    "escalation",
    "planner",
    "intent",
    "traversal",
    "nodes",
    "drills",
    "gates",
];

#[derive(Error, Debug, PartialEq)]
pub enum ContentError {
    #[error("authored content failed to parse: {0}")]
    Malformed(String),
    #[error("authored content is schema_version {found}, and this engine reads {SCHEMA_VERSION}")]
    SchemaVersion { found: String },
    #[error("{referrer} refers to {kind} \"{id}\", which nothing authors")]
    Dangling {
        referrer: String,
        kind: &'static str,
        id: String,
    },
    #[error("gate \"{0}\" is on node \"{1}\", but the ladder listing it belongs to \"{2}\"")]
    NodeMismatch(String, String, String),
    #[error("gate \"{0}\" is unreachable: no drill runs it")]
    UnreachableGate(String),
    #[error("gate \"{0}\" is l0 and names a tempo; acquisition happens out of time")]
    TimedAcquisitionGate(String),
    #[error("gate \"{0}\" states no requirement anything can count")]
    UncountableGate(String),
    #[error("node \"{0}\" is a stub and a stub has no drills")]
    StubWithDrills(String),
    #[error("\"{0}\" is not a section this file has: a mistyped one is invisible")]
    UnknownSection(String),
}

// ── What the engine reads ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub circle: Circle,
    pub mode: Mode,
    pub prerequisites: Vec<String>,
    /// The ladder, in the order the file lists it. Empty for a stub.
    pub drills: Vec<String>,
    /// Content's seeded `(estimate, band)`, which spec §2 turns into a prior:
    /// the estimate is what the planner reads until the live mastery store
    /// lands, and the band is the pseudo-count strength that store will use
    /// (#1148). `None` on a stub, which has nothing to estimate.
    pub estimate: f32,
    pub band: Option<Band>,
    pub grind: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Drill {
    pub id: String,
    pub node: String,
    pub gate: String,
    pub section: Option<String>,
    pub destination: Option<String>,
    /// A drill parameterised by real music. The template needs one and closes
    /// on one (spec §5 stage 5).
    pub applied: bool,
    pub bars: u16,
    /// What the drill is worth of the session, where the content says so.
    pub minutes: Option<u16>,
    /// `None` where the drill has no click to run against (an away-from-the-keys
    /// rung), which is what stops the loop scheduling it.
    pub level: Option<ParameterLevel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Traversal {
    pub goal: String,
    pub new_keys_min: u8,
    pub new_keys_max: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Escalation {
    pub consecutive_fail_trigger: u8,
    pub ladder: Vec<Rung>,
    pub tempo_down_pct: u16,
    pub tempo_floor_bpm: u16,
    pub gate_open_hold_s: u32,
    pub glance_hold_s: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannerLimits {
    pub maintenance_estimate: f32,
    pub acquisition_minutes: u16,
    pub maintenance_minutes: u16,
    pub grind_max_minutes_per_session: u16,
    pub grind_max_blocks: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentIndex {
    pub session_minutes: u16,
    pub beats_per_bar: u8,
    pub count_in_beats: u8,
    pub escalation: Escalation,
    pub planner: PlannerLimits,
    pub intent: Intent,
    pub traversal: BTreeMap<String, Traversal>,
    pub nodes: BTreeMap<String, Node>,
    pub drills: BTreeMap<String, Drill>,
    pub gates: BTreeMap<String, GateCriteria>,
}

impl ContentIndex {
    /// The content compiled into the binary. Parsed once: the file is
    /// `include_str!`-embedded, so a failure here is a build-time bug that
    /// `the_shipped_content_parses` catches, not a runtime condition.
    pub fn shipped() -> &'static ContentIndex {
        static SHIPPED_INDEX: OnceLock<ContentIndex> = OnceLock::new();
        SHIPPED_INDEX.get_or_init(|| {
            ContentIndex::parse(SHIPPED).expect("shipped content must parse (see content.rs tests)")
        })
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn drill(&self, id: &str) -> Option<&Drill> {
        self.drills.get(id)
    }

    pub fn gate(&self, id: &str) -> Option<&GateCriteria> {
        self.gates.get(id)
    }

    /// The embedded file, so a test elsewhere in `engine/` can plan against an
    /// edited copy of the real content rather than a hand-built fixture.
    #[cfg(test)]
    pub(crate) fn shipped_source() -> &'static str {
        SHIPPED
    }

    pub fn parse(source: &str) -> Result<ContentIndex, ContentError> {
        let sections: toml::Table =
            toml::from_str(source).map_err(|error| ContentError::Malformed(error.to_string()))?;
        for name in sections.keys() {
            if !SECTIONS.contains(&name.as_str()) {
                return Err(ContentError::UnknownSection(name.clone()));
            }
        }
        let raw: RawContent =
            toml::from_str(source).map_err(|error| ContentError::Malformed(error.to_string()))?;
        if raw.schema_version != SCHEMA_VERSION {
            return Err(ContentError::SchemaVersion {
                found: raw.schema_version,
            });
        }
        raw.resolve()
    }
}

// ── The file ────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct RawContent {
    schema_version: String,
    defaults: RawDefaults,
    escalation: RawEscalation,
    planner: RawPlanner,
    intent: Intent,
    traversal: BTreeMap<String, Traversal2>,
    nodes: BTreeMap<String, RawNode>,
    drills: BTreeMap<String, RawDrill>,
    gates: BTreeMap<String, RawGate>,
    /// The ladder and the tiers every gate and default has to name: resolved
    /// below, so a click level or a transport nothing authors fails the parse.
    click_levels: BTreeMap<String, String>,
    transport_tiers: BTreeMap<String, Vec<String>>,
}

// The three config bags below deliberately allow unknown fields, unlike every
// table that carries a reference. They hold authored values the engine has no
// reader for yet (what "clean" means, the name-the-wall cap, the grind caps),
// and dropping them from the file to satisfy strictness would lose content the
// human practises from. Every field the engine does read here is required, so a
// typo is still a parse failure, and `SECTIONS` still catches a mistyped table.
#[derive(Deserialize, Debug)]
struct RawDefaults {
    time_ceiling_min: u32,
    judge: JudgeToken,
    transport: String,
    session_minutes: u16,
    bars: u16,
    beats_per_bar: u8,
    count_in_beats: u8,
}

#[derive(Deserialize, Debug)]
struct RawEscalation {
    consecutive_fail_trigger: u8,
    ladder: Vec<RungToken>,
    tempo_down_pct: u16,
    tempo_floor_bpm: u16,
    gate_open_hold_s: u32,
    glance_hold_s: u32,
}

#[derive(Deserialize, Debug)]
struct RawPlanner {
    maintenance_estimate: f32,
    acquisition_minutes: u16,
    maintenance_minutes: u16,
    grind_max_minutes_per_session: u16,
    grind_max_blocks: u8,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub goal: String,
    pub campaign: String,
    pub destination: String,
    pub targets: Vec<String>,
    pub opaque_targets: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Traversal2 {
    goal: String,
    new_keys_per_session_min: u8,
    new_keys_per_session_max: u8,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RawNode {
    title: String,
    circle: CircleToken,
    mode: ModeToken,
    #[serde(default)]
    prerequisites: Vec<String>,
    #[serde(default)]
    drills: Vec<String>,
    #[serde(default)]
    mastery_estimate: f32,
    #[serde(default)]
    mastery_band: Option<BandToken>,
    #[serde(default)]
    grind: bool,
    #[serde(default)]
    stub: bool,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RawDrill {
    gate: String,
    #[serde(default)]
    section: Option<String>,
    #[serde(default)]
    destination: Option<String>,
    #[serde(default)]
    applied: bool,
    #[serde(default)]
    bars: Option<u16>,
    #[serde(default)]
    minutes: Option<u16>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RawGate {
    node: String,
    criterion: String,
    #[serde(default)]
    judge: Option<JudgeToken>,
    #[serde(default)]
    clean_passes: Option<u8>,
    #[serde(default)]
    consecutive: bool,
    #[serde(default)]
    keys_required: Option<u8>,
    #[serde(default)]
    keys_chained_min: Option<u8>,
    #[serde(default)]
    first_attempt: bool,
    #[serde(default)]
    max_listens: Option<u8>,
    #[serde(default)]
    tempo_bpm: Option<u16>,
    #[serde(default)]
    click_level: Option<ClickLevelToken>,
    #[serde(default)]
    time_ceiling_min: Option<u32>,
}

// Tokens, so the bridge types carry no serde attributes of their own: a rename
// that only makes sense for a self-describing format has no business on a type
// that also crosses the bincode wire (#846).
/// Seed strength, spec §2's `low 2 / medium 5 / high 10` pseudo-counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Low,
    Medium,
    High,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum BandToken {
    Low,
    Medium,
    High,
}

impl From<BandToken> for Band {
    fn from(token: BandToken) -> Self {
        match token {
            BandToken::Low => Band::Low,
            BandToken::Medium => Band::Medium,
            BandToken::High => Band::High,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum CircleToken {
    Head,
    Hands,
    Bridge,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum ModeToken {
    Keys,
    Away,
    KeysToAway,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum JudgeToken {
    Machine,
    TapVerdict,
    SelfConfirmed,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ClickLevelToken {
    L0,
    L1,
    L2,
    L3,
    L4,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum RungToken {
    TempoDown,
    ShrinkScope,
    ChangeMode,
    SwapDrill,
}

impl From<CircleToken> for Circle {
    fn from(token: CircleToken) -> Self {
        match token {
            CircleToken::Head => Circle::Head,
            CircleToken::Hands => Circle::Hands,
            CircleToken::Bridge => Circle::Bridge,
        }
    }
}

impl From<ModeToken> for Mode {
    fn from(token: ModeToken) -> Self {
        match token {
            ModeToken::Keys => Mode::Keys,
            ModeToken::Away => Mode::Away,
            ModeToken::KeysToAway => Mode::KeysToAway,
        }
    }
}

impl From<JudgeToken> for Judge {
    fn from(token: JudgeToken) -> Self {
        match token {
            JudgeToken::Machine => Judge::Machine,
            JudgeToken::TapVerdict => Judge::TapVerdict,
            JudgeToken::SelfConfirmed => Judge::SelfConfirmed,
        }
    }
}

impl ClickLevelToken {
    fn key(self) -> &'static str {
        match self {
            ClickLevelToken::L0 => "l0",
            ClickLevelToken::L1 => "l1",
            ClickLevelToken::L2 => "l2",
            ClickLevelToken::L3 => "l3",
            ClickLevelToken::L4 => "l4",
        }
    }
}

impl From<ClickLevelToken> for ClickLevel {
    fn from(token: ClickLevelToken) -> Self {
        match token {
            ClickLevelToken::L0 => ClickLevel::NoClick,
            ClickLevelToken::L1 => ClickLevel::EveryBeat,
            ClickLevelToken::L2 => ClickLevel::TwoAndFour,
            ClickLevelToken::L3 => ClickLevel::BarDownbeat,
            ClickLevelToken::L4 => ClickLevel::EveryOtherBar,
        }
    }
}

impl From<RungToken> for Rung {
    fn from(token: RungToken) -> Self {
        match token {
            RungToken::TempoDown => Rung::TempoDown,
            RungToken::ShrinkScope => Rung::ShrinkScope,
            RungToken::ChangeMode => Rung::ChangeMode,
            RungToken::SwapDrill => Rung::SwapDrill,
        }
    }
}

// ── Resolution: every reference, or a startup failure ───────────────────

impl RawContent {
    fn resolve(self) -> Result<ContentIndex, ContentError> {
        let dangling = |referrer: &str, kind: &'static str, id: &str| ContentError::Dangling {
            referrer: referrer.to_string(),
            kind,
            id: id.to_string(),
        };

        let mut nodes = BTreeMap::new();
        let mut drills: BTreeMap<String, Drill> = BTreeMap::new();
        let mut gates = BTreeMap::new();

        for (id, raw) in &self.gates {
            if !self.nodes.contains_key(&raw.node) {
                return Err(dangling(&format!("gate \"{id}\""), "node", &raw.node));
            }
            gates.insert(
                id.clone(),
                GateCriteria {
                    id: id.clone(),
                    node: raw.node.clone(),
                    criterion: raw.criterion.clone(),
                    requirement: raw.requirement(id)?,
                    judge: raw.judge.unwrap_or(self.defaults.judge).into(),
                    time_ceiling_s: Some(
                        raw.time_ceiling_min
                            .unwrap_or(self.defaults.time_ceiling_min)
                            * 60,
                    ),
                },
            );
        }

        for (id, raw) in &self.nodes {
            if raw.stub && !raw.drills.is_empty() {
                return Err(ContentError::StubWithDrills(id.clone()));
            }
            for prerequisite in &raw.prerequisites {
                if !self.nodes.contains_key(prerequisite) {
                    return Err(dangling(&format!("node \"{id}\""), "node", prerequisite));
                }
            }
            for drill_id in &raw.drills {
                let Some(raw_drill) = self.drills.get(drill_id) else {
                    return Err(dangling(&format!("node \"{id}\""), "drill", drill_id));
                };
                let Some(gate) = gates.get(&raw_drill.gate) else {
                    return Err(dangling(
                        &format!("drill \"{drill_id}\""),
                        "gate",
                        &raw_drill.gate,
                    ));
                };
                if &gate.node != id {
                    return Err(ContentError::NodeMismatch(
                        gate.id.clone(),
                        gate.node.clone(),
                        id.clone(),
                    ));
                }
                let raw_gate = &self.gates[&raw_drill.gate];
                drills.insert(
                    drill_id.clone(),
                    Drill {
                        id: drill_id.clone(),
                        node: id.clone(),
                        gate: raw_drill.gate.clone(),
                        section: raw_drill.section.clone(),
                        destination: raw_drill.destination.clone(),
                        applied: raw_drill.applied,
                        bars: raw_drill.bars.unwrap_or(self.defaults.bars),
                        minutes: raw_drill.minutes,
                        level: raw_gate.level(&raw_drill.gate)?,
                    },
                );
            }
            nodes.insert(
                id.clone(),
                Node {
                    id: id.clone(),
                    title: raw.title.clone(),
                    circle: raw.circle.into(),
                    mode: raw.mode.into(),
                    prerequisites: raw.prerequisites.clone(),
                    drills: raw.drills.clone(),
                    estimate: raw.mastery_estimate,
                    band: raw.mastery_band.map(Band::from),
                    grind: raw.grind,
                },
            );
        }

        for id in self.drills.keys() {
            if !drills.contains_key(id) {
                return Err(dangling("no node's ladder", "drill", id));
            }
        }
        for id in gates.keys() {
            if !drills.values().any(|drill| &drill.gate == id) {
                return Err(ContentError::UnreachableGate(id.clone()));
            }
        }
        // A target names a node or the drill it wants (content/intent.md states
        // both kinds), and either way it has to resolve.
        if !self.transport_tiers.contains_key(&self.defaults.transport) {
            return Err(dangling(
                "the default transport",
                "transport tier",
                &self.defaults.transport,
            ));
        }
        for (id, raw) in &self.gates {
            if let Some(level) = raw.click_level {
                if !self.click_levels.contains_key(level.key()) {
                    return Err(dangling(
                        &format!("gate \"{id}\""),
                        "click level",
                        level.key(),
                    ));
                }
            }
        }
        for target in &self.intent.targets {
            if !nodes.contains_key(target) && !drills.contains_key(target) {
                return Err(dangling("the declared intent", "node or drill", target));
            }
        }
        let mut traversal = BTreeMap::new();
        for (node, quota) in self.traversal {
            if !nodes.contains_key(&node) {
                return Err(dangling("a traversal quota", "node", &node));
            }
            traversal.insert(
                node,
                Traversal {
                    goal: quota.goal,
                    new_keys_min: quota.new_keys_per_session_min,
                    new_keys_max: quota.new_keys_per_session_max,
                },
            );
        }

        Ok(ContentIndex {
            session_minutes: self.defaults.session_minutes,
            beats_per_bar: self.defaults.beats_per_bar,
            count_in_beats: self.defaults.count_in_beats,
            escalation: Escalation {
                consecutive_fail_trigger: self.escalation.consecutive_fail_trigger,
                ladder: self.escalation.ladder.into_iter().map(Rung::from).collect(),
                tempo_down_pct: self.escalation.tempo_down_pct,
                tempo_floor_bpm: self.escalation.tempo_floor_bpm,
                gate_open_hold_s: self.escalation.gate_open_hold_s,
                glance_hold_s: self.escalation.glance_hold_s,
            },
            planner: PlannerLimits {
                maintenance_estimate: self.planner.maintenance_estimate,
                acquisition_minutes: self.planner.acquisition_minutes,
                maintenance_minutes: self.planner.maintenance_minutes,
                grind_max_minutes_per_session: self.planner.grind_max_minutes_per_session,
                grind_max_blocks: self.planner.grind_max_blocks,
            },
            intent: self.intent,
            traversal,
            nodes,
            drills,
            gates,
        })
    }
}

impl RawGate {
    /// The rung this gate is played at, where the file states a runnable one.
    /// An l0 gate carries no `tempo_bpm` by construction (decision 20), so one
    /// that names a tempo states two contradictory things and fails rather than
    /// having one of them quietly win.
    fn level(&self, id: &str) -> Result<Option<ParameterLevel>, ContentError> {
        match (self.click_level, self.tempo_bpm) {
            (Some(ClickLevelToken::L0), Some(_)) => {
                Err(ContentError::TimedAcquisitionGate(id.to_string()))
            }
            (Some(ClickLevelToken::L0), None) => Ok(Some(ParameterLevel {
                tempo_bpm: 0,
                click_level: ClickLevel::NoClick,
            })),
            (Some(click_level), Some(tempo_bpm)) => Ok(Some(ParameterLevel {
                tempo_bpm,
                click_level: click_level.into(),
            })),
            // A rung the loop cannot run, which the planner defers.
            (Some(_), None) | (None, _) => Ok(None),
        }
    }

    /// One resolution order, so an unrepresentable gate fails to parse rather
    /// than silently losing the field that made it what it is (spec §8).
    fn requirement(&self, id: &str) -> Result<Requirement, ContentError> {
        if matches!(self.judge, Some(JudgeToken::SelfConfirmed)) {
            return Ok(Requirement::SelfConfirmed {
                max_listens: self.max_listens,
                first_attempt: self.first_attempt,
            });
        }
        if let Some(min_keys) = self.keys_chained_min {
            return Ok(Requirement::Chained { min_keys });
        }
        if let Some(keys_required) = self.keys_required {
            return Ok(Requirement::KeyCoverage {
                keys_required,
                per_key_passes: self.clean_passes.unwrap_or(1),
                first_attempt: self.first_attempt,
            });
        }
        match self.clean_passes {
            Some(count) => Ok(Requirement::CleanPasses {
                count,
                consecutive: self.consecutive,
            }),
            None => Err(ContentError::UncountableGate(id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_content_parses() {
        let content = ContentIndex::shipped();
        assert_eq!(content.nodes.len(), 15, "5 authored nodes and 10 stubs");
        assert_eq!(
            content.gates.len(),
            17,
            "18 authored criteria less rootless-traversal, which was never a gate"
        );
        assert_eq!(content.drills.len(), 17, "one drill per gate");
    }

    #[test]
    fn a_node_carries_its_fluency_frame_tags_and_its_ladder() {
        let content = ContentIndex::shipped();
        let node = content.node("rootless-a-b").expect("an authored node");

        assert_eq!(node.title, "Rootless voicings");
        assert_eq!((node.circle, node.mode), (Circle::Hands, Mode::Keys));
        assert_eq!(node.prerequisites, vec!["shells-ii-v-i".to_string()]);
        assert!(node.grind, "the middle keys are grind (content/nodes.md)");
        assert_eq!(
            (node.estimate, node.band),
            (0.3, Some(Band::Low)),
            "the seed and the strength #1148 will read it at"
        );
        assert_eq!(
            node.drills.first().map(String::as_str),
            Some("rootless-one-key"),
            "the ladder runs in the order the file lists, not alphabetically"
        );
    }

    #[test]
    fn a_counted_gate_becomes_a_counted_requirement() {
        let gate = ContentIndex::shipped()
            .gate("rootless-under-melody")
            .expect("an authored gate");

        assert_eq!(
            gate.requirement,
            Requirement::CleanPasses {
                count: 2,
                consecutive: false
            }
        );
        assert_eq!(gate.judge, Judge::TapVerdict, "the [defaults] judge");
        assert_eq!(gate.time_ceiling_s, Some(360), "6 minutes, from [defaults]");
    }

    #[test]
    fn a_consecutive_gate_keeps_its_run() {
        let gate = ContentIndex::shipped().gate("targeting-approach").unwrap();
        assert_eq!(
            gate.requirement,
            Requirement::CleanPasses {
                count: 4,
                consecutive: true
            }
        );
    }

    #[test]
    fn keys_required_becomes_key_coverage() {
        let content = ContentIndex::shipped();
        assert_eq!(
            content.gate("rootless-one-key").unwrap().requirement,
            Requirement::KeyCoverage {
                keys_required: 12,
                per_key_passes: 3,
                first_attempt: false
            }
        );
        assert_eq!(
            content.gate("phrase-random-key").unwrap().requirement,
            Requirement::KeyCoverage {
                keys_required: 12,
                per_key_passes: 1,
                first_attempt: true
            },
            "a dealt key with no pass count is one clean first attempt each"
        );
    }

    #[test]
    fn a_chained_gate_becomes_a_chained_requirement() {
        assert_eq!(
            ContentIndex::shipped()
                .gate("phrase-cycle")
                .unwrap()
                .requirement,
            Requirement::Chained { min_keys: 4 }
        );
    }

    // ── l0, the acquisition rung (decision 20) ──

    /// The shells cycle gate, rewritten as the clickless rung it would be.
    fn shells_cycle_at_l0(tempo: &str) -> String {
        SHIPPED.replace(
            "criterion = \"ii-V-I shells through the full cycle of fourths, no stops\"\n\
             clean_passes = 1\n\
             tempo_bpm = 100\n\
             click_level = \"l2\"",
            &format!(
                "criterion = \"ii-V-I shells through the full cycle of fourths, no stops\"\n\
                 clean_passes = 1\n{tempo}click_level = \"l0\""
            ),
        )
    }

    #[test]
    fn an_l0_gate_is_runnable_without_a_tempo() {
        let content = ContentIndex::parse(&shells_cycle_at_l0("")).expect("an l0 gate parses");

        assert_eq!(
            content.drill("shells-cycle").expect("the drill").level,
            Some(ParameterLevel {
                tempo_bpm: 0,
                click_level: ClickLevel::NoClick,
            }),
            "no click and no tempo is a rung the loop can run, not a rung it \
             has to skip for want of a click"
        );
    }

    #[test]
    fn an_l0_gate_that_names_a_tempo_fails_to_parse() {
        assert_eq!(
            ContentIndex::parse(&shells_cycle_at_l0("tempo_bpm = 100\n")),
            Err(ContentError::TimedAcquisitionGate(
                "shells-cycle".to_string()
            )),
            "a tempo at l0 states two contradictory things, and a gate that \
             cannot be represented fails rather than losing one of them"
        );
    }

    #[test]
    fn a_self_confirmed_gate_never_counts_as_a_measured_one() {
        let gate = ContentIndex::shipped().gate("transcription-entry").unwrap();
        assert_eq!(
            gate.requirement,
            Requirement::SelfConfirmed {
                max_listens: Some(5),
                first_attempt: false
            }
        );
        assert_eq!(
            gate.judge,
            Judge::SelfConfirmed,
            "decisions 3 and 13: this kind of pass may never unlock a prerequisite"
        );
    }

    #[test]
    fn the_traversal_quota_is_a_planner_constraint_and_not_a_gate() {
        let content = ContentIndex::shipped();
        assert!(
            content.gate("rootless-traversal").is_none(),
            "a session quota has no pass or fail, so it is not a gate (spec §9.5)"
        );

        let quota = content
            .traversal
            .get("rootless-a-b")
            .expect("a traversal quota on the frontier node");
        assert_eq!((quota.new_keys_min, quota.new_keys_max), (2, 3));
        assert!(quota.goal.contains("new keys"));
    }

    #[test]
    fn the_declared_intent_names_targets_that_resolve() {
        let content = ContentIndex::shipped();
        assert_eq!(content.intent.destination, "Strasbourg / St. Denis");
        assert_eq!(
            content.intent.targets.first().map(String::as_str),
            Some("chord-tone-targeting")
        );
        assert_eq!(
            content.intent.opaque_targets.len(),
            1,
            "only the genuinely unmeasurable stays opaque (decision 19)"
        );
    }

    #[test]
    fn the_escalation_ladder_and_its_thresholds_come_from_the_file() {
        let content = ContentIndex::shipped();
        assert_eq!(content.escalation.consecutive_fail_trigger, 3);
        assert_eq!(
            content.escalation.ladder,
            vec![
                Rung::TempoDown,
                Rung::ShrinkScope,
                Rung::ChangeMode,
                Rung::SwapDrill
            ]
        );
        assert_eq!(content.escalation.tempo_down_pct, 20);
        assert_eq!(content.escalation.tempo_floor_bpm, 40);
        assert_eq!(content.escalation.gate_open_hold_s, 2);
        assert_eq!(content.escalation.glance_hold_s, 1);
    }

    // ── Validation: what must fail to parse (spec §8) ──

    fn shipped_with(edit: impl Fn(&str) -> String) -> Result<ContentIndex, ContentError> {
        ContentIndex::parse(&edit(SHIPPED))
    }

    fn error_of(result: Result<ContentIndex, ContentError>) -> String {
        result.expect_err("expected a parse failure").to_string()
    }

    #[test]
    fn an_unknown_field_is_an_error_rather_than_a_field_quietly_lost() {
        let error = error_of(shipped_with(|s| {
            s.replace(
                "[gates.shells-cycle]",
                "[gates.shells-cycle]\nclean_pass_count = 1",
            )
        }));
        assert!(error.contains("clean_pass_count"), "{error}");
    }

    #[test]
    fn a_schema_version_that_does_not_match_is_an_error() {
        let error = error_of(shipped_with(|s| {
            s.replace("schema_version = \"0.2\"", "schema_version = \"0.1\"")
        }));
        assert!(error.contains("schema_version"), "{error}");
    }

    #[test]
    fn a_mistyped_section_fails_rather_than_being_ignored() {
        let error = error_of(shipped_with(|s| {
            s.replace("[gates.shells-cycle]", "[gatess]")
        }));
        assert!(
            error.contains("gatess"),
            "a section nothing reads would take a gate out of the graph in silence: {error}"
        );
    }

    #[test]
    fn a_click_level_or_transport_nothing_authors_fails_at_startup() {
        let error = error_of(shipped_with(|s| {
            s.replace("l2 = \"click on 2 and 4\"", "l5 = \"click on 2 and 4\"")
        }));
        assert!(error.contains("l2"), "{error}");

        let error = error_of(shipped_with(|s| {
            s.replace("transport = \"none\"", "transport = \"telepathy\"")
        }));
        assert!(error.contains("telepathy"), "{error}");
    }

    #[test]
    fn a_dangling_node_reference_fails_at_startup() {
        let error = error_of(shipped_with(|s| {
            s.replace("\"chord-tone-targeting\",\n", "\"chord-tone-target\",\n")
        }));
        assert!(error.contains("chord-tone-target"), "{error}");
    }

    #[test]
    fn a_dangling_gate_reference_fails_at_startup() {
        let error = error_of(shipped_with(|s| {
            s.replace(
                "[drills.shells-cycle]\ngate = \"shells-cycle\"",
                "[drills.shells-cycle]\ngate = \"shells-cycl\"",
            )
        }));
        assert!(error.contains("shells-cycl"), "{error}");
    }

    #[test]
    fn a_gate_no_drill_can_reach_fails_at_startup() {
        let error = error_of(shipped_with(|s| {
            s.replace("  \"shells-random-key\",\n", "")
        }));
        assert!(
            error.contains("shells-random-key"),
            "an unreachable gate is otherwise invisible: {error}"
        );
    }

    #[test]
    fn a_gate_on_a_different_node_than_the_ladder_listing_it_fails() {
        let error = error_of(shipped_with(|s| {
            s.replace(
                "[gates.shells-cycle]\nnode = \"shells-ii-v-i\"",
                "[gates.shells-cycle]\nnode = \"rootless-a-b\"",
            )
        }));
        assert!(error.contains("shells-cycle"), "{error}");
    }

    #[test]
    fn a_gate_whose_requirement_cannot_be_represented_fails_to_parse() {
        let error = error_of(shipped_with(|s| {
            s.replace(
                "criterion = \"ii-V-I shells through the full cycle of fourths, no stops\"\nclean_passes = 1",
                "criterion = \"something nobody can count\"",
            )
        }));
        assert!(error.contains("shells-cycle"), "{error}");
    }

    #[test]
    fn a_stub_with_drills_fails_because_a_stub_is_unspecified_by_definition() {
        let error = error_of(shipped_with(|s| {
            s.replace(
                "[nodes.diatonic-7ths]\ntitle = \"Diatonic 7ths\"",
                "[nodes.diatonic-7ths]\ndrills = [\"shells-cycle\"]\ntitle = \"Diatonic 7ths\"",
            )
        }));
        assert!(error.contains("diatonic-7ths"), "{error}");
    }
}

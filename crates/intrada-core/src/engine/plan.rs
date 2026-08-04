//! What a session was asked to run. Spec §5's five-stage planner is a later
//! slice; until it lands the engine carries one seeded block so the drill loop
//! has something to run — the design's worked example, moved out of the Swift
//! harness it used to live in (#1176). Its gate values come from the design
//! brief rather than `content/gates.toml`, which no parser reads yet (§9.6).

use serde::{Deserialize, Serialize};

use super::gate::{ClickLevel, GateCriteria, Judge, Requirement};
use crate::domain::item::ItemKind;

/// The rung of a parameter ladder a block is being practised at. Mastery is
/// held per `(node, parameter_level)` (spec §2), so this is state, not display.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParameterLevel {
    pub tempo_bpm: u16,
    pub click_level: ClickLevel,
}

/// One block of a plan: which drill, at which rung, against which gate.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockSpec {
    pub node: String,
    pub drill: String,
    pub drill_title: String,
    /// Where in the material it sits — "A section", "key of F".
    pub section: Option<String>,
    /// The in-flight tune or campaign this block serves.
    pub destination: Option<String>,
    pub kind: ItemKind,
    pub gate: GateCriteria,
    pub level: ParameterLevel,
    pub bars: u16,
    pub beats_per_bar: u8,
    pub count_in_beats: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Plan {
    pub blocks: Vec<BlockSpec>,
    /// The seeded dealer for random-key gates, stored so any session replays
    /// in a test (spec §5).
    pub rng_seed: u64,
}

impl Plan {
    pub fn seed_drill_loop() -> Self {
        Self {
            blocks: vec![BlockSpec {
                node: "rootless-a-b".to_string(),
                drill: "rootless-under-melody".to_string(),
                drill_title: "Rootless voicings".to_string(),
                section: Some("A section".to_string()),
                destination: Some("Strasbourg / St. Denis".to_string()),
                kind: ItemKind::Exercise,
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
                bars: 4,
                beats_per_bar: 4,
                count_in_beats: 4,
            }],
            rng_seed: 0,
        }
    }
}

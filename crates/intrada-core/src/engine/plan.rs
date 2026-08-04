//! What a session was asked to run. Spec §5's five-stage planner is a later
//! slice; until it lands the engine carries one seeded block so the drill loop
//! has something to run — the design brief's worked example. Its gate values
//! come from that brief rather than from `content/gates.toml`, which no parser
//! reads yet (#1180).

use serde::{Deserialize, Serialize};

use super::gate::{ClickLevel, GateCriteria, Judge, Requirement};
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
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
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
            }],
            rng_seed: 0,
        }
    }
}

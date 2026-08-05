//! The practice-coach engine. Quarantined by design (`specs/intrada-coach-engine.md`
//! §1): it never imports the self-report scoring path, and
//! `tests/engine_boundary.rs` asserts it. Local-first only — no `local_first`
//! branch lives here.
//!
//! Scoped to the tap-verdict loop (decision 18): the session state machine (§4)
//! and the bridge surface the drill screen renders (§6). The attempt
//! segmentation this module started as returns with the scoring path — what it
//! established is in `docs/segmentation-findings.md`.

mod coach;
mod content;
mod gate;
mod mastery;
mod plan;
mod session;

pub use coach::{CoachState, CoachView, DrillPhase, DrillView};
pub use gate::{
    ClickLevel, EvidenceSource, GateCriteria, GateProgress, Judge, Requirement, Verdict,
};
pub use plan::{BlockSpec, Circle, Mode, ParameterLevel, Plan};
pub use session::{
    AttemptSummary, BlockRecord, BlockState, CoachEvent, CoachWrites, EngineConfig, EngineSession,
    Exit, Phase, Rung, SessionState, SnapshotAction, WanderRecord,
};

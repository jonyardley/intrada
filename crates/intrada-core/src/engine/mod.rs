//! The practice-coach engine. Quarantined by design (`specs/intrada-coach-engine.md`
//! §1): it never imports the self-report scoring path, and
//! `tests/engine_boundary.rs` asserts it. Local-first only — no `local_first`
//! branch lives here.
//!
//! Two halves so far: attempt segmentation over a captured note stream (pure
//! functions on plain data, `docs/segmentation-findings.md`), and the session
//! state machine the tap-verdict drill loop runs on (§4, §6).

mod coach;
mod gate;
mod grid;
mod note;
mod phrase;
mod plan;
mod segment;
mod session;

pub use coach::{seed_plan, CoachState, CoachView, DrillPhase, DrillView};
pub use gate::{
    ClickLevel, EvidenceSource, GateCriteria, GateProgress, Judge, Requirement, Verdict,
};
pub use grid::{BeatRef, ClickGrid};
pub use note::{cluster_onsets, NoteEvent, Onset};
pub use phrase::{PhraseStep, TargetPhrase};
pub use plan::{BlockSpec, ParameterLevel, Plan};
pub use segment::{
    rest_spans, segment, Attempt, AttemptOutcome, Pause, SegmentConfig, Segmentation, TimingStats,
};
pub use session::{
    AttemptSummary, BlockRecord, BlockState, CoachEvent, EngineConfig, EngineSession, Exit, Phase,
    Rung, SessionState, WanderRecord,
};

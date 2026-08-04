//! The practice-coach engine: attempt segmentation over a captured note
//! stream. Pure functions on plain data — no `Model`, no `Effect`, no FFI
//! surface yet (`docs/rebuild-review.md` §6, PR 3). What it can and cannot
//! decide is recorded in `docs/segmentation-findings.md`.

mod grid;
mod note;
mod phrase;
mod segment;

pub use grid::{BeatRef, ClickGrid};
pub use note::{cluster_onsets, NoteEvent, Onset, TransportTier};
pub use phrase::{PhraseStep, TargetPhrase};
pub use segment::{
    rest_spans, segment, Attempt, AttemptOutcome, Pause, SegmentConfig, Segmentation, TimingStats,
};

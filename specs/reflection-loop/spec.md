# Reflection loop — Phase 1 core model

> Tier 3 spec, riding with the first implementation PR per the workflow.
> Design inputs: `design/BRIEF` (design/briefs/2026-07-reflection-and-narrative.md),
> `design/DECISIONS.md`, `design/HANDOFF.md` in this folder, and
> design-principles decision T7. Journey steps 7–8 (docs/journeys.md).

## Problem

The end-of-session reflection is a single unlabelled note box. The journey
requires a structured beat — what improved, what's still rough, what to
target next time — stored as distinct fields so narrative progress (step 9)
and future session planning (step 5) can consume the answers. Free text in
one string is write-only prose downstream.

## Approach (this PR: core + local persistence only, no UI)

1. **Fields.** `PracticeSession` and `SummarySession` gain
   `reflection_improved`, `reflection_still_rough`, `reflection_next_target`
   — all `Option<String>`, `#[serde(default)]` (same pattern as
   `session_score`, safe on both JSON and bincode wires: no format-specific
   attrs).
2. **Event.** One Summary-gated event,
   `UpdateSessionReflection { field: ReflectionField, text: Option<String> }`
   with `ReflectionField::{Improved, StillRough, NextTarget}` — one handler,
   one validation path, instead of three clones of `UpdateSessionNotes`
   (consolidate-before-template).
3. **Semantics.** Blank or whitespace-only text normalises to `None`
   ("blank prompts save as nothing" — DECISIONS surface 3). Length cap
   `MAX_REFLECTION = 500` chars per field (single-line prompts;
   `validation.rs` stays the single source of truth).
4. **Carry-through.** `SaveSession` copies the three fields onto the saved
   `PracticeSession`; `build_summary_view` and `session_to_view` expose them
   so the summary screen and (later) past-session surfaces can render them.
5. **Local persistence.** GRDB migration (additive, three nullable TEXT
   columns on `session`), upsert + row codec updated together, plus an
   upgrade-path test from the previous migration version (offline
   invariants 2 and the migration checklist).
6. **FFI.** bincode round-trip tests for the new event and the widened
   `PracticeSession` (real-wire guarantee, #846 class).

## Key decisions

- **Field enum over three events**: fewer FFI surfaces, one gate + one
  validator; the shell stays a dumb pipe either way.
- **500-char cap, not MAX_NOTES (5000)**: these are one-line prompts by
  design; a lower cap keeps them honest without being restrictive.
- **Online path deferred**: the API's `SaveSessionRequest` ignores unknown
  JSON fields, so online mode keeps working but does not persist
  reflections server-side. Tracked as a sync-parity issue (same family as
  #842/#1021); acceptable while web is paused and sync is future work.
- **Session-level, not per-entry**: per-entry reflection already exists
  (entry notes/score/tempo). These three answer for the session as a whole.

## Out of scope (later tasks in the phase)

- iOS UI for the prompts (task: summary reflection surface).
- Mid-item quick capture and its append-vs-overwrite core decision.
- Feeding `reflection_next_target` forward as a suggested Aim (needs the
  recommendation surface; the field lands now so the data accrues).

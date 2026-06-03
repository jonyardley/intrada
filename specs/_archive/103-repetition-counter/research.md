# Research: Repetition Counter

**Branch**: `103-repetition-counter` | **Date**: 2026-02-21

## R1: Counter State Representation

**Decision**: Repetition state is split into two layers:
1. **Transient** (during active practice): `rep_target: Option<u8>`, `rep_count: Option<u8>`, `rep_target_reached: bool` stored per-entry in `SetlistEntry`. The `rep_count` and `rep_target_reached` are ephemeral — they only exist while the session is active and are serialised to localStorage for crash recovery.
2. **Persisted** (after save): `rep_target: Option<u8>`, `rep_count: Option<u8>`, `rep_target_reached: Option<bool>` stored in the database.

**Rationale**: Using `Option<u8>` for both target and count (rather than a separate `RepetitionState` struct) keeps the data model flat and consistent with how `score`, `notes`, and `intention` are stored — as optional fields on `SetlistEntry`. The maximum rep target is 10 and max count will never exceed 10, so `u8` is sufficient.

**Alternatives considered**:
- Separate `RepetitionState` struct: Adds structural complexity for a simple counter. Rejected — 3 flat fields are simpler.
- `u16` for count: Overkill for a max value of 10. Rejected.
- Separate `rep_enabled: bool` field: Redundant — `rep_target.is_some()` serves as the enabled flag. Rejected.

## R2: Counter Toggle in Building Phase

**Decision**: In the building phase, `rep_target` starts as `None`. When the musician taps "Add rep target", it becomes `Some(5)` (the default). The stepper adjusts it within 3–10. Tapping a "remove" action sets it back to `None`.

**Rationale**: This mirrors the opt-in pattern — `None` means "not using the counter for this entry". `Some(target)` means "counter enabled with this target". No separate boolean flag needed.

**Alternatives considered**:
- Always-visible stepper with a checkbox: More UI clutter. Rejected per clarification Q3.
- Session-level toggle: Less flexible — different items need different targets. Rejected per clarification Q3.

## R3: Counter Behaviour on Item Transitions

**Decision**: When the musician moves to the next item (NextItem/SkipItem/FinishSession), the current entry's `rep_count` is frozen at its current value. `rep_target_reached` is set to `true` if `rep_count >= rep_target`. The next item starts with its own counter state (0 count if it has a rep_target, or None if not).

**Rationale**: Each entry's counter is independent. The transition simply captures the final state. This matches how `duration_secs` is captured on transition.

**Alternatives considered**:
- Carrying counter state between items: Makes no sense — each item has its own target. Rejected.

## R4: Counter Interaction During Active Practice

**Decision**: "Got it" and "missed" are handled as new `SessionEvent` variants dispatched from the shell. The core increments/decrements `rep_count` on the current entry, clamps at 0 (floor) and target (ceiling — counter freezes at target per clarification Q1). These are pure state mutations — no effects needed.

**Rationale**: Follows the Crux pattern — all state changes go through events. No side effects needed for counter taps (unlike save/load which produce HTTP effects).

**Alternatives considered**:
- Shell-only counter (Leptos signals): Would bypass Crux and break crash recovery. Rejected — counter state must be in the core Model for localStorage persistence.

## R5: Database Schema Extension

**Decision**: Three new nullable columns on `setlist_entries`:
- `rep_target INTEGER` — the configured target (3–10), NULL if counter not used
- `rep_count INTEGER` — final count at save time, NULL if counter not used
- `rep_target_reached INTEGER` — 0 or 1 (SQLite boolean), NULL if counter not used

**Rationale**: Matches the pattern of `score` (nullable INTEGER) and `intention` (nullable TEXT). `rep_target_reached` is denormalised (could be computed from count >= target) but avoids requiring the client to recompute it, keeping display logic simple.

**Alternatives considered**:
- Separate `repetition_data` table: Over-engineered for 3 columns. Rejected.
- JSON blob column: Would break query ability. Rejected.
- Omit `rep_target_reached` (compute it): Minor optimisation but makes display code slightly more complex. Including it is simpler.

## R6: Backward Compatibility Pattern

**Decision**: All new fields use `Option<T>` with `#[serde(default)]`. Existing sessions have `NULL` for all three columns, which deserialise to `None`. No data migration needed.

**Rationale**: Identical pattern to `score`, `intention`, and `session_intention` — proven to work.

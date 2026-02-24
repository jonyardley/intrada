# Research: Basic Goal Setting

**Feature**: 152-goal-setting
**Date**: 2026-02-24

## Decision Log

### 1. Goal Kind Storage Strategy

**Decision**: Flat columns with discriminant — store `goal_type` as TEXT discriminant with nullable type-specific columns (`target_days_per_week`, `target_minutes_per_week`, `item_id`, `target_score`, `milestone_description`).

**Rationale**: Matches the existing `items` table pattern where `item_type` discriminates between "piece" and "exercise" with shared + type-specific nullable columns. SQLite has no native union types; a JSON blob column would sacrifice queryability and indexability. Flat columns allow direct SQL filtering (e.g., `WHERE goal_type = 'item_mastery' AND item_id = ?`).

**Alternatives considered**:
- JSON column for kind-specific data → Rejected: can't index or query efficiently, breaks the pattern
- Separate tables per goal type → Rejected: over-normalised for 4 simple types, complicates list queries
- EAV (entity-attribute-value) → Rejected: adds complexity, poor type safety

### 2. Progress Computation Strategy

**Decision**: Compute progress as a pure function at view-build time in `app.rs`, not stored in the database.

**Rationale**: Progress depends on session data that changes independently of goals. Storing progress would require recalculation triggers on every session save/delete, creating a synchronisation problem. Computing at view time ensures correctness and aligns with the Crux pattern where `view()` is a pure projection of `Model`.

**Alternatives considered**:
- Materialised progress column updated on session change → Rejected: sync complexity, stale data risk
- Background recalculation job → Rejected: unnecessary for single-user app with small data volume

### 3. GoalKind Serde Strategy

**Decision**: Use `#[serde(tag = "type", rename_all = "snake_case")]` on the GoalKind enum for API serialisation. Store as flat columns in SQLite with manual reconstruction in `row_to_goal()`.

**Rationale**: Internally-tagged serde produces clean JSON (`{"type": "session_frequency", "target_days_per_week": 5}`) for the API contract. The DB layer manually reconstructs the enum from flat columns, matching how `items.rs` reconstructs item types. This gives clean API contracts and efficient SQL storage.

**Alternatives considered**:
- Adjacently tagged (`#[serde(tag = "t", content = "c")]`) → Rejected: noisier JSON
- Untagged → Rejected: fragile, relies on field uniqueness for disambiguation

### 4. Week Boundary for Progress

**Decision**: Use ISO 8601 week (Monday start) computed from session `started_at` timestamps in UTC.

**Rationale**: ISO weeks are unambiguous and well-supported by `chrono::IsoWeek`. Using UTC avoids timezone complexity in v1. The edge case note in the spec acknowledges this may cause minor discrepancies for users near the date line, but this is acceptable for v1.

**Alternatives considered**:
- User-local timezone week → Deferred: requires timezone preference storage (not yet built), adds significant complexity
- Calendar week (Sunday start) → Rejected: ISO standard is more widely expected

### 5. Navigation Tab Position

**Decision**: Goals is the 5th tab, placed between Analytics and the end of the tab bar.

**Rationale**: The existing order (Library, Sessions, Routines, Analytics) follows the practice flow. Goals is a planning activity that spans the flow, so it sits at the end. This was explicitly chosen by the user during plan approval.

**Alternatives considered**:
- Before Sessions → Rejected: breaks the existing muscle memory
- Replace Analytics → Rejected: analytics is actively used

### 6. State Transition Model

**Decision**: Active → Completed (final, irreversible), Active → Archived (reversible via Reactivate), no Completed → Active path.

**Rationale**: Completion is a meaningful achievement moment — making it reversible would undermine the celebration. Archiving is a soft removal that should be reversible since the user may change their mind. This was confirmed during spec clarification.

**Alternatives considered**:
- All transitions reversible → Rejected: undermines completion as an achievement
- All transitions one-way → Rejected: too rigid for archiving (user error recovery)

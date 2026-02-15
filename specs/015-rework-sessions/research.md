# Research: Rework Sessions (Setlist Model)

**Feature**: 015-rework-sessions
**Date**: 2026-02-15

## Research Summary

No NEEDS CLARIFICATION items were identified during technical context analysis. The existing codebase provides all required patterns and precedents. This research documents key design decisions and rationale.

---

## Decision 1: Session State Machine in Crux Core

**Decision**: Model the session lifecycle as an explicit state machine (`SessionStatus` enum) within the Crux core `Model`, with events driving transitions.

**Rationale**: The Crux architecture requires all state transitions to flow through events processed by `update()`. An explicit state enum (`Building`, `Active`, `Summary`, `Completed`) maps cleanly to the existing event-driven pattern and makes illegal states unrepresentable. The current codebase already uses this pattern for library operations (events → model mutations → effects).

**Alternatives considered**:
- Implicit state via presence/absence of fields — rejected because it allows invalid combinations (e.g., timer running with no active item) and makes the shell UI harder to drive correctly.
- State machine as a separate domain module — considered but unnecessary; the session module already handles its own events (`handle_session_event`), and the state machine fits naturally within it.

---

## Decision 2: Timer Architecture (Shell-Side with Core Timestamps)

**Decision**: The running timer ticks in the shell (web: `setInterval`), but the core records `DateTime<Utc>` timestamps for start/advance/finish events. Elapsed time is computed from timestamp differences, not accumulated tick counts.

**Rationale**: This matches the spec assumption ("the timer runs client-side in the shell") and the Crux pure-core principle (core must have zero I/O, zero platform dependencies). The existing `PracticeTimer` component already uses `web_sys::set_interval` — the new implementation extends this pattern. Timestamp-based duration calculation is more accurate than tick counting (resilient to tab backgrounding, timer drift).

**Alternatives considered**:
- Core-side timer via a Time capability — rejected because Crux's `Command` API doesn't support recurring timed callbacks, and adding a custom capability would violate the project's "no unnecessary dependencies" principle.
- Pure tick-count accumulation — rejected because browser tabs can be backgrounded (throttled `setInterval`), making tick counts unreliable.

---

## Decision 3: Session-in-Progress Persistence Strategy

**Decision**: Persist the session-in-progress state to localStorage on every `Next`/`Skip`/`Add` event and on a periodic interval (every 30 seconds) during active practice, using a separate key (`intrada:session-in-progress`).

**Rationale**: FR-020 requires session recovery after browser close. The existing pattern persists completed sessions to `intrada:sessions` via `StorageEffect`. Using a separate key for in-progress state avoids mixing transient (building/active) and permanent (completed) data. Persisting on each transition event (not just periodic) ensures no item's time data is lost. The 30-second periodic save covers the case where a user spends a long time on one item without transitioning.

**Alternatives considered**:
- Persist to the same `intrada:sessions` key with a status flag — rejected because it would require filtering out incomplete sessions on load and introduces the risk of partially-saved sessions appearing in history.
- Persist only on transition events (no periodic save) — rejected because a user could practice one item for 30+ minutes and lose all that time if the browser crashes.
- Use `beforeunload` event only — rejected because `beforeunload` is unreliable (not fired on mobile, not guaranteed on crash).

---

## Decision 4: Data Migration Strategy (Old → New Sessions)

**Decision**: Wipe old session data from `intrada:sessions` on first load. No migration of old flat sessions to the new setlist model.

**Rationale**: Per FR-017 and spec assumption ("old data is wiped, not migrated"), the old flat model (`Session { item_id, duration_minutes }`) is fundamentally incompatible with the new setlist model (`PracticeSession { entries: Vec<SetlistEntry> }`). The app is pre-release with no production users, making data migration unnecessary. On first load, the shell detects the old schema (array of objects with `item_id` field) and replaces it with an empty `SessionsData` using the new schema.

**Alternatives considered**:
- Migrate old sessions as single-item setlists — technically possible but adds complexity for zero user benefit (no production data exists).
- Version field in stored data — considered but deferred; not needed while there are no production users. Can be added in a future feature if needed.

---

## Decision 5: SetlistEntry Identity

**Decision**: Each `SetlistEntry` has its own ULID and stores a snapshot of the library item's title and type at session time.

**Rationale**: FR-018 requires sessions to remain readable after library items are deleted. FR-019 allows the same item to appear multiple times in a setlist, so entries need independent identity. A ULID per entry (generated when added to the setlist) provides stable identity for per-item notes, skip status, and time tracking. The title/type snapshot is taken when the entry is added (during building phase or mid-session add).

**Alternatives considered**:
- Use position index as identity — rejected because mid-session additions would shift indices, breaking references.
- Reference library item by ID without snapshot — rejected because FR-018 explicitly requires readability after item deletion.

---

## Decision 6: StorageEffect Rework

**Decision**: Replace the existing session-related `StorageEffect` variants (`SaveSession`, `UpdateSession`, `DeleteSession`, `LoadSessions`) with new variants aligned to the setlist model: `SavePracticeSession`, `SaveSessionInProgress`, `ClearSessionInProgress`, `LoadSessions` (reused name, new schema), `DeletePracticeSession`.

**Rationale**: The shell handles storage effects — changing the variants is the cleanest way to communicate the new data shapes. The existing library-related variants (`SavePiece`, `SaveExercise`, etc.) remain unchanged. The `SessionsLoaded` event already exists and can be reused with the new `PracticeSession` type.

**Alternatives considered**:
- Generic `SaveData(String, String)` effect — rejected because it loses type safety and the Crux pattern benefits from typed operations.
- Keep old variant names with new payloads — rejected because it would be confusing to have `SaveSession` carry a `PracticeSession`.

---

## Decision 7: ViewModel Shape for Session UI

**Decision**: Replace `SessionView` (flat per-old-session) with `PracticeSessionView` (one per completed session, containing a list of `SetlistEntryView` items) and add `ActiveSessionView` (optional, present only when a session is in progress).

**Rationale**: The shell needs different view shapes for different session lifecycle states. The current `ViewModel { sessions: Vec<SessionView> }` maps 1:1 to old flat sessions. The new model needs: (1) a list of completed sessions for history, (2) the in-progress session state for the practice UI, and (3) per-item practice summaries for library detail views (already exists as `ItemPracticeSummary`). Splitting into distinct view types follows the existing pattern of computing views in `view()` from model state.

**Alternatives considered**:
- Single unified view type with status flags — rejected because it would require the shell to filter/branch on status for every render.
- Multiple view models (separate trait impls) — rejected because Crux supports one `ViewModel` per app; composition within a single struct is the standard approach.

---

## Technology Patterns Reference

### Crux Event/Effect Pattern (existing, unchanged)
```
Shell → Event → Core.update() → Command<Effect, Event>
         ↓                          ↓
    Model mutation          StorageEffect → Shell handles persistence
                            RenderOp → Shell calls core.view()
```

### Session Lifecycle State Machine
```
Building → Active → Summary → Completed
   ↑         ↓
   └── (discard/abandon without save)
```

### localStorage Key Layout
```
intrada:library              — LibraryData (unchanged)
intrada:sessions             — SessionsData (new schema: Vec<PracticeSession>)
intrada:session-in-progress  — Option<ActiveSession> (new key)
```

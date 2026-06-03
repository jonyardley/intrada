# Research: Practice Sessions

**Feature**: 012-practice-sessions | **Date**: 2026-02-15

## R1: Session storage — separate file vs. extending LibraryData

**Decision**: Use a separate `SessionsData` struct persisted to `sessions.json` (CLI) and `intrada:sessions` localStorage key (web). Do NOT extend `LibraryData`.

**Rationale**: The spec (FR-010) explicitly mandates the segmented JSON pattern from 011-json-persistence. Separate files/keys mean library and session data can evolve independently. Sessions may grow to hundreds or thousands of records while the library stays small — keeping them separate avoids bloating library reads. The segmented pattern also prepares for future `goals.json`.

**Alternatives considered**:
- Embed sessions inside `LibraryData`: Simpler (one file) but violates the segmented pattern, couples session growth to library I/O, and contradicts FR-010.
- Sessions as sub-arrays inside each Piece/Exercise: Would make orphaned session handling (FR-011) complex and break the flat data model.

## R2: StorageEffect design — unified vs. separate session effects

**Decision**: Add session-specific variants to the existing `StorageEffect` enum: `SaveSession(Session)`, `UpdateSession(Session)`, `DeleteSession { id: String }`, `LoadSessions`.

**Rationale**: Follows the same pattern as `SavePiece`/`SaveExercise`. The shell already matches on `StorageEffect` — adding variants is mechanical. A separate `SessionStorageEffect` enum would require a second `Effect` variant and duplicate all the shell wiring.

**Alternatives considered**:
- Separate `SessionStorageEffect` enum + `Effect::SessionStorage(...)` variant: More complex for no benefit — the existing pattern works fine.
- Reuse `DeleteItem` for session deletion: Could work since IDs are ULIDs (globally unique), but is semantically confusing (sessions aren't "items"). Better to have explicit `DeleteSession` for clarity.

## R3: ViewModel design — sessions in ViewModel

**Decision**: Add a `sessions: Vec<SessionView>` field to `ViewModel` alongside the existing `items: Vec<LibraryItemView>`. Sessions are NOT library items and should not be mixed into `items`.

**Rationale**: Sessions have fundamentally different display fields (duration, started-at, linked item name) vs. library items (composer, key, tempo, tags). Forcing sessions into `LibraryItemView` would require many unused optional fields and confuse the web/CLI display logic. A separate `SessionView` struct keeps things clean.

**Alternatives considered**:
- Add sessions to `LibraryItemView` with `item_type = "session"`: Would require `Option<u32>` for duration, `Option<String>` for linked item, etc. Pollutes the view model with session-specific concerns.
- Separate `SessionViewModel`: Over-engineered. A simple `Vec<SessionView>` on the existing `ViewModel` is sufficient.

## R4: Event routing — Session events in the Event enum

**Decision**: Add `Event::Session(SessionEvent)` and `Event::SessionsLoaded { sessions: Vec<Session> }` to the root `Event` enum, following the existing `Event::Piece(PieceEvent)` / `Event::Exercise(ExerciseEvent)` pattern.

**Rationale**: Consistent with the existing per-domain event delegation pattern. The `handle_session_event` function follows the same signature as `handle_piece_event` and `handle_exercise_event`.

**Alternatives considered**:
- Flat session events on `Event` (e.g., `Event::LogSession`, `Event::DeleteSession`): Breaks the per-domain grouping convention. Would clutter the root enum.

## R5: Web timer implementation — client-side only

**Decision**: The practice timer is entirely client-side state in the Leptos web shell. It does NOT flow through the Crux core. When the timer stops, the elapsed duration is rounded to minutes and sent as a normal `SessionEvent::Log` event.

**Rationale**: Timer ticking is a UI concern — updating a display counter every second. The Crux core doesn't need to know about timer state. Only the final result (duration in minutes) matters to the domain. This keeps the core pure and avoids a flood of tick events.

**Alternatives considered**:
- Timer state in Crux Model: Would require a `TimerTick` event every second, polluting the event log and making the core aware of UI timing. Against Crux's pure core principle.
- Timer state in localStorage: Unnecessary complexity for P3. Tab close = timer lost (spec edge case confirms this is acceptable).

## R6: Data loading — two-phase init

**Decision**: App initialisation loads both library data and session data. The shell sends `Event::DataLoaded` (existing) for library items and `Event::SessionsLoaded` (new) for sessions. Two separate load calls, two separate storage reads.

**Rationale**: Follows the segmented storage pattern. Each domain loads independently. The CLI shell's `load_data()` method will load both files. The web shell's `load_library_data()` and a new `load_sessions_data()` will load from their respective localStorage keys.

**Alternatives considered**:
- Single combined `DataLoaded` event carrying both library and sessions: Would couple the two domains at the event level and make `DataLoaded` grow as new domains are added. Better to keep them separate.

## R7: Notes validation for sessions

**Decision**: Reuse the existing `MAX_NOTES = 5000` character limit from the library validation. Session notes follow the same validation rules as piece/exercise notes.

**Rationale**: Consistency across the app. No reason to have different limits for session notes vs. library item notes.

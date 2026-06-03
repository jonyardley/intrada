# Tasks: Rework Sessions (Setlist Model)

**Input**: Design documents from `/specs/015-rework-sessions/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Remove old session code and establish new type foundations

- [X] T001 Remove old session types (`Session`, `LogSession`, `UpdateSession`, `SessionsData`) from `crates/intrada-core/src/domain/types.rs` — delete the `Session` import, `SessionsData` struct, `LogSession` struct, and `UpdateSession` struct; keep all library-related types unchanged
- [X] T002 Remove old session validation functions (`validate_log_session`, `validate_update_session`) from `crates/intrada-core/src/validation.rs` — delete both functions and their imports; remove `MIN_DURATION` and `MAX_DURATION` constants (sessions now track seconds, not minutes)
- [X] T003 Remove old `session.rs` module contents — clear `crates/intrada-core/src/domain/session.rs` of the old `Session` struct, `SessionEvent` enum, `handle_session_event` function, and all old tests; leave the file in place for the new implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: New domain types, enums, and updated app infrastructure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 [P] Define `EntryStatus` enum (`Completed`, `Skipped`, `NotAttempted`) and `CompletionStatus` enum (`Completed`, `EndedEarly`) with Serialize/Deserialize/Debug/Clone/PartialEq derives in `crates/intrada-core/src/domain/session.rs`
- [X] T005 [P] Define `SetlistEntry` struct (id, item_id, item_title, item_type, position, duration_secs, status, notes) with Serialize/Deserialize/Debug/Clone/PartialEq derives in `crates/intrada-core/src/domain/session.rs`
- [X] T006 [P] Define `PracticeSession` struct (id, entries, session_notes, started_at, completed_at, total_duration_secs, completion_status) with Serialize/Deserialize/Debug/Clone/PartialEq derives in `crates/intrada-core/src/domain/session.rs`
- [X] T007 [P] Define transient state types: `BuildingSession` (entries), `ActiveSession` (id, entries, current_index, current_item_started_at, session_started_at), `SummarySession` (id, entries, session_started_at, session_ended_at, session_notes) with Serialize/Deserialize derives in `crates/intrada-core/src/domain/session.rs`
- [X] T008 [P] Define `SessionStatus` enum (`Idle`, `Building(BuildingSession)`, `Active(ActiveSession)`, `Summary(SummarySession)`) with Debug/Clone in `crates/intrada-core/src/domain/session.rs` — implement `Default` returning `Idle`
- [X] T009 [P] Define new `SessionsData` struct containing `sessions: Vec<PracticeSession>` with `#[serde(default)]` in `crates/intrada-core/src/domain/types.rs` — replace the removed old `SessionsData`
- [X] T010 Define new `SessionEvent` enum with all 20 variants (StartBuilding, AddToSetlist, AddNewItemToSetlist, RemoveFromSetlist, ReorderSetlist, StartSession, CancelBuilding, NextItem, SkipItem, AddItemMidSession, AddNewItemMidSession, FinishSession, EndSessionEarly, UpdateEntryNotes, UpdateSessionNotes, SaveSession, DiscardSession, RecoverSession, DeleteSession) per contracts/session-events.md in `crates/intrada-core/src/domain/session.rs`
- [X] T011 Update `StorageEffect` enum in `crates/intrada-core/src/app.rs` — remove `SaveSession(Session)`, `UpdateSession(Session)`, `DeleteSession { id }` variants; add `SavePracticeSession(PracticeSession)`, `DeletePracticeSession { id }`, `SaveSessionInProgress(ActiveSession)`, `ClearSessionInProgress`, `LoadSessionInProgress` variants; keep `LoadSessions` (same name, now loads `Vec<PracticeSession>`)
- [X] T012 Update `Event` enum in `crates/intrada-core/src/app.rs` — change `SessionsLoaded { sessions }` to use `Vec<PracticeSession>` instead of `Vec<Session>`; update `Session` import to new types
- [X] T013 Update `Model` struct in `crates/intrada-core/src/model.rs` — replace `sessions: Vec<Session>` with `sessions: Vec<PracticeSession>` and add `session_status: SessionStatus` field (defaulting to `Idle`)
- [X] T014 [P] Define new view model types in `crates/intrada-core/src/model.rs` — add `PracticeSessionView`, `SetlistEntryView`, `ActiveSessionView`, `SummaryView` structs per data-model.md; remove old `SessionView` struct
- [X] T015 Update `ViewModel` struct in `crates/intrada-core/src/model.rs` — replace `sessions: Vec<SessionView>` with `sessions: Vec<PracticeSessionView>`; add `active_session: Option<ActiveSessionView>`, `session_status: String`, `building_setlist: Option<Vec<SetlistEntryView>>`, `summary: Option<SummaryView>` fields
- [X] T016 Add new session validation functions in `crates/intrada-core/src/validation.rs` — `validate_session_notes(notes: &Option<String>)` and `validate_entry_notes(notes: &Option<String>)` both checking `MAX_NOTES` (5000 chars); `validate_setlist_not_empty(entries: &[SetlistEntry])` checking len >= 1
- [X] T017 Update `domain/mod.rs` to re-export new session types (`PracticeSession`, `SetlistEntry`, `EntryStatus`, `CompletionStatus`, `SessionStatus`, `ActiveSession`, `SessionEvent`) in `crates/intrada-core/src/domain/mod.rs`

**Checkpoint**: All new types defined, old types removed, app.rs compiles with new Effect/Event variants. `cargo check` should pass (tests may fail until event handlers are implemented).

---

## Phase 3: User Story 5 - Replace Existing Session Data and Code (Priority: P1) 🎯 Prerequisite

**Goal**: Old session model fully replaced — no legacy code remains; new empty handler in place

**Independent Test**: `cargo test` passes; grep for old type names returns zero results in `crates/`; `cargo clippy` clean

**Why Phase 3 (before US1/US2)**: US5 is a prerequisite for all other stories — the old model must be cleared to make way for the new one. The foundational types are now in place from Phase 2.

### Implementation for User Story 5

- [X] T018 [US5] Implement stub `handle_session_event` function in `crates/intrada-core/src/domain/session.rs` — match on all `SessionEvent` variants, return `crux_core::render::render()` for each (no-op stubs); wire into `app.rs` `update()` match arm
- [X] T019 [US5] Update `view()` function in `crates/intrada-core/src/app.rs` — replace old `SessionView` construction with new `PracticeSessionView` construction from `model.sessions: Vec<PracticeSession>`; update `compute_practice_summary()` to iterate `PracticeSession.entries` where `entry.status == Completed`; populate new `session_status`, `active_session`, `building_setlist`, `summary` fields from `model.session_status`
- [X] T020 [US5] Update `SessionsLoaded` handler in `crates/intrada-core/src/app.rs` — change `model.sessions = sessions` to accept `Vec<PracticeSession>`
- [X] T021 [US5] Remove old session-related web components — delete `crates/intrada-web/src/components/practice_timer.rs` and `crates/intrada-web/src/components/session_history.rs`; remove their module declarations from `crates/intrada-web/src/components/mod.rs`
- [X] T022 [US5] Update `crates/intrada-web/src/core_bridge.rs` — replace old `StorageEffect::SaveSession`, `UpdateSession`, `DeleteSession` handlers with new `SavePracticeSession`, `DeletePracticeSession`, `SaveSessionInProgress`, `ClearSessionInProgress`, `LoadSessionInProgress` handlers; update `SESSIONS` thread_local to use new `SessionsData` with `Vec<PracticeSession>`; add `SESSIONS_IN_PROGRESS_KEY = "intrada:session-in-progress"` constant
- [X] T023 [US5] Add old-data detection and wipe in `crates/intrada-web/src/core_bridge.rs` — in `load_sessions_from_local_storage()`, detect old schema (JSON objects with `item_id` + `duration_minutes` fields), wipe and replace with empty new-schema `SessionsData`; add `load_session_in_progress()` and `save_session_in_progress()` functions for the `intrada:session-in-progress` key
- [X] T024 [US5] Update `crates/intrada-web/src/views/detail.rs` — remove `PracticeTimer` and `SessionHistory` component usage; update to read practice summary from new `ViewModel` fields
- [X] T025 [US5] Update `crates/intrada-web/src/views/sessions.rs` — replace old `SessionsListView` with a placeholder view that reads from `vm.sessions: Vec<PracticeSessionView>` (full implementation in Phase 8)
- [X] T026 [US5] Fix all compilation errors in `crates/intrada-core/` and `crates/intrada-web/` — resolve any remaining references to old types; ensure `cargo check` passes for the entire workspace
- [X] T027 [US5] Update existing tests in `crates/intrada-core/src/app.rs` — fix session-related tests (view tests, practice summary tests) to use new `PracticeSession` and `SetlistEntry` types; ensure `cargo test` passes
- [X] T028 [US5] Write core tests verifying old types are gone — add a test confirming `SessionsData { sessions: vec![] }` serializes/deserializes correctly with new schema in `crates/intrada-core/src/domain/session.rs`

**Checkpoint**: `cargo test` passes. `cargo clippy` clean. Zero references to `LogSession`, old `UpdateSession`, old `SessionEvent::Log/Update/Delete` in `crates/`. Old web components removed. SC-007 met.

---

## Phase 4: User Story 1 - Build a Practice Setlist (Priority: P1) 🎯 MVP Part 1

**Goal**: Users can select library items, arrange them into an ordered setlist, reorder and remove items

**Independent Test**: Start building a session → add items from library → see them in order → reorder → remove one → verify order preserved

### Implementation for User Story 1

- [X] T029 [US1] Implement `StartBuilding` event handler in `crates/intrada-core/src/domain/session.rs` — transition from `Idle` to `Building(BuildingSession { entries: vec![] })`; error if not `Idle`
- [X] T030 [US1] Implement `AddToSetlist { item_id }` event handler in `crates/intrada-core/src/domain/session.rs` — look up item in `model.pieces`/`model.exercises`, create `SetlistEntry` with ULID, snapshot title/type, position = entries.len(), status = NotAttempted; error if item not found or not in Building state
- [X] T031 [US1] Implement `RemoveFromSetlist { entry_id }` event handler in `crates/intrada-core/src/domain/session.rs` — remove entry by id, re-index remaining positions; error if entry not found or not in Building state
- [X] T032 [US1] Implement `ReorderSetlist { entry_id, new_position }` event handler in `crates/intrada-core/src/domain/session.rs` — move entry to new_position, re-index all positions; error if invalid position or not Building
- [X] T033 [US1] Implement `CancelBuilding` event handler in `crates/intrada-core/src/domain/session.rs` — transition from Building to Idle, clear building state
- [X] T034 [US1] Implement `StartSession` event handler (setlist validation only) in `crates/intrada-core/src/domain/session.rs` — validate entries.len() >= 1 (FR-004); if valid, transition from Building to Active with current_index=0, timestamps set; emit `SaveSessionInProgress` effect
- [X] T035 [US1] Update `view()` in `crates/intrada-core/src/app.rs` to populate `building_setlist` field — when `model.session_status` is `Building`, map entries to `Vec<SetlistEntryView>`
- [X] T036 [US1] Write core unit tests for Building phase events in `crates/intrada-core/src/domain/session.rs` — test StartBuilding, AddToSetlist (valid, item not found), RemoveFromSetlist, ReorderSetlist, CancelBuilding, StartSession with empty setlist (error), StartSession with items (success)
- [X] T037 [US1] Create `SetlistEntry` component in `crates/intrada-web/src/components/setlist_entry.rs` — displays item title, type badge, position number; add/remove button; register in `components/mod.rs`
- [X] T038 [US1] Create `SetlistBuilder` component in `crates/intrada-web/src/components/setlist_builder.rs` — shows library items to select from (reuse existing `LibraryItemView` data from ViewModel), "Add" button per item, current setlist display with `SetlistEntry` components, reorder up/down buttons, remove button, "Start Session" and "Cancel" buttons; register in `components/mod.rs`
- [X] T039 [US1] Create `SessionNewView` in `crates/intrada-web/src/views/session_new.rs` — route `/sessions/new`, wraps `SetlistBuilder`, dispatches `SessionEvent::StartBuilding` on mount, dispatches building-phase events via `process_effects()`; register in `views/mod.rs`
- [X] T040 [US1] Add route `/sessions/new` in `crates/intrada-web/src/app.rs` — map to `SessionNewView` component; add "New Session" navigation link to appropriate location (e.g., app header or sessions list)

**Checkpoint**: User can navigate to `/sessions/new`, browse library, add items to setlist, reorder, remove, and press "Start Session" (which transitions to Active state). FR-001, FR-002, FR-003, FR-004 verified.

---

## Phase 5: User Story 2 - Practice Through a Setlist with Timed Tracking (Priority: P1) 🎯 MVP Part 2

**Goal**: Users work through setlist with running timer, advance with "Next", finish with "Finish", see progress, end early

**Independent Test**: Start session → see first item with timer → press Next → verify time recorded → advance to last → press Finish → see summary transition

**Depends on**: Phase 4 (US1) for setlist building and StartSession transition

### Implementation for User Story 2

- [X] T041 [US2] Implement `NextItem { now }` event handler in `crates/intrada-core/src/domain/session.rs` — compute duration_secs from timestamps, set current entry status to Completed, advance current_index, reset current_item_started_at; emit `SaveSessionInProgress`; if current_index was last item, transition to Summary instead
- [X] T042 [US2] Implement `FinishSession { now }` event handler in `crates/intrada-core/src/domain/session.rs` — record last item's time, set status Completed, transition to Summary with completion_status=Completed; emit `SaveSessionInProgress`
- [X] T043 [US2] Implement `EndSessionEarly { now }` event handler in `crates/intrada-core/src/domain/session.rs` — record current item's time (status Completed), mark all remaining items as NotAttempted with duration_secs=0, transition to Summary with completion_status=EndedEarly; emit `SaveSessionInProgress`
- [X] T044 [US2] Update `view()` in `crates/intrada-core/src/app.rs` to populate `active_session` field — when `session_status` is `Active`, compute `ActiveSessionView` with current_item_title, progress_label ("X of Y"), completed/upcoming entries
- [X] T045 [US2] Write core unit tests for Active phase events in `crates/intrada-core/src/domain/session.rs` — test NextItem (time recording, index advance), NextItem on last item (transitions to Summary), FinishSession, EndSessionEarly (partial completion, remaining marked NotAttempted), progress tracking
- [X] T046 [US2] Create `SessionTimer` component in `crates/intrada-web/src/components/session_timer.rs` — displays current item name, type badge, elapsed time (driven by `setInterval` ticking a local signal), progress label "X of Y"; "Next"/"Finish"/"End Early" buttons that dispatch events with `chrono::Utc::now()` timestamp; register in `components/mod.rs`
- [X] T047 [US2] Create `SessionActiveView` in `crates/intrada-web/src/views/session_active.rs` — route `/sessions/active`, wraps `SessionTimer`, shows completed items list below timer, redirects to `/sessions/new` if no active session; register in `views/mod.rs`
- [X] T048 [US2] Add route `/sessions/active` in `crates/intrada-web/src/app.rs` — map to `SessionActiveView`; add navigation logic: after `StartSession` succeeds, redirect to `/sessions/active`

**Checkpoint**: Full practice flow works: build setlist → start → timer runs → Next advances → Finish ends → End Early works. FR-005 through FR-008, FR-023 verified. US1+US2 together form the MVP.

---

## Phase 6: User Story 4 - End-of-Session Summary with Notes (Priority: P2)

**Goal**: Users see session breakdown after finishing, can add per-item and session notes, save the session

**Independent Test**: Complete a session → summary shows items with times → add per-item note → add session note → save → session appears in history with all data

**Why before US3**: The summary is the natural endpoint of US2 (practice flow). Without it, sessions can't be saved. US3 (skip/add mid-session) extends the active phase but doesn't block saving.

**Depends on**: Phase 5 (US2) for the transition to Summary state

### Implementation for User Story 4

- [X] T049 [US4] Implement `UpdateEntryNotes { entry_id, notes }` event handler in `crates/intrada-core/src/domain/session.rs` — validate notes length (MAX_NOTES), update entry's notes field; error if not Summary or entry not found
- [X] T050 [US4] Implement `UpdateSessionNotes { notes }` event handler in `crates/intrada-core/src/domain/session.rs` — validate notes length (MAX_NOTES), update session_notes; error if not Summary
- [X] T051 [US4] Implement `SaveSession { now }` event handler in `crates/intrada-core/src/domain/session.rs` — construct `PracticeSession` from `SummarySession` data (id, entries, notes, timestamps, total_duration_secs computed from entries, completion_status), push to `model.sessions`, transition to Idle; emit `SavePracticeSession` + `ClearSessionInProgress` effects
- [X] T052 [US4] Implement `DiscardSession` event handler in `crates/intrada-core/src/domain/session.rs` — transition to Idle, clear summary state; emit `ClearSessionInProgress`
- [X] T053 [US4] Update `view()` in `crates/intrada-core/src/app.rs` to populate `summary` field — when `session_status` is `Summary`, compute `SummaryView` with total_duration_display, entries with duration_display, items_completed/skipped/not_attempted counts, completion_status
- [X] T054 [US4] Write core unit tests for Summary phase events in `crates/intrada-core/src/domain/session.rs` — test UpdateEntryNotes (valid, too long, entry not found), UpdateSessionNotes (valid, too long), SaveSession (session persisted, status returns to Idle), DiscardSession (no persistence)
- [X] T055 [US4] Create `SessionSummary` component in `crates/intrada-web/src/components/session_summary.rs` — displays total duration, list of entries (each showing title, type, duration, status), text area for per-item notes, text area for overall session notes, "Save" and "Discard" buttons; register in `components/mod.rs`
- [X] T056 [US4] Create `SessionSummaryView` in `crates/intrada-web/src/views/session_summary.rs` — route `/sessions/summary`, wraps `SessionSummary`, dispatches summary events, redirects after save/discard; register in `views/mod.rs`
- [X] T057 [US4] Add route `/sessions/summary` in `crates/intrada-web/src/app.rs` — map to `SessionSummaryView`; add navigation logic: after FinishSession/EndSessionEarly transitions to Summary, redirect to `/sessions/summary`; after SaveSession/DiscardSession, redirect to `/sessions`
- [X] T058 [US4] Implement `DeleteSession { id }` event handler in `crates/intrada-core/src/domain/session.rs` — remove session from `model.sessions` by id; emit `DeletePracticeSession { id }`; error if not found

**Checkpoint**: Full session lifecycle complete: build → practice → summary → save. Sessions persist to localStorage. FR-012 through FR-015 verified.

---

## Phase 7: User Story 3 - Skip Items and Add Items Mid-Session (Priority: P2)

**Goal**: Users can skip current item, add existing or new library items to setlist during an active session

**Independent Test**: Start session → skip first item → add item from library → add new item → finish → summary shows skipped item and added items

**Depends on**: Phase 5 (US2) for active session state

### Implementation for User Story 3

- [X] T059 [US3] Implement `SkipItem { now }` event handler in `crates/intrada-core/src/domain/session.rs` — set current entry status to Skipped, duration_secs to 0; advance to next item or transition to Summary if last; emit `SaveSessionInProgress`
- [X] T060 [US3] Implement `AddItemMidSession { item_id }` event handler in `crates/intrada-core/src/domain/session.rs` — look up item, create `SetlistEntry` with snapshot, append to end of entries with status NotAttempted; do NOT modify current_index or current_item_started_at (SC-004); emit `SaveSessionInProgress`
- [X] T061 [US3] Implement `AddNewItemMidSession { title, item_type }` event handler in `crates/intrada-core/src/domain/session.rs` — create new Piece or Exercise in model (ULID, title, minimal fields), create SetlistEntry with snapshot, append to entries; emit `SavePiece`/`SaveExercise` + `SaveSessionInProgress`
- [X] T062 [US3] Implement `AddNewItemToSetlist { title, item_type }` event handler (building phase) in `crates/intrada-core/src/domain/session.rs` — same as AddNewItemMidSession but for Building state; create library item and add entry
- [X] T063 [US3] Write core unit tests for Skip/Add events in `crates/intrada-core/src/domain/session.rs` — test SkipItem (zero duration, skipped status, advances), SkipItem on last item (Summary transition), AddItemMidSession (appended, timer unaffected), AddNewItemMidSession (new library item + entry), AddNewItemToSetlist
- [X] T064 [US3] Update `SessionTimer` component in `crates/intrada-web/src/components/session_timer.rs` — add "Skip" button that dispatches `SkipItem`; add "Add Item" button/modal that shows library picker and dispatches `AddItemMidSession` or opens a mini form for `AddNewItemMidSession`
- [X] T065 [US3] Update `SetlistBuilder` component in `crates/intrada-web/src/components/setlist_builder.rs` — add "Create New Item" button/form for `AddNewItemToSetlist` during building phase

**Checkpoint**: Full mid-session flexibility works: skip items, add from library, add brand new items. FR-009 through FR-011 verified. SC-004 verified (timer not interrupted).

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Session history, practice summary, recovery, and final cleanup

- [X] T066 Implement session history list view in `crates/intrada-web/src/views/sessions.rs` — display each `PracticeSessionView` with date, total duration, item count, completion status; link to session detail; newest first (FR-024)
- [X] T067 Update library detail view practice summary in `crates/intrada-web/src/views/detail.rs` — verify `ItemPracticeSummary` displays correctly from new model; show total sessions and total minutes aggregated from setlist entries where status=Completed (FR-025)
- [X] T068 Implement `RecoverSession` event handler in `crates/intrada-core/src/domain/session.rs` — accept `ActiveSession`, set `model.session_status = Active(session)`; error if not Idle
- [X] T069 Implement session-in-progress recovery flow in `crates/intrada-web/src/core_bridge.rs` — on app init, call `load_session_in_progress()`; if data found, emit `RecoverSession` event; handle `LoadSessionInProgress` effect
- [X] T070 Implement periodic session-in-progress save in `crates/intrada-web/src/views/session_active.rs` — add 30-second `setInterval` that emits a `SaveSessionInProgress` effect via the shell (not through core events, since the timer state is shell-side); clear interval on unmount
- [X] T071 Write core unit tests for RecoverSession in `crates/intrada-core/src/domain/session.rs` — test recovery from valid ActiveSession, test error when not Idle
- [X] T072 Write core unit tests for complete session lifecycle in `crates/intrada-core/src/domain/session.rs` — end-to-end test: StartBuilding → AddToSetlist (3 items) → StartSession → NextItem → SkipItem → FinishSession → UpdateEntryNotes → SaveSession; verify final PracticeSession has correct entries, times, notes, statuses
- [X] T073 Write core unit test for `compute_practice_summary()` with new model in `crates/intrada-core/src/app.rs` — verify aggregation from PracticeSession entries: counts only Completed entries, sums duration_secs, converts to minutes; skipped entries excluded
- [X] T074 Write core unit test for edge cases in `crates/intrada-core/src/domain/session.rs` — all items skipped (zero total time, saveable), duplicate items (independent tracking), single-item setlist (Finish immediately available)
- [X] T075 [P] Add helper function `format_duration_display(secs: u64) -> String` in `crates/intrada-core/src/domain/session.rs` — format seconds as "Xh Ym Zs", "Ym Zs", or "Zs"; used by view() for duration_display fields
- [X] T076 Run `cargo test` and fix any failures across the workspace
- [X] T077 Run `cargo clippy -- -D warnings` and fix all warnings
- [ ] T078 Verify quickstart.md validation steps 1–10 manually via `trunk serve`

**Checkpoint**: All user stories complete. All tests pass (SC-006). Zero old references (SC-007). Session recovery works (SC-005). Practice summary correct (FR-025). Session history displays properly (FR-024).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — defines all new types
- **Phase 3 (US5)**: Depends on Phase 2 — removes old code, wires new types
- **Phase 4 (US1)**: Depends on Phase 3 — building phase events
- **Phase 5 (US2)**: Depends on Phase 4 — active phase events (needs StartSession from US1)
- **Phase 6 (US4)**: Depends on Phase 5 — summary phase events (needs transition from Active)
- **Phase 7 (US3)**: Depends on Phase 5 — mid-session events (needs Active state)
- **Phase 8 (Polish)**: Depends on Phases 6 and 7 — final integration and testing

### User Story Dependencies

```
US5 (replace old code) ← foundational, must be first
  ↓
US1 (build setlist) ← needs new types in place
  ↓
US2 (timed practice) ← needs StartSession from US1
  ↓
  ├── US4 (summary + notes) ← needs Active→Summary transition
  └── US3 (skip + add) ← needs Active state
        ↓
     Polish ← needs all stories complete
```

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T004, T005, T006, T007, T008, T009 can all run in parallel (different structs/enums, same file but independent sections)
- T014 can run in parallel with other Phase 2 tasks (different file: model.rs)

**Phase 6 & 7**:
- US4 (summary) and US3 (skip/add) can be implemented in parallel since both depend only on US2's Active state being in place
- Core event handlers are in the same file but touch different match arms

**Within each phase**:
- Core domain tasks and web component tasks are in different crates — can be parallelised once the core types they depend on exist

---

## Parallel Example: Phase 2 (Foundational)

```bash
# Launch all type definitions in parallel:
Task: T004 "Define EntryStatus and CompletionStatus enums"
Task: T005 "Define SetlistEntry struct"
Task: T006 "Define PracticeSession struct"
Task: T007 "Define transient state types"
Task: T008 "Define SessionStatus enum"
Task: T009 "Define new SessionsData struct"
Task: T014 "Define new view model types in model.rs"
```

## Parallel Example: Phase 7 (US3) — core vs web

```bash
# Core implementation can be done first:
Task: T059–T063 (core event handlers and tests)

# Then web components:
Task: T064–T065 (UI updates)
```

---

## Implementation Strategy

### MVP First (US5 + US1 + US2 + US4)

1. Complete Phase 1: Setup (remove old types)
2. Complete Phase 2: Foundational (define new types)
3. Complete Phase 3: US5 (wire new types, remove old code)
4. Complete Phase 4: US1 (setlist building)
5. Complete Phase 5: US2 (timed practice)
6. Complete Phase 6: US4 (summary + save)
7. **STOP and VALIDATE**: Full session lifecycle works end-to-end
8. Deploy/demo if ready — skip/add (US3) is enhancement

### Incremental Delivery

1. Phases 1–3 → Old code replaced, new skeleton in place
2. Phase 4 → Users can build setlists (functional but can't practice yet)
3. Phase 5 → Users can practice with timer (MVP functional)
4. Phase 6 → Users can review and save sessions (MVP complete!)
5. Phase 7 → Users can skip and add mid-session (enhancement)
6. Phase 8 → Polish, recovery, history, edge cases

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US5 is ordered first despite being "replace old code" because it's a prerequisite — the old types must be removed before new event handlers can be implemented
- Sessions are immutable after save — no UpdateSession needed (simpler than old model)
- Timer ticks in shell (setInterval), core records timestamps — keep this separation strict
- The `now: DateTime<Utc>` parameter on Active phase events is provided by the shell at dispatch time
- Commit after each phase completion for clean revert points

# Tasks: Practice Sessions

**Input**: Design documents from `/specs/012-practice-sessions/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No new crate dependencies needed. Verify existing workspace compiles and tests pass before starting.

- [x] T001 Run `cargo test` and `cargo clippy` to confirm clean baseline before changes
- [x] T002 Create empty `crates/intrada-core/src/domain/session.rs` and add `pub mod session;` to `crates/intrada-core/src/domain/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types, events, effects, validation, and storage infrastructure that ALL user stories depend on. No user story work can begin until this phase is complete.

**CRITICAL**: No user story work can begin until this phase is complete.

### Core Domain Types

- [x] T003 [P] Add `Session` struct (id, item_id, duration_minutes, started_at, logged_at, notes) with Serialize/Deserialize derives in `crates/intrada-core/src/domain/session.rs`
- [x] T004 [P] Add `SessionsData` struct with `#[serde(default)]` sessions field, `LogSession` input struct (item_id, duration_minutes, notes), and `UpdateSession` input struct (duration_minutes, notes with `Option<Option<String>>`) in `crates/intrada-core/src/domain/types.rs`
- [x] T005 [P] Add `SessionEvent` enum (Log, Update, Delete variants) in `crates/intrada-core/src/domain/session.rs`

### Event, Effect, and Model Wiring

- [x] T006 Add `Event::Session(SessionEvent)` and `Event::SessionsLoaded { sessions: Vec<Session> }` variants to Event enum in `crates/intrada-core/src/app.rs`
- [x] T007 Add `StorageEffect::SaveSession(Session)`, `UpdateSession(Session)`, `DeleteSession { id: String }`, `LoadSessions` variants to StorageEffect enum in `crates/intrada-core/src/app.rs`
- [x] T008 Add `sessions: Vec<Session>` field to `Model` struct in `crates/intrada-core/src/model.rs`
- [x] T009 Add `SessionsLoaded` handler in `update()` to populate `model.sessions` in `crates/intrada-core/src/app.rs`

### Validation

- [x] T010 [P] Add `MIN_DURATION: u32 = 1` and `MAX_DURATION: u32 = 1440` constants, `validate_log_session()` and `validate_update_session()` functions in `crates/intrada-core/src/validation.rs`

### CLI Storage Infrastructure

- [x] T011 [P] Add `load_sessions()`, `save_session()`, `update_session()`, `delete_session()` methods to `JsonStore` with atomic writes to `sessions.json` in `crates/intrada-cli/src/storage.rs`
- [x] T012 Update `Shell::load_data()` to also load sessions and send `Event::SessionsLoaded` in `crates/intrada-cli/src/shell.rs`
- [x] T013 Add match arms for `StorageEffect::SaveSession`, `UpdateSession`, `DeleteSession`, `LoadSessions` in `Shell::handle_effects()` in `crates/intrada-cli/src/shell.rs`

### Web Storage Infrastructure

- [x] T014 [P] Add `load_sessions_data()` and `save_sessions_data()` functions for localStorage key `intrada:sessions` in `crates/intrada-web/src/core_bridge.rs`
- [x] T015 Update web app initialisation to load sessions data and send `Event::SessionsLoaded` in `crates/intrada-web/src/app.rs`
- [x] T016 Add match arms for session StorageEffect variants in web shell effect handling in `crates/intrada-web/src/app.rs`

**Checkpoint**: Foundation ready — all types, events, effects, validation, and storage wiring in place. Compilation should pass with unused warnings. User story implementation can now begin.

---

## Phase 3: User Story 1 — Log a completed practice session (Priority: P1)

**Goal**: A musician can log a practice session (item ID + duration + optional notes) and have it persisted. The most fundamental action.

**Independent Test**: Log a session against a library item, close and reopen the app, verify the session appears with correct duration and notes.

### Implementation for User Story 1

- [x] T017 [US1] Implement `handle_session_event()` for `SessionEvent::Log` variant — validate input, create Session with ULID + timestamps, add to model.sessions, emit `SaveSession` storage effect + render in `crates/intrada-core/src/domain/session.rs`
- [x] T018 [US1] Wire `Event::Session(session_event) => handle_session_event(session_event, model)` in `update()` in `crates/intrada-core/src/app.rs`
- [x] T019 [US1] Add unit tests for session logging: valid log, log without notes, validation errors (duration 0, duration 1441, empty item_id, notes too long) in `crates/intrada-core/src/domain/session.rs`
- [x] T020 [US1] Add `intrada log <item-id> --duration <minutes> [--notes "text"]` CLI subcommand in `crates/intrada-cli/src/main.rs`
- [x] T021 [US1] Add `print_session_logged()` success display function in `crates/intrada-cli/src/display.rs`
- [x] T022 [US1] Add CLI integration tests for log command: valid log, log without notes, validation error in `crates/intrada-cli/tests/` or inline in `crates/intrada-cli/src/main.rs`
- [x] T023 [US1] Add session logging from item detail view (log button + duration/notes form) in `crates/intrada-web/src/views/detail.rs`

**Checkpoint**: User Story 1 complete — sessions can be logged via CLI and web, persisted to sessions.json / localStorage.

---

## Phase 4: User Story 2 — View practice history for a library item (Priority: P2)

**Goal**: A musician can see all sessions for a specific library item, newest-first, with duration, date, and notes. The item detail view shows practice summary (session count + total time).

**Independent Test**: Log three sessions against one item, view the item's detail, verify all three appear in reverse chronological order.

### Implementation for User Story 2

- [x] T024 [P] [US2] Add `SessionView` struct (id, item_id, item_title, item_type, duration_minutes, started_at, logged_at, notes) in `crates/intrada-core/src/model.rs`
- [x] T025 [P] [US2] Add `ItemPracticeSummary` struct (session_count, total_minutes) and add `practice: Option<ItemPracticeSummary>` field to `LibraryItemView` in `crates/intrada-core/src/model.rs`
- [x] T026 [US2] Add `sessions: Vec<SessionView>` field to `ViewModel` and update `view()` to build SessionView list (sorted newest-first) with orphaned item handling ("Deleted item") and compute ItemPracticeSummary per library item in `crates/intrada-core/src/app.rs`
- [x] T027 [US2] Add unit tests for view(): sessions sorted newest-first, orphaned session shows "Deleted item", practice summary counts are correct, empty sessions list in `crates/intrada-core/src/app.rs`
- [x] T028 [US2] Update `print_item_detail()` to show practice summary (session count + total minutes) and session history list, including empty-state message when no sessions exist, in `crates/intrada-cli/src/display.rs`
- [x] T029 [US2] Add `intrada session show <session-id>` CLI subcommand with `print_session_detail()` display in `crates/intrada-cli/src/main.rs` and `crates/intrada-cli/src/display.rs`
- [x] T030 [US2] Create session history component showing sessions for a specific item in `crates/intrada-web/src/components/session_history.rs`
- [x] T031 [US2] Integrate session history component and practice summary into item detail view, including empty-state message when no sessions exist, in `crates/intrada-web/src/views/detail.rs`

**Checkpoint**: User Stories 1 AND 2 complete — sessions can be logged and viewed per item in both CLI and web.

---

## Phase 5: User Story 3 — Use a practice timer (Priority: P3)

**Goal**: The web shell provides a start/stop timer on the item detail view. Timer is client-side only. When stopped, elapsed time is rounded to minutes and logged as a session.

**Independent Test**: Start a timer on an item, wait 10 seconds, stop, add notes, save — verify a session is created.

### Implementation for User Story 3

- [x] T032 [US3] Create practice timer component with start/stop, elapsed time display, notes input, and submit — using Leptos signals (not Crux core) — round to nearest minute on stop in `crates/intrada-web/src/components/practice_timer.rs`
- [x] T033 [US3] Add global timer state (active item ID signal) to prevent multiple simultaneous timers, show warning if timer already active in `crates/intrada-web/src/components/practice_timer.rs`
- [x] T034 [US3] Integrate practice timer component into item detail view in `crates/intrada-web/src/views/detail.rs`
- [x] T035 [US3] Handle timer edge case: duration rounds to 0 minutes (< 30 seconds) — show validation error instead of saving in `crates/intrada-web/src/components/practice_timer.rs`

**Checkpoint**: User Stories 1, 2, AND 3 complete — web users can use the timer for practice sessions.

---

## Phase 6: User Story 4 — View all recent sessions across the library (Priority: P4)

**Goal**: A musician can view all sessions across all library items, newest-first, with item title and type shown.

**Independent Test**: Log sessions against 3 different items, view the "all sessions" list, verify all appear with associated item names.

### Implementation for User Story 4

- [x] T036 [US4] Add `intrada sessions [--item <id>]` CLI subcommand with session list display (filtered by item if flag provided) in `crates/intrada-cli/src/main.rs`
- [x] T037 [US4] Add `print_session_list()` display function showing session date, duration, item title, and notes preview, including empty-state message when no sessions exist, in `crates/intrada-cli/src/display.rs`
- [x] T038 [US4] Create all-sessions list view component with empty-state message when no sessions exist in `crates/intrada-web/src/views/sessions.rs`
- [x] T039 [US4] Add `/sessions` route to web app router and navigation in `crates/intrada-web/src/app.rs`
- [x] T040 [US4] Add unit test for orphaned sessions displaying "Deleted item" placeholder in all-sessions view in `crates/intrada-core/src/app.rs`

**Checkpoint**: User Stories 1–4 complete — full session logging, viewing per item and across library.

---

## Phase 7: User Story 5 — Edit a practice session (Priority: P5)

**Goal**: A musician can edit an existing session's duration and/or notes after creation.

**Independent Test**: Log a session with 30 minutes and no notes, edit it to 45 minutes with notes, verify updated values appear.

### Implementation for User Story 5

- [x] T041 [US5] Implement `handle_session_event()` for `SessionEvent::Update` variant — find session in model, validate updates, apply changes, emit `UpdateSession` storage effect + render in `crates/intrada-core/src/domain/session.rs`
- [x] T042 [US5] Add unit tests for session update: update duration only, update notes only, update both, clear notes, invalid duration, session not found in `crates/intrada-core/src/domain/session.rs`
- [x] T043 [US5] Add `intrada session edit <session-id> [--duration N] [--notes "text"]` CLI subcommand in `crates/intrada-cli/src/main.rs`
- [x] T044 [US5] Add session edit UI (inline edit or modal) to session history and detail views in `crates/intrada-web/src/components/session_history.rs`

**Checkpoint**: User Stories 1–5 complete — sessions can be logged, viewed, and edited.

---

## Phase 8: User Story 6 — Delete a practice session (Priority: P6)

**Goal**: A musician can delete a session they logged by mistake.

**Independent Test**: Log a session, delete it, verify it no longer appears in any view.

### Implementation for User Story 6

- [x] T045 [US6] Implement `handle_session_event()` for `SessionEvent::Delete` variant — remove session from model.sessions, emit `DeleteSession` storage effect + render in `crates/intrada-core/src/domain/session.rs`
- [x] T046 [US6] Add unit tests for session deletion: delete existing session, delete non-existent session in `crates/intrada-core/src/domain/session.rs`
- [x] T047 [US6] Add `intrada session delete <session-id> [-y]` CLI subcommand with confirmation prompt in `crates/intrada-cli/src/main.rs`
- [x] T048 [US6] Add delete button with confirmation to session history entries in `crates/intrada-web/src/components/session_history.rs`

**Checkpoint**: All 6 user stories complete — full session CRUD in both CLI and web.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final quality checks, edge cases, and regression verification.

- [x] T049 Run full test suite (`cargo test`) and fix any failures
- [x] T050 Run `cargo clippy` and fix any warnings
- [x] T051 Run `cargo fmt --all --check` and fix any formatting issues
- [x] T052 Verify existing library tests still pass (no regressions from Event/Effect/Model changes)
- [x] T053 Test orphaned session edge case end-to-end: log session, delete the library item, verify session displays with "Deleted item" placeholder
- [x] T054 Test boundary durations end-to-end: 1 minute (minimum), 1440 minutes (maximum), 0 minutes (rejected), 1441 minutes (rejected)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — can start immediately after Phase 2
- **US2 (Phase 4)**: Depends on US1 (needs sessions in model to display)
- **US3 (Phase 5)**: Depends on US1 (timer logs sessions via SessionEvent::Log)
- **US4 (Phase 6)**: Depends on US2 (needs SessionView in ViewModel)
- **US5 (Phase 7)**: Depends on US1 (needs sessions to exist to edit)
- **US6 (Phase 8)**: Depends on US1 (needs sessions to exist to delete)
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — no dependencies on other stories
- **US2 (P2)**: Depends on US1 (sessions must exist in model to build SessionView)
- **US3 (P3)**: Depends on US1 (timer calls SessionEvent::Log). Can run in parallel with US2
- **US4 (P4)**: Depends on US2 (needs SessionView struct and view() logic)
- **US5 (P5)**: Depends on US1. Can run in parallel with US2, US3, US4
- **US6 (P6)**: Depends on US1. Can run in parallel with US2, US3, US4, US5

### Within Each User Story

- Core domain (events, handlers, tests) before shell integration (CLI, web)
- CLI and web tasks within a story can run in parallel

### Parallel Opportunities

- T003, T004, T005 (core types) can run in parallel
- T010 (validation), T011 (CLI storage), T014 (web storage) can run in parallel
- US3, US5, US6 can each start after US1 without waiting for US2
- Within any story: CLI and web tasks are independent (different crates)

---

## Parallel Example: User Story 1

```bash
# After T018 (core wiring complete), launch CLI and web in parallel:
Task: "T020 [US1] Add intrada log CLI subcommand in crates/intrada-cli/src/main.rs"
Task: "T023 [US1] Add session logging from item detail view in crates/intrada-web/src/views/detail.rs"
```

## Parallel Example: Foundational Phase

```bash
# These touch different files and can run in parallel:
Task: "T003 [P] Add Session struct in crates/intrada-core/src/domain/session.rs"
Task: "T004 [P] Add SessionsData, LogSession, UpdateSession in crates/intrada-core/src/domain/types.rs"
Task: "T010 [P] Add validation functions in crates/intrada-core/src/validation.rs"
Task: "T011 [P] Add sessions storage methods in crates/intrada-cli/src/storage.rs"
Task: "T014 [P] Add sessions localStorage functions in crates/intrada-web/src/core_bridge.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Log a session via CLI and web, verify persistence
5. Commit and potentially deploy

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Test independently → Commit (MVP!)
3. Add US2 → Sessions visible in item detail → Commit
4. Add US3 → Web timer working → Commit
5. Add US4 → All-sessions view → Commit
6. Add US5 → Edit sessions → Commit
7. Add US6 → Delete sessions → Commit
8. Polish → Full test suite green → Ready for PR

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- No new crate dependencies required — uses existing ulid, chrono, serde, serde_json, web-sys
- The practice timer (US3) is web-only and client-side only — it does NOT flow through the Crux core
- Orphaned sessions (FR-011) must be handled in view() — never delete sessions when a library item is deleted
- All session durations are whole minutes (u32), range 1–1440

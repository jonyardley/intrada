# Tasks: Rep History Tracking

**Input**: Design documents from `/specs/104-rep-history/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api-changes.md, quickstart.md

**Tests**: Included — quickstart.md defines 8 specific unit tests and the constitution requires test coverage for each behaviour change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Add new dependency required for integer enum serialisation

- [X] T001 Add `serde_repr` dependency to `crates/intrada-core/Cargo.toml`

---

## Phase 2: Foundational (RepAction Type + Field Additions)

**Purpose**: Define the `RepAction` enum and add `rep_history` fields to all domain, view, and API types. MUST complete before any user story work.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 Define `RepAction` enum with `#[repr(i8)]`, variants `Missed = -1` and `Success = 1`, deriving `Serialize_repr`, `Deserialize_repr`, `Debug`, `Clone`, `Copy`, `PartialEq` in `crates/intrada-core/src/domain/session.rs`
- [X] T003 Add `rep_history: Option<Vec<RepAction>>` field with `#[serde(default)]` to `SetlistEntry` in `crates/intrada-core/src/domain/session.rs`
- [X] T004 [P] Add `rep_history: Option<Vec<RepAction>>` to `SetlistEntryView`, add `current_rep_history: Option<Vec<RepAction>>` to `ActiveSessionView`, and update `entry_to_view()` and `build_active_session_view()` to pass the new fields through in `crates/intrada-core/src/model.rs`

**Checkpoint**: Core domain types now carry `rep_history`. All crates should compile (`cargo check`).

---

## Phase 3: User Story 1 — Full Rep History Capture (Priority: P1) MVP

**Goal**: Record every Got it / Missed action as an ordered sequence on each SetlistEntry, persist through the full data pipeline (core -> API -> DB), and display attempt count in the session summary.

**Independent Test**: Run a session with a rep counter, tap Got it and Missed in various orders, complete the session, verify the attempt history is persisted and displayed in the summary.

### Implementation for User Story 1

- [X] T005 [US1] Update event handlers in `crates/intrada-core/src/domain/session.rs`: `RepGotIt` appends `RepAction::Success` to `rep_history`, `RepMissed` appends `RepAction::Missed`, `EnableRepCounter` initialises `rep_history` to `Some(vec![])`
- [X] T006 [P] [US1] Add DB migration for `rep_history TEXT` column: `ALTER TABLE setlist_entries ADD COLUMN rep_history TEXT;` in `crates/intrada-api/src/migrations.rs`
- [X] T007 [P] [US1] Add `rep_history: Option<Vec<RepAction>>` with `#[serde(default)]` to `SaveSessionEntry`, update `INSERT` SQL to include `rep_history` as parameter ?15 (serialised via `serde_json::to_string`), update `SELECT_COLUMNS` and `SELECT` SQL, update row parsing at column index 14 (deserialise via `serde_json::from_str`) in `crates/intrada-api/src/db/sessions.rs`
- [X] T008 [US1] Add API validation rule: if `rep_target` is `None` but `rep_history` is `Some`, return 400 with message `"rep_history requires rep_target"` in `crates/intrada-api/src/routes/sessions.rs`
- [X] T009 [P] [US1] Display attempt count in session summary: show `" · N attempts"` as muted text after the rep count line when `rep_history.len() != rep_target` in `crates/intrada-web/src/components/session_summary.rs`
- [X] T010 [US1] Write unit tests in `crates/intrada-core/src/domain/session.rs` (tests module): `test_rep_history_appended_on_got_it`, `test_rep_history_appended_on_missed`, `test_rep_history_frozen_on_next_item`, `test_rep_history_persisted_through_save`, `test_rep_history_none_without_counter`, `test_rep_history_initialised_on_first_enable`

**Checkpoint**: Rep history is recorded, persisted, and displayed. `cargo test` passes. US1 is fully functional.

---

## Phase 4: User Story 2 — Preserve Rep State on Hide/Show (Priority: P2)

**Goal**: Hiding the counter preserves all rep state (target, count, reached, history). Re-showing restores it. Only first-time enable sets defaults.

**Independent Test**: Enable a counter, tap Got it several times, hide the counter, re-enable it, verify count and target are unchanged.

### Implementation for User Story 2

- [X] T011 [US2] Rename `EnableRepCounter` to `InitRepCounter` in `SessionEvent` enum and update handler to only initialise defaults when `rep_target.is_none()` (skip init if rep state already exists) in `crates/intrada-core/src/domain/session.rs`
- [X] T012 [US2] Remove `DisableRepCounter` variant from `SessionEvent` enum and remove its handler in `crates/intrada-core/src/domain/session.rs`
- [X] T013 [US2] Replace enable/disable event dispatching with a Leptos `RwSignal<bool>` for counter visibility in `crates/intrada-web/src/components/session_timer.rs`: toggle signal on button tap, dispatch `InitRepCounter` only when showing and `rep_target.is_none()`, remove `DisableRepCounter` dispatch
- [X] T014 [US2] Write unit tests in `crates/intrada-core/src/domain/session.rs` (tests module): `test_enable_preserves_existing_state` (re-init with existing state keeps count/history), `test_disable_preserves_state` (no state clearing on disable/hide)

**Checkpoint**: Hide/show toggles UI only, rep state is always preserved. `cargo test` passes. US2 is fully functional.

---

## Phase 5: User Story 3 — Discoverable Enable Button with Icon (Priority: P3)

**Goal**: Add a visual icon to the "Rep Counter" enable button for discoverability.

**Independent Test**: Start an active session without a rep counter, verify the enable button renders with a icon alongside the label text.

### Implementation for User Story 3

- [X] T015 [US3] Add icon prefix to "Rep Counter" button label: change from `"Rep Counter"` to `"Rep Counter"` in `crates/intrada-web/src/components/session_timer.rs`

**Checkpoint**: Button displays with icon. No behaviour change.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification across all stories

- [X] T016 Run `cargo fmt --check` across workspace and fix any formatting issues
- [X] T017 Run `cargo clippy -- -D warnings` across workspace and fix any lint warnings
- [X] T018 Run `cargo test` across all crates and fix any failures
- [X] T019 Run quickstart.md manual verification steps (active session, crash recovery, session summary)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **User Stories (Phases 3–5)**: All depend on Phase 2 completion
  - US1, US2, US3 can proceed in parallel (independent stories)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 — No dependencies on other stories
- **User Story 2 (P2)**: Can start after Phase 2 — No dependencies on other stories (uses rep_history field from Phase 2, not US1 behaviour)
- **User Story 3 (P3)**: Can start after Phase 2 — No dependencies on other stories. Note: if US2 (T013) changes session_timer.rs first, T015 should be applied to the updated file.

### Within Each User Story

- Core domain changes before API changes
- API migration before SQL query updates
- Core + API before web UI changes
- Implementation before tests (tests verify the implementation)

### Parallel Opportunities

- T003 and T004 (Phase 2): different files (`session.rs` vs `model.rs`)
- T006, T007, T009 (US1): different crates (`migrations.rs` vs `db/sessions.rs` vs `session_summary.rs`)
- All three user stories can start in parallel after Phase 2

---

## Parallel Example: User Story 1

```bash
# After T005 completes (core event handlers), launch API and web tasks in parallel:
Task: "T006 — Add DB migration in crates/intrada-api/src/migrations.rs"
Task: "T007 — Update SaveSessionEntry and SQL in crates/intrada-api/src/db/sessions.rs"
Task: "T009 — Display attempt count in crates/intrada-web/src/components/session_summary.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002–T004)
3. Complete Phase 3: User Story 1 (T005–T010)
4. **STOP and VALIDATE**: Run `cargo test`, verify rep history is captured and displayed
5. Deploy/demo if ready — history tracking is the core value

### Incremental Delivery

1. Setup + Foundational → Foundation ready (T001–T004)
2. Add User Story 1 → Test independently → Deploy (MVP! History tracking works)
3. Add User Story 2 → Test independently → Deploy (Hide/show no longer destroys state)
4. Add User Story 3 → Test independently → Deploy (Button has icon)
5. Polish → Final verification (T016–T019)

### Task Summary

| Phase | Tasks | Count |
|-------|-------|-------|
| Setup | T001 | 1 |
| Foundational | T002–T004 | 3 |
| US1: Full Rep History | T005–T010 | 6 |
| US2: Preserve State | T011–T014 | 4 |
| US3: Button Icon | T015 | 1 |
| Polish | T016–T019 | 4 |
| **Total** | | **19** |

---

## Notes

- [P] tasks = different files, no compile-time dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- `serde_repr` is a lightweight crate in the serde ecosystem — no version conflicts expected
- The `RepAction` enum uses `i8` representation: `Missed = -1`, `Success = 1` (delta-based, per research.md R2)
- Counter visibility is a Leptos signal, not domain state (per research.md R3 and CLAUDE.md state boundary rules)
- Attempt count display only appears when attempts differ from target (per research.md R5)

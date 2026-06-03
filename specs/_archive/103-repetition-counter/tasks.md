# Tasks: Repetition Counter

**Input**: Design documents from `/specs/103-repetition-counter/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested — tests are covered by existing `cargo test` in the polish phase.

**Organization**: Tasks grouped by user story. US4 (Persist) is absorbed into the Foundational phase since it's infrastructure that blocks US3 and delivers no direct user value alone.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Foundational — Domain Model & Persistence (US4 absorbed)

**Purpose**: Add rep fields to the domain model, extend persistence, and run DB migrations. All user stories depend on this phase.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T001 [P] Add validation constants (`DEFAULT_REP_TARGET=5`, `MIN_REP_TARGET=3`, `MAX_REP_TARGET=10`) and `validate_rep_target()` function in `crates/intrada-core/src/validation.rs`
- [X] T002 [P] Add 3 new DB migrations (0021: `rep_target INTEGER`, 0022: `rep_count INTEGER`, 0023: `rep_target_reached INTEGER`) on `setlist_entries` table in `crates/intrada-api/src/migrations.rs`
- [X] T003 Add `rep_target: Option<u8>`, `rep_count: Option<u8>`, `rep_target_reached: Option<bool>` fields with `#[serde(default)]` to `SetlistEntry` in `crates/intrada-core/src/domain/session.rs` — update all construction sites within intrada-core (struct literals, state transitions Building→Active→Summary→Save)
- [X] T004 [P] Add `rep_target`, `rep_count`, `rep_target_reached` to `SetlistEntryView` and update `entry_to_view()` to pass through the new fields in `crates/intrada-core/src/app.rs`
- [X] T005 [P] Extend `SaveSessionEntry` with 3 new `#[serde(default)]` fields, update `SELECT_COLUMNS` (add indices 11–13), update `row_to_entry()` to read `rep_target`/`rep_count`/`rep_target_reached`, update INSERT SQL in `crates/intrada-api/src/routes/sessions.rs`
- [X] T006 Update `SetlistEntry` construction in wasm test helpers to include `rep_target: None, rep_count: None, rep_target_reached: None` in `crates/intrada-web/tests/wasm.rs`

**Checkpoint**: `cargo test` passes — all existing tests green with new fields defaulting to `None`. DB can persist and retrieve rep data. Backward compatibility confirmed (existing sessions load with `None` for all rep fields).

---

## Phase 2: User Story 1 — Counter During Active Practice (Priority: P1) 🎯 MVP

**Goal**: Musician can enable a repetition counter during active practice, tap "got it" / "missed" to track consecutive correct reps, and see an achievement indicator when the target is reached.

**Independent Test**: Start a session with one item, enable the counter, tap "got it" until 5/5 is reached, verify achievement prompt appears, dismiss it, verify counter freezes and timer continues.

### Implementation for User Story 1

- [X] T007 [US1] Add `RepGotIt`, `RepMissed`, `EnableRepCounter`, `DisableRepCounter` variants to `SessionEvent` and implement their handlers in `crates/intrada-core/src/domain/session.rs` — RepGotIt: increment `rep_count` capped at `rep_target`, set `rep_target_reached=true` when count equals target; RepMissed: decrement `rep_count` clamped to 0; EnableRepCounter: set `rep_count=Some(0)`, `rep_target_reached=Some(false)` if target exists; DisableRepCounter: set `rep_target=None`, `rep_count=None`, `rep_target_reached=None`
- [X] T008 [US1] Initialise `rep_count=Some(0)` and `rep_target_reached=Some(false)` for entries with `rep_target.is_some()` when transitioning from Building to Active phase in `crates/intrada-core/src/domain/session.rs`
- [X] T009 [US1] Freeze rep state on item transitions — when processing `NextItem`, `SkipItem`, or `FinishSession`, set `rep_target_reached=Some(true)` if `rep_count >= rep_target` on current entry, then freeze (no further mutations) in `crates/intrada-core/src/domain/session.rs`
- [X] T010 [US1] Add `current_rep_target: Option<u8>`, `current_rep_count: Option<u8>`, `current_rep_target_reached: Option<bool>` to `ActiveSessionView` and update `build_active_session_view()` to populate from current entry in `crates/intrada-core/src/app.rs`
- [X] T011 [US1] Add RepCounter UI to active practice view in `crates/intrada-web/src/components/session_timer.rs` — show count ("3 / 5") when counter enabled, "Got it" (accent) and "Missed" (subdued) buttons (≥44px touch targets), enable/disable toggle link, achievement state with "Next Item" prompt when target reached, hide buttons when `rep_target_reached=true`

**Checkpoint**: Counter works end-to-end during active practice. Got-it increments, missed decrements (floor 0), target reached shows achievement, dismiss freezes counter. Timer continues after achievement.

---

## Phase 3: User Story 2 — Configure Target Per Item (Priority: P2)

**Goal**: Musician can set different rep targets (3–10) for individual setlist entries during the building phase via an opt-in toggle.

**Independent Test**: Add two items, set target 3 on the first and 7 on the second, start session, verify first item shows 0/3 and second shows 0/7.

### Implementation for User Story 2

- [X] T012 [US2] Add `SetRepTarget { entry_id: String, target: Option<u8> }` variant to `SessionEvent` and implement handler in building phase — validate target with `validate_rep_target()`, set or clear `rep_target` on the matching entry in `crates/intrada-core/src/domain/session.rs`
- [X] T013 [US2] Add RepTargetInput UI per entry in `crates/intrada-web/src/components/setlist_builder.rs` — small "Add rep target" link below the intention input, tapping reveals a numeric stepper (3–10, default 5) and a remove option, dispatch `SetRepTarget` event on change

**Checkpoint**: Building phase shows opt-in rep target per entry. Targets carry through to active phase. Entries without targets show no counter.

---

## Phase 4: User Story 3 — View Rep Count in Summary and History (Priority: P3)

**Goal**: Final rep count and target-achieved status are visible in session summary and session history views.

**Independent Test**: Complete a session with counter (reach target on one item, skip another mid-count), verify summary shows "5/5 ✓" and "2/5", save, verify history shows rep data alongside score and duration.

### Implementation for User Story 3

- [X] T014 [P] [US3] Add rep count display per entry in session summary — show "Reps: 5/5 ✓" (target achieved) or "Reps: 2/5" (partial), hide when `rep_target` is `None`, position near score/duration in `crates/intrada-web/src/components/session_summary.rs`
- [X] T015 [P] [US3] Add rep count display in session history detail — show rep data alongside existing score, duration, and notes when `rep_target.is_some()` in `crates/intrada-web/src/views/sessions.rs`

**Checkpoint**: Summary and history show rep data for entries that used the counter. Entries without counter show no rep information. Existing sessions display correctly.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Formatting, linting, test verification, and manual validation.

- [X] T016 Run `cargo fmt --check` and fix any formatting issues
- [X] T017 Run `cargo clippy -- -D warnings` and fix any warnings
- [X] T018 Run `cargo test` and fix any test failures
- [X] T019 Manual verification per `specs/103-repetition-counter/quickstart.md` — counter during practice (US1), configure target (US2), summary/history (US3), crash recovery (FR-007), backward compatibility (FR-011)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — can start immediately. **BLOCKS all user stories.**
- **US1 (Phase 2)**: Depends on Foundational completion.
- **US2 (Phase 3)**: Depends on Foundational completion. Independent of US1 (different event, different UI component).
- **US3 (Phase 4)**: Depends on Foundational completion. Best done after US1+US2 so summary/history can show real data.
- **Polish (Phase 5)**: Depends on all user stories being complete.

### User Story Dependencies

- **US1 (P1)**: After Foundational — no dependencies on other stories
- **US2 (P2)**: After Foundational — no dependencies on other stories (can run in parallel with US1)
- **US3 (P3)**: After Foundational — no dependencies on US1/US2 (view logic only reads existing fields), but best done after for realistic testing
- **US4 (P4)**: Absorbed into Foundational — persistence is infrastructure

### Within Each User Story

- Core domain changes before ViewModel changes
- ViewModel changes before UI components
- Session.rs changes before app.rs changes
- Complete story before moving to next priority

### Parallel Opportunities

**Foundational phase:**
- T001 (validation) and T002 (migrations) can run in parallel (different crates)
- T004 (views) and T005 (API) can run in parallel after T003 (different crates)

**User story phases:**
- US1 and US2 can run in parallel (different events, different UI components)
- T014 and T015 within US3 can run in parallel (different files)

---

## Parallel Example: Foundational Phase

```text
# Parallel batch 1:
T001: Add validation constants in validation.rs
T002: Add DB migrations in migrations.rs

# Sequential (depends on T001):
T003: Add fields to SetlistEntry in session.rs

# Parallel batch 2 (depends on T003):
T004: Add fields to SetlistEntryView in app.rs
T005: Update API persistence in routes/sessions.rs
T006: Update wasm test helpers
```

## Parallel Example: User Stories

```text
# After Foundational is complete, US1 and US2 can start in parallel:
# Stream A (US1): T007 → T008 → T009 → T010 → T011
# Stream B (US2): T012 → T013

# After both complete:
# US3: T014 + T015 (parallel)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Foundational (domain model + persistence)
2. Complete Phase 2: User Story 1 (counter during active practice)
3. **STOP and VALIDATE**: Test counter works end-to-end
4. Counter is usable with default target of 5

### Incremental Delivery

1. Foundational → Domain model, persistence, backward compat ✓
2. US1 → Counter works during practice → Validate (MVP!)
3. US2 → Targets configurable per item → Validate
4. US3 → Summary and history show rep data → Validate
5. Polish → fmt, clippy, test, manual verification → Done

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US4 absorbed into Foundational — persistence is blocking infrastructure, not user-facing
- Crash recovery works automatically — rep fields on SetlistEntry are serialised via `#[serde(default)]` to the existing localStorage crash recovery key
- No new files created — all changes extend existing modules per plan.md structure decision
- Counter state is pure in Crux core — no new effects needed for counter interactions

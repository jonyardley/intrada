# Tasks: Focus Mode

**Input**: Design documents from `/specs/048-focus-mode/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/api-changes.md, research.md, quickstart.md

**Tests**: Not explicitly requested — test tasks omitted. Validation via quickstart.md in Polish phase.

**Organization**: Tasks grouped by user story. US1+US2 are both P1 but separated into distinct phases — US1 (minimal UI) can be tested independently before the progress ring (US2) is added.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Exact file paths included in every task description

---

## Phase 1: Setup

**Purpose**: Verify environment and branch readiness

- [ ] T001 Verify 048-focus-mode branch is checked out, run `cargo test` to confirm baseline passes

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add `planned_duration_secs` across the full stack (DB → API → Core → Web builder). MUST be complete before any user story work begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T002 [P] Add `MIN_PLANNED_DURATION_SECS` (60) and `MAX_PLANNED_DURATION_SECS` (3600) validation constants in `crates/intrada-core/src/validation.rs`
- [ ] T003 [P] Add `planned_duration_secs: Option<u32>` field to `SetlistEntry` struct and add `SetEntryDuration { entry_id: String, duration_secs: Option<u32> }` variant to `SessionEvent` in `crates/intrada-core/src/domain/session.rs`
- [ ] T004 [P] Add migration 0025: `ALTER TABLE setlist_entries ADD COLUMN planned_duration_secs INTEGER` in `crates/intrada-api/src/migrations.rs`
- [ ] T005 Handle `SetEntryDuration` event in the session update function — set `entry.planned_duration_secs` with validation against min/max constants in `crates/intrada-core/src/app.rs` (depends on T002, T003)
- [ ] T006 Update `SetlistEntryView` (add `planned_duration_secs: Option<u32>`, `planned_duration_display: Option<String>`) and `ActiveSessionView` (add `current_planned_duration_secs: Option<u32>`, `next_item_title: Option<String>`) in `crates/intrada-core/src/model.rs` (depends on T003)
- [ ] T007 Update `SELECT_COLUMNS`, SQL queries, and row parsing to include `planned_duration_secs` column in `crates/intrada-api/src/db/sessions.rs` (depends on T004)
- [ ] T008 Add `planned_duration_secs: Option<u32>` to `SaveSessionEntry` struct and update request deserialization / response serialization in `crates/intrada-api/src/routes/sessions.rs` (depends on T007)
- [ ] T009 Add optional per-entry duration input (minutes, 1–60 range) to the setlist builder matching the rep target "Add duration" / "Remove" pattern in `crates/intrada-web/src/components/setlist_builder.rs` (depends on T005, T006)

**Checkpoint**: `cargo test` passes. Duration field flows end-to-end: builder → core model → API → DB. Existing sessions unaffected (NULL default).

---

## Phase 3: User Story 1 — Distraction-Free Practice (Priority: P1) 🎯 MVP

**Goal**: Strip the active practice screen to essentials — current item name, timer, rep counter (when active), and session controls. Hide navigation bar, completed items list, and non-essential UI.

**Independent Test**: Start any practice session → verify only essential elements visible, nav bar hidden, completed items list hidden.

### Implementation for User Story 1

- [ ] T010 [P] [US1] Add `focus_mode: RwSignal<bool>` to app-level Leptos context (provided from `AuthenticatedApp`). Wrap `AppHeader`, `AppFooter`, and `BottomTabBar` rendering in `Show` conditioned on `!focus_mode.get()` in `crates/intrada-web/src/app.rs`
- [ ] T011 [P] [US1] Add focus mode layout styles — vertically centred content container, generous whitespace, hidden-nav overrides — in `crates/intrada-web/input.css`
- [ ] T012 [US1] Set `focus_mode` signal to `true` on mount and `false` on unmount (via `on_cleanup`) in `crates/intrada-web/src/views/session_active.rs` (depends on T010)
- [ ] T013 [US1] Restructure `SessionTimer` component for focused layout: item name + type badge at top, timer centre, rep counter (when active), control buttons below. Remove completed items list and intention text from default render in `crates/intrada-web/src/components/session_timer.rs` (depends on T010, T012)

**Checkpoint**: Start a session → nav bar hidden, completed items hidden, only item name + digital timer + controls visible. Focus mode is the default. `cargo test` passes.

---

## Phase 4: User Story 2 — Visual Progress Timer (Priority: P1)

**Goal**: Add a circular progress ring around the timer that fills as elapsed time approaches the planned duration. Digital timer (MM:SS) sits centred inside the ring. Items without a planned duration show digital timer only.

**Independent Test**: Start a session with an item that has a planned duration → verify ring shows and fills proportionally. Start with an item without duration → verify no ring, just digital timer.

### Implementation for User Story 2

- [ ] T014 [P] [US2] Create `ProgressRing` component — SVG `<circle>` with `stroke-dasharray` / `stroke-dashoffset` driven by `elapsed_secs` and `planned_duration_secs` signals, digital timer (MM:SS) centred via absolute positioning inside the ring in `crates/intrada-web/src/components/progress_ring.rs`
- [ ] T015 [P] [US2] Add progress ring CSS — `stroke-dashoffset` transition for smooth animation, ring sizing tokens (mobile vs desktop), colour using `--color-progress-fill` / `--color-progress-track` design tokens in `crates/intrada-web/input.css`
- [ ] T016 [US2] Export `ProgressRing` component from `crates/intrada-web/src/components/mod.rs` (depends on T014)
- [ ] T017 [US2] Integrate `ProgressRing` into `SessionTimer` — render ring when `current_planned_duration_secs.is_some()`, render digital timer only otherwise. Pass `elapsed_secs` and `planned_duration_secs` as signals in `crates/intrada-web/src/components/session_timer.rs` (depends on T013, T014, T016)

**Checkpoint**: Item with 2-min duration shows filling ring. Item without duration shows digital timer only. Ring at ~50% after half the time. `cargo test` passes.

---

## Phase 5: User Story 3 — Reveal Full Controls on Demand (Priority: P2)

**Goal**: Add a toggle button (chevron icon) that temporarily reveals hidden UI elements (navigation bar, completed items list, session intention) without leaving the session.

**Independent Test**: While in focus mode, click toggle → verify nav bar and completed items reappear. Click again → verify return to focused state.

### Implementation for User Story 3

- [ ] T018 [US3] Add a toggle button (chevron icon, positioned below session controls) that sets `focus_mode` signal to `false` when clicked, and back to `true` on second click, in `crates/intrada-web/src/components/session_timer.rs` (depends on T013)
- [ ] T019 [US3] Ensure expanded state (focus_mode = false) renders completed items list and session intention text below controls in `crates/intrada-web/src/components/session_timer.rs` (depends on T018)

**Checkpoint**: Toggle button reveals/hides navigation + completed items + intention in one click each. Controls remain accessible in both states.

---

## Phase 6: User Story 4 — Gentle Transition Prompts (Priority: P2)

**Goal**: When an item's planned duration elapses, show a non-blocking visual prompt with the next item's name (or "session complete" for the last item).

**Independent Test**: Set a 1-minute duration on an item, wait for it to elapse → verify transition prompt appears with next item name. Verify prompt does not block timer or controls.

### Implementation for User Story 4

- [ ] T020 [P] [US4] Create `TransitionPrompt` component — subtle banner/card showing "Up next: [Item Name]" or "Session complete — ready to finish?" when `duration_elapsed` is true. Non-blocking layout between timer and controls in `crates/intrada-web/src/components/transition_prompt.rs`
- [ ] T021 [US4] Export `TransitionPrompt` component from `crates/intrada-web/src/components/mod.rs` (depends on T020)
- [ ] T022 [US4] Add `duration_elapsed: RwSignal<bool>` signal driven by `elapsed_secs >= planned_duration_secs` comparison on timer tick. Integrate `TransitionPrompt` into `SessionTimer` — show when elapsed and planned duration exists, clear when advancing to next item in `crates/intrada-web/src/components/session_timer.rs` (depends on T017, T020, T021)

**Checkpoint**: 1-min duration item elapses → prompt appears with next item name. Last item → prompt says "session complete". Prompt doesn't block controls. Items without duration → no prompt.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Design catalogue, code quality, and end-to-end validation

- [ ] T023 [P] Add `ProgressRing` and `TransitionPrompt` showcase entries to `crates/intrada-web/src/views/design_catalogue.rs`
- [ ] T024 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` across workspace — fix any issues
- [ ] T025 Run quickstart.md verification scenarios (all 8 sections) end-to-end in the browser

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Phase 2 completion
- **US2 (Phase 4)**: Depends on Phase 3 (US1) — progress ring integrates into the restructured SessionTimer
- **US3 (Phase 5)**: Depends on Phase 3 (US1) — toggle button operates on the focus_mode signal
- **US4 (Phase 6)**: Depends on Phase 4 (US2) — transition prompt integrates alongside progress ring in SessionTimer
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational → independently testable (minimal UI, no ring needed)
- **US2 (P1)**: Depends on US1 → SessionTimer must be restructured before ring is integrated
- **US3 (P2)**: Depends on US1 → toggle button controls the focus_mode signal from US1
- **US4 (P2)**: Depends on US2 → transition prompt lives alongside progress ring in SessionTimer

### Within Each Phase

- Tasks marked [P] within the same phase can run in parallel
- Non-[P] tasks have explicit dependency notes
- Complete all tasks in a phase before moving to the next

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T002 + T003 + T004 can all run in parallel (different files, different crates)
- After T003: T005 and T006 can run in parallel
- After T004: T007 runs, then T008

**Phase 3 (US1)**:
- T010 + T011 can run in parallel (app.rs vs input.css)

**Phase 4 (US2)**:
- T014 + T015 can run in parallel (new component file vs input.css)

**Phase 6 (US4)**:
- T020 can start while Phase 5 is finishing (new file, no conflicts)

---

## Parallel Example: Foundational Phase

```text
# Wave 1 — all different files, no dependencies:
Task T002: "Add validation constants in validation.rs"
Task T003: "Add planned_duration_secs + SetEntryDuration event in domain/session.rs"
Task T004: "Add migration 0025 in migrations.rs"

# Wave 2 — depends on Wave 1:
Task T005: "Handle SetEntryDuration event in app.rs" (needs T002, T003)
Task T006: "Update view models in model.rs" (needs T003)
Task T007: "Update SQL + row parsing in db/sessions.rs" (needs T004)

# Wave 3 — depends on Wave 2:
Task T008: "Update SaveSessionEntry in routes/sessions.rs" (needs T007)
Task T009: "Add duration input to setlist_builder.rs" (needs T005, T006)
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (planned_duration_secs across full stack)
3. Complete Phase 3: US1 — Distraction-Free Practice
4. **STOP and VALIDATE**: Focus mode works — minimal UI, nav hidden, controls accessible
5. This is a usable, testable increment even without the progress ring

### Incremental Delivery

1. Setup + Foundational → Duration field works end-to-end
2. Add US1 → Minimal focus UI during practice (MVP!)
3. Add US2 → Visual progress ring for timed items
4. Add US3 → Toggle to reveal hidden controls
5. Add US4 → Transition prompts when time elapses
6. Polish → Design catalogue, code quality, full E2E validation

### File Touch Summary

| File | Phases | Tasks |
|------|--------|-------|
| `intrada-core/src/validation.rs` | 2 | T002 |
| `intrada-core/src/domain/session.rs` | 2 | T003 |
| `intrada-core/src/app.rs` | 2 | T005 |
| `intrada-core/src/model.rs` | 2 | T006 |
| `intrada-api/src/migrations.rs` | 2 | T004 |
| `intrada-api/src/db/sessions.rs` | 2 | T007 |
| `intrada-api/src/routes/sessions.rs` | 2 | T008 |
| `intrada-web/src/components/setlist_builder.rs` | 2 | T009 |
| `intrada-web/src/app.rs` | 3 | T010 |
| `intrada-web/input.css` | 3, 4 | T011, T015 |
| `intrada-web/src/views/session_active.rs` | 3 | T012 |
| `intrada-web/src/components/session_timer.rs` | 3, 4, 5, 6 | T013, T017, T018, T019, T022 |
| `intrada-web/src/components/progress_ring.rs` | 4 | T014 *(new file)* |
| `intrada-web/src/components/mod.rs` | 4, 6 | T016, T021 |
| `intrada-web/src/components/transition_prompt.rs` | 6 | T020 *(new file)* |
| `intrada-web/src/views/design_catalogue.rs` | 7 | T023 |

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks in the same wave
- [Story] label maps task to its user story for traceability
- session_timer.rs is the most-touched file (5 tasks across 4 phases) — tasks are ordered to avoid conflicts
- No new external dependencies — progress ring uses inline SVG, transitions use CSS
- Focus mode toggle state is ephemeral (Leptos signal) — not persisted to API
- planned_duration_secs is domain data — persisted through crash recovery and API

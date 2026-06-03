# Tasks: Drag-and-Drop Session Builder

**Input**: Design documents from `/specs/026-drag-drop-builder/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: No tests explicitly requested in the spec. Tests are omitted.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Add web-sys features and create module scaffolding for the drag-and-drop hook

- [x] T001 Add web-sys features (`PointerEvent`, `Element`, `DomRect`, `HtmlElement`) to `crates/intrada-web/Cargo.toml`, preserving existing features
- [x] T002 Create hooks module at `crates/intrada-web/src/hooks/mod.rs` and declare it in `crates/intrada-web/src/lib.rs`
- [x] T003 Rebase branch onto latest `main` (which includes 025-reusable-routines) to get `RoutineEditView` and related code

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the reusable drag-and-drop hook and new UI components that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Implement `DragState` struct and `use_drag_reorder` hook in `crates/intrada-web/src/hooks/use_drag_reorder.rs` — encapsulates `RwSignal<Option<DragState>>`, pointer event handler closures (`on_pointer_down`, `on_pointer_move`, `on_pointer_up`, `on_pointer_cancel`), hover-index computation from element bounding rects, and exposes derived signals (`is_dragging`, `dragged_id`, `hover_index`). The hook accepts a reorder callback `Callback<(String, usize)>` (entry_id, new_position) and an `item_count: Signal<usize>`.
- [x] T005 [P] Create `DragHandle` component in `crates/intrada-web/src/components/drag_handle.rs` — renders a six-dot grip SVG icon, applies `touch-action: none`, `user-select: none`, `cursor: grab` CSS, minimum 44x44px touch target, and attaches `on:pointerdown` from the hook. Include `role="button"` and `aria-label="Drag to reorder"` for accessibility.
- [x] T006 [P] Create `DropIndicator` component in `crates/intrada-web/src/components/drop_indicator.rs` — renders a 2px `bg-indigo-400` horizontal line with `motion-safe:transition-all` and `motion-safe:animate-in`. Accepts a `visible: Signal<bool>` prop.
- [x] T007 Export `DragHandle` and `DropIndicator` from `crates/intrada-web/src/components/mod.rs`

**Checkpoint**: Foundation ready — drag hook and new components available for integration

---

## Phase 3: User Story 1 — Drag-and-Drop Setlist Reordering (Priority: P1) 🎯 MVP

**Goal**: Musicians can reorder setlist entries by dragging them to a new position during the session building phase

**Independent Test**: Build a session with 4+ items, drag one to a new position, and confirm the setlist reflects the new order (quickstart V2)

### Implementation for User Story 1

- [x] T008 [US1] Modify `SetlistEntryRow` in `crates/intrada-web/src/components/setlist_entry.rs` — add `DragHandle` as the leftmost element (before position number), add new props for drag state signals (`is_dragging_this: Signal<bool>`, `drag_handle_props`), apply `opacity-50 ring-2 ring-indigo-400` CSS class when `is_dragging_this` is true
- [x] T009 [US1] Integrate `use_drag_reorder` hook into `SetlistBuilder` in `crates/intrada-web/src/components/setlist_builder.rs` — initialise the hook with a reorder callback that dispatches `SessionEvent::ReorderSetlist { entry_id, new_position }` via `process_effects`, pass drag state signals to each `SetlistEntryRow`, render `DropIndicator` between entries based on `hover_index` signal
- [x] T010 [US1] Add drag-active and drop-target CSS utility classes to `crates/intrada-web/styles/app.css` — `.drag-active` (opacity, ring highlight), `.drop-indicator` (height, colour, transition), wrap transitions in `@media (prefers-reduced-motion: no-preference)` for FR-011
- [x] T011 [US1] Handle edge cases in the hook: single-item list (no-op), drag beyond bounds (clamp to 0 or len-1), release outside setlist area (cancel and restore via `pointercancel`/`lostpointercapture`)
- [x] T012 [US1] Verify `cargo fmt --check && cargo clippy -- -D warnings && cargo test` all pass — existing core tests must not be affected (SC-006)

**Checkpoint**: Drag-and-drop reordering works in the session builder. Arrow buttons still work. Core tests pass.

---

## Phase 4: User Story 2 — Tap Entire Library Row to Add (Priority: P2)

**Goal**: Musicians can add a library item to the setlist by tapping anywhere on the library item row, not just the small "+ Add" button

**Independent Test**: Tap on a library item row (not the button) and confirm the item appears in the setlist (quickstart V4)

### Implementation for User Story 2

- [x] T013 [US2] Modify library item rows in `SetlistBuilder` in `crates/intrada-web/src/components/setlist_builder.rs` — wrap the outer `<div>` in a clickable element (or move the `on:click` handler to the row `<div>` itself), add `cursor-pointer` class, preserve the existing `hover:bg-white/10` visual affordance, keep the "+ Add" button text visible for clarity (FR-006)
- [x] T014 [US2] Apply the same tap-to-add change to the "Add from Library" section in `RoutineEditView` at `crates/intrada-web/src/views/routine_edit.rs` — make the library item rows fully clickable with `cursor-pointer` and `hover:bg-white/10`
- [x] T015 [US2] Verify the "+ Add" button click does not double-fire (since the row click now also triggers add). Use `ev.stop_propagation()` on the button if needed, or remove the separate button handler and rely solely on the row handler
- [x] T016 [US2] Verify `cargo fmt --check && cargo clippy -- -D warnings && cargo test` all pass

**Checkpoint**: Library rows are fully tappable in both session builder and routine edit. Arrow buttons and drag still work.

---

## Phase 5: User Story 3 — Touch-Friendly Drag Handle (Priority: P3)

**Goal**: On mobile devices, drag only initiates from the drag handle, preserving normal page scrolling

**Independent Test**: On a mobile viewport, scroll through a long setlist without triggering drag, then drag via the handle (quickstart V5)

### Implementation for User Story 3

- [x] T017 [US3] Verify `touch-action: none` and `user-select: none` CSS are applied to the `DragHandle` component (should already be set in T005) and NOT applied to the entry row body
- [x] T018 [US3] Add a 5px movement threshold in `use_drag_reorder` hook at `crates/intrada-web/src/hooks/use_drag_reorder.rs` — after `pointerdown`, do not commit to drag mode until `pointermove` exceeds 5px vertical distance from `start_y`, preventing accidental drags from imprecise touches
- [x] T019 [US3] Add `on:contextmenu` handler with `ev.prevent_default()` on the `DragHandle` component to suppress long-press context menus on mobile
- [x] T020 [US3] Verify `cargo fmt --check && cargo clippy -- -D warnings && cargo test` all pass

**Checkpoint**: Touch scrolling works normally. Drag only initiates from the handle with sufficient movement.

---

## Phase 6: Routine Edit Page Integration (FR-010)

**Purpose**: Apply drag-and-drop reordering to the routine edit page entry list

**Note**: Depends on 025-reusable-routines being merged to main (T003 rebase)

- [x] T021 [US1] Integrate `use_drag_reorder` hook into `RoutineEditView` in `crates/intrada-web/src/views/routine_edit.rs` — initialise the hook with a reorder callback that updates the local `entries: RwSignal<Vec<RoutineEntryView>>` via `entries.update(|e| { e.remove(src); e.insert(dst, entry); })`, add `DragHandle` to each entry row, render `DropIndicator` between entries based on `hover_index`
- [x] T022 [US1] Ensure the routine edit entry rows show drag handles AND up/down arrow buttons (always visible, FR-012) — update the inline entry row markup to include `DragHandle` at the leftmost position
- [x] T023 Verify `cargo fmt --check && cargo clippy -- -D warnings && cargo test` all pass

**Checkpoint**: Drag-and-drop works on both session builder and routine edit page.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and quality checks

- [x] T024 [P] Add `aria-roledescription="sortable"` to the setlist container div and `aria-grabbed` state to entry rows for screen reader support in `crates/intrada-web/src/components/setlist_builder.rs`
- [x] T025 [P] Verify `prefers-reduced-motion` suppresses all drag transitions by testing with `@media (prefers-reduced-motion: reduce)` override in browser dev tools (quickstart V6)
- [x] T026 Run full quickstart.md verification (V1–V10)
- [x] T027 Run `cargo fmt --check && cargo clippy -- -D warnings && cargo test` final CI check

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase
- **User Story 2 (Phase 4)**: Depends on Foundational phase — can run in parallel with US1
- **User Story 3 (Phase 5)**: Depends on Foundational phase (and US1 for drag handle to exist)
- **Routine Edit (Phase 6)**: Depends on T003 (rebase) + Foundational phase — can start after US1
- **Polish (Phase 7)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) — independent of US1
- **User Story 3 (P3)**: Logically depends on US1 (drag handle must exist) — best done after Phase 3
- **Routine Edit (FR-010)**: Depends on 025 being merged (T003) and US1 foundational components

### Within Each User Story

- Hook and components before integration
- Integration before edge case handling
- Edge cases before CI verification

### Parallel Opportunities

- T005 and T006 (DragHandle and DropIndicator) can run in parallel
- T013 and T014 (library row changes in setlist builder and routine edit) can run in parallel
- T024 and T025 (ARIA and reduced-motion) can run in parallel
- US1 and US2 can run in parallel after Foundational phase

---

## Parallel Example: Foundational Phase

```bash
# Launch DragHandle and DropIndicator in parallel (different files):
Task: "Create DragHandle component in crates/intrada-web/src/components/drag_handle.rs"
Task: "Create DropIndicator component in crates/intrada-web/src/components/drop_indicator.rs"
```

## Parallel Example: User Story 2

```bash
# Launch library row changes in parallel (different files):
Task: "Modify library rows in SetlistBuilder in crates/intrada-web/src/components/setlist_builder.rs"
Task: "Modify library rows in RoutineEditView in crates/intrada-web/src/views/routine_edit.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T007)
3. Complete Phase 3: User Story 1 (T008–T012)
4. **STOP and VALIDATE**: Test drag-and-drop reordering independently (quickstart V1, V2, V3)
5. This delivers the primary value of Issue #37

### Incremental Delivery

1. Complete Setup + Foundational → Hook and components ready
2. Add User Story 1 → Drag-and-drop reorder works → Validate (MVP!)
3. Add User Story 2 → Library rows fully tappable → Validate
4. Add User Story 3 → Touch scroll preserved → Validate
5. Add Routine Edit → Drag-and-drop on routine page → Validate
6. Polish → ARIA, reduced-motion, final quickstart → Ship

---

## Notes

- This is a **shell-only feature** — `intrada-core` and `intrada-api` are NOT modified
- The existing `ReorderSetlist` core event is reused as-is
- The routine edit page uses a different reorder mechanism (local signal swap) but the same drag hook
- Auto-scrolling during drag on long lists is deferred (nice-to-have per spec Assumptions)
- All CSS transitions MUST be wrapped in `motion-safe:` for `prefers-reduced-motion` compliance

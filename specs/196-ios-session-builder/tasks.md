# Tasks: iOS Session Builder

**Input**: Design documents from `/specs/196-ios-session-builder/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Not explicitly requested. iOS validation via `just ios-swift-check`, `just ios-smoke-test`, and SwiftUI Previews.

**Organization**: Tasks grouped by user story. Each story is independently implementable and testable after the foundational phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create directory structure and shared helper files for the session builder feature.

- [x] T001 Create Practice views directory at `ios/Intrada/Views/Practice/`
- [x] T002 [P] Create `PracticeTabRouter.swift` — state-driven switch rendering Idle/Building/Active/Summary views based on `session_status` from ViewModel in `ios/Intrada/Views/Practice/PracticeTabRouter.swift`
- [x] T003 [P] Update `MainTabView.swift` to use `PracticeTabRouter` for the Practice tab instead of placeholder, and add accent dot indicator when `session_status` is `.active` or `.building` in `ios/Intrada/Navigation/MainTabView.swift`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared components that multiple user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 [P] Create `LibraryQueueRow` component — tappable row with title, subtitle, type badge, toggle state (unselected: + icon, selected: accent left bar + check-circle icon) in `ios/Intrada/Components/LibraryQueueRow.swift`
- [x] T005 [P] Create `StickyBottomBar` component — persistent bar with item count, total time, "Tap to edit setlist" hint, and "Start Session →" button (disabled when count is 0) in `ios/Intrada/Components/StickyBottomBar.swift`
- [x] T006 [P] Create `SetlistEntryRow` component — compact row with drag handle (grip-vertical icon), title, duration/reps metadata, type badge, and × remove button. Tappable to expand for editing (progressive disclosure) in `ios/Intrada/Components/SetlistEntryRow.swift`
- [x] T007 Run `just ios-swift-check` to verify all new components compile

**Checkpoint**: All shared components exist and compile. User story implementation can begin.

---

## Phase 3: User Story 1 — Build a setlist from library items (Priority: P1) 🎯 MVP

**Goal**: Musicians can tap library items to queue them, see selection state, and start a session. Minimum friction: 3 taps to select + 1 tap to start.

**Independent Test**: Open builder → tap 3 items → verify accent bars + check icons appear → verify bottom bar shows "3 items" → tap "Start Session" → verify transition to active state.

### Implementation for User Story 1

- [x] T008 [US1] Create `SessionBuilderView.swift` — main builder view. On appear, dispatch `Event.session(.startBuilding)`. Use `@Environment(\.horizontalSizeClass)` to switch between iPhone (single column + bottom bar) and iPad (HStack split view) layouts in `ios/Intrada/Views/Practice/SessionBuilderView.swift`
- [x] T009 [US1] Create `SessionBuilderListContent.swift` — scrollable library list with search bar (`@State searchText`), filtered `items` from ViewModel, and `LibraryQueueRow` for each item. Derive selection state by checking if `building_setlist.entries` contains a matching `item_id`. On tap: dispatch `addToSetlist` or `removeFromSetlist` in `ios/Intrada/Views/Practice/SessionBuilderListContent.swift`
- [x] T010 [US1] Wire up iPhone layout in `SessionBuilderView`: back link ("< Practice"), "New Session" heading, `SessionBuilderListContent` filling available space, `StickyBottomBar` pinned at bottom. Bottom bar count derived from `building_setlist.entries.count` and total planned duration. "Start Session" dispatches `Event.session(.startSession(now:))` with ISO 8601 timestamp in `ios/Intrada/Views/Practice/SessionBuilderView.swift`
- [x] T011 [US1] Wire up iPad layout in `SessionBuilderView`: HStack with left column (back link, "Library" heading, search bar, `SessionBuilderListContent`) and right column (placeholder for setlist panel — will be filled in US2). "Start Session" button at bottom of right column in `ios/Intrada/Views/Practice/SessionBuilderView.swift`
- [x] T012 [US1] Handle cancel/back: dispatch `Event.session(.cancelBuilding)` when back link tapped. Handle empty library edge case with `EmptyStateView` in `ios/Intrada/Views/Practice/SessionBuilderListContent.swift`
- [x] T013 [US1] Add `#Preview` blocks for `SessionBuilderView`, `SessionBuilderListContent`, `LibraryQueueRow`, and `StickyBottomBar` in their respective files
- [x] T014 [US1] Run `just ios-swift-check` and `just ios-smoke-test` to verify builder compiles and launches without crash
- [x] T015 [US1] Run quickstart.md verifications V1 (Builder Opens), V2 (Tap-to-Queue), V3 (Start Session), V6 (Search Filter), V7 (State-Driven Navigation), V9 (Cancel Building)

**Checkpoint**: User Story 1 is fully functional. Musicians can build and start sessions on both iPhone and iPad.

---

## Phase 4: User Story 2 — Customise setlist entries (Priority: P2)

**Goal**: Musicians can open the setlist to reorder entries via drag, remove entries, and expand entries to edit duration/intention/reps using progressive disclosure.

**Independent Test**: Build a 3-item setlist → open setlist (bottom sheet on iPhone / right panel on iPad) → drag to reorder → tap entry to expand → set duration and intention → tap × to remove → verify all changes reflected.

### Implementation for User Story 2

- [x] T016 [US2] Create `SetlistSheetContent.swift` — shared content for both bottom sheet (iPhone) and right panel (iPad). Contains: "Your Setlist" heading, "Load Routine" link (placeholder for US4), session intention field (wired in US3), `List` with `SetlistEntryRow` items using `.onMove` for drag reorder. On move: dispatch `Event.session(.reorderSetlist(entryId:, newPosition:))`. Total time display. "Start Session →" full-width button in `ios/Intrada/Views/Practice/SetlistSheetContent.swift`
- [x] T017 [US2] Add progressive disclosure to `SetlistEntryRow` — `@State expandedEntryId` controls which entry is expanded. Expanded state shows: duration picker (dispatch `setEntryDuration`), intention text field (dispatch `setEntryIntention`), rep target stepper (dispatch `setRepTarget`). Collapsed state shows title + metadata summary in `ios/Intrada/Components/SetlistEntryRow.swift`
- [x] T018 [US2] Wire iPhone bottom sheet in `SessionBuilderView` — `.sheet(isPresented: $isSheetPresented)` with `PresentationDetent` (.medium, .large). Sheet opens when user taps count area of `StickyBottomBar`. Sheet contains `SetlistSheetContent` in `ios/Intrada/Views/Practice/SessionBuilderView.swift`
- [x] T019 [US2] Wire iPad right panel — replace placeholder from T011 with `SetlistSheetContent` embedded directly in the right column (no sheet needed on iPad) in `ios/Intrada/Views/Practice/SessionBuilderView.swift`
- [x] T020 [US2] Handle remove entry: × button dispatches `Event.session(.removeFromSetlist(entryId:))`. Verify library row returns to unselected state in `ios/Intrada/Components/SetlistEntryRow.swift`
- [x] T021 [US2] Add `#Preview` blocks for `SetlistSheetContent` and expanded `SetlistEntryRow` in their respective files
- [x] T022 [US2] Run `just ios-swift-check` and `just ios-smoke-test`
- [x] T023 [US2] Run quickstart.md verification V4 (Setlist Sheet — iPhone) and V5 (iPad Split View)

**Checkpoint**: User Stories 1 AND 2 are functional. Full builder experience with customisation.

---

## Phase 5: User Story 3 — Set a session-level intention (Priority: P2)

**Goal**: Musicians can set an overarching focus for the session that carries through to the active session and summary.

**Independent Test**: Open builder → open setlist → type a session intention → start session → verify intention is preserved in ViewModel.

### Implementation for User Story 3

- [x] T024 [US3] Wire session intention field in `SetlistSheetContent` — `TextFieldView` bound to `building_setlist.sessionIntention`. On change: dispatch `Event.session(.setSessionIntention(intention:))` in `ios/Intrada/Views/Practice/SetlistSheetContent.swift`
- [x] T025 [US3] Run `just ios-swift-check`

**Checkpoint**: Session intention is functional. Minimal task — it's just wiring a text field.

---

## Phase 6: User Story 4 — Load a saved routine (Priority: P3)

**Goal**: Musicians can load a previously saved routine to populate the setlist with one tap.

**Independent Test**: Open builder → tap "Load Routine" → select a routine → verify setlist populates with routine entries.

### Implementation for User Story 4

- [x] T026 [US4] Add routine loading UI to `SetlistSheetContent` — "Load Routine" link opens a list of available routines from ViewModel. Tapping a routine dispatches `Event.session(.loadRoutine(routineId:))` (or equivalent Crux event). List hidden when no routines available in `ios/Intrada/Views/Practice/SetlistSheetContent.swift`
- [x] T027 [US4] Run `just ios-swift-check` and `just ios-smoke-test`

**Checkpoint**: Routine loading works. Users can quickly build sessions from saved templates.

---

## Phase 7: User Story 5 — Save current setlist as a routine (Priority: P3)

**Goal**: Musicians can save their current setlist as a reusable routine.

**Independent Test**: Build a setlist → tap "Save as Routine" → enter a name → save → verify routine appears in routines list.

### Implementation for User Story 5

- [x] T028 [US5] Add "Save as Routine" UI to `SetlistSheetContent` — visible only when setlist has entries. Tapping opens an inline form with name field and save button. On save: dispatch routine save event. Validation: name required in `ios/Intrada/Views/Practice/SetlistSheetContent.swift`
- [x] T029 [US5] Run `just ios-swift-check` and `just ios-smoke-test`

**Checkpoint**: Full routine save/load cycle works.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final verification, edge cases, and cleanup.

- [x] T030 [P] Add error display — show `ViewModel.error` as a toast or `ErrorBanner` in `SessionBuilderView` in `ios/Intrada/Views/Practice/SessionBuilderView.swift`
- [x] T031 [P] Verify tab bar indicator — accent dot on Practice tab when `session_status` is `.building` or `.active` (quickstart V8) in `ios/Intrada/Navigation/MainTabView.swift`
- [x] T032 Run quickstart.md full verification (V1–V9)
- [x] T033 Run `just ios-swift-check --clean` and `just ios-smoke-test` for final validation
- [x] T034 Update `CLAUDE.md` — add new components (`LibraryQueueRow`, `SetlistEntryRow`, `StickyBottomBar`) to iOS components table, add `SessionBuilderView` to iOS views table
- [x] T035 Update Pencil design file if implementation diverged from mockups in `design/intrada.pen`
- [x] T036 Tidy up Pencil design file — ensure new frames are properly named/positioned in iOS row, remove any superseded frames, update catalogue if needed in `design/intrada.pen`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3–7)**: All depend on Foundational phase completion
  - US1 (P1): No dependencies on other stories
  - US2 (P2): Depends on US1 (needs library list and builder view to exist)
  - US3 (P2): Depends on US2 (needs SetlistSheetContent to exist)
  - US4 (P3): Depends on US2 (needs SetlistSheetContent to exist)
  - US5 (P3): Depends on US2 (needs SetlistSheetContent to exist)
- **Polish (Phase 8)**: Depends on US1 + US2 at minimum

### Within Each User Story

- Components before views
- Views before integration/wiring
- Compile check after each phase
- Smoke test for runtime validation

### Parallel Opportunities

- T002 + T003 can run in parallel (different files)
- T004 + T005 + T006 can run in parallel (different component files)
- US3, US4, US5 can run in parallel after US2 completes (all modify SetlistSheetContent but different sections)
- T030 + T031 can run in parallel (different files)

---

## Parallel Example: Foundational Phase

```bash
# Launch all component tasks together:
Task: "Create LibraryQueueRow component in ios/Intrada/Components/LibraryQueueRow.swift"
Task: "Create StickyBottomBar component in ios/Intrada/Components/StickyBottomBar.swift"
Task: "Create SetlistEntryRow component in ios/Intrada/Components/SetlistEntryRow.swift"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T007)
3. Complete Phase 3: User Story 1 (T008–T015)
4. **STOP and VALIDATE**: Test tap-to-queue builder independently
5. This alone delivers a working session builder — musicians can build and start sessions

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. User Story 1 → Test → **MVP! Sessions can be built and started** 🎯
3. User Story 2 → Test → Setlist customisation (reorder, edit, remove)
4. User Story 3 → Test → Session intentions
5. User Stories 4+5 → Test → Routine load/save
6. Polish → Final validation

---

## Notes

- All business logic is in the Crux core — Swift code only dispatches events and reads ViewModel
- Run `just ios-swift-check` after every task for fast feedback (~30s)
- Run `just ios-smoke-test` after each phase for runtime crash detection
- Use `@Indirect` property wrapper access pattern for generated types (access directly, not `.value`)
- Always read generated types in `ios/Intrada/Generated/SharedTypes/SharedTypes.swift` before assuming field names
- Commit after each phase or logical group of tasks

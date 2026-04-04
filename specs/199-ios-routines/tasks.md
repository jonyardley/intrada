# Tasks: iOS Routines

**Input**: Design documents from `/specs/199-ios-routines/`

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [ ] T001 [P] Create `RoutineSaveForm` in `ios/Intrada/Components/RoutineSaveForm.swift` — collapsible form: collapsed shows "Save as Routine" button, expanded shows name TextField + Save/Cancel. Validates non-empty name ≤200 chars. Calls `onSave: (String) -> Void`. Include `#Preview`.

---

## Phase 2: Foundational

- [ ] T002 Create `RoutineListView` in `ios/Intrada/Views/Routines/RoutineListView.swift` — reads `core.viewModel.routines`. NavigationStack on iPhone, NavigationSplitView on iPad. Shows routine cards (name + entry count). Empty state via EmptyStateView. Swipe-to-delete with confirmation. Include `#Preview`.
- [ ] T003 Create `RoutineDetailView` in `ios/Intrada/Views/Routines/RoutineDetailView.swift` — shows routine name (heading), ordered item list with TypeBadge, Edit button in toolbar. Include `#Preview`.
- [ ] T004 Replace Routines tab placeholder in `ios/Intrada/Navigation/MainTabView.swift` — swap `PlaceholderView(tab: .routines)` NavigationStack with `RoutineListView()`.
- [ ] T005 Run `just ios-swift-check` — fix any errors.

**Checkpoint**: Routines tab shows list, tap for detail, delete works.

---

## Phase 3: User Story 2 — Edit Routine (P2)

- [ ] T006 [US2] Create `RoutineEditView` in `ios/Intrada/Views/Routines/RoutineEditView.swift` — name TextField, List with `.onMove` for reorder, remove button per entry, "Add from Library" section showing available items. Save dispatches `.routine(.updateRoutine(id:, name:, entries:))`. Cancel pops navigation. Include `#Preview`.
- [ ] T007 [US2] Wire Edit button in `RoutineDetailView` — NavigationLink to RoutineEditView.
- [ ] T008 [US2] Run `just ios-swift-check` — fix any errors.

**Checkpoint**: Can edit routine name, reorder, add/remove items.

---

## Phase 4: User Story 3 — Load Routine (P3)

- [ ] T009 [US3] Wire "Load Routine" in `ios/Intrada/Views/Practice/SetlistSheetContent.swift` — replace placeholder button with sheet/menu listing available routines. Tap dispatches `.routine(.loadRoutineIntoSetlist(routineId:))`.
- [ ] T010 [US3] Run `just ios-swift-check` — fix any errors.

**Checkpoint**: Can load routine into session builder.

---

## Phase 5: User Story 4 — Save as Routine (P4)

- [ ] T011 [US4] Add `RoutineSaveForm` to `SetlistSheetContent` — below the Start Session button. On save dispatches `.routine(.saveBuildingAsRoutine(name:))`.
- [ ] T012 [US4] Add `RoutineSaveForm` to `SessionSummaryView` — above Save/Discard actions. On save dispatches `.routine(.saveSummaryAsRoutine(name:))`.
- [ ] T013 [US4] Run `just ios-swift-check` — fix any errors.

**Checkpoint**: Can save routines from builder and summary.

---

## Phase 6: Polish

- [ ] T014 Run full validation: `just ios-swift-check`, `just ios-smoke-test`
- [ ] T015 Update `docs/roadmap.md` — move #199 to "What's Built Today"
- [ ] T016 Update `CLAUDE.md` — add new views to iOS views table

---

## Dependencies

```text
Phase 1 (RoutineSaveForm) ──────────────────────┐
Phase 2 (List + Detail + Router) ───────────────┐│
                                                ││
Phase 3 (US2: Edit) ◄──────────────────────────┘│
Phase 4 (US3: Load) ◄───────────────────────────┘
Phase 5 (US4: Save as Routine) ◄─────────────────┘
```

## Implementation Strategy

### MVP (US1 only): Phases 1–2 — browse and delete routines
### Full: All phases — CRUD + load + save

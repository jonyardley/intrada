# Tasks: Web — Adopt iOS Layout Patterns

**Input**: Design documents from `/specs/219-web-ios-layout-parity/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Prepare new component files and CSS utilities

- [ ] T001 Create empty component file `crates/intrada-web/src/components/library_list_row.rs` with module declaration
- [ ] T002 [P] Create empty component file `crates/intrada-web/src/components/split_view_layout.rs` with module declaration
- [ ] T003 [P] Create empty component file `crates/intrada-web/src/components/sticky_bottom_bar.rs` with module declaration
- [ ] T004 [P] Create empty component file `crates/intrada-web/src/components/slide_up_sheet.rs` with module declaration
- [ ] T005 Register new component modules in `crates/intrada-web/src/components/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared layout components that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Implement `SplitViewLayout` component in `crates/intrada-web/src/components/split_view_layout.rs` — accepts `sidebar` and `detail` children slots; renders side-by-side on `md:` (≥768px) with ~320px sidebar and flexible detail pane; renders only the active slot on mobile (<768px) using Tailwind `hidden md:flex` / `md:hidden` classes
- [ ] T007 [P] Implement `LibraryListRow` component in `crates/intrada-web/src/components/library_list_row.rs` — accepts `LibraryItemView`, renders compact horizontal row with title, subtitle, metadata line (key · tempo), and `TypeBadge`; supports `is_selected: bool` prop (accent left bar + check icon when selected, + icon when not); hover state with `bg-surface-hover`; divider bottom border
- [ ] T008 [P] Add any new Tailwind utility classes needed for split-view and compact rows in `crates/intrada-web/input.css` (e.g., `sidebar-width`, slide-up transition utilities)
- [ ] T009 Add `LibraryListRow`, `SplitViewLayout`, `StickyBottomBar`, `SlideUpSheet` exports to `crates/intrada-web/src/components/mod.rs`

**Checkpoint**: Foundation components ready — user story implementation can now begin

---

## Phase 3: User Story 1 — Split-View Library on Desktop (Priority: P1) 🎯 MVP

**Goal**: Desktop library shows sidebar list + detail pane; clicking an item updates the detail without page navigation; URL updates for deep-linking; mobile is unchanged

**Independent Test**: Open library on desktop → sidebar shows compact rows → click item → detail loads in right pane → URL updates → resize to mobile → stacked navigation

### Implementation for User Story 1

- [ ] T010 [US1] Update library route in `crates/intrada-web/src/app.rs` from `/library` to `/library/:id?` with optional item ID parameter; add nested routes for `/library/add` and `/library/:id/edit`
- [ ] T011 [US1] Refactor `crates/intrada-web/src/views/library.rs` — wrap in `SplitViewLayout`; sidebar renders scrollable list of `LibraryListRow` components with click handler that navigates to `/library/{id}`; highlight selected row based on route param; auto-select first item when no `:id` param on desktop; show `PageHeading` + filter tabs above the list
- [ ] T012 [US1] Update `crates/intrada-web/src/views/detail.rs` — make it render inside the split-view detail pane on desktop (remove outer page wrapper when embedded); keep full-page layout on mobile; accept item ID from route param or parent prop
- [ ] T013 [US1] Update `crates/intrada-web/src/views/add_item.rs` — render inside split-view detail pane on desktop when accessed via `/library/add`; keep full-page on mobile
- [ ] T014 [US1] Update `crates/intrada-web/src/views/edit_item.rs` — render inside split-view detail pane on desktop when accessed via `/library/:id/edit`; keep full-page on mobile
- [ ] T015 [US1] Handle empty library edge case — when library has 0 items, detail pane shows empty state with "Add your first item" CTA
- [ ] T016 [US1] Handle direct URL navigation — visiting `/library/{id}` on desktop pre-selects that item in sidebar and shows its detail; on mobile shows detail page with back link

**Checkpoint**: Split-view library fully functional on desktop, stacked nav on mobile

---

## Phase 4: User Story 2 — Compact Library List Rows (Priority: P2)

**Goal**: Library items display as compact rows instead of glassmorphism cards everywhere in the app

**Independent Test**: Open library → items show as compact rows with title, subtitle, metadata, badge → hover shows highlight → items with missing metadata gracefully omit fields

### Implementation for User Story 2

- [ ] T017 [US2] Replace `LibraryItemCard` usage in `crates/intrada-web/src/views/library.rs` with `LibraryListRow` for the sidebar list (if not already done in T011)
- [ ] T018 [US2] Ensure `LibraryListRow` gracefully handles missing fields — empty subtitle hides the subtitle line; empty key/tempo hides the metadata line; component never renders blank space for absent data
- [ ] T019 [US2] Update `crates/intrada-web/src/views/design_catalogue.rs` — add `LibraryListRow` showcase entry showing both selected and unselected states; keep `LibraryItemCard` in catalogue for reference
- [ ] T020 [US2] Verify all library-related skeleton loading states still work — update `SkeletonItemCard` usage if needed to match compact row layout, or create a `SkeletonListRow` variant

**Checkpoint**: All library lists use compact rows; cards retained only for non-list contexts

---

## Phase 5: User Story 3 — Tap-to-Queue Session Builder (Priority: P3)

**Goal**: Session builder uses tap-to-queue pattern with split-view on desktop and sticky bottom bar + slide-up sheet on mobile

**Independent Test**: Open session builder → tap 3 items → selected state shows → setlist panel (desktop) or bottom bar (mobile) updates → tap "Start Session"

### Implementation for User Story 3

- [ ] T021 [US3] Implement `StickyBottomBar` component in `crates/intrada-web/src/components/sticky_bottom_bar.rs` — fixed to viewport bottom; shows item count, total duration, "Start Session" button (disabled when empty); left side is tappable to open setlist sheet
- [ ] T022 [P] [US3] Implement `SlideUpSheet` component in `crates/intrada-web/src/components/slide_up_sheet.rs` — CSS transform slide-up panel with backdrop overlay; accepts children slot; toggled by signal; backdrop tap dismisses; smooth transition animation
- [ ] T023 [US3] Refactor `crates/intrada-web/src/views/session_new.rs` — on desktop: use `SplitViewLayout` with library list (left, using `LibraryListRow` with `is_selected` based on setlist membership) and setlist panel (right); on mobile: full-screen library list with `StickyBottomBar`
- [ ] T024 [US3] Update `crates/intrada-web/src/components/setlist_builder.rs` — extract setlist panel content (intention input, entry list with drag-reorder, Start/Cancel buttons) into a reusable section that renders in the split-view detail pane (desktop) or inside the `SlideUpSheet` (mobile)
- [ ] T025 [US3] Update `crates/intrada-web/src/components/setlist_entry_row.rs` — add progressive disclosure: collapsed state shows title + type badge + drag handle; expanded state (on click) reveals duration, intention, and rep fields; add expand/collapse toggle icon
- [ ] T026 [US3] Wire tap-to-queue in session builder library list — clicking a `LibraryListRow` dispatches `AddToSetlist` (if not selected) or `RemoveFromSetlist` (if selected); derive selected state from `session_entries` in ViewModel
- [ ] T027 [US3] Wire mobile slide-up sheet — tapping bottom bar summary opens `SlideUpSheet` containing the setlist panel; "Start Session" in either the sheet or bottom bar dispatches `StartSession`

**Checkpoint**: Session builder works with tap-to-queue on both desktop and mobile

---

## Phase 6: User Story 4 — Session Builder Search & Filter (Priority: P4)

**Goal**: Library list in the session builder supports text search and type filter tabs

**Independent Test**: Open session builder → type search query → list filters → tap type tab → list filters by type → clear → all items shown

### Implementation for User Story 4

- [ ] T028 [US4] Add search `TextField` above the library list in session builder view in `crates/intrada-web/src/views/session_new.rs` — Leptos signal filters `items` from ViewModel by title or subtitle (case-insensitive contains match)
- [ ] T029 [US4] Add `TypeTabs` (All / Pieces / Exercises) below search field in session builder — Leptos signal filters items by `item_type`; combines with search filter
- [ ] T030 [US4] Ensure selected items remain visible even when filtered out by search/type — OR show a "X items selected but hidden by filter" notice

**Checkpoint**: Session builder has full search and filter capabilities

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: E2E test updates, design catalogue, documentation

- [ ] T031 Update E2E tests in `e2e/tests/library.spec.ts` — update selectors for compact rows and split-view layout; add desktop split-view assertion; verify mobile stacked nav unchanged
- [ ] T032 [P] Update E2E tests in `e2e/tests/sessions.spec.ts` — update selectors for tap-to-queue builder; verify item selection toggle; verify Start Session flow
- [ ] T033 [P] Update E2E tests in `e2e/tests/navigation.spec.ts` — verify split-view library navigation on desktop; verify URL updates on item selection
- [ ] T034 Update `crates/intrada-web/src/views/design_catalogue.rs` — add `SplitViewLayout`, `StickyBottomBar`, and `SlideUpSheet` showcase entries
- [ ] T035 Update Pencil design file `design/intrada.pen` — add web desktop split-view library frame and web mobile session builder frame to match implementation
- [ ] T036 Run quickstart.md verification steps and confirm all 6 scenarios pass
- [ ] T037 Run full E2E test suite (`npx playwright test`) and fix any failures

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — the MVP
- **US2 (Phase 4)**: Depends on Foundational; benefits from US1 but independently testable
- **US3 (Phase 5)**: Depends on Foundational; uses `LibraryListRow` from Phase 2
- **US4 (Phase 6)**: Depends on US3 (session builder must exist first)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Split-View Library)**: Independent after Foundational — **this is the MVP**
- **US2 (Compact Rows)**: Independent after Foundational; partially overlaps with US1 (T011 already uses rows)
- **US3 (Tap-to-Queue Builder)**: Independent after Foundational; uses `LibraryListRow` and `SplitViewLayout` from Phase 2
- **US4 (Search & Filter)**: Depends on US3 (needs the session builder view to add search/filter to)

### Parallel Opportunities

- T002, T003, T004 can run in parallel (separate files)
- T007, T008 can run in parallel with T006 (separate files)
- T022 can run in parallel with T021 (separate components)
- T031, T032, T033 can run in parallel (separate test files)

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T005)
2. Complete Phase 2: Foundational (T006–T009)
3. Complete Phase 3: US1 — Split-View Library (T010–T016)
4. **STOP and VALIDATE**: Test split-view on desktop, stacked nav on mobile
5. Deploy/demo — users immediately benefit from no-navigation detail viewing

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (split-view library) → Test → Deploy (MVP!)
3. US2 (compact rows) → Test → Deploy (visual polish)
4. US3 (tap-to-queue builder) → Test → Deploy (session workflow improvement)
5. US4 (search & filter) → Test → Deploy (discoverability)
6. Polish → E2E tests, catalogue, Pencil → Final PR

---

## Notes

- All changes are within `crates/intrada-web/` — no core or API changes
- Existing `LibraryItemCard` is preserved (used in design catalogue); `LibraryListRow` replaces it in list contexts
- Existing drag-and-drop from `SetlistBuilder` is reused, not reimplemented
- The `SplitViewLayout` component is intentionally generic for reuse in routines/analytics views later

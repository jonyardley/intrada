# Tasks: Glassmorphism UI & Responsive Layout

**Input**: Design documents from `/specs/016-glassmorphism-responsive/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: No new test tasks — this is a visual-only change. All 14 existing E2E tests and 142 unit tests must continue to pass (verified in Polish phase).

**Organization**: Tasks are grouped by user story. US1 (Glassmorphism) is foundational — US2 and US3 build on top of it by adding responsive classes to the same files.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: CSS configuration and viewport setup that all user stories depend on

- [x] T001 Add `@custom-variant supports-backdrop` and `@utility pb-safe` definitions to `crates/intrada-web/input.css`
- [x] T002 Add `viewport-fit=cover` to viewport meta tag in `crates/intrada-web/index.html`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Replace the app-level background and base card component — ALL other visual changes build on these

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Replace light gradient background with fixed deep purple/indigo gradient wrapper div and add `min-h-screen` to main content area in `crates/intrada-web/src/app.rs` — use pattern: `<div class="fixed inset-0 -z-10 bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950"></div>` before main, add `pb-20 sm:pb-0` to main for tab bar clearance
- [x] T004 Replace opaque white card styling with glassmorphism pattern in `crates/intrada-web/src/components/card.rs` — replace `bg-white rounded-xl shadow-sm border border-slate-200 p-6` with `bg-indigo-950/80 supports-backdrop:bg-white/10 supports-backdrop:backdrop-blur-md border border-white/15 rounded-xl shadow-lg p-6`

**Checkpoint**: Background gradient visible, cards are translucent glass panels. All existing content still renders (may have contrast issues until US1 text colours are applied).

---

## Phase 3: User Story 1 — Glassmorphism Visual Theme (Priority: P1) 🎯 MVP

**Goal**: Complete the glassmorphism transformation — glass header/footer, light text hierarchy, themed buttons/inputs, and glass treatment on all content components. Every page should look polished with the new theme.

**Independent Test**: Load any page at desktop width — verify gradient background is visible behind translucent glass cards, all text is light-coloured and readable, header/footer have glass treatment, buttons and inputs are themed for dark context.

### Implementation for User Story 1

#### Structural Components (Header, Footer)

- [x] T005 [P] [US1] Restyle header with glass treatment in `crates/intrada-web/src/components/app_header.rs` — add `bg-gray-900/60 supports-backdrop:backdrop-blur-md border-b border-white/10`, change text to `text-white`, change nav links to `text-gray-300 hover:text-white`, hide nav on mobile with `hidden sm:flex`
- [x] T006 [P] [US1] Restyle footer with glass/transparent treatment and light text in `crates/intrada-web/src/components/app_footer.rs`

#### Text & Label Components

- [x] T007 [P] [US1] Change heading text colour to `text-white` in `crates/intrada-web/src/components/page_heading.rs`
- [x] T008 [P] [US1] Change back link colour to `text-gray-400 hover:text-white` with `motion-safe:transition-colors` in `crates/intrada-web/src/components/back_link.rs`
- [x] T009 [P] [US1] Change field label colour to `text-gray-200` in `crates/intrada-web/src/components/field_label.rs`
- [x] T010 [P] [US1] Change error text colour to `text-red-400` in `crates/intrada-web/src/components/form_field_error.rs`

#### Interactive Components (Buttons, Inputs)

- [x] T011 [P] [US1] Restyle all button variants for dark glass context in `crates/intrada-web/src/components/button.rs` — primary: indigo bg with light text; secondary: translucent bg with light text; adjust hover/focus states; add `motion-safe:transition-colors`
- [x] T012 [P] [US1] Restyle text input with glass styling in `crates/intrada-web/src/components/text_field.rs` — translucent bg (`bg-white/10`), light text (`text-white`), border (`border-white/20`), placeholder (`placeholder-gray-400`), focus ring (`focus:ring-indigo-400`)
- [x] T013 [P] [US1] Restyle textarea with glass styling in `crates/intrada-web/src/components/text_area.rs` — same pattern as text_field.rs

#### Content Components

- [x] T014 [P] [US1] Restyle library item card with light text hierarchy and tag colours for dark background in `crates/intrada-web/src/components/library_item_card.rs` — title: `text-white`, metadata: `text-gray-300`, timestamps: `text-gray-400`
- [x] T015 [P] [US1] Adjust type badge colours for visibility on dark glass background in `crates/intrada-web/src/components/type_badge.rs`
- [x] T016 [P] [US1] Restyle type tabs with glass container and adjusted active/inactive states in `crates/intrada-web/src/components/type_tabs.rs`
- [x] T017 [P] [US1] Restyle setlist builder with light text and glass sub-cards in `crates/intrada-web/src/components/setlist_builder.rs`
- [x] T018 [P] [US1] Restyle setlist entry rows with light text and glass row styling in `crates/intrada-web/src/components/setlist_entry.rs`
- [x] T019 [P] [US1] Restyle session timer with light text and glass card backgrounds in `crates/intrada-web/src/components/session_timer.rs`
- [x] T020 [P] [US1] Restyle session summary with light text and glass entry cards in `crates/intrada-web/src/components/session_summary.rs`

#### Views (Light Text Pass)

- [x] T021 [P] [US1] Update library list view with light text colours in `crates/intrada-web/src/views/library_list.rs`
- [x] T022 [P] [US1] Update detail view with light text colours in `crates/intrada-web/src/views/detail.rs`
- [x] T023 [P] [US1] Update add form view with light text colours in `crates/intrada-web/src/views/add_form.rs`
- [x] T024 [P] [US1] Update edit form view with light text colours in `crates/intrada-web/src/views/edit_form.rs`
- [x] T025 [P] [US1] Update sessions view with light text colours in `crates/intrada-web/src/views/sessions.rs`
- [x] T026 [P] [US1] Update not found view with light text colours in `crates/intrada-web/src/views/not_found.rs`

**Checkpoint**: At this point, the full glassmorphism theme is applied. All pages should have gradient background, glass cards, light text, themed inputs/buttons. All 14 E2E tests should still pass. Visually verify at 1280px desktop width.

---

## Phase 4: User Story 2 — Mobile-Friendly Layout (Priority: P2)

**Goal**: Make the app fully usable on mobile screens (<640px) with bottom tab bar navigation, single-column layouts, full-width forms, and adequate touch targets.

**Independent Test**: Set viewport to 375px (iPhone SE) — verify no horizontal scrollbar on any page, bottom tab bar visible with Library and Sessions tabs, header nav links hidden, all buttons/links are at least 44x44px, forms are full-width with readable labels.

### Implementation for User Story 2

#### New Component: Bottom Tab Bar

- [x] T027 [US2] Create bottom tab bar component in `crates/intrada-web/src/components/bottom_tab_bar.rs` — fixed position, glass-styled, `sm:hidden`, with Library (`/`) and Sessions (`/sessions`) icon+label tabs, active tab highlighting based on current route
- [x] T028 [US2] Export `BottomTabBar` from `crates/intrada-web/src/components/mod.rs`
- [x] T029 [US2] Wire `<BottomTabBar />` into app layout in `crates/intrada-web/src/app.rs` — place after main content, before closing tags

#### Responsive Layout Updates

- [x] T030 [P] [US2] Add responsive padding and single-column mobile layout to library list in `crates/intrada-web/src/views/library_list.rs` — `grid-cols-1` base, responsive padding `px-4 sm:px-6`
- [x] T031 [P] [US2] Add responsive layout to detail view in `crates/intrada-web/src/views/detail.rs` — responsive padding, ensure metadata wraps on mobile
- [x] T032 [P] [US2] Add responsive form layout to add form in `crates/intrada-web/src/views/add_form.rs` — full-width inputs on mobile, responsive padding
- [x] T033 [P] [US2] Add responsive form layout to edit form in `crates/intrada-web/src/views/edit_form.rs` — full-width inputs on mobile, responsive padding
- [x] T034 [P] [US2] Add responsive layout to sessions view in `crates/intrada-web/src/views/sessions.rs` — responsive padding, stack cards vertically
- [x] T035 [P] [US2] Ensure session timer digits and control buttons are large and tappable on mobile in `crates/intrada-web/src/components/session_timer.rs` — minimum 44x44px touch targets
- [x] T036 [P] [US2] Ensure all buttons meet 44x44px minimum touch target on mobile in `crates/intrada-web/src/components/button.rs` — add `min-h-[44px] min-w-[44px]` or equivalent padding

**Checkpoint**: At this point, the app should be fully usable at 375px width. Bottom tab bar visible on mobile, hidden on tablet/desktop. No horizontal scrollbar. All touch targets ≥ 44px. E2E tests should still pass.

---

## Phase 5: User Story 3 — Tablet-Optimised Layout (Priority: P3)

**Goal**: Optimise the layout for tablet screens (640px–1024px) with multi-column grids where appropriate, comfortable form widths, and efficient use of available space.

**Independent Test**: Set viewport to 768px (iPad portrait) — verify library items display in two-column grid, header horizontal nav is visible (no bottom tab bar), forms use comfortable width (not stretched full-screen), detail view uses width efficiently.

### Implementation for User Story 3

- [x] T037 [P] [US3] Add two-column grid for tablet breakpoint to library list in `crates/intrada-web/src/views/library_list.rs` — `sm:grid-cols-2 lg:grid-cols-1` (two columns on tablet, single column with sidebar space on desktop)
- [x] T038 [P] [US3] Ensure detail view metadata uses available width efficiently at tablet breakpoint in `crates/intrada-web/src/views/detail.rs` — verify `sm:grid-cols-2` metadata grid works well
- [x] T039 [P] [US3] Constrain form width at tablet breakpoint in `crates/intrada-web/src/views/add_form.rs` — `sm:max-w-lg sm:mx-auto` or similar comfortable width
- [x] T040 [P] [US3] Constrain form width at tablet breakpoint in `crates/intrada-web/src/views/edit_form.rs` — `sm:max-w-lg sm:mx-auto` or similar comfortable width
- [x] T041 [P] [US3] Ensure session timer and item info use tablet width effectively in `crates/intrada-web/src/components/session_timer.rs`

**Checkpoint**: All three user stories complete. The app should look polished at 375px, 768px, and 1280px viewports. All E2E tests should still pass.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verification, accessibility, and performance validation across all user stories

- [x] T042 Run `cargo test` and verify all 142 unit tests pass
- [x] T043 Run `cargo clippy -- -D warnings` and verify zero warnings
- [x] T044 Run `cd e2e && npx playwright test` and verify all 14 E2E tests pass unchanged
- [ ] T045 Visual verification at 1280px desktop width — gradient background, glass cards, light text, themed buttons/inputs on all pages (Library, Detail, Add, Edit, Sessions, New Session, Active Session, Summary)
- [ ] T046 Visual verification at 375px mobile width — no horizontal scroll, bottom tab bar visible, single-column layout, large touch targets, full-width forms
- [ ] T047 Visual verification at 768px tablet width — two-column library grid, horizontal nav visible, no bottom tab bar, comfortable form widths
- [ ] T048 Test `prefers-reduced-motion: reduce` in Chrome DevTools Rendering panel — verify no transitions or animations play
- [ ] T049 Test backdrop-filter fallback — disable `backdrop-filter` via CSS override in DevTools, verify cards fall back to solid semi-opaque background with readable text
- [ ] T050 Test very wide screen (1920px) — verify content is centered and constrained to max width
- [ ] T051 Test very small screen (320px) — verify app is still usable, timer is readable

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational — BLOCKS US2 and US3 (glass theme must be in place before responsive changes)
- **User Story 2 (Phase 4)**: Depends on US1 completion — mobile layout builds on themed components
- **User Story 3 (Phase 5)**: Depends on US1 completion — can run in parallel with US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 (glass theme must be complete before adding responsive mobile classes to same files)
- **User Story 3 (P3)**: Depends on US1 (glass theme must be complete before adding responsive tablet classes to same files) — can run in parallel with US2

### Within Each User Story

- Structural components (header, footer) before content components
- Components before views (views depend on component styling)
- All [P] tasks within a story can run in parallel

### Parallel Opportunities

- T001 and T002 (Setup) can run in parallel
- T005–T026 (all US1 component/view tasks) can run in parallel — each modifies a different file
- T030–T036 (US2 responsive layout tasks) can run in parallel — each modifies a different file
- T037–T041 (US3 tablet layout tasks) can run in parallel — each modifies a different file
- US2 and US3 can run in parallel after US1 completes (different breakpoint classes, minimal conflict)

---

## Parallel Example: User Story 1

```bash
# All US1 component tasks can run in parallel (different files):
Task: "T005 [US1] Restyle header with glass treatment in app_header.rs"
Task: "T006 [US1] Restyle footer in app_footer.rs"
Task: "T007 [US1] Change heading text colour in page_heading.rs"
Task: "T008 [US1] Change back link colour in back_link.rs"
Task: "T009 [US1] Change field label colour in field_label.rs"
Task: "T010 [US1] Change error text colour in form_field_error.rs"
Task: "T011 [US1] Restyle buttons in button.rs"
Task: "T012 [US1] Restyle text input in text_field.rs"
Task: "T013 [US1] Restyle textarea in text_area.rs"
Task: "T014 [US1] Restyle library item card in library_item_card.rs"
Task: "T015 [US1] Adjust type badge colours in type_badge.rs"
Task: "T016 [US1] Restyle type tabs in type_tabs.rs"
Task: "T017 [US1] Restyle setlist builder in setlist_builder.rs"
Task: "T018 [US1] Restyle setlist entry in setlist_entry.rs"
Task: "T019 [US1] Restyle session timer in session_timer.rs"
Task: "T020 [US1] Restyle session summary in session_summary.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T004)
3. Complete Phase 3: User Story 1 (T005–T026)
4. **STOP and VALIDATE**: Run `cargo test`, `cargo clippy`, E2E tests. Visually verify at 1280px.
5. The glassmorphism theme is complete — deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Glass infrastructure ready
2. Add User Story 1 → Full glassmorphism theme at desktop width → Validate (MVP!)
3. Add User Story 2 → Mobile-friendly with bottom tab bar → Validate at 375px
4. Add User Story 3 → Tablet-optimised with multi-column grids → Validate at 768px
5. Polish → Full verification across all viewports, accessibility, performance

### Sequential Approach (Recommended for Solo Developer)

Since many tasks modify the same files across stories (e.g., `library_list.rs` is touched in US1, US2, and US3), a sequential approach minimises merge conflicts:

1. Phase 1 + 2: Setup + Foundational
2. Phase 3: All US1 tasks (glass theme everywhere)
3. Phase 4: All US2 tasks (mobile responsiveness)
4. Phase 5: All US3 tasks (tablet optimisation)
5. Phase 6: Full verification pass

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- This is a visual-only feature — no behaviour, data, or routing changes
- All 14 E2E tests must pass unchanged after each phase
- The same files are often touched across multiple user stories (e.g., adding glass in US1, then mobile padding in US2, then tablet grid in US3) — this is expected and manageable since changes target different CSS classes
- Commit after each completed phase for clean git history
- Stop at any checkpoint to validate independently

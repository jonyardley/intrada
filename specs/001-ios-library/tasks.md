# Tasks: iOS Library — Browse, Search & Manage Repertoire

**Input**: Design documents from `/specs/001-ios-library/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested — validation via `just ios-swift-check`, `just ios-smoke-test`, `just ios-preview-check`.

**Organization**: Tasks grouped by user story. Each story is independently testable after its phase completes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create the directory structure and shared helper types needed by all stories

- [x] T001 Create `ios/Intrada/Views/Library/` directory for library feature views
- [x] T002 [P] Create `FilterTab` enum and `LibraryFormValidator` struct in `ios/Intrada/Views/Library/LibraryHelpers.swift` — define `FilterTab` (all/pieces/exercises with `itemKind` computed property) and validation constants mirroring `intrada-core/src/validation.rs` (maxTitle=500, maxComposer=200, maxNotes=5000, maxTag=100, maxTempoMarking=100, minBpm=1, maxBpm=400) with `validate(kind:title:composer:key:tempoMarking:bpm:notes:tags:) -> [String: String]` returning field-keyed error map
- [x] T003 [P] Create `LibraryViewModel` helper extension in `ios/Intrada/Views/Library/LibraryHelpers.swift` — add computed properties `uniqueComposers: [String]` (deduplicated sorted subtitles from `core.viewModel.items`) and `uniqueTags: [String]` (deduplicated sorted tags from all items) for autocomplete data

**Checkpoint**: Shared infrastructure ready — user story implementation can begin

---

## Phase 2: User Story 1 — Browse Library (Priority: P1) MVP

**Goal**: Users see their full repertoire as a scrollable list with type filter tabs. iPad shows NavigationSplitView with sidebar. Loading skeleton and empty state handled.

**Independent Test**: Sign in → Library tab → see items list with titles, composers, type badges, key, tempo, tags. Tap filter tabs to toggle. See skeleton while loading, empty state when no items.

**Functional Requirements**: FR-001, FR-002, FR-003, FR-004, FR-005, FR-022, FR-024, FR-025

### Components for User Story 1

- [x] T004 [P] [US1] Create `LibraryItemRow` component in `ios/Intrada/Components/LibraryItemRow.swift` — displays a single library item as a list row: title (primary text, semibold, truncated), subtitle/composer (secondary text, muted, truncated), `TypeBadge` (trailing), metadata line (key with "♯" prefix, tempo with "♩" prefix, combined tempo "108 / 120 BPM" when `latest_achieved_tempo` exists), tags as small pills. Uses design tokens only (textPrimary, textSecondary, textMuted, textFaint). Takes `LibraryItemView` as parameter. Include `#Preview`.
- [x] T005 [P] [US1] Create `TypeTabs` component in `ios/Intrada/Components/TypeTabs.swift` — segmented control with three segments: All, Pieces, Exercises. Two modes: interactive (with `onChange` callback, `@Binding selection: FilterTab`) and display-only (shows current type, disabled styling with textFaint). Use `Picker` with `.segmented` style or custom pill toggle matching web design. Interactive mode has accent background on active pill. Include `#Preview` for both modes.
- [x] T006 [P] [US1] Create `LibrarySkeletonView` in `ios/Intrada/Views/Library/LibrarySkeletonView.swift` — loading placeholder showing 4-6 skeleton item rows using `SkeletonLine` and `SkeletonBlock` components with `.pulsing()` modifier. Match the layout of `LibraryItemRow` (title line, subtitle line, metadata line, tag placeholders). Include `#Preview`.

### Views for User Story 1

- [x] T007 [US1] Create `LibraryListContent` in `ios/Intrada/Views/Library/LibraryListContent.swift` — scrollable list content used inside both NavigationSplitView sidebar and NavigationStack. Takes `@Binding selectedItemId: String?` for split-view selection. Contains: `TypeTabs` (interactive, bound to `@State filterTab: FilterTab`), item count label (e.g. "12 items" in textMuted), `LazyVStack` of `LibraryItemRow` items (from `core.viewModel.items`, filtered by `filterTab`). Dispatches `Event.setQuery(ListQuery)` when filter changes. Shows `LibrarySkeletonView` when `core.isLoading`, `EmptyStateView` when items empty (icon: "music.note.list", title: "No items yet", actionTitle: "Add Item"). "+" toolbar button for add item. Read `@Environment(IntradaCore.self)`. Include `#Preview`.
- [x] T008 [US1] Create `LibraryView` (root) in `ios/Intrada/Views/Library/LibraryView.swift` — the root library view using `NavigationSplitView` with two columns. Sidebar: `LibraryListContent(selectedItemId: $selectedItemId)` with `.navigationTitle("Library")`. Detail: if `selectedItemId` is set, show `ItemDetailView(itemId:)` (placeholder text "Coming in US2" for now); else show `ContentUnavailableView("Select an Item", systemImage: "music.note", description: "Choose an item from your library")`. Include `@State private var selectedItemId: String?`. NavigationSplitView auto-collapses to stack on iPhone. Include `#Preview`.
- [x] T009 [US1] Update `MainTabView` in `ios/Intrada/Navigation/MainTabView.swift` — replace the Library tab placeholder with `LibraryView()`. Remove the "Coming soon" text for the Library tab. Keep other tabs as placeholders.
- [x] T010 [US1] Run `just ios-swift-check` to verify compilation. Fix any type errors, missing imports, or argument ordering issues.

**Checkpoint**: Library tab shows scrollable list with filter tabs, skeleton loading, empty state. iPad shows split view. Can tap items (detail is placeholder). Run `just ios-swift-check` passes.

---

## Phase 3: User Story 2 — View Item Detail (Priority: P1)

**Goal**: Users tap an item to see full details — title, composer, key, tempo, notes, tags, timestamps. Practice summary shown when available with session count, total minutes, latest score, score history, and tempo history. iPad shows metadata and practice side-by-side.

**Independent Test**: Tap any item → detail view shows all populated fields. Item with practice history shows summary card. Item without practice shows no summary. Missing optional fields are omitted. iPad shows two-column layout.

**Functional Requirements**: FR-005, FR-006, FR-007, FR-024

### Components for User Story 2

- [x] T011 [P] [US2] Create `ScoreHistoryList` component in `ios/Intrada/Components/ScoreHistoryList.swift` — displays a list of `ScoreHistoryEntry` items. Each row: date (textFaint, formatted from RFC3339 string), score badge (1-5 with star icon, using accent colours). Takes `[ScoreHistoryEntry]` parameter. If list is empty, don't render. Include `#Preview`.

### Views for User Story 2

- [x] T012 [US2] Create `ItemDetailView` in `ios/Intrada/Views/Library/ItemDetailView.swift` — full item detail view. Takes `itemId: String` parameter. Looks up item from `core.viewModel.items.first(where: { $0.id == itemId })`. Shows: title (large, heading font), subtitle/composer (textSecondary), `TypeBadge`, metadata section in `CardView` (key label + value, tempo label + value — only show populated fields), tags as pills (each with borderDefault, badgeBg), notes in `CardView` (if present, with whitespace-preserving text), timestamps (textFaint, "Created" / "Updated" formatted dates). Practice summary section (if `item.practice` exists): `StatCardView` row for session count, total minutes, latest score; `ScoreHistoryList` for score history; tempo history entries. iPad layout: use `ViewThatFits` or `@Environment(\.horizontalSizeClass)` to show metadata and practice side-by-side on regular width. Toolbar "..." menu with Edit and Delete (placeholder actions for now — "Coming in US4/US5"). Include `#Preview`.
- [x] T013 [US2] Wire `ItemDetailView` into `LibraryView` in `ios/Intrada/Views/Library/LibraryView.swift` — replace the placeholder detail content with `ItemDetailView(itemId: id)`. On iPhone (NavigationStack), add `NavigationLink` or `.navigationDestination(for: String.self)` to push detail from list row tap.
- [x] T014 [US2] Run `just ios-swift-check` to verify compilation. Fix any issues.

**Checkpoint**: Tapping an item shows full detail view with all fields, practice summary, and adaptive iPad layout. Run `just ios-swift-check` passes.

---

## Phase 4: User Story 3 — Add New Item (Priority: P1)

**Goal**: Users tap "Add Item", choose Piece or Exercise, fill in fields with validation, get autocomplete on composer and tags, and see the new item in the list after submission.

**Independent Test**: Tap "+" → fill form → submit → see success toast → new item in list. Try submitting empty required fields → see inline validation errors. Composer autocomplete shows existing composers. Tag input shows existing tags.

**Functional Requirements**: FR-008, FR-009, FR-010, FR-011, FR-012, FR-013, FR-016, FR-020

### Components for User Story 3

- [x] T015 [P] [US3] Create `AutocompleteField` component in `ios/Intrada/Components/AutocompleteField.swift` — text field with dropdown suggestions overlay. Takes: label, @Binding text, placeholder, hint, error (String?), suggestions ([String]), minChars (default 2), maxSuggestions (default 8). Shows filtered suggestions (case-insensitive, prefix-first ranking) in a dropdown below the field when text length >= minChars. Tap suggestion to set text. Uses `.inputStyle()` modifier on text field. Shows `FormFieldError` when error is non-nil. Include `#Preview`.
- [x] T016 [P] [US3] Create `TagInputView` component in `ios/Intrada/Components/TagInputView.swift` — chip-based multi-tag input with autocomplete. Takes: @Binding tags ([String]), availableTags ([String]), error (String?). Displays current tags as removable pills (tag text + "×" button). Inline text field for entering new tags. On submit (return key): trim, check non-empty, check not duplicate (case-insensitive), add to tags array. Autocomplete dropdown from availableTags (excluding already-selected, case-insensitive). Shows `FormFieldError` when error is non-nil. Include `#Preview`.

### Views for User Story 3

- [x] T017 [US3] Create `AddItemView` in `ios/Intrada/Views/Library/AddItemView.swift` — form for creating a new library item. Presented as a sheet (iPad) or pushed view (iPhone). Contains: `TypeTabs` (interactive, bound to `@State itemKind: ItemKind`, default `.piece`). Form fields: title (`TextFieldView`, required), composer (`AutocompleteField` with `uniqueComposers`, required when `itemKind == .piece`), key (`TextFieldView`, placeholder "e.g. C Major"), tempo marking (`TextFieldView`), BPM (`TextFieldView`, keyboard `.numberPad`, placeholder "1-400"), notes (`TextAreaView`, hint "Practice notes, goals, or reminders"), tags (`TagInputView` with `uniqueTags`). Cancel button (toolbar leading, dismisses). Save button (toolbar trailing or bottom `ButtonView.primary`). On save: run `LibraryFormValidator.validate()`, if errors show inline via `errors` map, if valid construct `CreateItem` and dispatch `Event.item(.add(createItem))`, show success toast via `toastManager.show()`, dismiss/navigate back. Read `@Environment(IntradaCore.self)`, `@Environment(ToastManager.self)`, `@Environment(\.dismiss)`. Include `#Preview`.
- [x] T018 [US3] Wire add item navigation — in `LibraryListContent`, make the "+" toolbar button present `AddItemView` (as `.sheet` on iPad via `@State showAddSheet`, or `NavigationLink` push on iPhone). In `EmptyStateView` action, also navigate to add.
- [x] T019 [US3] Run `just ios-swift-check` to verify compilation. Fix any issues.

**Checkpoint**: Can add new items with full validation, autocomplete on composer and tags. Success toast and return to list. Run `just ios-swift-check` passes.

---

## Phase 5: User Story 4 — Edit Item (Priority: P2)

**Goal**: Users edit existing items from the detail view. All fields pre-populated. Type is read-only. Changes saved via UpdateItem with Option<Option<T>> semantics.

**Independent Test**: From detail "..." menu → tap Edit → form pre-populated → change fields → save → return to detail → see updated values. Clear optional field → save → field hidden in detail.

**Functional Requirements**: FR-009, FR-010, FR-011, FR-014, FR-015, FR-017, FR-020

### Views for User Story 4

- [x] T020 [US4] Create `EditItemView` in `ios/Intrada/Views/Library/EditItemView.swift` — form for editing an existing library item. Takes `itemId: String`. Looks up item from `core.viewModel.items`. Pre-populates all `@State` form fields from item data: title, composer (from subtitle), key, tempoMarking and bpm (parsed from tempo string — extract marking text and BPM number), notes, tags. Shows `TypeTabs` in display-only mode (item's type, not editable). Same form fields as AddItemView (title, composer, key, tempo marking, BPM, notes, tags) with same validation. On save: run `LibraryFormValidator.validate()`, construct `UpdateItem` using `Option<Option<T>>` semantics — `Some(Some(value))` for non-empty fields, `Some(None)` for cleared optional fields, compare with original to only send changes. Dispatch `Event.item(.update(id: itemId, input: updateItem))`. Show success toast, dismiss. Read `@Environment(IntradaCore.self)`, `@Environment(ToastManager.self)`, `@Environment(\.dismiss)`. Include `#Preview`.
- [x] T021 [US4] Wire edit action in `ItemDetailView` in `ios/Intrada/Views/Library/ItemDetailView.swift` — make the "Edit" option in the toolbar "..." menu present `EditItemView(itemId:)` as a sheet or pushed view. Add `@State showEditSheet: Bool` and `.sheet(isPresented:)`.
- [x] T022 [US4] Run `just ios-swift-check` to verify compilation. Fix any issues.

**Checkpoint**: Can edit items with pre-populated fields, read-only type, validation, and UpdateItem semantics. Run `just ios-swift-check` passes.

---

## Phase 6: User Story 5 — Delete Item (Priority: P2)

**Goal**: Users delete items from the detail view via the overflow menu. Confirmation dialog prevents accidental deletion.

**Independent Test**: From detail "..." menu → tap Delete → confirmation dialog → confirm → item removed, returned to list, success toast. Cancel → nothing happens.

**Functional Requirements**: FR-018, FR-019, FR-020

### Implementation for User Story 5

- [x] T023 [US5] Implement delete with confirmation in `ItemDetailView` in `ios/Intrada/Views/Library/ItemDetailView.swift` — make the "Delete" option in the toolbar "..." menu set `@State showDeleteConfirmation = true`. Add `.confirmationDialog("Delete \(item.title)?", isPresented: $showDeleteConfirmation)` with message "This cannot be undone." and destructive "Delete" button that dispatches `Event.item(.delete(id: itemId))`, shows success toast, clears `selectedItemId` (on iPad to return to placeholder), or pops navigation (on iPhone). Cancel button dismisses dialog.
- [x] T024 [US5] Run `just ios-swift-check` to verify compilation. Fix any issues.

**Checkpoint**: Can delete items with confirmation. Success toast and return to list. Run `just ios-swift-check` passes.

---

## Phase 7: User Story 6 — Search Library (Priority: P3)

**Goal**: Users type in a search field to filter items by title or composer.

**Independent Test**: Type in search → list filters to matching items. Clear search → all items return. No matches → "No results" message.

**Functional Requirements**: FR-023

### Implementation for User Story 6

- [x] T025 [US6] Add search to `LibraryListContent` in `ios/Intrada/Views/Library/LibraryListContent.swift` — add `.searchable(text: $searchText, prompt: "Search by title or composer")` modifier. On search text change, dispatch `Event.setQuery(ListQuery(text: searchText.isEmpty ? nil : searchText, itemType: filterTab.itemKind))`. Combine search with existing type filter — both search text AND type filter should be included in the `ListQuery`. When search active + no results, show "No results" empty state (different from "no items" empty state). Clear search restores full filtered list.
- [x] T026 [US6] Run `just ios-swift-check` to verify compilation. Fix any issues.

**Checkpoint**: Search filters items by title/composer. Combined with type filter. No-results state shown. Run `just ios-swift-check` passes.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, error handling, and cross-platform consistency

- [x] T027 [P] Add error handling to `LibraryListContent` in `ios/Intrada/Views/Library/LibraryListContent.swift` — when `core.viewModel.error` is non-nil, show `ErrorBanner` at top of list with retry action that dispatches `Event.startApp(apiBaseUrl:)` to re-fetch data (FR-021)
- [x] T028 [P] Add error toast handling to `AddItemView` and `EditItemView` — when `core.viewModel.error` changes after form submission, show error toast via `toastManager.show(message, variant: .danger)` (FR-020)
- [x] T029 [P] Review all views for Dynamic Type support — ensure text scales with system text size setting, no fixed heights that clip text, ScrollView wraps long content
- [x] T030 [P] Review all views for iPad landscape layout — verify NavigationSplitView adapts, detail content uses available width, forms don't stretch uncomfortably wide (max-width constraint if needed)
- [x] T031 Run full validation: `just ios-swift-check` (compile), `just ios-smoke-test` (runtime crash check), `just ios-preview-check` (preview compilation)
- [x] T032 Run quickstart.md verification checklist (V1–V7) on both iPhone and iPad simulators
- [x] T033 Update `CLAUDE.md` — add new components (LibraryItemRow, TypeTabs, TagInputView, AutocompleteField, ScoreHistoryList) to the iOS components table. Add LibraryView, LibraryListContent, ItemDetailView, AddItemView, EditItemView to the Views section.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (US1 Browse)**: Depends on Phase 1 — BLOCKS US2, US4, US5, US6
- **Phase 3 (US2 Detail)**: Depends on Phase 2 (needs list navigation to detail)
- **Phase 4 (US3 Add)**: Depends on Phase 2 (needs list for add entry point). Can run in parallel with Phase 3.
- **Phase 5 (US4 Edit)**: Depends on Phase 3 (edit triggered from detail view) AND Phase 4 (shares form components)
- **Phase 6 (US5 Delete)**: Depends on Phase 3 (delete triggered from detail view). Can run in parallel with Phase 5.
- **Phase 7 (US6 Search)**: Depends on Phase 2 (search is on list view). Can run in parallel with Phases 3-6.
- **Phase 8 (Polish)**: Depends on all story phases

### User Story Dependencies

```text
Phase 1 (Setup)
    │
    v
Phase 2 (US1: Browse) ──── MVP CHECKPOINT
    │         │         │
    v         v         v
Phase 3    Phase 4    Phase 7
(US2)      (US3)      (US6)
    │         │
    v         │
Phase 5 ◄─────┘
(US4: Edit — needs detail + form components)
    │
Phase 6 (can also start after Phase 3)
(US5: Delete)
    │
    v
Phase 8 (Polish)
```

### Within Each User Story

1. Components (marked [P]) can be built in parallel
2. Views depend on their components
3. Wiring/navigation depends on views
4. Compile check (`just ios-swift-check`) after each phase

### Parallel Opportunities

**Within Phase 2 (US1)**: T004, T005, T006 can all run in parallel (separate component files)
**Within Phase 4 (US3)**: T015, T016 can run in parallel (separate component files)
**After Phase 2**: US2 (Phase 3), US3 (Phase 4), and US6 (Phase 7) can start in parallel
**After Phase 3+4**: US4 (Phase 5) and US5 (Phase 6) can run in parallel
**Phase 8**: T027, T028, T029, T030 can all run in parallel

---

## Parallel Example: Phase 2 (User Story 1)

```text
# Launch all components in parallel:
Task T004: "Create LibraryItemRow in ios/Intrada/Components/LibraryItemRow.swift"
Task T005: "Create TypeTabs in ios/Intrada/Components/TypeTabs.swift"
Task T006: "Create LibrarySkeletonView in ios/Intrada/Views/Library/LibrarySkeletonView.swift"

# Then sequentially:
Task T007: "Create LibraryListContent" (depends on T004, T005, T006)
Task T008: "Create LibraryView" (depends on T007)
Task T009: "Update MainTabView" (depends on T008)
Task T010: "Run ios-swift-check" (depends on T009)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: US1 Browse (T004–T010)
3. **STOP and VALIDATE**: `just ios-swift-check` + `just ios-smoke-test`
4. Library tab shows items, filter works, skeleton/empty states work, iPad split view works
5. This is a shippable increment — users can browse their library on iOS

### Incremental Delivery

1. Phase 1 + Phase 2 → **MVP**: Browse library with filter tabs
2. + Phase 3 (US2) → View item details with practice summary
3. + Phase 4 (US3) → Add new items with validation and autocomplete
4. + Phase 5 (US4) → Edit existing items
5. + Phase 6 (US5) → Delete items with confirmation
6. + Phase 7 (US6) → Search library
7. + Phase 8 → Polish, error handling, validation
8. Each increment is independently testable and adds user value

### Total: 33 tasks across 8 phases

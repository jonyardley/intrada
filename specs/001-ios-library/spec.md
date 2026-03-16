# Feature Specification: iOS Library — Browse, Search & Manage Repertoire

**Feature Branch**: `001-ios-library`
**Created**: 2026-03-16
**Status**: Draft
**Input**: GitHub issue #195 — iOS: Library — browse, search & manage repertoire

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Browse Library (Priority: P1)

A musician opens the app and sees their full repertoire — pieces and exercises — as a scrollable list. They can quickly scan titles, composers, keys, tempos, tags, and practice history to decide what to work on. They can filter between pieces and exercises using a toggle. On iPad, items are displayed in a multi-column grid that makes use of the wider screen.

**Why this priority**: The library list is the foundation of the app. Without it, no other library feature has context. This is the first screen users see in the Plan tab.

**Independent Test**: Can be fully tested by signing in and viewing the Library tab. Delivers immediate value by giving users access to their repertoire.

**Acceptance Scenarios**:

1. **Given** a user with library items, **When** they open the Library tab, **Then** they see a scrollable list of all items with title, composer, type badge, key, tempo, and tags displayed for each item.
2. **Given** a user with both pieces and exercises, **When** they tap the "Pieces" or "Exercises" toggle, **Then** the list filters to show only items of the selected type.
3. **Given** a user with no library items, **When** they open the Library tab, **Then** they see an empty state with a message and a prompt to add their first item.
4. **Given** data is loading, **When** the user opens the Library tab, **Then** they see skeleton placeholders until data arrives.
5. **Given** a user on iPad, **When** they open the Library tab, **Then** items are displayed in a multi-column grid that makes use of the wider screen.

---

### User Story 2 — View Item Detail (Priority: P1)

A musician taps an item in the library list to see its full details — title, composer, key, tempo (marking and BPM), notes, tags, and creation/update timestamps. If they've practised the item before, they also see a practice summary — session count, total minutes, latest confidence score, score history, and tempo progress. On iPad, metadata and practice history are shown side by side.

**Why this priority**: Detail view is essential for the musician to review what they know about a piece before practising. It's also the gateway to edit and delete actions.

**Independent Test**: Can be tested by tapping any item card and verifying all fields display correctly, including practice summary when available.

**Acceptance Scenarios**:

1. **Given** an item with all fields populated, **When** the user taps it, **Then** a detail view shows title, composer, type badge, key, tempo, notes, tags, and timestamps.
2. **Given** an item with practice history, **When** viewing its detail, **Then** session count, total minutes, latest score, score history entries (date + score), and tempo history are displayed.
3. **Given** an item with no practice history, **When** viewing its detail, **Then** the practice summary section is not shown.
4. **Given** an item with only some optional fields populated, **When** viewing detail, **Then** missing fields are gracefully omitted rather than showing empty labels.
5. **Given** a user on iPad, **When** they view item detail, **Then** the layout uses side-by-side content areas (metadata and practice history) rather than stacking everything vertically.

---

### User Story 3 — Add New Item (Priority: P1)

A musician wants to add a new piece or exercise to their library. They tap "Add Item", choose Piece or Exercise, fill in the title (required), composer (required for pieces), and optionally key, tempo marking, BPM, notes, and tags. The tag input and composer field suggest existing values from their library. On save, the item appears in the library list.

**Why this priority**: Users must be able to build their library from the iOS app. Without add, the library is read-only.

**Independent Test**: Can be tested by tapping "Add Item", filling the form, submitting, and verifying the new item appears in the list.

**Acceptance Scenarios**:

1. **Given** the user taps the add button, **When** the add form appears, **Then** they see type tabs (Piece/Exercise), title field, and all optional fields.
2. **Given** a user selects "Piece", **When** they leave composer empty and submit, **Then** they see a validation error on the composer field.
3. **Given** a user selects "Exercise", **When** they leave composer empty and submit, **Then** the form submits successfully (composer is optional for exercises).
4. **Given** a user types in the composer field, **When** existing composers match the input, **Then** autocomplete suggestions appear below the field.
5. **Given** a user adds tags, **When** they type and press enter or tap a suggestion, **Then** the tag appears as a removable chip.
6. **Given** a user submits a valid form, **When** the item is created, **Then** they are returned to the library list and see a success toast.
7. **Given** a user enters a title longer than 500 characters, **When** they submit, **Then** they see a validation error.

---

### User Story 4 — Edit Item (Priority: P2)

A musician wants to update details of an existing item — correct a composer name, add a tempo target, update notes, or manage tags. They can edit any field except the item type (piece/exercise), which is immutable after creation.

**Why this priority**: Important for maintaining an accurate library, but users can function with add-only initially.

**Independent Test**: Can be tested by navigating to an item's detail view, tapping edit, changing fields, and saving.

**Acceptance Scenarios**:

1. **Given** a user taps "Edit" on an item detail view, **When** the edit form appears, **Then** all fields are pre-populated with the item's current data.
2. **Given** a user is editing, **When** they view the type tabs, **Then** the item's type is displayed but not editable.
3. **Given** a user changes the title and saves, **When** they return to the detail view, **Then** the updated title is shown immediately.
4. **Given** a user clears an optional field (e.g. notes) and saves, **When** they return to the detail view, **Then** the field is no longer displayed.

---

### User Story 5 — Delete Item (Priority: P2)

A musician wants to remove an item they no longer practise from their library. A confirmation step prevents accidental deletion.

**Why this priority**: Necessary for library hygiene, but less urgent than CRUD creation.

**Independent Test**: Can be tested by navigating to an item, tapping delete, confirming, and verifying the item is removed from the list.

**Acceptance Scenarios**:

1. **Given** a user taps "Delete" on an item detail view, **When** the confirmation dialog appears, **Then** they see the item's title and a warning that this cannot be undone.
2. **Given** a user confirms deletion, **When** the item is deleted, **Then** they are returned to the library list, the item is gone, and a success toast is shown.
3. **Given** a user cancels deletion, **When** the confirmation dialog dismisses, **Then** nothing changes.

---

### User Story 6 — Search Library (Priority: P3)

A musician with a large library wants to find a specific item quickly by typing part of the title or composer name.

**Why this priority**: Valuable for large libraries but not essential for initial launch — users can scroll and filter by type.

**Independent Test**: Can be tested by typing in the search field and verifying matching items appear.

**Acceptance Scenarios**:

1. **Given** a user types in the search field, **When** the text matches item titles or composers, **Then** the list filters to show only matching items.
2. **Given** a user clears the search field, **When** the field is empty, **Then** all items are shown again.
3. **Given** a search matches no items, **When** the results are empty, **Then** a "no results" message is shown.

---

### Edge Cases

- What happens when the API returns an error while loading items? — Error banner shown with retry option.
- What happens when creating an item fails (network error)? — Error toast shown, form data preserved so the user can retry.
- What happens when a user adds a duplicate tag? — Tag is silently ignored (case-insensitive deduplication).
- What happens when tempo has only a marking, only BPM, or both? — Display adapts: "Allegro (132 BPM)", "Allegro", or "132 BPM".
- What happens on very long titles or composer names? — Text truncates with ellipsis in the list view, wraps in the detail view.
- What happens when the user rotates their device? — Layout adapts between portrait and landscape on both iPhone and iPad.
- What happens when the user has hundreds of items? — The list must scroll smoothly without performance degradation.
- What happens if the user switches type tabs rapidly? — Only the final filter state is applied.
- What happens if the user navigates away from a form with unsaved changes? — Standard iOS back-navigation behaviour applies (no unsaved-changes warning in v1).
- What happens when two tempo values exist (achieved and target)? — The card shows both (e.g. "108 / 120 BPM").

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display all library items from the ViewModel in a scrollable list, showing title, composer, type badge, key, tempo, and tags on each item.
- **FR-002**: System MUST provide type filter tabs (All / Pieces / Exercises) that filter the displayed items.
- **FR-003**: System MUST show a loading skeleton while the library data is being fetched.
- **FR-004**: System MUST show an empty state with a call-to-action when no items exist (or no items match the current filter).
- **FR-005**: System MUST navigate to a detail view when the user taps a library item.
- **FR-006**: The detail view MUST display all populated fields: title, composer, type, key, tempo, notes, tags, created date, and updated date.
- **FR-007**: The detail view MUST show a practice summary section when practice data exists: session count, total minutes, latest confidence score, score history, and tempo history.
- **FR-008**: System MUST provide an "Add Item" entry point accessible from the library list.
- **FR-009**: The add/edit form MUST allow selection of item type (Piece or Exercise) on create, and display type as read-only on edit.
- **FR-010**: The add/edit form MUST validate all inputs according to the core validation rules: title required (max 500), composer required for pieces (max 200), BPM 1–400 if provided, tempo marking max 100, notes max 5000, tags max 100 chars each.
- **FR-011**: Validation errors MUST appear inline on the relevant field.
- **FR-012**: The tag input MUST provide autocomplete suggestions from the user's existing library tags.
- **FR-013**: The composer field MUST provide autocomplete suggestions from existing composers in the library.
- **FR-014**: The edit form MUST pre-fill all fields with the item's current values.
- **FR-015**: The edit form MUST support clearing optional fields (setting them to empty).
- **FR-016**: After successful creation, the user MUST be returned to the library list with a success toast.
- **FR-017**: After successful update, the user MUST be returned to the item detail view with a success toast.
- **FR-018**: Deletion MUST require explicit confirmation via a dialog before proceeding.
- **FR-019**: After successful deletion, the user MUST be returned to the library list with a success toast.
- **FR-020**: System MUST display a toast notification when an API operation fails.
- **FR-021**: System MUST display an error banner with retry when library data fails to load.
- **FR-022**: Library item cards MUST display combined tempo when both achieved and target tempo exist (e.g. "108 / 120 BPM").
- **FR-023**: System MUST provide a text search that filters items by title and composer.
- **FR-024**: System MUST provide iPad-optimised layouts that take advantage of wider screens (multi-column grids, side-by-side content areas) rather than scaling up the iPhone layout.
- **FR-025**: System MUST support both portrait and landscape orientations on iPhone and iPad.

### Key Entities

- **Library Item**: A piece or exercise in the user's repertoire. Has a title, type (piece/exercise), optional composer, key, tempo (marking and/or BPM), notes, and tags. Tracks practice history including session count, total minutes, confidence scores, and tempo progress.
- **Tag**: A user-defined label attached to items for organisation. Case-insensitive, max 100 characters. Autocompleted from existing tags in the library.
- **Practice Summary**: A read-only projection of practice data for an item, showing session count, total practice time, latest confidence score (1–5), score history, and tempo history.

## Design *(include if feature has UI)*

### Existing Components Used

- **CardView** — Container for item detail sections and practice summary
- **ButtonView** — Add, Edit, Delete, Save, Cancel actions
- **TextFieldView** — Form inputs for title, composer, key, tempo marking, tempo BPM
- **TextAreaView** — Notes field in add/edit forms
- **TypeBadge** — Piece/Exercise indicator on list items and detail view
- **StatCardView** — Practice statistics (session count, total minutes, latest score)
- **PageHeading** — "Library" heading on the list view
- **SkeletonLine / SkeletonBlock** — Loading state placeholders
- **BackLink** — Navigation back from detail/form views
- **ErrorBanner** — API error display with retry
- **FormFieldError** — Inline validation errors on form fields
- **Toast** — Success/error notifications
- **EmptyStateView** — No items in library

### New Components Needed

- **LibraryItemRow**: A list row displaying an item's title, composer, type badge, key, tempo, tags, and latest practice info. Tappable to navigate to detail. Adapts to available width.
- **TypeTabs (iOS)**: A segmented control to toggle between All / Pieces / Exercises filtering (interactive on list/add, display-only on edit).
- **TagInput (iOS)**: A chip-based input for adding/removing tags with autocomplete suggestions from existing tags.
- **AutocompleteField**: A text input with dropdown suggestions (used for composer and tag inputs).
- **SearchBar**: A search input for filtering the library list by title/composer.
- **ScoreHistoryList**: Displays a list of past confidence scores with dates.
- **TempoProgressView**: Visualises tempo progress over time for an item.

### Wireframe / Layout Description

Detailed visual designs will be created in Pencil (`design/intrada.pen`) after this spec is approved. Designs will cover:

- Library list (iPhone portrait, iPad portrait, iPad landscape)
- Item detail (iPhone, iPad)
- Add item form (iPhone, iPad)
- Edit item form (iPhone, iPad)
- Empty state, loading state, error state
- Delete confirmation
- Search active state

### Responsive Behaviour

- **iPhone (compact width)**: Single-column scrollable list. Forms use full-width fields stacked vertically. Detail view is a scrollable single column.
- **iPad (regular width, portrait)**: Library list uses a 2–3 column grid. Detail view uses side-by-side layout (metadata left, practice history right). Forms use a centred, comfortable-width layout.
- **iPad (regular width, landscape)**: Library list uses a 3–4 column grid. Detail view and forms make full use of horizontal space with multi-column layouts.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can browse their library, view item details, and return to the list within 3 taps.
- **SC-002**: Users can add a new item to their library in under 60 seconds.
- **SC-003**: The library list loads and displays items within 2 seconds of opening the tab.
- **SC-004**: All form validation errors are visible inline next to the relevant field without scrolling away from the error.
- **SC-005**: iPad users see a meaningfully different layout that uses the larger screen — not a scaled-up iPhone view.
- **SC-006**: The iOS library experience is visually consistent with the web app (same dark glassmorphism aesthetic, same information hierarchy).
- **SC-007**: 100% of library operations available on web (browse, filter, search, view, add, edit, delete) are available on iOS.
- **SC-008**: The library list scrolls smoothly with 100+ items with no visible frame drops.

## Assumptions

- The Crux core already handles all business logic, HTTP requests, and state management. The iOS shell only needs to process effects and render the ViewModel.
- Authentication (Clerk) is already integrated in the iOS shell and provides valid JWT tokens for API calls.
- The existing iOS design system (tokens, modifiers, base components) from #194 is complete and available.
- Tag autocomplete suggestions are derived from existing tags across all items in the ViewModel — no additional API endpoint is needed.
- Composer autocomplete suggestions are derived from existing composers across all items in the ViewModel — no additional API endpoint is needed.
- The item type (Piece/Exercise) cannot be changed after creation — this is an existing business rule.
- Category field has been removed (PR #218) — items use tags only for organisation.
- Tempo display follows the web pattern: "Allegro (132 BPM)", "132 BPM", or "Allegro" depending on which fields are populated.

## Out of Scope

- Sort controls (future enhancement)
- Bulk operations (multi-select, bulk delete)
- Item import/export
- Offline support (tracked separately as #41)
- Session/routine references from item detail (will come with session features #196–#199)
- Unsaved changes warning on form back-navigation (future enhancement)

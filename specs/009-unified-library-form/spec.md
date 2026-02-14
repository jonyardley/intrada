# Feature Specification: Unified Library Item Form

**Feature Branch**: `009-unified-library-form`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Merge piece and exercise forms into variations of the same library item (maybe call it a study? what do you think?) and on the add form add a top tab to switch between the two types and change the form dynamically based on the users selection."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Add Library Item via Unified Form with Type Tabs (Priority: P1)

A musician opens the "Add" form and sees a tabbed interface at the top with two options: "Piece" and "Exercise". The "Piece" tab is selected by default. The form displays fields appropriate for a piece (title, composer required, key, tempo, notes, tags). When the musician switches to the "Exercise" tab, the form updates dynamically: the composer field becomes optional, and a new "Category" field appears. The musician fills out the form and saves. The item is created with the correct type based on which tab was active at the time of submission.

**Why this priority**: This is the core interaction change — replacing two separate "Add" forms with a single unified form. Without this, the feature has no value.

**Independent Test**: Open the add form, verify tabs appear, switch between tabs and confirm the form fields change appropriately, fill out and submit the form for each type, and verify the correct item type is created.

**Acceptance Scenarios**:

1. **Given** the user is on the library list, **When** they click "Add", **Then** a single unified form opens with "Piece" and "Exercise" tabs visible at the top, with "Piece" selected by default.
2. **Given** the add form is open with "Piece" tab active, **When** the user views the form, **Then** they see: Title (required), Composer (required), Key, Tempo Marking, BPM, Notes, Tags.
3. **Given** the add form is open with "Piece" tab active, **When** the user clicks the "Exercise" tab, **Then** the form dynamically updates to show: Title (required), Composer (optional), Category, Key, Tempo Marking, BPM, Notes, Tags.
4. **Given** the user has filled in some shared fields (e.g. title, composer, key) on the Piece tab, **When** they switch to the Exercise tab, **Then** the shared field values are preserved (title, composer, key, tempo, notes, tags remain filled in).
5. **Given** the add form is open with "Exercise" tab active and all required fields filled, **When** the user clicks Save, **Then** a new exercise is created in the library with the correct type and all entered data.
6. **Given** the add form is open with "Piece" tab active and all required fields filled, **When** the user clicks Save, **Then** a new piece is created in the library with the correct type and all entered data.

---

### User Story 2 — Edit Form Adapts to Item Type (Priority: P2)

When a musician edits an existing library item, the edit form opens with the correct tab already selected based on the item's type (piece or exercise). The form displays the appropriate fields pre-populated with the item's current data. The type tabs are visible but display-only — the user cannot switch the type of an existing item. The tabs serve as a visual indicator of which type is being edited, maintaining visual consistency with the add form.

**Why this priority**: Editing is the natural complement to adding. Users expect a consistent form experience between adding and editing.

**Independent Test**: Open an existing piece for editing, verify the Piece tab is selected and fields are pre-populated. Open an existing exercise for editing, verify the Exercise tab is selected with exercise-specific fields visible and pre-populated.

**Acceptance Scenarios**:

1. **Given** the user clicks "Edit" on a piece, **When** the edit form opens, **Then** the "Piece" tab is selected, all piece fields are visible, and values are pre-populated with the item's current data.
2. **Given** the user clicks "Edit" on an exercise, **When** the edit form opens, **Then** the "Exercise" tab is selected, exercise-specific fields (Category) are visible and pre-populated with the item's current data (if the exercise has a category).
3. **Given** the user is editing a piece and the "Exercise" tab is visible but disabled, **When** the user clicks the "Exercise" tab, **Then** nothing happens — the tab does not switch and the form remains on the Piece configuration.
4. **Given** the user is editing a piece, **When** they modify fields and click Save, **Then** the piece is updated with the new values and the user returns to the detail view.

---

### User Story 3 — Unified URL Structure for Add Form (Priority: P3)

The add form uses a single URL path instead of separate paths for pieces and exercises. The user navigates to the add form via one path, and the active tab determines which type is created. This simplifies the app's routing and creates a more intuitive navigation experience.

**Why this priority**: Streamlining the URL structure is a natural consequence of unifying the form, but the app functions correctly with the existing routes. This is a quality-of-life improvement.

**Independent Test**: Navigate to the unified add form URL, verify it loads the tabbed form. Verify the old piece-specific and exercise-specific add URLs either redirect to the new unified URL or are removed.

**Acceptance Scenarios**:

1. **Given** the user navigates to the add form URL, **When** the page loads, **Then** the unified tabbed form is displayed at a single URL path.
2. **Given** the library list dropdown previously had separate "Piece" and "Exercise" links, **When** the user views the dropdown, **Then** it now shows a single "Add Item" option (or the dropdown is removed in favour of a single add button).
3. **Given** the user opens the add form, **When** they switch between tabs, **Then** the URL does not change (the tab selection is local form state, not part of the URL).

---

### Edge Cases

- What happens when the user switches tabs after entering data in a type-specific field (e.g. fills in "Category" on Exercise tab, then switches to Piece tab)? The Category value should be preserved in memory so switching back restores it, but it should not be submitted when saving as a Piece.
- What happens when the user switches from Piece to Exercise tab but doesn't fill in Category? The Category field is optional, so the form should still submit successfully.
- What happens if the user switches from Piece tab (where Composer is required) to Exercise tab (where Composer is optional) and the Composer field is empty? The form should validate according to the currently active tab's rules.
- What happens when the user has validation errors, switches tabs, then switches back? Validation errors should be cleared when switching tabs to avoid showing stale errors for a different form configuration.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The add form MUST display a tabbed interface at the top with two tabs: "Piece" and "Exercise".
- **FR-002**: The "Piece" tab MUST be selected by default when the add form opens.
- **FR-003**: Switching tabs MUST dynamically update the form to show fields appropriate for the selected type: Piece shows Composer (required) without Category; Exercise shows Composer (optional) and Category (optional).
- **FR-004**: Fields shared between both types (Title, Composer, Key, Tempo Marking, BPM, Notes, Tags) MUST retain their values when the user switches between tabs. The Composer field is shared — its value persists across tab switches; only the required/optional validation rule changes per tab.
- **FR-005**: Type-specific field values (Category for Exercise) MUST be preserved in memory when switching away from a tab and restored when switching back, but MUST NOT be included in the submission data for the other type.
- **FR-006**: Form validation MUST apply rules based on the currently active tab: Piece requires title and composer; Exercise requires only title.
- **FR-007**: Validation errors MUST be cleared when the user switches between tabs.
- **FR-008**: The Save button MUST create the correct item type (piece or exercise) based on which tab is active at the time of submission.
- **FR-009**: The edit form MUST open with the correct tab pre-selected based on the item's type and all fields pre-populated.
- **FR-010**: The add form MUST be accessible via a single URL path (replacing the current separate piece and exercise add paths).
- **FR-011**: The library list's "Add" dropdown MUST be replaced with a single "Add Item" action that navigates to the unified form.
- **FR-012**: All existing validation rules for pieces and exercises MUST continue to be enforced (title 1–500 characters, composer 1–200 characters, notes max 5000 characters, tags each 1–100 characters, BPM 1–400).
- **FR-013**: The tab interface MUST be keyboard-accessible: users can navigate between tabs using Arrow Left/Right keys and activate a tab using Enter or Space. On the edit form (display-only tabs), tabs MUST receive focus but arrow keys and activation keys MUST NOT switch the active tab. (See quickstart.md Scenario 7 for full verification steps.)
- **FR-014**: The currently active tab MUST be visually distinct from the inactive tab (clear selected state).
- **FR-015**: On the edit form, the type tabs MUST be display-only (not clickable/switchable). The tab matching the item's type is shown as selected, and the inactive tab is visually disabled. The item's type is immutable after creation.

### Key Entities

- **Library Item**: The umbrella concept for both pieces and exercises. A library item always has a type ("piece" or "exercise"), a required title, and optional shared fields (key, tempo, notes, tags). Type-specific fields: pieces require a composer; exercises have optional composer and optional category.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can add both pieces and exercises from a single form, completing the flow in under 30 seconds for each type.
- **SC-002**: Switching between Piece and Exercise tabs preserves all shared field values with zero data loss.
- **SC-003**: 100% of existing library items can be edited through the unified edit form with the correct type-specific fields displayed.
- **SC-004**: Tab switching between Piece and Exercise completes instantly with no perceptible delay.
- **SC-005**: The total number of distinct add/edit form routes in the application is reduced from 4 to 2 (one add, one edit).
- **SC-006**: All 82+ existing tests continue to pass after the migration, confirming backward compatibility of the underlying data model.

## Clarifications

### Session 2026-02-14

- Q: Should the user be able to switch the type of an existing item during editing, or should the tabs be display-only on the edit form? → A: Tabs are display-only on the edit form. Item type is immutable after creation.
- Q: Is the Composer field shared across tabs (value persists when switching) or type-specific (independent values per tab)? → A: Composer is shared — its value persists across tab switches. Only the required/optional validation rule changes per tab.

## Assumptions

- The underlying data model (separate Piece and Exercise domain entities) remains unchanged. This feature is a UI/UX change only — the form presents a unified interface but creates the same domain entities as before.
- The terms "Piece" and "Exercise" are retained as the tab labels and item type identifiers. No new terminology (e.g. "Study") is introduced, to maintain consistency with the existing CLI and core domain language.
- The "Piece" tab is the default because pieces are the more common item type for most musicians.
- Tab state is local to the form component and is not persisted in the URL or browser history. Only the form page URL itself is routable.
- The edit form retains separate routes per item (e.g. `/library/:id/edit`) since the item type is determined by the existing data, not by user selection.

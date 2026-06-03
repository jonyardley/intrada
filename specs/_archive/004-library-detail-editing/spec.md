# Feature Specification: Library Add, Detail View & Editing

**Feature Branch**: `004-library-detail-editing`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Add to library and detail view / editing"

## Clarifications

### Session 2026-02-14

- Q: How should the add form, detail view, and edit form be presented relative to the library list? → A: Full-page views — list, detail, add form, and edit form are separate views, only one shown at a time.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a New Piece or Exercise (Priority: P1)

A musician wants to add a new piece or exercise to their library from the web application. They click an "Add" button, choose whether they are adding a piece or an exercise, fill in the required fields (title, and composer for pieces), and optionally provide key, tempo, notes, and tags. After submitting, the new item appears in the library list.

**Why this priority**: Adding items is the fundamental write action. Without it, the library is read-only and limited to stub data, providing no real utility to users.

**Independent Test**: Can be fully tested by loading the web app, clicking "Add", filling in fields, submitting, and verifying the new item appears in the library list with correct details.

**Acceptance Scenarios**:

1. **Given** the library page is displayed, **When** the user clicks the "Add" button and selects "Piece", **Then** a form appears with fields for title (required), composer (required), key, tempo marking, BPM, notes, and tags.
2. **Given** the add piece form is displayed, **When** the user fills in title and composer and submits, **Then** the new piece appears in the library list and the form closes.
3. **Given** the add form is displayed, **When** the user submits with an empty title, **Then** a validation error is shown and the form remains open.
4. **Given** the library page is displayed, **When** the user clicks "Add" and selects "Exercise", **Then** a form appears with fields for title (required), composer (optional), category, key, tempo marking, BPM, notes, and tags.
5. **Given** the add exercise form is displayed, **When** the user fills in the title and submits, **Then** the new exercise appears in the library list.
6. **Given** the add form is displayed, **When** the user clicks "Cancel", **Then** the form closes without adding an item and the library is unchanged.

---

### User Story 2 - View Item Details (Priority: P1)

A musician wants to see the full details of a piece or exercise in their library. They click on an item in the library list, and a detail view opens showing all of the item's information: title, composer, type, key, tempo, notes, tags, and when it was created and last updated.

**Why this priority**: Viewing details is the counterpart to the list view. The list shows summary information; users need a way to see everything about an item before they can meaningfully edit or manage it.

**Independent Test**: Can be fully tested by loading the web app, clicking on a library item, and verifying all fields are displayed correctly in the detail view.

**Acceptance Scenarios**:

1. **Given** the library list contains items, **When** the user clicks on a piece, **Then** a detail view is displayed showing title, composer, key, tempo (marking and BPM), notes, tags, created date, and last updated date.
2. **Given** the library list contains items, **When** the user clicks on an exercise, **Then** a detail view is displayed showing title, composer, category, key, tempo, notes, tags, created date, and last updated date.
3. **Given** a detail view is displayed, **When** the user clicks a "Back" button or navigation element, **Then** the detail view closes and the library list is shown.
4. **Given** a detail view is displayed for an item with no optional fields set, **When** the user views it, **Then** empty optional fields are omitted or shown as placeholders (not shown as "null" or blank labels).

---

### User Story 3 - Edit an Existing Item (Priority: P2)

A musician realises they made a mistake or wants to update information about a piece or exercise. From the detail view, they click "Edit", modify the fields they want to change, and save. The updated information is immediately reflected in both the detail view and the library list.

**Why this priority**: Editing builds on the add and detail view functionality. Users need to correct mistakes and update their library as they learn more about a piece.

**Independent Test**: Can be fully tested by viewing an item's detail, clicking Edit, changing a field (e.g., title), saving, and verifying the change is reflected in both the detail view and the list.

**Acceptance Scenarios**:

1. **Given** a detail view is displayed for a piece, **When** the user clicks "Edit", **Then** the view transitions to an editable form pre-populated with the item's current values.
2. **Given** the edit form is displayed, **When** the user changes the title and saves, **Then** the detail view shows the updated title and the library list also reflects the change.
3. **Given** the edit form is displayed, **When** the user clears a required field (title or composer for pieces) and tries to save, **Then** a validation error is shown and the changes are not applied.
4. **Given** the edit form is displayed, **When** the user clears an optional field (e.g., notes) and saves, **Then** the field is successfully cleared.
5. **Given** the edit form is displayed, **When** the user clicks "Cancel", **Then** the form closes, any unsaved changes are discarded, and the detail view shows the original values.

---

### User Story 4 - Delete an Item (Priority: P3)

A musician wants to remove a piece or exercise from their library. From the detail view, they click "Delete", confirm the action, and the item is removed from the library list.

**Why this priority**: Deletion is important for library management but is a less frequent action than adding or editing. It also requires a confirmation step to prevent accidental data loss.

**Independent Test**: Can be fully tested by viewing an item's detail, clicking Delete, confirming, and verifying the item no longer appears in the library list.

**Acceptance Scenarios**:

1. **Given** a detail view is displayed, **When** the user clicks "Delete", **Then** a confirmation prompt appears asking the user to confirm the deletion.
2. **Given** the confirmation prompt is displayed, **When** the user confirms, **Then** the item is removed from the library, the view returns to the library list, and the item count decreases by one.
3. **Given** the confirmation prompt is displayed, **When** the user cancels, **Then** the prompt closes and the item remains in the library unchanged.

---

### Edge Cases

- What happens when the user submits a form with a title at the maximum length (500 characters)? The item should be saved successfully.
- What happens when the user enters a BPM of 0 or 401? A validation error should be shown indicating BPM must be between 1 and 400.
- What happens when the user enters an empty tag? A validation error should be shown.
- What happens when the user is on the detail view and refreshes the page? Since the web app uses in-memory stub data, the app should reload to the library list with the default stub data (no persistence).
- What happens when the user tries to edit an item that was deleted (e.g., by another action)? An error message should be displayed indicating the item was not found.
- What happens when the user enters Unicode characters in form fields (e.g., accented names, CJK text)? The system should accept and display them correctly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The web application MUST provide an "Add" action accessible from the library list view that allows the user to choose between adding a piece or an exercise.
- **FR-002**: The add piece form MUST include fields for: title (required, text), composer (required, text), key (optional, text), tempo marking (optional, text), BPM (optional, number), notes (optional, text area), and tags (optional, comma-separated or individual entry).
- **FR-003**: The add exercise form MUST include fields for: title (required, text), composer (optional, text), category (optional, text), key (optional, text), tempo marking (optional, text), BPM (optional, number), notes (optional, text area), and tags (optional).
- **FR-004**: All form submissions MUST validate input according to the existing domain rules: title 1-500 chars, composer 1-200 chars (required for pieces, optional for exercises), notes max 5000 chars, tags 1-100 chars each, BPM 1-400, category 1-100 chars.
- **FR-005**: Validation errors MUST be displayed inline on the form, next to the relevant field, without clearing the user's other input.
- **FR-006**: Users MUST be able to click on any item in the library list to open a detail view showing all of the item's fields.
- **FR-007**: The detail view MUST display: title, item type (piece or exercise), composer, key, tempo (marking and/or BPM), notes, tags, created date, and last updated date. For exercises, category MUST also be displayed.
- **FR-008**: Optional fields that have no value MUST be omitted from the detail view (not shown as empty labels or "null").
- **FR-009**: The detail view MUST provide an "Edit" action that transitions the display to an editable form pre-populated with the item's current values.
- **FR-010**: The edit form MUST apply the same validation rules as the add form. Successfully saving MUST update both the detail view and the library list.
- **FR-011**: The detail view MUST provide a "Delete" action that displays a confirmation prompt before removing the item.
- **FR-012**: After deleting an item, the view MUST return to the library list and the item count MUST be updated.
- **FR-013**: Both add and edit forms MUST provide a "Cancel" action that discards changes and returns to the previous view.
- **FR-014**: All actions (add, edit, delete) MUST flow through the Crux core event system, maintaining the architecture established in the MVP web shell.
- **FR-015**: The web application MUST continue to work with in-memory stub data (no persistence). Page refresh resets to default data.
- **FR-016**: The application MUST use a full-page view pattern: the library list, detail view, add form, and edit form are separate views with only one displayed at a time. Navigation between views MUST be managed through application state.

### Key Entities

- **Piece**: A musical composition the user is learning or practicing. Required attributes: title, composer. Optional: key, tempo (marking + BPM), notes, tags.
- **Exercise**: A practice exercise or technical study. Required attributes: title. Optional: composer, category, key, tempo, notes, tags.
- **LibraryItemView**: The ViewModel representation of either a piece or exercise, used to render both the list and detail views.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can add a new piece to the library in under 30 seconds (fill form and submit).
- **SC-002**: Users can view the full details of any library item with a single click from the list.
- **SC-003**: Users can edit any field of an existing item and see the change reflected immediately in both the detail and list views.
- **SC-004**: Users can delete an item with a two-step process (click delete, confirm) in under 5 seconds.
- **SC-005**: All validation errors are visible and actionable without losing the user's other form input.
- **SC-006**: The existing test suite (82+ tests) continues to pass without modification.
- **SC-007**: All new interactions work correctly in current versions of Chrome, Firefox, and Safari.

## Scope & Assumptions

### In Scope

- Add piece form with all domain fields
- Add exercise form with all domain fields
- Item detail view displaying all fields
- Edit form pre-populated with current values
- Delete with confirmation
- Inline form validation matching existing domain rules
- Navigation between list view, detail view, and forms
- All actions routed through Crux core events

### Out of Scope

- Browser persistence (data resets on page reload, same as MVP)
- URL-based routing (navigation is managed via application state, not browser URLs)
- Drag-and-drop reordering of library items
- Bulk operations (multi-select, bulk delete)
- Search or filtering from the web UI (existing core filtering logic is available but web UI for it is deferred)
- File attachments (e.g., sheet music PDFs)

### Assumptions

- The web shell continues to use in-memory stub data with no-op write effects (same as the MVP). The Crux core already handles all CRUD events; the web shell just needs to send them.
- Navigation uses full-page views: list, detail, add form, and edit form are separate views with only one shown at a time. View state is managed through application state, not browser URL routing.
- Tag input will use a simple comma-separated text field. A richer tag input component (autocomplete, chips) is deferred.
- Date/time display in the detail view will use a human-readable format.
- The "Add Sample Item" button from the MVP will be retained alongside the new "Add" form for quick testing.

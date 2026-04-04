# Feature Specification: iOS Routines — Create, Edit & Manage Practice Routines

**Feature Branch**: `199-ios-routines`
**Created**: 2026-04-04
**Status**: Draft
**Input**: iOS routines — browse, create, edit, delete reusable practice routines. Load routines into session builder. Save current setlist or completed session as a routine. iPhone and iPad layouts. Replaces Routines tab placeholder.

## User Scenarios & Testing

### User Story 1 — Browse & Manage Routines (Priority: P1)

A musician opens the Routines tab to see their saved practice routines. Each routine shows its name and the items it contains. They can delete routines they no longer need. This is the foundation — without a list view, routines have no visibility.

**Why this priority**: The list view is the entry point for all routine interactions. Everything else builds on being able to see what routines exist.

**Independent Test**: Create a routine (via web or save-as-routine) → go to Routines tab → see routine listed with name and item count → swipe to delete → confirmation → routine removed.

**Acceptance Scenarios**:

1. **Given** the user has saved routines, **When** they tap the Routines tab, **Then** a list of routines is displayed showing name and item count for each.
2. **Given** a routine in the list, **When** the user taps it, **Then** they see the routine's detail — name and ordered list of items with their types.
3. **Given** a routine in the list, **When** the user swipes to delete, **Then** a confirmation appears. On confirm, the routine is removed.
4. **Given** no routines exist, **When** the user opens the Routines tab, **Then** an empty state is shown with a message explaining what routines are.

---

### User Story 2 — Edit a Routine (Priority: P2)

A musician taps "Edit" on a routine to change its name, reorder items, or remove items. They can also add items from their library. Changes are saved when they tap "Save".

**Why this priority**: Editing lets musicians refine routines over time as their practice needs change. It's the natural next step after browsing.

**Independent Test**: Open a routine → tap Edit → change the name → remove an item → reorder remaining items → add an item from library → Save → changes persist.

**Acceptance Scenarios**:

1. **Given** a routine detail view, **When** the user taps "Edit", **Then** the name becomes editable and items show reorder/remove controls.
2. **Given** edit mode, **When** the user changes the routine name, **Then** the new name is shown.
3. **Given** edit mode, **When** the user drags an item to a new position, **Then** the item order updates.
4. **Given** edit mode, **When** the user taps remove on an item, **Then** the item is removed from the routine.
5. **Given** edit mode, **When** the user taps "Add from Library", **Then** they see available library items and can tap to add them.
6. **Given** edit mode with changes, **When** the user taps "Save", **Then** changes are persisted and the view returns to the routine detail.
7. **Given** edit mode, **When** the user taps "Cancel", **Then** changes are discarded.

---

### User Story 3 — Load Routine into Session Builder (Priority: P3)

From the session builder, a musician can load a saved routine to quickly populate the setlist. The routine's items are added to the current setlist (additive — doesn't replace existing items).

**Why this priority**: This is the core value of routines — reducing setup time for repeated practice patterns.

**Independent Test**: Start a new session → load a 3-item routine → setlist shows 3 items → add another item manually → start session with 4 items.

**Acceptance Scenarios**:

1. **Given** the session builder with routines available, **When** the user sees the setlist panel, **Then** a "Load Routine" option is visible.
2. **Given** the user taps "Load Routine", **When** they select a routine, **Then** the routine's items are appended to the current setlist.
3. **Given** items already in the setlist, **When** a routine is loaded, **Then** the routine items are added after existing items (not replacing them).

---

### User Story 4 — Save as Routine (Priority: P4)

From the session builder or session summary, a musician can save the current items as a new routine for future reuse.

**Why this priority**: This completes the routine lifecycle — musicians discover useful practice combinations during sessions and want to save them for next time.

**Independent Test**: Build a 3-item setlist → tap "Save as Routine" → enter name → routine appears in Routines tab.

**Acceptance Scenarios**:

1. **Given** a setlist with items in the session builder, **When** the user taps "Save as Routine", **Then** a name input appears.
2. **Given** the name input, **When** the user enters a name and confirms, **Then** a new routine is created with the current setlist items.
3. **Given** the session summary, **When** the user taps "Save as Routine", **Then** the same flow creates a routine from the completed session's items.
4. **Given** an empty name, **When** the user tries to save, **Then** validation prevents it.

---

### Edge Cases

- What happens when a routine references a library item that was deleted? The routine entry still shows the cached title and type, but the item won't be found when loaded into a setlist.
- What happens when loading a routine into a setlist that already has items? Items are appended, not replaced. Duplicates are allowed.
- What happens when a routine has only one item? Valid — single-item routines are fine for focused practice.

## Requirements

### Functional Requirements

- **FR-001**: The Routines tab MUST display a list of saved routines showing name and item count.
- **FR-002**: Tapping a routine MUST show its detail — name and ordered item list with types.
- **FR-003**: The user MUST be able to delete a routine with confirmation.
- **FR-004**: The user MUST be able to edit a routine's name, reorder items, remove items, and add items from the library.
- **FR-005**: The session builder MUST offer a "Load Routine" option that appends routine items to the setlist.
- **FR-006**: The session builder and session summary MUST offer a "Save as Routine" option with name input.
- **FR-007**: Routine names MUST be validated (non-empty, max 200 characters).
- **FR-008**: An empty state MUST be shown when no routines exist.
- **FR-009**: All routine views MUST support both iPhone and iPad layouts.

### Key Entities

- **RoutineView**: Saved routine — id, name, entry count, ordered entries
- **RoutineEntryView**: Single item in a routine — id, item ID, cached title, cached type, position

## Design

### Existing Components Used

- `ButtonView` — all actions
- `CardView` — content containers
- `TypeBadge` — item type indicators
- `EmptyStateView` — no routines state
- `LibraryQueueRow` — item selection for adding to routines

### New Components Needed

- **RoutineListView**: Routines tab root — list of routine cards, empty state, iPad NavigationSplitView
- **RoutineDetailView**: Routine detail — name, item list, Edit button
- **RoutineEditView**: Edit mode — name field, reorderable item list, add from library, Save/Cancel
- **RoutineSaveForm**: Collapsible save-as-routine form (name input + save button), used in SetlistSheet and SessionSummary

### Responsive Behaviour

- **iPhone**: NavigationStack with list → detail → edit flow
- **iPad**: NavigationSplitView — routine list sidebar, detail/edit in main pane

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can create a routine from a session in under 10 seconds.
- **SC-002**: Loading a routine into the session builder takes a single tap.
- **SC-003**: Routine list loads instantly with 20+ routines.
- **SC-004**: 100% of routine edits (rename, reorder, add, remove) persist correctly.

## Assumptions

- All Crux core events exist: `saveBuildingAsRoutine`, `saveSummaryAsRoutine`, `loadRoutineIntoSetlist`, `deleteRoutine`, `updateRoutine`.
- `RoutineView` and `RoutineEntryView` are already in the ViewModel.
- The `routines` array in the ViewModel contains all saved routines.
- No new API endpoints needed — all persistence flows through existing Crux HTTP effects.
- The SetlistSheetContent already has a placeholder "Load Routine" button that needs wiring.

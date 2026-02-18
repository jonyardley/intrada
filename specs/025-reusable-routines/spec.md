# Feature Specification: Reusable Routines

**Feature Branch**: `025-reusable-routines`
**Created**: 2026-02-18
**Status**: Draft
**Input**: User description: "Allow musicians to save practice setlists as reusable templates ('routines'). These serve as warm-up routines or recurring practice plans that can be inserted into any session with one tap. Reduces daily decision-making and is particularly valuable for musicians who struggle with session initiation."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Save Current Setlist as a Routine (Priority: P1)

A musician has assembled a good practice setlist during session building — perhaps a warm-up sequence of long tones, scales, and a technical exercise. Rather than rebuilding this from scratch next time, they tap "Save as Routine", give it a name like "Morning Warm-up", and the current setlist is saved as a reusable template. The building state is unaffected — they can continue straight into practice.

**Why this priority**: Saving is the foundational action. Without the ability to save a routine, no other routine features (loading, editing, managing) have any value. This is the entry point for the entire feature.

**Independent Test**: Can be fully tested by building a setlist with 2+ items, tapping "Save as Routine", entering a name, and confirming it was saved. Delivers immediate value by capturing a working setlist for future reuse.

**Acceptance Scenarios**:

1. **Given** a user is building a session with at least one item in the setlist, **When** they tap "Save as Routine" and enter the name "Morning Warm-up", **Then** a routine named "Morning Warm-up" is created containing all current setlist entries in their current order.
2. **Given** a user is building a session with items, **When** they save a routine, **Then** the building state remains unchanged — the setlist is still there and they can continue to start practice or modify it.
3. **Given** a user is building a session with no items in the setlist, **When** they look at the interface, **Then** the "Save as Routine" option is not available (hidden or disabled).
4. **Given** a user tries to save a routine with an empty name, **When** they attempt to confirm, **Then** a validation error is shown requesting a name.
5. **Given** a user tries to save a routine with a name exceeding 200 characters, **When** they attempt to confirm, **Then** a validation error is shown indicating the name is too long.

---

### User Story 2 - Load a Routine into a New Session (Priority: P1)

A musician starts building a new practice session and wants to use their saved warm-up routine. They see a list of their saved routines in the session builder, tap "Load" on "Morning Warm-up", and the routine's entries are added to the current setlist. They can then add more items on top, or load a second routine to combine multiple routines into one session.

**Why this priority**: Loading is the core payoff — it eliminates the repetitive work of rebuilding the same setlist every day. Together with saving (Story 1), this forms the minimum viable feature.

**Independent Test**: Can be fully tested by saving a routine (Story 1), then starting a new session and loading it. Verify the routine entries appear in the setlist. Delivers the primary value: one-tap session setup.

**Acceptance Scenarios**:

1. **Given** the user has saved routines, **When** they start building a new session, **Then** they see a section listing their saved routines with name and entry count.
2. **Given** the user has an empty setlist, **When** they tap "Load" on a routine, **Then** the routine's entries are added to the setlist in their saved order.
3. **Given** the user already has items in the setlist, **When** they load a routine, **Then** the routine's entries are appended after the existing items (not replacing them).
4. **Given** the user loads a routine and then loads a second routine, **When** both loads complete, **Then** both routines' entries appear in the setlist, first routine's items followed by the second.
5. **Given** a routine references a library item that has been renamed since the routine was saved, **When** the routine is loaded, **Then** the entry uses the title as it was when the routine was saved (denormalized title).
6. **Given** the user has no saved routines, **When** they start building a session, **Then** no routine section is displayed (or it shows an empty state prompt).

---

### User Story 3 - Save Routine from Session Summary (Priority: P2)

After completing a productive practice session, a musician sees their summary and thinks "that was a great combination — I want to do this again." They tap "Save as Routine" on the summary screen, name it "Bach Recital Prep", and the session's setlist is saved as a routine for future use.

**Why this priority**: This captures the post-practice "that was good" moment. While saving from the building phase covers the proactive case, saving from summary covers the reactive case. It is a secondary entry point — the feature works without it (via building-phase save).

**Independent Test**: Can be fully tested by completing a session, arriving at the summary, tapping "Save as Routine", entering a name, and verifying the routine is saved. Delivers value by capturing successful session patterns.

**Acceptance Scenarios**:

1. **Given** a user is viewing a session summary, **When** they tap "Save as Routine" and enter a name, **Then** a routine is created containing the session's setlist entries in their original order.
2. **Given** a user saves a routine from a summary, **When** they later start a new session and view the routine list, **Then** the routine they saved from the summary appears and can be loaded.

---

### User Story 4 - Manage Routines (Priority: P2)

A musician wants to review, rename, or remove their saved routines. They navigate to a routines management page where they see all saved routines. They can delete a routine they no longer need, or tap into one to edit its name or reorder/add/remove entries.

**Why this priority**: Management ensures routines stay useful over time. Without it, users accumulate stale routines with no way to clean up. However, the feature delivers its core value (save + load) without a management page.

**Independent Test**: Can be tested by navigating to the routines page, seeing a list of routines, deleting one, and editing another. Delivers value by keeping the routine library tidy and up to date.

**Acceptance Scenarios**:

1. **Given** the user has saved routines, **When** they navigate to the routines management page, **Then** they see a list of all routines with their names and entry counts.
2. **Given** the user is on the routines list, **When** they tap delete on a routine and confirm, **Then** the routine is permanently removed.
3. **Given** the user taps edit on a routine, **When** the edit page loads, **Then** they see the routine name and its ordered list of entries.
4. **Given** the user is editing a routine, **When** they change the name and save, **Then** the routine is updated with the new name.
5. **Given** the user is editing a routine, **When** they remove an entry and save, **Then** the entry is removed and remaining entries are reordered.
6. **Given** the user is editing a routine, **When** they reorder entries (drag or move up/down) and save, **Then** entries are persisted in the new order.
7. **Given** the user is editing a routine, **When** they add an entry from the library and save, **Then** the new entry appears in the routine at the chosen position.

---

### User Story 5 - Edit Routine Details (Priority: P3)

A musician wants to refine a routine by adding new items from the library, removing items no longer relevant, or rearranging the practice order — all from a dedicated edit page. This mirrors the setlist-building experience but operates on a saved routine rather than a live session.

**Why this priority**: Detailed editing (add from library, reorder) is a refinement of the basic management in Story 4. Users can work around it by deleting and re-saving routines. It enhances the experience but is not essential for core value.

**Independent Test**: Can be tested by editing a routine, adding a library item, reordering entries, and saving. Delivers value by letting users evolve routines without recreating them.

**Acceptance Scenarios**:

1. **Given** the user is editing a routine, **When** they tap "Add from Library", **Then** they see available library items (pieces and exercises) to add.
2. **Given** the user adds a library item to the routine, **When** they save, **Then** the routine includes the new entry at the end of the list.
3. **Given** the user is editing a routine with 5 entries, **When** they move entry 3 to position 1, **Then** all entries shift to accommodate and positions are recalculated.

---

### Edge Cases

- What happens when a routine's library item has been deleted from the library? The routine retains the entry with its denormalized title. When loaded into a session, the entry still appears but references a non-existent library item. This is acceptable — the musician may want the placeholder as a reminder.
- What happens when the user tries to save a routine with a name that already exists? Duplicate names are allowed. Routines are identified by their unique ID, not by name. Musicians may have "Morning Warm-up v1" and "Morning Warm-up v2" or even two identically named routines for different instruments.
- How many routines can a user save? No hard limit. The system supports an unbounded number of routines. Performance is acceptable with typical musician usage (dozens, not thousands).
- What happens when the user loads a routine into a setlist that already has items from the same routine? The entries are added again as new entries. Loading is purely additive — no deduplication is performed.
- What happens if the user navigates away from the routine edit page without saving? Changes are discarded. No unsaved-changes warning is required for the initial implementation.
- Can a routine be empty (all entries removed during editing)? No. A routine must have at least one entry. The save/update action validates this.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to save the current building-phase setlist as a named routine.
- **FR-002**: Users MUST be able to save a completed session's setlist as a named routine from the session summary.
- **FR-003**: Users MUST be able to load a saved routine into a building-phase setlist, with entries appended after any existing items.
- **FR-004**: Loading a routine MUST create independent copies of entries (new IDs), not references to the routine's entries.
- **FR-005**: Users MUST be able to view a list of all their saved routines.
- **FR-006**: Users MUST be able to delete a saved routine.
- **FR-007**: Users MUST be able to edit a routine's name.
- **FR-008**: Users MUST be able to add, remove, and reorder entries within a saved routine.
- **FR-009**: Routine names MUST be validated: required, maximum 200 characters.
- **FR-010**: Routines MUST contain at least one entry; empty routines cannot be saved or updated.
- **FR-011**: Each routine entry MUST store the library item reference, item title (denormalized), and item type (piece or exercise).
- **FR-012**: Routine data MUST be persisted to the server and survive page reloads and device changes.
- **FR-013**: Routines MUST be fetched on application startup alongside library and session data.
- **FR-014**: The "Save as Routine" option MUST only be visible when the setlist contains at least one entry.
- **FR-015**: The routines management page MUST be accessible via the URL path `/routines`.
- **FR-016**: The routine edit page MUST be accessible via the URL path `/routines/:id/edit`.

### Key Entities

- **Routine**: A named, reusable setlist template. Key attributes: unique identifier, name (1–200 characters), ordered list of entries, creation timestamp, last-updated timestamp.
- **Routine Entry**: A single item within a routine, representing a library piece or exercise. Key attributes: unique identifier, reference to the library item, denormalized item title, item type (piece or exercise), position in the routine's order.

## Design *(include if feature has UI)*

### Existing Components Used

- **SetlistBuilder** — extended with a "Load Routine" section and "Save as Routine" inline form at the bottom of the current setlist card.
- **Session Summary** — extended with a "Save as Routine" button and inline name input.
- **BackLink** — used on the routine edit page to navigate back to `/routines`.
- **PageHeading** — used on the routines list page and routine edit page for the page title.
- **TextField** — used for the routine name input on both the save form and edit page.
- **Glass card** (`glass-card` utility) — used for routine list items and the edit form container.
- **Badge** (piece/exercise) — used to indicate item type in routine entry lists.

### New Components Needed

- **Routine List Item**: Displays a routine's name and entry count within a glass card. Supports "Edit" and "Delete" actions. Used on the `/routines` management page.
- **Routine Save Form**: An inline form within the SetlistBuilder and Session Summary that shows a text input for the routine name and "Save" / "Cancel" buttons. Toggles visibility when the user taps "Save as Routine".
- **Routine Loader**: A section within the SetlistBuilder that lists available routines (name + entry count) with a "Load" button on each. Only visible when saved routines exist.
- **Routine Entry Editor**: An ordered list of routine entries on the edit page with remove buttons and reorder controls (move up/down). Includes an "Add from Library" action.

### Wireframe / Layout Description

**SetlistBuilder (modified)**:
- Below the current setlist entries card, a new "Saved Routines" section appears (only when routines exist). Each routine shows as a row: name on the left, entry count badge on the right, "Load" button.
- Below the setlist entries, a "Save as Routine" link/button. When tapped, it expands to show a text input and Save/Cancel buttons inline.

**Session Summary (modified)**:
- Below the session stats, a "Save as Routine" button. When tapped, it expands to show a text input and Save/Cancel buttons inline, same pattern as in SetlistBuilder.

**Routines List Page (`/routines`)**:
- Page heading: "Routines"
- Back link to home/dashboard
- List of glass cards, one per routine. Each card shows: routine name (left), entry count (right), with Edit and Delete action buttons.
- Empty state message when no routines exist.

**Routine Edit Page (`/routines/:id/edit`)**:
- Back link to `/routines`
- Page heading: "Edit Routine"
- Name text field at the top
- Ordered list of entries below, each showing: position number, item title, item type badge, remove button, move up/down buttons
- "Add from Library" button below the list (opens library item picker)
- "Save" button at the bottom

### Responsive Behaviour

- **Mobile**: All layouts are single-column, full-width. Routine list items stack vertically. The routine save form spans the full width. Entry reorder controls use up/down buttons (no drag-and-drop required).
- **Desktop**: Same single-column layout (the app does not currently use side-by-side layouts for content). Cards and forms may have max-width constraints for readability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can save a building-phase setlist as a named routine in under 5 seconds (tap save, type name, confirm).
- **SC-002**: Users can load a saved routine into a new session in under 3 seconds (one tap on "Load").
- **SC-003**: Users who save at least one routine use it in 50% or more of subsequent sessions within the first week.
- **SC-004**: Routine data persists correctly across page reloads and device switches — saved routines appear identically after refresh.
- **SC-005**: The routines management page allows users to view, edit, and delete routines without navigating to unrelated pages.

## Assumptions

- Routine data follows the same persistence pattern as library items and sessions: stored on the server via the REST API, fetched on application startup.
- Routine names do not need to be unique. Musicians may create similarly or identically named routines.
- Entries are denormalized (title stored in the routine entry) to ensure routines remain readable even if the source library item is later renamed or deleted.
- Reorder controls use up/down buttons rather than drag-and-drop, consistent with the existing setlist builder pattern.
- The routines feature does not require a new entry in the bottom navigation/tab bar. Routines are accessed through the SetlistBuilder (during session building) and a dedicated `/routines` page linked from the app navigation.

## References

- GitHub Issue #38 — Reusable setlists / goals

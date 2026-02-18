# Feature Specification: Drag-and-Drop Session Builder

**Feature Branch**: `026-drag-drop-builder`
**Created**: 2026-02-18
**Status**: Draft
**Input**: User description: "#37 - Drag and drop session builder + make whole bar addable"

## Clarifications

### Session 2026-02-18

- Q: Should up/down arrow buttons remain always visible alongside drag handles, or be hidden/de-emphasised? → A: Always visible — drag handles and arrow buttons both shown on every entry at all times, on both mobile and desktop.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Drag-and-Drop Setlist Reordering (Priority: P1)

A musician building a practice session wants to rearrange setlist items quickly by dragging them to a new position, rather than pressing up/down arrow buttons repeatedly. The user presses and holds on a setlist entry, drags it to the desired position, and releases. The setlist reorders in real time with a visual indicator showing where the item will land.

**Why this priority**: Drag-and-drop is the primary interaction improvement requested in Issue #37. It replaces the tedious up/down arrow buttons with an intuitive, natural interaction that significantly reduces the number of taps needed to reorder a long setlist.

**Independent Test**: Can be fully tested by building a session with 4+ items, dragging one to a new position, and confirming the setlist reflects the new order.

**Acceptance Scenarios**:

1. **Given** a session in the building phase with 3+ setlist entries, **When** the user presses and holds an entry then drags it to a new position, **Then** the entry moves to the new position and all other entries shift accordingly.
2. **Given** a setlist entry being dragged, **When** the user holds the entry over a gap between two other entries, **Then** a visual drop indicator shows where the item will be placed.
3. **Given** a setlist entry being dragged, **When** the user releases it at the new position, **Then** the entry snaps into place and position numbers update immediately.
4. **Given** a setlist entry being dragged, **When** the user drags it back to its original position and releases, **Then** nothing changes (no-op).
5. **Given** a session in the building phase with only 1 entry, **When** the user views the setlist, **Then** the drag handle is visible but dragging has no effect (no crash or error).

---

### User Story 2 - Tap Entire Library Row to Add (Priority: P2)

A musician browsing the library items section wants to add an item to the setlist by tapping anywhere on the row, not just the small "+ Add" button. The entire library item row acts as a tap target, making it faster and easier to build a setlist, especially on mobile where precise taps on small buttons are difficult.

**Why this priority**: This is the second half of Issue #37 ("make whole bar addable"). It improves the add-to-setlist interaction, which is used more frequently than reordering. Wider tap targets follow WCAG accessibility guidelines and reduce frustration on touch devices.

**Independent Test**: Can be fully tested by navigating to the session builder, tapping on a library item row (not the button), and confirming the item appears in the setlist.

**Acceptance Scenarios**:

1. **Given** a session in the building phase with library items available, **When** the user taps anywhere on a library item row, **Then** the item is added to the setlist.
2. **Given** a library item row, **When** the user hovers or taps on it, **Then** the row provides a visual affordance (e.g. cursor change, background highlight) indicating it is interactive.
3. **Given** a library item is added by tapping the row, **When** the item appears in the setlist, **Then** the behaviour is identical to tapping the existing "+ Add" button (same item, same position).

---

### User Story 3 - Touch-Friendly Drag Handle (Priority: P3)

On mobile devices, the drag interaction must feel natural and not interfere with scrolling. A visible drag handle (grip icon) on each setlist entry clearly communicates which area to grab. Touching the handle initiates a drag; touching other parts of the entry does not (preserving normal scrolling behaviour).

**Why this priority**: Without a clearly separated drag handle, the drag interaction could hijack vertical scrolling on touch devices, making the app frustrating to use. This story ensures the drag interaction coexists with normal mobile scrolling.

**Independent Test**: Can be fully tested on a mobile device by scrolling through a long setlist (touch on entry body) without triggering a drag, then pressing the drag handle to initiate reordering.

**Acceptance Scenarios**:

1. **Given** a setlist entry on a touch device, **When** the user touches and drags the drag handle, **Then** the entry begins dragging for reorder.
2. **Given** a setlist entry on a touch device, **When** the user touches and swipes on the entry body (outside the drag handle), **Then** the page scrolls normally without initiating a drag.
3. **Given** a setlist entry on desktop, **When** the user clicks and drags the drag handle or the entry row itself, **Then** the entry can be reordered (desktop allows dragging from anywhere on the row).

---

### Edge Cases

- What happens when the user drags an entry but the setlist has only one item? The drag should be a no-op with no error.
- What happens if the user drags an entry beyond the top or bottom of the setlist? The entry should clamp to the first or last position.
- What happens if the user starts a drag then moves outside the setlist area and releases? The drag should be cancelled and the entry returns to its original position.
- What happens during drag on a very long setlist (10+ items) that requires scrolling? The setlist should auto-scroll when the dragged item reaches the top or bottom edge of the visible area.
- What happens if the user's device has `prefers-reduced-motion` enabled? Drag animations should be suppressed — items snap immediately to their new positions without transition effects.
- What happens if the user drags a library item row? Only setlist entries are draggable; library rows are tap-to-add only.
- What happens when the routine edit page has reorderable entries? The same drag-and-drop interaction should apply there as well.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to reorder setlist entries by drag-and-drop during the session building phase.
- **FR-002**: System MUST display a visible drag handle (grip icon) on each setlist entry to indicate draggability.
- **FR-003**: System MUST show a drop indicator (visual placeholder) at the target position while a drag is in progress.
- **FR-004**: System MUST update entry position numbers immediately after a drag completes.
- **FR-005**: System MUST allow users to add a library item to the setlist by tapping anywhere on the library item row, not just the "+ Add" button.
- **FR-006**: System MUST preserve the existing "+ Add" button on library item rows (the entire row becomes the tap target, button remains for visual clarity).
- **FR-007**: On touch devices, system MUST only initiate a drag when the user interacts with the drag handle, not the entry body, to preserve scroll behaviour.
- **FR-008**: On desktop, system MUST allow drag initiation from the drag handle.
- **FR-009**: System MUST cancel the drag and restore original position if the user releases outside the setlist area.
- **FR-010**: System MUST apply the same drag-and-drop reordering to the routine edit page entry list.
- **FR-011**: System MUST suppress drag animations when the user has `prefers-reduced-motion` enabled.
- **FR-012**: System MUST retain the existing up/down arrow buttons, always visible on every entry (both mobile and desktop), as an accessible alternative to drag-and-drop for keyboard and assistive technology users.

### Key Entities

- **SetlistEntry**: Existing entity — an individual item within a session's setlist. Has an `id`, `item_title`, `item_type`, and `position`. Position is updated on reorder.
- **Library Item**: Existing entity — a piece or exercise in the user's library. Displayed in the builder for adding to the setlist. Now the entire row is a tap target.

## Design *(include if feature has UI)*

### Existing Components Used

- **Card** — Wraps the "Your Setlist" and "Library Items" sections, unchanged.
- **SetlistEntryRow** — Modified to include a drag handle and support drag-and-drop interactions.
- **TypeBadge** — Unchanged, continues to show piece/exercise badge within entry rows.
- **Button** — Unchanged, used for "Start Session" and "Cancel" actions.

### New Components Needed

- **DragHandle**: A grip icon (six-dot or three-line pattern) displayed at the leading edge of each setlist entry row. Communicates draggability. On touch devices, only this element initiates drag. Sized at minimum 44x44px for WCAG touch target compliance.
- **DropIndicator**: A horizontal line or highlighted gap shown between entries during a drag to indicate where the dragged item will land. Uses the `indigo-400` accent colour from the design system for visibility against the dark glass background.

### Wireframe / Layout Description

**Setlist Entry Row (updated)**:
```
[drag-handle] [position#] [title + type badge] [duration] [↑] [↓] [✕]
```
- Drag handle appears as the leftmost element (before position number)
- Grip icon uses `text-gray-500` colour, becoming `text-gray-300` on hover/active
- During drag: the dragged row gets a slight elevation effect (increased opacity, subtle scale) and the original position shows a placeholder gap
- Drop indicator: a 2px `bg-indigo-400` horizontal line between entries at the target position

**Library Item Row (updated)**:
```
[title + type badge]                              [+ Add]
```
- Entire row is now wrapped in an interactive element (clickable)
- Row uses `cursor-pointer` on desktop
- Visual affordance: existing `hover:bg-white/10` already provides feedback
- The "+ Add" button text remains for visual clarity but the tap target is the full row

### Responsive Behaviour

- **Mobile**: Drag handle is the exclusive drag initiator (to preserve scroll). Touch target for drag handle is minimum 44x44px. Library rows use full-width tap targets. Up/down arrow buttons are always visible alongside drag handles.
- **Desktop**: Drag can be initiated from the drag handle. Library rows show pointer cursor on hover. Up/down arrow buttons are always visible alongside drag handles.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can reorder a setlist entry from position 5 to position 1 in a single drag gesture, compared to 4 taps with the previous up-arrow approach.
- **SC-002**: Users can add a library item to the setlist by tapping anywhere on the item row, with the tap target being at least 44px tall (WCAG 2.5.5 compliance).
- **SC-003**: Drag-and-drop interactions complete without visual glitch or layout shift on both mobile and desktop viewports.
- **SC-004**: Vertical scrolling continues to work normally on touch devices when touching outside the drag handle area.
- **SC-005**: Users with `prefers-reduced-motion` enabled experience no motion animations during drag operations.
- **SC-006**: All existing tests continue to pass and the existing ReorderSetlist core event is reused (no core logic changes required).

## Assumptions

- The HTML5 Drag and Drop API (or a lightweight pointer-event-based approach) provides sufficient cross-browser support for both desktop and mobile. If the HTML5 API proves inadequate on mobile (which is common), a pointer-events-based approach using `pointerdown`/`pointermove`/`pointerup` will be used instead.
- The existing `ReorderSetlist` core event (which takes `entry_id` and `new_position`) can be reused as-is. The drag-and-drop is purely a shell/UI concern — the core domain logic does not need to change.
- Auto-scrolling during drag on long lists is a nice-to-have enhancement that can be deferred if it adds significant complexity.
- The routine edit page entry list uses the same `SetlistEntryRow` component or a similar pattern, so drag-and-drop applies there as well.

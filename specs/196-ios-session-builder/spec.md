# Feature Specification: iOS Session Builder

**Feature Branch**: `196-ios-session-builder`
**Created**: 2026-03-17
**Status**: Draft
**Input**: iOS: Session builder — construct practice setlists (#196)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build a setlist from library items (Priority: P1)

A musician navigates to "New Session" from the Practice tab. They see their full library as a scrollable list. They tap items to toggle them into (or out of) their setlist — selected items show an accent bar and check icon. A sticky bottom bar shows the item count, total time, and a "Start Session" button. When ready, they tap "Start Session" to begin.

**Why this priority**: This is the core purpose of the builder — without it, no sessions can be created on iOS.

**Independent Test**: Can be fully tested by tapping 1-3 items in the library list, verifying they show selected state (accent bar + check icon), confirming the bottom bar updates, and tapping "Start Session" to confirm the transition to the active session state.

**Acceptance Scenarios**:

1. **Given** the user is on the Practice tab, **When** they tap "New Session", **Then** the session builder appears showing the full library list with a sticky bottom bar.
2. **Given** the builder is open, **When** the user taps an unselected library item, **Then** it is added to the setlist (accent bar + check icon appears, bottom bar count updates).
3. **Given** the builder is open, **When** the user taps a selected library item, **Then** it is removed from the setlist (accent bar + check icon removed, bottom bar count updates).
4. **Given** the setlist has at least one item, **When** the user taps "Start Session", **Then** the app transitions to the active session state.
5. **Given** the setlist is empty, **When** the user views the "Start Session" button, **Then** it is disabled.

---

### User Story 2 - Customise setlist entries (Priority: P2)

Before starting a session, the musician opens the setlist (via the bottom bar on iPhone, or the right panel on iPad) to fine-tune entries: set a planned duration, add a per-entry intention ("focus on dynamics"), set a rep target, drag to reorder, or remove items. Entry details use progressive disclosure — tap an entry to expand its options.

**Why this priority**: Customisation makes sessions more intentional, but the builder is usable without it (P1 covers the basic add-and-start flow).

**Independent Test**: Can be tested by adding items, opening the setlist, then modifying duration/intention/rep target, dragging to reorder, and removing an entry — verifying each change is reflected in the setlist.

**Acceptance Scenarios**:

1. **Given** an entry is in the setlist, **When** the user taps it, **Then** it expands to show duration/intention/rep target fields (progressive disclosure).
2. **Given** an expanded entry, **When** the user sets a planned duration, **Then** the duration is displayed on the entry row.
3. **Given** an expanded entry, **When** the user types an intention, **Then** it is saved for that entry.
4. **Given** an expanded entry, **When** the user sets a rep target, **Then** the target is displayed on the entry row.
5. **Given** multiple entries in the setlist, **When** the user drags an entry via the drag handle, **Then** the entry changes position (SwiftUI `.onMove`).
6. **Given** an entry is in the setlist, **When** the user taps the remove (×) button, **Then** it is removed from the setlist and the library item returns to unselected state.

---

### User Story 3 - Set a session-level intention (Priority: P2)

The musician wants to set an overarching focus for the entire session — "work on sight-reading" or "prepare for recital". This appears at the top of the builder.

**Why this priority**: Enhances intentional practice but is optional metadata.

**Independent Test**: Can be tested by typing a session intention, starting the session, and verifying the intention carries through to the active session and summary views.

**Acceptance Scenarios**:

1. **Given** the builder is open, **When** the user types a session intention, **Then** it is displayed in the intention field.
2. **Given** a session intention is set, **When** the user starts the session, **Then** the intention is preserved through the session lifecycle.

---

### User Story 4 - Load a saved routine (Priority: P3)

A musician has previously saved a routine (e.g. "Morning Warm-up"). They want to load it into the builder to quickly populate their setlist without manually adding each item.

**Why this priority**: A convenience shortcut — users can always build manually (P1). Depends on routines being available in the ViewModel.

**Independent Test**: Can be tested by loading a routine with 2+ entries and verifying the setlist is populated with those entries.

**Acceptance Scenarios**:

1. **Given** the user has saved routines, **When** the builder is open, **Then** a "Saved Routines" section shows available routines.
2. **Given** a routine is listed, **When** the user taps "Load", **Then** the routine's entries populate the setlist.
3. **Given** the user has no saved routines, **When** the builder is open, **Then** the "Saved Routines" section is hidden.

---

### User Story 5 - Save current setlist as a routine (Priority: P3)

After assembling a good setlist, the musician wants to save it as a reusable routine for next time.

**Why this priority**: A convenience feature — sessions work fine without saving routines.

**Independent Test**: Can be tested by building a setlist, tapping "Save as Routine", entering a name, and verifying the routine is saved.

**Acceptance Scenarios**:

1. **Given** the setlist has entries, **When** the user taps "Save as Routine", **Then** a form appears for entering a routine name.
2. **Given** the form is open with a valid name, **When** the user taps "Save", **Then** the routine is saved and the form collapses.
3. **Given** the form is open with an empty name, **When** the user taps "Save", **Then** a validation error appears.
4. **Given** the setlist is empty, **When** the user looks for "Save as Routine", **Then** it is not visible.

---

### ~~User Story 6 - Resume or discard an active session~~ (DEFERRED to #197)

> **Removed from this feature.** The session builder is unreachable when a session is
> active — state-driven rendering ensures the Practice tab shows the active session
> view instead. Resume/discard UX will be handled in #197 (Active Session).

---

### Edge Cases

- What happens when the library is empty? The library list shows an empty state ("No library items — add some first") and the Start Session button is disabled.
- What happens when all library items are already in the setlist? All rows show selected state. The user can still start the session or deselect items.
- What happens when the user cancels building? The back link returns to the Practice session list.
- What happens when the user taps "Start Session" with no items? The button is disabled; nothing happens.
- What happens when the user searches and no results match? The list shows "No matching items" with the search term.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The app MUST display a session builder when the user navigates to "New Session".
- **FR-002**: The builder MUST show the user's library items as an item picker.
- **FR-003**: Users MUST be able to add library items to the setlist by tapping them.
- **FR-004**: Users MUST be able to remove items from the setlist.
- **FR-005**: Users MUST be able to reorder setlist entries via drag handles (SwiftUI `.onMove`).
- **FR-006**: Users MUST be able to set a planned duration per entry (minutes).
- **FR-007**: Users MUST be able to set an intention per entry (free text).
- **FR-008**: Users MUST be able to set a rep target per entry.
- **FR-009**: Users MUST be able to set a session-level intention.
- **FR-010**: The "Start Session" button MUST be disabled when the setlist is empty.
- **FR-011**: Tapping "Start Session" MUST transition to the active session state.
- **FR-012**: Users MUST be able to cancel building, returning to the session list.
- **FR-013**: The session builder MUST be unreachable when `session_status` is `Active` — state-driven rendering shows the active session view instead. (Resume/discard UX deferred to #197.)
- **FR-014**: Users MUST be able to load a saved routine into the setlist.
- **FR-015**: Users MUST be able to save the current setlist as a named routine.
- **FR-016**: Items in the setlist MUST remain visible in the library list but show a selected state (accent bar + check icon). Tapping toggles selection.
- **FR-017**: The builder MUST display total session time based on entry durations in the sticky bottom bar (iPhone) and setlist panel (iPad).
- **FR-021**: The builder MUST include a search bar to filter library items by name/composer.
- **FR-022**: Setlist entry details (duration, intention, reps) MUST use progressive disclosure — collapsed by default, tap to expand.

### Key Entities

- **Building Setlist**: An ordered collection of entries being assembled before starting a session. Has an optional session-level intention.
- **Setlist Entry**: A single item in the setlist with optional planned duration, intention, and rep target. References a library item.
- **Routine**: A saved, reusable setlist template with a name and ordered entries.

## Design *(include if feature has UI)*

### Pencil Design Frames

- **iOS / Session Builder v2 (iPhone)** — main tap-to-queue library list with sticky bottom bar
- **iOS / Session Builder v2 (Sheet Open)** — bottom sheet with setlist details open
- **iPad / Session Builder** — split-view layout (library left, setlist right)

### Existing Components Used

- `ButtonView` — primary ("Start Session")
- `TextFieldView` — session intention input, per-entry fields, search bar
- `TypeBadge` — piece/exercise badge on library and setlist items
- `EmptyStateView` — empty library messaging
- `ErrorBanner` — error display from ViewModel
- `BackLink` — "Back to Practice" navigation

### New Components Needed

- **LibraryQueueRow**: A tappable row showing a library item (title, subtitle, type badge) with toggle state. Unselected: + icon on right. Selected: accent left bar + check-circle icon on right. Tap toggles in/out of setlist.
- **SetlistEntryRow**: A compact row in the setlist showing title, duration/reps metadata, type badge, drag handle (left), and remove × button (right). Tappable to expand for editing duration/intention/reps (progressive disclosure).
- **StickyBottomBar**: Persistent bar at bottom of iPhone layout showing item count, total time, "Tap to edit setlist" hint, and "Start Session" button.
- **SetlistSheet**: Bottom sheet (iPhone) containing session intention field, setlist entries with drag handles, total time, "Load Routine" link, and full-width "Start Session" button.

### Wireframe / Layout Description

#### iPhone layout (tap-to-queue with bottom sheet)

The library **is** the builder. One unified scrollable view:

1. **Navigation**: Back link ("< Practice") and "New Session" heading
2. **Search bar**: Filter library items by name/composer
3. **Library list**: Full library as tappable rows. Selected items show accent left bar + check icon. Unselected items show + icon.
4. **Sticky bottom bar** (always visible): "{N} items · {M} min" count on left, "Tap to edit setlist" hint, "Start Session →" button on right.
5. **Bottom sheet** (on demand): Opens when user taps the count area of the bottom bar. Contains:
   - Drag handle
   - "Your Setlist" heading + "Load Routine" link
   - Session intention field
   - Setlist entries with drag handles, badges, × remove buttons
   - Total time
   - Full-width "Start Session →" button

Building a 3-item session = **3 taps + 1 tap**. Minimum friction.

#### iPad layout (split view)

`NavigationSplitView`-style layout, everything visible simultaneously:

- **Left column** (~420px): Back link, "Library" heading, search bar, library list with tap-to-queue rows (same visual language as iPhone)
- **Right column** (fill): "New Session" heading, "Load Routine" link, session intention field, setlist with drag-to-reorder entries, total time, "Start Session" button

No sheets needed on iPad — both panels are always visible.

### Responsive Behaviour

- **iPhone**: Single-column library list + sticky bottom bar + bottom sheet for setlist details.
- **iPad**: Side-by-side split view — library left, setlist right. Full use of screen real estate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can build a setlist and start a session in under 10 seconds (for a 3-item session — 3 taps to select + 1 tap to start).
- **SC-002**: All session builder interactions dispatch the correct events through the Crux core and produce the expected ViewModel updates.
- **SC-003**: The builder meets all 22 functional requirements on iOS.
- **SC-004**: The builder provides an optimised layout for both iPhone (bottom bar + sheet) and iPad (split view).

## Clarifications

### Session 2026-03-17

- Q: How should iOS handle navigation when the user leaves and returns mid-session? → A: State-driven rendering — the Practice tab reads `session_status` from the ViewModel and renders the appropriate screen (Idle → session list, Building → builder, Active → active session, Summary → summary). No NavigationStack push/pop for session flow.
- Q: How does the user know there's a live session when browsing other tabs? → A: The Practice tab icon changes colour or swaps to an active indicator (e.g. filled icon or accent-coloured dot) when `session_status` is `Active` or `Building`.

### iOS Navigation Architecture

The Practice tab uses **state-driven rendering** rather than NavigationStack for the session lifecycle. The tab reads `session_status` from the ViewModel and renders the matching screen:

| `session_status` | Practice tab shows |
|-----------------|-------------------|
| `Idle` | Session list (history + "New Session" CTA) |
| `Building` | Session builder |
| `Active` | Active session (placeholder until #197) |
| `Summary` | Session summary (placeholder until #198) |

This approach means:
- **No stale navigation stack** — the user always sees the screen matching the Crux state.
- **Builder is unreachable during active session** — the state machine makes it impossible to open the builder when `session_status` is `Active`. No resume/discard banner needed in the builder.
- **App backgrounding/foregrounding** works automatically — Crux state persists in memory, active sessions also in UserDefaults for crash recovery.
- **No "rejoin" flow needed** — tapping the Practice tab always shows the right screen.
- **Tab bar indicator** — when `session_status` is `Active` or `Building`, the Practice tab shows a visual indicator (accent-coloured dot or filled icon) so the user knows they have a live session from any tab.

### Functional Requirements (additions)

- **FR-018**: The Practice tab MUST render the screen matching the current `session_status` (state-driven routing).
- **FR-019**: When `session_status` is `Active` or `Building`, the Practice tab icon MUST show a visual indicator (accent dot or filled variant).
- **FR-020**: Navigating away from the Practice tab and back MUST return the user to the correct session lifecycle screen.

## Assumptions

- The Crux core already handles all session builder logic — the iOS shell only needs to dispatch events and render the ViewModel.
- Library items and routines are already fetched and available in the ViewModel by the time the user opens the builder.
- Drag-to-reorder (SwiftUI `.onMove`) is the primary reordering mechanism. No move up/down buttons.
- The active session view (#197) does not need to exist for this feature — "Start Session" just needs to dispatch the event correctly. The app can show a placeholder until #197 is implemented.
- The web session builder will be updated separately (#229) to match this same UX pattern.

## Dependencies

- #194 (iOS Design system) — completed
- #195 (iOS Library) — completed
- Crux core session builder events — already implemented

# Feature Specification: Web — Adopt iOS Layout Patterns

**Feature Branch**: `219-web-ios-layout-parity`
**Created**: 2026-03-21
**Status**: Draft
**Input**: Update web UI to match iOS layout patterns: compact library list rows, tap-to-queue session builder, and split-view layouts for desktop. Covers library views, session builder, and consistent component vocabulary across platforms.

## User Scenarios & Testing

### User Story 1 — Split-View Library on Desktop (Priority: P1)

On desktop (≥768px), the library adopts a sidebar + detail pane layout. The left column shows a scrollable list of items with compact rows (title, subtitle, type badge). Clicking an item loads its detail view in the right pane without a full-page navigation. On mobile (<768px), the existing stacked full-page navigation is unchanged.

**Why this priority**: The split-view is the single biggest UX improvement — it eliminates constant back-and-forth navigation on desktop and brings the web in line with the iPad experience. Every library user benefits immediately.

**Independent Test**: Navigate to the library on a desktop browser. Click an item in the sidebar — detail loads in the right pane. Click another item — detail updates without losing scroll position. Resize to mobile — layout switches to stacked navigation.

**Acceptance Scenarios**:

1. **Given** a desktop viewport (≥768px), **When** the user opens the library, **Then** a sidebar list is shown on the left and the first item's detail is shown in the right pane (auto-selected).
2. **Given** the split-view is active, **When** the user clicks an item in the sidebar, **Then** the detail pane updates to show that item's details without a page reload.
3. **Given** the split-view is active, **When** the user selects an item, **Then** the selected item is visually highlighted in the sidebar.
4. **Given** a mobile viewport (<768px), **When** the user opens the library, **Then** the existing full-page stacked layout is shown (no sidebar).
5. **Given** the split-view is active, **When** the user clicks "Add Item" or "Edit", **Then** the form loads in the detail pane (not a separate page).

---

### User Story 2 — Compact Library List Rows (Priority: P2)

Library items display as compact rows instead of glassmorphism cards. Each row shows: title, subtitle (composer/description), key/tempo metadata, and type badge. Rows use subtle dividers instead of card borders. The row design matches the iOS `LibraryItemRow` component.

**Why this priority**: The compact rows reduce visual clutter and information density, making the library scannable. This works hand-in-hand with the split-view (P1) since the sidebar needs compact rows to fit enough items.

**Independent Test**: Open the library — items display as compact rows with title, subtitle, metadata, and badge. No glassmorphism card wrappers on individual items.

**Acceptance Scenarios**:

1. **Given** the library has items, **When** the user views the list, **Then** each item is displayed as a compact row with title, subtitle, metadata line, and type badge.
2. **Given** a library row, **When** the user hovers over it, **Then** a subtle hover state is shown (background highlight).
3. **Given** items of both types, **When** viewing the list, **Then** Piece and Exercise badges are correctly displayed on each row.
4. **Given** an item with no subtitle or metadata, **When** viewing the row, **Then** the row gracefully omits the missing fields without blank space.

---

### User Story 3 — Tap-to-Queue Session Builder (Priority: P3)

The session builder adopts the iOS tap-to-queue pattern. On desktop, a split-view shows the library list on the left and the setlist panel on the right. On mobile, the library list fills the screen with a sticky bottom bar showing the item count and "Start Session" button. Tapping a library item toggles it in/out of the setlist (selected state: accent left bar + check icon).

**Why this priority**: The current session builder has a cluttered two-section layout. The tap-to-queue pattern reduces friction — building a 3-item session takes 3 taps + 1 tap to start. This aligns with #229 and matches the iOS UX from #196.

**Independent Test**: Navigate to the session builder. Tap three library items — each gets a selected indicator. The setlist panel (desktop) or bottom bar (mobile) updates to show "3 items". Tap "Start Session".

**Acceptance Scenarios**:

1. **Given** the session builder on desktop, **When** the user opens it, **Then** a split-view shows the library on the left and the setlist on the right.
2. **Given** an unselected library item, **When** the user clicks it, **Then** the item is added to the setlist and the row shows a selected state (accent left bar + check icon).
3. **Given** a selected library item, **When** the user clicks it again, **Then** the item is removed from the setlist and returns to unselected state.
4. **Given** the session builder on mobile, **When** items are selected, **Then** a sticky bottom bar shows the item count, total duration, and a "Start Session" button.
5. **Given** the mobile session builder with items selected, **When** the user taps the bottom bar summary area, **Then** a slide-up sheet opens showing the full setlist with reorder, entry editing, and session intention.
5. **Given** the setlist on desktop, **When** items are in the setlist, **Then** the user can drag to reorder entries using drag handles.
6. **Given** a setlist entry, **When** the user clicks to expand it, **Then** duration, intention, and rep fields are revealed (progressive disclosure).

---

### User Story 4 — Session Builder Search & Filter (Priority: P4)

The library list within the session builder supports search (filter by title/composer) and type filter tabs (All / Pieces / Exercises) to help users quickly find items to add.

**Why this priority**: Important for users with large libraries but the builder is usable without it. Enhances discoverability.

**Independent Test**: Open the session builder. Type in the search field — list filters to matching items. Tap a filter tab — list filters by type.

**Acceptance Scenarios**:

1. **Given** the session builder library list, **When** the user types in the search field, **Then** the list filters to items matching the query by title or subtitle.
2. **Given** the session builder, **When** the user taps a type filter tab, **Then** only items of that type are shown.
3. **Given** an active search filter, **When** the user clears the search, **Then** all items are shown again.

---

### Edge Cases

- What happens when the viewport is resized from desktop to mobile while in split-view? The layout should switch to stacked navigation, preserving the currently viewed item.
- What happens when the library is empty in split-view? The detail pane shows an empty state message with a CTA to add items.
- What happens when the user has 0 items selected in the session builder? The "Start Session" button is disabled.
- What happens when the user navigates directly to a detail URL on desktop? The split-view loads with that item pre-selected in the sidebar.

## Requirements

### Functional Requirements

- **FR-001**: System MUST display a split-view layout (sidebar list + detail pane) on viewports ≥768px for the library. When no item is specified in the URL, the first item in the list MUST be auto-selected.
- **FR-002**: System MUST display compact row layout for library items (title, subtitle, metadata, type badge) instead of card-based layout.
- **FR-003**: System MUST highlight the currently selected item in the sidebar with a visual indicator.
- **FR-004**: System MUST load item detail, add, and edit views within the detail pane on desktop (no full-page navigation). The browser URL MUST update to reflect the selected item (e.g., `/library/{item-id}`) to support deep-linking and browser back/forward navigation.
- **FR-005**: System MUST fall back to stacked full-page navigation on viewports <768px.
- **FR-006**: System MUST display the session builder as a split-view on desktop: library list (left) + setlist panel (right).
- **FR-007**: System MUST toggle library items in/out of the setlist on click, with a selected state indicator (accent left bar + check icon).
- **FR-008**: System MUST display a sticky bottom bar on mobile session builder with item count, total duration, and "Start Session" button. Tapping the summary area of the bottom bar MUST open a slide-up sheet showing the full setlist (reorder, entry details, session intention).
- **FR-009**: System MUST support drag-to-reorder of setlist entries via drag handles.
- **FR-010**: System MUST support progressive disclosure of entry details (duration, intention, reps) on click/expand.
- **FR-011**: System MUST support search filtering of library items by title or subtitle in the session builder.
- **FR-012**: System MUST support type filter tabs (All / Pieces / Exercises) in the session builder.
- **FR-013**: System MUST disable the "Start Session" button when the setlist is empty.
- **FR-014**: System MUST preserve the existing design token system (colours, typography, spacing) — this is a layout and component update, not a visual redesign.

## Design

### Existing Components Used

- `PageHeading` — library page title (used in mobile view header)
- `TypeBadge` — type pills on library rows
- `TypeTabs` — filter tabs (All / Pieces / Exercises)
- `Button` — actions (Start Session, Add Item, etc.)
- `TextField` — search input, entry fields
- `BackLink` — mobile back navigation
- `Card` — detail pane sections, setlist panel container
- `StatCard` — item detail metadata cards
- `DragHandle` / `DropIndicator` — setlist reorder
- `SetlistEntryRow` — entry rows in setlist (updated for progressive disclosure)
- `Toast` — feedback notifications
- `SkeletonLine` / `SkeletonBlock` — loading states

### New Components Needed

- **LibraryListRow**: Compact row for library items — title, subtitle, metadata line, type badge. Replaces `LibraryItemCard` in list contexts. Supports hover highlight and selected state (accent left bar for session builder context).
- **SplitViewLayout**: Responsive container that renders sidebar + detail pane on desktop (≥768px) and stacked navigation on mobile (<768px). Reusable for library, session builder, and future views (routines, analytics).
- **StickyBottomBar**: Mobile session builder bar — item count, total duration, "Start Session" button. Fixed to viewport bottom.

### Wireframe / Layout Description

Reference Pencil frames in `design/intrada.pen`:
- "iOS / Session Builder v2 (iPhone)" — tap-to-queue with sticky bottom bar
- "iOS / Session Builder v2 (Sheet Open)" — setlist detail sheet
- "iPad / Session Builder" — split-view layout (library left, setlist right)

Desktop library and session builder follow the iPad split-view pattern. New Pencil frames for web desktop and mobile will be added during the design phase.

### Responsive Behaviour

- **Mobile (<768px)**: Stacked full-page navigation (unchanged for library). Session builder shows full-screen library list with sticky bottom bar.
- **Desktop (≥768px)**: Split-view with sidebar list (~320px) and detail pane (remaining width). Session builder shows library left, setlist right.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can view an item's details from the library without leaving the list view (zero back-button navigations on desktop).
- **SC-002**: Users can build a 3-item session in 4 interactions or fewer (3 taps to select + 1 to start).
- **SC-003**: The library list displays at least 8 items visible above the fold on desktop (compact rows vs current 4-6 cards).
- **SC-004**: All existing E2E tests continue to pass — no regression in mobile library or session functionality.
- **SC-005**: Desktop and mobile layouts are visually consistent with the iOS app's library and session builder screens.

## Clarifications

### Session 2026-03-21

- Q: What should the detail pane show when no item is selected on desktop? → A: Auto-select the first item in the list.
- Q: Should the browser URL update when selecting an item in the sidebar? → A: Yes, URL updates to include the item ID (e.g., `/library/item-123`) — enables deep-linking and back/forward.
- Q: How does the user access the full setlist on mobile? → A: Tapping the bottom bar opens a slide-up sheet/panel with the setlist (matching iOS bottom sheet pattern).

## Assumptions

- The breakpoint for split-view is 768px (matches `md:` Tailwind breakpoint).
- The session builder search and filter operate on the client-side ViewModel data (no new API endpoints needed).
- The existing drag-and-drop implementation in `SetlistBuilder` can be adapted for the new layout.
- Routine loading/saving UI stays in the setlist panel (not affected by this layout change).
- This is a layout and component change — the design token system (colours, fonts, spacing) is preserved.

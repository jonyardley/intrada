# Research: iOS Session Builder

**Feature**: #196 — iOS Session Builder
**Date**: 2026-03-18

## R1: Crux Core Session Builder Events

**Decision**: Use the existing Crux core session builder events as-is. No core changes needed.

**Rationale**: The core already provides all required events:
- `StartBuilding`, `CancelBuilding` — lifecycle management
- `AddToSetlist(item_id)`, `RemoveFromSetlist(entry_id)` — tap-to-queue toggle
- `ReorderSetlist(entry_id, new_position)` — drag-to-reorder
- `SetEntryDuration`, `SetEntryIntention`, `SetRepTarget` — entry customisation
- `SetSessionIntention` — session-level intention
- `StartSession(now)` — transition to active

The ViewModel exposes `session_status`, `building_setlist` (entries, item_count, session_intention), and `items` (full library). All Swift types are auto-generated.

**Alternatives considered**: None — modifying the core would violate architecture integrity.

## R2: iPhone Layout — Bottom Sheet vs Modal

**Decision**: Use SwiftUI `.sheet(isPresented:)` with a detent-based presentation for the setlist editor on iPhone.

**Rationale**: iOS 16+ supports `PresentationDetent` (`.medium`, `.large`), which provides the native bottom sheet UX users expect. The sheet opens when the user taps the count area of the sticky bottom bar. This avoids building a custom sheet component and gives us native gesture handling (swipe to dismiss, detent snapping).

**Alternatives considered**:
- Custom overlay sheet: More control over appearance but significant implementation effort for gesture handling, backdrop dimming, and animation. Not worth it for v1.
- Full-screen modal: Too heavy — the user just wants a quick glance at and edit of the setlist.

## R3: iPad Layout — Split View Approach

**Decision**: Use `horizontalSizeClass` environment value to switch between iPhone (single column + sheet) and iPad (HStack with two columns) layout within a single `SessionBuilderView`.

**Rationale**: The existing library uses `UIDevice.current.userInterfaceIdiom == .pad` for iPad detection (discovered during #222 fix — `horizontalSizeClass` reports `.compact` in NavigationSplitView sidebars). However, the session builder is not inside a NavigationSplitView sidebar — it's the full content area. So `horizontalSizeClass` works correctly here and is the more SwiftUI-idiomatic approach.

**Alternatives considered**:
- Separate iPhone/iPad view files: Code duplication for what's essentially the same data flow with different layout. Rejected.
- `UIDevice.current.userInterfaceIdiom`: Works but is less responsive to multitasking window sizes. Reserved for cases where `horizontalSizeClass` fails (sidebar context).

## R4: Tap-to-Toggle vs Add/Remove Separate Actions

**Decision**: Single tap toggles an item in/out of the setlist. The library list stays visible with selected state indicators.

**Rationale**: This is the core UX innovation. The old design removed items from the library picker when added to the setlist (FR-016 old version). The new design keeps all items visible and toggles their visual state. This means:
- Adding: Dispatch `Event.session(.addToSetlist(itemId: item.id))`
- Removing: Find the entry ID for the item, dispatch `Event.session(.removeFromSetlist(entryId: entry.id))`
- Selection state: Derive from ViewModel — check if any entry in `building_setlist.entries` has `item_id == item.id`

**Alternatives considered**: None — this is the spec'd UX.

## R5: Drag-to-Reorder Implementation

**Decision**: Use SwiftUI `List` with `.onMove` modifier for the setlist entries in the bottom sheet / right panel.

**Rationale**: `.onMove` provides native drag-to-reorder with proper haptic feedback and animation. When the user moves an entry, dispatch `Event.session(.reorderSetlist(entryId: entry.id, newPosition: UInt64(newIndex)))`. The Crux core handles the state update and emits a new ViewModel.

**Alternatives considered**:
- `ForEach` with custom `DragGesture`: More control but significantly more code for gesture handling, animation, and edge cases. Not worth it for v1.
- `LazyVStack` with `onDrag/onDrop`: Requires more boilerplate and doesn't provide the same polish as `.onMove`.

## R6: Search/Filter Implementation

**Decision**: Client-side filtering on the `items` list from the ViewModel using a `@State` search string.

**Rationale**: The ViewModel already provides the full library item list. Filtering is a pure shell concern — filter the `items` array by title/composer matching the search string. No Crux event needed. This keeps the core simple and avoids adding a filter event for what is purely a UI concern.

**Alternatives considered**:
- Server-side search via Crux event: Overkill — the library fits in memory and doesn't need server round-trips for filtering.

## R7: DateTime Handling for StartSession

**Decision**: Convert `Date()` to ISO 8601 string when dispatching `StartSession`.

**Rationale**: The Crux core's `StartSession` event expects a `DateTime<Utc>` serialized as an ISO 8601 string (via BCS). In Swift:
```swift
let now = ISO8601DateFormatter().string(from: Date())
core.processEvent(.session(.startSession(now: now)))
```

This matches how the web shell passes timestamps.

**Alternatives considered**: None — this is the standard pattern.

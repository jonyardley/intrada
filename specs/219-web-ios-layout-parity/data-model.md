# Data Model: Web — Adopt iOS Layout Patterns

**Date**: 2026-03-21
**Feature**: `219-web-ios-layout-parity`

## No Data Model Changes

This feature is a pure presentation-layer update. No new entities, fields, relationships, or API endpoints are introduced.

### Existing Entities Used (unchanged)

| Entity | Usage in this feature |
|--------|----------------------|
| `LibraryItemView` | Rendered in `LibraryListRow` (compact rows) instead of `LibraryItemCard` |
| `SetlistEntryView` | Rendered in setlist panel with progressive disclosure |
| `SessionStatus` | Drives session builder state (Idle/Building/Active/Summary) |
| `ViewModel` | Provides `items`, `session_entries`, `session_status` to views |

### Existing Events Used (unchanged)

| Event | Usage |
|-------|-------|
| `AddToSetlist(item_id)` | Tap-to-queue: add item to setlist |
| `RemoveFromSetlist(entry_id)` | Tap-to-dequeue: remove item from setlist |
| `MoveSetlistEntry { from, to }` | Drag-to-reorder in setlist |
| `StartBuilding` | Enter session builder |
| `StartSession` | Start the session from builder |

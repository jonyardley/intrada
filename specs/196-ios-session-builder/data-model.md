# Data Model: iOS Session Builder

**Feature**: #196 — iOS Session Builder
**Date**: 2026-03-18

## Overview

No new data entities are defined by this feature. The iOS session builder is a pure shell that renders existing Crux ViewModel types and dispatches existing Crux events. All types are auto-generated from Rust via the typegen pipeline.

## Entities (from Crux ViewModel — read only)

### SessionStatusView

```
enum SessionStatusView {
  idle        — no session activity
  building    — setlist is being assembled
  active      — session in progress (deferred to #197)
  summary     — session complete, reviewing (deferred to #198)
}
```

State-driven routing: Practice tab renders the view matching the current status.

### BuildingSetlistView

```
struct BuildingSetlistView {
  entries: [SetlistEntryView]     — ordered list of setlist items
  itemCount: UInt64               — count of entries
  sessionIntention: String?       — optional session-level focus
}
```

Available when `session_status == .building`. Nil otherwise.

### SetlistEntryView

```
struct SetlistEntryView {
  id: String                      — unique entry ID (ULID)
  itemId: String                  — references a LibraryItemView
  itemTitle: String               — display title
  itemType: ItemKind              — .piece or .exercise
  position: UInt64                — order in setlist (0-indexed)
  durationDisplay: String         — formatted active duration
  status: EntryStatus             — .pending, .active, .completed, .skipped
  intention: String?              — per-entry focus
  repTarget: UInt8?               — target number of repetitions
  repCount: UInt8?                — current rep count (active phase)
  repTargetReached: Bool?         — whether target met
  plannedDurationSecs: UInt32?    — planned duration in seconds
  plannedDurationDisplay: String? — formatted planned duration
  achievedTempo: UInt16?          — from session history
}
```

### LibraryItemView

```
struct LibraryItemView {
  id: String                      — unique item ID (ULID)
  title: String                   — item name
  subtitle: String                — composer or description
  itemType: ItemKind              — .piece or .exercise
  key: String?                    — musical key
  targetTempo: UInt16?            — target BPM
  tags: [String]                  — user tags
  latestScore: UInt8?             — most recent score
  sessionCount: UInt64            — times practised
}
```

Used for the library list in the builder. Selection state is derived by checking if any entry in `building_setlist.entries` has a matching `item_id`.

## Shell-Local State (SwiftUI @State — not in Crux)

| State | Type | Purpose |
|-------|------|---------|
| `searchText` | `String` | Filter library items by title/composer |
| `isSheetPresented` | `Bool` | Controls setlist bottom sheet visibility (iPhone) |
| `expandedEntryId` | `String?` | Which entry is expanded for editing (progressive disclosure) |

These are ephemeral UI concerns that don't belong in the Crux model.

## Events Dispatched (from iOS → Crux)

| User Action | Crux Event | Notes |
|-------------|------------|-------|
| Open builder | `session(.startBuilding)` | Transitions to Building state |
| Tap library item (unselected) | `session(.addToSetlist(itemId:))` | Adds to setlist |
| Tap library item (selected) | `session(.removeFromSetlist(entryId:))` | Removes from setlist (find entry by itemId) |
| Drag to reorder | `session(.reorderSetlist(entryId:, newPosition:))` | Via `.onMove` |
| Set entry duration | `session(.setEntryDuration(entryId:, durationSecs:))` | Progressive disclosure field |
| Set entry intention | `session(.setEntryIntention(entryId:, intention:))` | Progressive disclosure field |
| Set entry rep target | `session(.setRepTarget(entryId:, target:))` | Progressive disclosure field |
| Set session intention | `session(.setSessionIntention(intention:))` | In sheet/right panel |
| Tap Start Session | `session(.startSession(now:))` | ISO 8601 timestamp |
| Tap Back/Cancel | `session(.cancelBuilding)` | Returns to Idle |

## Relationships

```
ViewModel
  ├── sessionStatus: SessionStatusView (drives which view renders)
  ├── buildingSetlist: BuildingSetlistView? (nil when not building)
  │     ├── entries: [SetlistEntryView] (ordered, references items by itemId)
  │     └── sessionIntention: String?
  └── items: [LibraryItemView] (full library, always available)
```

No new API contracts needed — the iOS shell uses the same Crux HTTP effects that the web shell uses.

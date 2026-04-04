# Data Model: iOS Routines

## Existing Types (SharedTypes.swift)

### RoutineView

| Field | Type | Usage |
|-------|------|-------|
| `id` | `String` | Unique ID for delete/edit/load |
| `name` | `String` | Display name |
| `entryCount` | `UInt64` | Badge count |
| `entries` | `[RoutineEntryView]` | Ordered item list |

### RoutineEntryView

| Field | Type | Usage |
|-------|------|-------|
| `id` | `String` | Entry ID |
| `itemId` | `String` | Library item reference |
| `itemTitle` | `String` | Cached display title |
| `itemType` | `ItemKind` | TypeBadge rendering |
| `position` | `UInt64` | Sort order |

### RoutineEntry (for events)

| Field | Type | Usage |
|-------|------|-------|
| `id` | `String` | Entry ID |
| `itemId` | `String` | Library item ref |
| `itemTitle` | `String` | Cached title |
| `itemType` | `ItemKind` | Cached type |
| `position` | `UInt64` | Position |

### Events

| Event | Parameters | When |
|-------|-----------|------|
| `.deleteRoutine(id:)` | `String` | Swipe-to-delete confirmed |
| `.updateRoutine(id:, name:, entries:)` | `String`, `String`, `[RoutineEntry]` | Edit saved |
| `.loadRoutineIntoSetlist(routineId:)` | `String` | Load tapped in builder |
| `.saveBuildingAsRoutine(name:)` | `String` | Save from builder |
| `.saveSummaryAsRoutine(name:)` | `String` | Save from summary |

### Validation

- Routine name: non-empty, ≤200 characters, not whitespace-only
- Routine entries: at least 1 entry required for save

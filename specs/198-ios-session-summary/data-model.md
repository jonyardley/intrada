# Data Model: iOS Session Summary & History

## Overview

Shell-only — no new data types. All types exist in auto-generated `SharedTypes.swift`.

## Existing Types

### SummaryView (post-session review)

| Field | Type | Usage |
|-------|------|-------|
| `totalDurationDisplay` | `String` | "23 min 15 sec" header stat |
| `completionStatus` | `CompletionStatus` | Completed or EndedEarly badge |
| `notes` | `String?` | Session-level notes (editable) |
| `entries` | `[SetlistEntryView]` | Per-item results list |
| `sessionIntention` | `String?` | Session intention (display only) |

### PracticeSessionView (saved session in history)

| Field | Type | Usage |
|-------|------|-------|
| `id` | `String` | Unique ID for delete |
| `startedAt` | `String` | Date grouping + display |
| `finishedAt` | `String` | Duration calculation |
| `totalDurationDisplay` | `String` | Card stat |
| `completionStatus` | `CompletionStatus` | Badge in list |
| `notes` | `String?` | Shown in detail |
| `entries` | `[SetlistEntryView]` | Detail view list |
| `sessionIntention` | `String?` | Card subtitle |

### SetlistEntryView (per-item result)

| Field | Type | Summary Usage |
|-------|------|---------------|
| `itemTitle` | `String` | Entry title |
| `itemType` | `ItemKind` | TypeBadge |
| `status` | `EntryStatus` | Status icon (completed/skipped/not attempted) |
| `durationDisplay` | `String` | Time spent |
| `score` | `UInt8?` | ScoreSelectorView (editable in summary) |
| `achievedTempo` | `UInt16?` | BPM badge |
| `notes` | `String?` | Notes text (editable in summary) |
| `repTarget` | `UInt8?` | Rep badge |
| `repCount` | `UInt8?` | Rep count display |
| `repTargetReached` | `Bool?` | Success/incomplete indicator |
| `intention` | `String?` | Entry intention display |

### CompletionStatus

| Variant | Display |
|---------|---------|
| `.completed` | Green "Completed" text |
| `.endedEarly` | Warm accent "Ended Early" badge |

### Events (dispatched by shell)

| Event | When |
|-------|------|
| `.updateEntryScore(entryId:, score:)` | Score dot tapped in summary |
| `.updateEntryTempo(entryId:, tempo:)` | Tempo edited in summary |
| `.updateEntryNotes(entryId:, notes:)` | Notes edited in summary |
| `.updateSessionNotes(notes:)` | Session notes edited |
| `.saveSession(now:)` | Save button tapped |
| `.discardSession` | Discard confirmed |
| `.deleteSession(id:)` | History delete confirmed |
| `.startBuilding` | "New Session" tapped from history |

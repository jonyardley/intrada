# Data Model: iOS Active Session

## Overview

This feature is **shell-only** — no new data types, API endpoints, or core changes. All domain types already exist in the auto-generated `SharedTypes.swift`. This document maps existing types to their usage in the active session views.

## Existing Types (from SharedTypes.swift)

### ActiveSessionView (ViewModel)

| Field | Type | Usage |
|-------|------|-------|
| `currentItemTitle` | `String` | Displayed as main title in focus area |
| `currentItemType` | `ItemKind` | TypeBadge rendering |
| `currentPosition` | `UInt64` | "ITEM X OF Y" label |
| `totalItems` | `UInt64` | "ITEM X OF Y" label |
| `startedAt` | `String` | Session elapsed time calculation |
| `entries` | `[SetlistEntryView]` | iPad sidebar setlist, entry status tracking |
| `sessionIntention` | `String?` | iPad sidebar, shown if present |
| `currentRepTarget` | `UInt8?` | Rep counter visibility and target |
| `currentRepCount` | `UInt8?` | Rep counter current value |
| `currentRepTargetReached` | `Bool?` | Celebration state trigger |
| `currentRepHistory` | `[RepAction]?` | Not displayed in MVP (future history view) |
| `currentPlannedDurationSecs` | `UInt32?` | Progress ring mode: ring if Some, elapsed-only if None |
| `nextItemTitle` | `String?` | Transition prompt "Up Next" preview |

### SessionEvent (Events dispatched by shell)

| Event | Parameters | When |
|-------|-----------|------|
| `.nextItem(now:)` | `String` (ISO8601) | User advances to next item |
| `.skipItem(now:)` | `String` (ISO8601) | User skips current item |
| `.finishSession(now:)` | `String` (ISO8601) | Last item completed |
| `.endSessionEarly(now:)` | `String` (ISO8601) | User ends before all items |
| `.abandonSession` | — | User discards session |
| `.repGotIt` | — | Rep success |
| `.repMissed` | — | Rep miss |
| `.updateEntryScore(entryId:score:)` | `String`, `UInt8?` | Score 1–5 on transition |
| `.updateEntryTempo(entryId:tempo:)` | `String`, `UInt16?` | BPM on transition |
| `.updateEntryNotes(entryId:notes:)` | `String`, `String?` | Notes on transition |

### Shell-Local State (not in ViewModel)

| State | Type | Scope | Purpose |
|-------|------|-------|---------|
| `elapsedSeconds` | `Int` | Per-item, resets on advance | Timer display + ring progress |
| `showTransitionPrompt` | `Bool` | Per-item | Bottom sheet visibility |
| `showPauseOverlay` | `Bool` | Per-session | Pause modal visibility |
| `showEndEarlyConfirmation` | `Bool` | Per-action | Confirmation dialog |
| `showAbandonConfirmation` | `Bool` | Per-action | Confirmation dialog |
| `pendingScore` | `UInt8?` | Per-transition | Score before dispatch |
| `pendingTempo` | `String` | Per-transition | Tempo text before parsing |
| `pendingNotes` | `String` | Per-transition | Notes before dispatch |
| `isPaused` | `Bool` | Per-session | Pauses timer publisher |

## State Transitions

```text
.building ──StartSession──▶ .active
                              │
                    ┌─────────┼──────────┐
                    │         │          │
              NextItem   FinishSession  EndSessionEarly
              (loops)        │          │
                    │         ▼          ▼
                    │      .summary   .summary
                    │
              AbandonSession
                    │
                    ▼
                  .idle
```

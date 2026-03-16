# Data Model: iOS Library

**Feature**: 001-ios-library | **Date**: 2026-03-16

## Overview

The iOS shell renders existing Crux ViewModel types — no new domain types are needed.
All types below are **auto-generated** from Rust via facet typegen and exist in
`ios/Intrada/Generated/SharedTypes/SharedTypes.swift`.

This document maps the generated types to their UI usage in the library feature.

## Generated Types Used

### LibraryItemView (read from ViewModel)

| Field | Type | UI Usage |
|-------|------|----------|
| `id` | `String` | Navigation identifier, list selection |
| `item_type` | `ItemKind` (.piece / .exercise) | TypeBadge colour, filter matching |
| `title` | `String` | List row primary text, detail heading |
| `subtitle` | `String?` | Composer name (list row secondary text, detail field) |
| `key` | `String?` | Detail field, list row metadata |
| `tempo` | `String?` | Pre-formatted display string (e.g. "Allegro (72 BPM)") |
| `notes` | `String?` | Detail view notes section |
| `tags` | `[String]` | Tag pills on list row and detail view |
| `created_at` | `String` | Detail view timestamp (RFC3339) |
| `updated_at` | `String` | Detail view timestamp (RFC3339) |
| `practice` | `ItemPracticeSummary?` | Practice summary card on detail view |
| `latest_achieved_tempo` | `UInt16?` | Combined tempo display on list row |

### ItemPracticeSummary (nested in LibraryItemView)

| Field | Type | UI Usage |
|-------|------|----------|
| `session_count` | `UInt32` | StatCard value |
| `total_minutes` | `UInt32` | StatCard value |
| `latest_score` | `UInt8?` | StatCard value (confidence 1-5) |
| `score_history` | `[ScoreHistoryEntry]` | ScoreHistoryList component |
| `tempo_history` | `[TempoHistoryEntry]` | Tempo progress display |

### ScoreHistoryEntry

| Field | Type | UI Usage |
|-------|------|----------|
| `date` | `String` | History list date label |
| `score` | `UInt8` | History list score badge |

### TempoHistoryEntry

| Field | Type | UI Usage |
|-------|------|----------|
| `date` | `String` | Progress list date label |
| `tempo` | `UInt16` | Progress list tempo value |

### ItemKind (enum)

| Variant | UI Usage |
|---------|----------|
| `.piece` | Blue/indigo badge, composer required in forms |
| `.exercise` | Gold badge, composer optional in forms |

## Events Dispatched

### ItemEvent (via Event.item)

| Event | When dispatched | Shell data needed |
|-------|----------------|-------------------|
| `.add(CreateItem)` | User submits add form | Form field values → CreateItem struct |
| `.update(id, UpdateItem)` | User submits edit form | Item ID + changed fields → UpdateItem struct |
| `.delete(id)` | User confirms deletion | Item ID |

### ListQuery (via Event.setQuery)

| Field | Type | When set |
|-------|------|----------|
| `text` | `String?` | Search bar text changes |
| `item_type` | `ItemKind?` | Type tab selection (nil = All) |

### CreateItem (form → event)

| Field | Type | Form source |
|-------|------|-------------|
| `title` | `String` | Title TextField |
| `kind` | `ItemKind` | TypeTabs selection |
| `composer` | `String?` | Composer AutocompleteField |
| `key` | `String?` | Key TextField |
| `tempo` | `Tempo?` | Marking TextField + BPM TextField |
| `notes` | `String?` | Notes TextArea |
| `tags` | `[String]` | TagInput chips |

### UpdateItem (form → event)

All fields use `Option<Option<T>>` semantics (auto-generated as nested optionals):
- `nil` = don't change this field
- `.some(nil)` = clear this field
- `.some(.some(value))` = set to value

| Field | Type |
|-------|------|
| `title` | `String??` |
| `composer` | `String??` |
| `key` | `String??` |
| `tempo` | `Tempo??` |
| `notes` | `String??` |
| `tags` | `[String]??` |

## Shell-Local State (NOT in Crux)

These are ephemeral UI states managed by SwiftUI `@State`:

| State | Type | View | Purpose |
|-------|------|------|---------|
| `selectedItemId` | `String?` | LibraryView | NavigationSplitView selection |
| `searchText` | `String` | LibraryListContent | Search bar binding |
| `selectedTab` | `FilterTab` | LibraryListContent | All/Pieces/Exercises toggle |
| `title` | `String` | AddItemView/EditItemView | Form field |
| `composer` | `String` | AddItemView/EditItemView | Form field |
| `key` | `String` | AddItemView/EditItemView | Form field |
| `tempoMarking` | `String` | AddItemView/EditItemView | Form field |
| `bpm` | `String` | AddItemView/EditItemView | Form field |
| `notes` | `String` | AddItemView/EditItemView | Form field |
| `tags` | `[String]` | AddItemView/EditItemView | Form field |
| `itemKind` | `ItemKind` | AddItemView | Piece/Exercise selection |
| `errors` | `[String: String]` | AddItemView/EditItemView | Validation error map |
| `isSubmitting` | `Bool` | AddItemView/EditItemView | Submit button loading state |
| `showDeleteConfirmation` | `Bool` | ItemDetailView | Confirmation dialog |

## Helper Types (Swift-only, not generated)

### FilterTab (local enum)

```swift
enum FilterTab: CaseIterable {
    case all, pieces, exercises

    var itemKind: ItemKind? {
        switch self {
        case .all: nil
        case .pieces: .piece
        case .exercises: .exercise
        }
    }
}
```

### LibraryFormValidator (validation mirror)

```swift
struct LibraryFormValidator {
    static let maxTitle = 500
    static let maxComposer = 200
    static let maxNotes = 5000
    static let maxTag = 100
    static let maxTempoMarking = 100
    static let minBpm = 1
    static let maxBpm = 400

    static func validate(kind: ItemKind, title: String, composer: String, ...) -> [String: String]
}
```

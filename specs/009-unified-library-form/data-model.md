# Data Model: Unified Library Item Form

**Feature**: 009-unified-library-form
**Date**: 2026-02-14

## Overview

This feature introduces **no data model changes**. The existing `Piece` and `Exercise` domain entities, their creation/update types, and the `ViewModel`/`LibraryItemView` structure remain completely unchanged. The feature is purely a UI/UX change in the web shell.

## Existing Entities (Unchanged)

### Piece (intrada-core)

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| id | String (ULID) | Yes (auto-generated) | — |
| title | String | Yes | 1–500 characters |
| composer | String | Yes | 1–200 characters |
| key | Option\<String\> | No | — |
| tempo | Option\<Tempo\> | No | marking max 100 chars; BPM 1–400 |
| notes | Option\<String\> | No | Max 5,000 characters |
| tags | Vec\<String\> | No | Each tag 1–100 characters |
| created_at | DateTime | Yes (auto-generated) | — |
| updated_at | DateTime | Yes (auto-generated) | — |

### Exercise (intrada-core)

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| id | String (ULID) | Yes (auto-generated) | — |
| title | String | Yes | 1–500 characters |
| composer | Option\<String\> | No | Max 200 characters |
| category | Option\<String\> | No | Max 100 characters |
| key | Option\<String\> | No | — |
| tempo | Option\<Tempo\> | No | marking max 100 chars; BPM 1–400 |
| notes | Option\<String\> | No | Max 5,000 characters |
| tags | Vec\<String\> | No | Each tag 1–100 characters |
| created_at | DateTime | Yes (auto-generated) | — |
| updated_at | DateTime | Yes (auto-generated) | — |

### LibraryItemView (ViewModel — intrada-core)

| Field | Type | Notes |
|-------|------|-------|
| id | String | ULID |
| item_type | String | "piece" or "exercise" |
| title | String | |
| subtitle | String | Piece: composer. Exercise: category.or(composer) |
| category | Option\<String\> | Only present for exercises |
| key | Option\<String\> | |
| tempo | Option\<String\> | Formatted display string (e.g., "Allegro (132 BPM)") |
| notes | Option\<String\> | |
| tags | Vec\<String\> | |
| created_at | String | Formatted timestamp |
| updated_at | String | Formatted timestamp |

## New UI-Only Type (intrada-web)

### ItemType Enum

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Piece,
    Exercise,
}
```

This is a **web shell type only** — not part of the core domain. It lives in `crates/intrada-web/src/types.rs` and is used to:
1. Drive the tab state signal in the add form
2. Determine which validation rules and submission event to use
3. Map to/from the existing `item_type: String` field in `LibraryItemView`

### Mapping

| ItemType | LibraryItemView.item_type | Core Event |
|----------|--------------------------|------------|
| `ItemType::Piece` | `"piece"` | `Event::Piece(PieceEvent::Add/Update)` |
| `ItemType::Exercise` | `"exercise"` | `Event::Exercise(ExerciseEvent::Add/Update)` |

## Relationships

No changes to entity relationships. Pieces and exercises remain independent domain entities with no foreign key relationships. The "Library Item" is a UI abstraction only — the unified form creates/updates the same Piece and Exercise entities as the current separate forms.

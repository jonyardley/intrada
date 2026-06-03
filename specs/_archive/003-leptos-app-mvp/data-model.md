# Data Model: 003-leptos-app-mvp

**Date**: 2026-02-14
**Feature**: Leptos Web App MVP

## Overview

This feature does not introduce new domain entities. The web shell renders the existing `ViewModel` and `LibraryItemView` types from `intrada-core`. The only new "data" is the hardcoded stub data returned by the web shell's effect handlers.

## Existing Entities (from intrada-core)

### ViewModel

The Crux core's `view()` function produces a `ViewModel` consumed by all shells (CLI and web).

| Field | Type | Description |
|-------|------|-------------|
| items | `Vec<LibraryItemView>` | Filtered/sorted library items |
| item_count | `usize` | Total count of visible items |
| error | `Option<String>` | Current error message, if any |
| status | `Option<String>` | Current status message, if any |

### LibraryItemView

Flattened view of a library item (piece or exercise) for display.

| Field | Type | Description |
|-------|------|-------------|
| id | `String` | ULID identifier |
| item_type | `String` | "piece" or "exercise" |
| title | `String` | Item title |
| subtitle | `String` | Composer (pieces) or category (exercises) |
| category | `Option<String>` | Category for exercises |
| key | `Option<String>` | Musical key |
| tempo | `Option<String>` | Formatted tempo string |
| notes | `Option<String>` | Free-text notes |
| tags | `Vec<String>` | Tags list |
| created_at | `String` | Formatted creation timestamp |
| updated_at | `String` | Formatted update timestamp |

### Domain Entities (used for stub data construction)

**Piece** (from `intrada_core::domain::piece`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | `String` | Yes | ULID |
| title | `String` | Yes | 1-500 chars |
| composer | `String` | Yes | 1-200 chars |
| key | `Option<String>` | No | Musical key |
| tempo | `Option<Tempo>` | No | Tempo marking + BPM |
| notes | `Option<String>` | No | 0-5000 chars |
| tags | `Vec<String>` | Yes (can be empty) | Tag strings, 1-100 chars each |
| created_at | `DateTime<Utc>` | Yes | Creation timestamp |
| updated_at | `DateTime<Utc>` | Yes | Last update timestamp |

**Exercise** (from `intrada_core::domain::exercise`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | `String` | Yes | ULID |
| title | `String` | Yes | 1-500 chars |
| composer | `Option<String>` | No | 0-200 chars |
| category | `Option<String>` | No | 0-100 chars |
| key | `Option<String>` | No | Musical key |
| tempo | `Option<Tempo>` | No | Tempo marking + BPM |
| notes | `Option<String>` | No | 0-5000 chars |
| tags | `Vec<String>` | Yes (can be empty) | Tag strings |
| created_at | `DateTime<Utc>` | Yes | Creation timestamp |
| updated_at | `DateTime<Utc>` | Yes | Last update timestamp |

## Stub Data Specification

The web shell's `LoadAll` effect handler must return hardcoded stub data with at least 2 items (FR-009). The stub data demonstrates the ViewModel rendering with realistic values.

### Stub Piece

| Field | Value |
|-------|-------|
| id | Generated ULID at runtime |
| title | "Clair de Lune" |
| composer | "Claude Debussy" |
| key | Some("Db Major") |
| tempo | Some(Tempo { marking: Some("Andante tres expressif"), bpm: Some(66) }) |
| notes | Some("Suite bergamasque, third movement") |
| tags | ["impressionist", "piano"] |
| created_at | Utc::now() |
| updated_at | Utc::now() |

### Stub Exercise

| Field | Value |
|-------|-------|
| id | Generated ULID at runtime |
| title | "Hanon No. 1" |
| composer | Some("Charles-Louis Hanon") |
| category | Some("Technique") |
| key | Some("C Major") |
| tempo | Some(Tempo { marking: Some("Moderato"), bpm: Some(108) }) |
| notes | None |
| tags | ["technique", "warm-up"] |
| created_at | Utc::now() |
| updated_at | Utc::now() |

## State Transitions

No new state transitions. The existing Crux event flow is reused:

1. App init → web shell sends `Event::DataLoaded { pieces, exercises }` with stub data
2. Core processes event → updates Model → produces `Render` + `Storage(LoadAll)` effects
3. Web shell handles effects → updates `RwSignal<ViewModel>` → Leptos re-renders
4. User action (e.g., button click) → web shell sends event → core produces new effects → cycle repeats
5. Write effects (Save, Update, Delete) → web shell no-ops → ViewModel updates in-memory only
6. Page reload → resets to stub data (no persistence)

## Relationships

```text
intrada-core (existing)          intrada-web (new)
┌──────────────────┐             ┌──────────────────────┐
│ Core<Intrada>    │◄────────────│ Leptos App Component  │
│   .process_event()│             │   RwSignal<ViewModel>│
│   .view()        │             │   stub effect handlers│
│                  │             │                      │
│ Piece, Exercise  │ (stub data) │ Hardcoded Piece +    │
│ ViewModel        │────────────►│   Exercise instances  │
│ LibraryItemView  │             │                      │
└──────────────────┘             └──────────────────────┘
```

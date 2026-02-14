# Data Model: Music Library

**Feature Branch**: `001-music-library`
**Date**: 2026-02-08

## Entities

### LibraryItem (base concept)

Piece and Exercise share common fields. In the domain model, they are represented as an enum with shared metadata, allowing unified listing, searching, and filtering while maintaining type-specific fields.

### Piece

A musical composition a musician is working on.

| Field    | Type           | Required | Constraints                        |
|----------|----------------|----------|------------------------------------|
| id       | ULID           | Yes      | System-generated, immutable        |
| title    | String         | Yes      | 1–500 characters, Unicode          |
| composer | String         | Yes      | 1–200 characters, Unicode          |
| key      | String (opt)   | No       | Freeform, e.g. "Db Major", "A Minor" |
| tempo    | Tempo (opt)    | No       | See Tempo value type below         |
| notes    | String (opt)   | No       | 0–5000 characters, Unicode         |
| tags     | Vec\<String\>  | No       | Each tag: 1–100 chars, deduplicated per item |
| created_at  | Timestamp   | Yes      | System-generated, immutable        |
| updated_at  | Timestamp   | Yes      | System-managed, updated on change  |

### Exercise

A practice drill or technique study.

| Field    | Type           | Required | Constraints                        |
|----------|----------------|----------|------------------------------------|
| id       | ULID           | Yes      | System-generated, immutable        |
| title    | String         | Yes      | 1–500 characters, Unicode          |
| composer | String (opt)   | No       | 1–200 characters when provided, Unicode |
| category | String (opt)   | No       | Freeform, 1–100 characters. Suggested defaults: Scales, Arpeggios, Technique, Sight-Reading, Etudes, Rhythm |
| key      | String (opt)   | No       | Freeform, e.g. "C Major", "F# Minor" |
| tempo    | Tempo (opt)    | No       | See Tempo value type below         |
| notes    | String (opt)   | No       | 0–5000 characters, Unicode         |
| tags     | Vec\<String\>  | No       | Each tag: 1–100 chars, deduplicated per item |
| created_at  | Timestamp   | Yes      | System-generated, immutable        |
| updated_at  | Timestamp   | Yes      | System-managed, updated on change  |

### Value Types

#### Tempo

Flexible representation supporting text markings, numeric BPM, or both:

| Variant       | Example                    |
|---------------|----------------------------|
| Text only     | "Andante", "Allegro con brio" |
| BPM only      | 120                        |
| Both          | "Allegro" at 132 BPM      |

Stored as an optional struct with two optional inner fields (`marking: Option<String>`, `bpm: Option<u16>`). At least one must be set when provided.

#### Tag

A plain string (1–100 characters). Tags are normalised by trimming whitespace. Comparison is case-insensitive for deduplication (e.g. "Warm-Up" and "warm-up" are the same tag). Display preserves the original casing of the first occurrence.

#### ListQuery

Used to filter and search the library. All fields are optional; when omitted, no filtering is applied for that dimension. Multiple filters combine with AND logic.

| Field     | Type              | Description                                          |
|-----------|-------------------|------------------------------------------------------|
| text      | Option\<String\>  | Case-insensitive substring match across title, composer, category, notes |
| item_type | Option\<String\>  | Filter by "piece" or "exercise"                      |
| key       | Option\<String\>  | Filter by musical key (exact match)                  |
| category  | Option\<String\>  | Filter by exercise category (exact match)            |
| tags      | Option\<Vec\<String\>\> | Filter by tags (item must have ALL specified tags) |

## Relationships

```
Piece  ──has many──▶  Tag (embedded, not a separate entity)
Exercise ──has many──▶  Tag (embedded, not a separate entity)
```

Tags are stored as a list on each item, not as a separate normalised table. This keeps the model simple for a single-user local app. If tag management becomes complex (e.g. renaming a tag across all items), this can be revisited.

## Uniqueness & Identity

- Each item is identified by a system-generated ULID.
- No uniqueness constraints on title, composer, or any other field.
- Duplicate items (same title + composer) are explicitly allowed.

## Validation Rules

| Rule | Applies to | Error message |
|------|-----------|---------------|
| Title required | Piece, Exercise | "Title is required" |
| Composer required | Piece | "Composer is required" |
| Title length | Piece, Exercise | "Title must be between 1 and 500 characters" |
| Composer length | Piece, Exercise | "Composer must be between 1 and 200 characters" |
| Notes length | Piece, Exercise | "Notes must not exceed 5000 characters" |
| Tag length | Piece, Exercise | "Each tag must be between 1 and 100 characters" |
| Category length | Exercise | "Category must be between 1 and 100 characters" |
| Tempo BPM range | Piece, Exercise | "BPM must be between 1 and 400" |
| Tempo marking length | Piece, Exercise | "Tempo marking must not exceed 100 characters" |
| Tempo completeness | Piece, Exercise | "Tempo must have at least a marking or BPM value" |

## State Transitions

Items have no explicit lifecycle states in this feature. They exist or they don't. Future features (e.g. archiving, practice session tracking) may introduce states.

## Search Behaviour

- Text search is case-insensitive and matches against: title, composer, category, notes.
- Search uses substring matching (contains), not exact match.
- Empty search query returns all items.
- Filters can be combined with text search (AND logic).
- Available filters: item type (piece/exercise), key, category, tags.

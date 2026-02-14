# Data Model: Library Add, Detail View & Editing

**Feature**: `004-library-detail-editing`
**Date**: 2026-02-14

## Existing Entities (Unchanged)

These entities are already implemented in `intrada-core`. No modifications needed.

### Piece

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| id | String (ULID) | Yes | Auto-generated |
| title | String | Yes | 1-500 characters |
| composer | String | Yes | 1-200 characters |
| key | Option\<String\> | No | Free text |
| tempo | Option\<Tempo\> | No | See Tempo below |
| notes | Option\<String\> | No | Max 5000 characters |
| tags | Vec\<String\> | No | Each tag 1-100 characters |
| created_at | DateTime\<Utc\> | Yes | Auto-set on creation |
| updated_at | DateTime\<Utc\> | Yes | Auto-updated on modification |

### Exercise

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| id | String (ULID) | Yes | Auto-generated |
| title | String | Yes | 1-500 characters |
| composer | Option\<String\> | No | 1-200 characters if present |
| category | Option\<String\> | No | 1-100 characters if present |
| key | Option\<String\> | No | Free text |
| tempo | Option\<Tempo\> | No | See Tempo below |
| notes | Option\<String\> | No | Max 5000 characters |
| tags | Vec\<String\> | No | Each tag 1-100 characters |
| created_at | DateTime\<Utc\> | Yes | Auto-set on creation |
| updated_at | DateTime\<Utc\> | Yes | Auto-updated on modification |

### Tempo

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| marking | Option\<String\> | No | Max 100 characters; at least one of marking/bpm required |
| bpm | Option\<u16\> | No | 1-400; at least one of marking/bpm required |

### LibraryItemView (ViewModel)

| Field | Type | Notes |
|-------|------|-------|
| id | String | Used for navigation to detail/edit views |
| item_type | String | "piece" or "exercise" |
| title | String | |
| subtitle | String | Composer (pieces) or category/composer (exercises) |
| category | Option\<String\> | Exercises only |
| key | Option\<String\> | Formatted for display |
| tempo | Option\<String\> | Formatted: "Marking (BPM BPM)" or parts |
| notes | Option\<String\> | |
| tags | Vec\<String\> | |
| created_at | String | RFC 3339 formatted |
| updated_at | String | RFC 3339 formatted |

## New Shell-Side Types (Web Shell Only)

These types exist only in `crates/intrada-web/src/main.rs`. They are not part of the Crux core.

### ViewState (enum)

Controls which full-page view is displayed:

| Variant | Data | Description |
|---------|------|-------------|
| List | — | Library list view (default) |
| Detail | id: String | Detail view for an item |
| AddPiece | — | Add piece form |
| AddExercise | — | Add exercise form |
| EditPiece | id: String | Edit form for a piece |
| EditExercise | id: String | Edit form for an exercise |

**State transitions**:

```
List ──click item──► Detail(id)
List ──click add──► AddPiece / AddExercise
Detail(id) ──click edit──► EditPiece(id) / EditExercise(id)
Detail(id) ──click delete + confirm──► List
Detail(id) ──click back──► List
AddPiece ──submit / cancel──► List
AddExercise ──submit / cancel──► List
EditPiece(id) ──save / cancel──► Detail(id)
EditExercise(id) ──save / cancel──► Detail(id)
```

### Form Field Signals

Each form uses individual reactive signals per field:

**Piece form fields** (add and edit):
- `title: RwSignal<String>`
- `composer: RwSignal<String>`
- `key: RwSignal<String>` (empty string = None)
- `tempo_marking: RwSignal<String>` (empty string = None)
- `bpm: RwSignal<String>` (empty string = None, parsed to u16)
- `notes: RwSignal<String>` (empty string = None)
- `tags: RwSignal<String>` (comma-separated, parsed to Vec\<String\>)

**Exercise form fields** (add and edit) — same as piece plus:
- `category: RwSignal<String>` (empty string = None)
- `composer` is optional (empty string = None)

### Validation Error State

Per-field validation errors for inline display:

- `errors: RwSignal<HashMap<String, String>>` — maps field name to error message
- Cleared on each submit attempt, repopulated with any validation failures
- Fields: "title", "composer", "key", "tempo", "notes", "tags", "category", "bpm"

## Relationships

```
ViewState ──references by id──► LibraryItemView.id
LibraryItemView ──rendered from──► ViewModel.items
ViewModel ──computed from──► Model (pieces + exercises)
Form submission ──dispatches──► Event::Piece/Exercise(Add/Update)
```

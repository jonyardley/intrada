# Library API Contract: Music Library

**Feature Branch**: `001-music-library`
**Date**: 2026-02-08

This defines the public API of the `intrada-core` Crux App — the contract between the pure core and any shell (CLI, iOS, web). The core processes Events and returns Commands containing Effects. Shells fulfil Effects and resolve them back as Events.

## Core App: `Intrada`

Implements `crux_core::App`. The shell sends Events, the core returns Commands.

### Event Enum

```rust
pub enum Event {
    // Domain events — delegated to per-domain handlers
    Piece(PieceEvent),
    Exercise(ExerciseEvent),

    // Data lifecycle
    DataLoaded { pieces: Vec<Piece>, exercises: Vec<Exercise> },
    LoadFailed(String),

    // Error handling
    ClearError,
}
```

### PieceEvent

```rust
pub enum PieceEvent {
    // User actions (sent by shell)
    Add(CreatePiece),
    Update { id: String, input: UpdatePiece },
    Delete { id: String },
    AddTags { id: String, tags: Vec<String> },
    RemoveTags { id: String, tags: Vec<String> },

    // Storage responses (resolved by shell after fulfilling effects)
    Saved(Piece),
    Updated(Piece),
    Deleted { id: String },
}
```

### ExerciseEvent

```rust
pub enum ExerciseEvent {
    // User actions
    Add(CreateExercise),
    Update { id: String, input: UpdateExercise },
    Delete { id: String },
    AddTags { id: String, tags: Vec<String> },
    RemoveTags { id: String, tags: Vec<String> },

    // Storage responses
    Saved(Exercise),
    Updated(Exercise),
    Deleted { id: String },
}
```

### Effect Enum

```rust
pub enum Effect {
    Render(RenderOperation),
    Storage(StorageEffect),
}
```

### StorageEffect

```rust
pub enum StorageEffect {
    LoadAll,
    SavePiece(Piece),
    SaveExercise(Exercise),
    UpdatePiece(Piece),
    UpdateExercise(Exercise),
    DeleteItem { id: String },
}
```

## Event → Effect Mapping

| Event | Model Change | Effect Returned |
|-------|-------------|-----------------|
| `PieceEvent::Add(input)` | Validates input, generates ULID, adds to `model.pieces` | `Storage(SavePiece(piece))` + `Render` |
| `PieceEvent::Saved(piece)` | No change (already added optimistically) | `Render` |
| `PieceEvent::Update { id, input }` | Validates input, updates piece in `model.pieces` | `Storage(UpdatePiece(piece))` + `Render` |
| `PieceEvent::Delete { id }` | Removes piece from `model.pieces` | `Storage(DeleteItem { id })` + `Render` |
| `PieceEvent::AddTags { id, tags }` | Validates tags, merges into piece's tags (case-insensitive dedup) | `Storage(UpdatePiece(piece))` + `Render` |
| `PieceEvent::RemoveTags { id, tags }` | Removes matching tags from piece | `Storage(UpdatePiece(piece))` + `Render` |
| `ExerciseEvent::Add(input)` | Validates, generates ULID, adds to `model.exercises` | `Storage(SaveExercise(exercise))` + `Render` |
| `ExerciseEvent::Update { id, input }` | Validates, updates exercise | `Storage(UpdateExercise(exercise))` + `Render` |
| `ExerciseEvent::Delete { id }` | Removes exercise | `Storage(DeleteItem { id })` + `Render` |
| `DataLoaded { pieces, exercises }` | Sets `model.pieces` and `model.exercises` | `Render` |
| `ClearError` | Clears `model.last_error` | `Render` |

**Validation errors**: When validation fails, the event handler sets `model.last_error` to a descriptive message and returns only `Render` (no Storage effect). The ViewModel exposes the error for the shell to display.

## ListQuery

```rust
pub struct ListQuery {
    pub text: Option<String>,
    pub item_type: Option<String>,
    pub key: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}
```

## Model

```rust
pub struct Model {
    pub pieces: Vec<Piece>,
    pub exercises: Vec<Exercise>,
    pub active_query: Option<ListQuery>,
    pub last_error: Option<String>,
}
```

## ViewModel

```rust
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,       // All items, filtered/sorted
    pub item_count: usize,                 // Total items in library
    pub error: Option<String>,             // Current error message
    pub status: Option<String>,            // Status message (e.g. "Piece added")
}
```

### LibraryItemView

```rust
pub struct LibraryItemView {
    pub id: String,
    pub item_type: String,                 // "piece" or "exercise"
    pub title: String,
    pub subtitle: String,                  // Composer (piece) or Category (exercise)
    pub key: Option<String>,
    pub tempo: Option<String>,             // Formatted: "Allegro (132 BPM)"
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,                // Formatted timestamp
    pub updated_at: String,
}
```

## Input Types

Unchanged from data-model.md:

- **CreatePiece**: title (required), composer (required), key, tempo, notes, tags
- **CreateExercise**: title (required), composer, category, key, tempo, notes, tags
- **UpdatePiece**: title, composer, key (clearable), tempo (clearable), notes (clearable), tags
- **UpdateExercise**: title, composer (clearable), category (clearable), key (clearable), tempo (clearable), notes (clearable), tags

## Error Types

```
LibraryError (used internally in core, displayed via ViewModel)
├── Validation { field: String, message: String }
└── NotFound { id: String }
```

Note: `StorageError` is no longer in the core — storage errors are shell-side. If the shell encounters a storage error, it can send a `LoadFailed(message)` event.

## CLI Command Mapping

| CLI Command | Event Sent | Example |
|-------------|-----------|---------|
| `intrada add piece` | `Event::Piece(PieceEvent::Add(..))` | `intrada add piece "Clair de Lune" --composer "Debussy"` |
| `intrada add exercise` | `Event::Exercise(ExerciseEvent::Add(..))` | `intrada add exercise "C Major Scale" --category "Scales"` |
| `intrada list` | Shell reads `ViewModel.items` (loads data first via `DataLoaded`) | `intrada list --type exercise` |
| `intrada show <id>` | Shell reads `ViewModel.items` and filters by ID | `intrada show 01HYX...` |
| `intrada edit <id>` | `Event::Piece(PieceEvent::Update { .. })` or `Event::Exercise(ExerciseEvent::Update { .. })` | `intrada edit 01HYX... --title "New"` |
| `intrada delete <id>` | `Event::Piece(PieceEvent::Delete { .. })` or `Event::Exercise(ExerciseEvent::Delete { .. })` | `intrada delete 01HYX...` |
| `intrada tag <id>` | `Event::Piece(PieceEvent::AddTags { .. })` or `Event::Exercise(ExerciseEvent::AddTags { .. })` | `intrada tag 01HYX... "exam"` |
| `intrada untag <id>` | `Event::Piece(PieceEvent::RemoveTags { .. })` or `Event::Exercise(ExerciseEvent::RemoveTags { .. })` | `intrada untag 01HYX... "exam"` |
| `intrada search` | Shell applies search/filter to `ViewModel.items` | `intrada search "beethoven"` |

### Shell Flow (CLI)

1. Shell starts, loads data from SQLite
2. Shell sends `Event::DataLoaded { pieces, exercises }` to core
3. Shell parses CLI args, constructs the appropriate Event
4. Shell calls `core.update(event)`, receives `Command<Effect, Event>`
5. Shell processes Effects:
   - `Render` → read ViewModel, print to terminal
   - `Storage(op)` → execute against SQLite, resolve with response Event
6. Shell exits

For list/show/search: the shell can filter `ViewModel.items` client-side (no Event needed for read-only operations once data is loaded).

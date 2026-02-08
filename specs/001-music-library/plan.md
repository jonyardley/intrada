# Implementation Plan: Music Library

**Branch**: `001-music-library` | **Date**: 2026-02-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-music-library/spec.md`

## Summary

Build a personal music library as a Rust workspace using **Crux** (crux_core 0.16.2) for cross-platform shared business logic. The core is a pure Crux App that handles all domain logic (add, browse, tag, search, edit, delete pieces and exercises) via an Event/Command architecture. The CLI is the first shell, handling SQLite persistence and terminal I/O. The core is designed to be consumed unchanged by future iOS and web shells.

## Technical Context

**Language/Version**: Rust stable (1.88+, 2021 edition)
**Application Framework**: Crux (crux_core 0.16.2) — pure core / effectful shell architecture
**Primary Dependencies**: crux_core, serde, serde_json, facet, ulid, chrono, thiserror (core); rusqlite (bundled), clap 4.5 (derive), anyhow, dirs (CLI shell)
**Storage**: SQLite via rusqlite in the CLI shell (storage is a shell concern, not in core)
**Testing**: cargo test — pure unit tests on core (no mocking needed), integration tests on CLI shell
**Target Platform**: macOS, Linux (CLI binary); core is platform-agnostic (future iOS/web)
**Project Type**: Cargo workspace — shared Crux core + CLI shell binary
**Performance Goals**: All operations < 100ms for 10,000 items; search < 200ms
**Constraints**: Local-first, single-user, offline-capable, no network dependencies

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality — PASS

- **Clarity over cleverness**: Crux enforces a clear Event → update → Command pattern. Domain types are self-documenting.
- **Single Responsibility**: Core (pure logic) separated from shell (I/O). Within core: domain types, validation, event handlers are distinct modules.
- **Consistent Style**: Will use `rustfmt` and `clippy` with default settings.
- **No Dead Code**: `#[warn(dead_code)]` enabled by default in Rust.
- **Explicit over Implicit**: All side effects are explicit Commands returned from update(). No hidden I/O.
- **Type Safety**: Rust + Crux's typed Event/Effect enums enforce compile-time correctness.

### II. Testing Standards — PASS

- **Test Coverage**: All events tested via pure unit tests. Validation tested independently.
- **Test Independence**: Core tests are pure (no I/O, no shared state). Shell integration tests use in-memory SQLite.
- **Meaningful Assertions**: Tests verify behaviour ("sending AddPiece event returns Storage effect and updates model") not implementation.
- **Fast Feedback**: Core tests are pure computation — sub-millisecond. No database needed.
- **Failure Clarity**: Tests use descriptive names and assertion messages.
- **Contract Tests**: Event → Effect contracts tested directly.

### III. User Experience Consistency — PASS

- **Error Communication**: All validation errors produce clear, non-technical messages via the ViewModel.
- **Interaction Patterns**: CLI subcommands follow consistent patterns (`add`, `list`, `show`, `edit`, `delete`, `tag`, `untag`, `search`).
- **Accessibility**: CLI output is plain text, compatible with screen readers and pipe workflows.
- **Design System Adherence**: N/A for CLI. CLI follows standard Unix conventions.
- **Loading States**: N/A — all operations are synchronous and sub-100ms.
- **Progressive Enhancement**: N/A — CLI is the baseline.

### IV. Performance Requirements — PASS

- **Response Time Budgets**: All operations target < 100ms. SQLite indexed queries + in-memory model achieve this.
- **Payload Efficiency**: ViewModel contains only display-relevant data.
- **Resource Limits**: SQLite file grows linearly. At 10,000 items, well under 10MB.
- **Lazy Loading**: Not needed at this scale.
- **Caching Strategy**: Model is the in-memory cache. SQLite has built-in page cache.
- **Measurement**: Performance validated with a 10,000-item benchmark test.

### Post-Design Re-check — PASS

No violations introduced during design. The Crux architecture adds the Event/Command layer but this is justified by the multi-frontend requirement (FR-015) and provides better testability.

## Project Structure

### Documentation (this feature)

```text
specs/001-music-library/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── library-api.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml                      # Virtual workspace manifest
crates/
├── intrada-core/               # Shared Crux App — pure business logic
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Public API: App struct, re-exports
│       ├── app.rs              # Intrada App impl (update + view), Event/Effect enums
│       ├── model.rs            # Model (in-memory state) + ViewModel (display)
│       ├── domain/
│       │   ├── mod.rs          # Re-exports
│       │   ├── piece.rs        # Piece type + PieceEvent + handle_piece_event()
│       │   ├── exercise.rs     # Exercise type + ExerciseEvent + handle_exercise_event()
│       │   └── types.rs        # Shared types: Tempo, LibraryItem, ItemType, query types
│       ├── validation.rs       # Pure validation functions (kept from current code)
│       └── error.rs            # LibraryError enum (Validation, NotFound)
└── intrada-cli/                # CLI shell — handles I/O + SQLite
    ├── Cargo.toml
    └── src/
        ├── main.rs             # Entry point, clap app, shell loop
        ├── shell.rs            # Shell: processes Effects, resolves to Events
        ├── storage.rs          # SQLite storage: fulfils Storage effects
        └── display.rs          # Format ViewModel for terminal output
```

**Structure Decision**: Crux App in `intrada-core` with per-domain event handlers. The CLI shell in `intrada-cli` processes effects and handles SQLite. Domain handlers follow the pattern from the previous Intrada: each domain (piece, exercise) has its own event enum and handler function, keeping the main `update()` as a thin dispatcher.

## Crux Architecture

### Core (intrada-core)

The core implements `crux_core::App`:

```rust
impl App for Intrada {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = ();        // Using Command API, not old capabilities
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model, _caps: &()) -> Command<Effect, Event> {
        // Dispatches to per-domain handlers
    }

    fn view(&self, model: &Model) -> ViewModel {
        // Computes display-ready ViewModel from Model
    }
}
```

### Event Flow

```
CLI parses command
    → constructs Event (e.g. Event::Piece(PieceEvent::Add(CreatePiece)))
    → core.update(event, &mut model)
    → returns Command<Effect, Event>
    → shell processes Effects:
        - Storage(SavePiece(piece)) → writes to SQLite → resolves Event::Piece(PieceEvent::Saved(piece))
        - Render → reads ViewModel → prints to terminal
```

### Effects

```rust
pub enum Effect {
    Render(RenderOperation),              // Trigger ViewModel display
    Storage(StorageEffect),               // Persistence operations
}

pub enum StorageEffect {
    SavePiece(Piece),
    SaveExercise(Exercise),
    LoadAll,                              // Load all items at startup
    DeleteItem { id: String },
    UpdatePiece(Piece),
    UpdateExercise(Exercise),
}
```

### Model vs ViewModel

- **Model**: Complete in-memory state. Holds `Vec<Piece>`, `Vec<Exercise>`, current error, active query/filter state.
- **ViewModel**: Computed for display. Contains formatted items, error messages, status text. Derives `Facet` + `Serialize` + `Deserialize` for FFI type generation.

## Complexity Tracking

> No constitution violations. No complexity justifications needed.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Crux Event/Command layer | FR-015 requires frontend-independent core; future iOS/web shells need the same core | Direct function calls would couple core to specific I/O |
| Per-domain event handlers | Keeps main update() clean as domain grows | Single monolithic update() would become unwieldy |

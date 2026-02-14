# Tasks: Music Library (Crux Architecture)

**Input**: Design documents from `/specs/001-music-library/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Workspace & Dependencies)

**Purpose**: Create Cargo workspace with Crux dependencies and initialize both crates

- [x] T001 Update workspace `Cargo.toml` to add Crux dependencies: add `crux_core = "0.17.0-rc2"` to `[workspace.dependencies]`. Keep existing deps (serde, ulid, chrono, thiserror, clap, anyhow, rusqlite, dirs, serde_json). Remove `dirs` from workspace deps (it's CLI-only, should be in intrada-cli's Cargo.toml directly)
- [x] T002 [P] Update `crates/intrada-core/Cargo.toml`: add workspace dep crux_core. Remove rusqlite and dirs (these are shell concerns, not core). Keep serde, serde_json, ulid, chrono, thiserror
- [x] T003 [P] Update `crates/intrada-cli/Cargo.toml`: add rusqlite (bundled), dirs as direct deps. Keep clap, anyhow, and path dependency on intrada-core
- [x] T004 [P] Verify `.gitignore` covers Rust targets, `.specify/` artefacts are not ignored

**Checkpoint**: `cargo build --workspace` compiles with Crux dependencies resolved

---

## Phase 2: Core Foundation (Crux App Skeleton)

**Purpose**: Define the Crux App, Event/Effect enums, Model, ViewModel, and domain types. MUST complete before any user story work.

**Warning**: No user story work can begin until this phase is complete

- [x] T005 Refactor domain types into `crates/intrada-core/src/domain/types.rs`: move Tempo, LibraryItem enum, ItemType enum, CreatePiece, CreateExercise, UpdatePiece, UpdateExercise, ListQuery from current `models.rs`. Keep existing Serialize/Deserialize/Debug/Clone derives. Create `crates/intrada-core/src/domain/mod.rs` with re-exports
- [x] T006 [P] Create Piece type in `crates/intrada-core/src/domain/piece.rs`: Piece struct (moved from models.rs), PieceEvent enum (Add, Update, Delete, AddTags, RemoveTags, Saved, Updated, Deleted), and `handle_piece_event()` stub that returns `Command::done()` for all variants
- [x] T007 [P] Create Exercise type in `crates/intrada-core/src/domain/exercise.rs`: Exercise struct (moved from models.rs), ExerciseEvent enum, and `handle_exercise_event()` stub returning `Command::done()`
- [x] T008 Define Model and ViewModel in `crates/intrada-core/src/model.rs`: Model { pieces: Vec<Piece>, exercises: Vec<Exercise>, last_error: Option<String> } with Default derive. ViewModel { items: Vec<LibraryItemView>, item_count: usize, error: Option<String>, status: Option<String> } with Serialize + Deserialize + Clone + Debug derives. LibraryItemView struct per contracts
- [x] T009 Define Effect enum in `crates/intrada-core/src/app.rs`: Effect { Render(RenderOperation), Storage(StorageEffect) }. Define StorageEffect enum { LoadAll, SavePiece, SaveExercise, UpdatePiece, UpdateExercise, DeleteItem }. Define top-level Event enum { Piece(PieceEvent), Exercise(ExerciseEvent), DataLoaded, LoadFailed, ClearError }
- [x] T010 Implement Intrada App struct in `crates/intrada-core/src/app.rs`: implement `crux_core::App` with `update()` dispatching to per-domain handlers and `view()` computing ViewModel from Model. Wire up DataLoaded event to populate model. Wire up ClearError
- [x] T011 Update `crates/intrada-core/src/lib.rs`: replace current module declarations with new structure (app, model, domain, validation, error). Re-export App struct, Event, Effect, Model, ViewModel, and all domain types
- [x] T012 Retain and update `crates/intrada-core/src/validation.rs`: keep all existing validation functions and 52 tests. Update imports to point to new domain module paths. Ensure `cargo test -p intrada-core` still passes all validation tests
- [x] T013 Update `crates/intrada-core/src/error.rs`: remove StorageError variant (storage is shell-side). Keep Validation and NotFound variants. Keep existing derives (Serialize, Deserialize)
- [x] T014 [P] Write unit tests for the Crux App skeleton in `crates/intrada-core/src/app.rs` (tests module): test that DataLoaded populates model, test that ClearError clears last_error, test that view() computes correct ViewModel from empty model, test that view() computes correct ViewModel with items

**Checkpoint**: `cargo build --workspace` compiles. `cargo test -p intrada-core` passes validation tests + new App skeleton tests. Core has a working Crux App with stubs for all domain handlers.

---

## Phase 3: CLI Shell Foundation

**Purpose**: Build the CLI shell that processes Crux Effects, handles SQLite, and displays output

- [x] T015 Implement SQLite storage in `crates/intrada-cli/src/storage.rs`: struct SqliteStore wrapping rusqlite::Connection. Methods: new() (file-based at ~/.local/share/intrada/library.db), new_in_memory() (for tests), initialize_schema() (CREATE TABLE IF NOT EXISTS for pieces and exercises). Method: load_all() -> (Vec<Piece>, Vec<Exercise>). Adapt schema from current storage/sqlite.rs
- [x] T016 Implement shell in `crates/intrada-cli/src/shell.rs`: struct Shell wrapping the Crux Core and SqliteStore. Method: process_command(event) that calls core.update(), processes returned Effects (Storage → execute against SQLite and resolve, Render → return ViewModel). This is the shell loop
- [x] T017 Implement CLI parsing in `crates/intrada-cli/src/main.rs`: define Cli struct with clap derive, Subcommands enum (Add with Piece/Exercise sub-subcommands, List, Show, Edit, Delete, Tag, Untag, Search). Wire main() to construct Shell, parse args, map to Events, call shell.process_command(), display results
- [x] T018 [P] Implement display formatting in `crates/intrada-cli/src/display.rs`: functions to format ViewModel for terminal output — format_item_list(), format_item_detail(), format_error(), format_status(). Handle empty library message

**Checkpoint**: `cargo run -p intrada-cli -- --help` shows all subcommands. Shell can start up, load from SQLite, and display "library is empty".

---

## Phase 4: User Story 1 — Add a Piece to the Library (Priority: P1)

**Goal**: Musicians can add pieces with title, composer, and optional metadata.

- [x] T019 [US1] Implement `handle_piece_event()` for PieceEvent::Add in `crates/intrada-core/src/domain/piece.rs`: validate input via validate_create_piece, generate ULID, create Piece with timestamps, add to model.pieces, return Command with Storage(SavePiece) + Render effects. Handle PieceEvent::Saved (no-op, already optimistic)
- [x] T020 [US1] Implement SqliteStore::save_piece() in `crates/intrada-cli/src/storage.rs`: INSERT into pieces table, serialize tags as JSON. Wire into Shell's effect processing
- [x] T021 [US1] Wire `intrada add piece` CLI command: parse clap args (title positional, --composer required, --key, --tempo-marking, --tempo-bpm, --notes, --tag repeatable), construct CreatePiece, send PieceEvent::Add, display result
- [x] T022 [US1] Write tests for add_piece in `crates/intrada-core/src/domain/piece.rs` (tests module): test PieceEvent::Add with valid input updates model and returns Storage effect, test validation rejects missing title, test validation rejects missing composer, test Unicode in title/composer, test tags provided at creation

**Checkpoint**: `cargo run -p intrada-cli -- add piece "Clair de Lune" --composer "Debussy"` succeeds. All add_piece core tests pass.

---

## Phase 5: User Story 2 — Add an Exercise to the Library (Priority: P1)

**Goal**: Musicians can add exercises with title and optional metadata.

- [x] T023 [US2] Implement `handle_exercise_event()` for ExerciseEvent::Add in `crates/intrada-core/src/domain/exercise.rs`: validate, generate ULID, create Exercise, add to model.exercises, return Storage(SaveExercise) + Render. Handle ExerciseEvent::Saved
- [x] T024 [US2] Implement SqliteStore::save_exercise() in `crates/intrada-cli/src/storage.rs`: INSERT into exercises table. Wire into Shell
- [x] T025 [US2] Wire `intrada add exercise` CLI command: parse clap args (title positional, --composer, --category, --key, --tempo-marking, --tempo-bpm, --notes, --tag), send ExerciseEvent::Add, display result
- [x] T026 [US2] Write tests for add_exercise in `crates/intrada-core/src/domain/exercise.rs` (tests module): test with title only, test with all fields, test validation rejects missing title, test freeform category

**Checkpoint**: `cargo run -p intrada-cli -- add exercise "C Major Scale" --category "Scales"` succeeds. All add_exercise core tests pass.

---

## Phase 6: User Story 3 — Browse and View Library Contents (Priority: P1)

**Goal**: Musicians can list all items and view full details of any individual item.

- [x] T027 [US3] Implement SqliteStore::load_all() fully in `crates/intrada-cli/src/storage.rs`: query both pieces and exercises tables, deserialize JSON tags, return (Vec<Piece>, Vec<Exercise>). Ensure Shell sends DataLoaded event at startup
- [x] T028 [US3] Implement view() in `crates/intrada-core/src/app.rs`: compute ViewModel from Model — combine pieces and exercises into Vec<LibraryItemView>, sort by created_at descending, format tempo strings, compute item_count
- [x] T029 [US3] Wire `intrada list` and `intrada show <id>` CLI commands: list displays ViewModel.items as formatted table. Show filters ViewModel.items by ID and displays full detail. Handle empty library and NotFound
- [x] T030 [US3] Write tests for view() and data loading in core: test view with mixed pieces/exercises produces correct LibraryItemView list, test sorting by created_at, test empty model produces empty items, test item_count

**Checkpoint**: Can add items and `intrada list` shows all of them. `intrada show <id>` displays details. **MVP complete — functional music library with add, list, view.**

---

## Phase 7: User Story 4 — Tag Library Items (Priority: P2)

**Goal**: Musicians can add and remove freeform tags with case-insensitive deduplication.

- [x] T031 [US4] Implement PieceEvent::AddTags and PieceEvent::RemoveTags in `crates/intrada-core/src/domain/piece.rs`: validate tags, merge/remove with case-insensitive dedup, update piece in model, return Storage(UpdatePiece) + Render. Implement same for ExerciseEvent in exercise.rs
- [x] T032 [US4] Implement SqliteStore::update_piece() and SqliteStore::update_exercise() in `crates/intrada-cli/src/storage.rs`: UPDATE row, serialize tags as JSON. Wire into Shell
- [x] T033 [US4] Wire `intrada tag` and `intrada untag` CLI commands: tag takes ID + tags, untag takes ID + tags. Determine item type (piece or exercise) from loaded model. Display updated item
- [x] T034 [US4] Write tests for tagging in core: test adding tags, test duplicate tags ignored (case-insensitive), test removing tags, test removing non-existent tag silently, test tag validation

**Checkpoint**: `intrada tag <id> "exam prep" "warm-up"` adds tags. `intrada untag <id> "warm-up"` removes. Tags visible in `intrada show`.

---

## Phase 8: User Story 5 — Search and Filter the Library (Priority: P2)

**Goal**: Search by text and filter by type, key, category, and tags.

- [x] T035 [US5] Implement search/filter in view() in `crates/intrada-core/src/app.rs`: add ListQuery to Model, apply filters when computing ViewModel.items — text search (case-insensitive substring across title, composer, category, notes), type filter, key filter, category filter, tag filter (AND logic)
- [x] T036 [US5] Wire `intrada search` and filter flags on `intrada list`: search takes query as positional arg. List adds --type, --key, --category, --tag flags. Both set ListQuery on model before computing view. Display "No results found" for empty results
- [x] T037 [US5] Write tests for search/filter in core: test text search matches title/composer/category/notes, test case-insensitive, test filter by type, test filter by key, test filter by tag, test combined search+filter, test empty search returns all

**Checkpoint**: `intrada search "debussy"` finds pieces. `intrada list --type exercise --tag "warm-up"` filters correctly.

---

## Phase 9: User Story 6 — Edit and Delete Library Items (Priority: P2)

**Goal**: Update metadata and permanently delete items.

- [x] T038 [US6] Implement PieceEvent::Update and PieceEvent::Delete in `crates/intrada-core/src/domain/piece.rs`: partial update (only provided fields changed, Some(None) clears optional fields), validate, update in model, return Storage + Render. Delete removes from model, returns Storage(DeleteItem) + Render. Same for ExerciseEvent
- [x] T039 [US6] Implement SqliteStore::delete_item() in `crates/intrada-cli/src/storage.rs`: DELETE from pieces or exercises by ID. Wire update and delete into Shell
- [x] T040 [US6] Wire `intrada edit` and `intrada delete` CLI commands: edit takes ID + optional flags. Delete takes ID, prompts confirmation (--yes to skip). Display updated/deleted status
- [x] T041 [US6] Write tests for update/delete in core: test updating piece title, test partial update leaves other fields unchanged, test clearing optional field, test update refreshes updated_at, test delete removes from model, test NotFound for invalid ID

**Checkpoint**: `intrada edit <id> --title "New Title"` updates. `intrada delete <id>` removes. All tests pass.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, performance validation, and final quality checks

- [x] T042 [P] Verify Unicode handling: add piece with "Dvořák", "Ménuet", search for Unicode text, correct round-trip through SQLite and Crux core
- [x] T043 [P] Verify edge cases: field length enforcement, empty search returns all, large library behaviour
- [x] T044 Run quickstart.md validation: update quickstart.md for Crux architecture, follow all commands end-to-end
- [x] T045 [P] Performance benchmark: 10,000 items in-memory model, measure add/list/search/delete. Assert < 100ms operations, < 200ms search

**Checkpoint**: All tests pass (`cargo test --workspace`). `cargo clippy --workspace` clean. Performance targets met.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Core Foundation (Phase 2)**: Depends on Phase 1. BLOCKS all user stories
- **CLI Shell (Phase 3)**: Depends on Phase 2. BLOCKS CLI integration of user stories
- **US1 (Phase 4)**: Depends on Phase 2 + Phase 3
- **US2 (Phase 5)**: Depends on Phase 2 + Phase 3. Can parallel with US1
- **US3 (Phase 6)**: Depends on Phase 2 + Phase 3. Benefits from US1/US2
- **US4 (Phase 7)**: Depends on US1 or US2 (needs items to tag)
- **US5 (Phase 8)**: Depends on US1/US2 + US4 (needs items with tags)
- **US6 (Phase 9)**: Depends on US1 or US2 (needs items to edit/delete)
- **Polish (Phase 10)**: Depends on all user stories

### Recommended Execution Order

```
Phase 1 (Setup)
    ↓
Phase 2 (Core Foundation)
    ↓
Phase 3 (CLI Shell)
    ↓
Phase 4 (US1: Add Piece) ⟷ Phase 5 (US2: Add Exercise) [can parallel]
    ↓
Phase 6 (US3: Browse/View) [MVP complete]
    ↓
Phase 7 (US4: Tags) → Phase 8 (US5: Search/Filter)
    ↓              ↘
Phase 9 (US6: Edit/Delete) [can parallel with US4/US5]
    ↓
Phase 10 (Polish)
```

### What We Keep from Current Code

- **models.rs types** → refactored into `domain/` with Facet derives added
- **validation.rs** (52 tests) → kept as-is, imports updated
- **error.rs** → simplified (remove StorageError)
- **SQLite schema** → moved to CLI shell's storage.rs

### What Changes

- **library.rs** (Library trait) → **replaced** by Crux App with Event/Command
- **storage/sqlite.rs** → **moved** to CLI shell (no longer in core)
- **lib.rs** → **updated** module structure

---

## Notes

- Core tests are pure — no database, no mocking, no I/O
- Shell tests use in-memory SQLite for fast, isolated execution
- Commit after each completed task or logical group
- Stop at any checkpoint to validate independently

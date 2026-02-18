# Tasks: Reusable Routines

**Input**: Design documents from `/specs/025-reusable-routines/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add the foundational domain types, validation, and module wiring that all user stories depend on. No new crate setup needed — this feature extends the existing 3-crate workspace.

- [x] T001 Create `crates/intrada-core/src/domain/routine.rs` with `Routine` and `RoutineEntry` structs — `Routine` has fields: `id: String`, `name: String`, `entries: Vec<RoutineEntry>`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`; `RoutineEntry` has fields: `id: String`, `item_id: String`, `item_title: String`, `item_type: String`, `position: usize`; both derive `Serialize, Deserialize, Debug, Clone, PartialEq`
- [x] T002 Add `pub mod routine;` to `crates/intrada-core/src/domain/mod.rs` and re-export `Routine`, `RoutineEntry`
- [x] T003 [P] Add `MAX_ROUTINE_NAME: usize = 200` constant and `validate_routine_name(name: &str) -> Result<(), LibraryError>` function to `crates/intrada-core/src/validation.rs` — validate that name is non-empty (trimmed) and does not exceed 200 characters; follow the pattern of existing validators; also add `validate_routine_entries_not_empty(entries: &[RoutineEntry]) -> Result<(), LibraryError>` to validate at least one entry
- [x] T004 [P] Add `routines: Vec<Routine>` field to `Model` struct in `crates/intrada-core/src/model.rs` (with `Default` providing empty vec); add `RoutineView` struct (id, name, entry_count, entries) and `RoutineEntryView` struct (id, item_title, item_type, position) to the same file; add `routines: Vec<RoutineView>` field to `ViewModel`
- [x] T005 Add public re-exports for `Routine`, `RoutineEntry`, `RoutineView`, `RoutineEntryView` to `crates/intrada-core/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core event handling, storage effects, database tables, and API endpoints that MUST be complete before any user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Add `RoutineEvent` enum to `crates/intrada-core/src/domain/routine.rs` with variants: `SaveBuildingAsRoutine { name: String }`, `SaveSummaryAsRoutine { name: String }`, `LoadRoutineIntoSetlist { routine_id: String }`, `DeleteRoutine { id: String }`, `UpdateRoutine { id: String, name: String, entries: Vec<RoutineEntry> }`; derive `Serialize, Deserialize, Debug, Clone, PartialEq`
- [x] T007 Add `Event::Routine(RoutineEvent)` and `Event::RoutinesLoaded { routines: Vec<Routine> }` variants to the `Event` enum in `crates/intrada-core/src/app.rs`; add `StorageEffect::LoadRoutines`, `StorageEffect::SaveRoutine(Routine)`, `StorageEffect::UpdateRoutine(Routine)`, `StorageEffect::DeleteRoutine { id: String }` variants to the `StorageEffect` enum in the same file
- [x] T008 Implement `handle_routine_event(event: RoutineEvent, model: &mut Model) -> Command<Effect, Event>` in `crates/intrada-core/src/domain/routine.rs` — implement all five event handlers following the piece/exercise handler patterns: validate inputs, update model, emit storage effects + render; see data-model.md for detailed preconditions and effects per event; for `LoadRoutineIntoSetlist`, create new `SetlistEntry` objects with `ulid::Ulid::new().to_string()` IDs, `duration_secs: 0`, `status: NotAttempted`, `notes: None`, `score: None`, and append to `BuildingSession.entries` with reindexed positions
- [x] T009 Wire `Event::Routine(routine_event)` to `handle_routine_event` and `Event::RoutinesLoaded { routines }` to `model.routines = routines` in the `update()` method in `crates/intrada-core/src/app.rs`; update the `view()` method to build `routines: Vec<RoutineView>` from `model.routines` (map each routine to RoutineView with entry_count and mapped entries)
- [x] T010 Re-export `RoutineEvent` from `crates/intrada-core/src/lib.rs`
- [x] T011 Add database migrations to `crates/intrada-api/src/migrations.rs` — three new migrations (one SQL statement each): `("0007_create_routines", "CREATE TABLE IF NOT EXISTS routines (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)")`, `("0008_create_routine_entries", "CREATE TABLE IF NOT EXISTS routine_entries (id TEXT PRIMARY KEY NOT NULL, routine_id TEXT NOT NULL REFERENCES routines(id) ON DELETE CASCADE, item_id TEXT NOT NULL, item_title TEXT NOT NULL, item_type TEXT NOT NULL, position INTEGER NOT NULL)")`, `("0009_index_routine_entries_routine_id", "CREATE INDEX IF NOT EXISTS idx_routine_entries_routine_id ON routine_entries(routine_id)")`
- [x] T012 Create `crates/intrada-api/src/db/routines.rs` with database CRUD functions following the `db/sessions.rs` parent+child pattern — implement: `list_routines(conn) -> Result<Vec<Routine>>` (query routines table, fetch entries per routine ordered by position), `get_routine(conn, id) -> Result<Option<Routine>>`, `insert_routine(conn, name, entries) -> Result<Routine>` (transaction: insert parent row with ULID + timestamps, then insert each entry with ULID + position), `update_routine(conn, id, name, entries) -> Result<Routine>` (transaction: update parent name + updated_at, delete all existing entries, insert new entries), `delete_routine(conn, id) -> Result<bool>`; add `pub mod routines;` to `crates/intrada-api/src/db/mod.rs`
- [x] T013 [P] Create `crates/intrada-api/src/routes/routines.rs` with Axum handlers following the `routes/sessions.rs` pattern — define `CreateRoutineRequest` and `UpdateRoutineRequest` structs (per contracts/api-changes.md); implement handlers: `list_routines` (GET /), `get_routine` (GET /:id), `create_routine` (POST / → 201), `update_routine` (PUT /:id), `delete_routine` (DELETE /:id); validate name (1–200 chars, using `intrada_core::validation::MAX_ROUTINE_NAME`) and entries (non-empty) in create and update handlers; return appropriate error responses (400, 404); define `pub fn router() -> Router<AppState>`
- [x] T014 Add `.nest("/routines", routines::router())` to the `api_routes()` function in `crates/intrada-api/src/routes/mod.rs`; add `pub mod routines;` to the same file
- [x] T015 [P] Add API client functions to `crates/intrada-web/src/api_client.rs` — implement: `pub async fn fetch_routines() -> Result<Vec<Routine>, ApiError>` (GET /api/routines), `pub async fn create_routine(routine: &Routine) -> Result<Routine, ApiError>` (POST /api/routines), `pub async fn update_routine(id: &str, routine: &Routine) -> Result<Routine, ApiError>` (PUT /api/routines/:id), `pub async fn delete_routine(id: &str) -> Result<(), ApiError>` (DELETE /api/routines/:id); follow existing `fetch_sessions`/`create_session`/`delete_session` patterns
- [x] T016 Add StorageEffect handlers and startup fetch to `crates/intrada-web/src/core_bridge.rs` — handle `StorageEffect::LoadRoutines` (spawn async task: fetch_routines → RoutinesLoaded), `StorageEffect::SaveRoutine` (spawn async task: create_routine → refresh_routines), `StorageEffect::UpdateRoutine` (spawn async task: update_routine → refresh_routines), `StorageEffect::DeleteRoutine` (spawn async task: delete_routine → refresh_routines); add `async fn refresh_routines(...)` helper following `refresh_sessions` pattern; add routine fetch to `fetch_initial_data` as a third parallel spawn_local task
- [x] T017 Add unit tests for all `RoutineEvent` handlers in `crates/intrada-core/src/domain/routine.rs` — test: save routine from building (valid name, entries copied, building state preserved), save routine with empty name (validation error), save routine with no entries (error), save routine from summary, load routine into setlist (entries appended with new IDs, positions reindexed), load routine into non-building state (no-op/error), delete routine (removed from model), update routine (name and entries replaced, updated_at changed), update with invalid name (error), update with empty entries (error), routine not found (error)

**Checkpoint**: Foundation ready — Routine domain types, events, effects, database, API, and web bridge are all wired. User story implementation can now begin.

---

## Phase 3: User Story 1 — Save Current Setlist as a Routine (Priority: P1) 🎯 MVP (part 1)

**Goal**: Users can save a building-phase setlist as a named routine from the session builder.

**Independent Test**: Build a setlist with 2+ items, tap "Save as Routine", enter a name, confirm it was saved. The building state remains unchanged.

### Implementation for User Story 1

- [x] T018 [US1] Create `crates/intrada-web/src/components/routine_save_form.rs` — a reusable inline save form component that accepts a callback for save events; renders a "Save as Routine" button that, when tapped, expands to show: a `TextField` for the routine name, "Save" and "Cancel" buttons; handles local validation (name required) and dispatches the provided callback with the entered name; use glassmorphism-consistent styling (glass-card, input-base, accent buttons); add `pub mod routine_save_form;` and re-export to `crates/intrada-web/src/components/mod.rs`
- [x] T019 [US1] Integrate the `RoutineSaveForm` into `crates/intrada-web/src/components/setlist_builder.rs` — below the current setlist entries, add the RoutineSaveForm component; the save callback dispatches `Event::Routine(RoutineEvent::SaveBuildingAsRoutine { name })`; only show the form when `building_setlist` has at least one entry (FR-014); process the resulting effects via `process_effects`

**Checkpoint**: At this point, users can save a setlist as a routine from the building phase.

---

## Phase 4: User Story 2 — Load a Routine into a New Session (Priority: P1) 🎯 MVP (part 2)

**Goal**: Users can see saved routines in the session builder and load them into the current setlist with one tap.

**Independent Test**: Save a routine (US1), start a new session, see the routine listed, tap Load, verify entries appear in the setlist. Load again — entries are additive.

### Implementation for User Story 2

- [x] T020 [US2] Create `crates/intrada-web/src/components/routine_loader.rs` — a component that reads `vm.routines` from the view model and renders a "Saved Routines" section; each routine displays as a row in a glass-card: name on the left, entry count badge on the right, "Load" button; tapping Load dispatches `Event::Routine(RoutineEvent::LoadRoutineIntoSetlist { routine_id })`; only render the section when routines exist (empty state: hidden or message); process effects via `process_effects`; add `pub mod routine_loader;` and re-export to `crates/intrada-web/src/components/mod.rs`
- [x] T021 [US2] Integrate `RoutineLoader` into `crates/intrada-web/src/components/setlist_builder.rs` — add the RoutineLoader component above the library items section (or below the current setlist entries); ensure it is visible during the building phase

**Checkpoint**: At this point, User Stories 1 AND 2 are complete — users can save and load routines. This is the MVP.

---

## Phase 5: User Story 3 — Save Routine from Session Summary (Priority: P2)

**Goal**: Users can save a completed session's setlist as a routine from the session summary screen.

**Independent Test**: Complete a practice session, arrive at the summary, tap "Save as Routine", enter a name, verify the routine appears when building a new session.

### Implementation for User Story 3

- [x] T022 [US3] Integrate the `RoutineSaveForm` component into `crates/intrada-web/src/components/session_summary.rs` — below the session stats section, add the RoutineSaveForm; the save callback dispatches `Event::Routine(RoutineEvent::SaveSummaryAsRoutine { name })`; only show the form when the summary has at least one entry; process the resulting effects via `process_effects`

**Checkpoint**: Users can now save routines from both building and summary phases.

---

## Phase 6: User Story 4 — Manage Routines (Priority: P2)

**Goal**: Users can view, delete, and navigate to edit their saved routines on a dedicated management page.

**Independent Test**: Navigate to `/routines`, see all routines listed, delete one, confirm it's removed after page refresh.

### Implementation for User Story 4

- [x] T023 [US4] Create `crates/intrada-web/src/views/routines.rs` — the `/routines` management page; render a `BackLink` to home, `PageHeading` "Routines", then a list of routines from `vm.routines`; each routine renders as a glass-card showing: name, entry count, "Edit" link (navigates to `/routines/:id/edit`), "Delete" button (dispatches `Event::Routine(RoutineEvent::DeleteRoutine { id })` with confirmation); show empty state message when no routines exist; add `pub mod routines;` to `crates/intrada-web/src/views/mod.rs`
- [x] T024 [US4] Add routes for `/routines` and `/routines/:id/edit` to the router in `crates/intrada-web/src/app.rs` — add `<Route path=path!("/routines") view=move || view! { <RoutinesListView /> } />` and `<Route path=path!("/routines/:id/edit") view=move || view! { <RoutineEditView /> } />`; ensure `/routines` comes before the parameterized route; import the new view components

**Checkpoint**: Users can manage routines (view list + delete). Edit page is wired but not yet implemented.

---

## Phase 7: User Story 5 — Edit Routine Details (Priority: P3)

**Goal**: Users can edit a routine's name, add/remove entries, and reorder entries on a dedicated edit page.

**Independent Test**: Navigate to `/routines/:id/edit`, change the name, remove an entry, reorder entries, add a new entry from the library, save, and verify changes persist.

### Implementation for User Story 5

- [x] T025 [US5] Create `crates/intrada-web/src/views/routine_edit.rs` — the `/routines/:id/edit` page; extract routine ID from URL params; read the routine from `vm.routines`; render: `BackLink` to `/routines`, `PageHeading` "Edit Routine", `TextField` for name, ordered list of entries (each showing: position number, item_title, item_type badge, remove button, move up/down buttons), "Add from Library" button, "Save" button; local state tracks edited name, edited entries list; on Save, dispatch `Event::Routine(RoutineEvent::UpdateRoutine { id, name, entries })` where entries are constructed from the local edited state with new RoutineEntry objects; validate before dispatch (name non-empty, entries non-empty); process effects and navigate back to `/routines` on success; add `pub mod routine_edit;` to `crates/intrada-web/src/views/mod.rs`
- [x] T026 [US5] Implement the "Add from Library" interaction in `crates/intrada-web/src/views/routine_edit.rs` — show a filterable list of library items (pieces and exercises) from `vm.items`; when the user selects an item, append a new `RoutineEntry` (with fresh ULID, item_id, item_title, item_type, position at end) to the local entries list; can use a simple toggleable section or a modal-like panel

**Checkpoint**: All user stories are now independently functional — save, load, summary save, manage, and edit.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup across all stories

- [x] T027 Run `cargo fmt --check` across the entire workspace and fix any formatting issues
- [x] T028 Run `cargo clippy -- -D warnings` across the entire workspace and fix any warnings
- [x] T029 Run `cargo test` across the entire workspace and verify all tests pass (existing + new)
- [x] T030 Run quickstart.md verification steps V1–V7 end-to-end and confirm all pass
- [x] T031 Verify routines do NOT appear in the library list view — spot-check that `vm.routines` is separate from `vm.items` and routines are not mixed into library search/filter

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion
- **User Story 2 (Phase 4)**: Depends on Foundational phase; functionally paired with US1 (need saved routines to load), but code is independent
- **User Story 3 (Phase 5)**: Depends on Foundational phase; code is independent of US1/US2
- **User Story 4 (Phase 6)**: Depends on Foundational phase; code is independent of US1–US3
- **User Story 5 (Phase 7)**: Depends on Foundational phase and US4 (needs the edit route wired in US4's T024)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2. No dependencies on other stories.
- **User Story 2 (P1)**: Can start after Phase 2. Testing requires US1 (need saved routines to load).
- **User Story 3 (P2)**: Can start after Phase 2. Independent of US1/US2.
- **User Story 4 (P2)**: Can start after Phase 2. Independent of US1–US3.
- **User Story 5 (P3)**: Depends on US4 (T024 wires the edit route). Can start edit page implementation after T024.

### Within Each User Story

- Core domain/model changes before web shell UI changes
- Event handlers before UI that dispatches events
- Unit tests in the foundational phase alongside handler implementation

### Parallel Opportunities

- T003 and T004 can run in parallel (different files: `validation.rs` vs `model.rs`)
- T013 and T015 can run in parallel (different crates: `intrada-api` vs `intrada-web`)
- T018 and T020 can run in parallel (different new component files)
- US1 (T018–T019) and US3 (T022) can be developed in parallel (different component files)
- US4 (T023–T024) can be developed in parallel with US1/US2/US3

---

## Parallel Example: User Story 1 + User Story 2

```bash
# After Phase 2 foundation is complete:

# Launch new component files in parallel:
Task T018: "Create RoutineSaveForm component in components/routine_save_form.rs"
Task T020: "Create RoutineLoader component in components/routine_loader.rs"

# Then integrate sequentially:
Task T019: "Integrate RoutineSaveForm into setlist_builder.rs" (depends on T018)
Task T021: "Integrate RoutineLoader into setlist_builder.rs" (depends on T020)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001–T005)
2. Complete Phase 2: Foundational (T006–T017)
3. Complete Phase 3: User Story 1 — Save from building (T018–T019)
4. Complete Phase 4: User Story 2 — Load into session (T020–T021)
5. **STOP and VALIDATE**: Save a routine, load it into a new session, verify additive loading
6. This delivers the core value — one-tap session setup from saved routines

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 + 2 → Save and load routines → **MVP!**
3. Add User Story 3 → Save from session summary → Deploy
4. Add User Story 4 → Routine management page → Deploy
5. Add User Story 5 → Routine editing → Deploy
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- All `Routine`/`RoutineEntry` types derive `Serialize, Deserialize, Debug, Clone, PartialEq`
- Existing tests must continue passing at every checkpoint
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- The RoutineSaveForm component is shared between US1 (building) and US3 (summary) — create once, integrate twice

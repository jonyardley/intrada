# Tasks: Session Item Scoring

**Input**: Design documents from `/specs/022-session-scoring/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No new project setup needed — this feature extends existing crates. This phase covers only the shared foundational changes that all user stories depend on.

- [x] T001 Add score validation constants `MIN_SCORE: u8 = 1` and `MAX_SCORE: u8 = 5` (and a `validate_score` function) to `crates/intrada-core/src/validation.rs`
- [x] T002 Add `score: Option<u8>` field to `SetlistEntry` struct in `crates/intrada-core/src/domain/session.rs` — include `#[serde(default)]` for backward compatibility with deserialized data; update all places where `SetlistEntry` is constructed to initialise `score: None`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Database migration and API-layer changes that MUST be complete before any user story can persist scores

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Add database migration to `crates/intrada-api/src/migrations.rs` — new migration: `ALTER TABLE setlist_entries ADD COLUMN score INTEGER;`
- [x] T004 [P] Add `score: Option<u8>` field to `SaveSessionEntry` struct in `crates/intrada-api/src/db/sessions.rs`; update the `INSERT INTO setlist_entries` SQL in `insert_session` to include the `score` column; update the `SELECT` query in `parse_entry_row` (or equivalent row-parsing code) to read the `score` column
- [x] T005 [P] Add server-side score validation in `crates/intrada-api/src/routes/sessions.rs` — in the `save_session` handler, validate each entry's score: if `score.is_some()`, value must be 1–5 (import constants from `intrada-core::validation`); return 400 Bad Request with message `"Score must be between 1 and 5"` if invalid
- [x] T006 Verify backward compatibility — run `cargo test` across the entire workspace to confirm all existing tests pass with the new `score` field (all constructors initialise `score: None`, deserialization defaults missing field to `None`)

**Checkpoint**: Foundation ready — score field flows through domain types, database, and API. User story implementation can now begin.

---

## Phase 3: User Story 1 — Score Items During Session Review (Priority: P1) 🎯 MVP

**Goal**: Users can assign a 1–5 confidence score to each completed entry on the session summary screen before saving.

**Independent Test**: Complete a practice session with multiple items, assign different scores on the summary screen, save, and verify scores are persisted when the session is viewed again.

### Implementation for User Story 1

- [x] T007 [US1] Add `UpdateEntryScore { entry_id: String, score: Option<u8> }` variant to `SessionEvent` enum in `crates/intrada-core/src/domain/session.rs`
- [x] T008 [US1] Implement `UpdateEntryScore` event handler in the session event processing logic in `crates/intrada-core/src/domain/session.rs` — when `SessionStatus::Summary`, find the entry by `entry_id`, verify `status == Completed`, validate score range (1–5 or None), and update the entry's `score` field; no-op if preconditions fail
- [x] T009 [US1] Add `score: Option<u8>` field to `SetlistEntryView` struct in `crates/intrada-core/src/model.rs`; update the `entry_to_view` (or equivalent) function that converts `SetlistEntry` → `SetlistEntryView` to pass through the score value
- [x] T010 [US1] Add unit tests for `UpdateEntryScore` event in `crates/intrada-core/src/domain/session.rs` (or `crates/intrada-core/src/app.rs` test module): test setting score on completed entry, test clearing score (toggle), test score ignored on skipped entry, test out-of-range score rejected, test score only works during Summary status
- [x] T011 [US1] Add score input UI to the session summary component in `crates/intrada-web/src/components/session_summary.rs` — for each entry with `status == "completed"`, render a row of 5 tappable number buttons (1–5) below the entry; highlight the selected button; on click, dispatch `Event::Session(SessionEvent::UpdateEntryScore { entry_id, score })` where score toggles (same value → None, different value → Some(n)); do NOT render score buttons for skipped/not-attempted entries
- [x] T012 [US1] Style the score buttons in `crates/intrada-web/src/components/session_summary.rs` — use glassmorphism-consistent styling: small rounded buttons with the app's indigo/white colour palette; selected state uses `bg-indigo-600 text-white`; unselected state uses `bg-white/10 text-white/60`; ensure touch targets are at least 44px for mobile

**Checkpoint**: At this point, User Story 1 should be fully functional — users can score items on the summary screen and save sessions with scores persisted to the database.

---

## Phase 4: User Story 2 — View Scores on Past Sessions (Priority: P2)

**Goal**: Users can see previously assigned scores when viewing completed session history.

**Independent Test**: View a saved session that has scored entries and confirm each entry displays its recorded score alongside duration, status, and notes.

### Implementation for User Story 2

- [x] T013 [US2] Update the session detail view in `crates/intrada-web/src/views/session_detail.rs` (or the component that renders a single past session) to display the `score` field from `SetlistEntryView` — show the score as a small badge or number next to each completed entry; show nothing for entries where `score` is `None`
- [x] T014 [US2] Add API integration test in `crates/intrada-api/src/` (existing test module) — test that `POST /sessions` with scored entries returns the scores in the response; test that `GET /sessions/{id}` returns the correct scores; test that entries without scores return `score: null`
- [x] T015 [US2] Verify backward compatibility display — manually or via test: create a session via the API without `score` fields in the request body and confirm it returns `score: null` for each entry; load a session detail view for this session and confirm no visual artefacts

**Checkpoint**: At this point, User Stories 1 AND 2 should both work — users can score items and then view those scores on past sessions.

---

## Phase 5: User Story 3 — Track Item Progress Over Time (Priority: P3)

**Goal**: Users can view a chronological history of confidence scores on each library item's detail page, with the most recent score highlighted.

**Independent Test**: Practice the same item across multiple sessions with varying scores, then view that item's detail page and confirm the progress summary reflects all recorded scores chronologically.

### Implementation for User Story 3

- [x] T016 [US3] Add `ScoreHistoryEntry` struct to `crates/intrada-core/src/model.rs` with fields: `session_date: String` (RFC3339), `score: u8`, `session_id: String`
- [x] T017 [US3] Add `latest_score: Option<u8>` and `score_history: Vec<ScoreHistoryEntry>` fields to `ItemPracticeSummary` struct in `crates/intrada-core/src/model.rs`
- [x] T018 [US3] Update the `compute_practice_summary` function in `crates/intrada-core/src/app.rs` to also collect score history — iterate session entries matching `item_id`, collect entries where `score.is_some()`, build `ScoreHistoryEntry` records with session date, sort by `session.started_at` descending (most recent first), set `latest_score` to the first entry's score (or None if empty)
- [x] T019 [US3] Add unit tests for score history computation in `crates/intrada-core/src/app.rs` test module — test item with multiple scored sessions returns correct chronological order; test item with no scored sessions returns empty history and `latest_score: None`; test item appearing multiple times in one session produces multiple history entries; test item with only skipped entries returns empty history
- [x] T020 [US3] Update the item detail view in `crates/intrada-web/src/views/detail.rs` to display the progress section — below the existing practice summary ("X sessions, Y min total"), add a new section: if `latest_score.is_some()`, show the latest score prominently (large number with label like "Current confidence: 4/5"); below it, render a list of `score_history` entries showing date and score; if `score_history` is empty, show "No confidence scores recorded yet"
- [x] T021 [US3] Style the progress section in `crates/intrada-web/src/views/detail.rs` — use a card or panel consistent with the glassmorphism design; latest score displayed as a large number; history list uses the app's standard list styling with subtle separators; ensure the section does NOT appear on the library list view (FR-011) — it only renders within the detail page

**Checkpoint**: All user stories should now be independently functional — scoring, viewing, and progress tracking all work end-to-end.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup across all stories

- [x] T022 Run `cargo clippy -- -D warnings` across the entire workspace and fix any warnings
- [x] T023 Run `cargo test` across the entire workspace and verify all tests pass (existing + new)
- [x] T024 Run quickstart.md verification steps V1–V5 end-to-end and confirm all pass
- [x] T025 Verify the library list view does NOT show any score-related information (FR-011 spot-check)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion
- **User Story 2 (Phase 4)**: Depends on Foundational phase; functionally independent of US1 but best done after US1 (scores must exist to view them)
- **User Story 3 (Phase 5)**: Depends on Foundational phase; requires US1 scored data to be meaningful, but code can be developed independently
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2. No dependencies on other stories.
- **User Story 2 (P2)**: Can start after Phase 2. Code is independent of US1, but testing requires scored sessions (from US1 flow).
- **User Story 3 (P3)**: Can start after Phase 2. Code is independent of US1/US2, but testing requires scored sessions.

### Within Each User Story

- Core domain/model changes before web shell UI changes
- Event handlers before UI that dispatches events
- Unit tests alongside or immediately after the code they test

### Parallel Opportunities

- T004 and T005 can run in parallel (different files: `db/sessions.rs` vs `routes/sessions.rs`)
- T007 and T009 can run in parallel (different files: `domain/session.rs` vs `model.rs`)
- T016 and T017 can run in parallel (both in `model.rs` but logically grouped — implement together)
- Once Phase 2 is complete, US1/US2/US3 core changes could theoretically proceed in parallel (different concerns)

---

## Parallel Example: User Story 1

```bash
# After Phase 2 foundation is complete:

# Core domain changes (can run in parallel across files):
Task T007: "Add UpdateEntryScore event variant in domain/session.rs"
Task T009: "Add score to SetlistEntryView in model.rs"

# Then sequentially:
Task T008: "Implement UpdateEntryScore handler" (depends on T007)
Task T010: "Add unit tests" (depends on T008)
Task T011: "Add score UI to session_summary.rs" (depends on T009)
Task T012: "Style score buttons" (depends on T011)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T006)
3. Complete Phase 3: User Story 1 (T007–T012)
4. **STOP and VALIDATE**: Score items on summary, save, and verify persistence
5. This alone delivers the core value — confidence scoring

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Score items and save → **MVP!**
3. Add User Story 2 → View scores on past sessions → Deploy
4. Add User Story 3 → Progress tracking on item detail → Deploy
5. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- All `score` field additions use `Option<u8>` with `#[serde(default)]` for backward compatibility
- Existing tests must continue passing at every checkpoint
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently

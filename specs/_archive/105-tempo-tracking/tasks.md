# Tasks: Tempo Tracking

**Input**: Design documents from `/specs/105-tempo-tracking/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, quickstart.md

**Tests**: Included — the constitution mandates test coverage for all public interfaces and critical paths.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Foundational — Core Domain Types & Validation

**Purpose**: Shared types, validation, and infrastructure that ALL user stories depend on. Must complete before any story work begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T001 [P] Add `MIN_ACHIEVED_TEMPO: u16 = 1` and `MAX_ACHIEVED_TEMPO: u16 = 500` constants, plus `validate_achieved_tempo(tempo: &Option<u16>)` function (following `validate_score()` pattern) with unit tests for None, valid range, boundaries (1, 500), and out-of-range (0, 501) in `crates/intrada-core/src/validation.rs`
- [X] T002 [P] Add `achieved_tempo: Option<u16>` field with `#[serde(default)]` to `SetlistEntry` struct in `crates/intrada-core/src/domain/session.rs`
- [X] T003 Add `TempoHistoryEntry` struct (fields: `session_date: String`, `tempo: u16`, `session_id: String`), add `latest_tempo: Option<u16>` and `tempo_history: Vec<TempoHistoryEntry>` to `ItemPracticeSummary`, add `achieved_tempo: Option<u16>` to `SetlistEntryView`, add `latest_achieved_tempo: Option<u16>` to `LibraryItemView`, and update `entry_to_view()` to map the new field in `crates/intrada-core/src/model.rs`
- [X] T004 Extend `build_practice_summaries()` to collect `TempoHistoryEntry` values in the same single-pass accumulator (change tuple from 3 to 4 elements), sort tempo history descending by date, and set `latest_tempo` from the first entry. Extend `view()` to populate `latest_achieved_tempo` on `LibraryItemView` from the practice summaries cache. Update existing `build_practice_summaries` tests to include achieved tempo data and verify tempo history ordering and latest_tempo values in `crates/intrada-core/src/app.rs`
- [X] T005 [P] Add database migration `ALTER TABLE setlist_entries ADD COLUMN achieved_tempo INTEGER` in `crates/intrada-api/src/migrations.rs` (next sequential migration number)
- [X] T006 Extend `ENTRY_COLUMNS` const to include `achieved_tempo` at position 15, update `row_to_entry()` to read column 15 as `Option<i64>` and cast to `Option<u16>`, and update the INSERT statement in `save_session_to_db()` to include `achieved_tempo` parameter in `crates/intrada-api/src/db/sessions.rs`
- [X] T007 Add `achieved_tempo` field to `SaveSessionEntry` struct (matching `SetlistEntry`), add `validate_achieved_tempo()` call in the entry validation loop of the `save_session` handler (import from `intrada_core::validation`), and verify existing API tests still pass in `crates/intrada-api/src/routes/sessions.rs`
- [X] T008 Update performance test `test_performance_10k_items` to include `achieved_tempo` values on session entries (e.g., `achieved_tempo: if e % 3 == 0 { Some(120) } else { None }`) to exercise tempo history in the cache benchmark in `crates/intrada-core/src/app.rs`

**Checkpoint**: All core types, validation, persistence, and caching are in place. Run `cargo test` — all existing tests must pass, new validation tests must pass. The achieved tempo field flows end-to-end: domain → API → database → cache.

---

## Phase 2: User Story 1 — Log Achieved Tempo During Practice (Priority: P1) 🎯 MVP

**Goal**: Musicians can record achieved BPM per completed entry during the session summary phase. Data persists and is visible in session history.

**Independent Test**: Complete a practice session, enter BPM values on the summary screen for completed entries, save, then verify the values appear in session history.

### Implementation for User Story 1

- [X] T009 [US1] Add `SessionEvent::UpdateEntryTempo { entry_id: String, tempo: Option<u16> }` variant to `SessionEvent` enum, implement handler that validates tempo using `validate_achieved_tempo()`, gates on `EntryStatus::Completed`, and sets `entry.achieved_tempo` (following the `UpdateEntryScore` pattern). Add unit tests: valid tempo on completed entry, None clears tempo, rejected on skipped entry, rejected out-of-range value in `crates/intrada-core/src/domain/session.rs`
- [X] T010 [P] [US1] Add client-side achieved tempo validation in `validate_achieved_tempo_input(value: &str)` (parse as u16, check range using core constants `MIN_ACHIEVED_TEMPO..=MAX_ACHIEVED_TEMPO`), following the `validate_bpm_input` pattern in `crates/intrada-web/src/validation.rs`
- [X] T011 [US1] Add achieved tempo input field to session summary component for each completed entry. Place below the confidence score buttons using `TextField` with `id="achieved-tempo-{entry_id}"`, `label="Achieved tempo (BPM)"`, `input_type="number"`, `placeholder="1–500"`. Only render when `entry.status == "completed"`. Wire `on:input` to dispatch `SessionEvent::UpdateEntryTempo` via the core bridge. Display validation errors inline in `crates/intrada-web/src/components/session_summary.rs`
- [X] T012 [US1] Add achieved tempo display in session history entry view. When viewing a past session's entries, show achieved tempo alongside score, notes, and duration for entries that have one (e.g., `"♩ 108 BPM"` using the existing music note icon pattern) in `crates/intrada-web/src/views/detail.rs` (session history section, not item detail — that's US2)

**Checkpoint**: User Story 1 is complete. A musician can: enter achieved tempo during summary → save session → see tempo in session history. Run `cargo test -p intrada-core` — all tests pass. Verify manually using quickstart.md steps V1, V2, V3.

---

## Phase 3: User Story 2 — View Tempo History for a Library Item (Priority: P2)

**Goal**: Musicians can see how their achieved tempo has changed over time on the item detail view, with target BPM shown as a reference.

**Independent Test**: Log achieved tempos for the same item across 3+ sessions, navigate to item detail, confirm chronological tempo history with target BPM reference.

**Dependencies**: Requires Phase 1 (cache produces `tempo_history`) and US1 (data capture). Can be implemented after foundational phase even if US1 UI isn't done, since test data can be inserted via API.

### Implementation for User Story 2

- [X] T013 [US2] Add tempo history section to item detail view. Within the existing practice summary `Card`, below the score history section: add a "Tempo History" heading (`field-label` class), display the item's target BPM as a reference if the item has `tempo` set (e.g., "Target: 120 BPM" in `text-muted`), then render each `TempoHistoryEntry` as a row with date (left, `text-muted`) and achieved BPM badge (right, same badge styling as score history). Only show the section when `tempo_history` is non-empty. Follow the existing score history display pattern exactly in `crates/intrada-web/src/views/detail.rs`

**Checkpoint**: User Story 2 is complete. A musician can view tempo progress for any item. Run `cargo test`. Verify manually using quickstart.md steps V4, V7.

---

## Phase 4: User Story 3 — See Latest Tempo on Item in Library List (Priority: P3)

**Goal**: Musicians see the latest achieved tempo alongside the target tempo for each item in the library list.

**Independent Test**: Log an achieved tempo for an item, view the library list, confirm the latest achieved BPM appears next to that item alongside the target.

**Dependencies**: Requires Phase 1 (`latest_achieved_tempo` on `LibraryItemView`). Independent of US1 and US2 UI.

### Implementation for User Story 3

- [X] T014 [US3] Add tempo badge to library item card. In the metadata cluster (alongside key and tempo display), extend the existing tempo display logic: when `latest_achieved_tempo` is present, show it alongside the target (e.g., "♩ 108 / 120 BPM" if both exist, or "♩ 108 BPM" if only achieved, keeping existing "♩ 120 BPM" for target-only). Use `text-muted` styling consistent with existing metadata. Handle all combinations: both present, achieved only, target only, neither in `crates/intrada-web/src/components/library_item_card.rs`

**Checkpoint**: User Story 3 is complete. Library list shows tempo at a glance. Verify manually using quickstart.md step V5.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final verification across all stories.

- [X] T015 Run full test suite: `cargo test` (all workspace tests pass), `cargo clippy -- -D warnings` (clean), `cargo fmt --check` (clean)
- [ ] T016 Run all quickstart.md verification steps (V1–V7) end-to-end against a running local instance
- [ ] T017 Verify session deletion removes tempo data (quickstart V6): delete a session with tempo data, confirm item's tempo history updates — this exercises the existing cascade delete + cache rebuild

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — can start immediately. BLOCKS all user stories.
- **User Story 1 (Phase 2)**: Depends on Phase 1 completion.
- **User Story 2 (Phase 3)**: Depends on Phase 1 completion. Logically depends on US1 for real data, but can be built and tested with API-inserted test data.
- **User Story 3 (Phase 4)**: Depends on Phase 1 completion. Independent of US1 and US2 UI.
- **Polish (Phase 5)**: Depends on all user stories being complete.

### Within Phase 1: Foundational

```
T001 (validation.rs) ──┐
T002 (session.rs)   ───┼── T003 (model.rs) ── T004 (app.rs) ── T008 (perf test)
T005 (migration)    ───┼── T006 (db/sessions.rs) ── T007 (routes/sessions.rs)
                       │
                       └── [P] T001, T002, T005 can run in parallel (different files)
```

### Within Phase 2: User Story 1

```
T009 (session.rs event) ──── T011 (summary UI) ── T012 (history display)
T010 (web validation)   ──┘  [P] T009, T010 can run in parallel
```

### Parallel Opportunities

- **Phase 1**: T001 (validation.rs), T002 (session.rs), T005 (migrations.rs) — three different files, no dependencies
- **Phase 2**: T009 (core event) and T010 (web validation) — different crates, no dependencies
- **Phases 3 & 4**: US2 (detail.rs) and US3 (library_item_card.rs) — different files, can run in parallel after Phase 1

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Foundational (T001–T008)
2. Complete Phase 2: User Story 1 (T009–T012)
3. **STOP and VALIDATE**: Test US1 independently — enter tempo, save, view in history
4. This delivers the core data capture that all other tempo features build on

### Incremental Delivery

1. Phase 1 → Foundation ready
2. Add US1 → Data capture works → **MVP deployed**
3. Add US2 → Tempo history visible on item detail
4. Add US3 → At-a-glance tempo in library list
5. Each story adds value without breaking previous stories

---

## Notes

- All changes extend existing files — no new modules or crates created
- The `achieved_tempo` field follows the exact same pattern as `score` throughout the entire stack
- The precomputed cache (#150) means tempo history has zero render-time cost
- Backward compatible: existing sessions without tempo data continue to work (serde default, nullable column)
- Edge cases (0, 501, non-numeric, skipped entries) are handled by validation at three levels: core, API, web client

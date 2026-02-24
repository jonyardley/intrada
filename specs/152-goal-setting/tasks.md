# Tasks: Basic Goal Setting

**Input**: Design documents from `/specs/152-goal-setting/`
**Prerequisites**: plan.md (required), spec.md (required), data-model.md, contracts/goals-api.md, research.md, quickstart.md

**Tests**: Included — the constitution mandates test coverage and the implementation plan specifies test counts (~20 core, ~10 API).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Core crate**: `crates/intrada-core/src/`
- **API crate**: `crates/intrada-api/src/`
- **Web crate**: `crates/intrada-web/src/`

---

## Phase 1: Setup

**Purpose**: Verify branch and workspace readiness — no new dependencies or structural changes needed.

- [x] T001 Verify `152-goal-setting` branch is checked out and workspace compiles (`cargo check`)

---

## Phase 2: Foundational — Domain, Validation & Core (Blocking Prerequisites)

**Purpose**: Build all domain types, validation, core app integration, and unit tests. These MUST complete before any view or API work.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

### Domain Types

- [x] T002 Create `Goal`, `GoalKind` (4 variants: SessionFrequency, PracticeTime, ItemMastery, Milestone), `GoalStatus` (Active, Completed, Archived), `GoalEvent` enum, and `handle_goal_event()` function in `crates/intrada-core/src/domain/goal.rs`. GoalKind uses `#[serde(tag = "type", rename_all = "snake_case")]` for internally-tagged serialisation. GoalEvent variants: Create, Update (title/status/deadline), Delete. Status transitions enforced: Active→Completed (sets completed_at, final), Active→Archived, Archived→Active.
- [x] T003 [P] Add `CreateGoal` (title, kind, deadline) and `UpdateGoal` (all Option — title, status, deadline as `Option<Option<DateTime<Utc>>>` for three-state semantics) to `crates/intrada-core/src/domain/types.rs`. Derive Serialize, Deserialize, Debug, Clone.
- [x] T004 Register goal module: add `pub mod goal;` to `crates/intrada-core/src/domain/mod.rs` and re-export Goal, GoalKind, GoalStatus, GoalEvent from the domain module.

### Validation

- [x] T005 Add goal validation constants and functions to `crates/intrada-core/src/validation.rs`: constants MAX_GOAL_TITLE_LEN=200, MAX_MILESTONE_DESCRIPTION_LEN=1000, MIN/MAX_TARGET_DAYS_PER_WEEK=1/7, MIN/MAX_TARGET_MINUTES_PER_WEEK=1/10080, MIN/MAX_TARGET_SCORE=1/5. Functions: `validate_create_goal(&CreateGoal) -> Result<(), LibraryError>` and `validate_update_goal(&UpdateGoal) -> Result<(), LibraryError>`. Follow existing `validate_create_item` pattern.

### Model & ViewModel

- [x] T006 Add `goals: Vec<Goal>` to `Model` and `goals: Vec<GoalView>` to `ViewModel` in `crates/intrada-core/src/model.rs`. Add `GoalView` struct (id, title, kind_label, kind_type, status, progress: Option<GoalProgress>, deadline, created_at, completed_at, item_id, item_title) and `GoalProgress` struct (current_value: f64, target_value: f64, percentage: f64 capped 0–100, display_text: String). Follow existing `SessionView`/`ItemView` patterns.

### Core App Integration

- [x] T007 Add goal-related variants to `Event` enum (`Goal(GoalEvent)`, `GoalsLoaded { goals: Vec<Goal> }`) and `AppEffect` enum (`SaveGoal(Goal)`, `UpdateGoal(Goal)`, `DeleteGoal { id: String }`, `LoadGoals`) in `crates/intrada-core/src/app.rs`. Handle each event in the `update()` match: GoalEvent dispatches to `handle_goal_event()`, GoalsLoaded stores goals in model, then all trigger view rebuild.
- [x] T008 Implement `compute_goal_progress(goals: &[Goal], sessions: &[Session], practice_summaries: &[ItemPracticeSummary], today: DateTime<Utc>) -> HashMap<String, GoalProgress>` as a pure function in `crates/intrada-core/src/app.rs`. SessionFrequency: count distinct session days in current ISO week / target. PracticeTime: sum session durations in current ISO week (minutes) / target. ItemMastery: latest score from practice_summaries for linked item / target. Milestone: 0% if active, 100% if completed. Positive display text: "3 of 5 days — great spacing for retention", "85 of 120 min — well on track", "Score 3 of 4 — steady improvement", "In progress — mark complete when ready". Zero-progress text: "Start your first session this week!".
- [x] T009 Add goal section to `view()` function in `crates/intrada-core/src/app.rs`: compute progress via `compute_goal_progress()`, map Goal + GoalProgress into GoalView for each goal, populate ViewModel.goals. Recompute on GoalsLoaded, SessionsLoaded, and any session mutation events.

### Core Unit Tests

- [x] T010 Add unit tests for goal domain types and validation in `crates/intrada-core/src/` (inline `#[cfg(test)]` modules or dedicated test file). Test: GoalKind serde round-trip for all 4 variants, GoalStatus serde, valid/invalid CreateGoal validation (title length, target ranges, score ranges), valid/invalid UpdateGoal validation, status transition enforcement (Active→Completed OK, Completed→Active rejected, Active→Archived OK, Archived→Active OK). Target: ~12 tests.
- [x] T011 Add unit tests for `compute_goal_progress()` in `crates/intrada-core/src/app.rs` (or inline test module). Test: frequency progress with 0/partial/full sessions this week, time progress with various durations, mastery progress with latest score, milestone binary progress, progress capped at 100%, empty sessions return 0 with encouraging text, week boundary reset (sessions from last week excluded). Target: ~10 tests.

**Checkpoint**: `cargo test -p intrada-core` passes with all domain, validation, and progress tests green. No WASM dependencies.

---

## Phase 3: Foundational — Database & API

**Purpose**: Build persistence layer and REST API endpoints. Depends on Phase 2 domain types.

### Database

- [x] T012 Add migration `0027_create_goals` (CREATE TABLE goals with columns: id TEXT PK, user_id TEXT NOT NULL DEFAULT '', title TEXT NOT NULL, goal_type TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'active', target_days_per_week INTEGER, target_minutes_per_week INTEGER, item_id TEXT, target_score INTEGER, milestone_description TEXT, deadline TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT) and `0028_index_goals_user_id` (CREATE INDEX idx_goals_user_id ON goals(user_id)) to `crates/intrada-api/src/migrations.rs`. Follow existing migration numbering pattern.
- [x] T013 Create `crates/intrada-api/src/db/goals.rs` with `SELECT_COLUMNS` const, `row_to_goal()` function (reconstruct GoalKind from goal_type discriminant + nullable columns), and CRUD functions: `list_goals(db, user_id)`, `get_goal(db, id, user_id)`, `insert_goal(db, goal)`, `update_goal(db, goal)`, `delete_goal(db, id, user_id)`. Follow `db/items.rs` patterns: `col!()` macro for positional indexing, ULID generation for new IDs, user_id scoping on all queries.
- [x] T014 Register goals DB module: add `pub mod goals;` to `crates/intrada-api/src/db/mod.rs`.

### API Routes

- [x] T015 Create `crates/intrada-api/src/routes/goals.rs` with 5 route handlers following `routes/items.rs` patterns: `list_goals` (GET /), `create_goal` (POST / — validate with core validation, generate ULID, insert, return 201), `get_goal` (GET /{id}), `update_goal` (PUT /{id} — validate, enforce status transitions, return 200), `delete_goal` (DELETE /{id} — return 200 with message). All handlers extract `AuthUser` for user_id scoping. Validation errors return 422 with field/message. Not-found returns 404.
- [x] T016 Nest goals router: add `.nest("/goals", goals::router())` to the API router in `crates/intrada-api/src/routes/mod.rs`.

### API Integration Tests

- [x] T017 Add integration tests for goal API in `crates/intrada-api/` (inline or test module). Test: create each of 4 goal types (201), list goals returns created goals, get single goal (200), update goal title (200), update goal status to completed (200 + completed_at set), update goal status to archived (200), reactivate archived goal (200), reject completed→active transition (400/422), delete goal (200), get deleted goal (404), validation rejection (422 for empty title, out-of-range target). Target: ~10 tests.

**Checkpoint**: `cargo test -p intrada-api` passes. API server starts and responds to curl commands from quickstart.md.

---

## Phase 4: Foundational — Web Shell Wiring

**Purpose**: Connect web shell to API and core for goal operations. Depends on Phases 2 and 3.

- [x] T018 Add goal API functions to `crates/intrada-web/src/api_client.rs`: `fetch_goals()` (GET /api/goals → Vec<Goal>), `create_goal(create: &CreateGoal)` (POST → Goal), `update_goal(id: &str, update: &UpdateGoal)` (PUT → Goal), `delete_goal(id: &str)` (DELETE). Follow existing `fetch_items`/`create_item` patterns with generic helpers and 401 retry.
- [x] T019 Wire goal effects in `crates/intrada-web/src/core_bridge.rs`: add `fetch_goals()` as 4th parallel task in `fetch_initial_data()` (alongside items, sessions, routines). Handle `AppEffect::SaveGoal`, `UpdateGoal`, `DeleteGoal`, `LoadGoals` in `process_effects()` using `spawn_mutate()` pattern — each API call followed by full goals re-fetch dispatching `GoalsLoaded`.

**Checkpoint**: Goals load on app startup. Creating/updating/deleting goals via browser console API calls works end-to-end.

---

## Phase 5: User Story 1 — Create a Practice Frequency Goal (Priority: P1) 🎯 MVP

**Goal**: Musicians can navigate to /goals, see an empty state, create a frequency goal (days/week), and see progress from their practice sessions.

**Independent Test**: Create a frequency goal of 5 days/week. Complete a practice session. Goals page shows "1 of 5 days" with a 20% progress bar and positive framing text.

### Implementation for User Story 1

- [x] T020 [US1] Create `crates/intrada-web/src/views/goals.rs` — GoalsListView component: PageHeading "Goals" with "Set a Goal" CTA button (links to /goals/new). Active goals section showing GoalCard for each active goal with: kind_type icon, title, GoalProgressBar (percentage filled + current/target text + positive display_text), deadline if set, and action buttons placeholder (complete/archive/delete — wired in US6). Empty state when no active goals: encouraging message + CTA. History section placeholder (populated in US6). Use design tokens: text-primary, text-secondary, text-muted, bg-surface-secondary, border-border-default. Components: Card, PageHeading, Button.
- [x] T021 [US1] Create `crates/intrada-web/src/views/goal_form.rs` — GoalFormView component: BackLink to /goals, PageHeading "Set a Goal". GoalTypeSelector — 4 selectable cards/tabs for goal types (Practice Frequency, Practice Time, Item Mastery, Milestone) using Leptos signals for selection state. For Frequency type: TextField for target days/week (1-7, numeric input). Auto-generated title from type + target (e.g. "Practise 5 days per week"), editable via TextField. Optional deadline date input. Create button dispatches Goal(GoalEvent::Create) with constructed CreateGoal. Client-side validation using core validation constants. Loading/submitting state via Leptos signals.
- [x] T022 [US1] Register goal view modules: add `pub mod goals; pub mod goal_form;` to `crates/intrada-web/src/views/mod.rs`.
- [x] T023 [US1] Add `/goals` and `/goals/new` routes to the Leptos router in `crates/intrada-web/src/app.rs`. `/goals` renders GoalsListView, `/goals/new` renders GoalFormView. Follow existing route registration pattern.

**Checkpoint**: Navigate to /goals, see empty state. Create a frequency goal. Goal appears with progress bar. After a practice session, progress updates.

---

## Phase 6: User Story 2 — Create a Practice Time Goal (Priority: P1)

**Goal**: Musicians can create a time goal (minutes/week) and see cumulative progress from session durations.

**Independent Test**: Create a time goal of 120 min/week. Complete a 45-minute session. Goals page shows "45 of 120 min" with ~38% progress bar.

### Implementation for User Story 2

- [x] T024 [US2] Extend GoalFormView in `crates/intrada-web/src/views/goal_form.rs` to handle Practice Time type: when "Practice Time" tab selected, show TextField for target minutes/week (numeric input, 1-10080). Auto-generate title "Practise {n} minutes per week". Wire CreateGoal with GoalKind::PracticeTime. Verify GoalProgressBar renders time progress correctly (minutes display, percentage).

**Checkpoint**: Create a time goal alongside frequency goal. Both appear on goals page with distinct progress indicators.

---

## Phase 7: User Story 3 — Create an Item Mastery Goal (Priority: P2)

**Goal**: Musicians can create a mastery goal linked to a library item with a target score, and see progress from practice scores.

**Independent Test**: Create a mastery goal for a library item with target score 4. Score the item 3 in a session. Goals page shows "Score 3 of 4" with 75% progress.

### Implementation for User Story 3

- [x] T025 [US3] Add ItemPicker component to GoalFormView in `crates/intrada-web/src/views/goal_form.rs`: when "Item Mastery" tab selected, show a dropdown/select populated from ViewModel items list (pieces and exercises). Show TextField for target score (1-5). If no library items exist, show message "Add items to your library first" with link to /library. Auto-generate title "Master {item_title}". Wire CreateGoal with GoalKind::ItemMastery { item_id, target_score }.
- [x] T026 [US3] Ensure GoalCard in `crates/intrada-web/src/views/goals.rs` displays item_title for mastery goals (from GoalView.item_title). Show "Item no longer in library" fallback when item_title is None for a mastery goal (orphaned reference case from data-model.md).

**Checkpoint**: Create a mastery goal for a library item. Progress shows current score vs target.

---

## Phase 8: User Story 4 — Create a Milestone Goal (Priority: P2)

**Goal**: Musicians can create a custom milestone goal with description and optional deadline, with binary progress (in progress / complete).

**Independent Test**: Create a milestone goal "Memorise first movement" with a deadline. Goals page shows "In progress — mark complete when ready" with 0% progress.

### Implementation for User Story 4

- [x] T027 [US4] Extend GoalFormView in `crates/intrada-web/src/views/goal_form.rs` to handle Milestone type: when "Milestone" tab selected, show TextField for title (required), TextArea for description (max 1000 chars), and date input for optional deadline. Wire CreateGoal with GoalKind::Milestone { description }. Verify GoalProgressBar shows "In progress — mark complete when ready" for active milestones.

**Checkpoint**: All four goal types can be created via the form. Each displays appropriate progress on the goals page.

---

## Phase 9: User Story 5 — View Goal Progress on Library Page (Priority: P2)

**Goal**: Musicians see a compact goals summary card on the library home page showing up to 3 active goals.

**Independent Test**: Create 2 active goals. Navigate to library page (/). Summary card appears above library list showing both goals with mini progress bars. Create 5 goals — only 3 shown with "View all" link.

### Implementation for User Story 5

- [x] T028 [US5] Add ActiveGoalsSummary component to `crates/intrada-web/src/views/library_list.rs`: compact Card rendered above the library list when ViewModel has active goals. Shows up to 3 active goals (ordered by created_at DESC — most recent first) with: title, thin inline progress bar, percentage text. "View all goals" link (A href="/goals") at bottom. Hidden entirely when no active goals. Use design tokens: card-title, text-muted, text-accent-text for link. Mini progress bars use same colour scheme as goals page but thinner.

**Checkpoint**: Library page shows goals summary when active goals exist. Hidden when none. "View all" navigates to /goals.

---

## Phase 10: User Story 6 — Complete and Archive Goals (Priority: P3)

**Goal**: Musicians can mark goals complete (final achievement), archive goals (soft removal), reactivate archived goals, and delete goals. History section shows completed/archived goals.

**Independent Test**: Create a goal, complete it — moves to history with completion date, no reactivate action. Create another, archive it — moves to history with "Reactivate" action. Reactivate it — returns to active with recalculated progress. Delete a goal — confirmation prompt, then permanently removed.

### Implementation for User Story 6

- [x] T029 [US6] Wire complete/archive/reactivate/delete actions in GoalCard within `crates/intrada-web/src/views/goals.rs`: "Complete" button dispatches Goal(GoalEvent::Update) with status=Completed (active goals only). "Archive" button dispatches with status=Archived (active goals only). "Reactivate" button dispatches with status=Active (archived goals only, not shown for completed goals). "Delete" button shows confirmation dialog then dispatches Goal(GoalEvent::Delete). Use Button component with appropriate variants (Primary for complete, Secondary for archive, Danger for delete).
- [x] T030 [US6] Add collapsible History section to GoalsListView in `crates/intrada-web/src/views/goals.rs`: below active goals, show completed and archived goals without progress bars. Completed goals show completion date and no action buttons except delete. Archived goals show "Reactivate" action. Section is collapsible (toggle via Leptos signal). Default collapsed if no history, expanded if history exists. Overdue visual indicator: goals with deadline in the past show an "Overdue" badge (text-warning-text).

**Checkpoint**: Full goal lifecycle works: create → view progress → complete/archive → history. Reactivate works for archived only. Delete with confirmation.

---

## Phase 11: User Story 7 — Navigate to Goals via Tab Bar (Priority: P3)

**Goal**: Goals tab appears in navigation on both desktop and mobile, with target/bullseye icon.

**Independent Test**: On desktop, Goals link appears in header navigation between Analytics and the end. On mobile, 5th Goals tab appears in bottom tab bar with bullseye icon. Both navigate to /goals and show active state when on /goals/*.

### Implementation for User Story 7

- [x] T031 [P] [US7] Add Goals navigation link to `crates/intrada-web/src/components/app_header.rs`: add "Goals" link after Analytics in the desktop header nav. Use target/bullseye SVG icon (consistent with existing nav icon style). Active state when current path starts with "/goals". Follow existing nav link pattern.
- [x] T032 [P] [US7] Add Goals tab to `crates/intrada-web/src/components/bottom_tab_bar.rs`: add 5th "Goals" tab after Analytics in the mobile bottom tab bar. Use matching bullseye SVG icon. Active state when current path starts with "/goals". Follow existing tab pattern (icon + label).

**Checkpoint**: Goals is accessible from any page via nav. Active state highlights correctly on /goals and /goals/new.

---

## Phase 12: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, documentation, and cleanup.

- [x] T033 Run full quickstart.md verification: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (all workspace), verify all curl commands from quickstart.md work against running API, complete UI verification steps (empty state, create each type, progress update, complete/archive, library summary, nav tab).
- [x] T034 [P] Update `CLAUDE.md`: add goal-related components (GoalProgressBar, GoalTypeSelector, GoalCard, ActiveGoalsSummary, ItemPicker) to the component table, add goal-related design tokens if any new ones were created, update Known Tech Debt if applicable.
- [x] T035 [P] Update `docs/roadmap.md`: move Issue #60 (Basic Goal Setting) from "Now" to "What's Built Today" section. Close GitHub issue #60.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Domain/Core)**: Depends on Phase 1 — BLOCKS all subsequent phases
- **Phase 3 (DB/API)**: Depends on Phase 2 (uses domain types)
- **Phase 4 (Web Wiring)**: Depends on Phases 2 and 3 (uses domain types + API endpoints)
- **Phase 5–11 (User Stories)**: All depend on Phase 4 completion
  - US1 (Phase 5) should be completed first — it builds the view infrastructure
  - US2 (Phase 6): Depends on US1 (extends same form view)
  - US3 (Phase 7): Depends on US1 (extends same form view)
  - US4 (Phase 8): Depends on US1 (extends same form view)
  - US5 (Phase 9): Can start after Phase 4 (independent view on library page)
  - US6 (Phase 10): Depends on US1 (extends goals list view)
  - US7 (Phase 11): Can start after Phase 4 (independent nav components)
- **Phase 12 (Polish)**: Depends on all desired user stories being complete

### User Story Dependencies

```text
Phase 2 (Domain) ──→ Phase 3 (DB/API) ──→ Phase 4 (Wiring) ──┐
                                                                │
                     ┌──────────────────────────────────────────┤
                     │                                          │
                     ▼                                          ▼
              US1 (Frequency) ◄── MVP                    US5 (Library Summary)
                     │                                          │
           ┌─────────┼──────────┐                              │
           ▼         ▼          ▼                              ▼
     US2 (Time) US3 (Mastery) US4 (Milestone)          US7 (Nav Tab)
           │         │          │
           └─────────┼──────────┘
                     ▼
              US6 (Complete/Archive)
                     │
                     ▼
               Phase 12 (Polish)
```

### Within Each User Story

- Implementation builds on the shared view infrastructure from US1
- Each story adds incremental functionality to existing files
- Story completion is independently verifiable per the "Independent Test" criteria

### Parallel Opportunities

- T003 (types.rs) can run in parallel with T002 (goal.rs) — different files
- T031 (app_header.rs) and T032 (bottom_tab_bar.rs) can run in parallel — different files
- US5 (Library Summary) and US7 (Nav Tab) can run in parallel with US2/US3/US4 — different files
- T034 (CLAUDE.md) and T035 (roadmap.md) can run in parallel — different files

---

## Parallel Example: Phases 2–3

```bash
# After T002 (goal.rs) completes, these can run in parallel:
Task T003: "Add CreateGoal/UpdateGoal to types.rs"
Task T005: "Add goal validation to validation.rs"

# After T007 (app.rs events/effects), these can run in parallel:
Task T008: "Implement compute_goal_progress()"
Task T010: "Add goal domain unit tests"
```

## Parallel Example: After Phase 4

```bash
# After web wiring (Phase 4) completes, these can start in parallel:
Task T020: "[US1] Goals list view"       # Goals page
Task T028: "[US5] Library summary card"  # Library page — different file
Task T031: "[US7] Header nav link"       # Header component — different file
Task T032: "[US7] Bottom tab bar"        # Bottom bar — different file
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify branch)
2. Complete Phase 2: Domain, Validation & Core (all foundational types + tests)
3. Complete Phase 3: Database & API (persistence + endpoints)
4. Complete Phase 4: Web Shell Wiring (connect frontend to backend)
5. Complete Phase 5: User Story 1 — Frequency Goals
6. **STOP and VALIDATE**: Create a frequency goal, verify progress updates after a session
7. This is a deployable MVP — musicians can set and track frequency goals

### Incremental Delivery

1. Phases 1–4 → Foundation ready (no user-visible changes yet)
2. Add US1 (Frequency) → Test independently → First deployable MVP!
3. Add US2 (Time) → Test independently → Two goal types available
4. Add US3 (Mastery) + US4 (Milestone) → All 4 goal types complete
5. Add US5 (Library Summary) → Goals visible on library page
6. Add US6 (Complete/Archive) → Full goal lifecycle
7. Add US7 (Nav Tab) → Full navigation discoverability
8. Polish → Documentation and verification

### Single Developer Strategy (Recommended)

Execute phases sequentially in order (1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12). Each phase builds on the previous. Commit after each task or logical group. Stop at any checkpoint to validate.

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks in the same phase
- [Story] label maps task to specific user story for traceability
- All domain types and validation are in Phase 2 for ALL goal types — the foundational phase defines all 4 GoalKind variants upfront because they share the same data model
- US1 builds the full view infrastructure (goals page + form skeleton); US2–US4 extend it with type-specific form fields
- Progress computation is a single pure function handling all 4 types — defined once in Phase 2
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: same file conflicts across parallel tasks

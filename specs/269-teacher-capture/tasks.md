# Tasks: Teacher Assignment Capture

**Input**: Design documents from `/specs/269-teacher-capture/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md
**Scope**: iOS + Core + API only. Web shell deferred to a follow-up.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Project initialisation — no new files needed, branch already exists.

- [x] T001 Verify branch `269-teacher-capture` is checked out and up-to-date with main

---

## Phase 2: Foundational (Core + API Backend)

**Purpose**: All backend infrastructure that MUST be complete before any UI work. This includes the full Lesson domain in core, API endpoints, database schema, and R2 storage client.

**⚠️ CRITICAL**: No user story UI work can begin until this phase is complete.

### Core Domain

- [x] T002 [P] Add Lesson, LessonPhoto, CreateLesson, and UpdateLesson types with Facet derives in `crates/intrada-core/src/domain/types.rs`
- [x] T003 [P] Add lesson validation rules (date not in future, notes max 10,000 chars, at least one of notes or photos required) in `crates/intrada-core/src/validation.rs`
- [x] T004 Add LessonEvent enum and handle_lesson_event function in `crates/intrada-core/src/domain/lesson.rs` — events: FetchLessons, FetchLesson(id), Add(CreateLesson), Update(id, UpdateLesson), Delete(id), SetLessons(response), SetLesson(response), LessonCreated(response), LessonUpdated(response), LessonDeleted(response)
- [x] T005 Register lesson module in `crates/intrada-core/src/domain/mod.rs` and wire LessonEvent into the top-level Event enum
- [x] T006 Add lesson HTTP request builders (list, get, create, update, delete) in `crates/intrada-core/src/http.rs`
- [x] T007 Add lessons list and current lesson to ViewModel in `crates/intrada-core/src/view_model.rs`

### API Backend

- [x] T008 [P] Add lessons and lesson_photos table migrations in `crates/intrada-api/src/migrations.rs` — follow schema from data-model.md
- [x] T009 [P] Create R2 storage client (upload, delete, generate public URL) in `crates/intrada-api/src/storage.rs` — uses S3-compatible API with env vars R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME
- [x] T010 Create lesson DB queries (list_lessons, get_lesson, insert_lesson, update_lesson, delete_lesson, insert_lesson_photo, delete_lesson_photo, list_lesson_photos) in `crates/intrada-api/src/db/lessons.rs` — follow SELECT_COLUMNS and col! macro pattern from items.rs
- [x] T011 Register lessons DB module in `crates/intrada-api/src/db/mod.rs`
- [x] T012 Add R2 storage client to AppState in `crates/intrada-api/src/main.rs` or `crates/intrada-api/src/routes/mod.rs`
- [x] T013 Create lesson API routes (POST, GET list, GET by id, PUT, DELETE /api/lessons + POST, DELETE /api/lessons/:id/photos) in `crates/intrada-api/src/routes/lessons.rs` — photo endpoints use axum Multipart extractor, validate at least one of notes or photos on create
- [x] T014 Mount lesson routes in `crates/intrada-api/src/routes/mod.rs`

### Type Generation

- [x] T015 Run `just typegen` to generate Swift types from new Lesson/LessonPhoto Facet types, verify output in `shared_types/`

**Checkpoint**: `cargo test` and `cargo clippy` pass for all crates. API serves lesson CRUD endpoints. Core handles lesson events and emits correct HTTP effects.

---

## Phase 3: User Story 1 — Capture a Lesson (Priority: P1) 🎯 MVP

**Goal**: A musician can open a capture form on iOS, write notes and/or attach photos, save, and see the lesson persisted.

**Independent Test**: Create a lesson with notes, verify it saves. Create a lesson with a photo, verify the photo persists. Relaunch the app, verify the lesson is still there.

- [x] T016 [P] [US1] Create PhotoCaptureView (camera + photo library picker, image compression to 2048px, upload to API) in `ios/Intrada/Features/Lessons/PhotoCaptureView.swift`
- [x] T017 [US1] Create LessonCaptureView (date picker defaulting to today, notes text area, photo capture integration, save button, preserve form state on navigate away via UserDefaults) in `ios/Intrada/Features/Lessons/LessonCaptureView.swift`
- [x] T018 [US1] Add "Log Lesson" button to Library view in `ios/Intrada/Features/Library/LibraryView.swift` — prominent action
- [x] T019 [US1] Wire photo upload to call POST /api/lessons/:id/photos directly from shell (URLSession), then dispatch event to refresh lesson data in `ios/Intrada/Features/Lessons/LessonCaptureView.swift`
- [x] T020 [US1] Create basic LessonDetailView (date heading, notes, photo gallery) in `ios/Intrada/Features/Lessons/LessonDetailView.swift` — navigate here after save to confirm capture worked
- [x] T021 [US1] Run `just ios-swift-check` to verify iOS compiles

**Checkpoint**: User can create a lesson with notes and/or photos on iOS. Lesson data persists to API. Photos stored in R2.

---

## Phase 4: User Story 2 — Review and Edit a Past Lesson (Priority: P2)

**Goal**: A musician can open a saved lesson, view it, enter edit mode, modify notes/date, add/remove photos, and save changes.

**Independent Test**: Create a lesson, reopen it, edit the notes, save, verify changes persist. Add a photo to an existing lesson. Remove a photo. Change the date.

- [x] T022 [US2] Add edit mode to LessonDetailView (editable fields, photo management, save) in `ios/Intrada/Features/Lessons/LessonDetailView.swift` — detail view created in T020
- [x] T023 [US2] Add delete lesson with `.confirmationDialog` (titleVisibility: .visible) in `ios/Intrada/Features/Lessons/LessonDetailView.swift`
- [x] T024 [US2] Run `just ios-swift-check` to verify iOS compiles

**Checkpoint**: User can view, edit, and delete lessons on iOS. Photo add/remove works in edit mode.

---

## Phase 5: User Story 3 — Browse Past Lessons (Priority: P3)

**Goal**: A musician can see all their lessons in a list, ordered by date, tap to view detail, and see an empty state when no lessons exist.

**Independent Test**: Create 3+ lessons with different dates, open the lessons list, verify reverse chronological order. Delete all lessons, verify empty state appears.

- [x] T025 [US3] Create LessonListView (reverse chronological list: date, notes preview, photo indicator, NavigationSplitView on iPad) in `ios/Intrada/Features/Lessons/LessonListView.swift`
- [x] T026 [US3] Add EmptyStateView to LessonListView when no lessons exist in `ios/Intrada/Features/Lessons/LessonListView.swift`
- [x] T027 [US3] Add navigation from lesson list to detail (NavigationLink) and `.navigationTitle("Lessons")` (large on root, inline on pushed) in `ios/Intrada/Features/Lessons/LessonListView.swift`
- [x] T028 [US3] Run `just ios-swift-check` to verify iOS compiles

**Checkpoint**: Full browse experience works on iOS. List → detail → edit flow is complete.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Ensure quality and documentation.

- [x] T029 [P] Add core unit tests for lesson event handling (create, update, delete, list, fetch) in `crates/intrada-core/` test module
- [x] T030 [P] Add API integration tests for lesson endpoints in `crates/intrada-api/` test module
- [x] T031 Verify iOS visual parity against Pencil designs in `design/intrada.pen`
- [x] T032 Verify all design system tokens used (no raw colours or spacing) across iOS lesson views
- [x] T033 Run `just ios-smoke-test` to verify full flow on simulator
- [x] T034 Update `docs/roadmap.md` — mark #267 as in progress, note iOS-first delivery

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — already complete
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — can start immediately after
- **US2 (Phase 4)**: Depends on Foundational — logically follows US1
- **US3 (Phase 5)**: Depends on Foundational — independent of US1/US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Capture)**: Needs foundational core + API. No dependency on other stories.
- **US2 (Edit)**: Needs foundational core + API. Logically depends on US1 (detail view created there).
- **US3 (Browse)**: Needs foundational core + API. Independent of US1/US2 (list view is separate).

### Parallel Opportunities

```
Phase 2 (Foundational):
  Parallel: T002 + T003 (types + validation — different files)
  Parallel: T008 + T009 (migrations + R2 client — different files)
  Sequential: T004 → T005 → T006 → T007 (core event chain)
  Sequential: T010 → T011 → T012 → T013 → T014 (API chain)

Phase 6 (Polish):
  Parallel: T029 + T030 (core tests + API tests)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 2: Foundational (core domain + API + DB)
2. Complete Phase 3: US1 — Capture form on iOS
3. **STOP and VALIDATE**: Can you create a lesson with notes and a photo? Does it persist?
4. Deploy/demo — this alone delivers the "notebook replacement" value

### Incremental Delivery

1. Foundational → Core + API ready
2. US1 (Capture) → MVP — musicians can log lessons on iOS ✓
3. US2 (Edit) → Lessons become living records ✓
4. US3 (Browse) → Full lesson history experience ✓
5. Polish → Tests, smoke test, docs ✓
6. **Future**: Web shell implementation (deferred)

---

## Notes

- **iOS-only scope**: Web shell tasks deferred to a follow-up feature
- Photo upload is shell-managed (not Crux effect) — iOS uses URLSession
- Photos compressed client-side to 2048px max edge before upload
- R2 storage requires new env vars: R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME
- iOS views MUST use `.navigationTitle()`, `CardView`, `ButtonView(variant:)`, `EmptyStateView` per CLAUDE.md
- Destructive actions (delete) require `.confirmationDialog` with `titleVisibility: .visible`
- Run `just typegen` after any Facet type changes, `just ios-swift-check` after Swift edits

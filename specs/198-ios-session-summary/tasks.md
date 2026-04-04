# Tasks: iOS Session Summary & History

**Input**: Design documents from `/specs/198-ios-session-summary/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Not explicitly requested. Validation via `just ios-swift-check`, `just ios-smoke-test`, `just ios-preview-check`.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

**Purpose**: Create shared component used by both summary and history

- [x] T001 Create `SessionEntryResultRow` in `ios/Intrada/Views/Practice/SessionEntryResultRow.swift` — shared entry result display with `isEditable: Bool` parameter. Shows: status icon (checkmark/x/minus by EntryStatus), item title + TypeBadge, duration, score badge (★ N), tempo badge (♪ N BPM), rep count badge (N/N reps), notes, intention. When editable: score uses `ScoreSelectorView`, tempo uses inline picker, notes uses TextField. Include `#Preview` with both editable and read-only states.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Wire the router to real views

- [x] T002 Create `SessionSummaryView` stub in `ios/Intrada/Views/Practice/SessionSummaryView.swift` — reads `core.viewModel.summary`. Minimal: shows "Session Complete!" header and Save/Discard buttons. Include `#Preview`.
- [x] T003 Replace `SummaryPlaceholderView` with `SessionSummaryView()` in `ios/Intrada/Views/Practice/PracticeTabRouter.swift` — update `.summary` case, remove placeholder struct.
- [x] T004 Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: App compiles, summary view renders after finishing a session.

---

## Phase 3: User Story 1 — Review & Save a Completed Session (Priority: P1) 🎯 MVP

**Goal**: Post-session review with inline editing, save to history.

**Independent Test**: Finish 3-item session → summary with item list → edit a score → add session notes → Save → returns to idle.

### Implementation

- [x] T005 [US1] Build summary header in `SessionSummaryView` — success icon, "Session Complete!" (serif heading), stats row (total duration · item count · completion status), session intention (italic, if set). "Ended Early" uses warm accent badge. Reference Pencil frame "iOS / Session Summary (iPhone)".
- [x] T006 [US1] Build entry list in `SessionSummaryView` — scrollable list of `SessionEntryResultRow` for each entry in `summary.entries`, with `isEditable: true`. Entries separated by dividers. Status icons: `.completed` → green checkmark, `.skipped` → faint X, `.notAttempted` → faint minus.
- [x] T007 [US1] Wire inline score editing — when user taps score dot in `SessionEntryResultRow`, dispatch `.session(.updateEntryScore(entryId: entry.id, score: selectedScore))`. Use `ScoreSelectorView` binding per entry. Only show for completed entries.
- [x] T008 [US1] Wire inline tempo editing — tappable tempo badge in `SessionEntryResultRow` opens inline picker (30–300 BPM, same as TransitionPromptSheet). Dispatch `.session(.updateEntryTempo(entryId: entry.id, tempo: value))`. Only show for completed entries.
- [x] T009 [US1] Wire inline notes editing — TextField in `SessionEntryResultRow` for per-item notes. Dispatch `.session(.updateEntryNotes(entryId: entry.id, notes: text))` on commit/blur.
- [x] T010 [US1] Add session notes section — "Session Notes" label + multi-line TextField. Dispatch `.session(.updateSessionNotes(notes: text))` on commit/blur.
- [x] T011 [US1] Wire Save/Discard actions — "Save Session" (Primary button) dispatches `.session(.saveSession(now: ISO8601DateFormatter().string(from: Date())))`. "Discard" (DangerOutline button) shows `.confirmationDialog`, on confirm dispatches `.session(.discardSession)`.
- [x] T012 [US1] iPad layout for summary — when `sizeClass == .regular`, render split view: left panel (header stats + session notes + Save/Discard), right panel (entry list). Reference Pencil frame "iPad / Session Summary".
- [x] T013 [US1] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Can complete a session, review results, edit scores, and save.

---

## Phase 4: User Story 2 — View Past Session History (Priority: P2)

**Goal**: Chronological session list on Practice tab idle state, tap to detail.

**Independent Test**: Save 2 sessions → go to Practice tab → see session list → tap one → see detail.

### Implementation

- [x] T014 [US2] Create `SessionHistoryView` in `ios/Intrada/Views/Practice/SessionHistoryView.swift` — reads `core.viewModel.sessions`. Shows NavigationStack with "Practice" title, "New Session" button, and session list. When `sessions` is empty, show `EmptyStateView` with "Start your first session" message and CTA. Include `#Preview`.
- [x] T015 [US2] Build session card rows — each `PracticeSessionView` rendered as a card: total duration (bold), item count, completion status badge (warm accent if ended early), timestamp, session intention (italic), truncated item names. Date-grouped with "Today", "Yesterday", or date headers using `RelativeDateTimeFormatter`.
- [x] T016 [US2] Create `SessionDetailView` in `ios/Intrada/Views/Practice/SessionDetailView.swift` — shows full session detail: header (date, duration, status, intention), entry list using `SessionEntryResultRow(isEditable: false)`. NavigationStack push from history list. Include `#Preview`.
- [x] T017 [US2] Replace `PracticeIdleView` — in `PracticeTabRouter.swift`, replace the idle case with `SessionHistoryView()`. Remove `PracticeIdleView` struct.
- [x] T018 [US2] Wire delete session — swipe-to-delete on session cards, `.confirmationDialog` on confirm, dispatch `.session(.deleteSession(id: session.id))`.
- [x] T019 [US2] iPad layout for history — when `sizeClass == .regular`, use `NavigationSplitView` with session list sidebar and detail pane (same pattern as `LibraryView`).
- [x] T020 [US2] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Session history visible on Practice tab, tap for detail, delete works.

---

## Phase 5: User Story 3 — Edit Scores After Saving (Priority: P3)

**Goal**: Tap into a past session detail and edit scores/tempo/notes.

**Independent Test**: View past session → change a score → score persists on return.

### Implementation

- [x] T021 [US3] Make `SessionDetailView` editable — add edit mode toggle or make `SessionEntryResultRow(isEditable: true)` for detail views. Wire score/tempo/notes events using the session's entry IDs.
- [x] T022 [US3] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Past sessions are editable from detail view.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T023 Add `#Preview` blocks to all new views with mock data
- [x] T024 Run full validation: `just ios-swift-check`, `just ios-smoke-test`, `just ios-preview-check`
- [x] T025 Run quickstart.md manual verification steps 1–6
- [x] T026 Update `docs/roadmap.md` — move #198 to "What's Built Today"
- [x] T027 Update `CLAUDE.md` — add new views to iOS views table

---

## Dependencies & Execution Order

```text
Phase 1 (Setup: SessionEntryResultRow) ──────────┐
Phase 2 (Foundation: stub + router) ─────────────┐│
                                                  ││
Phase 3 (US1: Summary) ◄─────────────────────────┘│
  │                                                │
  └──► Phase 4 (US2: History) ◄────────────────────┘
         │
         └──► Phase 5 (US3: Editable detail)
```

- US1 and US2 could run in parallel after Phase 2, but US2 benefits from US1's entry row being complete
- US3 is a small enhancement on US2

### Parallel Opportunities

- T005–T010 within US1 all edit different parts of SessionSummaryView (but same file, so sequential)
- T014 and T016 create different files and could be parallel

---

## Implementation Strategy

### MVP (US1 Only)

1. Phase 1: SessionEntryResultRow
2. Phase 2: Stub + router wiring
3. Phase 3: Full summary with save
4. **STOP**: Can complete and save sessions

### Incremental

1. US1 → save sessions (MVP)
2. US2 → browse history
3. US3 → edit past sessions
4. Polish → docs, previews, validation

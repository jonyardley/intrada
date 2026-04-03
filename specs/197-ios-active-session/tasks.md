# Tasks: iOS Active Session — Focus Mode, Timer & Scoring

**Input**: Design documents from `/specs/197-ios-active-session/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Not explicitly requested — test tasks omitted. Validation via `just ios-swift-check`, `just ios-smoke-test`, `just ios-preview-check`.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)

---

## Phase 1: Setup

**Purpose**: Create new files and component stubs

- [x] T001 [P] Create `ProgressRingView` component stub in `ios/Intrada/Components/ProgressRingView.swift` — circular progress indicator using `Circle().trim()` with track and fill strokes, accepts `progress: Double` (0–1) and `lineWidth: CGFloat`. Include `#Preview`.
- [x] T002 [P] Create `ScoreSelectorView` component stub in `ios/Intrada/Components/ScoreSelectorView.swift` — horizontal row of 5 tappable score dots (1–5), accepts `@Binding selectedScore: UInt8?`. Selected dot uses accent fill, unselected uses surface. Include `#Preview`.
- [x] T003 [P] Create `RepCounterView` component stub in `ios/Intrada/Components/RepCounterView.swift` — displays "X / Y" counter with Got it/Missed buttons, accepts `count: UInt8`, `target: UInt8`, `targetReached: Bool`, `onGotIt: () -> Void`, `onMissed: () -> Void`. Celebration state when target reached. Include `#Preview`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core active session view structure that all user stories build on

**⚠️ CRITICAL**: US1 depends on this phase

- [x] T004 Create `ActiveSessionView` in `ios/Intrada/Views/Practice/ActiveSessionView.swift` — main focus-mode container. Reads `core.viewModel.activeSession`. Uses `@Environment(IntradaCore.self)`. Hides navigation bar (`.toolbar(.hidden, for: .navigationBar)`). Shell-local `@State` for `elapsedSeconds: Int`, `isPaused: Bool`, `showTransitionPrompt: Bool`, `showPauseOverlay: Bool`. SwiftUI Timer publisher (1Hz) increments `elapsedSeconds`, pauses when `isPaused`. Resets `elapsedSeconds` to 0 when `currentPosition` changes. Uses `horizontalSizeClass` to switch between iPhone and iPad layouts. Include `#Preview` with mock data.
- [x] T005 Replace `ActiveSessionPlaceholderView` with `ActiveSessionView()` in `ios/Intrada/Views/Practice/PracticeTabRouter.swift` — update the `.active` case in the session status switch to render the real view. Remove the placeholder view struct.

**Checkpoint**: App compiles, active session view renders when session starts (content may be minimal)

---

## Phase 3: User Story 1 — Play Through a Session (Priority: P1) 🎯 MVP

**Goal**: Musician starts session, sees current item with timer, advances through items, finishes session.

**Independent Test**: Start 3-item session → see item 1 with timer → tap Next → item 2 → tap Next → item 3 → tap Finish → session summary.

### Implementation for User Story 1

- [x] T006 [US1] Build iPhone layout in `ActiveSessionView` — vertical stack: position label ("ITEM X OF Y" in `$text-secondary`, 12px semibold, letter-spaced), progress ring area (ghosted ring at 0.2 opacity with elapsed timer inside), item title (24px bold white), TypeBadge, intention text (14px `$text-secondary`), controls section with "Next Item" (Primary button) and "End Early" (Secondary button). Reference Pencil frame "iOS / Active Session (iPhone)". Use `space_between` justification for even distribution.
- [x] T007 [US1] Implement item advancement — "Next Item" button dispatches `.session(.nextItem(now: ISO8601DateFormatter().string(from: Date())))`. "Finish" button (shown on last item when `currentPosition + 1 == totalItems`) dispatches `.session(.finishSession(now:))`. Timer resets to 0 by detecting `currentPosition` change via `.onChange(of:)`.
- [x] T008 [US1] Implement focus mode — hide tab bar when active session is showing. In `MainTabView.swift` (or wherever the tab bar is rendered), conditionally hide or apply accent styling when `core.viewModel.sessionStatus == .active`. The active session view itself should not render any tab bar.
- [x] T009 [US1] Add tab bar accent indicator in `ios/Intrada/Navigation/MainTabView.swift` — when `sessionStatus` is `.active` or `.building`, change Practice tab icon fill to `Color.accentText` or add an accent dot. Reference spec FR-012.
- [x] T010 [US1] Run `just ios-swift-check` and `just ios-smoke-test` — fix any compile or runtime errors.

**Checkpoint**: Can play through a full session start to finish on iPhone. Tab bar shows indicator. Focus mode hides nav/tab.

---

## Phase 4: User Story 2 — Progress Ring Timer (Priority: P2)

**Goal**: Items with planned duration show countdown ring. Items without show elapsed time. Timer expiry triggers transition prompt.

**Independent Test**: Start session with 5-min item → ring animates → timer hits 0 → "Up next" prompt. Start session with no-duration item → elapsed timer only, no ring.

### Implementation for User Story 2

- [x] T011 [US2] Integrate `ProgressRingView` into `ActiveSessionView` — when `activeSession.currentPlannedDurationSecs` is non-nil, render ring with `progress = Double(elapsedSeconds) / Double(plannedDuration)`. Ring track at 0.2 opacity, fill at 0.15 opacity (ghosted per Pencil design). Timer text inside ring: 18px regular `#FFFFFF`. When `currentPlannedDurationSecs` is nil, show elapsed-only display ("MM:SS" format, no ring).
- [x] T012 [US2] Create `TransitionPromptSheet` in `ios/Intrada/Views/Practice/TransitionPromptSheet.swift` — bottom sheet (`.sheet` on iPhone) showing: drag handle, "Up Next" label + next item title + TypeBadge, separator, "How did it go?" with `ScoreSelectorView`, tempo BPM input field, notes input field, "Continue" primary button, "Skip scoring" link. Reference Pencil frame "iOS / Active Session (Transition)". On last item: show "Finish" instead of "Continue", "Session complete" instead of "Up Next". Include `#Preview`.
- [x] T013 [US2] Wire transition prompt trigger — when `elapsedSeconds >= currentPlannedDurationSecs`, set `showTransitionPrompt = true`. Present sheet via `.sheet(isPresented: $showTransitionPrompt)`. On "Continue": dispatch `updateEntryScore`, `updateEntryTempo`, `updateEntryNotes` (if values set) for current entry, then dispatch `nextItem(now:)`. On "Skip scoring": dispatch `nextItem(now:)` only. On last item "Finish": dispatch score/tempo/notes then `finishSession(now:)`. Reset `elapsedSeconds` after advancing.
- [x] T014 [US2] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Progress ring counts down. Transition prompt appears on timer expiry. Scoring optional. Can still manually tap Next anytime.

---

## Phase 5: User Story 3 — Per-Item Scoring & Feedback (Priority: P3)

**Goal**: User can rate confidence 1–5, enter tempo, add notes during transitions.

**Independent Test**: Complete item → score 4 → enter tempo 120 → add note → Continue → verify data dispatched.

### Implementation for User Story 3

- [x] T015 [US3] Finalize `ScoreSelectorView` component in `ios/Intrada/Components/ScoreSelectorView.swift` — 5 circular dots (40pt diameter), numbered 1–5. Selected dot: accent fill + white bold text. Unselected: surface fill + muted text. Tap toggles selection (tap again to deselect). Horizontal layout with center justification and 12pt gap.
- [x] T016 [US3] Add tempo and notes fields to `TransitionPromptSheet` — tempo: `TextField` with numeric keyboard, label "Tempo (BPM)", 80pt wide input with surface-input background. Notes: `TextField` with label "Notes", full-width input. Both optional, pre-filled from current entry values if they exist.
- [x] T017 [US3] Wire scoring dispatch — on "Continue" in `TransitionPromptSheet`, dispatch: `.session(.updateEntryScore(entryId: currentEntry.id, score: selectedScore))` if score selected, `.session(.updateEntryTempo(entryId: currentEntry.id, tempo: parsedTempo))` if tempo entered, `.session(.updateEntryNotes(entryId: currentEntry.id, notes: notesText))` if notes entered. Use `UInt8` for score, `UInt16` for tempo.
- [x] T018 [US3] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Scoring, tempo, and notes captured and dispatched during transitions.

---

## Phase 6: User Story 4 — Rep Counter (Priority: P4)

**Goal**: Items with rep target show counter with Got it/Missed buttons. Celebration on target reached.

**Independent Test**: Start session with exercise (rep target 5) → "0 / 5" → tap Got it 3x → "3 / 5" → tap Missed → "2 / 5" → reach 5/5 → celebration.

### Implementation for User Story 4

- [x] T019 [US4] Finalize `RepCounterView` in `ios/Intrada/Components/RepCounterView.swift` — card with `surfaceSecondary` background, `radiusCard` corners. Contains: "CONSECUTIVE REPS" label (9px faint, letter-spaced), "X / Y" count (20px bold white), progress bar (5pt height, accent fill proportional to count/target), horizontal Got it (Success button) / Missed (DangerOutline button) row. Celebration state: when `targetReached`, show checkmark icon with `successText` colour and "Target reached!" text.
- [x] T020 [US4] Integrate `RepCounterView` into `ActiveSessionView` — show below item info when `activeSession.currentRepTarget` is non-nil. Pass `count: currentRepCount ?? 0`, `target: currentRepTarget!`, `targetReached: currentRepTargetReached ?? false`. Wire `onGotIt` to `.session(.repGotIt)`, `onMissed` to `.session(.repMissed)`.
- [x] T021 [US4] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Rep counter displays and functions for items with targets. Hidden for items without.

---

## Phase 7: User Story 5 — End Early & Abandon (Priority: P5)

**Goal**: User can end session early (save partial) or abandon (discard all) with confirmations.

**Independent Test**: During item 2 of 4 → End Early → confirm → summary (2 done, 2 not attempted). Abandon → confirm → idle.

### Implementation for User Story 5

- [x] T022 [US5] Add pause overlay to `ActiveSessionView` — when pause icon tapped, set `showPauseOverlay = true` and `isPaused = true`. Render overlay: dark scrim (#000000CC), centered card with pause icon, "Session Paused" title, "Item X of Y · MM:SS elapsed" subtitle, separator, Resume (Primary), End Early (Secondary), Abandon Session (Danger) buttons. Reference Pencil frame "iOS / Active Session (Paused)". Resume: dismiss overlay, set `isPaused = false`.
- [x] T023 [US5] Wire End Early confirmation — "End Early" button (both in controls and pause overlay) shows `.confirmationDialog`. On confirm: dispatch `.session(.endSessionEarly(now:))`. Title: "End this session?" Message: "Completed items will be saved."
- [x] T024 [US5] Wire Abandon confirmation — "Abandon Session" button shows `.confirmationDialog`. On confirm: dispatch `.session(.abandonSession)`. Title: "Discard this session?" Message: "All progress will be lost." Use destructive role.
- [x] T025 [US5] Add pause button to controls area — small icon button (e.g., `pause.circle` SF Symbol) in the controls section, positioned appropriately. Only visible during active practice (not during transition prompt).
- [x] T026 [US5] Run `just ios-swift-check` — fix any compile errors.

**Checkpoint**: Pause/resume works. End Early saves partial. Abandon discards. Both have confirmation dialogs.

---

## Phase 8: iPad Layout (Enhancement)

**Goal**: iPad shows split view with session sidebar alongside focus area.

**Independent Test**: Run on iPad → start session → see sidebar with setlist + stats alongside focus area.

- [x] T027 [P] Build iPad layout in `ActiveSessionView` — when `sizeClass == .regular`, render `HStack`: left sidebar (320pt, border-right) + main focus area (fill). Reference Pencil frame "iPad / Active Session".
- [x] T028 Build sidebar content — session title (serif heading), session intention (if present), elapsed/remaining time stats, separator, "SETLIST" label, scrollable item list showing: completed items (checkmark, muted text, score if set), current item (accent left bar, highlight background, elapsed timer), pending items (circle icon, secondary text, planned duration).
- [x] T029 Run `just ios-swift-check` and `just ios-preview-check` — fix any errors.

**Checkpoint**: iPad split layout functional with session context sidebar.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final quality pass

- [x] T030 Add `#Preview` blocks to all new views — `ActiveSessionView`, `TransitionPromptSheet` with mock `ActiveSessionView` data. Ensure previews render correctly.
- [x] T031 Run full validation suite: `just ios-swift-check`, `just ios-smoke-test`, `just ios-preview-check`
- [ ] T032 Run quickstart.md manual verification steps 1–8
- [ ] T033 Update `docs/roadmap.md` — move #197 to "What's Built Today" section
- [ ] T034 Update `CLAUDE.md` — add `ActiveSessionView`, `TransitionPromptSheet`, `ProgressRingView`, `RepCounterView`, `ScoreSelectorView` to iOS components table

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phases 3–7)**: All depend on Foundational phase completion
  - US1 (P1): Can start after Phase 2
  - US2 (P2): Depends on US1 (needs the base view to add ring/transition)
  - US3 (P3): Depends on US2 (scoring lives in the transition prompt)
  - US4 (P4): Can start after Phase 2 (independent of US2/US3)
  - US5 (P5): Can start after Phase 2 (independent of US2/US3/US4)
- **iPad (Phase 8)**: Depends on US1 (needs base active session view)
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

```text
Phase 1 (Setup) ─────────────────────────────────────┐
Phase 2 (Foundation) ────────────────────────────────┐│
                                                     ││
Phase 3 (US1: Play Through) ◄────────────────────────┘│
  │                                                    │
  ├──► Phase 4 (US2: Progress Ring) ◄──────────────────┘
  │      │
  │      └──► Phase 5 (US3: Scoring)
  │
  ├──► Phase 6 (US4: Rep Counter) [parallel with US2]
  │
  ├──► Phase 7 (US5: End Early/Abandon) [parallel with US2]
  │
  └──► Phase 8 (iPad) [parallel with US2]
```

### Parallel Opportunities

- **Phase 1**: T001, T002, T003 all create different component files — run in parallel
- **After Phase 3**: US4, US5, and iPad can all proceed in parallel (independent of US2/US3)
- **Phase 8**: T027 (layout) can start while T028 (sidebar content) is deferred

---

## Parallel Example: Phase 1

```bash
# All three component stubs can be created simultaneously:
Task: "Create ProgressRingView in ios/Intrada/Components/ProgressRingView.swift"
Task: "Create ScoreSelectorView in ios/Intrada/Components/ScoreSelectorView.swift"
Task: "Create RepCounterView in ios/Intrada/Components/RepCounterView.swift"
```

## Parallel Example: After US1

```bash
# These three can proceed independently after US1 is complete:
Task: "Phase 6 (US4: Rep Counter)" — independent component + integration
Task: "Phase 7 (US5: End Early/Abandon)" — overlay + confirmations
Task: "Phase 8 (iPad)" — layout adaptation
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (3 component stubs)
2. Complete Phase 2: Foundational (ActiveSessionView + router update)
3. Complete Phase 3: User Story 1 (basic flow, Next/Finish, focus mode)
4. **STOP and VALIDATE**: Test full session flow on iPhone
5. This alone delivers a usable active session experience

### Incremental Delivery

1. Setup + Foundational → skeleton ready
2. US1 → basic session flow (MVP!)
3. US2 → progress ring + transition prompt
4. US3 → scoring in transitions
5. US4 → rep counter
6. US5 → pause/end early/abandon
7. iPad → split layout
8. Polish → previews, docs, full validation

---

## Notes

- All events already exist in the Crux core — this is shell-only work
- Timer is shell-local (`@State` + SwiftUI Timer publisher) — no FFI round-trips for ticks
- Always run `just ios-swift-check` after each task group
- Commit after each phase checkpoint
- Use design tokens exclusively — never raw SwiftUI colours

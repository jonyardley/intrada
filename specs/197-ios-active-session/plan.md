# Implementation Plan: iOS Active Session — Focus Mode, Timer & Scoring

**Branch**: `197-ios-active-session` | **Date**: 2026-04-03 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/197-ios-active-session/spec.md`

## Summary

Build the iOS active session view — a focus-mode experience that replaces the current `ActiveSessionPlaceholderView` with a real implementation covering countdown timer with progress ring, per-item scoring (1–5), rep counter, tempo/notes input, pause/resume, end early/abandon, and crash recovery. This is a **shell-only** feature — no Crux core or API changes needed. All events and ViewModel fields already exist.

## Technical Context

**Language/Version**: Swift 6.0, iOS 17.0+
**Primary Dependencies**: SwiftUI, UniFFI (CoreFfi), BCS serialization (auto-generated SharedTypes)
**Storage**: UserDefaults via `SessionStorage.swift` (crash recovery only, handled by existing effect processor)
**Testing**: `just ios-swift-check` (compile), `just ios-smoke-test` (runtime), `just ios-preview-check` (previews)
**Target Platform**: iOS 17.0+ (iPhone + iPad)
**Project Type**: Mobile (iOS shell consuming shared Crux core)
**Performance Goals**: 60fps timer animation, <30s non-practice interaction time per 5-item session
**Constraints**: Timer is shell-local (SwiftUI `Timer` publisher), scoring dispatched immediately to core
**Scale/Scope**: 4 new views, 3 new components, 1 placeholder replacement

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | Single responsibility per component, follows existing patterns |
| II. Testing Standards | ✅ PASS | Compile check, smoke test, preview check. Core logic already tested in Rust. |
| III. UX Consistency | ✅ PASS | Reuses existing component library (ButtonView, CardView, TypeBadge, etc.). Pencil designs reference shared design language. |
| IV. Performance | ✅ PASS | Shell-local timer avoids FFI round-trips. No new API calls needed. |
| V. Architecture Integrity | ✅ PASS | Pure shell implementation. Events dispatched via `core.update()`. No core modifications. |
| VI. Inclusive Design | ✅ PASS | Externalises time (progress ring), focus mode reduces decisions, celebration state for reps, optional scoring (no friction). |

## Project Structure

### Documentation (this feature)

```text
specs/197-ios-active-session/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research
├── data-model.md        # Phase 1 data model
├── quickstart.md        # Phase 1 verification steps
├── checklists/          # Requirements checklist
└── contracts/           # Empty (no new API endpoints)
```

### Source Code (repository root)

```text
ios/Intrada/
├── Views/Practice/
│   ├── PracticeTabRouter.swift          # UPDATE: Replace ActiveSessionPlaceholderView
│   ├── ActiveSessionView.swift          # NEW: Main focus-mode screen
│   ├── ActiveSessionContent.swift       # NEW: Shared content (ring, item info, controls)
│   └── TransitionPromptSheet.swift      # NEW: Between-item scoring overlay
├── Components/
│   ├── ProgressRingView.swift           # NEW: Circular countdown timer
│   ├── RepCounterView.swift             # NEW: Rep counter with Got it/Missed
│   └── ScoreSelectorView.swift          # NEW: 1-5 confidence score dots
├── Navigation/
│   └── MainTabView.swift                # UPDATE: Tab bar accent indicator for active session
└── DesignSystem/
    └── (no changes — uses existing tokens)
```

**Structure Decision**: All new files live in the existing iOS project structure following established patterns. Views in `Views/Practice/`, reusable components in `Components/`. No new directories needed.

## Design References

Pencil frames in `design/intrada.pen`:
- **iOS / Active Session (iPhone)** — Focus mode with ghosted progress ring, item title, rep counter, Next/End Early controls
- **iOS / Active Session (Transition)** — Bottom sheet with scoring dots, tempo input, notes, Continue button
- **iOS / Active Session (Paused)** — Centered overlay with Resume/End Early/Abandon buttons
- **iPad / Active Session** — Split layout: session sidebar (setlist + stats) + focus area

## Architecture

### State Flow

```text
ViewModel.sessionStatus == .active
  └─ PracticeTabRouter renders ActiveSessionView
       ├─ Reads: core.viewModel.activeSession (ActiveSessionView)
       ├─ Shell-local state:
       │   ├─ @State elapsedSeconds: Int (Timer publisher, 1Hz)
       │   ├─ @State showTransitionPrompt: Bool
       │   ├─ @State showPauseOverlay: Bool
       │   ├─ @State showEndEarlyConfirmation: Bool
       │   ├─ @State showAbandonConfirmation: Bool
       │   └─ @State entryScore/Tempo/Notes (temporary, dispatched on Continue)
       └─ Dispatches: SessionEvent variants to core.update()
```

### Timer Logic (Shell-Local)

- SwiftUI `Timer.publish(every: 1, on: .main, in: .common)` drives elapsed seconds
- When `elapsedSeconds >= currentPlannedDurationSecs`: show transition prompt
- Timer pauses when `showPauseOverlay == true`
- Timer resets to 0 on `NextItem` / `SkipItem` (core updates `currentPosition`)
- Items without `currentPlannedDurationSecs`: show elapsed-only (no ring, no auto-prompt)

### Transition Prompt Flow

1. Timer expires OR user taps "Next" → show TransitionPromptSheet
2. User optionally: selects score (1–5), enters tempo, adds notes
3. User taps "Continue" → dispatch events, advance to next item
4. User taps "Skip scoring" → advance without scoring
5. Last item: "Finish" instead of "Continue" → dispatches `finishSession`

### iPad Adaptation

- `@Environment(\.horizontalSizeClass)` drives layout
- `.regular` (iPad): Split view with session sidebar + focus area
- `.compact` (iPhone): Full-screen focus mode, transition as bottom sheet

### Event Mapping

| User Action | Shell State Change | Core Event |
|-------------|-------------------|------------|
| Tap "Next Item" | Show transition prompt | — (deferred until Continue) |
| Tap "Continue" (transition) | Reset timer, dismiss prompt | `.nextItem(now:)` + score/tempo/notes events |
| Tap "Skip scoring" | Reset timer, dismiss prompt | `.nextItem(now:)` |
| Tap "Finish" (last item) | — | `.finishSession(now:)` |
| Tap "Got it" | — | `.repGotIt` |
| Tap "Missed" | — | `.repMissed` |
| Tap pause icon | Show pause overlay | — (shell-local) |
| Tap "Resume" | Dismiss overlay, restart timer | — (shell-local) |
| Tap "End Early" | Show confirmation | — |
| Confirm "End Early" | — | `.endSessionEarly(now:)` |
| Tap "Abandon" | Show confirmation | — |
| Confirm "Abandon" | — | `.abandonSession` |

## Complexity Tracking

No constitution violations. All gates pass.

# Research: iOS Active Session

## R1: Timer Implementation Pattern

**Decision**: Use SwiftUI `Timer.publish(every: 1, on: .main, in: .common)` with `@State elapsedSeconds`

**Rationale**: The web shell uses `setInterval` for the same purpose. SwiftUI's Timer publisher integrates cleanly with the view lifecycle — automatically invalidated when the view disappears. The Crux core provides `currentPlannedDurationSecs` but the shell tracks elapsed time locally, matching the state boundary defined in CLAUDE.md.

**Alternatives considered**:
- Core-driven timer (rejected: would require FFI round-trips every second, violates shell-local timer principle)
- `DispatchSourceTimer` (rejected: more complex, no SwiftUI integration benefit)

## R2: Transition Prompt Trigger

**Decision**: Compare shell-local `elapsedSeconds >= currentPlannedDurationSecs` to trigger the transition prompt

**Rationale**: Matches web shell behaviour exactly. The prompt is non-blocking — the user can dismiss it and continue practicing. Timer keeps running after prompt appears.

**Alternatives considered**:
- Core-driven notification (rejected: core doesn't track elapsed time, timer is shell-local)

## R3: Score/Tempo/Notes Dispatch Timing

**Decision**: Dispatch score, tempo, and notes events when user taps "Continue" in the transition prompt, before dispatching `nextItem`

**Rationale**: The spec says scoring is optional. Dispatching on Continue (rather than on each keystroke) avoids unnecessary FFI round-trips. The web shell dispatches on transition. Events: `updateEntryScore`, `updateEntryTempo`, `updateEntryNotes` followed by `nextItem`.

**Alternatives considered**:
- Real-time dispatch on input (rejected: more FFI calls, no benefit since values only matter at transition)
- Batch in a single event (rejected: core expects separate events)

## R4: Focus Mode Implementation

**Decision**: Hide navigation bar and tab bar entirely during active session. No "Exit Focus Mode" button on mobile.

**Rationale**: The spec requires full-screen focus mode. The Pencil designs show no nav/tab bars. The user returns to the non-focused state by ending or abandoning the session. Pause overlay provides escape hatches (End Early, Abandon).

**Alternatives considered**:
- Toggle focus mode (rejected: adds UI complexity, user feedback during design preferred no exit button on mobile)

## R5: iPad Layout Strategy

**Decision**: Use `horizontalSizeClass` to render split view (sidebar + focus area) on iPad, full-screen on iPhone

**Rationale**: Matches the existing pattern in `SessionBuilderView.swift`. iPad has screen real estate for a session sidebar showing the setlist, elapsed/remaining time, and session intention.

**Alternatives considered**:
- Same layout for both (rejected: wastes iPad screen space, spec explicitly calls for iPad adaptation)

## R6: Crash Recovery

**Decision**: No new crash recovery code needed. Existing `AppEffect.saveSessionInProgress` / `clearSessionInProgress` handled by `IntradaCore.swift` effect processor. `SessionStorage.swift` persists to UserDefaults.

**Rationale**: The crash recovery pipeline is already implemented. Core emits save/clear effects during session state transitions. On relaunch, `startApp()` calls `SessionStorage.load()` and dispatches `recoverSession`. The shell-local timer resets to 0 on recovery (acceptable per spec edge cases).

**Alternatives considered**: None needed — fully implemented.

## R7: Progress Ring Rendering

**Decision**: Custom SwiftUI view using `Circle().trim(from:to:)` with `.stroke()` for the ring, animated via `withAnimation`

**Rationale**: SwiftUI's `trim` modifier creates clean stroke-only arcs (unlike Pencil's ellipse which renders pie wedges). The ring fill percentage = `elapsedSeconds / plannedDurationSecs`. Background track uses the same circle with `surfacePrimary` at low opacity.

**Alternatives considered**:
- `CAShapeLayer` (rejected: UIKit interop unnecessary, SwiftUI native solution is simpler)

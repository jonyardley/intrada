# Research: Focus Mode

**Date**: 2026-02-23
**Feature**: 048-focus-mode

## Decision 1: Planned Duration Storage

**Decision**: Add `planned_duration_secs: Option<u32>` to `SetlistEntry` domain model and persist via API/DB.

**Rationale**: Planned duration is domain data (it affects what the user planned, the progress ring, and transition prompts). Per the Crux architecture, domain state belongs in the core model, not in Leptos signals. It must also persist through crash recovery (localStorage serialises the `ActiveSession`) and be saved to the API for potential future analytics.

**Alternatives considered**:
- Leptos-only signal: Rejected — would not survive crash recovery, violates state boundary rules, and is inaccessible to core logic.
- Derive from total session time / item count: Rejected — too imprecise for mixed sessions (tracked in #142 for future exploration).

## Decision 2: Circular Progress Ring Implementation

**Decision**: SVG `<circle>` with `stroke-dasharray` / `stroke-dashoffset` driven by a reactive signal. Digital timer (MM:SS) centred inside the ring via absolute positioning.

**Rationale**: The existing codebase uses inline SVG in Leptos `view!` macros (see `line_chart.rs`). SVG circles with dash offset are the standard pattern for progress rings — no external dependencies needed. The ring animates smoothly via CSS transition on `stroke-dashoffset`.

**Alternatives considered**:
- Canvas-based rendering: Rejected — adds complexity, no existing canvas patterns in codebase.
- CSS conic-gradient: Rejected — less browser support, harder to animate smoothly, no existing pattern.
- External JS library: Rejected — adds dependency to WASM build, violates simplicity principle.

## Decision 3: Focus Mode Signal Architecture

**Decision**: App-level `RwSignal<bool>` provided as Leptos context from `AuthenticatedApp`. The active session view sets it to `true` on mount and `false` on unmount. `AppHeader`, `AppFooter`, and `BottomTabBar` read the signal to conditionally render.

**Rationale**: Focus mode is UI interaction state ("what the user is doing right now"), not domain data. Per the state boundary rules in CLAUDE.md, this belongs in Leptos signals, not the Crux model. An app-level context signal is the simplest approach — it's already the pattern used for `IsLoading` and `SharedCore`.

**Alternatives considered**:
- Route-based: Check current path in header. Rejected — fragile, couples layout to route strings.
- Crux model field: Rejected — focus mode has no meaning outside the current view, shouldn't inflate the Crux model.

## Decision 4: Navigation Hiding Approach

**Decision**: Conditionally render `AppHeader`, `AppFooter`, and `BottomTabBar` based on the focus mode signal. When focus mode is active, these components simply don't render. The toggle button in the session timer sets the signal to reveal/hide them.

**Rationale**: Simplest approach — conditional rendering with `Show` or `if` in Leptos is idiomatic. No CSS animation needed for the nav (it just appears/disappears). The toggle button lives inside `SessionTimer` and controls the same signal.

**Alternatives considered**:
- CSS visibility toggle (display:none): Works but leaves elements in DOM. Conditional rendering is cleaner and more Leptos-idiomatic.
- Slide animation: Nice but adds complexity for v1. Can be added later via CSS transition.

## Decision 5: Transition Prompt Trigger

**Decision**: Compare `elapsed_secs >= planned_duration_secs` on each timer tick (1s interval). When the condition first becomes true, set a `duration_elapsed: RwSignal<bool>` signal. The `TransitionPrompt` component renders when this signal is true.

**Rationale**: The timer already ticks every second via `setInterval`. Adding a comparison is trivial. The prompt is a Leptos signal (UI interaction state) — it has no meaning outside the current view and resets when advancing to the next item.

**Alternatives considered**:
- Core event: Rejected — this is a UI presentation concern, not domain logic. The core doesn't need to know about the visual prompt.
- setTimeout at planned duration: Rejected — would drift if the timer is paused/resumed; tick comparison is more reliable.

## Decision 6: Duration Input UI in Builder

**Decision**: Add an optional duration input alongside the existing rep target UI in the setlist builder. Use a simple minutes input (1–60 range, whole minutes only for v1) with "Add duration" / "Remove" pattern matching the rep target UX.

**Rationale**: Matching the existing rep target pattern keeps the UI consistent and predictable (constitution principle VI: Predictable Navigation). Whole minutes are sufficient for v1 — musicians think in minutes, not seconds, when allocating practice time.

**Alternatives considered**:
- Minutes + seconds input: More precise but adds complexity for v1. Can be enhanced later.
- Preset buttons (5/10/15 min): Limited. Free input with sensible range is more flexible.
- Slider: Harder to set precise values, not consistent with existing patterns.

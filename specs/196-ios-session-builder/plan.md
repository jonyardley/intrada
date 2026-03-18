# Implementation Plan: iOS Session Builder

**Branch**: `196-ios-session-builder` | **Date**: 2026-03-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/196-ios-session-builder/spec.md`

## Summary

Build the iOS session builder — a tap-to-queue interface where musicians select library items to construct a practice setlist. The Crux core already provides all session builder events and ViewModel fields. This is a pure iOS shell feature: compose SwiftUI views that dispatch Crux events and render the ViewModel. iPhone uses a library list with a sticky bottom bar and a bottom sheet for setlist details. iPad uses a split-view layout with library left and setlist right.

## Technical Context

**Language/Version**: Swift 6.0, iOS 17.0+
**Primary Dependencies**: SwiftUI, UniFFI (CoreFfi), BCS serialization (auto-generated types)
**Storage**: N/A (all persistence via Crux core HTTP effects → REST API → Turso; crash recovery via UserDefaults)
**Testing**: `just ios-swift-check` (compile), `just ios-smoke-test` (runtime), SwiftUI Previews
**Target Platform**: iOS 17.0+ (iPhone + iPad)
**Project Type**: Mobile (iOS shell for existing Crux core)
**Performance Goals**: 60fps scrolling, <100ms event dispatch round-trip, session building in under 10 seconds (3 taps + 1 tap)
**Constraints**: Pure shell — zero business logic in Swift. All state flows through Crux ViewModel. No hand-written type definitions.
**Scale/Scope**: 4 new SwiftUI components, 2 new views, 1 view modifier, ~15 files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | Single responsibility per component. Consistent with existing iOS patterns. |
| II. Testing Standards | ✅ PASS | Compile check + smoke test + previews. Core logic tested in Rust. |
| III. UX Consistency | ✅ PASS | Uses existing design system tokens, components, and glassmorphism language. Cross-platform parity with web. |
| IV. Performance | ✅ PASS | Library list uses LazyVStack for efficient scrolling. No redundant API calls — ViewModel provides all data. |
| V. Architecture Integrity | ✅ PASS | Pure shell. Events dispatched to Crux core. ViewModel is read-only projection. Zero business logic in Swift. |
| VI. Inclusive Design | ✅ PASS | Minimum-friction builder (3 taps to build, 1 to start). Visible time cues in bottom bar. Predictable navigation via state-driven rendering. |

## Project Structure

### Documentation (this feature)

```text
specs/196-ios-session-builder/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (speckit.tasks)
```

### Source Code (repository root)

```text
ios/Intrada/
├── Views/
│   └── Practice/
│       ├── PracticeTabRouter.swift       # State-driven switch on session_status
│       ├── SessionBuilderView.swift      # Main builder (iPhone + iPad adaptive)
│       ├── SessionBuilderListContent.swift  # Library list with tap-to-queue rows
│       └── SetlistSheetContent.swift     # Bottom sheet / right panel content
├── Components/
│   ├── LibraryQueueRow.swift            # Tappable library row with toggle state
│   ├── SetlistEntryRow.swift            # Compact entry with drag handle + progressive disclosure
│   ├── StickyBottomBar.swift            # iPhone bottom bar (count + Start Session)
│   └── SetlistSheet.swift               # iPhone bottom sheet container
├── DesignSystem/
│   └── Modifiers/
│       └── (existing modifiers reused)
└── Navigation/
    └── MainTabView.swift                # Updated: Practice tab indicator
```

**Structure Decision**: Follows the existing iOS app structure. Practice views go in `Views/Practice/` (matching `Views/Library/` pattern). New components go in `Components/` (one file per component). No new design system tokens needed — all colours and spacing already defined.

## Complexity Tracking

No constitution violations. No complexity justification needed.

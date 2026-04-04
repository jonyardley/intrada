# Implementation Plan: iOS Routines

**Branch**: `199-ios-routines` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)

## Summary

Build iOS routines management — list, detail, edit, delete, load into session builder, save as routine. Replaces the Routines tab placeholder and wires the existing "Load Routine" placeholder in SetlistSheetContent. **Shell-only** — all events and ViewModel types exist.

## Technical Context

**Language/Version**: Swift 6.0, iOS 17.0+
**Primary Dependencies**: SwiftUI, UniFFI (CoreFfi), BCS serialization (auto-generated SharedTypes)
**Storage**: N/A (all persistence via Crux core HTTP effects → REST API → Turso)
**Testing**: `just ios-swift-check`, `just ios-smoke-test`, `just ios-preview-check`
**Target Platform**: iOS 17.0+ (iPhone + iPad)
**Project Type**: Mobile (iOS shell consuming shared Crux core)
**Constraints**: No core/API changes. Reuse existing components.
**Scale/Scope**: 4 new views, 1 new component, wire 2 integration points

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | Standard CRUD views following established patterns |
| II. Testing Standards | ✅ PASS | Compile + smoke + preview checks |
| III. UX Consistency | ✅ PASS | Follows rules 10–17: CardView, ButtonView, EmptyStateView, NavigationSplitView |
| IV. Performance | ✅ PASS | LazyVStack for lists, no new API calls |
| V. Architecture Integrity | ✅ PASS | Pure shell. Events via `core.update()` |
| VI. Inclusive Design | ✅ PASS | Clear actions, confirmation on delete, predictable navigation |

## Project Structure

```text
ios/Intrada/
├── Views/Routines/
│   ├── RoutineListView.swift          # NEW: Routines tab root (list + empty state)
│   ├── RoutineDetailView.swift        # NEW: Routine detail (name + item list)
│   └── RoutineEditView.swift          # NEW: Edit routine (rename, reorder, add/remove)
├── Components/
│   └── RoutineSaveForm.swift          # NEW: Collapsible save-as-routine name input
├── Views/Practice/
│   ├── SetlistSheetContent.swift      # UPDATE: Wire "Load Routine" button
│   └── SessionSummaryView.swift       # UPDATE: Add save-as-routine form
├── Navigation/
│   └── MainTabView.swift              # UPDATE: Replace Routines tab placeholder
```

## Architecture

### Event Mapping

| User Action | Core Event |
|-------------|------------|
| Delete routine + confirm | `.routine(.deleteRoutine(id:))` |
| Save edits | `.routine(.updateRoutine(id:, name:, entries:))` |
| Load into setlist | `.routine(.loadRoutineIntoSetlist(routineId:))` |
| Save setlist as routine | `.routine(.saveBuildingAsRoutine(name:))` |
| Save summary as routine | `.routine(.saveSummaryAsRoutine(name:))` |

### iPad Adaptation

- **RoutineListView**: NavigationSplitView — routine list sidebar, detail in main pane
- **RoutineEditView**: Same pattern as Library edit — form fills available width

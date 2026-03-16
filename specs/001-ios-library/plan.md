# Implementation Plan: iOS Library — Browse, Search & Manage Repertoire

**Branch**: `001-ios-library` | **Date**: 2026-03-16 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-ios-library/spec.md`

## Summary

Build the iOS Library feature — the first full feature screen in the iOS app. Users can browse, search, filter, view details, add, edit, and delete library items (pieces and exercises). The implementation is a **pure SwiftUI shell** that renders the existing Crux ViewModel and dispatches Events — no new business logic, HTTP handling, or domain types needed. The Crux core already has complete library CRUD, validation, and optimistic updates. iPad uses `NavigationSplitView` for a sidebar + detail split layout; iPhone uses `NavigationStack` with push navigation.

## Technical Context

**Language/Version**: Swift 6.0, iOS 17.0+
**Primary Dependencies**: SwiftUI, ClerkKit, UniFFI (CoreFfi), BCS serialization (auto-generated)
**Storage**: N/A (all persistence via Crux core HTTP effects → REST API → Turso)
**Testing**: `just ios-swift-check` (compile), `just ios-smoke-test` (runtime), `just ios-preview-check` (previews)
**Target Platform**: iOS 17.0+ (iPhone + iPad, portrait + landscape)
**Project Type**: Mobile shell (Crux architecture — shell is a dumb I/O executor)
**Performance Goals**: 60fps list scrolling with 100+ items, <2s initial load
**Constraints**: All domain state in Crux Model; shell owns only ephemeral UI state (form fields, loading flags). Zero hand-written type definitions — all from typegen.
**Scale/Scope**: 7 new SwiftUI views, 5 new components, ~1500 lines of Swift

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | One component per file, design tokens only, no raw colours |
| II. Testing Standards | PASS | Compile check + smoke test + preview check for all views |
| III. UX Consistency | PASS | Mirrors web component library; uses shared design tokens |
| IV. Performance Requirements | PASS | LazyVStack for list scrolling; no redundant API calls |
| V. Architecture Integrity | PASS | Shell dispatches Events, renders ViewModel — zero I/O in views |
| VI. Inclusive Design | PASS | Predictable navigation, no auto-play, consistent patterns |

## Project Structure

### Documentation (this feature)

```text
specs/001-ios-library/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (ViewModel mapping)
├── quickstart.md        # Phase 1 output (verification steps)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
ios/Intrada/
├── Views/Library/
│   ├── LibraryView.swift           # Root view — NavigationSplitView (iPad) / NavigationStack (iPhone)
│   ├── LibraryListContent.swift    # Scrollable item list with search, filter, empty/loading states
│   ├── ItemDetailView.swift        # Full item detail with practice summary
│   ├── AddItemView.swift           # Create item form (modal sheet on iPad, pushed on iPhone)
│   ├── EditItemView.swift          # Edit item form (modal sheet on iPad, pushed on iPhone)
│   └── LibrarySkeletonView.swift   # Loading skeleton for list and detail
├── Components/
│   ├── LibraryItemRow.swift        # List row: title, composer, type badge, key, tempo, tags
│   ├── TypeTabs.swift              # Segmented control: All / Pieces / Exercises
│   ├── TagInputView.swift          # Chip-based multi-tag input with autocomplete
│   ├── AutocompleteField.swift     # Text field with dropdown suggestions
│   └── ScoreHistoryList.swift      # Practice score history entries
└── Navigation/
    └── MainTabView.swift           # Updated: Library tab routes to LibraryView
```

**Structure Decision**: Mobile shell (Option 3). All new code lives in `ios/Intrada/Views/Library/` for feature views and `ios/Intrada/Components/` for reusable components. Navigation is updated in-place. No Rust changes needed.

## Design Decisions

### D1: NavigationSplitView for iPad, NavigationStack for iPhone

`LibraryView` uses SwiftUI's `NavigationSplitView` which automatically:
- Shows sidebar + detail on iPad (regular width)
- Collapses to a single NavigationStack on iPhone (compact width)

This means **zero duplicate components**. The same `LibraryListContent`, `ItemDetailView`, etc. render in both contexts — SwiftUI handles the layout adaptation.

**Reference**: Pencil mockups — "iPad / Library + Detail (Portrait)", "iPad / Library + Detail (Landscape)"

### D2: Shell-only implementation — no Crux core changes

The Crux core already provides:
- `Event::Item(ItemEvent::Add/Update/Delete)` for all CRUD
- `Event::SetQuery(ListQuery)` for search and type filtering
- `ViewModel.items: Vec<LibraryItemView>` with all display data
- Complete validation in `validation.rs`
- Optimistic updates with HTTP effect processing
- Practice summary data (`ItemPracticeSummary`) on each item

The iOS shell only needs to:
1. Render `ViewModel.items` as a list
2. Dispatch Events when the user interacts
3. Manage ephemeral UI state (form fields, search text, loading flags)

### D3: Form validation — dual layer

1. **Client-side** (Swift): Quick feedback using the same rules as web (`validate_library_form`). Prevents invalid submissions.
2. **Core-side** (Rust): Authoritative validation on `Event::Item(ItemEvent::Add/Update)`. Returns `LibraryError::Validation` if invalid.

The Swift validation mirrors the web's `validation.rs` — same constants imported from the Crux core's generated types aren't available as constants in Swift, so we define a small `LibraryFormValidator` struct with the same rules.

### D4: Overflow menu replaces standalone Delete button

Per the HIG review, the detail view uses a toolbar "..." menu containing Edit and Delete actions, rather than a prominent red Delete button. Delete triggers a `.confirmationDialog()`.

### D5: Autocomplete from ViewModel data

Tag and composer autocomplete suggestions are derived from `ViewModel.items` — no additional API calls. Swift computes unique values client-side, same as the web shell.

## Complexity Tracking

No constitution violations. All work is standard SwiftUI shell implementation following established Crux patterns.

# Implementation Plan: Web — Adopt iOS Layout Patterns

**Branch**: `219-web-ios-layout-parity` | **Date**: 2026-03-21 | **Spec**: [spec.md](spec.md)

## Summary

Update the web UI to match iOS layout patterns: split-view library with sidebar + detail pane on desktop, compact library list rows replacing glassmorphism cards, and a tap-to-queue session builder with sticky bottom bar on mobile. Pure presentation-layer change — no Crux core, API, or data model changes.

## Technical Context

**Language/Version**: Rust stable (1.89.0), Leptos 0.8.x (CSR/WASM)
**Primary Dependencies**: leptos 0.8, leptos_router 0.8, web-sys 0.3, Tailwind CSS v4
**Storage**: N/A (no storage changes)
**Testing**: wasm-bindgen-test 0.3, Playwright E2E
**Target Platform**: WASM (web browser), CSR
**Project Type**: Web (Leptos CSR shell within Crux architecture)
**Performance Goals**: UI interactions feel instant; library list renders 100+ items without jank
**Constraints**: Must preserve existing design token system; mobile (<768px) behaviour unchanged for library
**Scale/Scope**: ~6 modified views, ~3 new components, ~2 updated components

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | New components follow single responsibility; extracted into `components/` |
| II. Testing Standards | PASS | E2E tests cover library and session flows; updated for new selectors |
| III. UX Consistency | PASS | This IS the consistency improvement — aligning web with iOS patterns |
| IV. Performance Requirements | PASS | Compact rows improve render density; no new API calls |
| V. Architecture Integrity | PASS | Pure shell change — core and API untouched |
| VI. Inclusive Design | PASS | Reduces decisions (auto-select first item), predictable split-view layout |

## Project Structure

### Documentation (this feature)

```text
specs/219-web-ios-layout-parity/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── checklists/
│   └── requirements.md  # Spec quality checklist
├── data-model.md        # N/A — no data changes
├── quickstart.md        # Verification steps
└── contracts/           # N/A — no API changes
```

### Source Code (repository root)

```text
crates/intrada-web/src/
├── components/
│   ├── mod.rs                    # Component exports (update)
│   ├── library_list_row.rs       # NEW — compact library row
│   ├── split_view_layout.rs      # NEW — responsive sidebar + detail container
│   ├── sticky_bottom_bar.rs      # NEW — mobile session builder bottom bar
│   ├── setlist_builder.rs        # UPDATE — tap-to-queue + split-view integration
│   ├── setlist_entry_row.rs      # UPDATE — progressive disclosure
│   └── [existing components]     # Unchanged
├── views/
│   ├── library.rs                # UPDATE — split-view layout, compact rows
│   ├── detail.rs                 # UPDATE — renders inside split-view detail pane
│   ├── add_item.rs               # UPDATE — renders inside detail pane on desktop
│   ├── edit_item.rs              # UPDATE — renders inside detail pane on desktop
│   ├── session_new.rs            # UPDATE — tap-to-queue builder layout
│   └── [other views]             # Unchanged
└── app.rs                        # UPDATE — routing for split-view library

intrada-web/input.css              # UPDATE — new utility classes if needed

e2e/tests/
├── library.spec.ts               # UPDATE — new selectors for split-view
├── sessions.spec.ts              # UPDATE — new selectors for builder
└── navigation.spec.ts            # UPDATE — split-view desktop nav
```

**Structure Decision**: All changes are within the existing `intrada-web` crate. No new crates, no core changes.

## Key Technical Decisions

### 1. Split-View Implementation

**Approach**: CSS flexbox layout with Tailwind responsive classes. On `md:` (≥768px), render a `flex` container with fixed-width sidebar and flexible detail pane. On mobile, render stacked full-page navigation using Leptos router.

**Why not a media query signal?** Leptos CSR doesn't have built-in breakpoint signals. Using CSS-only responsive layout (hidden/shown via Tailwind `hidden md:flex`) avoids JavaScript-based viewport detection and keeps the split-view purely declarative.

**Router integration**: The library route changes from `/library` → `/library/:id?`. On desktop, the sidebar always renders; the detail pane renders the matched `:id` or auto-selects the first item. On mobile, `/library` shows the list and `/library/:id` shows the detail page.

### 2. LibraryListRow vs LibraryItemCard

**Approach**: Create a new `LibraryListRow` component for list contexts. Keep `LibraryItemCard` available for any non-list usage (e.g., design catalogue showcase). The row component accepts an `is_selected` prop for session builder context.

### 3. Tap-to-Queue State Management

**Approach**: The setlist state already lives in the Crux core ViewModel (`session_entries`). Toggling items uses existing `AddToSetlist` / `RemoveFromSetlist` events. The "selected" visual state is derived by checking if an item's ID exists in the current setlist entries. No new state management needed.

### 4. Mobile Bottom Sheet

**Approach**: CSS-based slide-up panel using `transform: translateY()` with a transition. Toggled by a Leptos signal. No external sheet/modal library needed — Tailwind + a small component handles it.

### 5. Drag-to-Reorder

**Approach**: Reuse the existing drag-and-drop implementation from `SetlistBuilder`. The drag handles and drop indicators already work. They just need to render within the new split-view setlist panel layout.

## Complexity Tracking

No constitution violations. No complexity justifications needed.

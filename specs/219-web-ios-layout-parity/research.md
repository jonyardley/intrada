# Research: Web — Adopt iOS Layout Patterns

**Date**: 2026-03-21
**Feature**: `219-web-ios-layout-parity`

## R1: Leptos Split-View with Router Integration

**Decision**: Use CSS-only responsive layout with nested routes.

**Rationale**: Leptos 0.8 CSR uses `leptos_router` for client-side routing. The library route becomes `/library/:id?` where:
- On desktop (≥768px): sidebar list always renders alongside the detail pane. The `:id` param drives which item is shown. If absent, auto-select first item.
- On mobile (<768px): `/library` renders the list view only; `/library/:id` renders the detail view only. Standard stacked navigation.

The responsive switch is handled entirely with Tailwind classes (`hidden md:block`, `md:hidden`) — no JavaScript viewport detection needed. Both the list and detail components render in the DOM on desktop; CSS controls visibility on mobile.

**Alternatives considered**:
- JavaScript `matchMedia` signal: Adds complexity, potential flicker on initial render. Rejected.
- Separate desktop/mobile route trees: Over-engineering — CSS handles it cleanly. Rejected.
- `ResizeObserver` component: Unnecessary for a simple breakpoint switch. Rejected.

## R2: Slide-Up Sheet for Mobile Session Builder

**Decision**: CSS transform-based slide-up panel, toggled by Leptos signal.

**Rationale**: The mobile session builder needs a sheet to show the full setlist when the user taps the bottom bar. A simple CSS approach using `transform: translateY(100%)` → `translateY(0)` with `transition` provides a smooth animation without any external library. A backdrop overlay captures tap-to-dismiss.

**Alternatives considered**:
- External modal/sheet library (e.g., `web-sys` dialog element): Adds dependency; `<dialog>` doesn't provide the slide-up sheet UX. Rejected.
- Full-page navigation to setlist: Breaks the single-screen builder flow. Rejected.
- Always-visible setlist below the library: Cluttered on mobile, defeats the purpose. Rejected.

## R3: Compact Row Component Design

**Decision**: New `LibraryListRow` component, separate from `LibraryItemCard`.

**Rationale**: The row has different layout requirements (horizontal, compact, divider-separated) than the card (vertical, padded, glassmorphism border). Creating a new component avoids overloading `LibraryItemCard` with conditional layout logic. The row accepts `is_selected: bool` for session builder context (accent left bar + check icon). Both components consume the same `LibraryItemView` from the ViewModel.

**Alternatives considered**:
- Add a `compact` prop to `LibraryItemCard`: Would make the component complex with divergent layout paths. Rejected.
- Use the card but strip styling: Loses the semantic distinction. Rejected.

## R4: E2E Test Impact

**Decision**: Update existing E2E tests to account for new selectors and layout.

**Rationale**: The Playwright tests use text selectors and CSS selectors. The split-view changes the DOM structure but not the content. Tests need:
- Updated selectors for library items (rows instead of cards)
- Desktop viewport tests for split-view behaviour
- Mobile viewport tests to confirm stacked nav still works
- Session builder tests updated for tap-to-queue flow

No new test files needed — existing `library.spec.ts`, `sessions.spec.ts`, and `navigation.spec.ts` cover the flows.

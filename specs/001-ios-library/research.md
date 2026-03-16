# Research: iOS Library

**Feature**: 001-ios-library | **Date**: 2026-03-16

## R1: NavigationSplitView vs NavigationStack for iPad split layout

**Decision**: Use `NavigationSplitView` with two columns (sidebar + detail)

**Rationale**: `NavigationSplitView` is Apple's recommended approach for master-detail interfaces (WWDC22, HIG). It automatically adapts:
- iPad regular width → persistent sidebar + detail pane
- iPhone compact width → collapses to `NavigationStack` push navigation
- Landscape → wider sidebar, more detail space
- No conditional `if horizontalSizeClass == .regular` branching needed

**Alternatives considered**:
- Manual `@Environment(\.horizontalSizeClass)` branching — rejected because it duplicates what NavigationSplitView does natively, and requires maintaining two navigation flows
- Separate iPad/iPhone view hierarchies — rejected because it doubles the component count and maintenance burden

**Implementation pattern**:
```swift
NavigationSplitView {
    LibraryListContent(selection: $selectedItemId)
} detail: {
    if let id = selectedItemId {
        ItemDetailView(itemId: id)
    } else {
        ContentUnavailableView("Select an Item", ...)
    }
}
```

## R2: Crux core readiness — what already exists vs what's needed

**Decision**: Zero Rust/core changes required. Shell-only implementation.

**Rationale**: The Crux core already provides everything the iOS library needs:

| Capability | Status | Core location |
|-----------|--------|---------------|
| Library CRUD events | Complete | `domain/item.rs` — `ItemEvent::Add/Update/Delete` |
| List query/filter | Complete | `app.rs` — `Event::SetQuery(ListQuery)` with text + type filter |
| ViewModel projection | Complete | `model.rs` — `LibraryItemView` with all display fields |
| Validation | Complete | `validation.rs` — all constants and rules |
| Practice summary | Complete | `model.rs` — `ItemPracticeSummary` on each LibraryItemView |
| Optimistic updates | Complete | `domain/item.rs` — immediate Model updates before HTTP |
| HTTP effects | Complete | `http.rs` — fetch, create, update, delete endpoints |
| Generated Swift types | Complete | `SharedTypes.swift` — all structs and enums |
| Error handling | Complete | `Event::LoadFailed` populates `model.last_error` |

**Alternatives considered**:
- Adding iOS-specific events to core (e.g., `Event::iOSLibraryLoaded`) — rejected because the existing events are platform-agnostic by design
- Adding search as a separate API endpoint — rejected because `ListQuery` already handles client-side filtering from the ViewModel

## R3: Form validation strategy in Swift

**Decision**: Mirror web's client-side validation as a Swift `LibraryFormValidator` struct

**Rationale**: The Crux core validates on event dispatch, but waiting for a round-trip to show validation errors creates poor UX. The web shell has a `validate_library_form()` function that checks the same rules client-side. The iOS shell should do the same.

Constants to mirror (from `intrada-core/src/validation.rs`):
- `MAX_TITLE = 500`, `MAX_COMPOSER = 200`, `MAX_NOTES = 5000`
- `MAX_TAG = 100`, `MAX_TEMPO_MARKING = 100`
- `MIN_BPM = 1`, `MAX_BPM = 400`
- Title required; composer required for pieces; BPM range if provided

**Alternatives considered**:
- Importing validation constants from generated types — rejected because facet typegen doesn't export Rust constants, only types
- Relying solely on core validation — rejected because it requires a full event dispatch round-trip and doesn't show errors until after submission

## R4: Delete UX — overflow menu vs prominent button

**Decision**: Toolbar "..." overflow menu with Edit and Delete actions. Delete triggers `.confirmationDialog()`.

**Rationale**: Apple HIG recommends destructive actions be placed in contextual menus rather than as primary UI elements. A red "Delete" button given equal visual weight to content creates anxiety and increases accidental taps. The "..." menu follows the pattern used by Apple's own apps (Notes, Reminders, Music).

**Alternatives considered**:
- Swipe-to-delete on list rows — suitable as an addition but shouldn't be the only delete path (not discoverable for all users)
- Prominent red Delete button at bottom of detail — rejected after HIG review (already updated in Pencil designs)

## R5: Autocomplete data source

**Decision**: Derive suggestions from `ViewModel.items` in Swift, same as web shell

**Rationale**: The ViewModel already contains all library items with their tags and composers. Computing unique values is trivial and avoids additional API calls. The web shell uses `unique_tags()` and `unique_composers()` helper functions on the items array.

Swift implementation:
```swift
var uniqueComposers: [String] {
    Set(core.viewModel.items.compactMap { $0.subtitle?.value }).sorted()
}
var uniqueTags: [String] {
    Set(core.viewModel.items.flatMap { $0.tags }).sorted()
}
```

**Alternatives considered**:
- Dedicated autocomplete API endpoint — rejected (over-engineered for the data volume)
- Caching suggestions in a separate model — rejected (ViewModel is already reactive)

## R6: List performance with 100+ items

**Decision**: Use `LazyVStack` inside `ScrollView` (or `List` with custom styling)

**Rationale**: SwiftUI's `LazyVStack` only instantiates visible rows, providing smooth 60fps scrolling even with hundreds of items. This matches SC-008 (smooth scrolling with 100+ items).

**Alternatives considered**:
- Plain `VStack` — rejected because it instantiates all rows upfront, causing lag with large lists
- `UICollectionView` via UIViewRepresentable — rejected as unnecessary complexity when LazyVStack handles the use case
- `List` — viable but harder to style with glassmorphism aesthetic; LazyVStack in ScrollView gives full visual control

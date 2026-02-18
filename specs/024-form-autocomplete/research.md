# Research: Form Autocomplete

**Feature**: 024-form-autocomplete
**Date**: 2026-02-18

## R1: Autocomplete Component Pattern in Leptos CSR

**Decision**: Build a single reusable `Autocomplete` Leptos component that accepts a suggestions source signal and mode (single-select for composer, multi-select with chips for tags).

**Rationale**: The codebase already uses context-based `RwSignal<ViewModel>` for reactive state. The ViewModel exposes `items: Vec<LibraryItemView>` with `.tags` and `.subtitle` (composer) fields. A derived signal can extract unique values from this data without additional API calls or storage. Leptos `Signal::derive` naturally recomputes when library data changes.

**Alternatives considered**:
- Separate TagInput and ComposerInput components: Rejected — duplicates autocomplete logic (filtering, keyboard nav, dropdown). Better to parameterise a shared component.
- Server-side autocomplete endpoint: Rejected — library data is already loaded client-side. Adding a round-trip would violate the "no perceptible delay" requirement (SC-005) and is unnecessary for the expected data scale.

## R2: Suggestion Extraction from ViewModel

**Decision**: Extract unique tags and composers as derived signals computed from `view_model.get().items`.

**Rationale**: The `LibraryItemView` struct contains `tags: Vec<String>` and `subtitle: String` (which holds the composer for pieces). By deriving these at the component level using `Signal::derive`, the suggestion lists stay reactive and update automatically when items change. Case-insensitive deduplication preserves first-seen casing.

**Alternatives considered**:
- Precompute and cache in a separate signal at app level: Rejected — adds complexity; derived signals are efficient for the expected library size (<10k items).
- Store a separate normalised tag table in the API: Rejected — over-engineering for client-side autocomplete.

## R3: Keyboard Navigation Pattern

**Decision**: Use `on:keydown` event handler on the input element with `ev.key()` matching, maintaining a `RwSignal<Option<usize>>` for the highlighted index.

**Rationale**: The codebase already uses this exact pattern in `TypeTabs` for arrow key navigation with `web_sys::HtmlElement` focus management. Arrow keys navigate, Enter/Tab select, Escape dismisses. Wrapping at list boundaries is standard autocomplete behaviour.

**Alternatives considered**:
- Third-party autocomplete library: Rejected — no suitable WASM/Leptos-compatible library exists; the pattern is straightforward to implement from existing codebase patterns.

## R4: Click-Outside Dismissal

**Decision**: Use a `on:focusout` handler on the autocomplete wrapper with a short delay to allow click events on suggestions to fire first.

**Rationale**: Pure `focusout` on the container catches both tab-away and click-outside. A brief delay (e.g. `setTimeout` via `gloo-timers` or `set_timeout`) prevents the dropdown from closing before a suggestion click registers. This is the standard web pattern for combobox dismissal.

**Alternatives considered**:
- Global document click listener: Works but requires cleanup and is more complex. `focusout` is simpler and already part of the DOM event model.

## R5: Tag Chip Display Pattern

**Decision**: Replace the current comma-separated `TextField` for tags with a custom `TagInput` component that renders selected tags as chips with remove buttons, followed by an inline text input for new tag entry.

**Rationale**: Chip-based tag inputs are the industry standard for multi-value fields. They provide clear visual feedback of what's selected, easy removal of individual tags, and natural integration with autocomplete. The current comma-separated input requires users to mentally parse the field to see what's there.

**Alternatives considered**:
- Keep comma-separated input with autocomplete only: Rejected — doesn't solve the visibility problem; chips are universally expected for tag-style inputs.

## R6: Architecture Compliance

**Decision**: All autocomplete logic lives in `intrada-web` (the shell). No changes to `intrada-core` or `intrada-api` are needed.

**Rationale**: This feature is purely UI — filtering suggestions, managing dropdown state, and handling keyboard events are all shell concerns. The core already provides tags and composer data via the ViewModel. The existing `parse_tags` helper in `intrada-web/src/helpers.rs` handles tag string parsing. No new events, effects, or domain logic are required in core. This fully preserves the Crux pure-core pattern (Constitution Principle V).

**Alternatives considered**:
- Add autocomplete suggestion logic to core: Rejected — filtering a flat list for display is a view concern, not business logic. Adding it to core would violate "Pure Core" (no I/O, no platform dependencies) and add unnecessary complexity.

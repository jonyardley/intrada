# Tasks: Form Autocomplete

**Input**: Design documents from `/specs/024-form-autocomplete/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Web shell**: `crates/intrada-web/src/`
- **Components**: `crates/intrada-web/src/components/`
- **Views**: `crates/intrada-web/src/views/`
- **Helpers**: `crates/intrada-web/src/helpers.rs`

---

## Phase 1: Setup

**Purpose**: Register new component files and add helper functions for suggestion extraction

- [x] T001 Add `autocomplete` and `tag_input` module declarations to `crates/intrada-web/src/components/mod.rs` and re-export the new components
- [x] T002 [P] Add suggestion extraction helper functions (`unique_tags`, `unique_composers`) to `crates/intrada-web/src/helpers.rs` — collect all tags from `Vec<LibraryItemView>`, deduplicate case-insensitively preserving first-seen casing, sort alphabetically; for composers, extract `subtitle` from pieces and from exercises only when `category` is `None`, deduplicate and sort
- [x] T003 [P] Add suggestion filtering helper function (`filter_suggestions`) to `crates/intrada-web/src/helpers.rs` — accepts a list of suggestions, input text, exclusion list, and max count; filters case-insensitively; ranks prefix matches before substring matches; returns up to `max` results

**Checkpoint**: Helper functions exist and are unit-testable. New module files can be created.

---

## Phase 2: Foundational (Autocomplete Component)

**Purpose**: Build the reusable `Autocomplete` dropdown component that both TagInput and composer field will use

**⚠️ CRITICAL**: Both US1 and US2 depend on this component

- [x] T004 Create `Autocomplete` component in `crates/intrada-web/src/components/autocomplete.rs` — implement the component accepting props per contracts/components.md: `id`, `suggestions: Signal<Vec<String>>`, `value: RwSignal<String>`, `on_select: Callback<String>`, optional `placeholder`, `min_chars` (default 2), `max_suggestions` (default 8), `exclude: Signal<Vec<String>>`; use `filter_suggestions` helper from T003 to derive filtered list; render a text input with a positioned dropdown list below it; show dropdown only when filtered list is non-empty and input length ≥ `min_chars`; on click of a suggestion item, call `on_select` with the selected value and close dropdown; close dropdown on `focusout` with a short delay (use `gloo-timers` or `wasm_bindgen_futures::spawn_local` with `gloo_timers::future::sleep`) to allow click events to fire first
- [x] T005 Add ARIA attributes to `Autocomplete` component in `crates/intrada-web/src/components/autocomplete.rs` — input gets `role="combobox"`, `aria-autocomplete="list"`, `aria-expanded` (true when dropdown open), `aria-activedescendant` (id of highlighted item); dropdown `ul` gets `role="listbox"`; each suggestion `li` gets `role="option"` and unique `id`; highlighted item gets `aria-selected="true"`
- [x] T006 Style `Autocomplete` dropdown in `crates/intrada-web/src/components/autocomplete.rs` — use Tailwind classes consistent with existing glassmorphism design: dropdown gets `bg-gray-800/90 backdrop-blur-sm border border-white/10 rounded-lg shadow-lg`; suggestion items get `px-3 py-2 text-sm text-gray-200 cursor-pointer`; highlighted item gets `bg-indigo-600/50`; position dropdown absolutely below the input using `relative`/`absolute` positioning

**Checkpoint**: Autocomplete component renders a dropdown with filtered suggestions on typing. Click selection works. ARIA attributes present.

---

## Phase 3: User Story 1 — Tag Autocomplete on Library Forms (Priority: P1) 🎯 MVP

**Goal**: Replace comma-separated tag input with chip-based TagInput component that shows autocomplete suggestions from existing library tags

**Independent Test**: Open add/edit form, type in tags field, verify suggestions appear; select one to add as chip; type new tag and press comma to create; remove chip by clicking ×

### Implementation for User Story 1

- [x] T007 [US1] Create `TagInput` component in `crates/intrada-web/src/components/tag_input.rs` — implement per contracts/components.md: accepts `id`, `tags: RwSignal<Vec<String>>`, `available_tags: Signal<Vec<String>>`, `field_name`, `errors`; render each tag in `tags` as a chip/badge (`inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-indigo-600/30 text-indigo-200 text-xs`) with a remove button (×); after chips, render an inline text input for new tag entry; derive an `exclude` signal from `tags` to pass to the embedded `Autocomplete` component
- [x] T008 [US1] Wire tag creation in `TagInput` in `crates/intrada-web/src/components/tag_input.rs` — on `on:keydown` for comma or Enter key: trim input value, if non-empty and not already in tags, push to `tags` signal, clear input; on `on_select` callback from `Autocomplete`: add selected tag to `tags` signal, clear input; on paste event (`on:paste`): parse comma-separated values, add each unique trimmed tag to `tags`, clear input
- [x] T009 [US1] Wire tag removal in `TagInput` in `crates/intrada-web/src/components/tag_input.rs` — each chip's × button calls a handler that removes that tag from `tags` signal by value; add `FormFieldError` display for the `field_name` using the same pattern as `TextField`
- [x] T010 [US1] Replace tag `TextField` with `TagInput` in add form `crates/intrada-web/src/views/add_form.rs` — replace the `<TextField id="add-tags" .../>` with `<TagInput id="add-tags" tags=tags_signal available_tags=all_tags_signal field_name="tags" errors=errors />`; change `tags_input: RwSignal<String>` to `tags: RwSignal<Vec<String>>`; derive `all_tags_signal` from `view_model` using `unique_tags` helper; update form submission to read directly from `tags.get()` instead of `parse_tags(tags_input.get())`
- [x] T011 [US1] Replace tag `TextField` with `TagInput` in edit form `crates/intrada-web/src/views/edit_form.rs` — same changes as T010; pre-populate `tags` signal from the existing item's `tags` field in the ViewModel; update form submission to read from `tags.get()`
- [x] T012 [US1] Update form validation in `crates/intrada-web/src/validation.rs` — add or update `validate_library_form` to accept `&[String]` for tags directly (instead of parsing from a comma-separated string); ensure existing `validate_tags` from intrada-core is still called on the Vec

**Checkpoint**: Tag input shows chips, autocomplete suggests existing tags, new tags can be created, chips can be removed. Both add and edit forms work.

---

## Phase 4: User Story 2 — Composer Autocomplete on Library Forms (Priority: P1)

**Goal**: Add autocomplete suggestions to the composer text field on both add and edit forms, sourced from existing composers in the library

**Independent Test**: Open add form for a piece, type partial composer name, verify suggestions appear from existing library; select one to populate field; type new name and verify it's accepted as-is

### Implementation for User Story 2

- [x] T013 [P] [US2] Create `AutocompleteTextField` wrapper component in `crates/intrada-web/src/components/autocomplete.rs` (or a new section of the same file) — a thin wrapper that combines `Autocomplete` behaviour with `TextField`-like props (`id`, `label`, `value: RwSignal<String>`, `required`, `placeholder`, `field_name`, `errors`); on `on_select`, set `value` to the selected suggestion; render label, the autocomplete input, and `FormFieldError` display; this gives composer the same look as other form fields but with suggestions
- [x] T014 [US2] Replace composer `TextField` with `AutocompleteTextField` in add form `crates/intrada-web/src/views/add_form.rs` — derive `all_composers_signal` from `view_model` using `unique_composers` helper; replace both the Piece composer `<TextField>` and Exercise composer `<TextField>` with `<AutocompleteTextField>` passing `suggestions=all_composers_signal`; keep `required=true` for Piece, `required=false` for Exercise
- [x] T015 [US2] Replace composer `TextField` with `AutocompleteTextField` in edit form `crates/intrada-web/src/views/edit_form.rs` — same changes as T014; derive `all_composers_signal` from `view_model`; replace composer `<TextField>` with `<AutocompleteTextField>`

**Checkpoint**: Composer field shows suggestions from existing library on both add and edit forms. Free-text entry still works. Both Piece and Exercise forms have autocomplete.

---

## Phase 5: User Story 3 — Keyboard Navigation (Priority: P2)

**Goal**: Full keyboard navigation for the autocomplete dropdown: arrow keys to navigate, Enter/Tab to select, Escape to dismiss

**Independent Test**: Type to trigger suggestions, use arrow keys to move highlight, press Enter to select, press Escape to dismiss — all without mouse

### Implementation for User Story 3

- [x] T016 [US3] Add keyboard navigation state to `Autocomplete` component in `crates/intrada-web/src/components/autocomplete.rs` — add `RwSignal<Option<usize>>` for highlighted index; reset to `None` whenever the filtered suggestion list changes; on `on:keydown` for ArrowDown: increment index (wrap to 0 at end); on ArrowUp: decrement index (wrap to end at 0); on Enter or Tab when an item is highlighted: call `on_select` with highlighted item, close dropdown, prevent default; on Escape: close dropdown, reset highlight, keep focus on input
- [x] T017 [US3] Add visual highlight for keyboard-selected item in `crates/intrada-web/src/components/autocomplete.rs` — apply `bg-indigo-600/50` class to the suggestion `li` at the highlighted index; update `aria-activedescendant` on the input to point to the highlighted item's `id`; scroll highlighted item into view if dropdown is scrollable

**Checkpoint**: Full keyboard navigation works in both tag and composer autocomplete fields.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup, testing, and validation

- [x] T018 Run `cargo test` across workspace and fix any compilation or test failures introduced by the new components and form changes
- [x] T019 Run `cargo clippy` and `cargo fmt` — resolve all warnings and formatting issues
- [x] T020 Run quickstart.md verification steps end-to-end — walk through all 5 verification sections and confirm each passes
- [x] T021 Verify edge cases: empty library (no suggestions shown), paste comma-separated tags, 1-character input (no dropdown), all tags already selected (no dropdown)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on T002 and T003 from Setup (helper functions) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational (Autocomplete component T004–T006)
- **US2 (Phase 4)**: Depends on Foundational (Autocomplete component T004–T006); can run in parallel with US1
- **US3 (Phase 5)**: Depends on Autocomplete component (T004); can be done after or in parallel with US1/US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Tag Autocomplete)**: Depends on Foundational only — no dependency on US2 or US3
- **US2 (Composer Autocomplete)**: Depends on Foundational only — no dependency on US1 or US3
- **US3 (Keyboard Navigation)**: Depends on Autocomplete component existing (T004) — enhances it; no dependency on US1 or US2

### Within Each User Story

- Component creation before form integration
- Add form before edit form (same pattern, applied twice)
- Validation update after form changes

### Parallel Opportunities

- T002 and T003 (helpers) can run in parallel
- T007, T008, T009 (TagInput internals) are sequential within the same file
- T013 (AutocompleteTextField) can run in parallel with T007–T012 (different files)
- US1 and US2 can run in parallel after Foundational is complete

---

## Parallel Example: After Foundational

```bash
# US1 and US2 can start simultaneously:
# Developer A: T007 → T008 → T009 → T010 → T011 → T012 (Tag input)
# Developer B: T013 → T014 → T015 (Composer autocomplete)

# US3 can start as soon as T004 exists:
# Developer C: T016 → T017 (Keyboard navigation)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational — Autocomplete component (T004–T006)
3. Complete Phase 3: US1 — Tag autocomplete (T007–T012)
4. **STOP and VALIDATE**: Test tag autocomplete independently on add and edit forms
5. Deploy/demo if ready — tags already deliver significant UX value

### Incremental Delivery

1. Setup + Foundational → Autocomplete component ready
2. Add US1 (Tags) → Test → Deploy (MVP!)
3. Add US2 (Composer) → Test → Deploy
4. Add US3 (Keyboard nav) → Test → Deploy
5. Polish → Final validation → Done

---

## Notes

- All changes are in `crates/intrada-web/` — no core or API changes
- The `Autocomplete` component is the foundational building block; TagInput and AutocompleteTextField both compose it
- Existing `parse_tags` helper in `helpers.rs` is retained for backward compatibility but forms now pass `Vec<String>` directly
- Commit after each task or logical group
- Stop at any checkpoint to validate independently

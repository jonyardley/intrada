# The core decides the item form's rules

> Tier 3 spec (rides with the core PR). Issue: #2461, the core half of #2429.

## Problem

The add and edit form holds three rules outside the core:

- The iPhone enables Save when the title is not blank (`canSubmit` in
  `ItemFormModel.swift`).
- The iPhone drops variation rows left blank (`typedLabels`), and both phones
  send variation names only for an exercise, Android mirroring the iPhone
  (#2431).
- Clearing a key on edit cannot be sent from Kotlin. `UpdateItem.key` is
  `Option<Option<Key>>`, and the generated Kotlin type collapses "leave it"
  and "clear it" into one `null`.

Android needs all three to reach parity (#2429), and copying them into Kotlin
would put a domain rule in a shell twice.

## Approach

- **Clearing a key.** `UpdateItem.key` becomes `KeyEdit`, a three-way enum:
  `Keep` (the default), `Clear`, `Set { key }`. Each branch is a distinct
  variant on the bincode wire, so every shell can send it. Composer and notes
  keep `Option<Option<String>>`: an empty string already clears them once
  normalised, so Kotlin can already clear both.
- **Blank variation rows.** The core already trims labels and drops blank
  ones on add and edit (`distinct_ignoring_case`), so the iPhone's filter was
  a second copy. Table tests pin the rule; the shells send every unsaved row.
- **Variation names on a new piece.** `ItemEvent::Add` clears
  `variation_labels` unless the kind is an exercise. The form keeps the rows
  while the musician flips the kind, and the core decides what counts. The
  rule sits in the add handler, not in `normalize_create_item`, so a new
  exercise row on the scaffold path still refuses labels.
- **Save enabled.** `intrada-ffi` exports `item_form_can_save(title)`, beside
  `key_label`, backed by `validation::item_form_can_save`. The form's text is
  UI state and stays in the shell; the rule it is judged by lives in the core.

## Key decisions

- **An enum over an appended `clear_key: bool`.** One field saying one thing;
  a bool beside a nullable key leaves `Set` plus `clear_key` meaningless.
- **A pure export over a form draft in the model.** Holding every keystroke in
  the core would route each one through the bridge for one boolean; the
  `key_*` exports set the precedent for form helpers.
- **No blob change.** `UpdateItem` is an event payload, never stored, and is
  outside the `ActiveSession` graph, so no `BLOB_VERSION` bump.

## Touch points

- `intrada-core`: `KeyEdit` and `UpdateItem.key` (`domain/types.rs`),
  `apply_fields` (`domain/item/edit.rs`), the add handler
  (`domain/item/create.rs`), `item_form_can_save` (`validation.rs`).
- `intrada-ffi`: `item_form_can_save` export.
- Core PR, to keep both builds green: every shell `UpdateItem` moves to
  `KeyEdit` with its behaviour unchanged.
- Screens PR: `ItemFormModel.swift` drops `canSubmit` and `typedLabels`;
  `ItemFormScreen.kt` drops the exercise-only mirror and reads Save from the
  core. A title of only a newline then disables Save on the iPhone, where it
  used to enable it and be refused.

## Verification

- Table tests from typed input: empty, spaces-only and padded titles; blank
  and padded rows among real ones on add and edit; labels on a new piece; a
  key kept, set and cleared on edit.
- Wire pins: `UpdateItem` round-trips with each `KeyEdit` branch on the
  bincode FFI wire, and through `LiveBridge` from Swift and Kotlin in the
  core PR.
- `just check`; `just ios-test`; `just android-test`.

## Out of scope

- The Android key picker, suggestions, related exercises and variation
  reorder: the rest of #2429.
- The chord chart on Android (parked, #2025).

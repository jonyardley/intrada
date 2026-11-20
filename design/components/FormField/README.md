---
name: FormField
status: stable
ios_symbol: FormField
ios_file: ios/Intrada/Views/Components/FormField.swift
ios_snapshots: ComponentSnapshotTests/testFormField.1.png
decisions: T35
---

# FormField

A labelled text field on the item form, as one row inside the form's card.

## When to use

- A text entry on the add and edit forms for a piece or exercise: title, tempo marking, beats per minute, notes.

## When not to use

- A setting whose control is not a text field, such as pills or a stepper: use `FieldCard`.
- A field standing alone on a sheet: use `FieldCard`, which brings its own card.
- A field that offers suggestions, such as the composer: use `AutocompleteField`, which draws the same look with a list under it.
- A group of fields: put each `FormField` in one card with `HairlineDivider` between them, not a card each.

## Variants

- **One line** (the default): use for a short value such as a title.
- **Several lines** (`axis: .vertical`): use for notes. It grows as the text does.
- **With a note** (`note`): use when the field needs a line on what it takes.

## Behaviour

- When the form is refused and the core names this field, the field is faulted: the label turns `danger` and the row takes the `dangerWash`. The reason itself is on `FormErrorBanner` at the top of the form (#1595).
- A field filled from a photographed page shows "From the photo" under it, faint when the read was weak (#1436).
- The keyboard type and capitalisation are the caller's choice; sentences are capitalised by default.

## Content

- The label is `FieldLabel`, in `inkSecondary` or `danger` when faulted.
- The value is `IntradaFont.body` in `ink`, at 16 points or more so iPhone does not zoom on focus. The placeholder uses the system placeholder colour.
- The note is `IntradaFont.small` in `inkSecondary`.
- The row is padded `IntradaSpacing.card` (16) at the sides. Above and below it pads 10 points.
- The row has no fill of its own: it sits in the form's card and shows the card through it.

## Accessibility

- The field is read by its label. A faulted field adds "The message at the top of the form is about this", so a screen reader knows where the reason is.
- The fault and the photo mark are read as the field's hint.
- The label, value and note scale with Dynamic Type.

## Usage

```swift
FormField(
  label: "Title", text: $form.title, placeholder: "Required",
  faulted: form.faults(.title), identifier: "itemForm.title")
```

## Related

`FieldCard`, a labelled setting with its own card. `FieldLabel`, the label. `FormErrorBanner`, the reason a form was refused. `HairlineDivider`, between fields.

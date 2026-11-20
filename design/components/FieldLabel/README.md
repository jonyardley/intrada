---
name: FieldLabel
status: stable
ios_symbol: FieldLabel
ios_file: ios/Intrada/Views/Components/FieldCard.swift
ios_snapshots: ComponentSnapshotTests/testFieldLabel.1.png
decisions: T35
---

# FieldLabel

A field's name, in secondary ink, inside its card above the value.

## When to use

- A field's label inside a card, where `FieldCard` does not fit: a label beside its value (Key ›), a toggle's title, a field in a custom layout.

## When not to use

- A label with a control under it in its own card: use `FieldCard`, which draws this for you.
- A section's name: use `SectionTitle`, so the code says which it is.
- A value the musician reads or sets: values are 17 point in full ink (T35 in [the design principles](../../../docs/design-principles.md)).

## Variants

- **Default:** use for every label.
- **Error** (`tint: IntradaColor.danger`): use while the field is at fault.

## Behaviour

No motion and no haptic. The error colour comes and goes with the field's fault.

## Content

- One line of text: `IntradaFont.label`, 15 point medium, in `inkSecondary`; `danger` in the error state.
- A name in sentence case, no colon. A fact can follow after a middle dot, "Reflection · optional".
- Most labels are one to three words; the longest that ships is "What got in the way".
- The shell writes the labels.

## Accessibility

- Read as plain text, apart from the control it names; the control carries its own spoken label.
- Scales with Dynamic Type.

## Do and don't

- **Do** put it at the top of what it names, inside the card (T35).
- **Don't** pass another colour for emphasis; secondary ink is what makes every label read the same.

## Usage

```swift
FieldLabel("Key")
FieldLabel(label, tint: faulted ? IntradaColor.danger : IntradaColor.inkSecondary)
```

## Related

`FieldCard`, which draws it above a control. `SectionTitle`, which looks the same and names a section.

---
name: FieldCard
status: stable
ios_symbol: FieldCard, fieldCardSurface
ios_file: ios/Intrada/Views/Components/FieldCard.swift
ios_snapshots: ComponentSnapshotTests/testFieldCard.1.png
android_symbol: FieldCard
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/FieldCard.kt
android_snapshots: field-card.png
decisions: T35
---

# FieldCard

One labelled setting on a form or sheet: a card with the label inside, above whatever sets the value.

## When to use

- A setting on a sheet: Beats in the bar, Beat value and Sounds on in the metronome, a section's Kind.
- A field whose control is not a text field, such as pills or a stepper.

## When not to use

- A text field on the item form: use `FormField`, a row inside the form's one card on both apps. It draws the same label and adds the error wash.
- A read-only fact on a details screen: label left, value right, in a card's row (T35 in [the design principles](../../../docs/design-principles.md)).
- A group of fields: one card with `HairlineDivider` between rows.

## Variants

- **Label and content** (`FieldCard`): use for a field with one label.
- **Surface alone** (`fieldCardSurface()`, iPhone only): use when the label sits beside its value or is a toggle's title, such as Key › in entry settings. Android builds this surface by hand with differing padding; matching it is #2606.

## Behaviour

No motion and no haptic.

## Content

- The card is `cardSurface()`: white with a hairline edge and the 3 point corner.
- The label is `FieldLabel`, 15 point medium, in `inkSecondary`.
- The control sits `IntradaSpacing.controlGap` (8) under the label.
- The content sits 16 in from the sides (`IntradaSpacing.card`) and 10 from the top and bottom. Moving the 10 onto a spacing token is #2665 on Android and #2237 on iPhone.
- The item form's rows match this padding. Android's sit taller at 12dp, which #2606 settles.
- No states of its own; the control inside it has its states.
- The label is one to three words, sentence case, naming what the musician sets. The shells write the labels.

## Accessibility

- The label and the control are read separately; the control carries its own spoken label.
- Every part scales with Dynamic Type and font scale. The card grows with its content.

## Do and don't

- **Do** put the label inside the card, above the value (T35).
- **Don't** put a label above and outside the card; that doubles a form's height.
- **Don't** give each item form field its own `FieldCard`; the form's fields are rows in one card.

## Usage

```swift
FieldCard("Beats in the bar") {
  Stepper(value: $beats, in: beatsRange) { Text("\(beats)") }
}
```

```kotlin
FieldCard("Kind") {
    SegmentedPills(kinds, selection, onSelect, label = { it.label })
}
```

## Related

`FieldLabel`, which draws the label. `Card`, its surface. `FormErrorBanner` for why a form cannot be saved.

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

- A text field on the item form: use `FormField` on iPhone, which draws the same label and padding and adds the error wash.
- A read-only fact on a details screen: label left, value right, in a card's row (T35 in [the design principles](../../../docs/design-principles.md)).
- A group of fields: one card with `HairlineDivider` between rows.

## Anatomy

1. **Card:** `cardSurface()`, white with a hairline edge and the 3 point corner.
2. **Label:** `FieldLabel`, 15 point medium, in `inkSecondary` (#6E6A66).
3. **Content:** the control, `IntradaSpacing.controlGap` (8) under the label.

The content sits 16 in from the sides (`IntradaSpacing.card`) and 10 from the top and bottom.

## Variants

- **Label and content** (`FieldCard`): use for a field with one label.
- **Surface alone** (`fieldCardSurface()`, iPhone only): use when the label sits beside its value or is a toggle's title, such as Key › in entry settings and Include screenshot in feedback. 6 call sites in 4 files.

## States

None of its own. The control inside it has its states.

## Behaviour

No motion and no haptic.

iPhone uses it at 10 call sites in 5 files: the metronome sheet (Time signature, Beats in the bar, Beat value, Grouped, Sounds on), entry settings (Aim, Variation), Highlighter on the profile, Note on feedback and Kind on a section's sheet. Android uses it at 7 call sites in 3 files: the same five on the metronome sheet, Highlighter on the profile and Kind on a section's screen.

The two apps draw it the same. Android has no `fieldCardSurface()`. On iPhone the item form's fields (`FormField`, `AutocompleteField`, `TagChipInput`, `KeyPicker`) are rows of one card, not a card each, and repeat the 10 point top and bottom padding by hand. The item form on Android builds its field cards by hand with 12dp top and bottom where `FieldCard` has 10, so those rows sit taller; using `FieldCard` there is A28 in #2606 ([the October audit](../../../docs/audit-2026-10-design.md)). The 10 is a literal on both apps; moving the iPhone one onto a spacing token is I28 in #2608, which #2237 covers. Android's literal is not in an issue yet.

## Content

The label is one to three words, sentence case, naming what the musician sets. The shells write the labels.

## Accessibility

- The label and the control are read separately; the control carries its own spoken label.
- Every part scales with Dynamic Type and font scale. The card grows with its content.

## Do and don't

- **Do** put the label inside the card, above the value (T35).
- **Don't** put a label above and outside the card; that doubles a form's height.

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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records the Android item form's hand-built field cards (A28). |

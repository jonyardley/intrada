---
name: FieldLabel
status: stable
ios_symbol: FieldLabel
ios_file: ios/Intrada/Views/Components/FieldCard.swift
ios_snapshots: ComponentSnapshotTests/testFieldLabel.1.png
android_symbol: FieldLabel
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/FieldCard.kt
android_snapshots: field-card.png
decisions: T35
---

# FieldLabel

A field's name, in secondary ink, inside its card above the value.

## When to use

- A field's label inside a card, where `FieldCard` does not fit: a label beside its value (Key ›), a toggle's title, a field in a custom layout.
- Through the components that draw it: `FieldCard` and `SectionTitle` on iPhone, and `FieldCard` (`FieldCard.kt`) and `SectionHeader` (`PageParts.kt`) on Android.

## When not to use

- A label with a control under it in its own card: use `FieldCard`, which draws this for you.
- A section's name on iPhone: use `SectionTitle`, so the code says which it is.
- A value the musician reads or sets: values are 17 point in full ink (T35 in [the design principles](../../../docs/design-principles.md)).

## Anatomy

One line of text: `IntradaFont.label`, 15 point medium, in `inkSecondary` (#6E6A66), sentence case.

## Variants

- **Default:** use for every label.
- **Error** (`tint: IntradaColor.danger` on iPhone, `colour = IntradaColor.danger` on Android, #9C4A3A): use while the field is at fault. On iPhone, `FormField`, `AutocompleteField`, `TagChipInput` and `VariationRowsSection` pass it; on Android, the item form's text field (`ItemFormParts.kt`) is the one caller.

## States

Default and error. The error colour comes and goes with the field's fault.

## Behaviour

No motion and no haptic.

iPhone uses it at 27 call sites in 19 files. 2 are inside the catalogue's own components, `FieldCard` and `SectionTitle`, so screens and other components call it 25 times in 17 files. Android uses it at 41 call sites in 23 files. 2 are inside `FieldCard` and `SectionHeader`, so 39 call sites in 21 files.

Android calls it more because, with no `SectionTitle`, it also names sections: the reflection sheet alone calls it 8 times (`ReflectionSheet.kt`). It looks the same either way. Building `SectionTitle` in Compose, so Android's section names say what they are, is #2636.

## Content

A name in sentence case, no colon; Android's reflection sheet adds a fact after a middle dot, "Reflection · optional". Most labels are one to three words; the longer ones that ship are "What got in the way" and "Drills for" a section's name, on both apps. The shells write the labels.

## Accessibility

- Read as plain text, apart from the control it names; the control carries its own spoken label.
- Scales with Dynamic Type and font scale.

## Do and don't

- **Do** put it at the top of what it names, inside the card (T35).
- **Don't** pass another colour for emphasis; secondary ink is what makes every label read the same.

## Usage

```swift
FieldLabel("Key")
FieldLabel(label, tint: faulted ? IntradaColor.danger : IntradaColor.inkSecondary)
```

```kotlin
FieldLabel("Variations", Modifier.padding(horizontal = IntradaSpacing.card))
FieldLabel(label, colour = if (faulted) IntradaColor.danger else IntradaColor.inkSecondary)
```

## Related

`FieldCard`, which draws it above a control. `SectionTitle`, which looks the same and names a section.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records that Android also uses it for section names. |

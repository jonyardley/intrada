---
name: AddRowButton
status: stable
ios_symbol: AddRowButton
ios_file: ios/Intrada/Views/Components/AddRowButton.swift
ios_snapshots: ComponentSnapshotTests/testAddRowButtonVariants.1.png
android_symbol: AddRow
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PageParts.kt
android_snapshots: builder.png
---

# AddRowButton

A full-width "Add" row that invites the musician to put something into a list or an empty slot.

## When to use

- The end of a list the musician fills: a session's pieces, a piece's sections, variations or related exercises.
- An empty slot that says what goes there, with a hint underneath.

## When not to use

- The one call to action on a screen, such as Start: use `MarkerButton`.
- Adding a piece to the Library: that is the round add button in the Library header.
- An action that changes rather than adds, such as Edit or Ungroup all: use the text action in `SectionHeader`.

## Variants

- **Dashed** (`.dashed`, the default): use for an empty slot standing on its own. A dashed outline draws where the content will go.
- **Plain** (`.plain`): use at the foot of a list or inside a card, where an outline would box in a box.
- **With a hint** (`hint`): use when the slot needs a word on what it takes, such as "A1, B, Coda, or bars 12 to 14".

## Behaviour

- One tap runs the action. There is no haptic of its own; the screen it opens gives the feedback.
- Android draws every add row plain, and a caller puts it on a card where iPhone shows a dashed slot; dashed slots with a plus glyph are A18 in #2606.

## Content

- The label is `IntradaFont.bodyMedium` in `accent`, led by a plus glyph, centred.
- The hint is `IntradaFont.secondary` in `inkSecondary`, centred, and wraps.
- The dashed outline is one point of `addDashOutline`, dashed four on four, with the `IntradaRadius.card` corner, and `IntradaSpacing.card` (16) above and below the label. Plain has `cardCompact` (12).
- The label starts with a verb and names the thing: "Add section", "Add piece or exercise", "Choose variations". Keep it to one line on a phone.

## Accessibility

- One button, read by its label. Android takes a separate spoken label so it can say more than fits, such as "Add a piece or exercise".
- Android rows are at least 48dp tall. The label and hint scale with Dynamic Type and font scale.

## Do and don't

- **Do** use one add row at the end of a list, not one per section.
- **Don't** draw a dashed outline by hand: I24 in #2608 settles one dash pattern for the whole app.

## Usage

```swift
AddRowButton(title: "Add sections", hint: "A1, B, Coda, or bars 12 to 14", action: onAdd)
AddRowButton(title: "Add section", style: .plain, action: onAdd)
```

```kotlin
AddRow(
    "Add piece or exercise",
    "Add a piece or exercise",
    "builder.addItems",
    onAddItems,
    Modifier.cardSurface(),
)
```

## Related

`MarkerButton`, the one call to action. `SectionHeader`, a section's title with its action. `Card`, the surface an Android add row sits on.

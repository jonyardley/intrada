---
name: TagChip
status: stable
ios_symbol: TagChip
ios_file: ios/Intrada/DesignSystem/TagChip.swift
ios_snapshots: ComponentSnapshotTests/testTagChip.1.png
android_symbol: TagChip, TagChipStyle
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/TagChip.kt, android/app/src/main/kotlin/com/intrada/android/ui/components/TagChipStyle.kt
android_snapshots: tag-chip.png
decisions: T29, T35
---

# TagChip

A short label in a small rounded pill: a tag on an item, or a word that marks a row. Editable chips carry a cross to remove them.

## When to use

- A musician's own tags: on a library card, the piece header, a planned item in the builder.
- A short fact about a row that is not a control: "Tricky spot" and "Weakest" on a section.
- The tags field on the add and edit forms, where each chip removes itself when tapped.

## When not to use

- Piece or exercise: use `TypeBadge`, which carries the type's colour and glyph (T3, T29 in [the design principles](../../../docs/design-principles.md)).
- Choosing one of a few options: use `SegmentedPills`.
- A count or status word made by hand: use this component, not a copy. The iPhone copies move onto it in #2608.

## Variants

- **Sunken** (the default): use for a chip inside a card.
- **Outlined** (`style: .outlined`, `TagChipStyle.Outlined`): use for a chip on bare paper, where it needs its own edge.
- **Removable** (`onRemove`): use where the musician edits the tags.

A library card shows at most three chips and a sunken "+N" chip for the rest (`TagPills` on iPhone); tags on Android library cards are #2679.

## Behaviour

A removable chip has no pressed look; tapping it removes the tag at once. Its touch area is at least 44 points or 48dp each way, without the drawn chip growing. Android has it; the iPhone chip reaching 44 points is #2662.

## Content

- The fill is `surfaceSunken` (#F3EFE8) inside a card, `cardFill` on paper. The outlined chip adds a one point `hairline` (#ECE6DA) edge.
- The label is `IntradaFont.smallMedium`, 13 point medium, in `inkSecondary`, on one line. A long tag truncates with an ellipsis; a chip never wraps.
- The cross is the close glyph at `IntradaIconSize.badge` (9 points) in `inkFaintIcon`, 5 points after the label.
- The shape is a capsule. Padding is 3 by 8, 4 by 9 when removable, 5 by 10 when outlined. Both apps set it with literals; moving them onto spacing tokens is #2237 on iPhone and #2606 on Android.
- One or two words, as the musician typed them, so no case is forced.

## Accessibility

- A plain chip is read as its text. On iPhone, `TagPills` groups a card's chips and reads "Tags: " and the full list, including any hidden behind "+N".
- A removable chip is one button labelled with the tag, and both apps say "Double tap to remove": iPhone as a hint, Android through the click label "remove".
- The label is 13 points and scales with Dynamic Type and font scale.

## Do and don't

- **Do** use the outlined chip on paper and the sunken one inside a card, so the chip always has an edge against what it sits on.
- **Don't** draw a capsule with a fill and a small label by hand for a status word; use `TagChip`, or ask for a parameter.

## Usage

```swift
TagChip("Baroque")
TagChip(tag, style: .outlined)
TagChip(tag, onRemove: { remove(tag) })
```

```kotlin
TagChip("Baroque")
TagChip(tag, style = TagChipStyle.Outlined)
TagChip(chip, Modifier.testTag("$tag.chip"), onRemove = { onRemove(index) })
```

## Related

`TypeBadge` for piece or exercise, `SegmentedPills` for a choice.

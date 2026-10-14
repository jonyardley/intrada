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

- Piece or exercise: use `TypeBadge`, which carries the type's colour and glyph (T3, T29).
- Choosing one of a few options: use `SegmentedPills`.
- A count or status made by hand beside a chip: use this component, not a copy. Two such copies exist on iPhone ("Already added" and "Fallback" on the chord chart suggestions, "Group" in the builder), I23 in [the October audit](../../../docs/audit-2026-10-design.md), fixed in #2608.

## Anatomy

1. **Fill:** `surfaceSunken` (#F3EFE8) for a chip inside a card, `cardFill` for a chip on paper.
2. **Edge:** a one point `hairline` (#ECE6DA) on the outlined chip only.
3. **Label:** `IntradaFont.smallMedium`, 13 point medium, in `inkSecondary`, one line.
4. **Cross (removable only):** the close glyph at `IntradaIconSize.badge` (9 points) in `inkFaintIcon`, 5 points after the label.
5. **Shape:** a capsule.

## Variants

- **Sunken** (the default): use for a chip inside a card. Library cards, the builder's plan line, the section rows and the tags field all use it.
- **Outlined** (`style: .outlined`, `TagChipStyle.Outlined`): use for a chip on bare paper, where it needs its own edge. The piece header's tags and the Keys sheet's quick picks ("All major" and the like, wrapped in a button) use it.
- **Removable** (`onRemove`): use where the musician edits the tags. Only the sunken removable chip has a caller, the tags field on the add and edit forms on both apps. An outlined removable chip has no caller; the snapshots draw one to show the shape.

On iPhone, library cards show at most three chips and a sunken "+N" chip for the rest (`TagPills`).

## States

Default only. A removable chip has no pressed look; tapping it removes the tag at once.

## Behaviour

**On 14 October 2026 the two apps draw it the same:** the same fill, edge, padding (3 by 8, 4 by 9 when removable, 5 by 10 when outlined), font and glyph. They differ in how a removable chip is touched:

| | iPhone | Android |
|---|---|---|
| Touch area of a removable chip | the drawn chip only, well under 44 points tall | a box at least 48dp each way around the chip |
| Spoken | the tag, "Double tap to remove", as a button | the tag, "remove" as the click label, as a button |
| Tags on library cards | up to three, then "+N" | none |

The Android touch area is the one to keep; the iPhone chip's tap area should reach 44 points without the drawn chip growing. No issue tracks that yet; it belongs with the small tap targets in #2607. Both apps keep what they say: TalkBack reads Android's click label as "Double tap to remove", the words of the iPhone hint.

No issue tracks tags on Android library cards: `LibraryScreen.kt` draws none, and A20 in #2606 adds the key, tempo, linked exercise count and ring but not tags.

Both apps set the padding with literals rather than spacing tokens. On Android that is A29 in the audit, fixed in #2606. On iPhone, #2237 (open) adds a spacing step for the 2 to 6 point gaps and moves literals onto tokens; its list of files does not name `TagChip.swift`, whose 3, 4 and 5 point paddings fall in that range.

## Content

One or two words, as the musician typed them, so no case is forced. A chip never wraps; a long tag truncates with an ellipsis.

## Accessibility

- A plain chip is read as its text. On iPhone, `TagPills` groups a card's chips and reads "Tags: " and the full list, including any hidden behind "+N".
- A removable chip is one button labelled with the tag. iPhone adds the hint "Double tap to remove"; Android sets "remove" as the click label.
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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records the tap area difference and the outlined removable chip with no caller. |

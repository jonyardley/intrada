---
name: TypeBadge
status: stable
ios_symbol: TypeBadge
ios_file: ios/Intrada/Views/Components/TypeBadge.swift
ios_snapshots: LibrarySnapshotTests/testTypeBadges.1.png
not_on_android: Android draws a plain pill on the piece page and plain text on the player instead (A10, A11); the shared version is built in #2636
decisions: T3, T24, T29, T35
---

# TypeBadge

Says whether an item is a piece or an exercise, at a glance, with a glyph, a word and a tint.

## When to use

- At the top of an item's own page, as the first pill in the row of tags.
- On the player, above the title of what is playing.

## When not to use

- In a list row: library cards carry the type as the kind bar down their edge, so a badge would say it twice.
- A musician's own tag: use `TagChip`.
- Any control: the badge is never pressed. Its tints are kept duller than the highlighter so it never reads as a button (T24 in [the design principles](../../../docs/design-principles.md)).

## Variants

Two, one per kind. There is no size or style option. It is a pill wherever it appears, the player included (T29).

## Behaviour

None. The badge shows a fact and does nothing when tapped, with no motion and no haptic.

Android draws a plain pill and plain text instead; the shared badge is #2636, and the piece page and the player move onto it in #2606.

## Content

- The fill is `pieceBadgeBg` (#CBD6E0, blue grey) or `exerciseBadgeBg` (#DCD3C0, sand).
- The glyph is a note (`music.note`) for a piece, a dumbbell (`dumbbell.fill`) for an exercise, at small scale.
- The word is "Piece" or "Exercise", written once in each shell's `ItemKindStyle` (`ItemKind.label`) and used unchanged.
- The ink is `pieceBadgeFg` and `exerciseBadgeFg`, both `ink`, set in `IntradaFont.badge` (13 point semibold).
- The shape is a capsule, 5 by 10 points of padding, 5 points between glyph and word.

## Accessibility

- One element, read as "Piece" or "Exercise". The glyph adds nothing to it.
- Colour is never the only signal (T3): the glyph and the word always show.
- With the Powder highlighter chosen, the piece tint is close to the highlighter, so a piece badge can look pressable; #2638.

## Do and don't

- **Do** put it first in the row of pills on an item's page, before the tags.
- **Don't** tint anything else with the badge colours; they mean the kind of item and nothing else.

## Usage

```swift
TypeBadge(kind: item.itemType)
```

```kotlin
// No shared badge yet; #2636 builds it.
```

## Related

`TagChip`, the kind bar on `LibraryItemCard`.

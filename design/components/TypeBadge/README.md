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
- Any control: the badge is never pressed. Its tints are kept duller than the highlighter so it never reads as a button (T24).

## Anatomy

1. **Fill:** `pieceBadgeBg` (#CBD6E0, blue grey) or `exerciseBadgeBg` (#DCD3C0, sand).
2. **Glyph:** a note (`music.note`) for a piece, a dumbbell (`dumbbell.fill`) for an exercise, at small scale.
3. **Word:** "Piece" or "Exercise", from the shell's label for the core's kind (`ItemKind.label` in `ItemKindStyle.swift` and `ItemKindStyle.kt`).
4. **Ink:** `pieceBadgeFg` and `exerciseBadgeFg`, both `ink`, set in `IntradaFont.badge` (13 point semibold).
5. **Shape:** a capsule, 5 by 10 points of padding, 5 points between glyph and word.

## Variants

Two, one per kind. There is no size or style option.

## States

None. The badge shows a fact and does nothing when tapped.

## Behaviour

No motion and no haptic.

**On 14 October 2026 Android has no badge component.** iPhone uses `TypeBadge` on the piece and exercise page (`LibraryDetailScreen`) and on the player (`FocusPlayerScreen`). Android draws its own:

| | iPhone | Android piece page | Android player |
|---|---|---|---|
| Shape | tinted capsule | tinted capsule, padding set by hand | none, plain text |
| Glyph | note or dumbbell | none | none |
| Word | 13 point semibold, ink | 13 point medium, ink | 13 point semibold, secondary ink |

The iPhone badge is the one both apps use. T3 needs a cue beyond colour, which the glyph and the word give, and T29 makes the badge a pill wherever it appears, the player included. Android builds a shared `TypeBadge` in #2636, and moves the piece page and the player onto it with A10 and A11 in #2606 ([the October audit](../../../docs/audit-2026-10-design.md)). Android has no note or dumbbell drawable yet.

## Content

"Piece" or "Exercise", written once in each shell's `ItemKindStyle` and used unchanged.

## Accessibility

- One element, read as "Piece" or "Exercise". The glyph adds nothing to it.
- Colour is never the only signal (T3): the word always shows.
- With the Powder highlighter chosen, the piece tint is close to the highlighter (I38 in the audit), so a piece badge can look pressable. That waits for the refinement pass (#2638).

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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records that Android has no badge, only a plain pill and plain text. |

---
name: SegmentedProgress
status: stable
ios_symbol: SegmentedProgress
ios_file: ios/Intrada/Views/Components/SegmentedProgress.swift
ios_snapshots: ComponentSnapshotTests/testSegmentedProgress.1.png, FocusPlayerSnapshotTests/testOrientationBandLongSession.1.png
not_on_android: #2636 builds the shared Compose version; until then the player and Progress each draw their own strip by hand (see Behaviour).
decisions: T24
---

# SegmentedProgress

A row of equal segments, some filled: which item of a session you are on, or how many of a set are solid.

## When to use

- Where the musician is in a session: one segment per item, filled up to the one playing (the Focus player).
- How many of a fixed set are solid: one segment per variation or item, filled for each solid one (Progress).

## When not to use

- Time passing: the timer ring and the target bar show a continuous fill.
- A score from 1 to 10: use `ScoreRing`.
- A count with nothing to compare it to: write the number.

## Anatomy

1. **Segments:** one capsule per item, equal widths, 5 points apart.
2. **Filled segment:** `accent` (`ink`, #2A2725).
3. **Empty segment:** `divider` (#DDD7CA).

## Variants

- **Session position** (`init(types:filled:)`): use on the player. 4 points tall. It takes the session's item kinds, but both kinds fill in ink (T24 in [the design principles](../../../docs/design-principles.md)), so the kinds do not show.
- **Solid count** (`init(count:filled:label:)`): use for a count of a set. 6 points tall.

Each has one caller on iPhone. The height is fixed by the variant, and both fill in ink.

## States

None of its own. The filled count comes from the core.

## Behaviour

No motion and no haptic. The strip is not a control.

iPhone uses it at 2 call sites in 2 files: the player's `SessionOrientationBand` (the Focus player) and `SolidCountRow`, which Progress uses for Variations, Variations across the library and Keys across the library. Those are the only screens.

**On 14 October 2026 Android has no `SegmentedProgress`.** It draws the strip by hand twice:

| | iPhone | Android player (`OrientationBand`, `PlayerParts.kt`) | Android Progress (`SolidCountRow`, `ProgressParts.kt`) |
|---|---|---|---|
| Filled | ink | the kind's pastel, blue-grey or sand | ink |
| Empty | `divider` (#DDD7CA) | `timerTrack` (#EDE8DC) | `divider` (#DDD7CA) |
| Gap | 5 | 3dp | 5dp |
| Height | 4 player, 6 count | 4dp | 6dp |

Filled segments go to ink on both apps, because the pastel misses the 3:1 contrast a meaningful mark needs (WCAG 1.4.11, T24). Filling the Android player's segments in ink is A12 in #2605 ([the October audit](../../../docs/audit-2026-10-design.md)). The empty colour and gap both apps keep are decided in #2663. Building one shared Compose strip is #2636.

No iPhone screen draws a strip by hand.

## Content

No words of its own. The core writes the solid counts ("7 of 12 solid", "Solid on 2 of 3 items") and the spoken rows ("Scales, 2 of 3 variations solid"); the iPhone shell writes the player's spoken position.

## Accessibility

- On iPhone the strip is one element. The player strip says "Item 2 of 5", from the shell; a solid count says the label it is given, and `SolidCountRow` replaces that with the core's spoken row.
- On Android the player strip reports its range as progress to TalkBack, and the Progress row reads the core's spoken row.
- The segments do not scale with text size.

## Do and don't

- **Don't** draw a strip of your own from capsules or boxes; Android's two hand-drawn strips are the copies that drifted.

## Usage

```swift
SegmentedProgress(types: active.entries.map(\.itemType), filled: position)
SegmentedProgress(count: total, filled: solid, label: caption)
```

## Related

`ScoreRing` for a score. `SectionHeader`, which titles the Progress sections these sit in.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records Android's two hand-drawn strips and the player's pastel fill. |
| 2026-10-14 | #2635 | Deleted the unused `fill:` parameter and fixed each variant's height; commit d93d7a4e still has both parameters. |

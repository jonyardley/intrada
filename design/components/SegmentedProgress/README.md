---
name: SegmentedProgress
status: stable
ios_symbol: SegmentedProgress
ios_file: ios/Intrada/Views/Components/SegmentedProgress.swift
ios_snapshots: ComponentSnapshotTests/testSegmentedProgress.1.png, FocusPlayerSnapshotTests/testOrientationBandLongSession.1.png, FocusPlayerSnapshotTests/testFocusPlayerSectionTimeUp.1.png
not_on_android: #2636 builds the shared Compose version; until then the player, its section strip and Progress each draw their own strip by hand (see Behaviour).
decisions: T24
---

# SegmentedProgress

A row of equal segments, some filled: which item of a session you are on, or how many of a set are solid.

## When to use

- Where the musician is in a session: one segment per item, filled up to the one playing (the Focus player).
- How many of a fixed set are solid: one segment per variation or item, filled for each solid one (Progress).
- Where the musician is in an item split into sections: one segment per section, filled once done, with the section names and the core's caption beneath (#2394).

## When not to use

- Time passing: the timer ring and the target bar show a continuous fill.
- A score from 1 to 10: use `ScoreRing`.
- A count with nothing to compare it to: write the number.

## Variants

- **Session position** (`init(types:filled:)`): use on the player. 4 points tall. It takes the session's item kinds, but both kinds fill in ink, so the kinds do not show.
- **Solid count** (`init(count:filled:label:)`): use for a count of a set. 6 points tall.

The height is fixed by the variant.

## Behaviour

No motion and no haptic. The strip is not a control.

## Content

- One capsule per item, equal widths, `segmentGap` (5 points) apart.
- A filled segment is `accent` (`ink`, #2A2725) on both variants. A pastel misses the 3:1 contrast a meaningful mark needs (WCAG 1.4.11, T24 in [the design principles](../../../docs/design-principles.md)); the Android player's pastel fill goes to ink in #2605.
- An empty segment is `divider` (#DDD7CA). The empty colour and gap the player keeps are decided in #2663.
- No states of its own. The filled count comes from the core.
- No words of its own. The core writes the solid counts ("7 of 12 solid", "Solid on 2 of 3 items") and the spoken rows ("Scales, 2 of 3 variations solid"); the iPhone shell writes the player's spoken position.

## Accessibility

- On iPhone the strip is one element. The player strip says "Item 2 of 5", from the shell; a solid count says the label it is given, and `SolidCountRow` replaces that with the core's spoken row.
- On Android the player strip reports its range as progress to TalkBack, and the Progress row reads the core's spoken row.
- The section strip is one element on both apps and says the core's position label ("Section 2 of 3, 2 min taken from A2").
- The segments do not scale with text size.

## Do and don't

- **Don't** draw a strip of your own from capsules or boxes. Android draws three by hand until the shared Compose strip lands in #2636.

## Usage

```swift
SegmentedProgress(types: active.entries.map(\.itemType), filled: position)
SegmentedProgress(count: total, filled: solid, label: caption)
```

## Related

`ScoreRing` for a score. `SectionHeader`, which titles the Progress sections these sit in.

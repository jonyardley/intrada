---
name: ScoreRing
status: stable
ios_symbol: ScoreRing
ios_file: ios/Intrada/Views/Components/ScoreRing.swift
ios_snapshots: ComponentSnapshotTests/testScoreRing.1.png, ComponentSnapshotTests/testScoreRingHero.1.png, ComponentSnapshotTests/testScoreRingKeyLabels.1.png
android_symbol: ScoreRing
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PracticeParts.kt
android_snapshots: score-ring.png
decisions: T24, T31, T35
---

# ScoreRing

A mark from 1 to 10 as a ring that fills clockwise, with the number in the middle, or a rest when there is no mark yet.

## When to use

- The latest mark on anything the musician has played: a library card, a section, a key, a related exercise, a past session.
- The large overall mark at the top of an exercise's page.

## When not to use

- Setting a mark: use `ScoreSelector`, the row of boxes on the reflection and summary.
- A count or a duration: set the figure on its own, in mono (T35 in [the design principles](../../../docs/design-principles.md)).
- Progress through a session: use `SegmentedProgress`.

## Variants

- **Small** (32, 34, 38 or 44 points): use in a row.
- **Standard** (46, the default): use for a session's own mark on its detail page.
- **Hero with scale** (`showsScale`, 132 points): use once, at the top of an exercise's page. Adds "of 10" under the mark.
- **Key label** (`labelOverride`): use for a key in the Keys card. The key's name replaces the mark; the arc still shows the mark.

## Behaviour

On iPhone the arc draws in from empty when the ring appears, easing out over `IntradaMotion.countUpDuration`. Under Reduce Motion, and in snapshots and UI tests, it shows filled at once. No haptic; the ring is never pressed.

- The iPhone ring is the one both apps follow. Android's has no rest, no scale caption and no draw in yet; the Library card ring is #2606.
- Android's Keys card draws its own ring by hand (`KeyRing`) rather than `ScoreRing` with a key label; moving it onto `ScoreRing` is #2685.
- The rest reads as an empty clock rather than "not yet played"; that is the refinement pass, #2638.

## Content

- The track is a full circle in `masteryTrack`. The arc is `masteryFill` (`ink`), from twelve o'clock, with round ends, the mark's share of the top mark.
- The line is 9% of the ring's size, never under 3 points. The mark is `IntradaFont.scoreNumeral` at 36% of the size, in `ink`.
- The top mark comes from the core (`scoreRange` on iPhone, `topMark` on Android); never assume ten.
- No mark yet is the track alone with an eighth rest in `inkFaintIcon`. With a key label, the label shows in secondary ink instead.
- A key label shrinks to half size before it truncates. The hero's "of 10" is `IntradaFont.small` in secondary ink.
- The number alone: no "/10" and no word inside the ring.

## Accessibility

- One element, read as "Mark 7 of 10" or "Not yet rated". A row that holds a ring can merge it into one longer label ("How it went, your mark for the session, 7 of 10").
- The arc and number are `ink` on paper or white. The rest is `inkFaintIcon`, chosen to clear the 3:1 floor for graphics (T31).
- The numeral is sized by the ring, not by Dynamic Type or font scale (T35's exception for the score numerals).

## Do and don't

- **Do** keep one size per list, so the marks line up down the column.
- **Don't** colour the arc by the mark. One ink for every mark keeps a low mark from reading as an error.

## Usage

```swift
ScoreRing(score: item.practice?.latestScore.map(Int.init), size: 32)
ScoreRing(score: key.latestScore.map(Int.init), size: 44, labelOverride: shortLabel)
```

```kotlin
ScoreRing(score, topMark)
ring?.let { ScoreRing(it, topMark, size = 34.dp) }
```

## Related

`ScoreSelector` to set a mark, `MasteryDelta` for a change of mark.

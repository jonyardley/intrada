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

- The latest mark on anything the musician has played: a library card, a section, a key, a related exercise, a "Used in" row, a past session.
- The large overall mark at the top of an exercise's page.

## When not to use

- Setting a mark: use `ScoreSelector`, the row of boxes on the reflection and summary.
- A count or a duration: set the figure on its own, in mono (T35).
- Progress through a session: use `SegmentedProgress`.

## Anatomy

1. **Track:** a full circle in `masteryTrack` (#EDE8DC).
2. **Arc:** `masteryFill` (`ink`), from twelve o'clock, with round ends, the mark's share of the top mark.
3. **Middle:** the mark in `IntradaFont.scoreNumeral`, Hanken semibold at 36% of the ring's size, in `ink`.
4. **Line:** 9% of the ring's size, never under 3 points.

The top mark comes from the core (`scoreRange` on iPhone, `topMark` on Android), so a 1 to 5 range draws correctly.

## Variants

- **Small** (32, 34, 38 or 44 points): use in a row. Library cards and section rows use 32; the session detail's item rows 34; recent sessions 38; related exercises, "Used in" rows and keys 44.
- **Standard** (46, the default): the session's own mark on the session detail.
- **Hero with scale** (`showsScale`, 132 points): use once, at the top of an exercise's page. Adds "of 10" under the mark in `IntradaFont.small`, secondary ink.
- **Key label** (`labelOverride`): use for a key in the Keys card. The key's name replaces the mark, shrinking to half size before it truncates; the arc still shows the mark.
- **Solid** (`solid`): no caller passes it, and since `accent` and `masteryFill` are both `ink` it draws the same as the default. Remove it the next time this component changes.

## States

- **Marked:** the arc and the number.
- **No mark yet:** the track alone with an eighth rest in `inkFaintIcon`, the rest being "not yet played". With a key label, the label shows in secondary ink instead.

## Behaviour

On iPhone the arc draws in from empty when the ring appears, easing out over `IntradaMotion.countUpDuration`. Under Reduce Motion, and in snapshots and UI tests, it shows filled at once. No haptic; the ring is never pressed.

**On 14 October 2026 Android has a smaller version:**

| | iPhone | Android |
|---|---|---|
| Where | 8 places: library cards, the exercise hero, sections, keys, related exercises, "Used in", recent sessions, session detail | session detail only |
| No mark yet | the rest | not drawn; it takes a mark that is never empty |
| Scale caption | on the hero | none |
| Key label | in the Keys card | none |
| Draw in | yes | no |

The iPhone ring is the one both apps use. Android adds the rest, the caption and the key label as it builds the screens that need them; the Library card ring is A20 in [the October audit](../../../docs/audit-2026-10-design.md), fixed in #2606. Rings on the Android piece page are not in that issue yet.

The rest for "never marked" reads to some eyes as an empty clock (the audit's refinement note 1); that waits for the refinement pass (#2638).

## Content

The number alone. No "/10" and no word inside the ring; the hero's "of 10" is the one caption.

## Accessibility

- One element. iPhone reads "Mark 7 of 10" or "Not yet rated"; Android reads "Mark 7 of 10". A row that holds a ring can merge it into one longer label, as Android's session mark card does ("How it went, your mark for the session, 7 of 10").
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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records Android's smaller version and the solid variant with no caller. |

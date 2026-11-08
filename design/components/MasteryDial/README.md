---
name: MasteryDial
status: stable
ios_symbol: MasteryDial
ios_file: ios/Intrada/Views/Components/MasteryDial.swift
ios_snapshots: ProgressSnapshotTests/testMasteryDial.1.png, ProgressSnapshotTests/testMasteryDialTakesTheTopMarkFromTheCore.1.png
android_symbol: MasteryDial
android_file: android/app/src/main/kotlin/com/intrada/android/ui/ProgressParts.kt
android_snapshots: mastery-dial.png
decisions: T35
---

# MasteryDial

The musician's overall mastery: the average mark across the library, as a ring and a number out of the top mark.

## When to use

- The headline of the Progress screen.

## When not to use

- One item's mark: use `ScoreRing`.
- A change in a mark after a session: use `MasteryDeltaToast`.

## Variants

None. The iPhone version takes a `size`, 128 points by default.

## Behaviour

- On iPhone the ring draws and the number counts up together as it appears, easing out over `IntradaMotion.countUpDuration`.
- Under Reduce Motion, or with the app's motion turned off, both show their final value at once.
- Android shows the final value at once; the draw and count up are G2 in #2762, and #2673 decides the Progress screen's reveal on Android.

## Content

- A 9 point `dialTrack` ring, with the filled arc in the `ringSweep` gradient and round ends, starting at twelve o'clock.
- The arc fills the value's share of the top mark, held between empty and full.
- The number, to one decimal place, in `scoreNumeral` at 0.297 of the size, `ink`.
- Under it, "of" and the top mark to one decimal place, in `small`, `inkSecondary`.
- The core gives the value and the top mark; neither app assumes the top mark.

## Accessibility

- One element, read as "Overall mastery 3.4 of 5.0".
- On iPhone the whole dial grows with Dynamic Type, up to 1.6 times, so the number never crosses the ring (#1471). At accessibility sizes the text shrinks to fit inside.
- On Android the ring stays 128 dp while the text follows font scale, G3 in #2762.

## Usage

```swift
MasteryDial(value: mastery)
```

```kotlin
MasteryDial(mastery, topMark)
```

## Related

`ScoreRing`, one item's mark. `ConsistencyBars`, the other Progress headline. `MasteryDeltaToast`.

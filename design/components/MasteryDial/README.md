---
name: MasteryDial
status: stable
ios_symbol: MasteryDial
ios_file: ios/Intrada/Views/Components/MasteryDial.swift
ios_snapshots: ProgressSnapshotTests/testMasteryDial.1.png, ProgressSnapshotTests/testMasteryDialTakesTheTopMarkFromTheCore.1.png
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

## Content

- A 9 point `track` ring, with the filled arc in `ink` and round ends, starting at twelve o'clock.
- The arc fills the value's share of the top mark, held between empty and full.
- The number, to one decimal place, in `scoreNumeral` at 0.297 of the size, `ink`.
- Under it, "of" and the top mark to one decimal place, in `small`, `inkSecondary`.
- The core gives the value and the top mark; the app never assumes the top mark.

## Accessibility

- One element, read as "Overall mastery 3.4 of 5.0".
- On iPhone the whole dial grows with Dynamic Type, up to 1.6 times, so the number never crosses the ring (#1471).

## Usage

```swift
MasteryDial(value: mastery)
```

## Related

`ScoreRing`, one item's mark. `ConsistencyBars`, the other Progress headline. `MasteryDeltaToast`.

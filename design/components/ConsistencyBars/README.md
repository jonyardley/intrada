---
name: ConsistencyBars
status: stable
ios_symbol: ConsistencyBars
ios_file: ios/Intrada/Views/Components/ConsistencyBars.swift
ios_snapshots: ProgressSnapshotTests/testConsistencyBars.1.png
android_symbol: ConsistencyBars
android_file: android/app/src/main/kotlin/com/intrada/android/ui/ProgressParts.kt
android_snapshots: consistency-bars.png
---

# ConsistencyBars

Minutes practised in each recent week, as bars, with this week picked out.

## When to use

- The weekly rhythm of practice on Progress: how much, week by week, and how this week compares.

## When not to use

- Days within one week: use `WeekStrip`.
- A count to keep up, such as a streak: Progress shows a comeback, not a streak, so nothing here counts days in a row.
- Progress through a session's items: use `SegmentedProgress`.

## Behaviour

- On iPhone the bars grow from the baseline when the view appears, one after another, with `IntradaMotion.barGrow` and `barGrowStagger`. Under Reduce Motion they appear at full height.
- Android draws the bars at full height at once; the grow is G1 in #2762.
- The bars are not interactive.

## Content

- The busiest week's bar is 58 points tall and the others scale to it. A week with little or no practice still shows a 6 point stub, so every week has a mark.
- Past weeks fill with `consistencyTrack`. This week fills with the `inkBar` gradient and the `IntradaShadow.glow` lift.
- Each bar has a 5 point corner and the bars sit 9 points apart, sharing the width equally.
- Week labels are `IntradaFont.small` in `inkSecondary`; this week's is `accent` and semibold.
- The bars sit on the paper, not in a card.
- The core writes each week's label, its minutes and its spoken line.

## Accessibility

- Each week is one element, read by the core's spoken line for that week. The bar height is not read on its own.
- Labels scale with Dynamic Type and font scale; the bars keep their height.

## Usage

```swift
ConsistencyBars(weeks: analytics.consistencyWeeks)
```

```kotlin
ConsistencyBars(analytics.consistencyWeeks)
```

## Related

`WeekStrip`, the days of one week. `SectionHeader`, the title above the bars on Progress, with the best week as its trailing text.

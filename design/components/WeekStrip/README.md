---
name: WeekStrip
status: stable
ios_symbol: WeekStrip
ios_file: ios/Intrada/Views/Components/WeekStrip.swift
ios_snapshots: ComponentSnapshotTests/testWeekStripDimsDaysStillToCome.1.png
android_symbol: WeekStrip
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PracticeParts.kt
android_snapshots: week-strip.png
decisions: T35
---

# WeekStrip

A row of seven days, Monday to Sunday, that picks the day whose sessions show below it.

## When to use

- The top of the Practice home, to move between days in the week.

## When not to use

- Practice over several weeks: use `ConsistencyBars`.
- Choosing a date for something new: this strip only picks from days the core already has.

## Variants

None. Each day shows one or more of four states, below.

## Behaviour

- A tap on a day selects it. One day is always selected; the screen picks one when it opens.
- Swiping between weeks belongs to the screen around the strip, not the strip.
- No motion of its own.

## Content

- Each day is the weekday initial over the day number, with a dot underneath.
- **Today:** the initial in `accent`, semibold, and a 1.5 point `accent` ring round the number.
- **Selected:** the initial in `accent`, semibold, and the number in `onAccent` on an `accent` disc. Selection hides today's ring.
- **Practised:** a 5 point `accent` dot under the number. A day without practice keeps the space empty, so the row never shifts.
- **Still to come:** the number in `inkSecondary` rather than `ink`.
- The initial is `small`; the number is `smallMedium`. Days sit 4 points apart and share the width equally.
- The core writes the initial, the number, the full date and whether a day is today, still to come or practised.

## Accessibility

- Each day is one element, read as "Today," when it is, then the full date, then "practised" or "no practice".
- The selected day carries the selected trait on iPhone and the selected state on Android, where each day is a tab.
- The number's disc is 32 points and grows with Dynamic Type and font scale, capped at 36 so seven days still fit (#1730). On iPhone the number shrinks to fit the disc.
- Each day is at least 48 dp tall on Android.

## Usage

```swift
WeekStrip(days: week.days, selected: $selectedDay)
```

```kotlin
WeekStrip(week.days, day?.date) { selectedDay = it }
```

## Related

`SessionCard`, the sessions for the chosen day. `ConsistencyBars`, the longer view.

---
name: RecentSessions
status: stable
ios_symbol: RecentSessions, RecentSession
ios_file: ios/Intrada/Views/Components/RecentSessions.swift
ios_snapshots: LibraryDetailSnapshotTests/testRecentSessions.1.png, LibraryDetailSnapshotTests/testRecentSessionsDeclining.1.png
not_on_android: Android's piece and exercise pages show no practice history yet, G7 in #2762.
decisions: T24, T35
---

# RecentSessions

The marks a piece or exercise got in its last few sessions, newest first, with the trend across them.

## When to use

- A piece's or exercise's page, to show how it has gone lately.

## When not to use

- A list of whole sessions: use `SessionCard`.
- The mark alone, with no history: use `ScoreRing`.
- Progress across the whole library: use `MasteryDial`.

## Variants

- **With a trend:** use when the core gives one. The header shows the first and last mark joined by an arrow.
- **Without a trend:** the header shows the title alone.

## Behaviour

None. Rows are not tappable and nothing moves.

## Content

- A card, `cardSurface()` with the `IntradaRadius.panel` corner. That corner goes when #2664 removes the parameter.
- The header is `SectionTitle` "Recent sessions", with the trend on the right in `badge`.
- A rise shows in `success` with a rising chart glyph. A fall shows in `inkSecondary` with a falling glyph: a drop is neutral, never red.
- Each row is a 38 point `ScoreRing` and the date in `secondary`, `inkSecondary`, with `HairlineDivider` between rows.
- The core gives the marks, their order and the trend. The shell formats the date.

## Accessibility

- Each row is one element: the mark, then the date.
- The trend reads as "Trend, improved from 5 to 7", or "declined"; the glyph is hidden.
- Text scales with Dynamic Type; the rings stay 38 points.

## Usage

```swift
RecentSessions(
  sessions: item.practice?.recentSessionRows(locale: locale, calendar: calendar) ?? [],
  trend: item.practice?.scoreTrend)
```

## Related

`ScoreRing`, each mark. `HairlineDivider`, between rows. `SessionCard`, a whole session.

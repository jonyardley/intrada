---
name: MinutesUsedMeter
status: proposed
ios_symbol: MinutesUsedMeter
ios_file: ios/Intrada/Views/Components/MinutesUsedMeter.swift
ios_snapshots: ComponentSnapshotTests/testMinutesUsedMeter.1.png
---

# MinutesUsedMeter

Minutes used against the session length, in words and as a bar.

## When to use

- Under a plan with a session length, in the builder's footer and in an item's card while its minutes change.

## When not to use

- Progress through a session as it plays: use `SegmentedProgress`.
- Minutes practised over weeks: use `ConsistencyBars`.

## Content

- The core's summary on the left, such as "72 of 60 min, plus 2 untimed", in `IntradaFont.label` and `ink`.
- On the right, the state in words. Over: "12 min over" in `danger`, `IntradaFont.label`, with a warning glyph. Under: "45 min left" in `inkSecondary`, `IntradaFont.secondary`. Exact: "Fits exactly" in the same.
- The two lines stack when they do not fit side by side.
- The bar is 6 points tall on `track`: the minutes up to the length in `ink`, and past it a separate `danger` stretch after a small gap. The bar's full width is the larger of the used minutes and the length.
- The summary, the minutes and the state all come from the core's fit view; the meter does no sums beyond the words.

## Accessibility

- VoiceOver reads the summary and the state as one line. The bar is hidden, since the words carry it.
- The text scales with Dynamic Type.

## Usage

```swift
if let fit = setlist.fit {
  MinutesUsedMeter(
    summary: fit.summary, usedMins: Int(fit.usedMins), lengthMins: Int(fit.lengthMins),
    state: fit.state)
}
```

## Related

`OutlineButton` carries the offers that fit the plan to the length.

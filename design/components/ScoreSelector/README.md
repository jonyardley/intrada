---
name: ScoreSelector
status: stable
ios_symbol: ScoreSelector
ios_file: ios/Intrada/Views/Components/ScoreSelector.swift
ios_snapshots: ComponentSnapshotTests/testScoreSelector.1.png
android_symbol: ScoreSelector
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PlayerParts.kt
android_snapshots: score-selector.png
decisions: T36
---

# ScoreSelector

Gives an item, a play or a whole session its mark: one box for each mark from 1 to 10.

## When to use

- Marking how it went, on the reflection page and the session summary: each item, each play of an item played more than once, and the overall session mark.

## When not to use

- Showing a mark already given: use `ScoreRing`.
- Choosing one of a few named options: use `SegmentedPills`.

## Variants

None. The caller passes the mark, the range, a spoken label and what to do with the choice.

## Content

- One box per mark, numbered. The range comes from the core (`LimitsView.scoreMin` and `scoreMax`, 1 to 10 today); the selector never assumes ten.
- No mark yet is 0 and fills nothing. Tapping the chosen mark again clears it.
- The look both apps follow is iPhone's: every box up to the chosen mark filled in ink with white figures, the rest outlined in `slotOutline`. Android fills only the chosen box today, sets the unchosen ones on a sunken fill and numbers them in typewriter figures; all three are A9 in #2606.
- The spoken labels are written by the shells: "Mark for " and the item or play, and "Overall session mark".

## Behaviour

On iPhone each tap plays the selection haptic. Android plays none; that is #2676.

## Accessibility

- iPhone reads the selector as one adjustable control: the label, then "4 of 10" or "not marked". Swiping up or down moves the mark.
- Android reads each box as its own control, "Mark for Clair de Lune, 4", selected or not. Which model both apps follow is #2675.
- The boxes are smaller than a thumb on both apps (32 points tall on iPhone); #2674.

## Do and don't

- **Do** give each selector a label that names what is being marked.
- **Don't** hard-code ten boxes; pass the core's range.

## Usage

```swift
ScoreSelector(
  score: mark(for: only.id), range: limits.scoreRange,
  accessibilityLabel: "Mark for \(itemTitle)"
) { next in
  setMark(next, for: only.id)
}
```

```kotlin
ScoreSelector(
    summary.sessionScore?.toInt() ?: 0,
    range,
    "Overall session mark",
    "summary.overall",
    onSelect = { send(Event.Session(SessionEvent.UpdateSessionScore(it))) },
)
```

## Related

`ScoreRing` shows the mark this sets. `FieldLabel` names it on the summary.

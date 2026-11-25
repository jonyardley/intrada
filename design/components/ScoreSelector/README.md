---
name: ScoreSelector
status: stable
ios_symbol: ScoreSelector
ios_file: ios/Intrada/Views/Components/ScoreSelector.swift
ios_snapshots: ComponentSnapshotTests/testScoreSelector.1.png
decisions: T36
---

# ScoreSelector

Gives an item, a play or a whole session its mark: one box for each mark from 1 to 10.

## When to use

- Marking how it went on the session summary: each item, each play of an item played more than once, and the overall session mark.

## When not to use

- Scoring on the reflection sheet, where the score starts on last time's or a suggested one: use `ScoreStepper`.
- Showing a mark already given: use `ScoreRing`.
- Choosing one of a few named options: use `SegmentedPills`.

## Variants

None. The caller passes the mark, the range, a spoken label and what to do with the choice.

## Content

- One box per mark, numbered. The range comes from the core (`LimitsView.scoreMin` and `scoreMax`, 1 to 10 today); the selector never assumes ten.
- No mark yet is 0 and fills nothing. Tapping the chosen mark again clears it.
- Every box up to the chosen mark is filled in ink with white figures, the rest outlined in `slotOutline`.
- The spoken labels are written by the shell: "Mark for " and the item or play, and "Overall session mark".

## Behaviour

Each tap plays the selection haptic.

## Accessibility

- iPhone reads the selector as one adjustable control: the label, then "4 of 10" or "not marked". Swiping up or down moves the mark.
- The boxes are smaller than a thumb (32 points tall); #2674.

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

## Related

`ScoreRing` shows the mark this sets. `FieldLabel` names it on the summary.

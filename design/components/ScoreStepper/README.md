---
name: ScoreStepper
status: stable
ios_symbol: ScoreStepper, TempoStepButton
ios_file: ios/Intrada/Views/Components/ScoreStepper.swift, ios/Intrada/Views/Components/TempoStepButton.swift
ios_snapshots: ComponentSnapshotTests/testScoreStepper.1.png, ComponentSnapshotTests/testScoreStepperSessionLength.1.png
decisions: T35, T39
---

# ScoreStepper

Nudges a number up or down from where it starts, with minus and plus in thin rings and a word under the number: where a score came from, or the unit of a length.

## When to use

- Scoring each key or variation of an item on the reflection sheet, where the number starts on last time's score or a suggested one and most keys need a small change or none.
- Setting the session length on Practice (#3007), where the musician nudges the usual length or types the minutes they have.

## When not to use

- Giving a mark from nothing, or the overall session mark: use `ScoreSelector`.
- Showing a score already given: use `ScoreRing`.
- A tempo: use `DragValue` (`TempoDrag`).

## Variants

- **Stepper** (`ScoreStepper`): use for a score. The caller passes the score, where it came from, the range, a spoken label and what to do with the change.
- **Session length** (`ScoreStepper` with a unit caption): Look B, 32 point, with typing, picked by the owner on 2026-11-26. The number is in the large clock font (`IntradaFont.timer(32)`) with "minutes" under it, the step is the core's length step, and a tap on the number opens the number pad.
- **Ring button alone** (`TempoStepButton`): use for minus or plus beside another control, such as the metronome's readout while it sounds. The disabled state is an option on it.

## Behaviour

- Minus and plus move the number by the step (one for a score) and play the selection haptic. From a typed number off the step, such as 17 with a step of 5, plus gives 22: the stepper adds and clamps and never snaps.
- At the bottom of the range minus is disabled, at the top plus is; the glyph turns `inkFainter` and the ring stays, so the control keeps its shape.
- Holding minus or plus repeats the step after a moment, then faster, with the selection haptic on each step, and stops at the range end. Letting go after a hold adds no extra step. The metronome's rings do not repeat.
- For a score the number itself does nothing when tapped. Leaving a key's score unchanged keeps last time's score on save (decided on #3004), so no tap is needed to agree with it.
- With typing on, a tap on the number opens the number pad in its place, with Done on the keyboard bar, as `DragValue` does. Each keystroke sends the typed number clamped to the range, so Start tapped while typing plays it; an empty entry changes nothing. A tap on minus or plus while typing ends the typing and steps from the typed number, or from the number before typing when the entry is empty.

## Content

- The score in DM Mono at 20 points (`IntradaFont.figureLarge`), between the two rings, with one word centred under it in `IntradaFont.small`. The value column is the shared stepper width (`StepperValue.width`), so the rings line up down a screen.
- The word: "last time" for last time's score, "suggested" for a key never scored, "today" once the musician changes it.
- "last time" and "suggested" draw the number and the word in `inkSecondary`; "today" draws both in `ink`, the word in `smallMedium`. The word tells them apart as well as the colour (T39).
- A unit caption, such as "minutes", draws the number in `ink` and the word in `inkSecondary` at `IntradaFont.small`.
- Each ring is the shared stepper button: an ink glyph in a 32 point circle stroked one point in `divider`, on no box or fill.
- The stepper keeps its own width, so at large text sizes the key's name beside it wraps instead of the word spilling under the rings.
- The range comes from the core (`LimitsView.scoreMin` and `scoreMax`, 1 to 10 today); the stepper never assumes ten. The starting score and the word are worked out by the core.
- The pictures show each word and the top of the range, with plus disabled, and the session length at 20 minutes and at the bottom of its range.

## Accessibility

- VoiceOver reads it as one adjustable control: the label, then the score and its word, "Score for D major, 5, last time". Swiping up or down moves the number by the step and stops at the range ends. The caller can pass its own spoken value, as "20 minutes" for the session length.
- VoiceOver adjusts rather than types, so the number pad is never announced.
- Each ring has a 44 point tap area.
- The number and the word scale with Dynamic Type.

## Do and don't

- **Do** pass the core's range and starting score.
- **Do** give each stepper a label naming the key: "Score for D major".
- **Don't** hard-code 1 to 10, or work out the starting score in Swift.

## Usage

```swift
ScoreStepper(
  score: row.score, origin: .lastTime, range: limits.scoreRange,
  accessibilityLabel: "Score for \(row.keyName)"
) { next in
  setMark(next, for: row.id)
}

ScoreStepper(
  score: Int(plan.lengthMins), caption: .unit("minutes"), range: Int(limits.sessionLengthMinMins)...Int(limits.sessionLengthMaxMins),
  step: Int(limits.sessionLengthStepMins), figure: IntradaFont.timer(32), typing: true,
  accessibilityLabel: "Session length", accessibilityValue: "\(plan.lengthMins) minutes"
) { next in
  store.send(.session(.setPlanLength(lengthMins: UInt16(clamping: next))))
}
```

## Related

The metronome's readout shares the ring buttons; `DragValue` sets a tempo. `ScoreSelector` gives a mark from nothing. `ScoreRing` shows a score already given.

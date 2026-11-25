---
name: TempoStepper
status: stable
ios_symbol: TempoStepper, TempoStepButton
ios_file: ios/Intrada/Views/Components/TempoStepper.swift, ios/Intrada/Views/Components/TempoStepButton.swift
ios_snapshots: ComponentSnapshotTests/testTempoStepper.1.png
decisions: T16, T39
---

# TempoStepper

Sets a tempo: minus and plus in thin rings either side of the number, held to repeat, or the number typed.

## When to use

- The tempo a key or a play reached, on the reflection sheet.
- Any tempo the musician corrects by a few beats, or types when it is far off.

## When not to use

- Starting, stopping and nudging the metronome while it plays: use the click control, which uses the same ring buttons.
- A score: use `ScoreStepper`.

## Variants

- **Stepper** (`TempoStepper`): use for a tempo value.
- **Ring button alone** (`TempoStepButton`): use for minus or plus beside another control, such as the click control. Hold to repeat and the disabled state are options on it.

## Behaviour

- One tap moves the tempo by the core's step (`LimitsView.clickTempoStep`, 2 beats a minute today), as the metronome does, and plays the selection haptic.
- Holding minus or plus repeats: the first repeat comes after 0.4 seconds, then one every 0.12 seconds, then one every 0.04 seconds once held for a second.
- Tapping the number opens the number pad. The typed number is kept as typed and clamped to the band when typing ends, with Done above the keyboard or a tap on minus or plus.
- At either end of the band the button for that end is disabled and stops repeating.

## Content

- The core's readout split in two: the note and equals sign, "♩ =" or "♪ =" by the bar's unit, then the number. Both in `ink` in DM Mono at 20 points (`IntradaFont.figureLarge`), as the score stepper's number is, so every stepper value reads the same.
- Each ring is an ink glyph in a 32 point circle stroked one point in `divider`, on no box or fill (T39). Disabled, the glyph turns `inkFainter` and the ring stays.
- The band comes from the core per beat unit (`LimitsView.clickBand(unit:)`); the stepper never assumes its ends.
- The value sits in the shared stepper value column (`StepperValue.width`, at least 88 points, wider only when larger text needs it), so the rings of every stepper on a screen line up. The caller places it, trailing its row.
- The picture shows a crotchet tempo, a quaver tempo and the top of the band, with plus disabled.

## Accessibility

- VoiceOver reads it as one adjustable control: the label, then the core's spoken tempo, "Tempo for D major, 88 crotchet beats per minute". Swiping up or down moves it by one step.
- Typing is for sight; VoiceOver adjusts instead.
- Each ring has a 44 point tap area.

## Do and don't

- **Do** pass the core's step and band.
- **Don't** put a white box or a filled panel round the rings (T39).

## Usage

```swift
TempoStepper(
  value: tempoBinding(for: play.id), unit: row.click.metre.unit,
  step: limits.clickStep, band: row.band.range,
  accessibilityLabel: "Tempo for \(rowTitle(play))"
)
```

## Related

`ScoreStepper` shares the ring buttons. `FieldLabel` names the tempo beside it.

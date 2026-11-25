---
name: DragValue
status: proposed
ios_symbol: DragValue, TempoDrag
ios_file: ios/Intrada/Views/Components/DragValue.swift
ios_snapshots: ComponentSnapshotTests/testTempoDrag.1.png, FocusPlayerSnapshotTests/testClickControlStates.1.png, FocusPlayerSnapshotTests/testClickControlDraggingStates.1.png
decisions: T16
---

# DragValue

A number dragged up or down in steps, with a faint up and down glyph saying it can be dragged; a tap either does the caller's action or opens the number pad.

## When to use

- A tempo: the metronome's readout in the focus player, and the tempo reached on the reflection sheet and the key sheet (`TempoDrag`).
- Any number the musician moves by a few steps, where one compact readout reads better than minus and plus.

## When not to use

- A score on the reflection sheets: use `ScoreStepper`, until the owner picks between it and a drag value (#3004).
- A choice from a few named options: use `SegmentedPills`.

## Variants

- **Drag value** (`DragValue`): use for any stepped number; the caller draws the readout.
- **Tempo** (`TempoDrag`): use for a tempo the musician corrects; the core's readout in DM Mono, typed on a tap.

## Behaviour

- Dragging up raises the value and dragging down lowers it, one step for every 8 points of travel, anchored to where the drag began so steps never drift. Each step crossed plays the selection haptic.
- While dragging, the readout turns accent, the capsule washes in the accent, lifts with a shadow and grows slightly, and the glyph hides.
- The caller can cap how often a drag writes (the metronome keeps to its lead-in); the last step always lands on release.
- A tap runs the caller's action with the impact haptic, as starting the metronome, or opens the number pad. A typed number is kept as typed and clamped to the range when typing ends, with Done above the keyboard.
- The range comes from the core; the value never passes either end.

## Content

- The caller's readout, then `arrow.up.and.down` at caption size in `inkFaintIcon`, in a capsule padded by `IntradaSpacing.cardCompact`, at least 44 points tall.
- At rest the capsule is clear, or the caller's fill (the musician's highlighter while the metronome sounds). Dragging, it is `accent` at `IntradaOpacity.wash` with the lifted shadow.
- `TempoDrag` shows the core's readout, "♩ = 88" or "♪ = 168" by the bar's unit, in `ink` in DM Mono at 20 points (`IntradaFont.figureLarge`). The metronome keeps its own readout in Hanken Grotesk.
- The picture shows a crotchet tempo, a quaver tempo and the top of the band.

## Accessibility

- VoiceOver reads it as one adjustable control: the label, then the spoken value, "Tempo for D major, 88 crotchet beats per minute". Swiping up or down moves it by one step.
- A drag and typing are for sight; VoiceOver adjusts instead.
- The capsule is at least 44 points tall.

## Do and don't

- **Do** pass the core's step and range.
- **Don't** put a white box or a filled panel round it at rest.

## Usage

```swift
TempoDrag(
  value: tempoBinding(for: play.id), unit: row.click.metre.unit,
  step: limits.clickStep, band: row.band.range,
  accessibilityLabel: "Tempo for \(rowTitle(play))")
```

## Related

`ScoreStepper` nudges a score with the ring buttons. `StepperRowLabel` names the tempo beside it.

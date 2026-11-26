---
name: ReflectionKeySheet
status: stable
ios_symbol: ReflectionKeySheet
ios_file: ios/Intrada/Views/Screens/ReflectionKeySheet.swift
ios_snapshots: SessionSummarySnapshotTests/testReflectionKeySheet.1.png, SessionSummarySnapshotTests/testReflectionKeySheetRefusingANote.1.png
decisions: T39, T40
---

# ReflectionKeySheet

One key's tempo and note on the item-complete sheet, opened by tapping the key's name.

## When to use

- Correcting the tempo of one key or variation after an item played in several, or writing a note for that key alone.
- Changing which key or variation a row says was played, where the core offers that change.

## When not to use

- The score: it stays on the row, with `ScoreStepper`, so it can be changed without opening anything.
- The reflection for the whole item: that is the note field on the item-complete sheet.
- Editing a saved session's entry: use the entry sheet on the summary.

## Behaviour

- A `BottomSheet` that stays open on Done. Done sends the draft with this key's tempo and note to the core and closes only once the core keeps it.
- A refusal shows at the top of the sheet in a `FormErrorBanner` naming the key, and the sheet stays open with the note as typed so it can be shortened.
- A change of key or variation goes to the core straight away, so the choices read back from the core's row.
- Swiping the sheet down drops an unsaved tempo or note.
- The note field stops at the core's limit (`LimitsView.playNoteMaxChars`, 500 today), counted as the core counts, so a pasted note is cut to fit.

## Content

- The key's name is the sheet's title.
- The tempo row: "Tempo reached" leading and a `TempoDrag` trailing on the shared right edge. No tempo row when the play has no tempo to correct.
- The sheet opens at the height of its content and drags up to full height for the keyboard or a long note.
- The note: a `FieldCard` labelled "Note for D major", placeholder "Add a note", two to four lines in `IntradaFont.body`.
- The refusal reads "Couldn't keep the note for D major." and then the core's message.

## Accessibility

- The tempo stepper's label names the key: "Tempo for D major".
- The note field's label is its card label, "Note for D major".

## Do and don't

- **Do** read the limit and the starting values from the core.
- **Don't** close the sheet before the core answers, or show the refusal on the sheet underneath.

## Usage

```swift
ReflectionKeySheet(
  title: "D major", tempoRow: row, limits: limits,
  tempo: TrackedTempo(row: row), note: note,
  wayRow: nil, wayChoices: choices,
  onWay: { next in sendWay(next) },
  onDone: { tempo, note in sendDraft(tempo, note) }
)
```

## Related

`BottomSheet` is its chrome. `DragValue` (`TempoDrag`) sets the tempo. `ScoreStepper` scores the key on the row beneath.

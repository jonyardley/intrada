---
name: SegmentedPills
status: stable
ios_symbol: SegmentedPills
ios_file: ios/Intrada/Views/Components/SegmentedPills.swift
ios_snapshots: ComponentSnapshotTests/testSegmentedPills.1.png
decisions: T24, T36
---

# SegmentedPills

Choosing one of two to five options in place, with the choice showing at once.

## When to use

- A mode on the screen you are on: a piece or an exercise on the add form.
- A small fixed set of values, such as a time signature grouping or the practice defaults.

## When not to use

- More than five options, or options that come from the musician's own data: use a list with `TickRow` in a sheet.
- An on or off setting: use a toggle.
- An action: use a button. A pill changes what you see; it never saves or sends.

## Variants

- **Full width track** (`.fullWidthTrack`): use for a form choice. Equal segments.
- **Wrapping track** (`.wrappingTrack`): use when a label can be long, such as a grouping like 2 + 2 + 3. Whole pills move to a new row rather than a label breaking.

Every caller names its track; there is no default.

## Behaviour

- Tapping a pill chooses it at once. On iPhone the fill slides to it with `IntradaMotion.snappy`, and does not move under Reduce Motion.
- No haptic.

## Content

- The track is a white card with a `hairline` edge and round ends. The chosen pill is filled in ink with white text; the others are text in `inkSecondary`. Ink on white is the interactive colour (T24 in [the design principles](../../../docs/design-principles.md)), so the chosen option reads as chosen at a glance.
- No pill is ever filled with the highlighter, which is kept for the one call to action (`MarkerButton`).
- There is no disabled state: if an option cannot be chosen, leave it out.
- One or two words per label, sentence case. No icons alone; an icon pill needs a text label for VoiceOver.

## Accessibility

- Each pill is a button with the selected trait, so a screen reader says which is chosen.
- `hint` adds a spoken hint to the label. Use it when the label alone is unclear.
- The touch area reaches 44 points without the drawn pill growing; reaching it is #2686.

## Do and don't

- **Do** keep labels the same length where you can, so the segments look even.
- **Don't** use pills for a choice that needs explaining; a sheet with rows has room for a line each.

## Usage

```swift
SegmentedPills(
  options: [.piece, .exercise], selection: $kind, label: \.label,
  font: IntradaFont.segment, layout: .fullWidthTrack)
```

## Related

`TickRow` for longer choices, `ScoreSelector` for a score.

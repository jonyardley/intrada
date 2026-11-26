---
name: SwitchRow
status: stable
ios_symbol: SwitchRow
ios_file: ios/Intrada/Views/Components/SwitchRow.swift
ios_snapshots: ComponentSnapshotTests/testSwitchRow.1.png
decisions: T35
---

# SwitchRow

A setting the musician turns on or off, with its name beside the switch, inside a card.

## When to use

- A yes or no setting in a card: a session item's planned duration, whether an item joins the warm-up, whether a session starts with one.

## When not to use

- Choosing one of two to five named options: use `SegmentedPills`.
- An action that happens once: use `MarkerButton` or a plain button.

## Behaviour

- Bind it to the core's value when the setting is one, so a refused write snaps back and the refusal shows on the global banner (#1736).
- The system switch's own motion; no haptic.

## Content

- The name is a `FieldLabel`: `IntradaFont.label` in `inkSecondary`. The switch is tinted `accent` when on.
- A name in sentence case, one line. A question mark is fine where the setting reads as one, "Include in warm-up?".
- The row sits inside a card the caller draws, with `IntradaSpacing.card` padding when it stands alone.

## Accessibility

- VoiceOver reads the name and the switch's state as one control.
- Every row takes an accessibility identifier, so a UI test can find it.
- Scales with Dynamic Type.

## Usage

```swift
SwitchRow(
  title: "Include in warm-up?",
  isOn: Binding(get: { row.includeInWarmUp }, set: setIncludeInWarmUp),
  identifier: "libraryDetail.warmUp.toggle")
```

## Related

`FieldLabel`, which draws its name. `SegmentedPills`, for two to five named options.

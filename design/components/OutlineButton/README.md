---
name: OutlineButton
status: proposed
ios_symbol: OutlineButton
ios_file: ios/Intrada/Views/Components/OutlineButton.swift
ios_snapshots: ComponentSnapshotTests/testOutlineButton.1.png
---

# OutlineButton

A small outlined action under the content it changes, such as "Trim to fit" under the minutes used bar.

## When to use

- An offer that changes what sits just above it, one or two side by side, where a full width button would outweigh the content.

## When not to use

- The one call to action on a screen: use `MarkerButton`.
- Adding a row to a list: use `AddRowButton`.
- An action inside a row beside its content: use `CapsuleActionButton`.

## Behaviour

- A tap runs the action with the shared press rebound. The action decides any haptic, after the core confirms.

## Content

- The title in `IntradaFont.label`, `accent` ink, on one line, in a one point `divider` outline with the `IntradaRadius.card` corner and no fill.
- Insets of `cardCompact` across and `small` up and down. A few words, a verb first: "Trim to fit", "Drop last item".

## Accessibility

- VoiceOver reads the title as a button.
- The tap area is at least 44 points tall; the outline is drawn inside it.
- The title scales with Dynamic Type. Two buttons that no longer fit side by side stack, which the caller arranges.

## Usage

```swift
OutlineButton(title: "Trim to fit") {
  store.send(.session(.trimToFit))
}
```

## Related

`MinutesUsedMeter` sits above the offers. `CapsuleActionButton` is the smaller action inside a row.

---
name: MarkerButton
status: proposed
ios_symbol: MarkerButton, BrandBarButton
ios_file: ios/Intrada/DesignSystem/MarkerButton.swift, ios/Intrada/Views/Components/BrandBarButton.swift
ios_snapshots: ComponentSnapshotTests/testPrimaryButtons.1.png
decisions: T24
---

# MarkerButton

The one call to action on a screen, filled with the musician's chosen highlighter.

## When to use

- The action the screen exists for: start the session, save the reflection, continue in the first run.
- At most one per screen or sheet.

## When not to use

- A second or quieter action: use a plain ink text button.
- Choosing an option: use `SegmentedPills` or `TickRow`. A selected choice is never highlighter.
- Anything destructive: use `DeleteButton` and a confirm.

## Variants

One: full width, with a text label only. `MarkerButton` takes no glyph.

iPhone's `BrandBarButton` is the version with a glyph: a leading play, timer or plus, or a trailing arrow to move on. It also draws a drop shadow and has no press spring.

One button, named `MarkerButton`. `BrandBarButton` folds into it in #2684. Whether it has a lift and a bounce is settled in the refinement pass, #2638.

## Behaviour

- Tapping it sends one event to the core. Any success haptic fires only after the core confirms.
- Pressed, it changes no colour. `MarkerButton` also shrinks slightly and springs back (`PressRebound`); whether it keeps that is #2638.

## Content

- A verb first, two or three words: "Start session", "Save session". A count or time can follow after a middle dot: "Start session · 10 min".
- The fill is the musician's highlighter, from `\.marker`; butter by default.
- The label is `IntradaFont.button` in `onMarker` ink. The corner is `IntradaRadius.card`, 3 points, on `BrandBarButton` too. On `BrandBarButton` a glyph is the same ink.
- No disabled state: leave the button out, or say on the screen what is missing.

## Accessibility

- The label is the spoken name; a `BrandBarButton` glyph adds nothing to it.
- At least 44 points tall: the padding gives this.
- Ink on butter is 11.3:1 (T24 in [the design principles](../../../docs/design-principles.md)). No test yet checks ink on the other seven highlighters; one should land before this page moves to stable.

## Do and don't

- **Do** put it at the bottom of the content it acts on, or pinned above the keyboard on a form.
- **Don't** put two on one screen, or a highlighter fill on anything else that can be pressed.

## Usage

```swift
MarkerButton("Save session") {
  store.send(.session(.saveSession(now: SessionClock.nowRFC3339())))
}
.accessibilityIdentifier("summary.save")
```

## Related

`DeleteButton`, `CapsuleActionButton`, `AddRowButton`.

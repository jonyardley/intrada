---
name: HairlineDivider
status: stable
ios_symbol: HairlineDivider
ios_file: ios/Intrada/DesignSystem/HairlineDivider.swift
ios_snapshots: ComponentSnapshotTests/testHairlineDivider.1.png
android_symbol: HairlineDivider
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/HairlineDivider.kt
android_snapshots: hairline-divider.png
decisions: T24
---

# HairlineDivider

The 1 point line between rows in a card, and between columns.

## When to use

- Between the rows of one card: fields on a form, sessions in a list, sections on a piece.
- Between controls or columns side by side, such as the beat dots and the settings glyph on the metronome bar, or the columns of the iPad split.

## When not to use

- Between cards: the gap between them does that, `IntradaSpacing.cardCompact` (12).
- Under a page title: `ScreenScaffold` draws its own rule.
- Round a card: `cardSurface()` draws its own edge.

## Variants

- **Across** (the default): use between rows.
- **Upright** (`axis: .vertical`, `Orientation.Vertical`): use between columns or controls side by side.
- **Darker** (`colour: IntradaColor.divider`): use for a stronger line on a busy bar, such as the metronome bar.
- **Inset:** use where the line should start with the row's text; the caller pads the leading edge by `IntradaSpacing.card` (16).

Android's metronome bar draws its upright line by hand; moving it onto this component is #2661.

## Behaviour

No motion. Lists draw it before every row but the first (`if index > 0`), so the card's own edge closes the list.

Whether a divider should be inset at all is open, with the refinement observations in #2608; until it is settled, the inset variant stays as both apps ship it.

## Content

- One line, 1 point thick, in `hairline` (#ECE6DA) by default or `divider` (#DDD7CA) when darker, filling the width (or height, upright) of whatever holds it.
- No states of its own.

## Accessibility

Decorative: it adds nothing to VoiceOver or TalkBack. It does not scale with text size.

## Do and don't

- **Do** use it inside a card, between rows that belong together.
- **Don't** draw a line with a `Rectangle` or a `Box` of your own; that is the copy that drifts.

## Usage

```swift
if index > 0 { HairlineDivider() }
HairlineDivider().padding(.leading, IntradaSpacing.card)
HairlineDivider(axis: .vertical, colour: IntradaColor.divider)
```

```kotlin
if (index > 0) HairlineDivider()
HairlineDivider(Modifier.padding(start = IntradaSpacing.card))
```

## Related

`Card`, whose rows it separates.

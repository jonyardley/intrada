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

The one-pixel line between rows in a card, and between columns.

## When to use

- Between the rows of one card: fields on a form, sessions in a list, sections on a piece.
- Between the beat dots and the settings glyph on the metronome bar, and between the columns of the iPad split, upright.

## When not to use

- Between cards: the gap between them does that, `IntradaSpacing.cardCompact` (12).
- Under a page title: `ScreenScaffold` draws its own rule.
- Round a card: `cardSurface()` draws its own edge.

## Anatomy

One line, 1 point thick, in `hairline` (#ECE6DA) by default, filling the width (or height, upright) of whatever holds it.

## Variants

- **Across** (the default): use between rows.
- **Upright** (`axis: .vertical`, `Orientation.Vertical`): use between columns or controls side by side.
- **Darker** (`colour: IntradaColor.divider`, #DDD7CA): a stronger line; the metronome bar is its one caller.
- **Inset:** the caller pads the leading edge by `IntradaSpacing.card` (16) so the line starts where the row's text does.

## States

None.

## Behaviour

No motion. Lists draw it before every row but the first (`if index > 0`), so the card's own edge closes the list.

iPhone uses it at 50 call sites in 31 files: 7 inset at the leading edge and 1 at both edges (the variation picker). Android uses it at 45 call sites in 26 files: 5 inset at the start and 1 at both edges (the variation picker in `PlayerSheets.kt`).

**On 14 October 2026 Android uses only the default.** On iPhone the upright line has two callers, the iPad split (`ListDetailSplit`) and the metronome bar (`ClickBarLine`), and the darker colour one, the metronome bar. No Android caller passes `orientation` or `colour`; Android's metronome bar draws the same upright line by hand instead (`ClickBarLine.kt`, a `Box` 1dp wide in `divider`). The two parameters stay for that line, and for the split when Android gets one; moving the metronome bar onto `HairlineDivider` is not in an issue yet.

The audit found the Practice defaults dividers on Android inset at the start only, so the card looks unfinished (note 9 of the Android refinement observations in [the October audit](../../../docs/audit-2026-10-design.md)). It is not in an issue yet.

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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records that Android's metronome bar draws its upright line by hand. |

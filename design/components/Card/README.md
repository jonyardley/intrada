---
name: Card
status: stable
ios_symbol: cardSurface, cardShadow
ios_file: ios/Intrada/DesignSystem/CardSurface.swift, ios/Intrada/DesignSystem/CardShadow.swift
ios_snapshots: ComponentSnapshotTests/testCard.1.png
android_symbol: cardSurface, cardShadow
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/CardSurface.kt, android/app/src/main/kotlin/com/intrada/android/ui/components/CardShadow.kt
android_snapshots: card-surface.png, card-shadow.png
decisions: T24, T36
---

# Card

The white surface every grouped block of content sits on, lifted faintly off the paper.

## When to use

- A group of related content that reads as one thing: a library row, a detail section, a form's fields, a session.
- A sheet's body, a dialog, a banner that sits on paper.

## When not to use

- A single line of text or a section title: use `SectionTitle` on the paper.
- Rows inside a card: separate them with `HairlineDivider`, not a card each.
- A form field: use `FieldCard`, which is a card with the label inside.

## Anatomy

1. **Fill:** `cardFill`, white.
2. **Edge:** a one point `hairline` stroke along the corner (inside it on Android, centred on the edge on iPhone).
3. **Corner:** `IntradaRadius.card`, 3 points.
4. **Lift:** `IntradaShadow.card`, radius 1, one point down, through `cardShadow()`.

## Variants

- **Card with lift** (`cardSurface()` with `cardShadow()`): use for every card on paper. This is the look T24 decided ("white cards with a 3pt corner and a faint lift").
- **Card without lift** (`cardSurface()` alone): use only inside something already lifted, such as a sheet or a card in a card.
- **Custom corner** (`cardSurface(cornerRadius:)`): every caller today passes a 3 point radius, so it changes nothing; #2635 decides whether to keep it.

## States

A card has no states of its own. A pressable card (a library row) uses the press style of the control inside it.

## Behaviour

None. Motion belongs to what the card holds. A card that slides or fades in keeps its opaque fill, so the transition never ghosts.

**On 13 October 2026 no card on either app has the lift** (I36 in [the October audit](../../../docs/audit-2026-10-design.md)). On iPhone, 67 lines call `cardSurface()`; the four `cardShadow()` calls are on a capsule, a disc, a header and a hand-drawn card, none of them a `cardSurface()` card. On Android, 68 lines call `cardSurface()` and the one `cardShadow()` call is on the key wheel. Restoring the lift everywhere is the first step of the refinement pass (#2638, direction 6a), not a fix on its own.

## Content

Content sits `IntradaSpacing.card` (16) in from the edge, or `cardCompact` (12) in a dense list. A card's lead line is `cardTitle`; secondary lines are `inkSecondary`.

## Accessibility

A card adds nothing to VoiceOver or TalkBack. When a whole card is one action, combine its children into one element with one label.

## Do and don't

- **Do** put a group of rows in one card, separated by hairlines.
- **Don't** stack equal cards with no lead line (audit refinement note 2, the piece detail page).
- **Don't** draw a card by hand with a background, a clip and a stroke: that is the copy that drifts.

## Usage

```swift
VStack(alignment: .leading) { content }
  .padding(IntradaSpacing.card)
  .cardSurface()
  .cardShadow()
```

```kotlin
Column(Modifier.cardShadow().cardSurface().padding(IntradaSpacing.card)) { content() }
```

On Android the shadow comes first in the modifier chain, so it is drawn outside the clipped surface.

## Related

`FieldCard`, `HairlineDivider`, `SectionTitle`.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-13 | #2620 | First page. Records that most cards lack the lift (I36). |

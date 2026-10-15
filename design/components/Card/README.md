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

## Variants

- **Card with lift** (`cardSurface()` with `cardShadow()`): use for every card on paper. This is the look T24 in [the design principles](../../../docs/design-principles.md) decided ("white cards with a 3pt corner and a faint lift").
- **Card without lift** (`cardSurface()` alone): use only inside something already lifted, such as a sheet or a card in a card.

Every card has the same 3 point corner. The corner parameter goes from both apps in #2664.

## Behaviour

None. Motion belongs to what the card holds. A card that slides or fades in keeps its opaque fill, so the transition never ghosts.

No card on either app is drawn with the lift yet; restoring it everywhere is part of the refinement pass, #2638.

## Content

- The fill is `cardFill`, white, with a one point `hairline` edge, the `IntradaRadius.card` corner (3 points) and the `IntradaShadow.card` lift through `cardShadow()`.
- Content sits `IntradaSpacing.card` (16) in from the edge, or `cardCompact` (12) in a dense list.
- A card's lead line is `cardTitle`; secondary lines are `inkSecondary`.
- A card has no states of its own. A pressable card (a library row) uses the press style of the control inside it.

## Accessibility

A card adds nothing to VoiceOver or TalkBack. When a whole card is one action, combine its children into one element with one label.

## Do and don't

- **Do** put a group of rows in one card, separated by hairlines.
- **Don't** stack equal cards with no lead line.
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

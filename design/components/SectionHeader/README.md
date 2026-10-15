---
name: SectionHeader
status: stable
ios_symbol: SectionHeader
ios_file: ios/Intrada/Views/Components/SectionHeader.swift
ios_snapshots: ComponentSnapshotTests/testSectionHeader.1.png, ComponentSnapshotTests/testSectionHeaderAtAccessibilitySize.1.png, LibraryDetailSnapshotTests/testUsedInCardRowStates.1.png
android_symbol: SectionHeader, HeaderAction
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PageParts.kt
android_snapshots: section-header.png
decisions: T29, T35
---

# SectionHeader

Names a section above its card, with an optional count, a fact at the trailing edge, or one text action.

## When to use

- A section that carries more than its name: a count ("Used in · 3 pieces"), a fact ("best week · 95m", "2 of 4 solid"), or an action (Reorder, Choose, Add, Edit).
- On Android, also a section that carries only its name, until Android has `SectionTitle`; #2636 builds it or records why not.

## When not to use

- A section with only a name, on iPhone: use `SectionTitle`, unless it sits in a run of headers that carry more.
- A field's label inside its card: use `FieldLabel` in a `FieldCard`.
- The page title: that is `ScreenScaffold`'s.

## Variants

- **Title only:** use where a section sits in a run of headers that carry more, so the titles line up.
- **Caption:** use for a count that qualifies the title, straight after it.
- **Trailing fact:** use for a summary of the whole section, at the trailing edge.
- **Action:** use for one text action on the section, at the trailing edge.

## Behaviour

- At accessibility text sizes the header stacks: the title and action on the first line, the caption and fact each on a line below, so a long title never breaks mid-word. The caption drops its dot when it stacks.
- An action that is off is hidden (`isDisabled`, `HeaderAction.enabled`), so the header never shows a control that does nothing.
- No motion and no haptic of its own.

Stacking, reading the caption and hiding an off action are the rules; Android follows in #2660.

## Content

- The title is `IntradaFont.label` in `inkSecondary`, sentence case, one to three words. It is drawn by `SectionTitle` on iPhone and `FieldLabel` on Android, so the two match.
- The caption and the trailing fact are `IntradaFont.secondary` in `inkSecondary`.
- The caption follows one middle dot with single spaces, "Used in · 3 pieces" ([tone of voice](../../../docs/tone-of-voice.md), rule 5). The iPhone double gap before the dot is #2608; the dot on Android is #2658.
- The action is one verb in `accent` at 15 point, the size T35 in [the design principles](../../../docs/design-principles.md) gives buttons, in a 48dp target on Android. The iPhone action at 15 point is #2659.
- The core writes captions and facts that are data ("2 of 4 solid", "best week · 95m"); the shells write the rest.

## Accessibility

- The title, caption and fact are read in order. A caller can hide a bare count from VoiceOver when the content says it better (`captionAccessibilityHidden`).
- The action is its own button, read as its spoken label ("Reorder or remove sections").
- Every part scales with Dynamic Type and font scale.

## Do and don't

- **Do** keep to one action; a second goes in the section's content.
- **Do** put a count that qualifies the title, such as how many pieces use an exercise, in the caption, and a summary of the whole section, such as "5 of 15 solid" or "3 days practised", in the trailing fact.

## Usage

```swift
SectionHeader(title: "Used in", caption: pieceCountCaption)
SectionHeader(title: "Sections", action: headerAction)
```

```kotlin
SectionHeader("Last five weeks", trailing = trailing)
SectionHeader(
    "Keys",
    caption = item.keysCaption,
    action = HeaderAction("Choose", "Choose keys", "keys.header", perform = onChoose),
)
```

## Related

`SectionTitle` for a name alone. `FieldLabel`, which draws the same title inside a card.

---
name: SectionTitle
status: stable
ios_symbol: SectionTitle
ios_file: ios/Intrada/Views/Components/SectionHeader.swift
ios_snapshots: ComponentSnapshotTests/testSectionTitle.1.png
not_on_android: #2636 builds it; until then Android titles a section with SectionHeader or FieldLabel (see Behaviour).
decisions: T35
---

# SectionTitle

Names a section, in sentence case, above what it names.

## When to use

- A section with only a name: Recent sessions, What you played, Practice defaults.
- A name and one fact in a single line, joined by a middle dot: "Item complete · " and the time played, "Tempo reached · target " and the tempo, "Last practised · " and the day.
- A title on the dark Practice and Up next cards, with the light tint.

## When not to use

- A section with a count, a fact set apart on the right, or an action: use `SectionHeader`.
- A field's label inside its card: use `FieldLabel` in a `FieldCard`.
- The page title: that is `ScreenScaffold`'s.

## Variants

- **On paper** (the default): use for every section on a screen or sheet.
- **On a dark card** (`tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary)`): use only on the Practice and Up next cards, where T35 in [the design principles](../../../docs/design-principles.md) keeps the label light on dark.

## Behaviour

No motion and no haptic.

Android has no `SectionTitle` yet; building it is #2636. Until then a section with only a name uses `SectionHeader` with nothing else, or `FieldLabel`, and the dark cards draw their titles in `IntradaFont.label` with the same light tint. Each looks the same as on iPhone, because `SectionHeader`'s title is `FieldLabel`.

## Content

- One line of text: `IntradaFont.label` in `inkSecondary`. It draws `FieldLabel`, so a section title and a field label look the same (T35).
- A name, or name · fact: the name in sentence case, then one middle dot with single spaces and the fact, no full stop.
- Most names are one to three words; "From the chord chart" is the longest that ships. The shells write the titles.
- No states.

## Accessibility

- Read as plain text. On iPhone it does not carry the header trait.
- Scales with Dynamic Type and font scale.

## Do and don't

- **Do** use it for a name, or name · fact, and `SectionHeader` the moment the section gains a count or an action.
- **Don't** pass a font or colour of its own on paper; the tint is for the dark cards only.

## Usage

```swift
SectionTitle("Recent sessions")
SectionTitle("Then", tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
```

## Related

`SectionHeader` for a title with more. `FieldLabel`, which it draws.

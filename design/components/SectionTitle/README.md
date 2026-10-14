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
- A title on the dark Practice and Up next cards, with the light tint.

## When not to use

- A section with a count, a fact or an action: use `SectionHeader`.
- A field's label inside its card: use `FieldLabel` in a `FieldCard`.
- The page title: that is `ScreenScaffold`'s.

## Anatomy

One line of text: `IntradaFont.label`, 15 point medium, in `inkSecondary` (#6E6A66). It draws `FieldLabel`, so a section title and a field label look the same (T35 in [the design principles](../../../docs/design-principles.md)).

## Variants

- **On paper** (the default): use for every section on a screen or sheet.
- **On a dark card** (`tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary)`): use only on the Practice and Up next cards, where T35 keeps the label light on dark. It has 3 callers: the Practice card's heading and the Up next card's heading and "Then".

## States

None.

## Behaviour

No motion and no haptic.

iPhone uses it at 40 call sites in 25 files. 3 of them are `SectionHeader` drawing its own title, so screens and components call it 37 times in 24 files.

**On 14 October 2026 Android has no `SectionTitle`.** A section with only a name uses `SectionHeader` with no caption, fact or action (7 call sites in 3 files: Add to session, the icon picker and Practice defaults in `ProfileParts.kt`, and Profile's Highlighter and Beta), or `FieldLabel` directly, as the reflection sheet does. The dark cards draw their titles by hand in `IntradaFont.label` with the same light tint: the Practice card in `PracticeParts.kt` and the Up next card's heading and "Then" in `UpNextCard.kt`. Each looks the same as on iPhone, because `SectionHeader`'s title is `FieldLabel`. Building `SectionTitle` in Compose is #2636.

## Content

One to three words, sentence case, no full stop. The shells write the titles. The Practice card's heading joins "Last practised · " to the day the musician last practised (`relativeDay`).

## Accessibility

- Read as plain text. On iPhone it does not carry the header trait.
- Scales with Dynamic Type and font scale.

## Do and don't

- **Do** use it for a name alone, and `SectionHeader` the moment the section gains a count or an action.
- **Don't** pass a font or colour of its own on paper; the tint is for the dark cards only.

## Usage

```swift
SectionTitle("Recent sessions")
SectionTitle("Then", tint: IntradaColor.onAccent.opacity(IntradaOpacity.secondary))
```

## Related

`SectionHeader` for a title with more. `FieldLabel`, which it draws.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records how Android titles a section without it. |

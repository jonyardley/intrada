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

- A section that carries more than its name: a count ("Used in · 3 pieces"), a fact ("best week · 95m"), or an action (Reorder, Choose, Edit).
- On Android, also a section that carries only its name, until Android has `SectionTitle`.

## When not to use

- A section with only a name, on iPhone: use `SectionTitle`.
- A field's label inside its card: use `FieldLabel` in a `FieldCard`.
- The page title: that is `ScreenScaffold`'s.

## Anatomy

1. **Title:** `SectionTitle` on iPhone, `FieldLabel` on Android: `IntradaFont.label`, 15 point medium, in `inkSecondary` (#6E6A66), sentence case.
2. **Caption (optional):** `IntradaFont.secondary`, 15 point, in `inkSecondary`, straight after the title. A count that qualifies the title.
3. **Trailing fact (optional):** `IntradaFont.secondary` in `inkSecondary`, at the trailing edge. A comment on the section.
4. **Action (optional):** one text button in `accent` (`ink`, #2A2725) at the trailing edge.

## Variants

All four are in use on both apps:

- **Title only:** Recently practised and Library in Add to session.
- **Caption:** Used in, Keys, Variations and Related exercises on the piece page.
- **Trailing fact:** Last five weeks on Progress, the week and day lines on Practice.
- **Action:** Sections (Reorder, Done), Keys and Variations (Choose), and Related exercises on the piece page.

On iPhone the related exercises header also hides its bare count from VoiceOver (`captionAccessibilityHidden`); it is the one caller. On both apps the same header turns its action off while nothing is linked (`isDisabled`, `HeaderAction.enabled`), the one caller on each.

## States

Default, pressed and, for the action, off. An action that is off is hidden on iPhone and dimmed to `inkFainter` (#C2B8AA) on Android.

## Behaviour

- At accessibility text sizes on iPhone the header stacks: the title and action on the first line, the caption and fact each on a line below, so a long title never breaks mid-word (#1781). The caption drops its dot when it stacks.
- No motion and no haptic of its own.

iPhone uses it at 13 call sites in 10 files; Android at 15 call sites in 10 files. Android's include Profile's Highlighter and Beta sections and the Start here card, which on iPhone use `SectionTitle`.

**On 14 October 2026 the two apps differ:**

| | iPhone | Android |
|---|---|---|
| Caption | " · " before it, inside a row whose default spacing doubles the gap before the dot | no dot, 8dp after the title |
| Action type | `bodyMedium`, 17 point | `IntradaFont.label`, 15 point, in a 48dp target |
| Caption for TalkBack or VoiceOver | read, unless the caller hides it | never read |
| Action that is off | hidden | dimmed |
| Accessibility text sizes | stacks | stays on one line |

The look both apps keep: the caption after one middle dot with single spaces, "Used in · 3 pieces" ([tone of voice](../../../docs/tone-of-voice.md), rule 5). Closing the iPhone double gap is I21 in #2608 ([the October audit](../../../docs/audit-2026-10-design.md)); adding the dot on Android is not in an issue yet. The action takes 15 point, the size T35 gives buttons, in Android's 48dp target; iPhone's 17 point action is not in an issue yet. The iPhone stacking at large sizes, its choice of which captions VoiceOver reads, and hiding an action that is off, so the header never shows a control that does nothing, are the ones to keep; moving Android to them is not in an issue yet. The iPhone pictures show the double gap as it ships.

## Content

The title names the section in one to three words, sentence case. The core writes the keys and variations captions ("2 of 4 solid") and the best week ("best week · 95m"). The shells write the Practice week line ("3 days practised"), and the iPhone shell writes the Used in count ("3 pieces"). An action is one verb.

## Accessibility

- The title and fact are read in order, and the caption as the table above says. The action is its own button, read as its spoken label ("Reorder or remove sections").
- Every part scales with Dynamic Type and font scale.

## Do and don't

- **Do** keep to one action; a second goes in the section's content.
- **Don't** pass a count as the trailing fact; a count belongs against the title as the caption.

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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records the caption and action differences between the apps. |

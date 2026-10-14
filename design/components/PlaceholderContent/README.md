---
name: PlaceholderContent
status: stable
ios_symbol: PlaceholderContent, PlaceholderAction
ios_file: ios/Intrada/DesignSystem/PlaceholderContent.swift
ios_snapshots: ComponentSnapshotTests/testPlaceholderContent.1.png, ProgressSnapshotTests/testRoutinesScreen.1.png, LibrarySnapshotTests/testLibraryScreenSearchMatchesNothing.1.png
not_on_android: Android draws each empty screen by hand: one line of text on the Library (A21), a copy of this layout on Progress, and nothing on Routines; #2636 builds a shared version or records why not
decisions: T24, T35
---

# PlaceholderContent

Fills an empty screen or list with what belongs there and the one thing to do next.

## When to use

- A screen with nothing in it yet: the Library, Progress and Routines.
- A list or picker with no matches: the Library's filters and search, adding a related exercise, and adding to a session.
- A picker with nothing in it yet: the tag filter ("No tags yet.") and the linked item picker when it lists pieces for an exercise ("No pieces in your library yet."; the exercise picker offers to write one instead).
- The empty detail column of the iPad split, before anything is chosen.

## When not to use

- A card on a screen that has other content: the card says it in a line of its own, as Used in does.
- A loading wait or an error: use `GlobalBanner` for an error.

## Anatomy

1. **Glyph:** an SF Symbol at `IntradaIconSize.hero` (38 point, growing with Dynamic Type to 56), in `accent` (`ink`, #2A2725) at `IntradaOpacity.dimmed` (50%).
2. **Message:** `IntradaFont.body`, 17 point, in `inkSecondary` (#6E6A66), centred.
3. **First action (optional):** a `MarkerButton` in the musician's highlighter.
4. **Further actions (optional):** plain text buttons, `IntradaFont.bodyMedium` in `accent`, at least 44 points tall.
5. **Spacing:** `IntradaSpacing.cardCompact` (12) between glyph and message and between buttons, `section` (24) above the buttons, and 32 points round the whole, centred on the screen.

## Variants

- **Message only:** use for no matches and in the pickers.
- **With one action:** Progress and Routines, both "Build a session".
- **With two actions:** the empty Library, "Add piece or exercise" and "Scan a page".
- **Faint glyph** (`glyphTint`): the iPad split's empty detail column passes `inkFainter`, its one caller.

## States

None of its own. The buttons have their own pressed states.

## Behaviour

- The layout scrolls only when it cannot fit, so at large text sizes the buttons never squeeze the message.
- No motion and no haptic of its own; each action fires what its store event confirms.

On iPhone 9 call sites in 8 files use it: the Library (twice: empty, and no matches), Progress, Routines, the iPad split, the tag filter, the linked item picker, adding a related exercise and adding to a session.

**On 14 October 2026 Android has no placeholder component.** Each empty screen draws its own:

| Screen | iPhone | Android |
|---|---|---|
| Library, empty | glyph, message, "Add piece or exercise", "Scan a page" | the message alone, top left, nothing to tap |
| Progress, empty | glyph, message, "Build a session" | `ProgressEmpty`, a hand-built copy of this layout whose button is always butter, not the musician's highlighter |
| Routines | glyph, message, "Build a session" | the page title and nothing else (`EmptyTab`) |
| Exercise picker, add to session | glyph and message | the message alone |

The iPhone layout is the one both apps use. #2636 builds a shared Compose version or records why Android does not need one, and A21 in #2606 moves the empty Library onto it ([the October audit](../../../docs/audit-2026-10-design.md)). Moving Progress, Routines and the pickers onto it is not in an issue yet.

## Content

Three parts and nothing else: what belongs here, what to do, and the button ([tone of voice](../../../docs/tone-of-voice.md), rule 7). One sentence, no apology. The shell writes these messages. The Routines message offers to build a session, which is I33 in #2610.

## Accessibility

- The glyph and message are one element, read as the message. Each button is its own element.
- The message wraps and scales with Dynamic Type; the glyph grows with it to 56 points.

## Do and don't

- **Do** give the empty screen its one primary action as the first button.
- **Don't** apologise or explain why the screen is empty.

## Usage

```swift
PlaceholderContent(
  systemImage: "books.vertical",
  message: "Pieces and exercises will live here.",
  actions: [
    .init(title: "Add piece or exercise", identifier: "library.empty.add") { add() },
    .init(title: "Scan a page", identifier: "library.empty.scan") { add(opensCamera: true) },
  ])
```

```kotlin
// No shared placeholder; #2636 builds one or records why not.
```

## Related

`MarkerButton` for the first action. `ScreenScaffold`, whose content it fills on an empty screen.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records that Android draws each empty screen by hand. |

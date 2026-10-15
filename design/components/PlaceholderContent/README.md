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

- A screen with nothing in it yet.
- A list or picker with no matches, such as a filtered or searched library.
- A picker with nothing in it yet: "No tags yet.", "No pieces in your library yet."
- The empty detail column of the iPad split, before anything is chosen.

## When not to use

- A card on a screen that has other content: the card says it in a line of its own.
- A loading wait or an error: use `GlobalBanner` for an error.

## Variants

- **Message only:** use for no matches and in pickers.
- **With one action:** use where there is one clear next step, such as "Build a session".
- **With two actions:** use where there are two ways to begin, such as the empty Library's "Add piece or exercise" and "Scan a page".
- **Faint glyph** (`glyphTint`): use for a resting state that asks for nothing, such as the iPad split's empty detail column, in `inkFainter`.

## Behaviour

- The layout scrolls only when it cannot fit, so at large text sizes the buttons never squeeze the message.
- No motion and no haptic of its own; each action fires what its store event confirms.

Android has no shared version: #2636 builds one or records why not. The empty Library moves onto it in #2606, and Progress, Routines and the pickers in #2656.

## Content

- Three parts and nothing else: what belongs here, what to do, and the button ([tone of voice](../../../docs/tone-of-voice.md), rule 7). One sentence, no apology. The shell writes these messages.
- The glyph is an SF Symbol at `IntradaIconSize.hero` (38 points) in `accent` at `IntradaOpacity.dimmed`.
- The message is `IntradaFont.body` in `inkSecondary`, centred.
- The first action is a `MarkerButton`; further actions are plain text buttons, `IntradaFont.bodyMedium` in `accent`.
- Spacing: `IntradaSpacing.cardCompact` (12) between glyph and message and between buttons, `section` (24) above the buttons, and 32 points round the whole, centred on the screen.
- The Routines message is reworded to match what the tab does in #2610.

## Accessibility

- The glyph and message are one element, read as the message. Each button is its own element, at least 44 points tall.
- The message wraps and scales with Dynamic Type; the glyph grows with it to 56 points.

## Do and don't

- **Do** give the empty screen its one primary action as the first button.
- **Don't** leave a header add button beside it: the screen hides its round add until it has an item (Jon, 15 October 2026; #2694).
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

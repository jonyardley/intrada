---
name: ScreenScaffold
status: stable
ios_symbol: ScreenScaffold, ScreenScaffoldIconButton
ios_file: ios/Intrada/DesignSystem/ScreenScaffold.swift
ios_snapshots: ComponentSnapshotTests/testScreenScaffold.1.png, LibrarySnapshotTests/testLibraryScreenPopulated.1.png
decisions: T24, T28, T35, T36
---

# ScreenScaffold

The frame of every top-level screen: paper behind it, the page title and its line, the header actions and the rule under them.

## When to use

- Every top-level screen and every screen pushed from one: the tabs, the piece page, session detail, the summary and the session builder.

## When not to use

- A sheet on iPhone: use `BottomSheet`.
- The practice player, which has its own full-screen layout.

## Variants

- **Round add button** (`trailing`): use for the page's own add. A 30 point `accent` disc with an `onAccent` plus, in a 44 point target, beside the title.
- **Toolbar actions** (`trailingContent`): use for a screen's other actions on iPhone, as system toolbar items (T28 in [the design principles](../../../docs/design-principles.md)).
- **Header content** (`trailingPlacement: .header`): use for a view beside the title, such as the profile badge.
- **Leading action** (`leadingContent`): use only to replace the back chevron when leaving must catch unsaved work, such as the builder's Cancel.

`reservingSubtitleLine` keeps the rule level across an iPad split when only one column has a subtitle.

## Behaviour

- On iPhone the native navigation bar stays, with a blank inline title, so a pushed screen keeps the system back chevron and its edge swipe. A tab's first screen can hide the empty bar (`navigationBarHiddenAtRoot`).
- While the Library is empty its round add button is hidden: the empty state's own button is the one action, and the disc returns with the first item (Jon, 15 October 2026; the Library follows in #2694).
- No motion and no haptic.

The look is intrada's: the highlighter swipe, the round add disc and the rule.

- The iPhone snapshot tests cannot see toolbar items, so their pictures leave them out (#1880).

## Content

- The ground is `paperTop`, edge to edge. The header sits `IntradaSpacing.card` (16) in from each side and `controlGap` (8) below the top safe area.
- The title is `IntradaFont.pageTitle` in `ink`. On iPhone the highlighter swipes behind it when it fits on one line (`markerSwipe`).
- The subtitle is `IntradaFont.secondary` in `inkSecondary`, 3 points under the title.
- The rule is a 1 point line in `divider`, `IntradaSpacing.cardCompact` (12) under the header, full width. The content fills the rest and owns its own scrolling.
- The page title names the place in one or two words, sentence case, with no possessive ("Library", not "Your library"; [tone of voice](../../../docs/tone-of-voice.md), rule 3). The piece page uses the item's title.
- The subtitle is a fact about the page, joined with " · ". The core writes it when it is data (`libraryCountLine`).
- The empty Library has no subtitle, and the line keeps its space so the rule stays level (Jon, 15 October 2026; the Library follows in #2694).
- No states of its own.

## Accessibility

- The title and subtitle are one element, read as "Library, 8 pieces, 4 exercises": the separator becomes a comma. Header actions stay separate elements.
- The title is read as a heading; on iPhone that is #2657.
- The round add button reads as its label, such as "Add item".
- Text scales up to the largest standard size, where the app caps it (#1411). A title too long for one line wraps rather than truncating, and on iPhone it then loses the swipe.

## Do and don't

- **Do** build every top-level screen from it; it owns the ground, the safe areas and the header.
- **Don't** draw a page title by hand, or a second rule under it.

## Usage

```swift
ScreenScaffold(
  title: "Library", subtitle: store.viewModel?.libraryCountLine,
  trailing: .init(label: "Add item", identifier: "library.add", action: { add() })
) {
  content
}
```

## Related

`PlaceholderContent` for an empty screen. `BottomSheet` for a short task over the screen. `HairlineDivider` for rules inside the content.

---
name: ScreenScaffold
status: stable
ios_symbol: ScreenScaffold, ScreenScaffoldIconButton
ios_file: ios/Intrada/DesignSystem/ScreenScaffold.swift
ios_snapshots: ComponentSnapshotTests/testScreenScaffold.1.png, LibrarySnapshotTests/testLibraryScreenPopulated.1.png
android_symbol: ScreenScaffold
android_file: android/app/src/main/kotlin/com/intrada/android/ui/ScreenScaffold.kt
android_snapshots: screen-scaffold.png, library.png
decisions: T24, T28, T35, T36
---

# ScreenScaffold

The frame of every top-level screen: paper behind it, the page title and its line, the header actions and the rule under them.

## When to use

- Every top-level screen and every screen pushed from one: the tabs, the piece page, session detail, the summary and the session builder.
- On Android, also the forms that iPhone shows as sheets, until Android has a bottom sheet; #2636 builds one or records why not (see `BottomSheet`).

## When not to use

- A sheet on iPhone: use `BottomSheet`, which puts its title in the content for large text.
- The practice player, which has its own full-screen layout.

## Variants

- **Round add button** (`trailing`): use for the page's own add. A 30 point `accent` disc with an `onAccent` plus, in a 44 point target, beside the title.
- **Toolbar actions** (`trailingContent`): use for a screen's other actions on iPhone, as system toolbar items (T28 in [the design principles](../../../docs/design-principles.md)).
- **Header content** (`trailingPlacement: .header`): use for a view beside the title, such as the profile badge.
- **Leading action** (`leadingContent`): use only to replace the back chevron when leaving must catch unsaved work, such as the builder's Cancel.

`reservingSubtitleLine` keeps the rule level across an iPad split when only one column has a subtitle. Android has one slot, `actions`, a row at the trailing edge of the title row.

## Behaviour

- On iPhone the native navigation bar stays, with a blank inline title, so a pushed screen keeps the system back chevron and its edge swipe. A tab's first screen can hide the empty bar (`navigationBarHiddenAtRoot`).
- On Android it pads for the system bars and cutouts (`safeDrawing`).
- While the Library is empty its round add button is hidden: the empty state's own button is the one action, and the disc returns with the first item (Jon, 15 October 2026; the Library follows in #2694).
- No motion and no haptic.

The look both apps keep is intrada's: the highlighter swipe, the round add disc and the rule. The structure follows each platform (T36 in [the design principles](../../../docs/design-principles.md)).

- Android's back arrow on pushed screens, close cross on forms and headers without Cancel are #2619.
- The swipe on Android is #2605; the round add disc and how quick settings open on Android are #2606.
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
- Every text size scales, up to the largest accessibility size. A title too long for one line wraps rather than truncating, and on iPhone it then loses the swipe.

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

```kotlin
ScreenScaffold(
    item.title,
    modifier,
    actions = { TextAction("Edit", "piece.edit", actions.navigation.onEdit) },
) {
    content()
}
```

## Related

`PlaceholderContent` for an empty screen. `BottomSheet` for a short task over the screen. `HairlineDivider` for rules inside the content.

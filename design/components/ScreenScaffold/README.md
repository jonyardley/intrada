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
- On Android, also the forms and sheets that iPhone shows as sheets, such as keys, variations and the metronome, until Android has a bottom sheet (see `BottomSheet`).

## When not to use

- A sheet on iPhone: use `BottomSheet`, which puts its title in the content for large text.
- The practice player, which has its own full-screen layout.

## Anatomy

1. **Ground:** `paperTop` (#F7F4EF), edge to edge.
2. **Title:** `IntradaFont.pageTitle`, 30 point semibold, in `ink` (#2A2725). On iPhone the musician's highlighter swipes behind it when it fits on one line (`markerSwipe`).
3. **Subtitle (optional):** `IntradaFont.secondary`, 15 point, in `inkSecondary` (#6E6A66), 3 points under the title.
4. **Header actions (optional):** at the trailing edge of the title row.
5. **Rule:** a 1 point line in `divider` (#DDD7CA), `IntradaSpacing.cardCompact` (12) under the header, full width.
6. **Content:** fills the rest and owns its own scrolling.

The header sits `IntradaSpacing.card` (16) in from each side and `controlGap` (8) below the top safe area.

## Variants

iPhone has four ways to fill the header, each with a caller:

- **Round add button** (`trailing`): a 30 point `accent` disc with a `onAccent` plus, in a 44 point target, beside the title. Use for the page's own add; the Library is its one caller (T28).
- **Toolbar actions** (`trailingContent`): system toolbar items in the navigation bar, used by Profile (Edit), the piece page (the star and Edit) and the builder (T28).
- **Header content** (`trailingPlacement: .header`): a view beside the title, used by Practice for the profile badge.
- **Leading action** (`leadingContent`): replaces the back chevron, used by the builder's Cancel, which has to catch an unsaved plan.

`reservingSubtitleLine` keeps the rule level across an iPad split when one column has a subtitle; Practice is its one caller.

Android has one slot, `actions`, a row at the trailing edge of the title row. 19 of its 24 callers pass it: text actions (`TextAction`) such as Edit and Done, the Library's "+", and the Practice profile button.

## States

None of its own.

## Behaviour

- On iPhone the native navigation bar stays, with a blank inline title, so a pushed screen keeps the system back chevron and its edge swipe. The first screen of the Library and Practice tabs hides the empty bar (`navigationBarHiddenAtRoot`).
- On iPhone the scaffold allows every text size from extra small to the largest accessibility size.
- On Android it pads for the system bars and cutouts (`safeDrawing`).
- No motion and no haptic.

iPhone uses it at 9 call sites in 9 files: Library, Practice, Progress, Routines, Profile, the builder, the piece page, session detail and the summary. Android uses it at 24 call sites in 24 files: those screens, the forms and sheets that iPhone shows as sheets, the empty Routines tab, and `MissingItem` in the scaffold's own file.

**On 14 October 2026 the two apps differ:**

| | iPhone | Android |
|---|---|---|
| Highlighter swipe under the title | yes | none |
| Back on a pushed screen | the system chevron | none; system back only |
| Library add | the round ink disc | a "+" in title type |
| Header actions | system toolbar items (T28) | text actions in the title row |
| Title read as a heading | no | yes |
| Sheets | `BottomSheet` | full screens on this scaffold |

The look both apps keep is intrada's: the swipe, the round add disc and the rule. The structure follows each platform (T36): Android gains a back arrow on pushed screens and a close cross on modal forms, and drops Cancel from form headers, in #2619. The swipe is A5 in #2605 and the add disc A19 in #2606 ([the October audit](../../../docs/audit-2026-10-design.md)). How quick settings open on Android waits on the A25 ruling in #2606. Reading the title as a heading is the one to keep; adding that on iPhone is not in an issue yet.

The iPhone snapshot tests cannot see toolbar items, so their pictures leave out the piece page's Edit and star (#1880).

## Content

The page title names the place in one or two words, sentence case, with no possessive ("Library", not "Your library"; [tone of voice](../../../docs/tone-of-voice.md), rule 3). The piece page uses the item's title. The subtitle is a fact about the page, joined with " · ". The core writes the Library's (`libraryCountLine`); the builder's "Editing" is a shell string.

## Accessibility

- The title and subtitle are one element, read as "Library, 8 pieces, 4 exercises": the separator becomes a comma. Header actions stay separate elements.
- The round add button reads as its label, such as "Add item".
- Every text size scales. A title too long for one line wraps rather than truncating, and on iPhone it then loses the swipe.

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

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records the header differences #2605, #2606 and #2619 close. |

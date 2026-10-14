---
name: BottomSheet
status: stable
ios_symbol: BottomSheet
ios_file: ios/Intrada/Views/Components/BottomSheet.swift
ios_snapshots: ComponentSnapshotTests/testBottomSheet.1.png, ComponentSnapshotTests/testBottomSheetAtAccessibilitySize.1.png
not_on_android: Android opens these tasks as full-screen pages and centred dialogs; T36 says quick choices rise as bottom sheets, built in #2636
decisions: T36
---

# BottomSheet

A short task that rises over the current screen, with its title, a way out and one confirming action in a top bar.

## When to use

- A choice or a small edit that belongs to the screen underneath: keys, variations, a section, a trouble spot, the metronome, adding to a session.
- Anything the musician finishes and returns from, without leaving the screen they were on.

## When not to use

- A decision with two outcomes, such as deleting: use a system alert.
- A whole screen of its own, such as the player: use a full-screen cover.
- A message that needs no input: use `GlobalBanner`.

## Anatomy

1. **Background:** the paper (`PaperBackground`), so cards inside it lift off it as they do on a screen.
2. **Top bar:** the title inline in the centre, the optional leading action (usually Cancel) on the left, the confirming action on the right.
3. **Confirming action:** "Done" by default, or a verb such as "Save" or "Add spot". It carries the identifier `sheet.done`.
4. **Content:** whatever the caller passes, usually cards.
5. **Heights:** half and full height by default (`.medium`, `.large`), or full height only.

## Variants

Fourteen sheets use it on iPhone. The parameters they pass:

- **Leading action:** eight pass one. Seven are Cancel (section, variations, keys, link sections, trouble spot, the linked item picker, the chord chart suggestions); the tag filter's is Clear. The other six have the confirming action alone.
- **Confirmation label:** three change "Done": the section sheet ("Save"), trouble spot ("Add spot") and the chord chart suggestions ("Add", or "Add 3" with a count).
- **Confirmation disabled:** the chord chart suggestions only, while nothing is chosen.
- **Stays open on confirm** (`dismissesOnDone: false`): six sheets close themselves after saving, so a refused save keeps them open (section, variations, keys, link sections, trouble spot, the linked item picker).
- **Full height only** (`detents: [.large]`): eight sheets (section, variations, keys, link sections, add a related exercise, add to session, the instrument icon picker, the metronome). The other six open at half height and drag up.

## States

- Default.
- Confirm disabled: the confirming action greys out and does nothing.
- At accessibility text sizes the inline title would truncate, so it moves into the top of the content as a heading in `IntradaFont.cardTitle`, 17 point semibold at the default size and scaling with Dynamic Type (#2125). The second snapshot shows this.

## Behaviour

- Callers present it with `.sheet`, so it rises, drags down to dismiss and takes its corners from the system.
- Tapping the confirming action runs `onDone`, then closes the sheet unless the caller keeps it open.
- No haptic of its own; the action inside fires any haptic the store confirms.

Five iPhone sheets build their own top bar instead (the add and edit forms, the draft exercise sheet, the chord chart editor, profile edit and feedback), so they miss the large text title fix. That is I12 in [the October audit](../../../docs/audit-2026-10-design.md), fixed in #2608. The audit also found several sheets mostly empty at full height (its refinement note 7), which waits for the refinement pass (#2638).

**On 14 October 2026 Android has no bottom sheet.** It uses other shapes:

| Task | iPhone | Android |
|---|---|---|
| Keys, variations | `BottomSheet`, full height | `FullScreenDialog`, a full-screen page |
| The linked item picker, on the piece page and the add form | `BottomSheet`, half height first | a screen pushed from the piece page; `FullScreenDialog` on the add form |
| Add a related exercise, in the builder | `BottomSheet`, full height | a screen pushed in the builder |
| Switch variation, trouble spot | `BottomSheet`, half height first | `PlayerSheet`, a full-screen page with a page title and a close cross |
| Metronome | `BottomSheet`, full height | `ClickSheet`, a full-screen dialog |
| Add to session, entry settings | `BottomSheet` | a screen pushed in the builder |

Player options and renaming a variation open as centred dialogs on Android. None of these rise from the bottom or follow predictive back. T36 rules that on Android quick choices rise as bottom sheets, drawn in intrada's look. The audit's behaviour review proposes that look: paper, 3dp top corners, an ink drag handle, built on Compose foundation with no Material import, and Keys and Variations kept full screen. #2636 builds the component; A25 in #2606 moves the quick choices onto it.

## Content

The title names the task in sentence case, two or three words: "Keys", "Trouble spot", "Add to session". The confirming verb says what happens: "Done" when changes are already applied, "Save" or "Add spot" when the tap commits them.

## Accessibility

- The title is a heading, inline or in the content.
- Cancel and the confirming action are system toolbar buttons, with the system's tap areas.
- The snapshots host the sheet as a whole screen, and the renderer does not draw toolbar buttons, so neither picture shows the sheet's height, Cancel or the confirming action. The other sheet snapshots (`testTagFilterSheet` and the rest) have the same gap; check those parts on a simulator.

## Do and don't

- **Do** build every sheet on `BottomSheet`, so the top bar and the large text fix come with it.
- **Don't** put a second filled button in the content; the confirming action in the bar is the one.

## Usage

```swift
BottomSheet(
  title: "Trouble spot", confirmationLabel: "Add spot", dismissesOnDone: false,
  onDone: add,
  leadingAction: { Button("Cancel") { dismiss() } }
) {
  content
}
```

```kotlin
// No bottom sheet yet; #2636 builds it.
```

## Related

`GlobalBanner`, `FormErrorBanner` inside a sheet, `TickRow` for a list of choices.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records the five hand-built sheets and that Android has no bottom sheet. |

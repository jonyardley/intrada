---
name: BottomSheet
status: stable
ios_symbol: BottomSheet
ios_file: ios/Intrada/Views/Components/BottomSheet.swift
ios_snapshots: ComponentSnapshotTests/testBottomSheet.1.png
decisions: T36
---

# BottomSheet

A short task that rises over the current screen, with its title, a way out and one confirming action in a top bar.

## When to use

- A choice or a small edit that belongs to the screen underneath: keys, variations, the metronome, adding to a session.
- Anything the musician finishes and returns from, without leaving the screen they were on.

## When not to use

- A decision with two outcomes, such as deleting: use a system alert.
- A whole screen of its own, such as the player: use a full-screen cover.
- A message that needs no input: use `GlobalBanner`.

## Variants

- **Leading action:** use for a way out other than dragging down, usually Cancel, or Clear on a filter.
- **Confirmation label:** use a verb such as "Save" or "Add" when the tap commits the change; keep "Done" when changes are already applied.
- **Confirmation disabled:** use while there is nothing to confirm, such as no suggestion chosen.
- **Stays open on confirm** (`dismissesOnDone: false`): use when the sheet closes itself after a save, so a refused save keeps it open.
- **Full height only** (`detents: [.large]`): use for a long list or a form; otherwise the sheet opens at half height and drags up.

## Behaviour

- Callers present it with `.sheet`, so it rises, drags down to dismiss and takes its corners from the system.
- Tapping the confirming action runs `onDone`, then closes the sheet unless the caller keeps it open.
- No haptic of its own; the action inside fires any haptic the store confirms.
- iPhone sheets that build their own top bar move onto `BottomSheet` in #2608. Sheets that sit mostly empty at full height are #2683.

## Content

- The sheet's background is the paper (`PaperBackground`), so cards inside it lift off it as they do on a screen.
- The top bar holds the title inline in the centre, the leading action on the left and the confirming action on the right. The confirming action carries the identifier `sheet.done`.
- The title names the task in sentence case, two or three words: "Keys", "Filter by tag", "Add to session".
- A disabled confirming action greys out and does nothing.

## Accessibility

- The title is a heading.
- Cancel and the confirming action are system toolbar buttons, with the system's tap areas.
- The snapshots do not draw toolbar buttons or the sheet's height; check Cancel, the confirming action and the height on a simulator.

## Do and don't

- **Do** build every sheet on `BottomSheet`, so the top bar comes with it.
- **Don't** put a second filled button in the content; the confirming action in the bar is the one.

## Usage

```swift
BottomSheet(
  title: "Keys", detents: [.large], dismissesOnDone: false, onDone: save,
  leadingAction: { Button("Cancel") { dismiss() } }
) {
  content
}
```

## Related

`GlobalBanner`, `FormErrorBanner` inside a sheet, `TickRow` for a list of choices.

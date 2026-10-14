---
name: LibraryItemCard
status: stable
ios_symbol: LibraryItemCard, SelectableLibraryRow
ios_file: ios/Intrada/Views/Components/LibraryItemCard.swift, ios/Intrada/Views/Components/SelectableLibraryRow.swift
ios_snapshots: ComponentSnapshotTests/testLibraryItemCard.1.png
android_symbol: LibraryItemCard
android_file: android/app/src/main/kotlin/com/intrada/android/ui/LibraryScreen.kt
android_snapshots: library.png
decisions: T24, T35
---

# LibraryItemCard

One piece or exercise in a list, with what it is and how it is going at a glance.

## When to use

- A row in the Library.
- A row in a picker that adds library items to something, such as Add to session: use `SelectableLibraryRow`, which draws this card with a tick.

## When not to use

- A related exercise on a piece's page: that row is drawn by hand today (#2678).
- A row inside a grouped card, such as a session's items: those are rows with `HairlineDivider` between them.

## Variants

- **Library row** (`showsMastery` and `showsMissingDetailsPrompt` on): use in the Library. It adds the mark ring and, when an item has no composer, key or tempo, a prompt to add them.
- **Picker row** (`SelectableLibraryRow`): use where a tap adds or removes the item. Once added it gains a 2 point ink edge and a filled tick; before, a plus.

## Content

- A 4 point bar on the left in the item type's colour, the only type signal; rows carry no type badge.
- The title, then the composer, then key and tempo joined by a middle dot. The core writes the subtitle, key and tempo lines.
- Then, when there are any: a star for a priority, a count chip for linked exercises, and the tags.
- A count chip for keys, and one for variations, shows only when there is more than one.
- With no composer, key or tempo, the iPhone shell writes "Add composer, key and tempo" for a piece, or "Add key and tempo" for an exercise.
- The mark ring on the right shows the latest mark, or the rest when it has none.
- Android shows only the title and composer today. Key, tempo, linked exercises and the ring are A20 in #2606; the rest is #2679.

## Accessibility

- One element, read as the type, the title, then what the row shows: priority, linked exercises, keys, variations, composer, key and tempo.
- The iPhone label leaves out the mark and the tags; #2677 checks whether VoiceOver reads them.
- Android reads only the type, title and composer, on purpose until the card draws the rest (the comment above `spokenLabel`). TalkBack does not yet call it a button; A30 in #2606.
- Text wraps and scales with Dynamic Type and font scale. The card grows with it.

## Do and don't

- **Do** let the type bar carry the type.
- **Don't** add a type badge, or a second accent colour, to a row.

## Usage

```swift
NavigationLink(value: item.id) {
  LibraryItemCard(item: item, showsMastery: true, showsMissingDetailsPrompt: true)
}
.buttonStyle(.plain)
```

```kotlin
LibraryItemCard(
    item,
    Modifier.clickable(onClickLabel = if (item.itemType == ItemKind.PIECE) "open" else "edit") {
        onOpen(item)
    },
)
```

## Related

`ScoreRing`, the mark. `TagChip`, the tags. `Card`, the surface it draws on Android. `TypeBadge`, which rows do not use.

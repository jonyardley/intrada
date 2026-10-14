---
name: SegmentedPills
status: stable
ios_symbol: SegmentedPills
ios_file: ios/Intrada/Views/Components/SegmentedPills.swift
ios_snapshots: ComponentSnapshotTests/testSegmentedPills.1.png
android_symbol: SegmentedPills
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PageParts.kt
android_snapshots: segmented-pills.png
decisions: T24, T36
---

# SegmentedPills

Choosing one of two to five options in place, with the choice showing at once.

## When to use

- A mode on the screen you are on: a piece or an exercise on the add form.
- A small fixed set of values, such as a time signature grouping or the practice defaults.

## When not to use

- More than five options, or options that come from the musician's own data: use a list with `TickRow` in a sheet.
- An on or off setting: use a toggle.
- An action: use a button. A pill changes what you see; it never saves or sends.

## Anatomy

1. **Track:** the shape holding every option.
2. **Pill:** one option's label.
3. **Chosen pill:** the selected option, filled.

## Variants

- **Full width track** (`.fullWidthTrack`): use for a form choice. Equal segments on a bordered card track; at accessibility sizes the segments stack.
- **Wrapping track** (`.wrappingTrack`): use when a label can be long, such as a grouping like 2 + 2 + 3. Whole pills move to a new row rather than a label breaking (#2490).

Every caller names its track; there is no default. Android has one track with a `wraps` flag that covers both.

## States

Chosen, not chosen. There is no disabled state: if an option cannot be chosen, leave it out.

## Behaviour

- Tapping a pill chooses it at once. On iPhone the fill slides to it with `IntradaMotion.snappy`, and does not move under Reduce Motion.
- No haptic.

**On 13 October 2026 the apps draw it differently:**

| | iPhone | Android |
|---|---|---|
| Track | white card, hairline edge, round ends | sunken fill, 3dp corner, no edge |
| Chosen pill | ink fill, white text | white fill, ink text |
| Not chosen | secondary ink text | secondary ink text |

The iPhone look is the one this page keeps: ink on white is the interactive colour (T24), and the chosen option has to read as chosen at a glance. Android moves to it in the fix for one look per job (#2605 to #2611). Neither app fills a pill with the highlighter; that is kept for the one call to action (`MarkerButton`, I9 in [the October audit](../../../docs/audit-2026-10-design.md)).

## Content

One or two words per label, sentence case. No icons alone; an icon pill needs a text label for VoiceOver and TalkBack.

## Accessibility

- Each pill is a button with the selected trait on iPhone and a tab with `selected` on Android, so a screen reader says which is chosen.
- On iPhone, `hint` adds a spoken hint to the label. On Android, `spoken` replaces the label for TalkBack. Use them when the label alone is unclear.
- Android pills are at least 48dp tall. iPhone pills are shorter than 44 points on every track; the touch area should grow to 44 without the drawn pill growing.

## Do and don't

- **Do** keep labels the same length where you can, so the segments look even.
- **Don't** use pills for a choice that needs explaining; a sheet with rows has room for a line each.

## Usage

```swift
SegmentedPills(
  options: [.piece, .exercise], selection: $kind, label: \.label,
  font: IntradaFont.segment, layout: .fullWidthTrack)
```

```kotlin
SegmentedPills(
    ItemKind.entries,
    selection,
    onSelect,
    label = { it.label },
    tag = { "itemForm.kind.${it.name.lowercase()}" },
)
```

## Related

`TickRow` for longer choices, `ScoreSelector` for a score.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-13 | #2620 | First page. Records that the two apps draw it differently. |
| 2026-10-14 | #2635 | Deleted the inline scrolling track, which no screen used, in commit d93d7a4e; recover it from there. |

---
name: MasteryDeltaToast
status: stable
ios_symbol: MasteryDeltaToast
ios_file: ios/Intrada/Views/Components/MasteryDelta.swift
ios_snapshots: ComponentSnapshotTests/testMasteryDeltaToast.1.png
decisions: T24
---

# MasteryDeltaToast

The celebration banner: the one item whose mark rose most, shown as the mark it was and the mark it is now.

## When to use

- At the top of the session summary, when a mark rose in this session.
- At the top of Progress, when a mark rose in the period shown.

## When not to use

- A list of every change: use `MasteryDelta`, the white was and now row on Progress, which also shows a fall.
- An error or notice: use `GlobalBanner`.
- A mark that fell or stayed the same. The core only names a rise, and a first mark is not a rise.

## Variants

- **Title only:** use on the summary, "Clair de Lune moved up".
- **Title and subtitle:** use on Progress, "Mastery up" over the item's title.

## Content

- A dark card in `celebrationBg` (ink) with the 3 point corner.
- On the left, a sparkle on a disc in the musician's highlighter, then the title.
- On the right, the earlier mark, an arrow in the highlighter and the new mark, at title size.
- The core picks the item (`top_mover`: the largest rise). The shell writes the titles.
- The earlier mark is faded rather than a step of colour; #2607 (I30).

## Behaviour

It fades in and drops 10 points into place, half a second after the headline. It stays still when Reduce Motion is on.

## Accessibility

- One element, read as the title, then "mastery up from 3 to 4".
- The label leaves out the subtitle, so on Progress it never names the item: "Mastery up, mastery up from 5 to 7"; #2680.
- Scales with Dynamic Type. At the largest sizes on iPhone, "Mastery up" breaks inside a word; #2413.

## Do and don't

- **Do** show at most one banner on a screen, for the biggest rise.
- **Don't** use it for a fall or a first mark.

## Usage

```swift
MasteryDeltaToast(
  title: "\(toast.itemTitle) moved up",
  subtitle: nil,
  was: Int(toast.previousScore ?? 0),
  now: Int(toast.currentScore)
)
```

## Related

`MasteryDelta`, the list row. `ScoreRing`, the marks themselves. `ScoreSelector`, where a mark is given.

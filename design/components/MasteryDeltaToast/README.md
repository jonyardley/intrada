---
name: MasteryDeltaToast
status: stable
ios_symbol: MasteryDeltaToast
ios_file: ios/Intrada/Views/Components/MasteryDelta.swift
ios_snapshots: ComponentSnapshotTests/testMasteryDeltaToast.1.png
android_symbol: MasteryDeltaToast
android_file: android/app/src/main/kotlin/com/intrada/android/ui/ProgressParts.kt
android_snapshots: mastery-delta-toast.png
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
- The core picks the item (`top_mover`: the largest rise). The shells write the titles.
- On Android the highlighter is always butter today; #2605 (A6). Android's summary draws its own line instead of this banner; #2672.
- The earlier mark is faded rather than a step of colour; #2607 (I30).

## Behaviour

On iPhone it fades in and drops 10 points into place, half a second after the headline. It stays still when Reduce Motion is on. Android shows it without the reveal; #2673.

## Accessibility

- One element, read as the title, then "mastery up from 3 to 4".
- The label leaves out the subtitle, so on Progress it never names the item: "Mastery up, mastery up from 5 to 7"; #2680.
- Scales with Dynamic Type and font scale. At the largest sizes on iPhone, "Mastery up" breaks inside a word; #2413.

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

```kotlin
MasteryDeltaToast(
    "Mastery up",
    mover.itemTitle,
    was = mover.previousScore?.toInt() ?: 0,
    now = mover.currentScore.toInt(),
)
```

## Related

`MasteryDelta`, the list row. `ScoreRing`, the marks themselves. `ScoreSelector`, where a mark is given.

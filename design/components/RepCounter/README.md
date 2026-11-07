---
name: RepCounter
status: stable
ios_symbol: RepCounter
ios_file: ios/Intrada/Views/Components/RepCounter.swift
ios_snapshots: FocusPlayerSnapshotTests/testRepCounter.1.png, FocusPlayerSnapshotTests/testRepCounterAccessibilitySize.1.png
android_symbol: RepCounter, RepState, RepActions
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PlayerParts.kt
android_snapshots: rep-counter.png
decisions: T19, T35
---

# RepCounter

Counting repetitions of an item in the player: how many went right out of the target, with one tap for each try.

## When to use

- The player, when the current item has a repetition target.

## When not to use

- Marking how an item went overall: use `ScoreSelector`.
- Any other count or stepper, such as a tempo.

## Variants

None. At accessibility text sizes the iPhone layout stacks: the count moves under the label, and the buttons stack with "Got it" first and "Not quite right" in full.

## Behaviour

- **Got it** counts one repetition, and keeps counting past the target.
- **Not quite** records a miss and takes one off. It stays tappable at zero and at the target, because the core records the miss there (#1507).
- **Undo** takes back the last tap without recording a miss (#2107). It shows only when the core says there is something to undo.
- The counter is always there but never in the way: playing on without tapping is fine (T19).
- On iPhone a filled dot pops as it fills, and the buttons rebound when pressed.
- Neither app plays a haptic on these buttons yet, #1508.

## Content

- The label "Repetitions" through `FieldLabel`, then the count in `ink` and the rest in `inkSecondary`. On iPhone both are `secondary` with figures of equal width, the count semibold.
- After the count: "of" and the target. Once started and short of it, "to go" and how many, joined by a middle dot. Past the target, how many extra.
- A dot for every repetition in the target, drawn from the start so the row shows what is left (#1735). Done dots fill `success`; the rest are `slotOutline` rings.
- "Got it" with a tick in `repCleanFg` on `repCleanBg`. "Not quite" with a cross in `repMissedFg` on `repMissedBg`. Both have the `IntradaRadius.control` corner. On iPhone "Got it" is edged `repCleanBorder` and "Not quite" `slotOutline`.
- Undo is `segment` in `accent`, at the right of the header.
- Android has no dots and no tick or cross yet, A14 in #2606. Its other differences are G5 in #2762: Undo under the buttons rather than in the header, both buttons edged in `hairline`, the count in the figure font, no spoken hints and no stacked layout at large text sizes.
- The core gives the count, the target and the state; the shell works out how many are to go and writes the words around them.

## Accessibility

- The count is one element: "Repetitions", with the value read as "3 of 10, 7 to go". The label and dots are hidden.
- On iPhone "Got it" has the hint "Counts one repetition", "Not quite" has "Takes one repetition off" and Undo has "Takes back the last repetition".
- Undo keeps a 44 point target on iPhone; the buttons are at least 48 dp tall on Android.
- The dots grow with Dynamic Type from 11 points, capped at 18 so ten still fit the row (#1950).

## Usage

```swift
RepCounter(
  count: Int(active.currentRepCount ?? 0),
  slots: Int(active.currentRepSlots),
  touched: active.currentRepCount != nil,
  reached: active.currentRepTargetReached ?? false,
  extra: Int(active.currentRepsPastTarget), canUndo: active.currentCanUndo,
  onGotIt: { store.send(.session(.repGotIt(now: now, reading: tempoReading))) },
  onNotQuite: { store.send(.session(.repMissed(now: now, reading: tempoReading))) },
  onUndo: { store.send(.session(.repUndo(now: now, reading: tempoReading))) })
```

```kotlin
RepCounter(
    RepState(count, slots, touched, reached, extra, canUndo),
    RepActions(onGotIt, onNotQuite, onUndo),
)
```

## Related

`FieldLabel`, the label. `ScoreSelector`, the mark at the end of an item.

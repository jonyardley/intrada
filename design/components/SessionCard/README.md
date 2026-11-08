---
name: SessionCard
status: stable
ios_symbol: SessionCard
ios_file: ios/Intrada/Views/Components/SessionCard.swift
ios_snapshots: PracticeSnapshotTests/testSessionCardsPlayedSummary.1.png
android_symbol: SessionCard
android_file: android/app/src/main/kotlin/com/intrada/android/ui/PracticeParts.kt
android_snapshots: session-card.png
decisions: T24, T35
---

# SessionCard

One past practice session as a row: when it was, how long, how many items and what was played.

## When to use

- A session in the list under the week strip on the Practice home, one card per session on the chosen day.

## When not to use

- A piece's or exercise's own history on its page: use `RecentSessions`, which shows the mark for each session.
- A day with no practice: the Practice home shows an empty day card instead.
- The session that comes next: use `UpNextHero`.

## Variants

None. The lines below appear only when the core gives them content.

## Behaviour

- A tap opens the session. A swipe left offers Delete session, through the shared swipe to delete.
- On iPhone the screen adds the tap and the swipe around the card; on Android the card takes `onOpen` and `onDelete` itself. A musician sees the same thing.
- No motion of its own.

## Content

- The card is `cardFill` with a one point `hairline` edge and the `IntradaRadius.card` corner, padded `IntradaSpacing.card` (16) on every side. Sessions mix item types, so there is no type bar on the left.
- The day line in `cardTitle`, `ink`.
- The total time and the item count, joined by a middle dot, in `secondary`, `inkSecondary`.
- When the session has one, what was played in `body`, `ink`. It wraps rather than clipping: the core keeps it short, but a long one still shows in full.
- A session that ended early adds "Ended early" in `small`, `inkSecondary`.
- The core writes every line except "Ended early"; the shell joins the time and count.
- Both apps draw the card by hand rather than through `cardSurface()`: A27 in #2606 for Android; iPhone moves with the card refinement pass, #2638.

## Accessibility

- One element, read as the day, the time, the item count, then what was played and "ended early" when present.
- It reads as a button with the hint, or click label, "Opens the session". Delete session is a custom action for VoiceOver and TalkBack.
- Text wraps and scales with Dynamic Type and font scale; the card grows with it.

## Usage

```swift
SessionCard(session: session)
  .contentShape(Rectangle())
  .onTapGesture { open(session) }
  .accessibilityAddTraits(.isButton)
  .accessibilityAction { open(session) }
  .accessibilityHint("Opens the session")
  .swipeToDelete(label: "Delete session") { delete(session) }
```

```kotlin
SwipeToDelete("Delete session", delete) {
    SessionCard(session, onDelete = delete) { onOpen(session.id) }
}
```

## Related

`WeekStrip`, which picks the day. `RecentSessions`, one item's history. `Card`, the surface it should draw.

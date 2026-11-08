---
name: UpNextHero
status: stable
ios_symbol: UpNextHero
ios_file: ios/Intrada/Views/Components/UpNextHero.swift
ios_snapshots: PracticeSnapshotTests/testUpNextHeroCoral.1.png
android_symbol: UpNextCard
android_file: android/app/src/main/kotlin/com/intrada/android/ui/UpNextCard.kt
android_snapshots: practice-up-next.png, practice-up-next-filled.png
decisions: T15, T22
---

# UpNextHero

The suggested session at the top of Practice: what to play next, why, and one button to start it.

## When to use

- The plan the core suggests on the Practice tab, whether it is the next piece or a plan filled to today's length.

## When not to use

- A session the musician built: use the builder's list, not a hero.
- A past session: use `SessionCard`.
- Any other highlight on a screen: the hero is the one dark block on Practice, and a second one competes with it.

## Variants

- **Up next** (no length set): use when the core suggests one piece. The heading reads "Up next" and the totals are the lead piece's.
- **Today's plan** (a length set): use when the core fills a plan to a length. The heading reads "Today's plan", the totals are the plan's, and later pieces list under "Then".

## Behaviour

- **Start** opens the player with the plan. The button shows the plan's minutes and presses with `PressRebound`.
- **Change it first** opens the builder with the plan in it; **Build my own instead** opens an empty builder, a decision rather than a dismissal (#1617).
- The heading and totals share a line when they fit and stack when they do not; so do the two secondary actions, so no label is cut off.
- The Android version is named `UpNextCard`; settling on one name is G6 in #2762.

## Content

- The ground is the hero gradient for the musician's highlighter, from `heroGradientTop` (ink) to `heroGradientBottom`, with the `IntradaRadius.hero` corner, the `IntradaShadow.hero` lift and `IntradaSpacing.section` padding.
- The lead piece's title is `IntradaFont.title` in `paperTop`, up to three lines. The reason under it is `secondary` in `onAccent` at `IntradaOpacity.strong`, led by a star in the marker colour when the piece is a priority.
- Items sit in a washed list (`paperTop` at `IntradaOpacity.wash`, `IntradaRadius.card`). Each has a dot in its type's on-hero colour, the title, the core's reason and, when there is one, the weakest section.
- The Start button is filled with the marker, with `onMarker` text in `IntradaFont.button` and the `IntradaRadius.control` corner.
- Every reason, title and subtitle is the core's: the shell renders them and never writes its own (T15 in [the design principles](../../../docs/design-principles.md)).
- A plan with no blocks draws nothing; the core never sends one.

## Accessibility

- The heading, totals, title, composer and reason read as one sentence, so the totals are never read apart from the piece they describe.
- Each item row is one element: title, type, reason and weakest section. The middle dot is read as a comma.
- Start reads "Start practising", with the item count and minutes as its value. The secondary actions carry hints saying where they go.
- Text wraps and scales with Dynamic Type and font scale. The title shrinks to three quarters before it truncates.

## Usage

```swift
if let plan = viewModel.upNext,
  let hero = UpNextHero(plan: plan, onStart: start, onChange: change, onBuildOwn: buildOwn)
{
  hero
}
```

```kotlin
UpNextCard(plan, colour, UpNextActions(onStart = start, onChange = change, onBuildOwn = buildOwn))
```

## Related

`MarkerButton`, the one call to action elsewhere. `SectionTitle`, the heading. `ProfileBadge`, above it on Practice.

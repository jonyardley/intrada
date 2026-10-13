---
name: MarkerButton
status: proposed
ios_symbol: MarkerButton, BrandBarButton
ios_file: ios/Intrada/DesignSystem/MarkerButton.swift, ios/Intrada/Views/Components/BrandBarButton.swift
ios_snapshots: ComponentSnapshotTests/testPrimaryButtons.1.png
android_symbol: MarkerButton
android_file: android/app/src/main/kotlin/com/intrada/android/ui/FirstRunScreen.kt
android_snapshots: first-run-welcome.png
decisions: T24
---

# MarkerButton

The one call to action on a screen, filled with the musician's chosen highlighter.

## When to use

- The action the screen exists for: start the session, save the reflection, continue in the first run.
- At most one per screen or sheet.

## When not to use

- A second or quieter action: use a plain ink text button.
- Choosing an option: use `SegmentedPills` or `TickRow`. A selected choice is never highlighter (I9 in [the October audit](../../../docs/audit-2026-10-design.md)).
- Anything destructive: use `DeleteButton` and a confirm.

## Anatomy

1. **Fill:** the musician's highlighter, from `\.marker` (iPhone) or passed in (Android). Butter by default.
2. **Label:** `IntradaFont.button` in `onMarker` ink (11.3:1 on butter).
3. **Glyph (optional):** leading or trailing, the same ink. A play glyph to start, an arrow to move on.
4. **Corner:** `IntradaRadius.card`, 3 points.
5. **Lift:** `IntradaShadow.button`.

## Variants

One. Full width, with or without a glyph.

## States

- Default.
- Pressed: shrinks slightly and springs back (`PressRebound`); no change of colour.
- Disabled: not used. Leave the button out, or explain on the screen what is missing.

## Behaviour

Tapping it sends one event to the core. Any success haptic fires only after the core confirms.

**On 13 October 2026 there are three versions:**

| | Where | Glyph | Lift | Press |
|---|---|---|---|---|
| iPhone `MarkerButton` | first run, summary, page camera, every empty state's action | no | no | rebound |
| iPhone `BrandBarButton` | builder start, reflection save, piece detail, related exercises, the player's Move on | yes | yes | system default |
| Android `MarkerButton` | first run only, private to that screen | no | no | system default |

Most Android primary buttons are black (`InkButton`, A4 in the audit).

**Proposed:** one button with an optional glyph, named `MarkerButton` on both apps. iPhone folds `BrandBarButton` into it; Android makes its version shared and replaces `InkButton` (#2605). The two iPhone versions look almost the same on screen (Jon, 13 October 2026), so merging them changes no look. Whether the button has a lift and a bounce is settled in the refinement pass (#2638), where depth is visible enough to judge.

## Content

A verb first, two or three words: "Start session", "Save session". A count or time can follow after a middle dot: "Start session · 10 min".

## Accessibility

- The label is the spoken name; a glyph adds nothing to it.
- At least 44 points (iPhone) or 48dp (Android) tall: the padding gives this on iPhone, `heightIn(min = 48.dp)` on Android.
- Ink on butter is 11.3:1 (T24). No test yet checks ink on the other seven highlighters; one should land before this page moves to stable.

## Do and don't

- **Do** put it at the bottom of the content it acts on, or pinned above the keyboard on a form.
- **Don't** put two on one screen, or a highlighter fill on anything else that can be pressed.

## Usage

```swift
MarkerButton("Save session") {
  store.send(.session(.saveSession(now: SessionClock.nowRFC3339())))
}
.accessibilityIdentifier("summary.save")
```

```kotlin
// Private to FirstRunScreen.kt until #2605 shares it.
MarkerButton("Set up profile", "firstRun.setUpProfile", IntradaColor.marker, onSetUpProfile)
```

## Related

`DeleteButton`, `CapsuleActionButton`, `AddRowButton`.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-13 | #2620 | First page, proposed. Records three versions and proposes one. |

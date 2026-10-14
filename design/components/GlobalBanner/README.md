---
name: GlobalBanner
status: stable
ios_symbol: GlobalBanner, AppBannerStack
ios_file: ios/Intrada/DesignSystem/GlobalBanner.swift, ios/Intrada/Views/Components/AppBannerStack.swift
ios_snapshots: RootShellSnapshotTests/testGlobalBanner.1.png
android_symbol: GlobalBanner, BannerTone, BannerAction
android_file: android/app/src/main/kotlin/com/intrada/android/ui/GlobalBanner.kt
android_snapshots: global-banner.png
decisions: T30
---

# GlobalBanner

A full-width line under the status bar for something the whole app needs to say: a refusal, a failed save, a note on what just happened, or an Undo.

## When to use

- The core refused an action or a write failed, and the musician is not in a form that can show it.
- Something worked but needs a word, such as the metronome note at the end of a play.
- A removal that can be undone, with Undo.
- A standing condition: the app stopped, or storage is unavailable.

## When not to use

- Why a form could not be saved: use `FormErrorBanner`, inside the form, next to the fields.
- Success that needs no explanation: say nothing, or let the screen change show it.
- A question: use a system alert.

## Anatomy

1. **Fill:** `dangerBanner` (`danger` at 12%) or `cardFill`.
2. **Glyph:** the warning triangle (danger) or the info circle (notice), in the banner's ink.
3. **Message:** `IntradaFont.bodyMedium`, 17 point medium, wrapping onto as many lines as it needs, in `danger` (#9C4A3A) or `inkSecondary`.
4. **Action (optional):** one word such as "Undo", `IntradaFont.button` in `ink`.
5. **Dismiss (optional):** a close cross in the banner's ink.
6. **Edge (notice only):** a hairline under the banner on iPhone.

## Variants

- **Danger** (the default tone): use for a refusal or a failed write. The success haptic is withheld (T30).
- **Notice** (`tone: .notice`, `BannerTone.NOTICE`): use for a true outcome that is not a failure. The tap that raised it keeps its success haptic. The accessibility label starts "Note, ".
- **With action** (`action`): the notice that offers Undo after a session is deleted. No danger banner has an action.
- **Standing** (no `onDismiss`): the "app stopped" and "storage unavailable" banners, which stay until the condition ends.

## States

Shown or not. Several stack, one under another: on iPhone in a fixed order, stopped, storage, error, notice, Undo, each dismissed on its own (`AppBannerStack`). On Android the Undo banner sits above the screen's own stopped and error banners; the Android picture shows those three in that order.

## Behaviour

- Dismissing sends one event to the core (`clearError`, `clearNotice`, `dismissUndoDelete`), which clears the message. Undo sends `undoDelete`.
- On iPhone, `AppBannerStack` is the one caller. It sits in the tab shell and over the player, so a banner shows wherever the musician is (#2036).
- No motion of its own.

**On 14 October 2026 the two apps differ:**

| | iPhone | Android |
|---|---|---|
| Where it shows | every screen and the player, through `AppBannerStack` | nine screens place their own error and stopped banners: the Library and the piece page by hand, and Practice, the builder, the player, the summary, add to session, entry settings and Profile through `AlertBanners` (`BuilderScreen.kt`). The player's sheets and the reflection read the core's error when a save is refused and show it inside the sheet as a `FormErrorBanner`. The Undo banner sits above the whole navigation host (`AppFrame.kt`), so it shows on pushed screens too, but not over the player |
| Core notices (the metronome note) | shown | never read, so never shown |
| Storage unavailable | shown | not shown; the store knows, no screen asks |
| Hairline under a notice | yes | no |
| Dismiss tap area | the glyph only | a 48dp square |

The iPhone behaviour is the one both apps keep: one stack, every message the core raises, on every screen. Android's 48dp dismiss area is the one to keep; on iPhone the action reaches 44 points but the cross does not. Showing the core's notice is A1 in [the October audit](../../../docs/audit-2026-10-design.md), fixed in #2605; the audit's evidence for A1 predates the notice tone, which Android now has for Undo. One stack for every screen and the storage banner are not in an issue yet. The iPhone cross's tap area belongs with the small tap targets in #2607.

## Content

What happened and what it means for the musician's data, in one or two sentences ([tone of voice](../../../docs/tone-of-voice.md), rule 8). Sentence case. The core writes the error and notice wording; the shells write only the stopped and storage lines.

## Accessibility

- The banner is one element, read as its message, with "Note, " before a notice. The action and Dismiss are accessibility actions on it (custom actions on Android), because combining the children hides the buttons.
- The message scales with Dynamic Type and font scale, and wraps rather than truncates.

## Do and don't

- **Do** let the core own the message: the shell shows `viewModel.error` and `viewModel.notice`, never a sentence of its own about a domain outcome.
- **Don't** put a form's validation here; the musician needs it beside the field.

## Usage

```swift
GlobalBanner(message: message) { store.send(.clearError) }
GlobalBanner(message: message, tone: .notice) { store.send(.clearNotice) }
```

```kotlin
GlobalBanner(error, tag = "banner.error", onDismiss = actions.onDismissError)
GlobalBanner(
    message,
    tag = "banner.undoDelete",
    tone = BannerTone.NOTICE,
    action = BannerAction("Undo") { store.send(Event.Session(SessionEvent.UndoDelete(SessionClock.now()))) },
    onDismiss = { store.send(Event.Session(SessionEvent.DismissUndoDelete)) },
)
```

## Related

`FormErrorBanner` for forms.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records where Android still drops messages iPhone shows. |

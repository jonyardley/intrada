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

## Variants

- **Danger** (the default tone): use for a refusal or a failed write. The success haptic is withheld (T30 in [the design principles](../../../docs/design-principles.md)).
- **Notice** (`tone: .notice`, `BannerTone.NOTICE`): use for a true outcome that is not a failure. The tap that raised it keeps its success haptic.
- **With action** (`action`): use for the notice that offers Undo after a session is deleted. No danger banner has an action.
- **Standing** (no `onDismiss`): use for "app stopped" and "storage unavailable", which stay until the condition ends.

## Behaviour

- There is one stack, on every screen and over the player, showing every message the core raises. Several banners stack one under another, each dismissed on its own.
- Dismissing sends one event to the core (`clearError`, `clearNotice`, `dismissUndoDelete`), which clears the message. Undo sends `undoDelete`.
- No motion of its own.
- Android does not yet show the core's notices, such as the metronome note; #2605.

## Content

- What happened and what it means for the musician's data, in one or two sentences ([tone of voice](../../../docs/tone-of-voice.md), rule 8). Sentence case.
- The core writes the error and notice wording; the shells write only the stopped and storage lines.
- Danger: `dangerBanner` fill (`danger` at 12%), the warning triangle, the message in `danger`.
- Notice: `cardFill`, the info circle, the message in `inkSecondary`, with a hairline under the banner.
- The message is `IntradaFont.bodyMedium` and wraps onto as many lines as it needs. The action is one word in `IntradaFont.button`, `ink`.

## Accessibility

- The banner is one element, read as its message, with "Note, " before a notice. The action and Dismiss are accessibility actions on it (custom actions on Android), because combining the children hides the buttons.
- The message scales with Dynamic Type and font scale, and wraps rather than truncates.
- The dismiss target is at least 48dp on Android; the iPhone cross reaching 44 points is #2607.

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

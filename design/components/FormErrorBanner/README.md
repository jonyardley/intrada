---
name: FormErrorBanner
status: stable
ios_symbol: FormErrorBanner
ios_file: ios/Intrada/DesignSystem/FormErrorBanner.swift
ios_snapshots: ComponentSnapshotTests/testFormErrorBanner.1.png
android_symbol: FormErrorBanner
android_file: android/app/src/main/kotlin/com/intrada/android/ui/components/FormErrorBanner.kt
android_snapshots: form-error-banner.png
decisions: T35
---

# FormErrorBanner

Says, inside a form or sheet, why what the musician entered could not be saved.

## When to use

- The core refused a save from a form or sheet.
- A photo could not be taken or saved.

## When not to use

- An error that belongs to the whole app, or that arrives when no form is open: use `GlobalBanner`.
- A field that is wrong on its own: the field shows its fault in place, in `dangerWash`, and the banner says why.

## Variants

One. There is no tone, size or action option.

## Behaviour

- No motion of its own; a caller may fade it in.
- The banner announces its message to VoiceOver and TalkBack, as a polite live region. Android's banner does this today; moving it into the iPhone banner, so no iPhone form swallows its error, is #2655.
- One iPhone sheet, the chord chart editor, draws its own error box; #2608 moves it onto this banner.

## Content

- The fill is `dangerWash` (`danger` at 10%) with a 1 point `dangerEdge` line (`danger` at 25%) on the `IntradaRadius.card` (3) corner.
- The filled warning triangle in `danger`, sized to the text, then the message in `IntradaFont.bodyMedium` in `danger`, wrapping onto as many lines as it needs.
- `IntradaSpacing.cardCompact` (12) inside, `controlGap` (8) between the triangle and the message.
- Shown while the caller holds a message, gone when it does not.
- Say what happened and what it means for the musician's data, in one sentence ([tone of voice](../../../docs/tone-of-voice.md), rule 8). The core writes the form messages; #2610 rewrites them to say what is wrong, such as "Add a composer".
- The shells write only two kinds of message: photo and scan failures, which never reach the core ("Couldn't save the photo. Try again."), and a fallback when the core refused but left no message ("Couldn't save. Try again.").
- When the app has halted, the banner shows the halted message instead (`Store.haltedMessage`, `Store.HALTED_MESSAGE`).

## Accessibility

- One element, read as "Error: " and the message.
- The message scales with Dynamic Type and font scale, and wraps rather than truncates.
- Colour is not the only signal: the triangle and the word "Error" carry it too.

## Do and don't

- **Do** show the core's message as it comes. The shell writes only the photo and scan failures and the fallbacks; never a domain rule.
- **Don't** draw a danger box by hand; that is the copy that drifts.

## Usage

```swift
if let formError { FormErrorBanner(message: formError) }
```

```kotlin
formError?.let { FormErrorBanner(it, Modifier.testTag("keysSheet.error")) }
```

## Related

`GlobalBanner` for the whole app. `FieldCard` for the fields it sits above.

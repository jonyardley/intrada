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

- The core refused a save from a form or sheet: the add and edit forms, keys, variations, sections, linking sections, the related exercise picker, the trouble spot, the reflection, profile edit and the first run.
- A photo could not be taken or saved, on the photo card and the scan entry.

## When not to use

- An error that belongs to the whole app, or that arrives when no form is open: use `GlobalBanner`.
- A field that is wrong on its own: the field shows its fault in place, in `dangerWash`, and the banner says why.

## Anatomy

1. **Fill:** `dangerWash`, `danger` (#9C4A3A) at 10%.
2. **Edge:** a 1 point line in `dangerEdge`, `danger` at 25%, on a `IntradaRadius.card` (3) corner.
3. **Glyph:** the filled warning triangle in `danger`, sized to the text.
4. **Message:** `IntradaFont.bodyMedium`, 17 point medium, in `danger`, wrapping onto as many lines as it needs.
5. **Spacing:** `IntradaSpacing.cardCompact` (12) inside, `controlGap` (8) between glyph and message.

## Variants

One. There is no tone, size or action option.

## States

Shown or not: the caller shows it while it holds a message.

## Behaviour

No motion of its own; on iPhone the photo card and the scan entry fade it in.

**On 14 October 2026 the two apps differ:**

| | iPhone | Android |
|---|---|---|
| Callers | 12 call sites in 12 files | 13 call sites in 12 files; `PlayerSheets.kt` has two, the trouble spot and the record sheets |
| Announcement | the banner announces nothing; 3 of the 12 callers post their own (the add and edit forms, the photo card, the scan entry) | the banner is a polite live region, so every caller announces it |
| Glyph | an SF Symbol in the message's font, on the top edge | a drawable the size of the text, centred on the first line |

Android's announcement is the one both apps keep: an error a VoiceOver user cannot hear is swallowed. Moving it into the iPhone banner is not in an issue yet.

One iPhone sheet draws its own error box instead: the chord chart editor (`ChordChartEditSheet`), with a filled circle, ink text and no edge. That is I13 in [the October audit](../../../docs/audit-2026-10-design.md), fixed in #2608. Android has no chord chart editor.

## Content

What was wrong and how to put it right, in one sentence ([tone of voice](../../../docs/tone-of-voice.md), rule 8). The core writes the form messages (`crates/intrada-core/src/validation.rs`), such as "Composer must be between 1 and 200 characters", the message in both pictures. That wording states a rule rather than the fix; I39 in #2610 rewrites the validation messages to say what is wrong, such as "Add a composer". The iPhone photo card falls back to a shell string when the core gives none: "Couldn't save the photo. Try again."

## Accessibility

- One element, read as "Error: " and the message.
- The message scales with Dynamic Type and font scale, and wraps rather than truncates.
- Colour is not the only signal: the triangle and the word "Error" carry it too.

## Do and don't

- **Do** show the core's message as it comes; never write a domain error in the shell.
- **Don't** draw a danger box by hand; the chord chart editor is the copy that drifted.

## Usage

```swift
if let formError { FormErrorBanner(message: formError) }
```

```kotlin
formError?.let { FormErrorBanner(it, Modifier.testTag("keysSheet.error")) }
```

## Related

`GlobalBanner` for the whole app. `FieldCard` for the fields it sits above.

## Changes

| Date | Issue | What changed |
|---|---|---|
| 2026-10-14 | #2635 | First page. Records that most iPhone callers do not announce the error. |

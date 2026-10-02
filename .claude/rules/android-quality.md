---
paths:
  - "android/**"
---

# The Android shell

The second native shell on the unchanged core (`specs/android-shell.md`, epic
#2220). Every architecture rule in `CLAUDE.md` binds it exactly as it binds
the Swift shell: where this file and an iOS rule both apply, the iOS rule's
intent wins and this file says how it reads in Kotlin and Compose.

## The shell is a dumb pipe

- **No business rules, validation or domain state in Kotlin.** Compose state
  (`remember`, `rememberSaveable`) holds UI interaction only. If a screen needs
  a rule the Swift shell works out for itself, move the rule into
  `intrada-core` (#2223) rather than port it: a rule ported is a rule that
  drifts.
- **One way across the bridge.** Screens read the `ViewModel` through `Store`;
  only `LiveBridge` touches bincode or the UniFFI handle. Effects run off the
  main thread and post their results back on `Main`.
- **Persistence is raw SQL on `androidx.sqlite`, not Room**: Room wants entity
  classes, and the shell must not model the domain. Singletons and the
  crash-recovery blob go to `SharedPreferences` as the core's bincode bytes,
  under the versioned key the core supplies, never a key spelled in Kotlin
  (#1345, #2026).

## Kotlin

- **`!!` is banned like `try!` and force-unwraps in Swift**, and so are
  unchecked `as` casts and `lateinit` on anything the core supplies. Use `?.`
  and `?:`, and surface the failure.
- **Never edit `android/generated/`.** It is git-ignored and rebuilt by
  `just android-gen`; fix the Rust type and regenerate.
- **Format with `just android-fmt`.** CI runs `ktfmtCheck` and fails
  unformatted Kotlin.
- **Prefer the recipes to a bare `gradlew`**: `just android-run`,
  `just android-test`, `just android-fmt`. They source `android/env.sh`, which
  finds the SDK and JDK and refuses when the bindings are missing.

## Tokens and look

- **Every colour, font, spacing and radius comes from `ui/Theme.kt`**
  (`IntradaColor`, `IntradaFont`, `IntradaSpacing`, `IntradaRadius`). A token
  Android lacks is copied from `ios/Intrada/DesignSystem/Theme.swift` with the
  same name and value, never invented. Genuine one-offs stay literal, as on
  iOS.
- **Compose foundation with intrada's own look, not Material.** No
  `MaterialTheme` colours or Material components standing in for an intrada
  primitive.
- `docs/design-principles.md` and `docs/tone-of-voice.md` bind every Android
  surface and string. The same screen on both shells says the same words.

## The per-screen bar

Built with the screen, never retrofitted:

- **A Roborazzi snapshot through `LiveBridge`**, and for a new screen or sheet
  a second one at the largest font scale.
- **TalkBack labels** on every control: a `contentDescription` or merged
  semantics that read as the iOS VoiceOver label does. A control a UI test
  drives carries a `testTag` named `screen.control`, matching the iOS
  `accessibilityIdentifier`. Touch targets are at least 48dp.
- **Edge to edge and insets.** `MainActivity` calls `enableEdgeToEdge()`;
  every screen pads for system bars and the keyboard.
- **System back and predictive back behave as Android users expect.** Never
  intercept back without a reason a musician would recognise (unsaved input).
- **Surface, don't swallow.** Every `ViewModel.error` has a UI surface, and no
  success feedback fires before the core confirms, as on iOS.

## Snapshots

References live in `android/app/src/test/snapshots/` and CI fails one that no
longer matches (#2259). Re-record from `android/` with
`./gradlew :app:recordRoborazziDebug`, look at the image, then commit it. Delete
a test, delete its reference (#2241).

## Eyes before the PR

A change to what a musician sees on Android follows the iOS rule: show Jon a
`just android-run` screenshot and wait for his word before `just pr-open`.

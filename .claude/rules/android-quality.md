---
paths:
  - "android/**"
---

# The Android shell

The second native shell on the unchanged core (`specs/android-shell.md`, epic
#2220). Every architecture rule in `CLAUDE.md` binds it exactly as it binds
the Swift shell; this file says how those rules, and the iOS quality and UI
bars, read in Kotlin and Compose.

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
  crash-recovery blob go to `SharedPreferences` as the core's bincode bytes.
  Each key's version comes from the core's `*BlobVersion()` export, as on iOS,
  never a number spelled in Kotlin (#1345, #2026).

## Kotlin

- **`!!` is banned like `try!` and force-unwraps in Swift**, and so are
  plain `as` casts (use `as?`) and `lateinit` on anything the core supplies. Use `?.`
  and `?:`, and surface the failure.
- **Never edit `android/generated/`.** It is git-ignored and rebuilt by
  `just android-gen`; fix the Rust type and regenerate.
- **Drive Android through the recipes, never a bare `gradlew`**:
  `just android-run`, `just android-test`, `just android-fmt`. They source
  `android/env.sh`, which finds the SDK and Android Studio's JDK; an agent
  shell has neither on its path, so a bare `gradlew` fails.
- **Run `just android-gen` after a core change.** `just android-test` reuses
  the last bindings, while CI regenerates them.
- **Before pushing, run `just android-check`, `just android-test` and
  `just android-fmt`.** `android-check` runs CI's gates: ktfmt, the build with
  Kotlin warnings as errors, Android lint with warnings as errors, and detekt
  with the Compose rules (`android/config/detekt.yml`), which bans `!!`,
  `lateinit` and Material imports; plain `as` stays a review rule. No baseline
  and no suppressions: fix the finding (#2263).
- **The comment density, bridge test and snapshot checks read Kotlin and the
  Android snapshot references** (#2265, #2241), and so does the faint ink
  check (#2309), which keeps the token off the screens; neither theme carries
  it since #1881 and #2351. The haptics check stays Swift only: the Android shell has no haptic
  helper.
- **The generated bindings compile in `:bridge`**, outside the warnings gate,
  because crux's typegen emits warnings we cannot fix at the source (a
  redundant `?` on non-null types). Hand-written Kotlin lives in `:app`.

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
  surface and string.

## The per-screen bar

Built with the screen, never retrofitted:

- **A Roborazzi snapshot** at the default font scale only; the largest-scale
  ones are set aside with #2134 and the old ones go in #2426. Unlike iOS, it
  renders through `LiveBridge`: Android has no stub bridge, and the real core
  costs nothing on the JVM.
- **TalkBack labels** on every control: a `contentDescription` or merged
  semantics that read as the iOS VoiceOver label does. A control a UI test
  drives carries a `testTag` named `screen.control`, matching the iOS
  `accessibilityIdentifier`. Touch targets are at least 48dp, Android's accessibility minimum.
- **Edge to edge and insets.** `MainActivity` calls `enableEdgeToEdge()`;
  every screen pads for system bars and the keyboard.
- **System back and predictive back behave as Android users expect.** Never
  intercept back without a reason a musician would recognise (unsaved input).
- **Surface, don't swallow.** Every `ViewModel.error` has a UI surface, and no
  success feedback fires before the core confirms, as on iOS. The surface is
  `ui/GlobalBanner.kt`, dismissed with `Event.ClearError`; a halted `Store`
  shows iOS's standing banner above it (#2266).

## Snapshots

References live in `android/app/src/test/snapshots/` and CI fails one that no
longer matches (#2259). Re-record from the repo root with
`source android/env.sh && android/gradlew -q -p android :app:recordRoborazziDebug`,
the one bare `gradlew` allowed until a recipe exists, then look at the image
before committing it. Delete
a test, delete its reference (#2241).

## Eyes before the PR

A change to what a musician sees on Android follows the iOS rule: show Jon a
`just android-run` screenshot and wait for his word before `just pr-open`.

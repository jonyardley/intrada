---
paths:
  - "android/**"
---

# The Android shell

The second native shell on the unchanged core (epic #2220). `CLAUDE.md`, and
the iOS bars in `ios-quality.md` and `ios-ui.md`, bind it as they bind Swift;
this file holds only how they read in Kotlin and Compose.

## The shell is a dumb pipe

- **No business rules in Kotlin.** If a screen needs a rule the Swift shell
  works out for itself, move it into `intrada-core` rather than port it: a
  ported rule drifts (#2223).
- **One way across the bridge.** Screens read the `ViewModel` through `Store`;
  only `LiveBridge`, `SharedItemStore` and the settings slots touch bincode or
  a UniFFI handle. Effects run off the main thread and post back on `Main`.
- **Persistence is the shared Rust store** through `StoreFfi`: Kotlin passes
  bytes and never reads a row (#2421). Singletons and the crash-recovery blob go
  to `SharedPreferences` as Base64 of the core's bincode. Each key's version
  comes from the core's `*BlobVersion()` export, never a number in Kotlin
  (#1345, #2026); the one exception, shared with iOS, is practice defaults'
  `v2`, which has no export yet (#2216).

## Kotlin

- **Banned:** `!!`, plain `as` casts (use `as?`) and `lateinit` on anything the
  core supplies. Use `?.` and `?:`, and surface the failure.
- **Drive Android through the recipes, never a bare `gradlew`**: an agent shell
  lacks the SDK and JDK that `android/env.sh` finds.
- **Run `just android-gen` after a core change**: `just android-test` reuses
  the last bindings, while CI regenerates them.
- **Before pushing, run `just android-check`, `just android-test` and
  `just android-fmt`.** `android-check` runs ktfmt, the build and lint with
  warnings as errors, and detekt (`android/config/detekt.yml`), which bans
  `!!`, `lateinit` and Material imports; plain `as` stays a review rule. No
  baseline and no suppressions: fix the finding (#2263).
- **Haptics only through `ui/Haptic.kt`** (#2488); the haptics, comment
  density, bridge test and snapshot checks all read Kotlin.
- **The generated bindings compile in `:bridge`**, outside the warnings gate,
  because crux's typegen emits warnings we cannot fix. Hand-written Kotlin
  lives in `:app`.

## Tokens and look

- **Every colour, font, spacing and radius comes from `ui/Theme.kt`.** A token
  Android lacks is copied from `Theme.swift` with the same name and value,
  never invented. Genuine one-offs stay literal, as on iOS.
- **New screens use catalogue components and theme values only**;
  `scripts/check-theme-values.sh` fails a raw colour or spacing number, and a
  genuine one-off goes on its allowlist with a reason (#2699).
- **No `MaterialTheme` colours or Material components** standing in for an
  intrada primitive.
- **Reach for a component in `design/components/README.md` first** (#2620);
  where the apps differ, its page names the look both move to.
- `docs/design-principles.md` and `docs/tone-of-voice.md` bind every Android
  surface and string.

## The per-screen bar

Built with the screen, never retrofitted:

- **A Roborazzi snapshot** at the default font scale only (#2134). It renders
  through `LiveBridge`: Android has no stub bridge.
- **TalkBack labels** on every control, reading as the iOS VoiceOver label does; a control a UI
  test drives carries a `testTag` matching the iOS identifier. Touch targets
  are at least 48dp.
- **Edge to edge**: every screen pads for system bars and the keyboard.
- **Never intercept system or predictive back** without a reason a musician
  would recognise (unsaved input).
- **Surface, don't swallow.** Every `ViewModel.error` has a UI surface, and no
  success feedback fires before the core confirms, as on iOS. The surface is
  `ui/GlobalBanner.kt`, dismissed with
  `Event.ClearError`; a halted `Store` shows the standing banner above it
  (#2266).

## Snapshots

CI fails a reference in `android/app/src/test/snapshots/` that no longer
matches (#2259). Re-record from the repo root with
`source android/env.sh && android/gradlew -q -p android :app:recordRoborazziDebug`,
the one bare `gradlew` allowed until a recipe exists, then look at the image
before committing. Delete a test, delete its reference (#2241).

## Eyes before ready

A change to what a musician sees on Android follows the iOS rule in
`.claude/rules/ios-ui.md`: a draft with the `just android-run` screenshot, and
ready only after Jon's dated yes.

# Android App Shell on the Shared Core

> Tier 3: a second native shell on the unchanged Crux core. Spec issue #1774,
> epic #2220.

## Approach

A native Android app in `android/` (Kotlin, Jetpack Compose) on the unchanged
`intrada-core`, fulfilling the same effects through the same UniFFI surface.
`intrada-ffi/src/lib.rs` pins uniffi 0.29.4; the Kotlin bindgen must match or
the bridge skews silently.

## Layer mapping

| Layer | iOS | Android |
|-|-|-|
| Packaging | `cargo swift package` plus UniFFI Swift | `cargo ndk` for `arm64-v8a` and `x86_64`, UniFFI Kotlin at 0.29.4, JNA at runtime |
| Types | Swift `SharedTypes` | `com.intrada.shared` and `com.intrada.ffi`, generated into `android/generated/`, never hand-edited |
| Store | `@Observable @MainActor Store` | A Kotlin `Store` exposing `StateFlow<ViewModel>`, effects on `Dispatchers.IO` |
| Persistence | `crates/intrada-store` through the bridge | The same store |
| Settings and crash blob | `UserDefaults`, bincode bytes | `SharedPreferences`, the same bytes as Base64, same keys and key-bump rule (#1345) |
| Metronome | `AVAudioEngine` host-time grid | `AudioTrack`, clicks at sample positions |
| Page OCR | Apple Vision | ML Kit, split into `RecognisedLine`s |
| Page suggestions | `PageSuggester` on FoundationModels | None until one is chosen (Open questions) |
| Wake lock | `ScreenWakeLock` | `FLAG_KEEP_SCREEN_ON` on the player |
| Crash reporting | Sentry | Sentry's Android SDK; a tagged Play build uploads native symbols (#2517) |
| Camera | `PageCamera` | CameraX plus the system Photo Picker |
| Design system | `Theme.swift` tokens | A Compose theme, same token names, not Material's look |
| Tests | Swift Testing, snapshots, XCUITest | `kotlin.test`, Roborazzi on the JVM, Compose UI tests on an emulator |

## The shared store

Both phones call one Rust crate rather than keeping a Kotlin copy of the
iPhone's store (#2421).

- **`crates/intrada-store`** answers a `PersistenceOperation` with a
  `PersistenceOutput`, plus the stored values it could not read. The core does
  not depend on it.
- **The migrations** keep GRDB's ids and its `grdb_migrations` table, so a
  database the iPhone wrote carries on where it is.
- **The bridge** is one `StoreFfi` object, opened at a path the phone passes
  in, called off the main thread one request at a time.
- **Settings and the crash blob** stay out of the store.
- **A database that will not open** falls back to an in-memory store and a
  degraded flag (#2428).

## Working on a Mac with no Android device

Android Studio's `arm64` emulator on Apple silicon is enough for the bridge,
persistence, screens and snapshots. It cannot judge audio latency, real camera
OCR, or haptics: have a device (a Pixel 8a, or a second-hand Pixel 7a) before
the metronome phase starts.

Play Store (settled in #2499, from Play Console Help, [testing requirements
for new personal developer
accounts](https://support.google.com/googleplay/android-developer/answer/14151465)):
a personal account opened after 13 November 2023 must run a closed test with
12 testers opted in for 14 days in a row before it can publish to production.
The internal testing track has no requirement and is the TestFlight
equivalent: `release-play.yml` uploads every release tag there. Before any
build is shared, it is walked against
[`docs/android-release-check.md`](../docs/android-release-check.md).

## Build phases

- **Phase A, bridge and boot.** `just android-gen`, the Kotlin `Store`, a
  read-only Library list.
- **Phase B, persistence.** The shared store, settings, the crash blob;
  Library add and edit (#2421, #2428).
- **Phase C, practice.** Session builder, player, session clock, the
  metronome. Have the device in hand by this phase.
- **Phase D, capture and the rest.** Camera, OCR, analytics, profile; then
  Play internal testing.

## Open questions

- **ML Kit.** The on-device text recognition API and its licensing need a
  fresh look before the capture phase.
- **Page suggestions on Android.** Decided before the capture phase: an
  on-device model behind the same suggestion shape (Gemini Nano through ML
  Kit's GenAI APIs is the candidate, with narrow device support), or none.
- **Roborazzi.** Version and CI setup against the Ubuntu runner image.

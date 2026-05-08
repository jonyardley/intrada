# Live Activity plugin (iOS)

> Spec for [intrada#TBD](https://github.com/jonyardley/intrada/issues) —
> Tier 3 per CLAUDE.md (new Tauri plugin + Widget Extension target →
> IPC contract sensitivity override + Xcode project structure change).

## Problem

The background-audio plugin (#309) keeps the practice timer running while
the device is locked and shows a Now Playing card on the lock screen. That
card is the **music-player** affordance — it works because we hijacked the
audio session, but it lies to the user about what the app actually is. A
practice session is not playback. Apple's first-class affordance for
"transient session in progress" is **ActivityKit / Live Activities**:

- Lock-screen card showing item title + position + elapsed + progress
- Dynamic Island compact / expanded views (iPhone 14 Pro and later)
- Tap-to-open returning to `/sessions/active`
- Live updates as items advance, without leaning on the audio-session hack

Once shipped, Live Activity replaces Now Playing as the primary
lock-screen surface for a practice session. Background audio still does
the job of keeping the timer alive — but the visible chrome on the lock
screen becomes practice-shaped, not music-shaped.

## Goals

1. Lock-screen Live Activity for the duration of an active practice
   session, showing current item title, position (`Item N of M`), elapsed
   time, and visual progress against `current_planned_duration_secs`.
2. Dynamic Island compact view (icon + elapsed) and expanded view (full
   item info + progress) on supported devices (iPhone 14 Pro and later).
3. Updates fire on item advance (`NextItem` / `SkipItem`) within ~1s.
4. Tap on the activity (lock-screen card or Dynamic Island) opens the app
   at `/sessions/active`.
5. Activity ends cleanly on `FinishSession` / `EndSessionEarly` /
   `AbandonSession` / app crash — no stale Live Activity in the user's
   "Recent" tray.
6. Web build remains unaffected — plugin is iOS-only, gated by platform.

## Non-goals

- **Push-driven updates.** Live Activities can be updated via Apple Push
  Notification Service for cases where the host app is suspended. We
  don't need this — the background-audio plugin keeps the app alive for
  the session's lifetime, so app-driven updates are sufficient. Avoid
  the APNs config + server work.
- **iPad.** Live Activities are iPhone-only per Apple. The plugin
  no-ops on iPad / non-iOS platforms.
- **Custom interactive controls** in the activity (pause, skip, etc.).
  Tap-to-open is the only interaction in v1 — interactive controls
  require iOS 17+ App Intents wired through the widget extension and
  introduce nontrivial scope. Defer to a v2 if user feedback asks for
  it.
- **Background audio replacement.** This plugin layers on top — it does
  not subsume the audio-session work. Both ship.

## Approach

### Architecture

```text
┌──────────────────────────────────────────────────────────────────────┐
│  WKWebView (Leptos shell)                                            │
│                                                                      │
│   The lifecycle Effect lives in <AuthenticatedApp> (#309 Phase D).   │
│   Same Effect that fires background-audio also fires live-activity:  │
│                                                                      │
│   None → Some  ──► invoke('plugin:live-activity|begin', {…})         │
│   anchor change ──► invoke('plugin:live-activity|update', {…})       │
│   Some → None  ──► invoke('plugin:live-activity|end')                │
└──────────────────────────────────────────────────────────────────────┘
                              │ Tauri IPC
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│  Rust plugin host (intrada-mobile/plugins/live-activity/)            │
│                                                                      │
│   #[tauri::command] begin, update, end                               │
│                                                                      │
│   Bridges to Swift via the Tauri 2 mobile plugin macro.              │
│   Sentry breadcrumbs / capture_message on bridge errors (mirrors     │
│   the background-audio plugin pattern from Phase D of #309).         │
└──────────────────────────────────────────────────────────────────────┘
                              │ Swift bridge
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│  Swift plugin + Widget Extension                                     │
│                                                                      │
│   Plugin (LiveActivityPlugin.swift):                                 │
│     • begin: Activity<IntradaActivityAttributes>.request(...)        │
│     • update: activity.update(using: newState)                       │
│     • end: activity.end(using: finalState, dismissalPolicy: .immediate)│
│                                                                      │
│   Widget Extension (IntradaLiveActivity.swift, separate target):     │
│     • SwiftUI views for Dynamic Island compact / expanded            │
│     • SwiftUI view for Lock Screen card                              │
│     • Observes ContentState via TimelineProvider                     │
└──────────────────────────────────────────────────────────────────────┘
```

### Why an additional plugin (not extended background-audio)

Cleaner separation of concerns: AVAudioSession + MPNowPlayingInfoCenter
is a music affordance hack; ActivityKit is the first-class iOS API for
this exact use case. They have different lifecycles (audio = always
during session; activity = once available, persists in user's Recent
tray after end), different state shapes, different failure modes. Mixing
them inside one plugin would muddy the boundaries the existing
`crates/intrada-mobile/plugins/background-audio/` cleanly draws.

The lifecycle Effect in `<AuthenticatedApp>` calls *both* plugins on the
same `Some → None` anchor transitions — the WebView shell has one
behaviour, two telemetry surfaces.

### Widget extension target

This is the architectural unknown that drives the phasing. Live
Activities require a **Widget Extension** target — a separate process
running SwiftUI code provided by Apple's WidgetKit framework. The
widget extension hosts the SwiftUI views that render the Dynamic Island
+ Lock Screen card; the main app starts/updates the activity via
ActivityKit, and the widget extension renders content from
`ContentState`.

Tauri 2's iOS project generation (`cargo tauri ios init`) produces a
single app target. Adding a widget extension means one of:

a. **Manual Xcode addition post-`ios init`** — user opens
   `gen/apple/Intrada.xcodeproj` in Xcode, File → New → Target → Widget
   Extension. Generated project is gitignored, so the user re-does this
   on every fresh checkout. Works but breaks the "clone and build"
   developer experience.
b. **Tauri config** — investigate whether `tauri.conf.json` or the
   `tauri.ios.plist` mechanism supports declaring extension targets.
   Suspect not — Tauri 2's iOS support is geared at the single-app
   model.
c. **Post-`init` script** — a shell script in
   `crates/intrada-mobile/scripts/` that mutates the generated Xcode
   project (`xcodeproj` Ruby gem or `pbxproj` direct edits) to add the
   extension target. Brittle, but reproducible.
d. **Pre-built static lib + linker flags** — ship the widget extension
   as a pre-built `.appex` bundle and hook it into Tauri's build via
   custom build script. Fragile and bypasses the SwiftPM developer
   workflow.

(c) is the most likely pragmatic path. Phase A of this work is to
validate: (1) which option is feasible, (2) whether the path involves
upstream Tauri changes. **Resolution of this question gates the rest of
the implementation phasing** — see Open questions §1.

### Plugin surface

JS-side helper in `intrada-web/src/live_activity.rs`, mirroring the
shape of `intrada-web/src/background_audio.rs`:

```rust
pub fn begin(item_title: &str, position_label: &str,
             started_at: &str, planned_duration_secs: Option<u32>);
pub fn update(item_title: &str, position_label: &str,
              started_at: &str, planned_duration_secs: Option<u32>);
pub fn end();
```

Same fire-and-forget try/catch shape as background-audio: no JS errors
if the plugin isn't available (web, simulator without ActivityKit
support, iPad). Telemetry routed via Sentry breadcrumbs in the Rust
plugin commands.

### Lifecycle integration

The lifecycle Effect (`mount_background_audio_lifecycle` in
`background_audio.rs`) gets renamed and extended — call it
`mount_session_lifecycle` and have it call both plugins on each
transition. Single derive, single ownership, both side-effects fire in
sequence.

```rust
match (prev, next) {
    (None, Some((title, pos, total, started_at, planned))) => {
        background_audio::begin_session(&title, &started_at);
        live_activity::begin(&title, &position_label, &started_at, planned);
    }
    // …
}
```

This avoids two parallel Effects observing the same state with risk of
divergent ordering.

### Activity attributes & content state

```swift
struct IntradaActivityAttributes: ActivityAttributes {
    public struct ContentState: Codable, Hashable {
        var itemTitle: String
        var positionLabel: String  // "Item N of M"
        var startedAt: Date        // wall-clock anchor for ProgressView
        var plannedDurationSecs: Int? // optional — drives ring/bar
    }

    // Static for the duration of the session — useful if we want
    // session-level metadata later (theme, instrument, etc.).
    var sessionId: String
}
```

Critical: the SwiftUI views in the widget extension use
`ProgressView(timerInterval: startedAt...endsAt)` /
`Text(timerInterval:)` so they update **automatically** between explicit
`activity.update(...)` calls. We push on item advance; iOS keeps the
elapsed-time display ticking by itself. Same wall-clock approach as
the WebView's session_timer.

## Key decisions

1. **Separate plugin, not extension of background-audio.** Cleaner
   concerns; both plugins fire from the same lifecycle Effect.
2. **Wall-clock anchor in ContentState** so Apple's `ProgressView` /
   `Text(timerInterval:)` does the per-second update for free. Avoids
   us pushing one update per second across the IPC boundary.
3. **No push-driven updates.** Audio session keeps app alive →
   app-driven updates suffice. Skips APNs setup.
4. **Tap-to-open only** in v1. Interactive controls deferred.
5. **iOS 16.1+ minimum.** Already met (app targets iOS 17+).
6. **No iPad support.** Live Activities are iPhone-only per Apple.
7. **Reuse** the existing lifecycle Effect from #309 Phase D rather
   than spawning a parallel one — one observer of session state, two
   side-effects.

## Decisions locked

- **Widget extension setup** (was open question §1): post-`init` Ruby
  script using stdlib YAML + xcodegen. Implemented in Phase A as
  `crates/intrada-mobile/scripts/add-live-activity-target.rb`.
- **Dismissal policy** (was §2): `.immediate`. The activity disappears
  from the lock screen + Dynamic Island within ~1s of session end.
  Apple's `.default` keeps it in the Recent tray for ~8h, but for a
  practice app the activity is no longer relevant once the user puts
  the instrument down. Phase C uses `.immediate`.
- **Compact Dynamic Island content** (was §3): progress arc + elapsed
  time. Arc fills against `current_planned_duration_secs`; falls back
  to indeterminate / elapsed-only when no planned duration is set.

## Open questions

1. **Activity bundle ID + signing.** The widget extension is a separate
   bundle (`com.intrada.app.LiveActivity` proposed) and needs its own
   provisioning profile. Surfaces as a new step in the "first-time iOS
   setup" section of CLAUDE.md alongside the `add-live-activity-target.rb`
   script invocation.
2. **Failure modes if `Activity.request` throws** (user disabled Live
   Activities in Settings, or the app's `NSSupportsLiveActivities`
   plist key is missing). Sentry-capture and continue silently — same
   pattern as background-audio bridge errors.
3. **Asset sharing.** If the lock-screen card uses the intrada logo or
   an item-type icon, the widget extension target needs the asset
   bundled. Determine whether to share the main-app asset catalogue or
   duplicate.

## Phasing

Spec → Plan → implement in 4 chunks, each its own PR:

1. **Phase A — widget extension target setup** (research + scaffold).
   Resolve open question §1: pick the approach, get a no-op widget
   extension target building alongside the main app. No ActivityKit
   integration yet; just prove the build pipeline. **This is the
   highest-risk phase — Tauri 2's iOS workflow may need extending.**
2. **Phase B — plugin scaffold** (medium). Empty Tauri 2 plugin under
   `crates/intrada-mobile/plugins/live-activity/` mirroring the
   background-audio plugin's structure. Three commands stubbed
   (return `Ok(())`). JS bindings module. iOS build still compiles.
   Lifecycle Effect renamed to `mount_session_lifecycle`, calling
   both plugins.
3. **Phase C — Swift implementation** (large). ActivityKit calls in
   the plugin; SwiftUI views for Dynamic Island + Lock Screen in the
   widget extension. Behaviour testable on physical device only. Sentry
   breadcrumbs per bridge call.
4. **Phase D — polish + cleanup audit** (small). Edge cases (activity
   dismissal policy choice, asset bundling, plist verification),
   instrumentation review, in-app review prompt sequencing if relevant.

## Acceptance criteria

- [ ] Starting a practice session shows a Live Activity on the lock
      screen with item title, position, and elapsed time, verified on a
      physical iPhone (not simulator — Live Activities have flaky
      simulator support).
- [ ] On iPhone 14 Pro / 15 Pro / 16: Dynamic Island compact view shows
      a leading icon + elapsed time; expanded view shows full item info
      and progress.
- [ ] Tapping the lock-screen card or Dynamic Island opens the app at
      `/sessions/active`.
- [ ] Item advance updates the activity within ~1s (no stale title).
- [ ] On `FinishSession` / `EndSessionEarly` / `AbandonSession`: the
      activity ends cleanly. No stale entry persists in the user's
      Recent tray longer than the chosen `dismissalPolicy`.
- [ ] App crash mid-session: activity ends within iOS's 12-hour active
      duration ceiling (system handles this; verify no orphan-activity
      bug).
- [ ] User has Live Activities disabled in iOS Settings: app does not
      crash; Sentry captures the
      `Activity.request` failure; timer still works (background-audio
      plugin still functions).
- [ ] iPad: plugin commands no-op; no crash, no error banner.
- [ ] Web build: dev server, web tests, e2e all pass; no plugin imports
      leak into the web bundle.

## References

- ActivityKit:
  <https://developer.apple.com/documentation/activitykit>
- Live Activity HIG:
  <https://developer.apple.com/design/human-interface-guidelines/live-activities>
- WidgetKit Dynamic Island:
  <https://developer.apple.com/documentation/widgetkit/displaying-live-data-with-live-activities>
- Tauri 2 mobile plugin docs:
  <https://v2.tauri.app/develop/plugins/develop-mobile/>
- Existing similar plugin: `crates/intrada-mobile/plugins/background-audio/`
- `Info.ios.plist` auto-merge by Tauri 2 CLI: see how
  `UIBackgroundModes:[audio]` is wired in #309 Phase C.

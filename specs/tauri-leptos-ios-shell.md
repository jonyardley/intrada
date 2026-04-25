# Tauri 2 + Leptos iOS Shell

> Status: **Phase 0 in progress**
> Author: Jon Yardley
> Created: 2026-04-25
> Tier: 3 (architectural)
> Replaces: SwiftUI iOS shell (kept on disk, disabled in CI)

## 1. Decision

Replace the SwiftUI iOS shell (`ios/Intrada/`) with a Tauri 2 + Leptos shell that
reuses the existing `crates/intrada-web/` Leptos CSR app as the WebView content.
The Crux core, API, web shell, and shared types are untouched.

The existing SwiftUI shell stays in the repository as on-hold reference code. Its
CI workflow is disabled and it stops receiving feature work. It is not deleted —
if Tauri proves untenable on iOS, reactivating SwiftUI is a CI flag flip, not a
code restoration project.

This is a Tier 3 change per `CLAUDE.md` (multi-crate, multi-week, FFI-adjacent).

## 2. Why

- **One shell to maintain.** Solo developer. Building every feature twice (web
  Leptos + iOS SwiftUI) has been the dominant cost of recent iOS work
  (#194–#201). A single Leptos codebase that runs in both browsers and WKWebView
  collapses that cost.
- **Reuse what's already built.** `intrada-web` already has CardView, ButtonView,
  EmptyStateView, autocomplete, week strip, session timer, charts, drag-and-drop
  builder — all in Leptos. The iOS shell mostly reimplements these.
- **Cross-platform parity becomes free.** New features ship on web and iOS the
  same day. The "iOS-specific" design rules in CLAUDE.md exist because the two
  shells diverged; with one shell they collapse to "iOS-shaped variants of the
  shared component."
- **Path to Android and desktop.** Tauri 2 supports both. Not a v1 goal but a
  free option.

## 3. Goals and non-goals

### Goals
- iOS app indistinguishable from a native app to a user opening it on the home
  screen — no "webby" tells (taps, callouts, font zoom, scroll bounce on chrome).
- Feature parity with the current SwiftUI iOS shell at GA: library, sessions
  (build, active, summary, history), routines, analytics, scoring, rep counter,
  tempo tracking, focus mode.
- Background-audio practice timers (lock-screen-safe) — non-negotiable.
- Single CI pipeline that builds web + iOS from the same Leptos sources.
- Existing E2E suite (Playwright) continues to validate the shared shell.

### Non-goals (v1)
- Android, desktop, watchOS, CarPlay.
- Live Activities / Dynamic Island, HealthKit, home-screen widgets — desirable
  follow-ups, not parity blockers.
- Native Clerk iOS SDK migration — Clerk-JS in WebView is acceptable for v1.
- Removing SwiftUI shell code from the repo. It stays archived.

## 4. Architecture impact

### What stays
- `intrada-core` (Crux) — untouched. Shells are still dumb pipes.
- `intrada-api` — untouched.
- `crates/shared` (UniFFI) and `crates/shared_types` (Facet → Swift typegen) —
  build kept green so the SwiftUI shell can be reactivated. Not consumed by the
  Tauri/Leptos shell.
- `intrada-web` Leptos sources — become the canonical UI codebase for both web
  and iOS.

### What changes
- New crate `crates/intrada-mobile/` containing the Tauri 2 application
  (`src-tauri/` Rust + Tauri config). It bundles the `intrada-web` Trunk output
  as its WebView content.
- New CI workflow `.github/workflows/tauri-ios.yml`.
- Existing `.github/workflows/ios.yml` disabled (workflow-level `if: false` on
  every job, plus a header comment pointing at this spec). Files preserved.
- `CLAUDE.md` rewritten:
  - "Platform priority" reframed: Leptos shell is primary; iOS = Tauri build of
    that shell; SwiftUI shell on hold.
  - "iOS-specific rules" section replaced with a "Native-feel rules for the
    Leptos shell on iOS" section (CSS reset, safe areas, View Transitions,
    haptics, etc. — see §5).
  - Tech-stack table updated: add Tauri 2, drop SwiftUI/UniFFI/BCS as active.
- `docs/roadmap.md` gets new rows for the migration phases (see §11).

### What is preserved as on-hold
- `ios/Intrada/` — all Swift code, components, views, generated types.
- `scripts/build-ios.sh` and the Fastlane setup.
- `crates/shared` and `crates/shared_types` continue to compile in CI so the
  reactivation path is always one PR away.

## 5. Tauri 2 + Leptos setup

### Repo layout
```
crates/
  intrada-mobile/
    src-tauri/
      Cargo.toml
      tauri.conf.json
      src/lib.rs        # Tauri setup, plugin registration
      gen/apple/        # generated Xcode project (gitignored except entitlements)
      icons/            # iOS icon set
    plugins/
      background-audio/ # custom Swift plugin (see §8)
      ...
```

The mobile crate is a thin Tauri host. It does not import `intrada-core`
directly — the Leptos WASM bundle does that, the same way `intrada-web` does
today.

### Tauri configuration sketch
- `productName`: "Intrada"
- `identifier`: `com.intrada.app` (reuses the SwiftUI shell's bundle ID — preserves TestFlight history, Match signing certs, and App Store Connect record)
- `build.frontendDist`: `../../intrada-web/dist`
- `build.devUrl`: `http://localhost:8080` (Trunk's default, served by `trunk serve`)
- `build.beforeBuildCommand`: `cd ../../intrada-web && trunk build --release`
- `build.beforeDevCommand`: `cd ../../intrada-web && trunk serve --no-autoreload`
- `app.security.csp`: locked-down CSP that allows Clerk CDN + the API origin
- iOS minimum deployment target: 17.0 (matches current SwiftUI shell)

### Dev loop
- `cd crates/intrada-mobile/src-tauri && cargo tauri ios dev` — builds and
  launches on simulator with HMR.
- `just ios-dev` — wrap that as a recipe to keep parity with current `just`
  recipes.

### Environment variables
Reuse the existing compile-time injection from `intrada-web`:
- `INTRADA_API_URL`
- `CLERK_PUBLISHABLE_KEY`

These need to be present in the `trunk build` step that Tauri invokes
(`beforeBuildCommand`). The current `ios.yml` does not need these because the
SwiftUI shell links a static Rust library; the new pipeline will.

### Authentication path (v1)
Clerk-JS continues to load inside the WebView via the existing
`window.__intrada_auth` bridge in `index.html`. The OAuth redirect flow
(`/sso-callback`) needs verification inside WKWebView — Clerk's redirect target
must be allowed by Tauri's CSP, and we may need to handle the redirect via
Tauri's deep-link plugin (`intrada://sso-callback`) rather than relying on
in-WebView navigation.

If redirect-based Clerk-in-WebView proves janky, fallback is
`ASWebAuthenticationSession` (native iOS in-app browser) bridged to JS. Migration
to a native Clerk iOS SDK is explicitly out of scope for v1.

## 6. Look-and-feel toolkit

The following is settled (research already done — do not re-litigate). All of
this lands as Leptos components, CSS in `intrada-web/input.css`, and small
wasm-bindgen shims. None of it is iOS-only at the codepath level — desktop web
just doesn't trigger the iOS-conditional CSS.

### CSS reset (de-webify)
- `-webkit-touch-callout: none` on chrome
- `-webkit-tap-highlight-color: transparent` everywhere
- `-webkit-user-select: none` on non-text chrome (preserve in inputs and
  selectable content)
- `font-size: 16px` minimum on `<input>` (prevents iOS zoom-on-focus)
- `touch-action: manipulation` on tappable controls
- `overscroll-behavior: none` on the root scroll container

### Safe areas
- `<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">`
  (already present in `index.html`).
- Tab bar, headers, sticky bottom bars use `env(safe-area-inset-*)`.

### Scroll containment
- App root is non-scrolling.
- Inner regions scroll with `overscroll-behavior: contain`.
- No body-level rubber-banding behind modals/sheets.

### View Transitions API
- Safari 18+ (cross-document in 18.2). iOS deployment target needs to align —
  iOS 17 falls back to no transition; iOS 18+ gets native transitions.
- Wrap `document.startViewTransition()` from Leptos via `wasm-bindgen`. Highest
  perceived-quality single addition. Use for route changes and list/detail
  navigation.

### Haptics
- `@tauri-apps/plugin-haptics` invoked from Leptos via small JS shim.
- Patterns: `selection` for tab/segment changes, `light` for button taps,
  `success` for session save, `warning` for destructive confirms.

### Native-feel components to (re)build in Leptos
| Component            | Status today                       | Action           |
| -------------------- | ---------------------------------- | ---------------- |
| Bottom sheet         | None                               | Build            |
| Action sheet         | None                               | Build            |
| Segmented control    | `type_tabs.rs` close-but-not-iOS   | iOS variant      |
| iOS switch           | None (HTML checkbox today)         | Build            |
| List rows            | `library_item_card.rs` close       | iOS variant      |
| Tab bar              | `bottom_tab_bar.rs` exists         | iOS-polish       |
| Pull-to-refresh      | None                               | Build            |
| Date picker          | Use native `<input type="date">`   | None             |

Konsta UI is the visual reference (don't import — React/Vue/Svelte only).

### Spring animations
- Motion One (~3kb gzipped) loaded as a JS dep.
- Default spring config: `stiffness: 300, damping: 30` (≈ iOS default).

### Perceived-perf habits
- Skeleton screens already exist (`skeleton.rs`) — extend coverage.
- Optimistic updates: already done for session creates; extend to item creates.
- Preload-on-touchstart for navigation links.

### Typography
- SF Pro stack: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", system-ui, sans-serif`.
- Keep `Source Serif 4` for branded display headers (already in `index.html`).

### Known unfixable weakness
- Real iOS gesture-driven swipe-back navigation cannot be fully replicated in
  a WebView. We approximate with a CSS-driven swipe gesture on the page edge
  and accept it isn't perfect. This is the single biggest "tell."

## 7. Native features matrix

| Capability                       | How                                              |
| -------------------------------- | ------------------------------------------------ |
| Haptics                          | Official `tauri-plugin-haptics`                  |
| Biometric (FaceID/TouchID)       | Official `tauri-plugin-biometric` (future)       |
| Notifications (local)            | Official `tauri-plugin-notification`             |
| Deep linking                     | Official `tauri-plugin-deep-link`                |
| Dialog (alert/confirm/save)      | Official `tauri-plugin-dialog`                   |
| Clipboard                        | Official `tauri-plugin-clipboard-manager`        |
| Filesystem                       | Official `tauri-plugin-fs`                       |
| HTTP                             | Already done in core via `crux_http`             |
| OS Info                          | Official `tauri-plugin-os`                       |
| In-app purchase (StoreKit 2)     | Community `tauri-plugin-iap` (future)            |
| Web Audio + AudioWorklet         | Browser API (free)                               |
| Media Session (lock-screen)      | Browser API (free) — pairs with background-audio |
| getUserMedia / MediaRecorder     | Browser API (free)                               |
| Web MIDI                         | Browser API (iOS 18+, free)                      |
| Screen Wake Lock                 | Browser API (iOS 16.4+, free)                    |
| Speech Synthesis / Recognition   | Browser API (free)                               |
| Pointer Events (Apple Pencil)    | Browser API (free)                               |
| **Background audio session**     | **Custom Swift plugin** — see §8                 |
| **Live Activity / Dynamic Island** | **Custom Swift plugin** — see §8 (post-v1)     |
| **MusicKit (Apple Music library)** | **Custom Swift plugin** — see §8 (post-v1)     |
| HealthKit                        | Custom Swift plugin (out of scope)               |
| AppIntents / Siri                | Custom Swift plugin (out of scope)               |
| Home-screen widgets              | Custom Swift plugin (out of scope)               |

## 8. Custom Swift plugins

Tauri 2 plugins follow a standard structure: a Rust `crates/intrada-mobile/plugins/<name>/`
directory with Swift source under `ios/`, exposed to JS via the Tauri command IPC.

### P0 — Background audio session (parity-blocker)
- **Why**: practice timers must continue running with the phone locked. Without
  this, sessions end the moment the screen times out — broken product.
- **Swift surface**: `AVAudioSession.sharedInstance().setCategory(.playback)`,
  `setActive(true)` on session start, `setActive(false)` on end. Plus
  `UIBackgroundModes: [audio]` in `Info.plist`.
- **JS surface**: `await invoke('plugin:background-audio|begin_session')` and
  `end_session()`. Optional `set_now_playing({title, artist})` that wraps
  `MPNowPlayingInfoCenter` so the lock screen shows the active piece.
- **Tied to**: Media Session API in JS for lock-screen controls (play/pause).

### P1 — ActivityKit Live Activity (post-v1, before public release)
- **Why**: showing the active session on the lock screen and Dynamic Island is
  the highest-visibility "native" cue.
- **Swift surface**: `ActivityKit` — `Activity<IntradaSessionAttributes>.request(...)`
  on session start, `update(...)` on each tick / scoring event, `end(...)` on
  finish. Widget extension target with the Live Activity views.
- **JS surface**: `start_activity({sessionTitle, totalDuration})`, `update_activity({elapsed, currentItem})`, `end_activity()`.
- **Constraint**: Live Activities require a separate widget extension target in
  the Xcode project. Tauri's `gen/apple` generation must be patched or
  post-processed to add it. Real risk; spike before committing v1 dates.

### P2 — MusicKit (post-v1, differentiator) — tracked in [#299](https://github.com/jonyardley/intrada/issues/299)
- **Why**: lets users pull pieces directly from their Apple Music library when
  building practice routines. Differentiator vs every other practice app.
- **Swift surface**: `MusicKit` — `MusicAuthorization.request()`, then library
  search via `MusicLibraryRequest<Song>`.
- **JS surface**: `await invoke('plugin:musickit|search', { query })` returns
  `[{id, title, artist, album, artwork}]`. Selected songs get persisted as
  `Piece` entries via the existing core flow.
- **Privacy**: requires `NSAppleMusicUsageDescription` in `Info.plist`.

### P3 — PencilKit, AppIntents (deferred)
- PencilKit: iPad sheet-music annotation. Fits the "Capture" pillar for v2.
- AppIntents: Siri shortcuts ("Hey Siri, start practice session"). Nice-to-have.

## 9. Component-survival audit

Before writing iOS variants, audit every existing Leptos view and component to
classify the work. Output is a single table in this doc, populated in a
follow-up PR. Categories:

- **Ports as-is** — already iOS-shaped or platform-agnostic.
- **Needs iOS variant** — same logic, iOS-shaped CSS / interaction.
- **Rebuild** — fundamentally different on iOS (e.g. native sheets vs modal).
- **Drop** — desktop-only, not needed on iOS.

Source files to audit (current count, 2026-04-25):
- `crates/intrada-web/src/views/` — 14 view files (`add_form`, `analytics`,
  `design_catalogue`, `detail`, `edit_form`, `library_list`, `not_found`,
  `routine_edit`, `routines`, `session_active`, `session_new`,
  `session_summary`, `sessions`, `sessions_all`).
- `crates/intrada-web/src/components/` — 33 components.
- `ios/Intrada/Components/` — 26 components, used as a reference for what
  iOS-shaped variants should look like.

The audit itself is its own roadmap item (§11 phase 2a) — tracked but not done
in this spec.

## 10. CI/CD

### Current state
- `.github/workflows/ci.yml`: tests, clippy, fmt, WASM build, WASM tests, E2E,
  typegen, deploy web to Cloudflare Workers, deploy API to Fly.io.
- `.github/workflows/ios.yml`: builds Rust static lib for iOS targets, builds
  iOS app via XcodeGen + xcodebuild, deploys to TestFlight via Fastlane (the
  TestFlight job is `if: false` already, gated on #216).

### Target state
- `ci.yml`: unchanged for web + API path. Continues to run on every PR/push.
- `ios.yml`: every job gated `if: false` with a header comment pointing here.
  Files preserved. The typegen + Rust shared-library jobs continue to build
  in `ci.yml` so the reactivation path stays green (typegen is already there).
- New `.github/workflows/tauri-ios.yml`:
  - **build-web-bundle** — `trunk build --release` (re-uses ci.yml build).
    Probably refactor: move `wasm-build` into a reusable workflow consumed by
    both `ci.yml` and `tauri-ios.yml` to avoid duplicate builds.
  - **build-tauri-ios** — runs on `macos-15`, installs Xcode 16.2+, runs
    `cargo tauri ios build --target aarch64-apple-ios` against the prebuilt
    web bundle. Caches `~/Library/Caches/CocoaPods` and `target/`.
  - **deploy-testflight** — gated `if: false` initially. Tracked in
    [#300](https://github.com/jonyardley/intrada/issues/300); flip when
    active development has slowed. Mirrors existing Fastlane setup
    (ASC API key, MATCH password) and reuses secrets from `ios.yml`.

### Triggers
- Run `tauri-ios.yml` on changes to `crates/intrada-core/**`,
  `crates/intrada-web/**`, `crates/intrada-mobile/**`, and
  `.github/workflows/tauri-ios.yml`.
- Concurrency group `tauri-ios-${{ github.ref }}` with `cancel-in-progress: true`.

### Caching
- `Swatinem/rust-cache@v2` shared key `tauri-ios`.
- `actions/cache@v5` for `~/Library/Caches/CocoaPods` if Tauri pulls any pods
  in (it usually doesn't, but plugins may).
- Xcode DerivedData cache via `actions/cache@v5` keyed on
  `tauri.conf.json` + plugin Cargo.tomls.

### PR previews
- Cloudflare deploys still target the web shell. PR previews (if added later)
  remain web-only. iOS preview = manual TestFlight build, gated on the
  TestFlight job once enabled.

## 11. Migration phases

Estimates assume solo evening/weekend pace. All phases ship behind their own
PR.

### Phase 0 — Spec + scaffolding (this work)
- This spec doc.
- Disable `ios.yml` jobs (workflow flip).
- Scaffold `crates/intrada-mobile/` with a hello-world Tauri shell that loads
  the existing `intrada-web` build and launches on the simulator.
- Update `CLAUDE.md` platform-priority and iOS-specific sections.
- **Exit criterion**: `cargo tauri ios dev` opens the existing web app in a
  simulator, signed-in, with API calls working.

### Phase 1 — Look-and-feel toolkit
- CSS reset (§5), safe areas, scroll containment.
- View Transitions wasm-bindgen wrapper.
- Haptics plugin wired through.
- Spring animations via Motion One.
- SF Pro typography stack.
- `<SplitView sidebar=... detail=...>` primitive (Leptos equivalent of
  SwiftUI's `NavigationSplitView`) — CSS-grid based, viewport-driven sidebar
  visibility, URL routing aware so deep-links populate both panes. Built
  early so all Phase 2/3 views are iPad-aware by default rather than
  retrofitted.
- **Exit criterion**: existing screens stop *feeling* webby — no callouts, no
  zoom, smooth route transitions, haptic feedback on tab changes. `<SplitView>`
  works across iPhone, iPad, and iPad split-screen on simulator. No new
  features yet.

### Phase 2 — Vertical slice: Practice pillar at parity
- Component-survival audit (the table) — Phase 2a.
- Build iOS variants of: tab bar, segmented control, bottom sheet, action
  sheet, list row, pull-to-refresh, switch — Phase 2b.
- Re-skin: focus mode, session active, scoring, rep counter, transition
  prompt — Phase 2c.
- Background-audio Swift plugin (P0 from §8) — Phase 2d.
- **Exit criterion**: a full practice session — start, run with phone locked,
  score, save, reflect — works on simulator and physical device, indistinguishable
  from the SwiftUI shell.

### Phase 3 — Plan + Track pillars at parity
- Library, routines, analytics screens re-skinned.
- (iPad `<SplitView>` already in place from Phase 1; views just opt in.)
- **Exit criterion**: full feature parity with `ios/Intrada/`.

### Phase 4 — Dogfood on device (Xcode sideload)
- Build the Tauri shell to a physical iPhone via Xcode (no TestFlight, no App
  Store — still actively developing).
- Run a real practice session locked-screen with background audio.
- Profile on iPhone 12 (oldest supported device) for animation/scroll
  performance.
- **Exit criterion**: full flow works on physical device; performance
  acceptable; no P0/P1 bugs found in self-use.
- **TestFlight is deferred** — see [#300](https://github.com/jonyardley/intrada/issues/300),
  flip the gate when active development has slowed and there are real testers
  to receive builds.

### Phase 5 — Shell-of-record cutover
- `CLAUDE.md` Tier 3 SwiftUI rules removed (already moved to "on hold" in
  Phase 0; this is the formal sunset).
- `ios/Intrada/` and `crates/shared/` remain in repo, marked `// ON HOLD —
  see specs/tauri-leptos-ios-shell.md` in lib.rs.
- File the 6-month sunset wakeup issue (per Q7) for `crates/shared` deletion.
- **Exit criterion**: all new iOS work goes through Tauri; SwiftUI shell is
  reference-only; CLAUDE.md reflects the new reality.
- **App Store submission is deferred** — see [#301](https://github.com/jonyardley/intrada/issues/301),
  separate milestone gated on product readiness, not just shell readiness.

### Phase 6+ — Native enhancements (post-cutover)
- ActivityKit Live Activity (P1).
- MusicKit integration (P2).
- PencilKit, AppIntents (P3).

## 12. Risks and mitigations

### App Store review (Guidelines §4.2 / §4.3)
- **Risk**: Apple has historically rejected "wrapped web" apps that don't
  demonstrate substantial native value-add.
- **Mitigation**: Tauri apps with custom native plugins (background audio,
  Live Activities, MusicKit) clear the bar comfortably. Bundle ID continuity
  with the SwiftUI build helps if any TestFlight history exists. App Store
  submission tracked separately in [#301](https://github.com/jonyardley/intrada/issues/301);
  not gated on the shell-of-record cutover (Phase 5).

### WKWebView performance for animation-heavy screens
- **Risk**: 60fps for charts (line_chart, tempo_progress_chart) and the
  session timer is achievable but not guaranteed in WKWebView.
- **Mitigation**: profile in Phase 1 on iPhone 12 (oldest supported device).
  Charts can fall back to static snapshots during transitions. Use View
  Transitions API for navigation rather than CSS transforms.

### Loss of SwiftUI investment
- **Risk**: ~26 components and 5 view directories of polished SwiftUI work get
  shelved.
- **Mitigation**: keep on disk; don't delete. Many design decisions
  (NavigationSplitView, .confirmationDialog patterns) translate directly to
  Leptos equivalents. The work is not wasted — it's a reference design.

### Tauri 2 mobile maturity
- **Risk**: Tauri 2 mobile is stable but younger than SwiftUI; smaller
  Apple-platform community; plugin ecosystem still maturing.
- **Mitigation**: Phase 0 scaffolding is the spike — if the basic loop
  doesn't work, we know before committing to Phase 1+. The reversion path
  (re-enable `ios.yml`) is one CI flip away.

### Clerk-JS in WKWebView OAuth flow
- **Risk**: Google OAuth redirect-based flow may behave unexpectedly inside
  a WebView (Google has historically blocked OAuth in plain WebViews).
- **Mitigation**: spike in Phase 0. Fallback options, in order of preference:
  Tauri deep-link plugin → `ASWebAuthenticationSession` bridge → native Clerk
  iOS SDK. Don't migrate to native Clerk SDK preemptively; only if the
  WebView path doesn't work.

### Bundle ID and TestFlight continuity
- **Risk**: switching bundle IDs would lose TestFlight history and any
  external testers.
- **Mitigation**: keep the same bundle ID (`com.intrada.app` or whatever is
  registered) — Tauri config lets us choose it. See open questions.

### Background-audio plugin is a hard dependency
- **Risk**: if the background-audio plugin doesn't work cleanly (e.g.
  AVAudioSession conflicts with other apps' audio), the practice flow is
  broken on iOS.
- **Mitigation**: the SwiftUI shell already uses `AVAudioSession.playback` —
  port that exact configuration. Spike in Phase 2d before committing the
  rest of Phase 2.

## 13. Open questions

1. ~~**Bundle ID**~~ — **Resolved 2026-04-25**: reuse `com.intrada.app`
   throughout. SwiftUI shell sideloaded from local builds when comparison is
   needed; no parallel bundle ID during the parity period.
2. ~~**Tauri crate location**~~ — **Resolved 2026-04-25**: `crates/intrada-mobile/`
   with the Tauri host code at `crates/intrada-mobile/src-tauri/`. Keeps
   workspace consistency; nests plugin sub-crates cleanly.
3. ~~**Clerk auth in WebView**~~ — **Resolved 2026-04-25** (decision tree, not
   outcome). Phase 0 spike runs the actual test. Fallback ladder:
   1. **Try first**: Clerk-JS in the Tauri WKWebView, redirect-based Google
      OAuth flow as today.
   2. **If Google blocks the WebView UA**: deep-link bridge — open the OAuth
      URL in `ASWebAuthenticationSession` (system browser sandbox, gets a
      pass from Google), catch the redirect via Tauri's deep-link plugin,
      hand the URL back to Clerk-JS to finish the session. ~1–2 days.
   3. **If the bridge is too janky**: escalate before committing to native
      Clerk iOS SDK migration — that's v1 scope creep and triggers a
      re-plan.
4. ~~**iPad shell**~~ — **Resolved 2026-04-25**: build `<SplitView>` as a
   Phase 1 toolkit primitive, not a Phase 3 retrofit. Building views
   list-or-detail-pane-aware from the start is far cheaper than retrofitting.
   Adds ~2–3 days to Phase 1 plus an exit-criterion clause.
5. ~~**iOS-conditional CSS**~~ — **Resolved 2026-04-25**: runtime platform
   flag on `<html data-platform="ios">`, injected by Tauri via
   `app.windows[].initializationScript`. CSS uses `[data-platform="ios"]`
   selectors. Single bundle for web and iOS; web E2E coverage stays valid;
   iPhone Safari users keep desktop-style behaviour (e.g. text selection in
   notes) instead of getting iOS reset rules applied just because the
   viewport is small. Leaves room for `data-platform="android"` later.
6. ~~**MusicKit timing**~~ — **Resolved 2026-04-25**: deferred to post-cutover
   fast-follow. Tracked in [#299](https://github.com/jonyardley/intrada/issues/299).
   v1 ships without it; not a parity regression (SwiftUI shell doesn't have it
   either). Becomes the headline first-new-feature post-Tauri.
7. ~~**Dropping `crates/shared` long-term**~~ — **Resolved 2026-04-25**:
   sunset window with explicit trigger. Keep `crates/shared`,
   `crates/shared_types`, `ios/Intrada/`, `scripts/build-ios.sh`, and
   `ios.yml` (disabled) through Phase 5 cutover plus a 6-month soak. At
   Phase 5, file a wakeup issue scheduled ~6 months out. If by then SwiftUI
   has not been reactivated and Tauri is stable, that issue's job is to file
   the deletion PR. Avoids both the "delete the safety net at the moment of
   highest risk" failure (option 1) and the "carry dead code forever"
   failure (option 2).

## 14. Milestones and roadmap impact

New roadmap rows under a "Mobile shell" section in `docs/roadmap.md`:

| # | Phase | Size | Status |
|---|-------|------|--------|
| ? | Phase 0 — Spec + Tauri scaffold + CI flip | M | Now |
| ? | Phase 1 — Look-and-feel toolkit | L | Now |
| ? | Phase 2 — Practice pillar at parity (incl. background-audio plugin) | XL | Next |
| ? | Phase 3 — Plan + Track pillars at parity | L | Next |
| ? | Phase 4 — Dogfood on physical device (Xcode sideload) | S | Next |
| ? | Phase 5 — Shell-of-record cutover (CLAUDE.md, on-hold markers) | S | Later |
| [#300](https://github.com/jonyardley/intrada/issues/300) | TestFlight enablement (deferred) | S | Future |
| [#301](https://github.com/jonyardley/intrada/issues/301) | App Store submission (deferred) | M | Future |
| ? | ActivityKit Live Activity | M | Future |
| [#299](https://github.com/jonyardley/intrada/issues/299) | MusicKit integration | L | Future |

Issue numbers to be assigned when filing on the GitHub board. Existing iOS
issues (#194, #195–201, #202) stay closed (work was real, just superseded).
The SwiftUI design-system work is referenced in Phase 2a as the source-of-truth
for what iOS-shaped variants need to look like.

## 15. Phase 0 setup log

Issues encountered during the initial scaffold and `cargo tauri ios init` run,
in order. Captured so the setup path is predictable for future contributors.

### 1. `cargo-tauri` CLI not installed
- **Symptom**: `cargo tauri ios init` → `error: no such command: tauri`
- **Cause**: `cargo-tauri` is a separate binary, not part of the workspace.
- **Fix**: `cargo install tauri-cli --version "^2" --locked`

### 2. CI failing — glib-sys build error on ubuntu-latest
- **Symptom**: Test and Clippy CI jobs fail with `failed to run custom build
  command for glib-sys v0.18.1`.
- **Cause**: Tauri 2 pulls in GTK/glib as a desktop rendering backend on Linux.
  The ubuntu-latest runner doesn't have GTK system libraries installed.
- **Fix**: Add `--workspace --exclude intrada-mobile` to `cargo test` and
  `cargo clippy` in `ci.yml`. `intrada-mobile` is an iOS-only host with no
  meaningful unit tests; excluding it from Linux CI is correct.

### 3. `tauri.conf.json` schema error — `initializationScript` not allowed
- **Symptom**: `cargo tauri ios init` → `"tauri.conf.json" error on app >
  windows > 0: Additional properties are not allowed ('initializationScript'
  was unexpected)`
- **Cause**: `initializationScript` was a valid window config property in
  Tauri v1 but was removed from the JSON schema in Tauri v2.
- **Fix**: Removed `initializationScript` from `tauri.ios.conf.json`. Inject
  `data-platform="ios"` via `setup` + `eval` in `lib.rs` instead, scoped to
  `#[cfg(target_os = "ios")]`.

### 4. `tauri.conf.json` — `apple.development-team` empty
- **Symptom**: `cargo tauri ios init` → `Error failed to create Apple
  configuration: apple.development-team is empty`
- **Cause**: `bundle.iOS.developmentTeam` was left as `""` in the initial
  scaffold.
- **Fix**: Set to `9S5FG4LQAF` (matches `ios/project.yml`
  `DEVELOPMENT_TEAM`). Find your team ID in Xcode → Settings → Accounts, or
  at developer.apple.com → Membership.

### 5. CocoaPods not installed
- **Symptom**: `cargo tauri ios init` → `failed to run command pod install:
  Failed to install cocoapods: No such file or directory`
- **Cause**: Tauri's iOS init requires CocoaPods to manage native dependencies.
  It tried Homebrew first (package not found) then gem (needs sudo, rejected).
- **Fix**: `brew install cocoapods` before running `cargo tauri ios init`.

### 6. Missing window `label` in tauri.conf.json
- **Symptom**: Window/capability mismatch — capabilities reference `"main"`
  but window had no label field.
- **Cause**: Oversight in initial scaffold; Tauri requires the label to match
  capability window references.
- **Fix**: Added `"label": "main"` to the window object in `tauri.conf.json`.

### 7. Tauri iOS dev can't reach Trunk dev server
- **Symptom**: `cargo tauri ios dev` spams `Waiting for your frontend dev server
  to start on http://192.168.0.x:8080/...` then times out after 180s.
- **Cause**: Trunk binds to `127.0.0.1` by default. Tauri iOS dev resolves the
  host machine's LAN IP (e.g. `192.168.0.11`) so the simulator can reach it —
  `localhost` from the simulator's perspective isn't the host. Trunk isn't
  listening on that interface, so Tauri's health check loop never succeeds.
- **Fix**: Add `--address 0.0.0.0` to the `trunk serve` call in the `ios-dev`
  justfile recipe. This makes Trunk listen on all interfaces including the LAN
  IP. Only applied in `ios-dev`, not in the regular `dev` recipe.

### 8. xcodebuild timed out — no simulator installed, iPad disk image failed
- **Symptom**: `cargo tauri ios dev` picks up the connected iPad, fails with
  `The developer disk image could not be mounted on this device`, then times out.
- **Cause**: `cargo tauri ios dev` targets any available iOS destination. If no
  simulator runtime is installed, Xcode falls back to a connected physical device.
  iPads on newer iOS versions may require a developer disk image that the
  installed Xcode version doesn't ship with.
- **Fix**: Install the iOS Simulator runtime in Xcode → Settings → Platforms →
  iOS Simulator. Even with the simulator installed, `cargo tauri ios dev`
  prefers a connected physical device. Pass the simulator name as the
  positional `[DEVICE]` argument (not `--target`, which doesn't exist). The
  `ios-dev` justfile recipe now detects the first available iPhone simulator
  via `xcrun simctl list devices available` and passes its name explicitly.

### 9. `mapfile: command not found` in ios-dev recipe
- **Symptom**: `mapfile: command not found`, recipe exits with code 127.
- **Cause**: `mapfile` (aka `readarray`) was added in bash 4. macOS ships
  bash 3.2 at `/bin/bash` due to GPL licensing. The justfile recipe shebang
  `#!/usr/bin/env bash` resolves to 3.2 unless Homebrew bash is installed and
  first on PATH.
- **Fix**: Replace `mapfile -t ARRAY < <(...)` with a `while IFS= read -r line;
  do ARRAY+=("$line"); done < <(...)` loop, which is bash 3.2 compatible.

### 10. `just ios dev` / `just ios` recipe not found
- **Symptom**: `just ios dev` → `Justfile does not contain recipe 'ios'`
- **Cause**: Two issues. First, `just ios dev` (space) invokes recipe `ios`
  with argument `dev` — the recipe is named `ios-dev` (hyphen). Second, the
  old `ios` recipe was renamed to `ios-swiftui` when the SwiftUI shell went
  on hold.
- **Fix**: Use `just ios-dev` (hyphenated) from anywhere in the repo. `just`
  searches upward for the justfile, so it works from any subdirectory. Or run
  `cargo tauri ios dev` directly from `crates/intrada-mobile/src-tauri/`.

---

## 16. Out of scope for this spec

- Detailed Swift code for any custom plugin — written when the plugin is built.
- Component-survival audit table contents — Phase 2a deliverable.
- Android — explicitly deferred. Tauri makes it possible; not committing to it.
- Migration of any data (no schema changes; the app talks to the same API).

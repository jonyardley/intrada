# Native iOS Shell (SwiftUI + Crux)

> Tier 3 (Crux FFI bridge + new persistence/sync capability, multi-week,
> spans core + a net-new native shell). Spec rides with the Phase A branch per
> CLAUDE.md. Reverses PR #382, which removed the prior SwiftUI shell — the
> "app-first + local-first" framing below is the deliberate answer to *why
> reverse it*.

## Problem

intrada's iOS app today is a Tauri 2 WKWebView wrapping the Leptos web shell.
At the piano — the primary place the app is used — a webview doesn't feel like
a native instrument-side tool: scroll/gesture/transition fidelity, launch feel,
and platform affordances all read as "web in a frame."

We have a pure Crux (Rust) core that already owns all business logic. We have
never actually used Crux as designed — a pure core with a *thin native shell*
over FFI/typegen. The Tauri shell skips that step. Going native iOS finally
exercises the Crux payoff and sets up native Android later on the same core.

A complete native SwiftUI shell **already existed and was removed in PR #382**
(`bdd815c..84b26d6`) — it reached Plan/Practice/Track + iPad parity over
HTTP-through-core. So this is **resurrect-and-modernize, not greenfield**: the
core is fully intact and the old pipeline is recoverable via `git show
84b26d6^:<path>`.

## Approach

Resurrect the proven #382 architecture and modernize it (Swift 6 strict
concurrency, `@Observable`, NavigationStack/SplitView, Swift Testing, iOS 26
SDK), while adding the one thing #382 never had: **local-first persistence**.

Strategic decisions (settled; not re-litigated here):

- **App-first.** Native iOS now, native Android later — both on the shared
  Crux core. Web stays on Leptos, untouched. Web's data model is decided later.
- **Local-first.** On-device SQLite is the source of truth. The existing
  Axum+Turso backend demotes from live data source to a **sync target**.
- **Sync scope is deliberately small.** Single-user, multi-device,
  **last-write-wins on `updated_at` — not CRDTs**, not multi-user collaboration.

## The Crux → Swift bridge

The core stays pure and synchronous; all I/O is an `Effect` the shell fulfils.
The shell is a **dumb pipe** — it owns zero domain logic (see CLAUDE.md "Native
iOS Shell" section).

- **FFI:** UniFFI exposes the three byte-buffer bridge methods
  (`update` / `resolve` / `view`, unified in Crux 0.17). Bytes in, bytes out.
- **Typegen:** `facet-generate` (≥0.17) + the `facet_typegen` feature on
  `crux_core` 0.18 generates a Swift package of `Event` / `Effect` /
  `ViewModel` + domain types and their bincode (BCS) serializers. The core
  types in `app.rs` (`Event`, `AppEffect`, and `Effect` via
  `#[effect(facet_typegen)]`) get their `#[derive(Facet)]` (+ `#[repr(C)]` on
  enums) restored behind the feature (they were stripped in #382).
- **Packaging:** **spike `cargo-swift`** (the Crux book's current documented
  path: `cargo-swift` + `xcodegen` + `just`) which produces a Swift package
  wrapping the compiled static lib. **Fallback:** revive the hand-rolled
  `scripts/build-ios.sh` from #382 if cargo-swift fights us. Decision resolved
  by the Phase A spike.
- **HTTP-through-core:** the `Http` effect carries a fully-built `HttpRequest`
  (url/method/headers/body); the Swift shell runs `URLSession`, adds the
  platform auth header (Clerk JWT / iOS PAT), handles the 401-retry, and
  resolves the response bytes. No domain types cross the boundary as anything
  but bytes.

### Known build hazard — UniFFI #2818

UniFFI-generated Swift fails to compile under Xcode 26 / Swift 6.2 when the
module defaults to `MainActor` isolation ([mozilla/uniffi-rs#2818], open since
Feb 2026). Mitigation baked into the build recipe: keep the generated `Shared`
package **non-MainActor-defaulted**, or post-process the generated Swift to
prepend `nonisolated` (a `just` step) until UniFFI ships a config option.

[mozilla/uniffi-rs#2818]: https://github.com/mozilla/uniffi-rs/issues/2818

## NEW capability — local-first persistence + sync

Crux removed the Capability API in 0.17 (the core already uses `Command`
exclusively — see `Command::all` / `Command::notify_shell` in `app.rs`). So
persistence is **not** a custom capability; it's a **custom `Effect` driven by
`Command`**.

- **Core side:** add a `Persistence` effect variant carrying typed
  query/mutation operations (an `Operation` whose `Output` is the typed result
  — same shape as `AppEffect`). The core decides *what* to read/write and runs
  **LWW reconciliation**; that logic lives in core so it's shareable to Android.
- **Shell side:** **GRDB owns the real SQLite tables** and fulfils the
  persistence effect off the main actor, returning serialized rows. `crux_kv`
  is used **only** for small singletons (settings, `session-in-progress`
  crash-recovery), never for relational data.
- **Sync (LWW) design, with the known pitfalls handled:**
  - **Server-authoritative `updated_at`** stamped on write (avoids device
    clock skew silently losing newer edits).
  - **Tombstones** (`deleted_at`) compared under the same LWW rule (avoids the
    delete-vs-update resurrection race), GC'd after a safe window.
  - **Deterministic tiebreak** on equal timestamps (compare ulid / device id)
    so all devices converge on the same winner.
  - **Row-level** LWW (documented; field-level is out of scope for a
    single-user practice app).
  - Offline-first: writes apply locally immediately; sync is deferred and
    retried.

## Phased plan

- **Phase 0 (done):** focus mode (web/Tauri/API CI main-only + Dependabot
  pause) and iOS agent tooling (XcodeBuildMCP, swift-format hook, sourcekit-lsp
  plugin).
- **Phase A (this spec):** restore typegen attrs to core (TDD); new
  `crates/intrada-ffi` (UniFFI + facet typegen); cargo-swift spike → minimal
  SwiftUI app that sends one `Event` and renders the `ViewModel`. Bake in from
  day one: swift-snapshot-testing (pinned sim + runtime, perceptual tolerance),
  Sentry, accessibility (VoiceOver + Dynamic Type). **Gate:** pipeline green on
  device + simulator, #2818 mitigated.
- **Phase B:** persistence `Effect` + GRDB store + schema + LWW scaffolding;
  first real screen (Library) end-to-end local-first. **Gate:** per-screen cost
  + persistence design validated.
- **Phase C:** screen-by-screen parity (Plan → Practice → Track), each with
  iPad `SplitView` built *with* the screen (not retrofit), per-screen
  accessibility, snapshot tests, and HIG-referenced native shape/behaviour.
- **Phase D:** LWW sync to the Axum API; retire the Tauri host at parity.

## Quality baked in (day one, not retrofit)

- **swift-snapshot-testing** (pointfreeco, 1.19.x) — the automated agent's
  "eyes" for UI regression. Pin one CI simulator + iOS runtime; use perceptual
  tolerance; reference images are reviewable diffs.
- **Sentry** crash reporting from the first build.
- **Accessibility** per screen — VoiceOver labels + Dynamic Type — verified as
  each screen lands, not at the end.

## Open questions

- **Web data model** — deferred; web shell stays online-only for now.
- **Per-screen rewrite cost** — answered at the Phase B gate (first real screen).
- **cargo-swift vs build-ios.sh** — answered by the Phase A spike.
- **Sync transport detail** (full-table vs delta, batching) — designed in Phase D.

## Decisions log

- 2026-05-31 — App-first, local-first, LWW-not-CRDT. (Reverses #382.)
- 2026-05-31 — Build pipeline: spike cargo-swift first, build-ios.sh fallback.
- 2026-05-31 — Persistence is a custom `Effect`/`Command`, not a Capability
  (Capability API removed in Crux 0.17).
- 2026-05-31 — SQLite owned by the Swift shell (GRDB); reconciliation in core.
- 2026-05-31 — Mitigate UniFFI #2818 in the build recipe (non-MainActor /
  `nonisolated` post-process).

## YAGNI (explicitly out of scope for now)

Android shell, Xcode Cloud, fastlane match, localization.

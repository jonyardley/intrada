# intrada Development Guidelines

> Last reviewed: 2026-08-05.

> ## ⚠️ CURRENT FOCUS: NATIVE iOS ONLY — the web and Tauri shells are removed
>
> As of 2026-07, the **only** shell is the **native SwiftUI iOS app** (on the
> Crux core — see [`specs/native-ios.md`](specs/native-ios.md)). The Leptos
> web shell (`crates/intrada-web`) and the Tauri iOS host
> (`crates/intrada-mobile`) were **deleted** (see
> [`docs/rebuild-review.md`](docs/rebuild-review.md) for the rationale) — do
> not resurrect them or add new features assuming they exist. New UI work
> lands in the native iOS app. Design happens in **Claude Design** (see
> [`docs/design-workflow.md`](docs/design-workflow.md)); the living reference is
> [`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html),
> derived from `Theme.swift`. Implementations target SwiftUI. If a request
> seems to imply web/Leptos work, confirm the platform before writing code —
> that shell no longer exists in this repo.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics. Organised around three pillars:
**Plan** (library, routines), **Practice** (focus mode, timers, scoring),
**Track** (analytics, insights).

**Platform**: The **native SwiftUI iOS app** (on the Crux core) is the only
shell. The Leptos web shell and the Tauri 2 iOS WKWebView host were removed
(see the banner above); two pieces of native Swift worth reusing from the
Tauri plugins (background audio session handling, a Live Activity
implementation) were preserved as reference under `ios/Reference/` — see its
README.

## Project Structure

```text
crates/
  intrada-core/          # Pure Crux core — business logic, no I/O
  intrada-ffi/           # UniFFI bridge — generates the Swift bindings
  intrada-api/           # REST API — Axum 0.8 + Turso (libsql)
ios/                     # Native SwiftUI app (Intrada.xcodeproj via xcodegen)
  Reference/             #   Reference Swift preserved from the removed Tauri shell (not built)
content/                 # Practice-coach authored content — the engine's gates.toml parser reads it (#1194)
design/                  # Claude Design system (intrada-design-system.dc.html)
docs/                    # Roadmap (direction/phases) + status.md (what's in flight now)
specs/                   # Spec docs for major features (Tier 3 only — see Workflow)
```

## Tech Stack

- **Rust** stable (1.90.0 CI; MSRV 1.75+, intrada-api 1.78+)
- **Core**: crux_core 0.19.0, serde, ulid, chrono, thiserror
- **API**: axum 0.8, tokio, libsql 0.9 (Turso), tower-http (CORS), jsonwebtoken 10
- **Native iOS**: SwiftUI, iOS 17.0+, UniFFI + facet typegen for bindings, GRDB (on-device persistence)
- **Auth**: Clerk (Google OAuth) in the browser flow, exchanged for a long-lived PAT on iOS; JWT RS256 against JWKS
- **DB**: Turso (managed libsql/SQLite) via HTTP
- **CI/CD**: GitHub Actions → Fly.io (API) + native iOS build/test; TestFlight via a separate release lane

## Commands

```bash
just check                 # fmt-check → lint → test → hygiene; mirrors CI — run before push
just test                  # nextest, same as CI's `test` job
just lint                  # clippy -D warnings, same targets as CI's `clippy` job
just hygiene               # typos + cargo-shear (CI's Security & hygiene job)
cargo test -p intrada-api  # API tests only
just ios-fmt               # native app: format Swift sources in place (swift format)
just ios-fmt-check         # native app: Swift formatting gate (CI runs this too)
just ios                   # native app: regen bindings (if core changed) + open Xcode
just ios-run               # native app: build + launch on simulator + screenshot
just ios-logs              # native app: stream booted-sim logs, filtered to our subsystem
just testflight            # native app: signed Release .ipa → TestFlight (needs setup; see below)
```

`just testflight` builds a signed Release `.ipa` and uploads it to TestFlight
(internal testing), mirroring the `.github/workflows/release-testflight.yml` CI
lane (which runs on `workflow_dispatch` / `v*` tag, never per-PR). Signing is
fastlane **match**; needs Ruby ≥ 3 (system Ruby 2.6 is too old — use `rbenv`)
plus a one-time App Store Connect + match bootstrap. Full setup + decisions:
[`specs/ios-testflight-cicd.md`](specs/ios-testflight-cicd.md) and SETUP.md §6a.

`just ios-logs` filters the unified log to `subsystem == "com.intrada.native"`,
cutting the simulator's UIKit/keyboard/gesture noise so first-party signal is
visible. `report(_:)` (`ios/Intrada/Core/Logging.swift`) logs swallowed FFI /
bincode bridge errors there — the silent-no-op class (#846) that otherwise
leaves no trace where Sentry has no DSN (CI always; dev unless
`SENTRY_DSN_NATIVE` is set — see Environment Variables).

`just ios` / `just ios-run` auto-regenerate the Swift bindings only when
`intrada-core`/`intrada-ffi` changed (a `ios/generated/.gen-stamp` hash), so
they stay in sync without slowing pure-Swift edits. `just ios-gen` forces a
full regenerate.

**Simulator build/snapshot/UI testing** — the `xcrun simctl` / `xcodebuild`
CLI workflow, worktree sim-isolation, test tiering (`just ios-test` vs
`-full`), the green-stamp skip, and host gotchas are all documented in
[`docs/ios-testing.md`](docs/ios-testing.md). The one rule worth stating here
because it's a safety behaviour, not reference detail: the iOS Simulator and
`CoreSimulatorService` are machine-global, so before any global reset
(`simctl shutdown all`, `killall CoreSimulatorService`) or a fresh test run,
check for another live session (`xcrun simctl list devices | grep Booted`;
`pgrep -fl 'xcodebuild|XCTestAgent'`). If anything you didn't start is active,
**pause and ask the user** rather than risk killing their running sim/tests.

**Demo data vs. real on-device data.** A plain launch (`just ios` → Cmd+R on the
default **Intrada** scheme, or any build with no launch args) runs
**local-first**: the Library hydrates from the on-device GRDB store, so items
you add survive restarts. The 6 sample pieces are **opt-in** via the
`--seed-sample-data` launch arg. In Xcode, pick the **Intrada (Seeded)** scheme
from the scheme dropdown (defined in `ios/project.yml`) and Cmd+R — the
selection persists across `just ios` regenerations. `just ios-run` passes the
same arg by default (`SEED=1`); use `SEED=0 just ios-run` to launch against your
real data. Seed mode (`Event::LoadSampleData`) replaces the model with demo
items and **skips store hydration**, so don't use it when testing persistence —
your saved rows are still on disk but won't be read back.

Run `just check` *locally before pushing*, not just before committing. Its
`fmt-check` / `lint` / `test` / `hygiene` recipes mirror CI's fmt, clippy,
test, typos and cargo-shear gates with the exact flags and crate exclusions,
so local green means CI green (cargo-deny/Gitleaks run in CI only); pushing
then watching CI fail wastes a full ~3-minute roundtrip per agent or
contributor. Keep the justfile recipes and ci.yml in lockstep when either
changes.
Changes under `ios/` additionally need `just ios-fmt-check` (fix with
`just ios-fmt`); the native-ios CI job enforces it. It covers the hand-written
trees (`ios/Intrada`, `ios/IntradaTests`, `ios/IntradaUITests`) with the
toolchain-bundled `swift format` on default config. `ios/generated` is excluded
(generated bindings, never hand-edited). The one-time whole-tree reformat commit
is listed in `.git-blame-ignore-revs`; run
`git config blame.ignoreRevsFile .git-blame-ignore-revs` once so `git blame`
skips it.

Git hooks install automatically for Claude Code sessions (a `SessionStart`
hook runs `scripts/install-git-hooks.sh`), catching the "pushed onto a
merged-PR branch and the commits orphaned" pitfall via a pre-push check
against `gh`. Manual install, forking setup, and first-time iOS setup are in
the [README](README.md#prerequisites) — read that before your first `just ios`.

## Knowledge graph (graphify)

A graphify knowledge graph of the repo lives in the **main checkout** at
`graphify-out/` (gitignored; worktrees don't carry it. Find the main checkout
from any worktree via `git rev-parse --path-format=absolute --git-common-dir`,
then its parent). Scope is controlled by the committed `.graphifyignore`:
vendored/minified JS, `specs/_archive/`, `.specify/` and generated schemas are
excluded. **Never build or update the graph without that file in place**: an
unscoped run pollutes the god nodes with minified symbols and burns tokens on
retired docs.

- **Query it first for architecture, cross-document, or spec-archaeology
  questions** ("what touches persistence across core, specs, and the
  offline-first invariants?"): from the main checkout root run
  `graphify query "<question>"`; `graphify path A B` traces how two concepts
  connect. For symbol-level code navigation use grep/LSP instead; it's faster
  and always current.
- **Refresh**: the post-commit/post-checkout hooks in the main checkout do
  free AST-only code refreshes automatically. After doc-heavy merges (specs/,
  docs/, CLAUDE.md), run `graphify . --update` from the main checkout;
  incremental and content-hash cached, so it costs a small fraction of a full
  build.
- An empty or weak query result means the graph can't answer it: fall back
  to grep/LSP; don't trigger a rebuild for one question.

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User → Events → crux_core (Rust) → Effects (Http, Persistence, App, Render) → Shell (Swift) → I/O
```

1. **Core owns all logic.** HTTP requests built in core via `crux_http`. Core does
   all JSON serialization. The shell never understands domain types.
2. **The shell is a dumb pipe.** Receives `HttpRequest` (URL, method, headers, bytes)
   and fulfils it via `URLSession`, returns `HttpResponse`. No domain type imports.
3. **Typed bindings, no hand-written FFI.** `Event`/`Effect`/`ViewModel` cross the
   bridge via generated bincode serializers (facet typegen + UniFFI) — Swift never
   hand-encodes a domain type.

### State boundary

| State kind | Where it lives |
|------------|---------------|
| Domain data | Crux `Model` → `ViewModel` (single source of truth) |
| UI interaction | SwiftUI `@State` / `@Observable` view state |
| Crash recovery | iOS UserDefaults (`AppEffect::SaveSessionInProgress`) |
| Local-first persistence | On-device GRDB/SQLite (`PersistenceOperation` effect) |

Domain state flows through `Event` → `Model` → `ViewModel`. Never store domain
data in shell-local state. UI-only state stays in SwiftUI.

### Other patterns

- **Validation**: `intrada-core/src/validation.rs` is the single source of truth
- **DB**: Positional column indexing with `SELECT_COLUMNS` const
- **Migrations**: Sequential in `intrada-api/src/migrations.rs`, one SQL statement each
- **Mutate response**: Writes reconcile with the server response directly — no
  full-list refetch. Three create variants live in the codebase; pick the one
  that matches the entity's shape:
  - **Temp-id mutate-response** (`Item`): domain handler pushes the
    optimistic entry with a client-generated ulid; HTTP wrapper carries that
    ulid; `*Created { temp_id, entity }` event replaces the optimistic entry
    (server-assigned ulid differs from the client one). Default for new
    entities — use this unless one of the others applies.
  - **Client-owned ulid** (`Session`): client ulid is the canonical id. POST
    is fire-and-forget — `SessionSaved` just clears the error state and the
    model keeps the optimistic write.
  - **Save-counter + refetch** (`Set`): optimistic push + bump
    `set_saves_committed` + full refetch via `SetSaveSucceeded`. The counter
    drives the save-form's optimistic→confirmed UI flip; tracked as tech debt
    to migrate to temp-id once the counter is decoupled from the UI state.

  Updates use `*Updated { entity }` (server echoes the row); deletes use
  `DeleteConfirmed` (model already mutated optimistically).

## Native iOS Shell (SwiftUI + Crux)

> The native SwiftUI app is the only shell — see
> [`specs/native-ios.md`](specs/native-ios.md). App-first, local-first. These
> rules are non-negotiable when touching the native shell.

**The shell is a dumb pipe — it owns ZERO domain logic.** It sends `Event`s,
fulfils `Effect`s (HTTP via `URLSession`, persistence via GRDB, etc.), and
renders the `ViewModel`. No business rules, no validation, no domain decisions
in Swift. If you're tempted to write logic in Swift, the logic belongs in
`intrada-core` as an `Event`/`Command`.

- **Bindings are a build precondition, never source.** The Swift `Event` /
  `Effect` / `ViewModel` types and serializers are **generated** (facet-generate
  + UniFFI). **Never hand-edit generated bindings.** If a generated type is
  wrong or missing, **fix the Rust type in `intrada-core` and regenerate** —
  the typegen run is part of the build, not an optional step. A diff that edits
  generated Swift is a blocker.
- **`@Observable`, not `ObservableObject`.** The core-wrapping store is an
  `@Observable @MainActor` object exposing the `ViewModel` and an `update(Event)`
  method. Effect handlers run off the main actor, then hop back to resolve.
- **`try!` is banned like `unwrap()`.** No `try!` / force-unwraps / `as!`
  without a written justification (same bar as Rust `unwrap()`). FFI calls and
  bincode (de)serialization return real errors — handle them.
- **Persistence is a core `Effect` driven by `Command`, not Swift logic.** GRDB
  owns the SQLite tables and executes typed query/mutation effects; the core
  decides what to read/write and runs LWW reconciliation. `crux_kv` is for small
  singletons only, never relational data.
- **Quality is per-screen, not deferred.** Every screen ships with a
  swift-snapshot-test, VoiceOver labels + Dynamic Type, and an iPad `SplitView`
  built *with* the screen. Sentry is wired from the first build.
- **Spacing & radius are tokens, not literals** — same discipline as colour
  (`IntradaColor`) and type (`IntradaFont`). Padding / inset / list-gap values
  come from `IntradaSpacing` (`controlGap` 8, `cardCompact` 12, `row` 14,
  `card` 16, `section` 24, `stage` 40 — the first four mirror the web `p-card`
  scale, one step serves several roles; `stage` is the drill-screen rhythm that
  separates five facts into five glances) and
  corner radii from `IntradaRadius` (`card` 12); all in `Theme.swift`. Don't
  hard-code `.padding(16)` / `cornerRadius: 12` etc. — a raw value is how two
  screens silently drift. Genuine one-offs (a fixed component height, a 2pt
  baseline nudge) stay literal; don't force those into the scale.
- **Build hazard:** UniFFI-generated Swift fails under Xcode 26 / Swift 6.2
  `MainActor`-default isolation ([uniffi-rs#2818]). Keep the generated package
  non-MainActor-defaulted (build recipe handles it); don't "fix" it by editing
  generated code.

[uniffi-rs#2818]: https://github.com/mozilla/uniffi-rs/issues/2818

### Principles (from the 2026-06 review)

Hard-won lessons from the first full review of the native app. Treat them like
the non-negotiables above.

- **Surface, don't swallow — at every layer.** A core error state with no UI
  surface is the silent-no-op bug (#846) one level up. Every `ViewModel.error`
  must have a UI surface, and optimistic UI must reconcile with the core's
  confirmed outcome — never fire a success haptic or dismiss a sheet before the
  core confirms (re-read `viewModel.error` after `store.send`; see
  `LibraryAddScreen.add`).
- **A stated invariant must be an enforced invariant.** Back each offline-first
  invariant with a test *and* a CI gate. Prose plus an opt-in local hook is
  effectively off — assume the hook isn't installed.
- **Sync-boundary discipline now, not at Phase D.** Both writers (shell + core)
  must agree on the timestamp format, and reconciliation/merge policy lives in
  the core — even before the sync engine exists. Don't encode merge rules in
  shell SQL.
- **Consolidate before you template.** Extract a shared primitive/form the
  moment a *second* screen would copy it. The Library screens are the template
  the other pillars (Practice/Track) will clone — duplication here multiplies.
- **Bridge-crossing types need a real round-trip test as a build precondition.**
  Extend the Rust `assert_round_trips` helper to every write payload *before* it
  is wired to a screen — a stub-bridge test can't catch a bincode-wire break
  (#846).

### Snapshot test hygiene

The `swift-snapshot-test` references (`ios/IntradaTests/__Snapshots__/**/*.png`)
are PNG files committed to git and **re-recorded on every intentional UI change** —
binaries don't delta-compress, so each re-record adds a *full* copy to history
forever. Left unmanaged this compounds (a single theme/token sweep re-records
the whole suite at once). On the free offline tier this is the only quality
gate for the UI, so we keep the suite but keep it lean:

- **One device + scale, deterministic host.** Pin `.iPhone13` + `displayScale`,
  force light mode at the controller, use the stub bridge (already done). Do
  **not** multiply references by device/theme/size-class variants — snapshot a
  variant only when it can independently regress.
- **Snapshot load-bearing states, not the cross-product.** Prefer
  component-level (`sizeThatFits`) or structural/text snapshots where the
  assertion isn't pixel-perfect (e.g. "pills reflow, don't wrap").
- **Optimize before committing.** After (re)recording, run
  `just ios-snapshots-optimize` — losslessly drops Xcode's redundant all-opaque
  alpha channel (~75% smaller; pixels + sRGB preserved, so the comparison still
  passes). CI's **Snapshot Hygiene** job enforces a per-file size ceiling and
  fails on un-optimized references.
- **No orphans.** Delete a test → delete its PNG. The Snapshot Hygiene job
  fails any reference with no matching `func test…` (renamed/removed tests
  otherwise leave dead images in history). Run it locally with
  `just ios-snapshots-check`.

### Offline-first invariants (non-negotiable)

The native app is **offline-first**: on-device SQLite is the source of truth,
the app works with no network and no account, and sync is a future paid tier
(see [`specs/native-ios.md`](specs/native-ios.md)). These invariants protect
that as the app grows — break one and the app silently stops being offline.

1. **No network on the local-first path.** A local-first feature must work in
   airplane mode. New reads/writes go through the persistence `Effect`, never
   HTTP. *(Test-enforced: local-first launch + mutations assert zero `Http`
   effects.)*
2. **Every persisted entity is sync-ready from day one** — carries `updated_at`
   + a soft-delete `deleted_at` tombstone; **no hard deletes**. So the deferred
   sync engine never needs a migration. *(Test-enforced for the schema.)*
3. **Client-owned ids.** New entities mint their ulid locally as the canonical
   id — no server-assigned-id round-trip (no temp-id dance) in local-first.
4. **Reconciliation lives in the core**, not the Swift shell. Sync / LWW / merge
   logic is Rust (shareable to Android); the shell only executes typed storage
   ops. (The dumb-pipe rule, applied to persistence.)
5. **A failed local write is never a silent success.** Storage ops resolve a
   real failure output (`PersistenceOutput::Failed`) and the core surfaces it —
   never fake an `Ack` (#816).
6. **Existing dual-mode handlers stay intact; new code is local-first only.**
   Legacy domain handlers still branch on `local_first` (the online branches
   predate the web shell's removal) — when touching one, keep both branches
   passing, since core tests still exercise the online path. New engine/domain
   code targets local-first only: the build-and-test-both-modes requirement is
   retired for new work (see `docs/rebuild-review.md` §3).
7. **No account gate on core functionality.** Only sync (the paid tier) may
   require auth. The free app works fully signed-out.
8. **Relational data in the GRDB store; only small singletons in `crux_kv`**
   (settings, `session-in-progress` crash-recovery).

**PR checklist — any change touching persistence, sync, or a new domain entity:**

- [ ] New reads/writes go through the persistence `Effect`, not HTTP (invariant 1)
- [ ] New persisted table/columns have `updated_at` + `deleted_at`; no hard delete (2)
- [ ] New entities use a client-minted ulid as the canonical id (3)
- [ ] Any merge/reconciliation logic is in the core, not the shell (4)
- [ ] Write handlers branch on `local_first` (or use `save_or_put`) and a local
      failure resolves `Failed`, not `Ack` (5)
- [ ] Changes to an existing dual-mode handler keep both `local_first` branches
      tested; new code is local-first only (6)
- [ ] Data-model change: new migration appended (never edits a shipped one),
      additive where possible; core type + migration + codec updated together;
      ships an upgrade-path test (see Local data migrations)

### Local data migrations

The on-device SQLite schema (GRDB, in `LibraryStore`) evolves via
`DatabaseMigrator`. Treat these with **more** care than the server migrations in
`intrada-api/src/migrations.rs`: **on the free offline tier the device is the
only copy of the user's data** — no server backup, no DBA. A destructive or
buggy migration that ships is **unrecoverable** data loss for that user, and you
can't un-ship it.

- **Append-only, forward-only, ordered.** Add a new `registerMigration("vN_…")`;
  **never edit or delete a shipped migration** — it has already run on real
  devices. GRDB applies missing migrations in order and users skip versions, so
  the chain must run cleanly from *any* past version.
- **Additive by default.** New nullable columns / new tables are safe.
  Drop/rename/retype is dangerous — use a copy-table migration, and prefer to
  **defer destructive changes until sync/backup exists** (a recovery path).
- **Evolve the core type + schema + codec together.** A new `Item` field needs,
  in one change: the Rust field (`Option` / `#[serde(default)]`), a migration
  adding the column with a default for existing rows, and the row↔`Item` codec
  updated. (Extends "compile the whole workspace for shared core types".)
- **Test the upgrade path, not just the end state.** Every migration ships with
  a test that a DB *populated at the previous version* migrates with data
  intact — not only that it runs on an empty DB.

## Authentication

- **iOS**: Google OAuth runs in Safari via `ASWebAuthenticationSession` (Google
  blocks OAuth in an in-app browser). The resulting Clerk JWT is exchanged for
  a long-lived PAT via `POST /api/auth/ios/exchange`. All subsequent API calls
  use the PAT.

Common:
- JWT RS256 validated against JWKS. PATs validated via SHA-256 hash lookup.
- All DB queries scoped by `user_id` (from JWT `sub` or PAT owner).
- When `CLERK_ISSUER_URL` unset: auth disabled (local dev only).
- Key files: `intrada-api/src/auth.rs`, `intrada-api/src/routes/auth_ios.rs`,
  `intrada-api/src/clerk.rs`.

## Environment Variables

### API (intrada-api)
`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN` (required), `CLERK_ISSUER_URL` (required
in prod), `ALLOWED_ORIGIN` (see SETUP.md §2), `PORT` (default 3001)

### R2 photo storage (optional — API starts without it, photo endpoints return 500)
`R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET_NAME`,
`R2_PUBLIC_URL`. See `SETUP.md` §3 for provisioning steps.

### Native iOS build (compile-time)
`CLERK_PUBLISHABLE_KEY`, `INTRADA_API_URL` (default `https://intrada-api.fly.dev`)

### Native iOS (optional)
`SENTRY_DSN_NATIVE` — put in `.env` to capture crash/error events from local dev
builds, tagged `environment=development`. `xcodegen` writes it to the
`SENTRY_DSN` build setting; the target's partial `Info.plist`
(`ios/Intrada/Info.plist`) carries `SENTRY_DSN = $(SENTRY_DSN)` so the value
lands in the built plist — a custom key **can't** ride `INFOPLIST_KEY_*`, which
`GENERATE_INFOPLIST_FILE` only honours for Apple-recognised keys (that gap
meant Sentry silently never started). The justfile's `set dotenv-load` feeds
the var in. **Unset in CI**, so test/smoke runs send nothing. The app only
starts Sentry on a real `https://` DSN, so an empty or unexpanded value is a
safe no-op.

## Design System Rules

The native app uses a "Paper & Score" light theme: warm paper backgrounds,
serif titles (Source Serif 4), sans body text (Inter). All colour/type/motion
tokens live in `Theme.swift` (`ios/Intrada/DesignSystem/Theme.swift`); the
shareable reference export is
[`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html).

**Interaction & design principles** (the *why* behind the visual rules, plus how
we think about friction, simplicity, and clutter) live in
`docs/design-principles.md`. **Consult it before making any UI/UX design
decision** — new surface, layout, flow, or interaction. It is the source of
truth for *how the app should feel*: the "spend friction deliberately" model
(remove admin/setup friction, keep intention-before and reflection-after
friction), one-primary-action-per-screen, content-over-chrome, progressive
disclosure, and reversible-by-default. It also carries a dated decisions log
(T1–T6) recording the reasoning behind each ruling — when a new decision is
made, append to that log rather than deciding silently.

### Hierarchy: Tokens → Modifiers → Components → Screens

1. **Tokens first**: Every colour, font, spacing, and radius value traces to a
   named token in `Theme.swift` (`IntradaColor`, `IntradaFont`, `IntradaSpacing`,
   `IntradaRadius`). Never hard-code a hex, a raw `.padding(16)`, or
   `cornerRadius: 12`.
2. **Reuse before creating**: Check `ios/Intrada/DesignSystem/` and
   `ios/Intrada/Views/Components/` before building new markup.
3. **Known primitives to reach for**: `TagChip`, `TypeBadge`, `ScoreRing`,
   `BottomSheet`, `SegmentedPills`, `CardSurface`/`CardShadow`, `GlobalBanner`,
   `FormErrorBanner`, `PlaceholderContent` (empty state), `ScreenScaffold`
   (shared screen shell), `SectionHeader`, `HairlineDivider`,
   `SegmentedProgress` (session/block position).
   **Coach primitives** (`ios/Intrada/DesignSystem/Coach/`, canonical in the
   design system under *Components · Coach primitives*): `GateDots`,
   `RepVerdict`, `TapVerdict`, `OrientationStrip`, `BeatPosition`,
   `StuckTarget`, `CoachNote`. They size off `CoachScale` (phone vs
   iPad-on-a-stand) taken from the environment, not per-call sizes — the whole
   drill loop grows together. The deferred coach surfaces (B3/B4, the
   session-end narrative, the pipeline view) are assembled from these; reach for
   them before drawing a new one.
4. **Every top-level screen** is built from `ScreenScaffold`
   (`ios/Intrada/DesignSystem/ScreenScaffold.swift`) so navigation chrome,
   safe areas, and background stay consistent.

### Animated reveals need an opaque backing

Anything that **slides or fades in/out over other content** — a search bar, an
expanding row, a banner, a sheet-like panel — must paint an **opaque background
token** (`paperTop` / `cardFill`, never `clear`) so it cleanly covers what's
behind it during the transition. A transition over a transparent background
**ghosts**: you see both components overlap mid-animation (e.g. the Library
search bar drawing on top of the filter pills). Two parts:

- The **moving** view gets an opaque background so it hides whatever it travels over.
- When it should emerge from *behind* sibling chrome (pills, a toolbar), that
  chrome must also be opaque **and** sit on top (`.zIndex(1)` in SwiftUI) —
  otherwise the transparent chrome can't occlude it.

It's the background that hides the motion, not the transition. Don't ship a
reveal animation without checking what shows through behind it.

### Don't deviate from the system unless you're explicitly redesigning

Hand-rolled views that duplicate an existing primitive are the #1 source of
visual drift in this codebase. Before writing UI code:

- **Grep first.** If you're about to hand-roll a chip, badge, sheet, or card
  that already has a component under `ios/Intrada/DesignSystem/` or
  `ios/Intrada/Views/Components/` — stop and use the existing one instead.
- **Extend, don't clone.** If a primitive *almost* fits, add a parameter (the
  way `SegmentedPills` and `LibraryItemCard` already take variant params) to
  the shared component. Don't ship a parallel one-off.
- **Typography**: use the `IntradaFont` tokens (e.g. `.pageTitle`,
  `.cardTitle`, `.sectionTitle`, `.fieldLabel`, and for the coach loop
  `.drillTitle` / `.verdict` / `.ambient`), never a raw `.font(.system(...))`.
- **Spacing**: use `IntradaSpacing` tokens (`controlGap`, `cardCompact`, `row`,
  `card`), never a literal `.padding(16)`.

Deviation is only acceptable when **explicitly redesigning** a surface — and
that should be a deliberate, flagged conversation (Claude Design first, then Plan
mode), not an accident inside an unrelated feature PR. A redesign produces
*updated tokens / primitives* in `Theme.swift`, not a hand-rolled clone in a
single view.

### iOS native-feel rules

- **Haptics**: use `UIImpactFeedbackGenerator`/`UISelectionFeedbackGenerator`
  via the `Store+Feedback` helpers — `selection` for tabs, `light` for taps,
  `success` for saves (only after the core confirms — see "Surface, don't
  swallow" below), `warning` for destructive confirms.
- **iPad**: list→detail screens use `LibrarySplitView` (adaptive
  sidebar + detail pane at regular width class). Build it before the view,
  not as a retrofit.
- **Safe areas**: respect them by default (`ScreenScaffold` handles this for
  every top-level screen); don't fight SwiftUI's layout system with manual
  insets unless a genuine edge-to-edge treatment calls for it.
- **Animations**: use the tokens in `Motion.swift`, not ad hoc
  `.animation(.spring(...))` literals.

## Code Style

- Rust stable, 2021 edition. `cargo fmt` + `cargo clippy -- -D warnings` must pass.
- No `unwrap()` without justification.
- Prefer well-established libraries over custom implementations.

### Nothing unread stays in the tree

**Code with no reader gets deleted, not parked.** Not `#[allow(dead_code)]`, not
"inert until the feature returns", not a `pub` export nobody calls — those go
stale silently, they carry weight into the app, and they mislead the next reader
into thinking something is load-bearing. `git` is the parking space: delete it,
say in the PR (and in whatever doc described it) which PR to recover it from, and
keep the *findings* rather than the code that produced them. Prose about what a
spike established stays valuable; the spike's unread code does not.

This applies to deliberately-deferred work too — the MIDI capture spike and its
Rust segmentation module were deleted on exactly this rule (#1176) even though
the scoring path is expected back, with
[`docs/segmentation-findings.md`](docs/segmentation-findings.md) left as the
record. A "we'll need it later" exemption is how a codebase accumulates a
museum. Deferring a feature means deferring its code to history.

Distinguish this from a stub that a *test* reads, or an API a shell genuinely
calls: those have readers. The test is "who reads this today?", not "might
someone read this eventually?".

### Comments

Default to **no comments**. Self-explanatory code with well-named identifiers
beats commented code. A reader who knows the language and framework should be
able to answer "what does this do?" from the code alone.

A comment is justified ONLY if it falls into one of these **three buckets**.
Everything else gets deleted.

1. **Section headers in a large file** — single-line dividers that separate
   concerns, like `// ── Validation ──`. Never more than one line. (A one-line
   cross-file pointer the reader would otherwise miss counts here too.)
2. **Unusual / unreasonable things that need explaining** — a non-obvious WHY:
   a hidden constraint, subtle invariant, a workaround for a specific bug, or a
   framework quirk that would surprise a reader. Cite the reason concretely: an
   issue number, an incident, a doc link, a `BUG:` tag. Vague WHY is no better
   than restating WHAT.
3. **Hacky / tactical code that needs rework** — flag it so it's visible, but
   tie it to a tracked issue: `// HACK(#N): …` or `// FIXME(#N): …`. A bare
   `// TODO come back to this` with no issue is not acceptable — open the issue
   and cite it.

`///` doc comments get the **same** treatment as `//`: a `///` narrating a
self-evident private item or single-purpose component is noise — delete it.

Do **not** write a comment that:

- Restates WHAT the code does (`// Filter by status` above `.filter(|g| g.status == tab)`)
- Narrates self-evident styling/structure (e.g. a 4-line `///` explaining a
  transparent nav bar + tint + serif title the code already shows)
- References the current task / PR (`// Added for #719`) — rots, belongs in
  the PR description
- Apologises or hedges without a tracked issue (`// quick fix`,
  `// TODO come back to this`)
- Notes that a function "Mirrors X" when the shapes already make it obvious

Two-line cap as a smell test: if a comment is more than two lines, ask "can
this be a function name? a type? a CLAUDE.md entry?". Usually yes.

The `pre-push` hook (under `.githooks/`) flags branches that push too many
comment lines relative to code. Bypass for genuinely-justified cases
(an incident write-up, a copy-pasted upstream notice) with
`SKIP_COMMENT_CHECK=1 git push`.

When invoking the `superpowers:code-reviewer` agent, include "comment-policy
violations are Blockers, not Nits" in the prompt so the review treats drift
as a merge-blocker.

## Testing

**Default: ship tests with new code.** New API endpoints, DB functions, and
non-trivial pure logic must include tests. The existing suite
(`crates/intrada-api/tests/`) uses real SQLite via `common::setup_test_app()`
— no mocks needed for DB-backed tests.

What to test:
- API endpoints: at minimum auth rejection paths; happy path when reachable
  via the test harness (auth-disabled mode gives a fake user).
- DB write functions: correct rows affected, idempotency, cross-user isolation.
- Pure functions: edge cases, None/empty inputs.

**iOS test framework policy**: new unit/snapshot test files use **Swift
Testing** (`import Testing`; swift-snapshot-testing supports it). Migrate
existing XCTest files only when already touching them, no wholesale rewrite.
XCUITest (`IntradaUITests`) stays on XCTest: UI tests have no Swift Testing
equivalent.

When skipping tests, say so explicitly in the PR description with the reason
(e.g. "requires real HTTP to an external API and we don't have a mock server").
"All 157 tests pass" is not coverage — those are existing tests, not tests
for new code.

**Coverage** (Codecov, config in `codecov.yml`):
PRs get an automated patch-coverage comment (70% target, informational
— not blocking). How to use it depends on tier:

- **Tier 1**: No coverage justification needed (typos, config, dep bumps).
- **Tier 2+**: PR description must include a **Coverage** line noting
  expected gaps *before* CI finishes (e.g. "Coverage: diagnostic logging
  not reachable from unit tests" or "Coverage: full — new endpoint has
  happy-path + auth-rejection tests"). When CI completes, check the
  Codecov comment against your expectation. If patch coverage is below
  70% for reasons you didn't anticipate, either push a follow-up commit
  with tests or add an explanatory PR comment.

Ignored paths (no coverage expected): `ios/` (native Swift shell, see the
`codecov.yml` note), `migrations.rs` (SQL strings).

## Project-specific gotchas

Bear-traps that have caught us at least once. Skim before you start; the cost
of a recheck is a few seconds, the cost of one of these landing in main is a
follow-up PR.

### JSON-only serde attrs break the Crux bincode FFI bridge

The native iOS shell exchanges `Event` / `Effect` / `ViewModel` with the core as
**positional bincode** (a non-self-describing format). serde attributes that
only make sense for a self-describing format (JSON) silently corrupt that wire:
the Swift side serializes every field/level by structure, but a JSON-oriented
deserializer reads a different shape, **misaligns the byte stream, and the whole
event fails to decode** — and `Store.send` swallows the bridge error via
`guarded`, so the symptom is a silent no-op (e.g. "editing doesn't save", #846),
not a crash.

The specific offender we hit: `#[serde(deserialize_with = "double_option")]` on
`UpdateItem`'s three-state `Option<Option<T>>` fields. `double_option` reads a
single option level (right for JSON, where a present key is one `Option<T>` and
`null` = clear); bincode needs both levels. Fix: make such helpers **format-aware**
via `Deserializer::is_human_readable()` (JSON branch vs bincode branch) so the
same type round-trips on both wires.

Rules of thumb for any type that crosses the FFI bridge (`Event`, `Effect`,
`ViewModel`, and everything they contain):

- Be wary of `deserialize_with` / `serialize_with`, and of `skip_serializing_if`
  combined with non-trailing fields — anything that assumes "absent" vs "present"
  semantics. bincode has no "absent". If you need format-specific behaviour,
  branch on `is_human_readable()`.
- **Stub-bridge tests can't catch this.** Cover bridge-crossing types with a
  *real*-bridge round-trip (`LiveBridge` in `StoreEffectLoopTests`) that drives
  the actual Swift↔Rust bincode (de)serialization — see
  `testRealBridgeEditAppliesToViewModel`.

### `option_env!` needs `cargo:rerun-if-env-changed`

If a build script (or `option_env!` site indirectly via macro expansion)
reads an env var, pair it with `println!("cargo:rerun-if-env-changed=NAME")`
in `build.rs`. Without it, cargo caches the macro expansion across builds and
your "I changed the env var" rebuild silently uses stale values. We've hit this
on `CLERK_PUBLISHABLE_KEY` and `INTRADA_API_URL`.

## Workflow

Match ceremony to scope. Default to less. Escalate only when work demands it.

### Tier 1 — Just do it
Bug fixes, copy/text changes, style tweaks, renames, lint/clippy fixes,
single-file refactors, dependency bumps, doc updates.

No Plan mode, no spec doc. Read enough to confirm the change, make it,
verify, ship.

### Tier 2 — Plan mode (default for feature work)
New component/view following existing patterns, new API endpoint following
established conventions, adding a field to an existing model, new screen
in existing navigation.

For UI work: Claude Design first (see Design Workflow below), then
Plan mode, then implement. For non-UI work: Plan mode, then implement.
No spec doc.

### Tier 3 — Lightweight spec (rare; architectural only)
Net-new top-level features, Crux core / FFI bridge changes, auth or DB
schema changes, multi-week work spanning core + web + iOS.

Write ONE markdown doc in `specs/<feature>.md` (~100-200 lines: problem,
approach, key decisions, open questions). Then Claude Design for UI work, then
Plan mode, then implement.

**Spec doc rides with the first implementation phase, not its own PR.**
The spec is the first commit on the Phase A branch; Phase A scaffold is
the rest. The PR title/body reflects both. Reviewers sanity-check the
spec against working code rather than abstract architecture diagrams.
Phases B/C/D still ship as their own PRs — only the spec ↔ Phase A
boundary collapses.

Do not run `/speckit-*` slash commands. Historical SpecKit folders under
`specs/` are reference only.

### Domain sensitivity override
Changes to auth, the FFI bridge contract (Event/Effect/ViewModel), DB schema,
or migrations go up at least one tier regardless of file count or apparent size.

### Decision rule
If unsure between tiers, go one tier lighter. Drift up if scope expands.

### Examples

| Task | Tier | Why |
|------|------|-----|
| Fix typo in a label | 1 | Trivial copy change |
| Bump a dependency with no API change | 1 | Dep bump |
| New "Recently practiced" view following existing list patterns | 2 | New view, established patterns |
| Refactor `intrada-core/src/practice/session.rs` (no FFI change) | 2 | Single file, non-trivial domain logic |
| Tweak retry backoff in `auth.rs` | 2 | Sensitivity override from Tier 1 |
| Add `notes` field to a piece (touches FFI + DB) | 3 | Override: FFI + schema |
| New auth provider | 3 | Auth + multi-crate |
| Migrate persistence layer | 3 | Architectural |

### Optional skills (Superpowers, opt-in only)

The [Superpowers](https://github.com/obra/superpowers) plugin's full methodology
(`brainstorming`, `writing-plans`, `systematic-debugging`, etc.) conflicts with
the tier system above and is **not** enabled by default. Three of its skills
are useful **when invoked deliberately**, telling the agent to skip the rest:

- `test-driven-development` — non-UI Tier 2 / all Tier 3 work (skip for
  visual/gesture work verified on-device).
- `requesting-code-review` — before any Tier 3 PR, and Tier 2 PRs touching
  auth/DB/FFI.
- `using-git-worktrees` — when two or more PR branches are in flight at once.

If unsure whether a skill applies, default to the tier system instead.

### Lessons from recent skill use

Discipline tightening after #719/#724, on top of the guidance above:

- **TDD is the default for `intrada-core` changes** (`domain/*.rs`,
  `validation.rs`, `http.rs`, `model.rs`): invoke
  `superpowers:test-driven-development` and write the failing test first. The
  #719 delete-404 bug shipped because the test was retrofit to pass after the
  fix rather than written to constrain behaviour.
- **`requesting-code-review` is the standard channel for Tier 2+ PRs** — load
  the skill rather than hand-rolling a prompt; run
  `superpowers:receiving-code-review` on the findings before acting on them.
- **UI verification means actually driving the preview**, not just claiming
  "all green" when that means cargo test green (see "Doing tasks" above).

### Always
1. Find the roadmap item in `docs/roadmap.md`. No item = discuss first.
2. Check priority on the [project board](https://github.com/users/jonyardley/projects/2).
3. Never push to main. Always a feature branch + PR.
4. **Open/update any non-trivial PR through the `ship` skill — don't
   `gh pr create`/`git push` feature work directly.** `ship` runs the pre-push
   gates *and* the self-review in one funnel, so the review can't be skipped in
   a fast build→push cadence (which is exactly how it gets skipped when left to
   "remember to review"). `ship` uses the `superpowers:code-reviewer` subagent;
   post its summary as a `gh pr comment` (the reviewer doesn't see in-conversation
   subagent output), apply blockers / important findings inline, and defer the
   rest as tracked issues per (6). Tier 1 trivia (typos, dep bumps, single-line
   config tweaks) may skip `ship`'s review step but still run the gates.
5. **Check Codecov after CI** (Tier 2+ only). Compare the patch-coverage
   comment against the **Coverage** line in the PR description. If there
   are unexpected gaps, push tests or explain in a PR comment. Don't
   declare the PR ready until this is done.
6. **Open a tracked issue for every deferred / out-of-scope item**, with
   appropriate labels (`horizon:now|next|later`, kind: `ux` / `architecture`
   / `bug` / `accessibility` / `ios` / `pillar:*`). PR descriptions are not
   tracking — they get auto-collapsed after merge. Open the issues *before*
   posting the self-review comment, not after: phrasings like "will open a
   follow-up if it bites" are not acceptable. Every self-review comment
   must end with an explicit `Deferred items tracked: #N, #M` line (or
   `none — all flagged items addressed inline`) so the question is always
   answered. Silent omission is the failure mode.

### After completing work
1. Update `docs/status.md` (in flight / landed / next) in the same PR; update
   `docs/roadmap.md` if a phase or direction changed; close the GitHub issue.
2. Update this file if architecture/patterns changed.
3. Update the Claude Design system (`design/intrada-design-system.dc.html`) if UI
   diverged from design; re-export the shareable `.html`.

## Design Workflow

Design happens in **Claude Design**; full process in
[`docs/design-workflow.md`](docs/design-workflow.md). The living reference is
[`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html)
(+ `support.js`) — the "Paper & Score" system, **derived from `Theme.swift`**
(`ios/Intrada/DesignSystem/Theme.swift`), which stays the canonical token source.
Required for new views and significant UI changes: mock the screen against the
existing kit first, reuse tokens/components, and if something new is needed update
`Theme.swift` and the design reference together. Colours/spacing/radius reference
the `Intrada*` tokens, never raw hex.

(Pencil — `design/intrada.pen` — is retired. `design/light-mode-exploration.md`
remains as provenance.)

## Parallel work streams (agentic sessions)

Rules for running more than one Claude Code session against this repo at once.
Evidence base: coupling analysis of the last 400 commits (2026-08).

### Conventions

- British English in all UI copy, comments, commit messages and PR bodies.
- No em dashes and no double dashes in prose: docs, commits, comments, PR bodies.

### Stream rules

- **Exactly one core+iOS vertical stream at a time.** 31% of core commits also
  touch `ios/`; two concurrent vertical features will collide.
- A **second stream** may run only in the decoupled set: `crates/intrada-api`,
  `docs/`, `specs/`, `design/`, `content/`, or CI/tooling (`justfile`,
  `.github/workflows/`). An API task that needs a new domain field is a core
  change: it joins the vertical stream, it does not run beside it.
- **Serialisation points.** If your task and another live branch both touch one
  of these files, serialise rather than parallelise:
  `crates/intrada-core/src/app.rs`, `crates/intrada-core/src/domain/session.rs`,
  `ios/IntradaTests/ScreenSnapshotTests.swift`,
  `ios/Intrada/DesignSystem/PreviewSupport.swift`, `ios/project.yml`, and
  `Cargo.lock` (never pair anything with a dependency bump).
- One git worktree per stream, branched from fresh `origin/main`. Follow the
  shared-simulator rule under Commands. Close the second session when its task
  ships; do not keep it warm.

### Agent teams (in-session teammates)

An agent team (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`, terminal sessions only)
counts as **one** stream: the whole team shares a single checkout and one
feature branch, so a core+iOS team satisfies the one-vertical-stream rule by
itself. Never run a second vertical session beside it. Start one with
`/team-vertical <issues>` (`.claude/commands/team-vertical.md`).

- **Shared checkout, no file locking.** Teams lock task claiming, not file
  edits. Every file gets exactly one writing teammate, fixed in the brief;
  the serialisation-point files above are single-owner by definition.
- **Contract before code.** The core teammate publishes the
  Event/Effect/ViewModel contract for the slice before either side wires it.
  A bridge-type change is messaged to the team, never landed silently.
- **Per-teammate gates.** Each teammate runs its gates (`just check` for
  core; `just ios-fmt-check` + `just ios-test` for `ios/`) before marking a
  task done.
- **One PR, via `ship`, human merges** — the definition of done below applies
  to the team as a whole, not per teammate.

### Definition of done (every stream, before requesting review)

- [ ] `just check` green locally; `just ios-fmt-check` too if `ios/` touched
- [ ] Tests shipped with the new code (see Testing)
- [ ] PR opened via the `ship` skill; self-review comment posted
- [ ] Codecov compared against the PR's Coverage line (Tier 2+)
- [ ] `docs/status.md` updated (roadmap too if a phase/direction changed);
      deferred items tracked as issues
- [ ] A human reviews and merges. Agents never merge.

## Known Tech Debt

- `Set` creates still bump `set_saves_committed` + refetch instead of using
  the temp-id mutate-response pattern (see "Mutate response" under Other
  patterns). The counter drives the save-form's optimistic→confirmed flip;
  reworking it needs to keep that affordance.

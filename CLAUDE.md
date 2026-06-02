# intrada Development Guidelines

> Last reviewed: 2026-06-01.

> ## ⚠️ CURRENT FOCUS: NATIVE iOS ONLY — web is paused
>
> As of 2026-06, the **only** platform under active development is the
> **native SwiftUI iOS app** (on the Crux core — see
> [`specs/native-ios.md`](specs/native-ios.md)). **Do NOT build new features
> in the Leptos web shell (`crates/intrada-web`) or the Tauri shell
> (`crates/intrada-mobile`) — both are PAUSED.** New UI work lands in the
> native iOS app. Pencil designs are still the source of truth for look & feel,
> but implementations target SwiftUI, not Leptos. If a request seems to imply
> web/Leptos work, confirm the platform before writing code.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics. Organised around three pillars:
**Plan** (library, routines), **Practice** (focus mode, timers, scoring),
**Track** (analytics, insights).

**Platform**: Active development is the **native SwiftUI iOS app** (on the Crux
core). The **Leptos shell (`crates/intrada-web`)** historically shipped as both
the web app (Cloudflare Workers) and the Tauri 2 iOS WKWebView host
(`crates/intrada-mobile`) — **both are now paused** (see the banner above). The
web shell still builds and its tests still run in CI, but it is not receiving
new features; treat it as maintenance-only until web is explicitly un-paused.

## Project Structure

```text
crates/
  intrada-core/          # Pure Crux core — business logic, no I/O
  intrada-web/           # Leptos 0.8 CSR + WASM — UI shell (web + iOS)
  intrada-api/           # REST API — Axum 0.8 + Turso (libsql)
  intrada-mobile/        # Tauri 2 iOS host — wraps intrada-web in WKWebView
    src-tauri/           #   Rust host, tauri.conf.json, Swift plugins
design/                  # Pencil design system (intrada.pen)
docs/                    # Product roadmap (single source of truth)
e2e/                     # Playwright E2E tests
specs/                   # Spec docs for major features (Tier 3 only — see Workflow)
```

## Tech Stack

- **Rust** stable (1.90.0 CI; MSRV 1.75+, intrada-api 1.78+)
- **Core**: crux_core 0.18.0, serde, ulid, chrono, thiserror
- **API**: axum 0.8, tokio, libsql 0.9 (Turso), tower-http (CORS), jsonwebtoken 10
- **Web + iOS UI**: leptos 0.8 (CSR), Tailwind CSS v4, trunk 0.21
- **iOS host**: Tauri 2, iOS 17.0+, WKWebView, tauri-plugin-haptics, tauri-plugin-deep-link
- **Auth**: Clerk (Google OAuth), JWT RS256 against JWKS
- **DB**: Turso (managed libsql/SQLite) via HTTP
- **E2E**: Playwright
- **CI/CD**: GitHub Actions → Cloudflare Workers (web) + Fly.io (API)

## Commands

```bash
cargo fmt --check          # must pass before commit AND before push (CI runs both)
cargo test                 # all workspace tests
cargo clippy               # lint check — must pass before push
cargo test -p intrada-api  # API tests only
just ios                   # native app: regen bindings (if core changed) + open Xcode
just ios-run               # native app: build + launch on simulator + screenshot
just tauri-dev             # Tauri shell (on hold): iOS dev session (sim)
```

`just ios` / `just ios-run` auto-regenerate the Swift bindings only when
`intrada-core`/`intrada-ffi` changed (a `ios/generated/.gen-stamp` hash), so
they stay in sync without slowing pure-Swift edits. `just ios-gen` forces a
full regenerate. The Tauri `tauri-*` recipes are the on-hold WKWebView shell.

**Simulator build/snapshot/UI testing** — the `xcrun simctl` / `xcodebuild`
CLI workflow (screenshots via `xcrun simctl io <udid> screenshot`), the optional
XcodeBuildMCP server in `.mcp.json`, and host gotchas (e.g. quit Xcode before
`xcodebuild test` to avoid `Pseudo Terminal Setup Error`) — is documented in
[`docs/ios-testing.md`](docs/ios-testing.md).

**Demo data vs. real on-device data.** A plain launch (`just ios` → Cmd+R, or
any build with no launch args) runs **local-first**: the Library hydrates from
the on-device GRDB store, so items you add survive restarts. The 6 sample
pieces are **opt-in** via the `--seed-sample-data` launch arg — in Xcode:
Edit Scheme → Run → Arguments → Arguments Passed On Launch. `just ios-run`
passes it by default (`SEED=1`); use `SEED=0 just ios-run` to launch against
your real data. Seed mode (`Event::LoadSampleData`) replaces the model with
demo items and **skips store hydration**, so don't use it when testing
persistence — your saved rows are still on disk but won't be read back.

Run `cargo fmt --check` and `cargo clippy -- -D warnings` *locally before pushing* —
not just before committing. Pushing then watching CI fail wastes a full ~3-minute
roundtrip per agent or contributor; better to catch the formatting tab here.

Optional one-time hook install (catches the "pushed onto a merged-PR
branch and the commits orphaned" pitfall):
```bash
bash scripts/install-git-hooks.sh   # sets core.hooksPath = .githooks
```
The pre-push hook uses `gh` + `jq` to refuse pushes to a branch whose
most recent PR is already MERGED, with a hint to create a fresh branch
from `origin/main`. Bypass for legitimate edge cases:
`SKIP_PR_CHECK=1 git push`. Opt out: `git config --unset core.hooksPath`.

First-time iOS setup (run once after cloning or pulling this branch):
```bash
cargo install tauri-cli --version "^2" --locked   # Tauri CLI
brew install cocoapods                             # CocoaPods (required by Tauri iOS)
brew install xcodegen                              # xcodegen (required by Tauri iOS)
# Also requires: iOS Simulator runtime (Xcode → Settings → Platforms → iOS Simulator)
just tauri-init   # generates Xcode project + applies post-init patches
```

`just tauri-init` runs `cargo tauri ios init` then the two post-init Ruby scripts
(`fix-ios-build-config.rb`, `add-live-activity-target.rb`) that patch the
generated `project.yml`. Re-run it after any `cargo tauri ios init` regeneration.

If you're forking this repo, update `bundle.iOS.developmentTeam` in
`crates/intrada-mobile/src-tauri/tauri.conf.json` to your own Apple Team ID
(find it at developer.apple.com → Membership, or Xcode → Settings → Accounts).

`just tauri-dev` reads `INTRADA_API_URL` and `CLERK_PUBLISHABLE_KEY` from your
shell or a `.env` file at the repo root (the justfile uses `set dotenv-load`).
Without them set, the build will use defaults and Clerk auth won't work.

**Development security warning**: `just tauri-dev` binds the Trunk dev server to
`0.0.0.0:8080` so the iOS simulator can reach it via the host's LAN IP.
Anyone on your Wi-Fi network can reach it (and the proxied `/api/`) while it's
running. Don't run `tauri-dev` on public/untrusted Wi-Fi.

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User → Events → crux_core (Rust) → Effects (Http, KeyValue, Render) → Shell → I/O
```

1. **Core owns all logic.** HTTP requests built in core via `crux_http`. Core does
   all JSON serialization. Shells never understand domain types.
2. **Shells are dumb pipes.** Receive `HttpRequest` (URL, method, headers, bytes),
   return `HttpResponse`. No domain type imports.
3. **One UI codebase.** The Leptos shell talks to the core via `wasm-bindgen` —
   no typegen step. Domain types live in Rust and the WASM consumer uses them
   directly.

### State boundary

| State kind | Where it lives |
|------------|---------------|
| Domain data | Crux `Model` → `ViewModel` (single source of truth) |
| UI interaction | Leptos signals (web + iOS via Tauri) |
| Crash recovery | localStorage (`intrada:session-in-progress` only) |
| iOS auth credentials | localStorage (`__ios_pat`, `__ios_user_id`, `__ios_user_email`) |

Domain state flows through `Event` → `Model` → `ViewModel`. Never store domain
data in shell-local state. UI-only state stays in Leptos signals.

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

> Applies once Phase A lands. The native SwiftUI app is replacing the Tauri
> shell — see [`specs/native-ios.md`](specs/native-ios.md). App-first,
> local-first. These rules are non-negotiable when touching the native shell.

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
- **Build hazard:** UniFFI-generated Swift fails under Xcode 26 / Swift 6.2
  `MainActor`-default isolation ([uniffi-rs#2818]). Keep the generated package
  non-MainActor-defaulted (build recipe handles it); don't "fix" it by editing
  generated code.

[uniffi-rs#2818]: https://github.com/mozilla/uniffi-rs/issues/2818

### Snapshot test hygiene

The `swift-snapshot-test` references (`ios/IntradaTests/__Snapshots__/**/*.png`)
are PNGs committed to git and **re-recorded on every intentional UI change** —
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
- **Escalation path.** In-repo is fine now. When history gets heavy or the suite
  crosses ~50–100 references, move to Git LFS (note: adds a fetch to every CI
  run) or external hashed storage (S3 by SHA-256 / Screenshotbot). Don't reach
  for that machinery early.

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
6. **Both modes must keep working.** The shared core powers offline-iOS *and*
   online-web. Any change to a domain handler must preserve both `local_first`
   branches, and be tested both ways. (The likeliest regression: an iOS-driven
   core change that breaks web.)
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
- [ ] Domain-handler changes tested in **both** `local_first` and online modes (6)
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

Two auth paths, same API surface:

- **Web**: Clerk JS (cookies) → short-lived JWT on every request. Standard
  browser flow, Clerk handles Google OAuth natively.
- **iOS**: Google OAuth runs in Safari via `ASWebAuthenticationSession` (Google
  blocks OAuth in WKWebView). The resulting Clerk JWT is exchanged for a
  long-lived PAT via `POST /api/auth/ios/exchange`, stored in localStorage.
  All subsequent API calls use the PAT. No Clerk JS in the WKWebView.

Common:
- JWT RS256 validated against JWKS. PATs validated via SHA-256 hash lookup.
- All DB queries scoped by `user_id` (from JWT `sub` or PAT owner).
- When `CLERK_ISSUER_URL` unset: auth disabled (local dev only).
- Frontend retries once with fresh token on 401.
- Key files: `intrada-api/src/auth.rs`, `intrada-api/src/routes/auth_ios.rs`,
  `intrada-api/src/clerk.rs`, `intrada-web/index.html` (JS bridge),
  `intrada-web/static/ios-auth.html`

## Environment Variables

### API (intrada-api)
`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN` (required), `CLERK_ISSUER_URL` (required
in prod), `ALLOWED_ORIGIN` (default `http://localhost:8080`), `PORT` (default 3001)

### R2 photo storage (optional — API starts without it, photo endpoints return 500)
`R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET_NAME`,
`R2_PUBLIC_URL`. See `SETUP.md` §4 for provisioning steps.

### Web (compile-time)
`CLERK_PUBLISHABLE_KEY`, `INTRADA_API_URL` (default `https://intrada-api.fly.dev`)

## Design System Rules

The Leptos shell uses a dark-on-dark glassmorphism aesthetic. Web and iOS run
the same UI codebase; iOS-specific look-and-feel is layered on with platform
gating (see iOS native-feel rules below).

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

### Hierarchy: Tokens → Utilities → Components → Views

1. **Tokens first**: Every colour traces to a named token. Never use raw colours
   (`text-gray-400`). Source: `crates/intrada-web/style/input.css`.
2. **Reuse before creating**: Check existing components before building new
   markup. Source: `crates/intrada-web/src/components/`.
3. **Design catalogue**: New components get a showcase entry in
   `views/design_catalogue.rs`.
4. **Spacing tokens only**: `p-card` (16), `p-card-compact` (12),
   `p-card-comfortable` (24).

### Don't deviate from the system unless you're explicitly redesigning

Hand-rolled markup that duplicates an existing primitive is the #1 source of
visual drift in this codebase. Before writing UI code:

- **Grep first.** If you're about to write `inline-flex items-center rounded-md
  px-2 py-0.5 text-xs font-medium`, `text-2xl font-bold text-primary`,
  `rounded-lg bg-surface-secondary px-3 py-2`, or any other shape that already
  appears in the codebase — stop and use the existing utility/component instead.
- **Extend, don't clone.** If a primitive *almost* fits, add a variant prop
  (e.g. `compact: bool` on `SetlistEntryRow`) or a new utility class once, in
  the shared place. Don't ship a parallel one-off.
- **Known primitives to reach for**: `AccentRow`, `SetlistEntryRow` (with
  `compact` mode for review-sheet style rows), `BuilderItemRow`, `BottomSheet`
  (with `nav_action_label` for the iOS Mail-compose pattern), `Button`,
  `Card`, `DetailGroup`, `GroupedList`, `EmptyState`, `TypeBadge`,
  `InlineTypeIndicator`, `LibraryTypeTabs`, `TypeTabs`. Typography utilities:
  `page-title`, `card-title`, `section-title`, `field-label`, `form-label`.
  Spacing: `p-card`, `p-card-compact`, `p-card-comfortable`.

Deviation is only acceptable when **explicitly redesigning** a surface — and
that should be a deliberate, flagged conversation (Pencil first, then Plan
mode), not an accident inside an unrelated feature PR. A redesign produces
*updated tokens / primitives*, not a hand-rolled clone in a single view.

### iOS native-feel rules (Leptos shell in Tauri WKWebView)

These rules apply when building or modifying views/components that will run
inside the Tauri iOS shell. Gate iOS-only CSS with `[data-platform="ios"]`
(set on `<html>` from `lib.rs` `setup` hook on iOS) — never with raw media
queries alone.

- **CSS reset**: `-webkit-touch-callout: none`, `-webkit-tap-highlight-color: transparent`,
  `-webkit-user-select: none` on chrome (not text content), `touch-action: manipulation`
  on interactive elements, `overscroll-behavior: none` on root.
- **Inputs**: `font-size: 16px` minimum (prevents iOS zoom-on-focus).
- **Safe areas**: `env(safe-area-inset-*)` on tab bar, headers, sticky bars.
  `viewport-fit=cover` already set in `index.html`.
- **Scroll**: only inner regions scroll; `overscroll-behavior: contain` on scroll containers.
- **View Transitions**: wrap route changes in `document.startViewTransition()` via wasm-bindgen.
- **Haptics**: use `tauri-plugin-haptics` — `selection` for tabs, `light` for taps,
  `success` for saves, `warning` for destructive confirms.
- **iPad**: all list→detail screens use `<SplitView>` (CSS-grid sidebar + detail pane,
  viewport-driven visibility). Build it before the view, not as a retrofit.
- **Typography**: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", system-ui`.
- **Animations**: Motion One spring config `stiffness: 300, damping: 30` ≈ iOS default.

### Web-specific rules

- Never use raw Tailwind colour classes. Use token classes (`text-primary`, etc.).
- Typography utilities: `card-title`, `section-title`, `field-label`, `form-label`.
- New components go in `components/` and re-export from `components/mod.rs`.

## Code Style

- Rust stable, 2021 edition. `cargo fmt` + `cargo clippy -- -D warnings` must pass.
- No `unwrap()` without justification.
- Prefer well-established libraries over custom implementations.

### Comments

Default to **no comments**. Self-explanatory code with well-named identifiers
beats commented code. A reader who knows the language and framework should be
able to answer "what does this do?" from the code alone.

Add a comment only when one of these holds:

- **Non-obvious WHY** — a hidden constraint, subtle invariant, workaround for a
  specific bug, or framework quirk that would surprise a reader. Cite the
  reason concretely: an issue number, an incident, a doc link, a `BUG:` tag.
  Vague WHY is no better than restating WHAT.
- **Cross-file context** — pointing at a related file, a CLAUDE.md rule, or
  an external doc the reader could miss. One line max.
- **Section structure** — single-line dividers like `// ── Validation ──` in a
  long file. Never more than one line.

Do **not** write a comment that:

- Restates WHAT the code does (`// Filter by status` above `.filter(|g| g.status == tab)`)
- References the current task / PR (`// Added for #719`) — rots, belongs in
  the PR description
- Apologises or hedges (`// quick fix`, `// TODO come back to this`) — open a
  tracked issue instead
- Notes that a function "Mirrors X" when the shapes already make it obvious
- Is a `///` doc comment on a private item or a single-purpose component
  whose signature is self-evident

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

Ignored paths (no coverage expected): `intrada-web` (WASM shell),
`intrada-mobile` (iOS/Tauri), `migrations.rs` (SQL strings).

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

### Tauri WebView origin is `tauri://localhost`

Inside the Tauri 2 iOS shell the WebView's runtime origin is `tauri://localhost`
— **not** the dev server URL (`http://192.168.x.x:8080`) and not whatever the
production web app runs at. This affects:

- **CORS** allowlists in `intrada-api` (`ALLOWED_ORIGIN`) — must include
  `tauri://localhost`, otherwise simulator/device API calls hit preflight 403s.
- **OAuth redirect URIs** in Clerk — register the `tauri://` origin alongside
  the web ones, or sign-in returns to a broken URL.
- **CSP** in `tauri.conf.json` and any meta tags — `tauri:` scheme must be in
  `connect-src` / `frame-src` etc.

### Leptos + Crux callbacks need owner context

Callbacks invoked from raw `addEventListener` / `web_sys::EventTarget`
listeners run **outside** the Leptos owner that called `expect_context`. They
panic on context lookup. Use the `*_with_core` helpers (e.g.
`process_effects_with_core` instead of `process_effects`) which take
`SharedCore` explicitly. Sites where this matters: pull-to-refresh, drag
reorder pointer events, anything wired up via wasm-bindgen `Closure`.

### `option_env!` needs `cargo:rerun-if-env-changed`

If a build script (or `option_env!` site indirectly via macro expansion)
reads an env var, pair it with `println!("cargo:rerun-if-env-changed=NAME")`
in `build.rs`. Without it, cargo caches the macro expansion across builds and
your "I changed the env var" rebuild silently uses stale values. We've hit this
on `CLERK_PUBLISHABLE_KEY` and `INTRADA_API_URL`.

### Leptos SVG attribute values must be strings

`view! { <svg width=24 ... /> }` compiles but renders blank. SVG attribute
values in Leptos 0.8 must be strings — `width="24".to_string()` or
`width=format!("{px}")`. Same for `height`, `x`, `y`, `r`, `cx`, `cy`,
`viewBox`, etc. Numeric literals work for HTML attrs (which Leptos coerces)
but not SVG.

### Linux CI doesn't lint `#[cfg(target_os = "ios")]` branches in Tauri plugins

Plugins under `crates/intrada-mobile/plugins/` depend on `tauri = "2"` which
pulls in glib/GTK system libs that aren't on the Ubuntu CI runners. CI works
around it by `--exclude tauri-plugin-background-audio` from the workspace
clippy/test commands. That means **iOS-only code paths inside those plugins
get zero CI coverage**: a `#[cfg(target_os = "ios")] { ... return result; }`
that clippy would normally flag as `needless_return` will pass CI clean and
only fail the next time someone runs `cargo build --target aarch64-apple-ios`
locally — which on iOS-only changes is "never, until release prep."

Before pushing changes to any plugin under `crates/intrada-mobile/plugins/`,
run clippy against the iOS target locally:

```bash
cargo clippy -p tauri-plugin-<name> --target aarch64-apple-ios -- -D warnings
```

`aarch64-apple-ios` is enough — the simulator targets (`x86_64-apple-ios`,
`aarch64-apple-ios-sim`) don't add coverage clippy doesn't already get from
the device target.

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

For UI work: Pencil design first (see Pencil Design Workflow below), then
Plan mode, then implement. For non-UI work: Plan mode, then implement.
No spec doc.

### Tier 3 — Lightweight spec (rare; architectural only)
Net-new top-level features, Crux core / FFI bridge changes, auth or DB
schema changes, multi-week work spanning core + web + iOS.

Write ONE markdown doc in `specs/<feature>.md` (~100-200 lines: problem,
approach, key decisions, open questions). Then Pencil for UI work, then
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
Changes to auth, Tauri plugin IPC contracts, DB schema, or migrations go
up at least one tier regardless of file count or apparent size.

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

The [Superpowers](https://github.com/obra/superpowers) plugin provides ~14 auto-triggering skills + a methodology. We don't adopt it whole — its "always TDD, multi-stage subagent review, formal plans for everything" philosophy conflicts with the tier system above. But three of its skills are useful **when invoked deliberately**:

- **`test-driven-development`** — opt in for **non-UI Tier 2 / all Tier 3** work. Skip for visual / gesture / styling work where verification is on-device. The "watch the test fail before writing the code" discipline is what's valuable; the "delete code written before tests" rule is too strict for our pace.
- **`requesting-code-review`** — opt in **before opening any Tier 3 PR**, and for Tier 2 PRs touching auth / DB / FFI. Acts as a pre-flight checklist: does the diff match the spec, are tests passing, are there obvious quality issues. Cheaper than discovering them post-merge.
- **`using-git-worktrees`** — opt in when **two or more PR branches are in flight at once** (e.g., the recent #329 / #330 / #331 sequence). Prevents the rebase-conflict tangles that happen when squash-merges land while you're still working on the next branch.

**Do NOT enable** the rest of the methodology by default — `brainstorming`, `writing-plans`, `subagent-driven-development`, `executing-plans`, `finishing-a-development-branch`, `systematic-debugging`. Those collapse the tier system into one-size-fits-all heavyweight ceremony, which we explicitly don't want.

If you're unsure whether a skill applies, default to the tier system. The skills are sharper tools for specific situations, not replacements for "match ceremony to scope".

**Install (Claude Code):** `/plugin install superpowers@claude-plugins-official`

**Invoke selectively:** Tell the agent which skill to apply (e.g., "use test-driven-development for this"). Superpowers' default behaviour is to auto-trigger skills based on context — when invoking a single skill deliberately, also tell it to skip the others (e.g., "use just test-driven-development, no plan or subagent review needed").

**Re-evaluate** after the next 3 PRs that use any of these skills: did they catch a real issue, or did they add ceremony for its own sake? Expand scope, drop a skill, or trial another from the Superpowers set based on what we observe.

### Lessons from recent skill use

Captured after PR #719 / #724 — discipline tightening on top of the existing
guidance above.

- **TDD is the default for `intrada-core` changes.** When modifying handlers
  (`domain/*.rs`), `validation.rs`, `http.rs`, or `model.rs`, invoke
  `superpowers:test-driven-development` and **write the failing test first**.
  The core test suite runs in under a second — there's no excuse for the
  red-green-refactor shape to slip to "implement, then add tests." The
  #719 delete-404 bug shipped because the test was written after the fix,
  retrofit to pass rather than to constrain behaviour. Red phase forces you
  to state what the model owes its callers.

- **`requesting-code-review` is the standard channel for Tier 2+ PRs.**
  Don't hand-roll prompts to the `code-reviewer` agent — load the skill,
  use its structured invocation. Include "comment-policy violations are
  Blockers, not Nits" (see Code Style → Comments). When the review comes
  back, run `superpowers:receiving-code-review` on the findings before
  acting on them — "I agree" before implementing.

- **UI verification means driving the preview.** Tests don't catch flicker,
  state-after-delete, or transition timing. Either start the dev server
  yourself and exercise the flow, **or** state explicitly "I can't reach
  the running preview — user verification required for X / Y / Z." Don't
  claim "all green" when "all green" means cargo test green. CLAUDE.md
  already says this under "Doing tasks"; the lesson is to actually do it.

### Always
1. Find the roadmap item in `docs/roadmap.md`. No item = discuss first.
2. Check priority on the [project board](https://github.com/users/jonyardley/projects/2).
3. Never push to main. Always a feature branch + PR.
4. **Self-review every non-trivial PR** before declaring it ready. Use the
   `superpowers:code-reviewer` subagent (feature work) or the `/review` skill
   (small fixes), then post the summary as a `gh pr comment` so it's visible
   on the PR alongside CI — the reviewer doesn't see in-conversation subagent
   output. Apply blockers / important findings inline; defer the rest as
   tracked issues per (6). Skip self-review for Tier 1 trivia (typos, dep
   bumps, single-line config tweaks).
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
1. Update `docs/roadmap.md`, close the GitHub issue.
2. Update this file if architecture/patterns changed.
3. Update Pencil if UI diverged from design.

## Pencil Design Workflow

All design in `design/intrada.pen` (single file). Required for new views and
significant UI changes. Mobile (375px) frames are primary; Desktop (1440px) frames
are optional until web gets active investment. Reuse design system components.
Colours must reference Pencil variables, not raw hex.

## Known Tech Debt

- `Set` creates still bump `set_saves_committed` + refetch instead of using
  the temp-id mutate-response pattern (see "Mutate response" under Other
  patterns). The counter drives the save-form's optimistic→confirmed flip;
  reworking it needs to keep that affordance.

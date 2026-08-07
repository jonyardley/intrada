# intrada Development Guidelines

> Last reviewed: 2026-08-07.
>
> **This file is rules and invariants only.** The operational reference behind
> them — command mechanics, environment variables, and the incident write-ups
> that each rule came from — lives in
> [`docs/reference.md`](docs/reference.md). If you want to know *why* a rule
> exists, look there; if you want to know *what* to do, it is here.

> ## ⚠️ CURRENT FOCUS: NATIVE iOS ONLY — the web and Tauri shells are removed
>
> As of 2026-07 the **only** shell is the **native SwiftUI iOS app** on the Crux
> core (see [`specs/native-ios.md`](specs/native-ios.md)). The Leptos web shell
> (`crates/intrada-web`) and the Tauri iOS host (`crates/intrada-mobile`) were
> **deleted** (rationale: [`docs/rebuild-review.md`](docs/rebuild-review.md)) —
> do not resurrect them or add features assuming they exist. New UI work lands
> in the native iOS app and targets SwiftUI. **If a request seems to imply
> web/Leptos work, confirm the platform before writing code.**
>
> Two pieces of Swift worth reusing were preserved from the Tauri plugins —
> background audio session handling and a Live Activity implementation — under
> `ios/Reference/` (not built; see its README).

## Project Overview

intrada is a **practice coach** for musicians: the app decides what you
practise, gates every block on evidence (tap-verdicts against countable
criteria), and tells you when you're done. Around the coaching loop sit a music
library, timed sessions, and analytics, organised as three pillars — **Plan**
(library, what to practise), **Practice** (the drill loop), **Track**
(analytics, insights).

Direction and phases: `docs/roadmap.md`. What's in flight now: `docs/status.md`.

## Project Structure

```text
crates/
  intrada-core/          # Pure Crux core — business logic, no I/O
  intrada-ffi/           # UniFFI bridge — generates the Swift bindings
  intrada-api/           # REST API — Axum 0.8 + Turso (libsql)
ios/                     # Native SwiftUI app (Intrada.xcodeproj via xcodegen)
  Reference/             #   Swift kept from the removed Tauri shell (not built)
content/                 # Practice-coach authored content — gates.toml (#1194)
design/                  # Claude Design system (intrada-design-system.dc.html)
docs/                    # Roadmap, status, and the operational reference
specs/                   # Spec docs for major features (Tier 3 only)
```

## Tech Stack

- **Rust** stable (1.90.0 CI; MSRV 1.75+, intrada-api 1.78+)
- **Core**: crux_core 0.19.0, serde, ulid, chrono, thiserror
- **API**: axum 0.8, tokio, libsql 0.9 (Turso), tower-http (CORS), jsonwebtoken 10
- **Native iOS**: SwiftUI, iOS 17.0+, UniFFI + facet typegen, GRDB (on-device)
- **Auth**: Clerk (Google OAuth) exchanged for a long-lived PAT on iOS; JWT RS256
- **DB**: Turso (managed libsql/SQLite) via HTTP
- **CI/CD**: GitHub Actions → Fly.io (API) + native iOS build/test; TestFlight lane

## Commands

```bash
just check                 # fmt-check → lint → test → hygiene; mirrors CI
just test                  # nextest, same as CI's `test` job
just lint                  # clippy -D warnings, same targets as CI
just hygiene               # typos + cargo-shear
just ios-fmt-check         # Swift formatting gate (CI runs this too)
just ios                   # regen bindings (if core changed) + open Xcode
just ios-run               # build + launch on simulator + screenshot
just ios-logs              # stream booted-sim logs, filtered to our subsystem
just ios-test              # unit + snapshot (fast inner-loop tier)
just ios-test-full         # adds XCUITests (the merge gate; mirrors CI)
```

Full recipe list, TestFlight setup, binding regeneration, and the demo-data
schemes: [`docs/reference.md`](docs/reference.md). The `xcrun simctl` /
`xcodebuild` workflow, worktree sim-isolation, the green-stamp skip, and host
gotchas: [`docs/ios-testing.md`](docs/ios-testing.md).

**Run `just check` locally before pushing**, not just before committing. The
recipes mirror CI's flags and crate exclusions exactly, so local green means CI
green (cargo-deny and Gitleaks run in CI only); pushing then watching CI fail
wastes a ~3-minute roundtrip. Keep the justfile recipes and `ci.yml` in lockstep
when either changes. Changes under `ios/` additionally need `just ios-fmt-check`
(fix with `just ios-fmt`).

**Simulator safety.** The iOS Simulator and `CoreSimulatorService` are
machine-global. Before any global reset (`simctl shutdown all`,
`killall CoreSimulatorService`) or a fresh test run, check for another live
session (`xcrun simctl list devices | grep Booted`;
`pgrep -fl 'xcodebuild|XCTestAgent'`). If anything you didn't start is active,
**pause and ask the user** rather than risk killing their running sim or tests.

**Seed mode skips persistence.** `--seed-sample-data` (the **Intrada (Seeded)**
scheme, and `just ios-run`'s default) replaces the model with demo items and
**skips store hydration** — never use it when testing persistence. Use
`SEED=0 just ios-run` to launch against real on-device data.

## Knowledge graph (graphify)

A graphify graph of the repo lives in the **main checkout** at `graphify-out/`
(gitignored; worktrees don't carry it).

- **Query it first for architecture, cross-document, or spec-archaeology
  questions**; for symbol-level code navigation use grep/LSP instead, which is
  faster and always current.
- **Never build or update the graph without the committed `.graphifyignore` in
  place** — an unscoped run pollutes the god nodes with minified symbols and
  burns tokens on retired docs.
- An empty or weak result means the graph can't answer it: fall back to
  grep/LSP. Don't trigger a rebuild for one question.

Commands and refresh mechanics: [`docs/reference.md`](docs/reference.md).

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User → Events → crux_core (Rust) → Effects (Http, Persistence, App, Render) → Shell (Swift) → I/O
```

1. **Core owns all logic.** HTTP requests built in core via `crux_http`. Core
   does all JSON serialization. The shell never understands domain types.
2. **The shell is a dumb pipe.** Receives `HttpRequest` (URL, method, headers,
   bytes), fulfils it via `URLSession`, returns `HttpResponse`. No domain imports.
3. **Typed bindings, no hand-written FFI.** `Event`/`Effect`/`ViewModel` cross
   the bridge via generated bincode serializers (facet typegen + UniFFI) — Swift
   never hand-encodes a domain type.

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
  full-list refetch. Three create variants exist; **default to temp-id for new
  entities** unless one of the others applies:
  - **Temp-id mutate-response** (`Item`) — optimistic entry with a
    client-generated ulid; `*Created { temp_id, entity }` replaces it.
  - **Client-owned ulid** (`Session`) — client ulid is canonical, POST is
    fire-and-forget, the model keeps the optimistic write.
  - **Save-counter + refetch** (`Set`) — optimistic push, bump
    `set_saves_committed`, full refetch. Tech debt; don't copy it.

  Updates use `*Updated { entity }`; deletes use `DeleteConfirmed` (the model is
  already mutated optimistically). Detail: [`docs/reference.md`](docs/reference.md).

## Native iOS Shell (SwiftUI + Crux)

> The native SwiftUI app is the only shell — see
> [`specs/native-ios.md`](specs/native-ios.md). App-first, local-first. These
> rules are non-negotiable when touching the native shell.

**The shell is a dumb pipe — it owns ZERO domain logic.** It sends `Event`s,
fulfils `Effect`s (HTTP via `URLSession`, persistence via GRDB), and renders the
`ViewModel`. No business rules, no validation, no domain decisions in Swift. If
you're tempted to write logic in Swift, it belongs in `intrada-core` as an
`Event`/`Command`.

- **Bindings are a build precondition, never source.** The Swift `Event` /
  `Effect` / `ViewModel` types and serializers are **generated**. **Never
  hand-edit generated bindings** — fix the Rust type in `intrada-core` and
  regenerate. The typegen run is part of the build, not an optional step. A diff
  that edits generated Swift is a blocker.
- **`@Observable`, not `ObservableObject`.** The core-wrapping store is an
  `@Observable @MainActor` object exposing the `ViewModel` and `update(Event)`.
  Effect handlers run off the main actor, then hop back to resolve.
- **`try!` is banned like `unwrap()`.** No `try!` / force-unwraps / `as!`
  without written justification. FFI and bincode calls return real errors.
- **Persistence is a core `Effect` driven by `Command`, not Swift logic.** GRDB
  owns the tables and executes typed effects; the core decides what to read and
  write and runs LWW reconciliation. `crux_kv` is for small singletons only,
  never relational data.
- **Quality is per-screen, not deferred.** Every screen ships with a
  swift-snapshot-test, VoiceOver labels + Dynamic Type, and an iPad `SplitView`
  built *with* the screen. Sentry is wired from the first build.
- **Every colour, font, spacing and radius value is a named token** from
  `Theme.swift` (full rules under Design System Rules). Genuine one-offs (a fixed
  component height, a 2pt baseline nudge) stay literal — don't force those into
  the scale.
- **Build hazard:** UniFFI-generated Swift fails under Xcode 26 / Swift 6.2
  `MainActor`-default isolation ([uniffi-rs#2818]). Keep the generated package
  non-MainActor-defaulted (the build recipe handles it); don't "fix" it by
  editing generated code.

[uniffi-rs#2818]: https://github.com/mozilla/uniffi-rs/issues/2818

### Principles (from the 2026-06 review)

Hard-won lessons from the first full review of the native app. **Treat them like
the non-negotiables above.**

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
  the core, even before the sync engine exists. Don't encode merge rules in
  shell SQL.
- **Consolidate before you template.** Extract a shared primitive or form the
  moment a *second* screen would copy it. The Library screens are the template
  the other pillars will clone — duplication here multiplies.
- **Bridge-crossing types need a real round-trip test as a build precondition.**
  Extend the Rust `assert_round_trips` helper to every write payload *before* it
  is wired to a screen — a stub-bridge test can't catch a bincode-wire break
  (#846).

### Snapshot test hygiene

Snapshot references (`ios/IntradaTests/__Snapshots__/**/*.png`) are binaries
committed to git and re-recorded on every intentional UI change, so each
re-record adds a full copy to history forever. On the free offline tier they are
the only UI quality gate, so keep the suite but keep it lean:

- **One device + scale, deterministic host.** Pin `.iPhone13` + `displayScale`,
  force light mode at the controller, use the stub bridge. Do **not** multiply
  references by device/theme/size-class — snapshot a variant only when it can
  independently regress.
- **Snapshot load-bearing states, not the cross-product.** Prefer
  component-level (`sizeThatFits`) or structural/text snapshots where the
  assertion isn't pixel-perfect.
- **Optimize before committing.** Run `just ios-snapshots-optimize` after
  recording. CI's **Snapshot Hygiene** job enforces a per-file size ceiling and
  fails on un-optimized references.
- **No orphans.** Delete a test → delete its PNG. The same job fails any
  reference with no matching `func test…`. Check locally with
  `just ios-snapshots-check`.

### Offline-first invariants (non-negotiable)

The native app is **offline-first**: on-device SQLite is the source of truth, the
app works with no network and no account, and sync is a future paid tier. Break
one of these and the app silently stops being offline.

1. **No network on the local-first path.** A local-first feature must work in
   airplane mode. New reads/writes go through the persistence `Effect`, never
   HTTP. *(Test-enforced: local-first launch + mutations assert zero `Http`.)*
2. **Every persisted entity is sync-ready from day one** — carries `updated_at`
   and a soft-delete `deleted_at` tombstone; **no hard deletes**, so the deferred
   sync engine never needs a migration. *(Test-enforced for the schema.)*
3. **Client-owned ids.** New entities mint their ulid locally as the canonical
   id — no server-assigned-id round-trip, and no temp-id dance, in local-first.
   (The temp-id default under *Other patterns* governs server-backed writes.)
4. **Reconciliation lives in the core**, not the Swift shell. Sync / LWW / merge
   logic is Rust (shareable to Android); the shell only executes typed storage
   ops. (The dumb-pipe rule, applied to persistence.)
5. **A failed local write is never a silent success.** Storage ops resolve a real
   failure output (`PersistenceOutput::Failed`) and the core surfaces it — never
   fake an `Ack` (#816).
6. **Existing dual-mode handlers stay intact; new code is local-first only.**
   Legacy domain handlers still branch on `local_first` — when touching one, keep
   both branches passing, since core tests still exercise the online path. New
   engine/domain code targets local-first only: **the build-and-test-both-modes
   requirement is retired for new work** (see
   [`docs/rebuild-review.md`](docs/rebuild-review.md) §3).
7. **No account gate on core functionality.** Only sync (the paid tier) may
   require auth. The free app works fully signed-out.
8. **Relational data in the GRDB store; only small singletons in `crux_kv`**
   (settings, `session-in-progress` crash-recovery).

**PR checklist — any change touching persistence, sync, or a new domain entity:**

- [ ] New reads/writes go through the persistence `Effect`, not HTTP (1)
- [ ] New persisted table/columns have `updated_at` + `deleted_at`; no hard delete (2)
- [ ] New entities use a client-minted ulid as the canonical id (3)
- [ ] Any merge/reconciliation logic is in the core, not the shell (4)
- [ ] Write handlers branch on `local_first` (or use `save_or_put`) and a local
      failure resolves `Failed`, not `Ack` (5)
- [ ] Changes to an existing dual-mode handler keep both `local_first` branches
      tested; new code is local-first only (6)
- [ ] Data-model change: new migration appended (never edits a shipped one),
      additive where possible; core type + migration + codec updated together;
      ships an upgrade-path test (see below)

### Local data migrations

The on-device SQLite schema (GRDB, in `LibraryStore`) evolves via
`DatabaseMigrator`. Treat these with **more** care than the server migrations:
**on the free offline tier the device is the only copy of the user's data** — no
server backup, no DBA. A destructive or buggy migration that ships is
**unrecoverable** data loss, and you can't un-ship it.

- **Append-only, forward-only, ordered.** Add a new `registerMigration("vN_…")`;
  **never edit or delete a shipped migration** — it has already run on real
  devices, and users skip versions, so the chain must run cleanly from *any*
  past version.
- **Additive by default.** New nullable columns and new tables are safe.
  Drop/rename/retype is dangerous — use a copy-table migration, and prefer to
  **defer destructive changes until sync/backup exists**.
- **Evolve the core type + schema + codec together.** A new `Item` field needs,
  in one change: the Rust field (`Option` / `#[serde(default)]`), a migration
  adding the column with a default for existing rows, and the row↔`Item` codec.
- **Test the upgrade path, not just the end state.** Every migration ships with a
  test that a DB *populated at the previous version* migrates with data intact.

## Authentication

- **iOS**: Google OAuth runs in Safari via `ASWebAuthenticationSession` (Google
  blocks OAuth in an in-app browser). The Clerk JWT is exchanged for a long-lived
  PAT via `POST /api/auth/ios/exchange`; all later calls use the PAT.
- JWT RS256 validated against JWKS; PATs validated via SHA-256 hash lookup.
- All DB queries scoped by `user_id` (from JWT `sub` or PAT owner).
- When `CLERK_ISSUER_URL` is unset, auth is disabled (local dev only).
- Key files: `intrada-api/src/auth.rs`, `intrada-api/src/routes/auth_ios.rs`,
  `intrada-api/src/clerk.rs`.

Environment variables for every crate and the iOS build:
[`docs/reference.md`](docs/reference.md).

## Design System Rules

The native app uses a "Paper & Score" light theme: warm paper backgrounds, serif
titles (Source Serif 4), sans body text (Inter). All tokens live in `Theme.swift`
(`ios/Intrada/DesignSystem/Theme.swift`); the shareable export is
[`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html).

**Consult [`docs/design-principles.md`](docs/design-principles.md) before making
any UI/UX design decision** — new surface, layout, flow, or interaction. It is
the source of truth for how the app should feel: the "spend friction
deliberately" model, one-primary-action-per-screen, content-over-chrome,
progressive disclosure, reversible-by-default. It carries a dated decisions log
(T1–T6); when a new decision is made, append to that log rather than deciding
silently.

### Hierarchy: Tokens → Modifiers → Components → Screens

1. **Tokens first**: every colour, font, spacing and radius value traces to a
   named token (`IntradaColor`, `IntradaFont`, `IntradaSpacing`, `IntradaRadius`).
   Never hard-code a hex, a raw `.padding(16)`, or `cornerRadius: 12`.
2. **Reuse before creating**: check `ios/Intrada/DesignSystem/` and
   `ios/Intrada/Views/Components/` before building new markup.
3. **Known primitives to reach for**: `TagChip`, `TypeBadge`, `ScoreRing`,
   `BottomSheet`, `SegmentedPills`, `CardSurface`/`CardShadow`, `GlobalBanner`,
   `FormErrorBanner`, `PlaceholderContent`, `ScreenScaffold`, `SectionHeader`,
   `HairlineDivider`, `SegmentedProgress`. **Coach primitives**
   (`ios/Intrada/DesignSystem/Coach/`, canonical in the design system under
   *Components · Coach primitives*): `GateDots`, `RepVerdict`, `TapVerdict`,
   `OrientationStrip`, `BeatPosition`, `StuckTarget`, `CoachNote` — they size off
   `CoachScale` from the environment, not per-call sizes, so the whole drill loop
   grows together. Deferred coach surfaces are assembled from these; reach for
   them before drawing a new one.
4. **Every top-level screen** is built from `ScreenScaffold`
   (`ios/Intrada/DesignSystem/ScreenScaffold.swift`) so navigation chrome, safe
   areas, and background stay consistent.

### Animated reveals need an opaque backing

Anything that **slides or fades in/out over other content** — a search bar, an
expanding row, a banner, a sheet-like panel — must paint an **opaque background
token** (`paperTop` / `cardFill`, never `clear`), or the transition **ghosts**
and you see both components overlap mid-animation.

- The **moving** view gets an opaque background so it hides what it travels over.
- When it should emerge from *behind* sibling chrome, that chrome must also be
  opaque **and** sit on top (`.zIndex(1)`), or it can't occlude anything.

It's the background that hides the motion, not the transition. Don't ship a
reveal animation without checking what shows through behind it.

### Don't deviate from the system unless you're explicitly redesigning

Hand-rolled views that duplicate an existing primitive are the #1 source of
visual drift in this codebase. Before writing UI code:

- **Grep first.** About to hand-roll a chip, badge, sheet, or card that already
  exists under `DesignSystem/` or `Views/Components/`? Use the existing one.
- **Extend, don't clone.** If a primitive *almost* fits, add a parameter to the
  shared component (as `SegmentedPills` and `LibraryItemCard` already do). Don't
  ship a parallel one-off.
- **Typography**: use `IntradaFont` tokens (`.pageTitle`, `.cardTitle`,
  `.sectionTitle`, `.fieldLabel`, and for the coach loop `.drillTitle` /
  `.verdict` / `.ambient`), never a raw `.font(.system(...))`.
- **Spacing**: use `IntradaSpacing` tokens (`controlGap`, `cardCompact`, `row`,
  `card`), never a literal `.padding(16)`.

Deviation is only acceptable when **explicitly redesigning** a surface, and that
is a deliberate flagged conversation (Claude Design first, then Plan mode), not
an accident inside an unrelated feature PR. A redesign produces *updated tokens
and primitives* in `Theme.swift`, not a hand-rolled clone in a single view.

### iOS native-feel rules

- **Haptics**: use `UIImpactFeedbackGenerator` / `UISelectionFeedbackGenerator`
  via the `Store+Feedback` helpers — `selection` for tabs, `light` for taps,
  `success` for saves (only after the core confirms), `warning` for destructive
  confirms.
- **iPad**: list→detail screens use `LibrarySplitView`. Build it with the view,
  not as a retrofit.
- **Safe areas**: respect them by default (`ScreenScaffold` handles this); don't
  fight SwiftUI's layout with manual insets unless genuinely edge-to-edge.
- **Animations**: use the tokens in `Motion.swift`, not ad hoc `.spring(...)`.

## Design Workflow

Design happens in **Claude Design**; full process in
[`docs/design-workflow.md`](docs/design-workflow.md). The living reference is
[`design/intrada-design-system.dc.html`](design/intrada-design-system.dc.html),
**derived from `Theme.swift`**, which stays the canonical token source. Required
for new views and significant UI changes: mock the screen against the existing
kit first, reuse tokens and components, and if something new is needed update
`Theme.swift` and the design reference (plus its `support.js`) together.

Pencil (`design/intrada.pen`) is **retired** — do not use it or edit that file.
`design/light-mode-exploration.md` remains as provenance.

## Code Style

- Rust stable, 2021 edition. `cargo fmt` + `cargo clippy -- -D warnings` must pass.
- No `unwrap()` without justification.
- Prefer well-established libraries over custom implementations.

### Nothing unread stays in the tree

**Code with no reader gets deleted, not parked.** Not `#[allow(dead_code)]`, not
"inert until the feature returns", not a `pub` export nobody calls — those go
stale silently, carry weight into the app, and mislead the next reader into
thinking something is load-bearing. `git` is the parking space: delete it, say in
the PR (and in whatever doc described it) which PR to recover it from, and keep
the *findings* rather than the code that produced them.

This applies to deliberately-deferred work too: the MIDI capture spike and its
Rust segmentation module were deleted on exactly this rule (#1176) even though
the scoring path is expected back, with
[`docs/segmentation-findings.md`](docs/segmentation-findings.md) left as the
record. Deferring a feature means deferring its code to history.

Distinguish this from a stub a *test* reads, or an API a shell genuinely calls:
those have readers. The test is "who reads this today?", not "might someone read
this eventually?".

### Comments

Default to **no comments**. Self-explanatory code with well-named identifiers
beats commented code. A comment is justified ONLY in one of these **three
buckets**; everything else gets deleted.

1. **Section headers in a large file** — single-line dividers like
   `// ── Validation ──`. Never more than one line. (A one-line cross-file
   pointer the reader would otherwise miss counts here too.)
2. **Unusual things that need explaining** — a non-obvious WHY: a hidden
   constraint, a subtle invariant, a workaround for a specific bug, a framework
   quirk that would surprise a reader. Cite the reason concretely (issue number,
   incident, doc link, `BUG:` tag). Vague WHY is no better than restating WHAT.
3. **Hacky code that needs rework** — flag it, tied to a tracked issue:
   `// HACK(#N): …` or `// FIXME(#N): …`. A bare `// TODO come back to this`
   with no issue is not acceptable; open the issue and cite it.

`///` doc comments get the **same** treatment: a `///` narrating a self-evident
private item is noise. Delete it.

Do **not** write a comment that restates WHAT the code does; narrates
self-evident styling or structure; references the current task or PR
(`// Added for #719`, which rots and belongs in the PR description); apologises
or hedges without a tracked issue; or notes that a function "Mirrors X" when the
shapes already make it obvious.

Two-line cap as a smell test: if a comment runs longer, ask "can this be a
function name? a type? a CLAUDE.md entry?". Usually yes.

The `pre-push` hook (under `.githooks/`) flags branches pushing too many comment
lines relative to code. Bypass genuinely-justified cases with
`SKIP_COMMENT_CHECK=1 git push`.
When invoking the code-reviewer subagent (via the `requesting-code-review`
skill), include "comment-policy violations are Blockers, not Nits" so the review
treats drift as a merge-blocker.

## Testing

**Default: ship tests with new code.** New API endpoints, DB functions, and
non-trivial pure logic must include tests. The suite (`crates/intrada-api/tests/`)
uses real SQLite via `common::setup_test_app()` — no mocks needed.

- API endpoints: at minimum auth rejection paths; happy path when reachable via
  the test harness (auth-disabled mode gives a fake user).
- DB write functions: correct rows affected, idempotency, cross-user isolation.
- Pure functions: edge cases, None/empty inputs.

**iOS test framework policy**: new unit/snapshot test files use **Swift Testing**
(`import Testing`). Migrate existing XCTest files only when already touching
them — **no wholesale rewrite**. XCUITest (`IntradaUITests`) stays on XCTest.

**Before asserting, ask what the value was one line earlier.** A test whose
arrange step already satisfies its assert passes for the wrong reason and looks
like coverage forever — three landed on one branch (#1223) with exactly that
shape. When one turns up, mutation-test it (break the line it names, see if it
fails) before hardening. If nothing can distinguish the behaviour being present
from absent, the honest fix is to **delete** the test, not to bulk it out with
assertions about something else while keeping the name.

When skipping tests, say so explicitly in the PR description with the reason.
"All 157 tests pass" is not coverage — those are existing tests.

**Coverage** (Codecov, `codecov.yml`): PRs get a patch-coverage comment (70%
target, informational). **Tier 1** needs no justification. **Tier 2+** must
include a **Coverage** line in the PR description noting expected gaps *before*
CI finishes, then check the Codecov comment against it; if it's below 70% for
reasons you didn't anticipate, push tests or explain in a PR comment. Ignored
paths: `ios/`, `migrations.rs`.

## Project-specific gotchas

Bear-traps that have caught us at least once. Full write-ups with the symptom,
the diagnosis and the fix: [`docs/reference.md`](docs/reference.md).

- **JSON-only serde attrs break the bincode FFI bridge.** The bridge is
  positional bincode, which has no "absent". Be wary of `deserialize_with` /
  `serialize_with`, and of `skip_serializing_if` on non-trailing fields, on
  anything crossing the bridge. If you need format-specific behaviour, branch on
  `Deserializer::is_human_readable()`. The symptom is a **silent no-op**, not a
  crash (#846).
- **Stub-bridge tests can't catch a wire break.** Cover bridge-crossing types
  with a *real*-bridge round-trip (`LiveBridge` in `StoreEffectLoopTests`).
- **`option_env!` needs `cargo:rerun-if-env-changed`.** Without it cargo caches
  the macro expansion and your "I changed the env var" rebuild silently uses
  stale values. Hit on `CLERK_PUBLISHABLE_KEY` and `INTRADA_API_URL`.

## Workflow

Match ceremony to scope. Default to less. Escalate only when work demands it.

### Tier 1 — Just do it
Bug fixes, copy changes, style tweaks, renames, lint fixes, single-file
refactors, dependency bumps, doc updates. No Plan mode, no spec. Read enough to
confirm the change, make it, verify, ship.

### Tier 2 — Plan mode (default for feature work)
New component/view following existing patterns, new API endpoint following
established conventions, adding a field to an existing model, new screen in
existing navigation. For UI work: Claude Design first, then Plan mode, then
implement. For non-UI: Plan mode, then implement. No spec doc.

### Tier 3 — Lightweight spec (rare; architectural only)
Net-new top-level features, Crux core / FFI bridge changes, auth or DB schema
changes, multi-week work spanning core + API + iOS. Write ONE markdown doc in
`specs/<feature>.md` (~100-200 lines: problem, approach, key decisions, open
questions), then Claude Design for UI, then Plan mode, then implement.

**The spec rides with the first implementation phase, not its own PR.** The spec
is the first commit on the Phase A branch; Phase A scaffold is the rest, and the
PR title and body reflect both. Reviewers sanity-check the spec against working
code rather than abstract diagrams. Phases B/C/D still ship as their own PRs.

Do not run `/speckit-*` slash commands. Historical SpecKit folders under
`specs/` are reference only.

### Domain sensitivity override
Changes to auth, the FFI bridge contract (Event/Effect/ViewModel), DB schema, or
migrations go up at least one tier regardless of file count or apparent size.

### Decision rule
If unsure between tiers, go one tier lighter. Drift up if scope expands.

| Task | Tier | Why |
|------|------|-----|
| Fix typo in a label | 1 | Trivial copy change |
| Bump a dependency with no API change | 1 | Dep bump |
| New "Recently practiced" view following existing list patterns | 2 | Established patterns |
| Refactor `intrada-core/src/domain/session.rs` (no FFI change) | 2 | Single file, non-trivial |
| Tweak retry backoff in `auth.rs` | 2 | Sensitivity override from Tier 1 |
| Add `notes` field to a piece (touches FFI + DB) | 3 | Override: FFI + schema |
| New auth provider | 3 | Auth + multi-crate |
| Migrate persistence layer | 3 | Architectural |

### Skills worth invoking deliberately

The Superpowers plugin is **disabled** (its "invoke a skill for anything"
posture conflicts with the tier system). Four of its skills were kept as
standalone globals and are invoked **by name, deliberately**:

- `test-driven-development` — **non-UI Tier 2 work and all Tier 3 work**, and
  **the default for `intrada-core` changes** (`domain/*.rs`, `validation.rs`,
  `http.rs`, `model.rs`): write the failing test first. The #719 delete-404 bug
  shipped because the test was retrofit to pass after the fix rather than
  written to constrain behaviour. Skip for visual/gesture work verified
  on-device.
- `requesting-code-review` — the standard channel for Tier 2+ PRs. Load the
  skill rather than hand-rolling a prompt.
- `receiving-code-review` — run on the findings before acting on them.
- `using-git-worktrees` — when two or more PR branches are in flight.

If unsure whether a skill applies, default to the tier system.

**UI verification means actually driving the app on the simulator**, not claiming
"all green" when that means cargo test green. If you can't reach the running app,
say exactly what needs user verification.

### Always
1. **Claim the issue before building it, and check nobody else has.** First
   action on picking up issue N, before reading code:
   `gh pr list --repo jonyardley/intrada --state open --search "N"` and
   `gh issue view N --json closedByPullRequestsReferences`. If a PR is already
   open against it, **stop and say so** rather than implementing it again. If
   clear, claim it: add the `in-flight` label and comment the branch name on the
   issue. Drop the label when the PR merges or closes.
   - The same check binds any session *recommending* the next task or writing a
     handover opener: never name an issue as next without running it, and start
     every opener with "claim #N (stop if a PR already exists)".
   - Why (2026-08-07): #1214 got two complete independent implementations
     (#1243, #1247) fifteen hours apart. The merged one was the weaker, and
     #1250 had to port back what was lost. The issue carried no assignee, label
     or comment, and `docs/status.md`'s "in flight" only lands at merge — so an
     open PR was the sole live claim signal and nothing required looking.
2. Find the roadmap item in `docs/roadmap.md`. No item = discuss first.
3. Check priority on the [project board](https://github.com/users/jonyardley/projects/2).
4. Never push to main. Always a feature branch + PR.
5. **Open/update any non-trivial PR through the `ship` skill** — don't
   `gh pr create`/`git push` feature work directly. `ship` runs the pre-push
   gates *and* the self-review in one funnel, so the review can't be skipped in a
   fast build→push cadence (which is exactly how it gets skipped when left to
   "remember to review"). `ship` uses the code-reviewer subagent via
   `requesting-code-review`; post its summary as a `gh pr comment` (the reviewer
   doesn't see in-conversation subagent output), apply blockers and important
   findings inline, and defer the rest as tracked issues per (7).
   - **Tier 1 trivia** (typos, dep bumps, single-line config) may skip the review
     step but still run the gates.
   - **Small Tier 2** — one file, no bridge / DB / auth / migration surface — may
     use `/review` in place of the subagent. Anything on the domain-sensitivity
     list, or spanning files, takes the full subagent.
6. **Check Codecov after CI** (Tier 2+). Compare the patch-coverage comment
   against the **Coverage** line in the PR description. If there are unexpected
   gaps, push tests or explain in a PR comment before calling the PR ready.
7. **Open a tracked issue for every deferred / out-of-scope item**, labelled
   (`horizon:now|next|later`, kind: `ux` / `architecture` / `bug` /
   `accessibility` / `ios` / `pillar:*`). PR descriptions are not tracking — they
   get auto-collapsed after merge. Open the issues *before* posting the
   self-review comment: "will open a follow-up if it bites" is not acceptable.
   Every self-review comment must end with `Deferred items tracked: #N, #M` or
   `none — all flagged items addressed inline`. Silent omission is the failure
   mode.

### After completing work
1. Update `docs/status.md` in the same PR; update `docs/roadmap.md` if a phase or
   direction changed; close the GitHub issue and drop its `in-flight` label.
   A landed item also leaves the **Next** list, or the next session reads a
   stale plan and re-does it.
2. Update this file if architecture or patterns changed.
3. Update the Claude Design system
   (`design/intrada-design-system.dc.html`) if UI diverged from design, and
   re-export the shareable `.html`.

## Parallel work streams (agentic sessions)

Rules for running more than one Claude Code session against this repo at once.
Evidence base: coupling analysis of the last 400 commits (2026-08).

The claim protocol in Always(1) is what stops two streams building the same
issue; these rules stop two streams colliding in the same *files*. Both apply.

### Conventions

- British English in all UI copy, comments, commit messages and PR bodies.
- No em dashes and no double dashes in prose: docs, commits, comments, PR bodies.
  One exception, settled 2026-08-06 (#1231): ` — ` as the **label separator on a
  list item** in a structured doc (`docs/status.md`, `docs/roadmap.md`,
  `design/CLAUDE.md` and this file's own lists) is house style, so match the
  siblings there. Sentences never take one, in a list item or anywhere else.

### Stream rules

- **Exactly one core+iOS vertical stream at a time.** 31% of core commits also
  touch `ios/`; two concurrent vertical features will collide.
- A **second stream** may run only in the decoupled set: `crates/intrada-api`,
  `docs/`, `specs/`, `design/`, `content/`, or CI/tooling (`justfile`,
  `.github/workflows/`). An API task that needs a new domain field is a core
  change: it joins the vertical stream.
- **Serialisation points.** If your task and another live branch both touch one
  of these, serialise rather than parallelise:
  `crates/intrada-core/src/app.rs`, `crates/intrada-core/src/domain/session.rs`,
  `ios/IntradaTests/ScreenSnapshotTests.swift`,
  `ios/Intrada/DesignSystem/PreviewSupport.swift`, `ios/project.yml`, and
  `Cargo.lock` (never pair anything with a dependency bump).
- One git worktree per stream, branched from fresh `origin/main`. Follow the
  simulator safety rule under Commands. Close the second session when its task
  ships; do not keep it warm.

### One agent per slice; fan out only on independent work

**A vertical slice is one agent's job.** Do not split core and iOS across two
agents working the same slice. In-session agent teams were tried on #1223 and
retired: on a slice coupled by a bridge contract the split caused the worst bug
in the PR, because the shell teammate couldn't see the core invariant it needed.
The measured post-mortem is in [`docs/reference.md`](docs/reference.md).

**Fan out to worktrees when the pieces are genuinely independent** — no shared
contract in flight, no piece blocked on another's output. Good shapes: an audit
or migration sweep across many files, N independent approaches to one design
question, or unrelated tasks in the decoupled set. Bad shape: anything where two
agents would edit either side of one contract.

- **One git worktree per agent**, branched from fresh `origin/main`. Separate
  checkouts mean no shared-index hazard, so a bare `git commit` is safe — the
  explicit-pathspec rule existed only for the retired shared-checkout teams.
- **The lead integrates.** Fan-out agents report; they do not merge into each
  other's work. Reconcile in one place.
- **The simulator is machine-global.** Only one agent runs iOS tests at a time.
- `using-git-worktrees` for the mechanics; the `Workflow` tool's
  `isolation: 'worktree'` does the same per-agent when scripting a fan-out.

**Contract before code applies to one agent as much as to several.** Pin the
Event/Effect/ViewModel shape for a slice before wiring either side — that
discipline is what makes bridge changes reviewable, not a handoff protocol.

### Definition of done (every stream, before requesting review)

- [ ] `just check` green locally; `just ios-fmt-check` too if `ios/` touched
- [ ] Tests shipped with the new code (see Testing)
- [ ] PR opened via the `ship` skill; self-review comment posted
- [ ] Codecov compared against the PR's Coverage line (Tier 2+)
- [ ] `docs/status.md` updated (roadmap too if a phase changed); deferred items
      tracked as issues
- [ ] A human reviews and merges. Agents never merge.

## Known Tech Debt

- `Set` creates still bump `set_saves_committed` + refetch instead of the temp-id
  mutate-response pattern. The counter drives the save-form's
  optimistic→confirmed flip; reworking it needs to keep that affordance.

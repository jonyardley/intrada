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

intrada is a **practice notebook** for musicians: build a session from the
music library, group and reorder what you'll practise, play it through with a
timer and rep counting, and score how it went. Organised as three pillars —
**Plan** (library, what to practise), **Practice** (the built session),
**Track** (analytics, insights). (The 2026-07 practice-coach direction was
built through Phase 2b and reversed on 2026-08-13, #1344 — see
`docs/roadmap.md`'s banner.)

Direction and phases: `docs/roadmap.md`. Which release and phase we are on:
`docs/where-we-are.md`. What's in flight right now: run `just status`, which
reads GitHub rather than a file anyone has to remember to update.

## Project Structure

```text
crates/
  intrada-core/          # Pure Crux core — business logic, no I/O
  intrada-ffi/           # UniFFI bridge — generates the Swift bindings
  intrada-api/           # REST API — Axum 0.8 + Turso (libsql)
ios/                     # Native SwiftUI app (Intrada.xcodeproj via xcodegen)
  Reference/             #   Swift kept from the removed Tauri shell (not built)
.claude/skills/          # Repo rules loaded on demand (see the pointers below)
design/                  # Claude Design system (intrada-design-system.dc.html)
docs/                    # Roadmap, status, and the operational reference
specs/                   # Spec docs for major features (Tier 3 only)
```

## Tech Stack

- **Rust** stable (1.97.1 CI; MSRV 1.90 via crux_core 0.20, shared by all crates)
- **Core**: crux_core 0.20.0, serde, ulid, chrono, thiserror
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

**Drive iOS through the `just` recipes, never a bare `xcodebuild` or an MCP
build call.** The recipes carry the destination pin, `CODE_SIGNING_ALLOWED=NO`,
the build-freshness fingerprint and the concurrency guard; an invocation missing
any of them fails in ways that look like repo faults and are not (#1536, #1537).
A passing run prints its own counts, so silence is never the evidence.

**Run `just check` locally before pushing**, not just before committing. The
recipes mirror CI's flags and crate exclusions exactly, so local green means CI
green (cargo-deny and Gitleaks run in CI only); pushing then watching CI fail
wastes a ~3-minute roundtrip. Keep the justfile recipes and `ci.yml` in lockstep
when either changes. Changes under `ios/` additionally need `just ios-fmt-check`
(fix with `just ios-fmt`).

**Read every compile error before fixing the first.** `cargo check --all-targets`
(not `cargo build`) surfaces test-code breakage in the same pass as the lib —
adding a field to a widely-constructed type otherwise costs one build per call
site, discovered one at a time. Same for Swift: read the whole `just ios-test`
error list, then fix.

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
  - **`Set`** — shell-dead since the coach-pivot builder deletion (#1344): no
    Swift caller sends a `SetEvent`, and `domain/set.rs` fires HTTP
    unconditionally with no `local_first` branch. Don't wire a new caller to
    it without fixing that first (#1348).

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
  `Theme.swift` (full rules: `.claude/skills/intrada-design-system/SKILL.md`). Genuine one-offs (a fixed
  component height, a 2pt baseline nudge) stay literal — don't force those into
  the scale.
- **Build hazard:** UniFFI-generated Swift fails under Xcode 26 / Swift 6.2
  `MainActor`-default isolation ([uniffi-rs#2818]). Keep the generated package
  non-MainActor-defaulted (the build recipe handles it); don't "fix" it by
  editing generated code.

[uniffi-rs#2818]: https://github.com/mozilla/uniffi-rs/issues/2818

### Screen quality bar and snapshot hygiene

Quality is per-screen, not deferred, and snapshot references are committed
binaries that must stay lean. **Before adding or changing a screen, or touching
`ios/IntradaTests/__Snapshots__`, you MUST read `skill://intrada-ios-quality`
(`.claude/skills/intrada-ios-quality/SKILL.md`)**
— it carries the 2026-06 review principles and the snapshot rules, and they
bind whether or not you loaded it.

### Offline-first invariants (non-negotiable)

The native app is offline-first: on-device SQLite is the source of truth, the
app works with no network and no account, and sync is a future paid tier. Break
one of the invariants and the app silently stops being offline, and on the free
tier the device is the only copy of the user's data. **Before any change
touching persistence, sync, a new domain entity, the local schema, or gating a
feature behind sign-in, you MUST
read `skill://intrada-offline-first`
(`.claude/skills/intrada-offline-first/SKILL.md`)**. It carries the eight
numbered invariants, the PR checklist and the **Local data migrations** rules,
and they bind whether or not you loaded it.

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

The native app uses a "Paper & Score" light theme. Every colour, font, spacing
and radius value is a named token from `Theme.swift`
(`ios/Intrada/DesignSystem/Theme.swift`); hand-rolled views that duplicate an
existing primitive are the #1 source of visual drift here. **Before any UI or
UX change you MUST read `skill://intrada-design-system`
(`.claude/skills/intrada-design-system/SKILL.md`)** for enforcement,
`skill://intrada-design-principles`
(`.claude/skills/intrada-design-principles/SKILL.md`) for how the app should
feel, and `skill://intrada-tone-of-voice`
(`.claude/skills/intrada-tone-of-voice/SKILL.md`) for every user-facing
string. These bind whether or not you loaded them.

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
When invoking any code-review agent for a PR, include "comment-policy
violations are Blockers, not Nits" so the review treats drift as a
merge-blocker.

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

**Mutation-test by deleting the line, not by inverting it.** Inverting is the
weaker mutation and it hides exactly the failure above: reverse a sort and a
test fails whether or not it constrains anything, because reversed input is
wrong under any assertion about order. Delete the sort instead and a test whose
fixture was already in order goes green, which is the truth about it. Same for a
filter, a guard, a clamp: remove it rather than flipping it. If deletion won't
compile, substitute the naive version a future reader would plausibly write
(`points.last()` for a `rev().find_map(...)` that skips gaps) — that is the
regression you are actually guarding against.

**A parser or validator gets a table test against its consumer, not cases you
invented.** Hand-picked inputs are picked to match the implementation you just
wrote, so they agree with it by construction. Write the table as a list of
inputs a *user* would actually produce, and assert the property the next stage
needs — `every_parse_is_a_<thing>_validation_will_accept` is the shape (the
original instance shipped with the retired coach; the rule outlives it).

**Test fixtures for a type with many fields live in one place.** Rust: a
`fixture()` constructor composed with struct update
(`Record { exit: Exit::Skipped, ..fixture() }`). Swift: a fixture enum whose
default arguments do the same job for the generated types. Adding a field
should cost one edit, not one per call site found a build at a time.

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
- **Adding a field inside the crash-recovery snapshot invalidates every blob on
  every device.** `AppEffect::SaveSessionInProgress(ActiveSession)` is
  positional bincode in UserDefaults, written by one build and read by the
  next, so a new field anywhere in `ActiveSession`'s transitive graph makes an
  old blob decode into a valid-looking wrong session. `#[serde(default)]` does
  nothing here — serde never reaches the default on a non-self-describing wire.
  The coach era missed this three times (#1223, #1244, #1256) and answered it
  with a per-variant wire-pin test, and `active_session_blob_wire_is_pinned`
  (`domain/session.rs`) now pins this blob the same way (#1345). When it fails,
  bump `Store.sessionInProgressKey` first, then re-pin; never only re-pin.
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

**A phase that introduces a bridge shape, a migration, or a change inside the
`ActiveSession` blob graph ships as two PRs: core first, screens second.** The
core PR carries the domain types, the events, the migration and the tests; the
screens PR carries the SwiftUI and its snapshots. Spanning core and screens is
not itself the trigger: a phase that adds no silent-failure surface stays one
PR, because a wrong layout fails visibly and does not need its own review
cycle. The screens PR is expected in the same working session, or the core PR
waits, since a merged core PR with no caller is shell-dead by construction,
which is how #1348 and #1374 happened.

**Review the core PR before starting the screens.** Review early and often; on
a multi-surface phase, once at the end is too late to be cheap.

`/speckit-*` slash commands are Claude-Code-only tooling and are deprecated;
don't invoke them in any harness. Historical SpecKit folders under `specs/`
are reference only.

### Domain sensitivity override
Changes to auth, the FFI bridge contract (Event/Effect/ViewModel), DB schema, or
migrations go up at least one tier regardless of file count or apparent size.

### Decision rule
If unsure between tiers, go one tier lighter. Drift up if scope expands.

Tiers set the ceremony; [`docs/model-guide.md`](docs/model-guide.md) sets the
resourcing (which model and reasoning effort per activity, and what every plan
must say about model, effort, and parallel streams). The domain-sensitivity
override above applies to model choice too.

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

### Practices worth invoking deliberately

These disciplines matter regardless of harness. In Claude Code sessions with
the Superpowers plugin, four of its skills are kept as standalone globals and
invoked **by name, deliberately** (the plugin's blanket "invoke a skill for
anything" posture otherwise conflicts with the tier system). OMP discovers the
same `.claude/skills/` packs and resolves them as `skill://<name>`; in a harness
with no skill catalogue, read the `SKILL.md` path directly.

- **Test-first for non-UI Tier 2 work and all Tier 3 work**, and **the
  default for `intrada-core` changes** (`domain/*.rs`, `validation.rs`,
  `http.rs`, `model.rs`): write the failing test first. The #719 delete-404
  bug shipped because the test was retrofit to pass after the fix rather than
  written to constrain behaviour. Skip for visual/gesture work verified
  on-device. (Claude Code: `test-driven-development` skill.)
- **Request review as the standard channel for Tier 2+ PRs**, rather than
  hand-rolling a prompt each time. (Claude Code: `requesting-code-review`
  skill; OMP: the `reviewer` agent via `task`.)
- **Read review findings before acting on them** — triage real blockers from
  noise rather than mechanically applying every comment. (Claude Code:
  `receiving-code-review` skill.)
- **Isolate concurrent branches in separate worktrees** when two or more PR
  branches are in flight. (Claude Code: `using-git-worktrees` skill; OMP: the
  `isolated` worktree option on `task`/`agent`.)

If unsure whether a practice applies, default to the tier system.

**UI verification means actually driving the app on the simulator**, not claiming
"all green" when that means cargo test green. If you can't reach the running app,
say exactly what needs user verification.

### Picking up a work item

The order when I am handed an issue. Steps 1 to 4 all happen before any code
gets written, and step 3 before any code gets read.

1. **Claim it** per Always(1) below: search for an open PR, stop and say so if
   there is one, otherwise add `in-flight` and comment the branch name.
2. **Work in a worktree** for Tier 2 and above: `just worktree-new <name>`
   branches from fresh `origin/main` and seeds the warm `target/` and
   `ios/build` caches, so the first `just check` or `just ios-test` costs what
   the main checkout pays warm instead of 5 to 10 minutes cold. Tier 1 (typo,
   lint fix, dep bump) stays in the main checkout, which is also the only place
   `graphify-out/` exists.
3. **Read the issue and what it points at**: the body, its linked issues and
   PRs, and the roadmap item. An issue citing a spec or an earlier PR is naming
   the constraints; a plan written without them gets rewritten in review.
4. **Plan, and state the resourcing in one line** before writing code: the
   model and effort this session runs at (`docs/model-guide.md`), and what goes
   to a subagent. Routing means choosing the model for the session doing the
   slice. One vertical slice stays with one agent
   (`skill://intrada-parallel-streams`); only genuinely independent pieces
   (audits, sweeps, docs, API-only work) fan out.

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
2. Find the roadmap item in `docs/roadmap.md`. No item = discuss first.
3. Check priority on the [project board](https://github.com/users/jonyardley/projects/2).
4. Never push to main. Always a feature branch + PR. **A human reviews and
   merges. Agents never merge.** Getting CI green, though, is the session's
   job and not the reviewer's: after every push, watch the run to a
   conclusion, react to what fails, push again, and only surface the PR once
   it is green or genuinely stuck. Red is something to fix, not to report.
   Check the PR's mergeability too, not just the job list: a renamed job
   leaves its old required context "expected" for ever, so the PR hangs on a
   check that will never report and no job ever fails (#1542).
5. **Open/update any non-trivial PR through a single pre-push gate** — route through
   the gate funnel (checks and self-review); do not skip review in a fast cadence.
   Before pushing, read `skill://intrada-shipping` (`.claude/skills/intrada-shipping/SKILL.md`)
   for the gate mechanics, Codecov expectations, and deferred-issue protocol.

### After completing work
1. Close the GitHub issue and drop its `in-flight` label — that *is* the
   status update, since `just status` reads GitHub. Update `docs/roadmap.md`
   if a phase or direction changed, and `docs/where-we-are.md` if the release
   or phase did. **There is no status file to edit**, deliberately.
2. Update this file if architecture or patterns changed.
3. Update the Claude Design system
   (`design/intrada-design-system.dc.html`) if UI diverged from design, and
   re-export the shareable `.html`.
4. Remove the worktree once the PR merges: `just worktree-rm <name>`, which
   also deletes that worktree's throwaway simulator. A left-behind worktree
   holds a branch and a sim that the next session has to work out the status
   of before it can trust the machine is clean.

## Parallel work streams (agentic sessions)

More than one agent session against this repo at once is allowed, but only
under rules that stop two streams colliding in the same files — the claim
protocol in Always(1) covers the same issue, these cover the same *code*.
**Before starting a second concurrent stream, fanning out to subagents, or
coordinating worktrees you MUST read `skill://intrada-parallel-streams`
(`.claude/skills/intrada-parallel-streams/SKILL.md`).** It
also carries the stream rules, the serialisation points and the definition of
done. The conventions below bind every change, concurrent or not.

### Conventions

- British English in all UI copy, comments, commit messages and PR bodies. UI
  copy has its own rules on top: `skill://intrada-tone-of-voice`
  (`.claude/skills/intrada-tone-of-voice/SKILL.md`).
- No em dashes, en dashes or double dashes in prose, comments, commits or PR
  bodies. `scripts/check-dashes.sh` enforces the em and en dash ban on changed
  lines (CI and pre-push; bypass a justified case with `SKIP_DASH_CHECK=1`),
  with the label-separator exception (#1231) for the structured docs encoded as
  its exemptions. Double dashes, commit messages and PR bodies stay on the
  author, since the gate cannot see them.
- **Plain language in docs, issues and PR bodies** (Jon, 2026-08-14). Name
  features by the musician-visible outcome ("exercises from a chord chart",
  "the Up next card"), with the codename in brackets once if git archaeology
  needs it. Issue numbers are the only stable handles — never bare workstream
  letters ("B1", "Phase B") across docs; three unrelated "Phase B"s existed
  at once when this rule was made. Issue titles state the outcome. Sweep
  test: would you say the sentence to a musician? Process terms (slice,
  stream, tier…) live in the glossary in
  [`docs/reference.md`](docs/reference.md); older docs are renamed as
  touched, not swept.

### Writing PRs and issues

The PR and issue body templates, the ones that serve both the merge decision
and the "what changed" read in a single cold read months later, live in
`skill://intrada-shipping` (`.claude/skills/intrada-shipping/SKILL.md`), with
the glossary and plain-language rule. They bind whether or not you loaded it.


## Known Tech Debt

- `Set` (`domain/set.rs`) is shell-dead and violates offline-first invariant 1
  (`.claude/skills/intrada-offline-first/SKILL.md`):
  no Swift screen sends a `SetEvent`, and its HTTP creates fire unconditionally
  with no `local_first` branch or persistence op. Tracked in #1348 — decide
  whether it's deleted (nothing unread stays in the tree) or converted to
  local-first before `RoutinesScreen` gets wired to it.
- Session reflection (`reflection_improved`/`reflection_still_rough`/
  `reflection_next_target`, `ReflectionField`, `SessionEvent::UpdateSessionReflection`)
  is shell-dead the same way: #1368 removed the Improved/Still rough/Next
  target UI from `SessionSummaryScreen`, but the core fields, event and GRDB
  `session` table columns are a domain-sensitivity-override change (FFI
  bridge + DB schema), so they stayed. Tracked in #1374.

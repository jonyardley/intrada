# intrada Development Guidelines

<!-- Rules and invariants only; loads into every session and subagent, so it
stays under 200 lines. Mechanics and incidents: docs/reference.md. Surface-bound rules live
in .claude/rules/. A rule leaves when a gate enforces it, when its incident is over 90 days
old and has not recurred, or when its surface is deleted; silent-breach invariants never
leave. Last reviewed: 2026-09-14. -->

## Focus: native iOS only

The native SwiftUI app on the Crux core is the **only** shell
([`specs/native-ios.md`](specs/native-ios.md)). The Leptos web shell and the Tauri iOS host
were deleted in 2026-07 (`docs/rebuild-review.md`): never resurrect them or assume they
exist. **A request implying web work: confirm the platform first.**

## Project

intrada is a **practice notebook** for musicians: build a session from the music library,
group and reorder what you'll practise, play it through with a timer and rep counting, and
score how it went. Pillars: **Plan** (library), **Practice** (the built session), **Track**
(analytics). Direction: [`docs/roadmap.md`](docs/roadmap.md); release and phase:
[`docs/where-we-are.md`](docs/where-we-are.md); what is in flight: `just status`, which
reads GitHub. There is no status file, deliberately.

Crates: `intrada-core` (pure Crux core, no I/O), `intrada-ffi` (UniFFI bridge generating the
Swift bindings). `ios/` is the SwiftUI app (iOS 17+, GRDB on-device). Rust 2021, MSRV 1.90.

## Commands

```bash
just check            # CI's Rust and hygiene jobs, bar MSRV and coverage
just ios              # regen bindings (if core changed) + open Xcode
just ios-run          # build + launch on simulator + screenshot (seeded data)
just ios-test         # unit + snapshot (fast tier)
just ios-test-full    # adds XCUITests (the merge gate; mirrors CI)
```

- **Drive iOS through the `just` recipes, never a bare `xcodebuild` or an MCP build, run or test call.** Xcode's `RenderPreview` is the one exception ([`docs/ios-testing.md`](docs/ios-testing.md)).
  They carry the destination pin, `CODE_SIGNING_ALLOWED=NO`, the freshness fingerprint and
  the concurrency guard (#1536, #1537). A passing run prints its own counts; silence is
  never the evidence.
- **Run `just check` before pushing**, plus `just ios-fmt-check` for `ios/` (fix with
  `just ios-fmt`); local green means CI green bar MSRV and coverage (`just msrv`,
  `just coverage`), so keep the justfile and `ci.yml` in step.
  Read every compile error before fixing the first: `cargo check --all-targets`, and the
  full `just ios-test` error list.
- **The simulator is machine-global.** `just ios-test`/`ios-test-full` wait on a
  machine-wide lock (`scripts/ios-sim-lock.sh`) rather than refusing when another session's
  run is live, and leave their sim booted for the next run, shutting it down after ten idle
  minutes: a booted device on its own is no longer a signal of anything. Global resets are
  still denied; a device outside that flow (someone poking at Device Hub by hand) still
  needs asking about before you touch it.
- **Seed mode skips persistence**: `SEED=0 just ios-run` to test persistence.
- **Read source with the Read tool, not `cat`.** Path-scoped rules fire on Read.

Recipes, binding regeneration, simulator workflow and environment variables:
[`docs/reference.md`](docs/reference.md), [`docs/ios-testing.md`](docs/ios-testing.md). The
graphify graph (`graphify-out/`, main checkout only) answers architecture questions; grep
for symbols; never rebuild it without `.graphifyignore`.

## Architecture (non-negotiables)

```text
User → Events → crux_core (Rust) → Effects (Persistence, App, Render) → Shell (Swift) → I/O
```

1. **Core owns all logic.** The shell never understands domain types.
2. **The shell is a dumb pipe.** It fulfils persistence via GRDB and renders the
   `ViewModel`. No business rules, validation, domain decisions or domain state in Swift (UI
   interaction state only): if you are tempted, it belongs in `intrada-core` as an `Event`
   or `Command`. Crash recovery: UserDefaults (`AppEffect::SaveSessionInProgress`); local
   data: GRDB (`PersistenceOperation`).
3. **Typed bindings, no hand-written FFI.** `Event` / `Effect` / `ViewModel` cross the
   bridge as generated bincode. Never hand-edit `ios/generated/`; fix the Rust type and
   regenerate.

- **Validation** lives in `crates/intrada-core/src/validation.rs`.
- **Mutate response**: writes commit locally, no refetch. Temp-id for new entities,
  `*Updated { entity }`, `DeleteConfirmed`.
- **Swift**: an `@Observable @MainActor` store, not `ObservableObject`; effect handlers run
  off the main actor and hop back. `try!`, force-unwraps and `as!` are banned like
  `unwrap()`. Persistence is a core `Effect` driven by `Command`: GRDB executes typed
  effects, the core decides reads, writes and LWW reconciliation. Singletons go to
  UserDefaults through an `AppEffect`; each blob must take a versioned key and a Rust
  wire pin. Every colour, font, spacing and radius is a named token from `Theme.swift`.

The offline-first invariants, the UI and tone rules, the per-screen quality bar and the
silent-failure hazards load from `.claude/rules/` when you read a file they cover, and bind
whether or not you have seen them.

## Code style

- `cargo fmt` and `cargo clippy -- -D warnings` pass. No `unwrap()` without justification.
  Prefer established libraries over custom implementations.
- **Code with no reader gets deleted, not parked** (#1176): not `#[allow(dead_code)]`, not
  "inert until the feature returns", not a `pub` nobody calls. `git` is the parking space;
  keep the findings and name the recovery PR. A stub a test reads, or an API a shell calls,
  has a reader.
- **Comments: default to none.** The density gate fails a push that adds too many; what it
  cannot see is the kind. Three are justified: a one-line section header
  (`// ── Validation ──`), a non-obvious WHY citing a concrete reason (issue, incident,
  `BUG:` tag), or `HACK(#N)` / `FIXME(#N)` tied to a tracked issue. Never one that restates
  the code, narrates the task or PR, hedges without an issue, or notes that X "mirrors" Y.
  `///` on a self-evident item is the same noise. Over two lines, ask whether it should be a
  name or a type.
- **British English**; UI copy is written against `docs/tone-of-voice.md`. **No em dashes,
  en dashes or double dashes**: `scripts/check-dashes.sh` enforces the first two on changed
  lines, the rest stays on the author. **Plain language in docs, issues and PR bodies**:
  name features by the musician-visible outcome; issue numbers are the only stable handles.

## Testing

**Ship tests with new code.** Non-trivial pure logic includes tests: edge cases, None and
empty inputs. New iOS test files use Swift Testing; migrate an XCTest file only when already
touching it, never wholesale; XCUITest stays on XCTest.

- **Before asserting, ask what the value was one line earlier** (#1223); if nothing
  distinguishes the behaviour present from absent, delete the test. **Mutation-test by
  deleting the line, not inverting it** (#1423).
- **A parser or validator gets a table test against its consumer** from inputs a user would
  produce (#1256); cases you invented agree with your code by construction. Fixtures for a
  many-field type live in one place (Rust `fixture()` with struct update, Swift a fixture
  enum with default arguments).
- **Say so in the PR when you skip tests.** Tier 2+ carries a **Coverage** line naming
  expected gaps before CI finishes, then checks the Codecov comment against it (70% patch
  target, informational; `ios/` ignored).

## Gotchas (each has caught us at least once; write-ups in `docs/reference.md`)

- **JSON-only serde attrs break the bincode bridge silently** (#846), and a stub-bridge test
  cannot catch it: use `LiveBridge`.
- **A field inside the crash-recovery snapshot invalidates every blob** (#1345): bump
  `ActiveSession::BLOB_VERSION` first, then re-pin; the shell's key follows it (#1116).

## Workflow

Match ceremony to scope; if unsure, go one tier lighter and drift up.

- **Tier 1, just do it**: bug fixes, copy, style, renames, lint, one-file refactors, docs.
- **Tier 2, plan comment** (default for feature work): a component or screen on existing
  patterns, an endpoint on established conventions, a field on a model. Before the first
  commit a plan of about 150 words goes on the issue as a comment (done looks like,
  decisions taken, files and lines, tests, out of scope, routing), approved by Jon and read
  by the build; one that runs long is Tier 3. UI work does Claude Design first, and a new
  screen starts by reading an existing one, which loads the UI rules.
- **Tier 3, lightweight spec** (architectural): net-new top-level features, Crux core or
  bridge changes, auth or schema changes. The plan comment, then one `specs/<feature>.md`
  of 100 to 200 lines riding as the first commit of Phase A, never its own PR.

**Domain-sensitivity override**: auth, the bridge contract, DB schema or migrations go up at
least one tier and onto the sensitive surfaces. **A phase that introduces a bridge shape, a
migration, or a change inside the `ActiveSession` blob graph ships as two PRs, core first,
screens in the same working session**, or the core PR waits (#1374); spanning core
and screens is not itself the trigger. Review the core PR before the screens.

**Two strikes, then a decision** (#1890): two corrections on one point stop the change and ask
if the approach is wrong before a third; `just claim` refuses the same at two merged PRs.

**Epics hold the backlog** (#1968): three or more issues with a working order sit under an
`epic`-labelled parent as GitHub sub-issues (`just epic-add EPIC N...`), one parent each, one
level deep, the body naming the order. `just status` shows each epic's next open child and
`just claim` names the epic. A plan that creates issues attaches them; whoever closes the last
child closes the epic. The rules: [Epics](docs/roadmap.md#epics).

Test-first for non-UI Tier 2, all Tier 3 and `intrada-core` changes by default: a test
retrofit to pass agrees with the implementation by construction (#1256). `reviewer` reviews
Tier 2+, briefed with the issue so it checks the diff against done looks like; triage its
findings. Rungs, plans and usage: [working with agents](docs/working-with-agents.md). **UI
verification means driving the app on the simulator**; if you cannot, name the hand check.

### Always

1. **Claim the issue before building it: `just claim N`** (#1214, #1702). It refuses and
   names the other branch when the issue is already claimed (the `in-flight` label, or its
   newest "Claimed" comment) or an open PR references it; otherwise it adds the label,
   comments the branch and moves the board to In progress in one step, so the board never
   gets stuck between Backlog and Done (#1660). `just pr-open` wraps `gh pr create` and
   refuses if an issue number in the title has no claim naming the current branch. A
   handover opener names the issue; the new session claims it. Drop the label when the PR
   closes (automatic on merge; drop it by hand if the PR closes without merging).
2. Find the issue's epic, or the roadmap item, or discuss first; check the
   [project board](https://github.com/users/jonyardley/projects/2). Read the issue, its epic
   and what they point at, then state the rung and, for Tier 2 and 3, write the plan comment.
3. **Always a feature branch in its own worktree, and a PR; a human merges.** Jon starts
   every session in the main checkout. The session runs `just worktree-new <name>` itself,
   prefixes every shell command with `cd <worktree> && `, and reads the rules for the files
   it touches by hand, since cd-ing does not load them (#1720, #1837). It never hands Jon a
   worktree command: a handover is an opener pasted into a new chat in main, and that
   session makes its own worktree. Who may read, build and edit where, and what the
   worktree lease claims: [`docs/worktrees.md`](docs/worktrees.md). Agents touch the main
   checkout only to `git fetch origin`; a pull in main is denied by the guard, and
   `just worktree-new` branches from fresh `origin/main`, so nothing needs the local ref
   moved (#1686, #1740). Branch protection refuses a push to main and `gh pr merge` is
   denied in settings. CI green is the session's job: after every push
   watch the run to a conclusion, react, push again, and surface the PR only when green or
   stuck. Read the PR's mergeability, not just the job list: a renamed job leaves its old
   context "expected" for ever (#1542).
4. **Ship through `/ship`**, which follows
   [`.claude/skills/intrada-shipping/SKILL.md`](.claude/skills/intrada-shipping/SKILL.md).
5. **After completing work**: close the issue, drop `in-flight` and run
   `just project-status N "Done"` (that is the status update); update `docs/roadmap.md` and
   `docs/where-we-are.md` if a phase changed, this file if architecture did;
   `just worktree-rm` once merged.

**More than one session at once** follows
[`.claude/skills/intrada-parallel-streams/SKILL.md`](.claude/skills/intrada-parallel-streams/SKILL.md);
both skills bind whether or not you loaded them.

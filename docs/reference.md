# Operational reference

> The detail behind the rules in [`CLAUDE.md`](../CLAUDE.md), read on demand.
> Why each rule exists is in [`decisions.md`](decisions.md) or beside the rule.

## Commands

`just --list` prints every recipe with one line on what it does, grouped by
iOS, Android and worktrees; the justfile is the one copy. Test tiers and
worktree simulator isolation are in [`ios-testing.md`](ios-testing.md); the
stamp is in the glossary below.

### Binding regeneration

Recipes regenerate the Swift bindings only when `intrada-core` or
`intrada-ffi` changed. `ios/generated/.gen-stamp` holds the core source hash
and the slices built: simulator recipes build one slice (#2181), `just ios` and
`ios-build-release` need all three, and `just ios-gen` forces them. A stale
stamp is the tell when iOS tests misbehave after a core type change.

### Swift formatting

`just ios-fmt-check` runs the toolchain's `swift format` on the hand-written
trees, never `ios/generated`. So `git blame` skips the one-time reformat:

```bash
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

### Logging

`just ios-logs` filters the unified log to `subsystem == "com.intrada.native"`.
`report(_:)` (`ios/Intrada/Core/Logging.swift`) logs swallowed FFI and bincode
bridge errors there, the silent no-op class (#846), which otherwise leaves no
trace wherever Sentry has no DSN: CI always, and dev unless `SENTRY_DSN_NATIVE`
is set.

### TestFlight

`just testflight` mirrors `.github/workflows/release-testflight.yml`; setup is
in [SETUP.md section 3](../SETUP.md#3-native-ios-to-testflight). A tagged run
bakes in `SENTRY_DSN_NATIVE` and creates the Sentry release; a missing DSN
secret fails it at the trigger (#1553). Symbols are not uploaded yet (#1610).

### Git hooks

A `SessionStart` hook runs `scripts/install-git-hooks.sh` for Claude Code
sessions. The pre-push check catches a push onto a merged PR's branch, flags
comment bloat and runs the dash check. Manual install and first-time iOS setup
are in the [README](../README.md#prerequisites).

### Demo data vs real on-device data

A plain launch reads the on-device store. The six sample pieces are opt-in
through `--seed-sample-data`: the **Intrada (Seeded)** scheme, or `just ios-run`
by default (`SEED=0` for real data). Seed mode (`Event::LoadSampleData`) skips
store hydration, so never use it to test persistence.

## Knowledge graph (graphify)

The graph lives in the main checkout at `graphify-out/` (gitignored); from a
worktree, that is the parent of `git rev-parse --path-format=absolute
--git-common-dir`. Install with `uv tool install graphifyy` (two y's);
`.graphifyignore` sets the scope.

```bash
graphify query "<question>"   # from the main checkout root
graphify path A B             # trace how two concepts connect
graphify . --update           # refresh after a doc-heavy merge
```

Git hooks refresh the code graph; run `--update` by hand after doc-heavy
merges.

## Environment variables

### Native iOS (optional): Sentry

`SENTRY_DSN_NATIVE` in `.env` captures crash and error events from local dev
builds, tagged `environment=development`. The chain:

1. The justfile's `set dotenv-load` feeds the variable in.
2. `xcodegen` writes it to the `SENTRY_DSN` build setting.
3. The partial `ios/Intrada/Info.plist` carries `SENTRY_DSN = $(SENTRY_DSN)`,
   so the value lands in the built plist.

Step 3 matters because `INFOPLIST_KEY_*` carries only Apple's own keys;
without it Sentry silently never starts. It is unset in CI, and an empty value
is a safe no-op.

## Mutate-response variants, in full

A write mints a ulid in core, pushes the entry into the model and dispatches
the save; `PersistenceOutput::Ack` is a no-op, `Failed` surfaces `last_error`,
and nothing refetches. A finished practice is the exception: `SaveSession` parks it
until the store answers, so its `Ack` is the push and the recovery-copy clear,
and its `Failed` keeps the summary and raises the banner (#974).

## The comment density gate

The `pre-push` hook (`.githooks/`) flags a branch pushing too many comment lines
against code, exempting diffs under 25 added code lines. Bypass a justified
case with `SKIP_COMMENT_CHECK=1 git push`, or in CI with the
`comments-justified` label. When asking any review agent to review a PR,
include "comment-policy violations are Blockers, not Nits". The policy is in
`.claude/rules/code-and-tests.md`.

## How a rule leaves CLAUDE.md

`CLAUDE.md` loads into every session, so prune it continuously. A rule leaves
when:

1. **A gate replaced it**: its prose goes in the same change.
2. **The incident aged out**: "has this happened since?" No is a delete, yes is
   a gate.
3. **The surface went away**: it goes with the surface.

Silent-breach invariants never leave: the positional bridge, append-only
migrations, the device as the only copy of the data. Worthwhile reasons go to
`decisions.md`; never leave a rule marked obsolete.

## Glossary

Process words that recur in docs, issues and PR bodies. Feature names do not
belong here: they follow the plain-language rule (`docs/style-guide.md`).

- **slice**: the smallest independently shippable piece of a feature; each one
  ships and is used before the next is built.
- **stream**: one line of work in one worktree and session; *vertical* means
  core and shells together.
- **tier**: ceremony level per `working-with-agents.md` step 4: 1 just do it, 2
  plan comment, 3 spec first.
- **fast tier / full tier**, the two iOS test gates: `just ios-test` (unit and
  snapshot) and `just ios-test-full` (adds XCUITests). CI runs the full tier on
  every PR; locally it is for debugging a UI failure (#2114). Not the ceremony
  tier above.
- **the lane**, the TestFlight release workflow
  (`.github/workflows/release-testflight.yml`); runs on a tag only.
- **the stamp**, the freshness fingerprint the iOS recipes write so an
  unchanged tree skips a stage; delete it to force every stage to run.
- **hero**, the large block at the top of the Practice tab: the Up next
  suggestion where one exists, the plain hero otherwise.
- **worktree**: a separate git checkout so parallel streams do not collide.
- **bridge**: the generated FFI boundary (Event, Effect, ViewModel) between the
  Rust core and the shells. The library rows, session history and week strip
  cross inside `AppEffect`s sent only when they change, not the `ViewModel`
  (#1801).
- **local-first**: works offline against the on-device store; no HTTP on the
  path.
- **projection / derived**: computed from existing session history; no new
  stored data.
- **shell-dead**: core code no screen calls any more; a deletion candidate
  (#1374).
- **harness**, the machinery around the model: rules, skills, agents, hooks,
  settings, the recipes agents drive, and memory. Every change to it gets a row
  in [`harness-log.md`](harness-log.md).
- **process**, the steps one unit of work takes: claim, isolate, route, plan,
  build, review, ship, measure.
- **isolation**, worktrees and who may edit where
  ([`worktrees.md`](worktrees.md)).
- **routing**, which rung a piece of work runs on. A **rung** is one model and
  effort pair, such as Opus 5.5 at `high`.
- **context**, what a session carries and for how long.
- **speed**, Jon's time from claiming an issue to its merged PR. One of the
  three numbers the harness is measured by.
- **rework**, the fix PRs, reverts and corrections a piece of work needed. The
  second of the three numbers.
- **spend**, tokens and dollars. The third number, and the constraint rather
  than the goal.

## Worked tier examples

The tier rule is in `working-with-agents.md` step 4. These calls were arguable
when it was written.

|Task|Tier|Why|
|---|---|---|
|Fix typo in a label|1|Trivial copy change|
|Bump a dependency with no API change|1|Dep bump|
|New "Recently practised" view following existing list patterns|2|Established patterns|
|Refactor `intrada-core/src/domain/session.rs` (no FFI change)|2|Single file, non-trivial|
|Tweak a column default in a `crates/intrada-store` migration|2|Sensitivity override from Tier 1|
|Add `notes` field to a piece (touches FFI + DB)|3|Override: FFI + schema|
|Change what the crash-recovery blob stores|3|Blob graph + FFI, core first|
|Migrate persistence layer|3|Architectural|

# Working with OMP on intrada

> Reference for driving this repo from OMP (`omp.sh`), usually inside cmux.
> Companion to [`working-with-claude-code.md`](working-with-claude-code.md);
> the tier system in [`CLAUDE.md`](../CLAUDE.md) and the resourcing ladder in
> [`model-guide.md`](model-guide.md) apply to both harnesses.
>
> Last reviewed: 2026-09-04.

## Start a session from the repo root

OMP reads project config from the current directory's `.omp/` only. It does not
walk up. Launch from `/Users/jonyardley/Dev/intrada` (or a worktree root), or
`.omp/config.yml` is silently ignored.

## What loads, and from where

| Thing | Path | Notes |
|---|---|---|
| Project rules | `CLAUDE.md` | Injected into every session **and every subagent**, verbatim |
| Sticky user rules | `~/.omp/agent/RULES.md` | Re-attached near the current turn, survives long sessions |
| User profile | `~/.omp/agent/AGENTS.md` | Imports `~/.claude/CLAUDE.md`, one source of truth |
| Task-scoped rules | `.claude/skills/*/SKILL.md` | Metadata only until read; both harnesses discover these |
| Personal skills | `~/.claude/skills/*` | TDD, code review, worktrees, graphify |
| Subagents | `.omp/agents/*.md` | OMP ignores `.claude/agents` by design |
| Auto-format | `.omp/hooks/pre/format-on-edit.ts` | OMP does not run `.claude/settings.json` hooks |
| Xcode tools | `.mcp.json` | xcodebuildmcp, simulator workflow |

Skills cost one line of prompt until read, so put reference-grade rules there
and keep only invariants in `CLAUDE.md`.

## Models

`.omp/config.yml` pins the ladder from `model-guide.md`, so a session here
starts on the right rung:

| Role | Model | For |
|---|---|---|
| `default` | `opus-5:xhigh` | coding and agentic work |
| `plan` | `opus-5:high` | planning conversations |
| `slow` | `fable-5:max` | the silent-failure surfaces |
| `task` | `sonnet-5:xhigh` | fan-out writers |
| `advisor` | `opus-5:high` | set but off until `--advisor` |

Switch up to `slow` before touching the bincode bridge, a GRDB migration, the
`ActiveSession` blob, or auth. That is the sensitivity override, and it applies
to models as much as to ceremony.

Prewalk is **off in this repo**. It hands the session to a cheaper model at the
first edit after any `todo` call, which inverts the override. When the work
really is mechanical, arm it deliberately:

```bash
omp --prewalk-into anthropic/claude-sonnet-5:xhigh
```

## Delegating

Available agents: `scout` (read-only research), `reviewer`, `security-reviewer`,
`librarian`, `test-runner` (repo gates), `task`, `sonic` (mechanical).

- One agent per vertical slice. Core and iOS are one job, not two.
- Fan out only on genuinely independent pieces. The lead integrates.
- Read-only research belongs on `scout`; noisy test runs belong on
  `test-runner`, which keeps build logs out of the session.
- Tell every fan-out task to skip `just check` and the test suites. Run gates
  once, at the end, from the lead.

### Isolated workspace or real worktree

`isolated: true` gives an APFS clone with its own `.git`. Measured 2026-09-04:

- carries `target/` (2.6 GB), so cargo starts warm
- does **not** carry `ios/build`, so an iOS build there is cold, 5 to 10 minutes
- being a clone rather than a worktree, it defeats the graphify hooks' worktree
  guard, so each commit kicks a full rebuild that is thrown away

Use `isolated: true` for the decoupled set and core-only Rust. Use
`just worktree-new <name>` for anything touching `ios/`; it seeds the build
caches (#1205). Worktrees live at `$INTRADA_WORKTREE_ROOT`, default
`../intrada-worktrees`.

## Build and test control

- `just check` and `just ios-test` skip on an already-green HEAD. Delete
  `target/.check-stamp` or `ios/build/.ios-test-stamp` to force a run.
- The simulator is machine-global. `scripts/check-sim-free.sh` blocks a test
  run while another agent's simulator or `xcodebuild` is live. Only one stream
  runs iOS tests at a time.
- Never run a global simulator reset. Check `xcrun simctl list devices | grep
  Booted` first and leave anything you did not start alone.
- `just status` reads GitHub for what is in flight. Claim an issue before
  building it, and stop if a PR already exists.

## Token control

Roughly, per session and again per subagent:

- `CLAUDE.md` is the fixed cost. Keep task-scoped rules in skills.
- A five-way fan-out pays the project rules five times, so prefer one capable
  agent over five when the work is not genuinely parallel.
- `scout` and `librarian` return compressed context by design. Use them instead
  of reading file after file in the lead session.
- `hideThinkingBlock` is display only. Lowering `defaultThinkingLevel` is what
  actually reduces reasoning tokens, and it costs correctness.

## Shipping

1. `just check` locally, plus `just ios-fmt-check` if `ios/` changed.
2. Open the PR, then post a self-review. In OMP that is the `reviewer` agent
   via `task`; in Claude Code it is the `ship` skill. Either way the review
   happens before a human looks.
3. Compare Codecov against the PR's Coverage line for Tier 2 and above.
4. Open a tracked issue for every deferred item. PR descriptions are not
   tracking.
5. Agents never merge. A human reviews and merges.

Stacked PRs are supported: open the child with base set to the parent's branch,
depth 2 maximum. CI runs on stacked PRs because `ci.yml` has no branch filter
on its `pull_request` trigger.

## Claude Code parity

Both harnesses work here. What differs:

- Hooks do not cross. `.claude/settings.json` PostToolUse formatting is Claude
  Code only; `.omp/hooks/pre/format-on-edit.ts` is the OMP equivalent.
- Agents do not cross. `test-runner` exists twice because the frontmatter
  schemas are incompatible. Bodies must be kept in step.
- Skills and project commands do cross: `.claude/skills/` and
  `.claude/commands/` are read by both.
- `skill://<name>` is an OMP URI. Pointers in this repo name the file path too,
  so they resolve in either harness and for a human.

## Worked examples

Both are real open issues, and both start the same way: claim the issue and
stop if a PR already exists.

```bash
gh pr list --repo jonyardley/intrada --state open --search "<N>"
gh issue view <N> --json closedByPullRequestsReferences
```

### From a cold start

Bare `omp` at the repo root starts on `default` (`opus-5` at `xhigh`) with
prewalk off. Everything below is in-session, so no launch flags are needed.

| Want | In session |
|---|---|
| A different rung | `/model`, Roles view. A role carries its pinned effort, so switching role moves model and effort together. `Ctrl+P` cycles `smol`, `default`, `slow`. |
| A bare model | `/model`, All models. Effort does not come with it, so use `--thinking` at launch when the level matters. |
| Read a skill now | `/skill:intrada-design-system` |
| Run a command | `!just check`, or `$` for Python |
| Watch subagents | `Alt+A` for the hub, `/agents` for per-agent model, prewalk and advisor |
| Arm the cheap handoff | `/prewalk`, which always targets `@smol`; use `--prewalk-into` at launch for anything else |
| Keys and commands | `/hotkeys`, `/help` |

`ultrathink` in a prompt adds a careful-reasoning notice, but it only raises
effort when `defaultThinkingLevel` is `auto`. Ours is `high`, so treat it as a
prompt hint, not a rung change. `orchestrate` adds the fan-out contract, which
is the wrong instinct on a core plus iOS slice.

### Small: #1426, a hand-rolled primitive

`ReflectionSheet.swift` builds an eyebrow by hand with `kerning(1.2)` where the
`Eyebrow` primitive uses `tracking(1.5)`, so the sheet's labels are visibly
tighter than the twenty other eyebrows in the app, and the hand-roll drops
`Eyebrow`'s un-uppercased `accessibilityLabel`, so VoiceOver reads the shouty
version.

Tier 1. One file, no bridge, no schema, no auth, so no override applies.

1. One session, no plan mode, no subagents. Delegating a one-line change costs
   more than doing it. `sonnet-5` at `low` is the rung, but on a change this
   small the ceremony of moving there costs more than the tokens it saves, so
   from a cold `omp` just work on `default`. If you know before launching:
   ```bash
   omp --model anthropic/claude-sonnet-5 --thinking low
   ```
2. `skill://intrada-design-system` binds here: reuse before creating, and
   never hand-roll something that exists. Read it before editing.
3. Replace the hand-roll with `Eyebrow`. Check the neighbouring `.badge`
   hand-roll on the same screen, and if it is a different primitive leave it
   and say so rather than widening the change silently.
4. Re-record the three affected references in one pass, then verify:
   ```bash
   just ios-snapshots-record ScreenSnapshotTests/testReflectionSheet
   just ios-fmt-check && just ios-test
   ```
   Recording is delete-then-run-twice by design, so a first-run failure is
   expected. Read the diff: the labels should get looser, not move.
5. Ship. Tier 1 trivia may skip the review subagent but still runs the gates.

Opener, after `omp` at the repo root:

```text
Claim #1426 and stop if a PR already exists, then fix it.

Read skill://intrada-design-system first. ReflectionSheet.swift hand-rolls an
eyebrow at line 174; use the Eyebrow primitive instead. There is a second
hand-roll at line 87 on the badge font: that is a different primitive, so leave
it and flag it rather than widening this change.

Re-record the affected references with just ios-snapshots-record, then
just ios-fmt-check and just ios-test. Tell me which snapshots moved and why
before opening the PR.
```

### Larger: #1512, a bound the shell should not own

`ClickSheet.swift` hard-codes `2...12` and `EntrySettingsSheet.swift` hard-codes
`3...10`, both mirroring constants in `crates/intrada-core/src/validation.rs`.
When the core's bound moves, the sheet keeps offering the old range and starts
sending values the core rejects, and the write is refused with nothing on
screen. That is the swallowed-update failure the offline-first rules exist to
prevent.

Tier 2 on file count, but projecting a bound through the `ViewModel` changes
the bridge contract, so the domain-sensitivity override puts it up a tier.

1. Contract before code, at the top of the ladder. Pin the `ViewModel` shape
   first, in one session, and write it down before either side is wired.
   ```bash
   omp --model anthropic/claude-fable-5 --thinking xhigh
   ```
   `slow` in `.omp/config.yml` is the same model at `max`, which the guide
   reserves for migrations; `xhigh` is the rung for a bridge change.
2. This is a core plus iOS vertical slice, so **exactly one stream**. Do not
   fan out, and do not run a second agent against this repo while it is in
   flight.
3. Core PR first. TDD is the default for `intrada-core`: write the failing
   test, then project the bounds. Extend the Rust `assert_round_trips` helper
   to the new view type before any screen reads it, because a stub-bridge test
   cannot catch a bincode wire break (#846).
4. Get the core PR reviewed before starting the screens. On a multi-surface
   slice, one review at the end is too late to be cheap.
5. Screens PR second. Read both bounds from the `ViewModel`, delete both
   hard-coded ranges, and add the test the issue asks for: the offered range
   matches the core's, so widening the core cannot silently leave a sheet
   behind. A shell constant that merely repeats the number is not the fix.
6. Verify on the running app, not just in CI. Bindings regenerate through
   `_ios-sync`, but a core type change means iOS tests mean nothing until they
   have:
   ```bash
   just ios-test-full
   ```
7. Ship as two PRs, core then screens, each independently reviewable.

Opener for the first session, after `omp` at the repo root:

```text
Claim #1512 and stop if a PR already exists. Plan mode first, and do not
write code this session beyond the contract.

This changes the bridge contract, so switch to /model slow before you decide
anything. Pin the ViewModel shape that projects MIN_METRE_BEATS/MAX_METRE_BEATS
and the rep-target bound, and write it into the issue before either side is
wired. Read skill://intrada-offline-first: a refused write with nothing on
screen is the failure this exists to prevent.

This is a core plus iOS slice, so exactly one stream. Do not fan out.
Core PR only when we implement: TDD, and extend assert_round_trips for the new
view type before any screen reads it. Screens are a second PR after this one
is reviewed.
```

Opener for the screens session, once the core PR is reviewed:

```text
Screens half of #1512, core PR #<N> is merged. Read both bounds from the
ViewModel and delete the hard-coded 2...12 in ClickSheet.swift and 3...10 in
EntrySettingsSheet.swift. A shell constant repeating the number is not the fix.

Add the test the issue asks for: the offered range matches the core's, so
widening the core cannot silently leave a sheet behind. Then just ios-test-full,
because the core type changed. Re-record any snapshots the control changes
touch, and say which.
```

## Troubleshooting

| Symptom | Cause |
|---|---|
| `.omp/config.yml` ignored | Session started below the repo root |
| A skill will not resolve | Skills are discovered at session start; restart after adding one |
| Model dropped mid-session | Prewalk armed; check for `--prewalk` |
| `just check` says already green | Stamp matches HEAD and the tree is clean; delete the stamp |
| Unformatted Swift or Rust reaching CI | Hook only fires on new sessions; run `just fmt` and `just ios-fmt` |

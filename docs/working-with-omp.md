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

## Troubleshooting

| Symptom | Cause |
|---|---|
| `.omp/config.yml` ignored | Session started below the repo root |
| A skill will not resolve | Skills are discovered at session start; restart after adding one |
| Model dropped mid-session | Prewalk armed; check for `--prewalk` |
| `just check` says already green | Stamp matches HEAD and the tree is clean; delete the stamp |
| Unformatted Swift or Rust reaching CI | Hook only fires on new sessions; run `just fmt` and `just ios-fmt` |

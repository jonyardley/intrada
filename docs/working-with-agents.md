# Working with Claude Code on intrada

> How one unit of work runs here: claim, isolate, route, plan, build, review,
> ship, measure. Glossary: [`reference.md`](reference.md). For a new repo, read
> [Working with an agent: what good looks like](https://claude.ai/code/artifact/153ece12-8089-40c8-9f3d-5ce484503015).

One issue, one session, one worktree, one PR, then `/clear`. The session does
the mechanics; Jon picks, approves, merges and clears. A rule a hook can
enforce loses its prose in the same change.

## 1. Claim

| You are | Do |
|---|---|
| Jon, picking the next unit | `just status` in the main checkout, not a session; open a session there with the epic's next child and one sentence |
| Starting a unit of work | `just claim N`. It refuses, naming the other branch, when the issue is claimed or an open PR references it; otherwise it labels the issue and moves the board to In progress |
| Asked "what's next" or "what can run in parallel" | Answer from `just status` and the session-start claims list, with no further reads (#1836) |
| About to start a third fix on one issue | Stop: two corrections on one point mean asking whether the approach is wrong, and `just claim` refuses a third claim at two merged PRs titled with the issue (#1890, #2593) |

A claim is the check that the work is not already in flight:

```bash
gh pr list --repo jonyardley/intrada --state open --search "<N>"
gh issue view <N> --json closedByPullRequestsReferences
```

`just pr-open` refuses a PR whose title names an issue with no claim on the
branch (#1702).

## 2. Isolate

| You are | Do |
|---|---|
| About to edit anything | `just worktree-new <name>` from the main checkout, `cd <worktree> && ` before every shell command, and read the `.claude/rules/` files for the surfaces you touch by hand |
| Handing work to another session | `just handover [N]` prints the opener; paste it into a new chat in the main checkout. Never a worktree command for Jon |
| Thinking of a second stream | Read `intrada-parallel-streams` first: two by default, at most one touching `crates/` (#1839) |
| Spawning a subagent | Name the worktree by absolute path in the brief, or its edits land in main and `reviewer` diffs the wrong tree (#1861) |

Every session that edits works in its own worktree, made by that session from
the main checkout Jon started it in (#1837). Worktrees branch from fresh
`origin/main` into `../intrada-worktrees`; `just worktree-rm <name>` removes
one and its simulator. The graphify graph lives only in the main checkout:
query it there by path for architecture answers, never a full-repo read.

Read `just worktrees` before touching a worktree you did not create: a clean
tree at main is not evidence it is free. The lease and the prefix shapes are
in [`worktrees.md`](worktrees.md). `EnterWorktree` is banned: the bash guard
then refuses every git command.

## 3. Route

| You are | Do |
|---|---|
| Opening a session | Opus 5.5 `medium`, which it opens on; name the rung only for sensitive-surface work |
| Reaching work on the sensitive surfaces | `/effort high` before it starts, saying so, and `/effort medium` after; never a new chat |
| Handing work down | Settled work goes to a subagent (step 5); a new chat only frees the session (#1836) |

Nothing with judgement runs below Opus: the lower rungs cost more in
corrections than they saved (#1893). Haiku runs `test-runner` and `Explore`
only; pins live in `.claude/agents/` and `.claude/settings.json`.

**The sensitive surfaces** are where wrong is silent: the FFI bridge
(positional bincode, #846), store migrations, the `ActiveSession`
crash-recovery blob and auth. That work, including any bridge change that is
not additive (`.claude/rules/sensitive-surfaces.md`), runs at `high` (#2044,
#2052) with `reviewer` paired from the core diff. A plan comment, direction
call or unexplained bug counts only when its work is on that list (#1985).

## 4. Plan

| You are | Do |
|---|---|
| Holding a Tier 2 or Tier 3 issue | Write the plan comment on the issue before the first commit and wait for Jon's approval, unless it is the next Tier 2 child of an epic whose body names it, off the sensitive surfaces, naming no choice Jon has not made |
| Holding a Tier 1 fix | No plan comment; build it |

Match ceremony to scope; if unsure, go one tier lighter and drift up.

- **Tier 1, just do it**: bug fixes, copy, style, renames, lint, one-file
  refactors, docs.
- **Tier 2, plan comment** (the default for feature work): a component or
  screen on existing patterns, an endpoint on established conventions, a field
  on a model. UI work does Claude Design (project "Intrada") first, and a new
  screen starts by reading an existing one, which loads the UI rules.
- **Tier 3, lightweight spec** (architectural): net-new top-level features,
  Crux core or non-additive bridge changes, auth or schema changes. The plan
  comment, then one `specs/<feature>.md` of 100 to 200 lines riding as the
  PR's first commit, never its own PR. The spec is deleted in the PR that
  ships the screens, so `specs/` holds only work in flight.

A feature ships as one PR across the stack, core commits first, then iOS and
Android: a merged core with no caller is shell-dead (#1374). On the sensitive
surfaces `reviewer` reads the core commits before the screens, and lead-only
work goes up a tier.

A plan comment is about 150 words on the shipping skill's
[Plan comments](../.claude/skills/intrada-shipping/SKILL.md#plan-comments)
skeleton; one that runs long means Tier 3. It names the issue's epic, and new
issues go under it the same turn with `just epic-add`; three or more ordered
issues with no epic are a new one ([Epics](how-the-work-runs.md#epics)).

Every plan names, per task, whether it is on the sensitive surfaces (a
hand-written opener then starts with `/effort high`), where it runs, and its
streams and serialisation points per `intrada-parallel-streams`. Worked tier
examples: [`reference.md`](reference.md).

## 5. Build

| You are | Do |
|---|---|
| Holding a settled plan on an existing pattern | One slice per `task` spawn, briefed by worktree, file and line; check its report against `reviewer` and the gate counts, never by re-reading its files (#1846) |
| About to run a gate | `test-runner`, never in the lead or inside `task`; a fan-out runs the suites once at the end |
| Needing facts from many files | `Explore` with `model: haiku`; its findings are leads, not facts |
| Looking at a file for the second time | grep, then a range. The read guard denies a repeat whole-file read, by Read or `cat`, `sed`, `head` or `tail` (#1986), and one over 400 lines (#1844) |
| Warned at 150k context | Finish the slice in hand; the rest goes to `task` |
| At 200k | The rest of the unit goes to a subagent now |
| At 400k | `/compact` now, then finish the unit |
| Mid-task and the thread matters | `/compact`, not `/clear`: it keeps the decisions and drops the tool output |

Context length is the bill, not the rung (#1985, #1849).

- **One agent per vertical slice.** Fan out only on independent pieces; the
  lead integrates.
- **`task` takes a settled slice under ten files** (`.claude/agents/task.md`,
  #1888); the sensitive surfaces stay with the lead.
- **Pin model and effort in the definition**, not the spawn. `guard-spawn.sh`
  refuses a lift above the pin and any Fable spawn (#2044); `Explore`, `fork`
  and `general-purpose` inherit the session (#1838).
- **Brief by file and line**, with acceptance criteria that can fail tomorrow,
  and have research agents mark observed against inferred.
- **A fan-out, a Workflow or a full-repo scan** is costed before it runs, and
  only when Jon asked.

**Build and test.** `just check` and `just ios-test` skip a green HEAD
(delete `target/.check-stamp` or `ios/build/.ios-test-stamp` to force), and
`scripts/ios-sim-lock.sh` queues the simulator machine-wide (#1622, #1923).
The justfile and `ci.yml` change together. More in
[`ios-testing.md`](ios-testing.md).

**Code intelligence.** Run `just lsp-setup` once per machine and per worktree
that wants Swift; without it neither LSP answers. Swift's `references` returns
nothing, so callers stay a grep job. Kotlin needs `android/local.properties`,
which the recipe writes (#2630); restart Claude Code after. Installs:
[README](../README.md#prerequisites).

## 6. Review

| You are | Do |
|---|---|
| Holding a diff that changes code or tests | `reviewer` over the local diff before the push, briefed with the worktree's absolute path and the issue number |
| Holding a diff on a sensitive surface | The same `reviewer`, paired from the core diff: `crates/intrada-ffi`, `ios/generated/`, a migration, `ActiveSession` or auth |
| Holding a docs-only diff | `/code-review` inline stands in; say "comment-policy violations are Blockers" |

`reviewer` checks the diff against the plan comment first, reports and
yields: no GitHub writes, no waiting for a PR number. The lead posts its
summary verbatim at PR creation, naming the agent.

## 7. Ship

| You are | Do |
|---|---|
| About to open or update a PR | `/ship`: gates, `reviewer`, `just pr-open` as a draft; watch CI to a conclusion and read mergeability |
| The PR is green and reviewed | `just pr-ready PR ISSUE`; a screen change also needs Jon's yes on its iPhone and Android screenshots |
| Jon, reviewing | Read the body and the self-review; merge when green |
| The unit has shipped | `/clear` in the same sitting, after `just worktree-rm` |
| Leaving for over an hour | Finish or `/clear` first: the cache expires and the first turn back re-sends everything at write price (#1842) |

Gates, Codecov, deferred issues and body templates are in
`.claude/skills/intrada-shipping/SKILL.md`. Stacked PRs go two deep. `just pr-open` refuses a Tier 1 or 2 body over
350 words (#2593). `Closes #N` closes the issue on merge; update
`docs/roadmap.md` if a phase changed and `CLAUDE.md` if architecture did.

**Verification.** For UI, drive the simulator and emulator and show proof, or
say what a human must check by hand. "All green" never means only
`cargo test`. Report failures with the output, and never call a gate broken
without making it fail.


## 8. Measure

| You are | Do |
|---|---|
| Changing the harness | Add its row to [`harness-log.md`](harness-log.md) in the same PR: date, PR, change, the number it should move, first read |
| It is Monday | Read `just usage 7 --quality` against the log; a lever unmoved after two reads is reverted, not patched |
| Correcting a rule that failed | Edit that rule; never add a second beside it |
| Wanting session costs | `just usage`, or `just usage 14` for a fortnight |
| Switching a user hook off or on | Delete its line from `hooks` in `~/.claude/settings.json`, leaving the script; restore it from the dated backup in `~/.claude/backups/` (#1990) |

The numbers are **speed** (Jon's time from claim to merged PR), **rework**
(fix PRs, reverts, corrections) and **spend** (the constraint).

A rule lives in one layer: a gate (whose prose then goes), `CLAUDE.md` (near
450 words, #2595), a path-scoped rule, a skill, an agent pin, a hook, memory or
a doc. A setting that hurts only its author is personal; anything else is
checked in. When a hook's threshold or name changes, grep the docs for it.

**A trim carries its ledger.** A cut to `CLAUDE.md`, a skill or an agent doc
lists each removed line in the PR with its new home, or says it is dropped.

## Troubleshooting

| Symptom | Cause, and what to do |
|---|---|
| A rule did not load | `paths:` rules fire on Read only, and not in a worktree driven from main; `/context` shows what loaded |
| A skill will not resolve | Skills load at session start; restart |
| A hook change did nothing | `CLAUDE_CONFIG_DIR` points elsewhere (#1857) |
| `just check` says already green | The stamp matches HEAD; delete it |
| Unformatted code reaching CI | The format hook loads at session start; `just fmt` and `just ios-fmt` |
| PR hangs on a check that never reports | A renamed job left its old required context (#1542) |
| The spawn guard refused a subagent | It lifted the pin or asked for Fable; drop the override |
| Read denied as "already in context" after `/clear` | The guard reads the transcript (#1866); range-read |
| Edit denied in the main checkout | No worktree, or the prefix is missing with more than one lease held; see `worktrees.md` |
| `gh pr merge` or a push to main refused | Merging is Jon's, on GitHub |
| `xcodebuild` refused | Use the `just` recipe |
| Waiting on the simulator lock | Another run holds it; it queues |
| A dash on a changed line | Fix the line |
| `just claim` refused | The issue is taken or has two merged PRs (#1890): pick another, finish that PR, or rethink the approach |
| XCUITests fail, `SBMainWorkspace` busy | A simulator booted outside the recipes; ask before shutting it |
| First CI run cancelled or stale | `gh pr create --label` races itself; label later, or hand Jon `gh run rerun --failed` |
| Interrupted mid-tool | Retry another way and keep working |
| A gate looks like it ran nothing | Terse output is a pass; break an assertion to prove it |

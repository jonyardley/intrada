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
| Starting a unit of work | Make the worktree first, then `just claim N` from inside it, since the main checkout refuses it. It refuses, naming the other branch, when the issue is claimed or an open PR references it; otherwise it labels the issue and moves the board to In progress. Read the `intrada-shipping` skill next, before any code: its plan comment comes before the first commit and its retro before the gates (#2709) |
| Asked "what's next" or "what can run in parallel" | Answer from `just status` and the session-start claims list, with no further reads (#1836) |
| Jon, losing track of the worktrees | `just worktree-status`: whose move each one is, and how many are safe to remove (#2783) |
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
| About to edit anything | `just worktree-new <name>` from the main checkout, then the session that does the work starts inside the worktree with the command it prints, `cd <worktree> && claude` |
| Handing work to another session | `just handover [N]` prints the opener; add the stream by hand when there is more than one. For a fresh start, make the worktree with `just worktree-new` first and paste the opener into a session started inside it, not in the main checkout; the opener's first line says where it goes. The session that wrote the opener exits first, or its lease on the worktree blocks the next one (#2788) |
| Thinking of a second stream | Read `intrada-parallel-streams` first: two by default, at most one touching `crates/` (#1839) |
| Spawning a subagent | One that edits gets its own worktree (Claude Code's, under `.claude/worktrees`) or works in yours; name the worktree by absolute path for its reads, or `reviewer` diffs the wrong tree (#1861); launch it inside that worktree (move yourself in with `EnterWorktree` first, or give it worktree isolation), since one launched in the main checkout can neither write nor move itself in (#2967) |
| Your PR has merged | `just worktree-done` inside the worktree, then end the session |

Every session or helper agent that edits works in its own worktree (#1837,
#2918). The main checkout is for status, claims and making worktrees:
its working tree is read-only, a `cd <worktree> && ` prefix from it is
refused for writes while reads pass, and `.claude/rules/` files load only when
the session's folder is the worktree and it opens a matching file with the Read
tool, never through the shell. A helper launched from such a session usually
gets them the same way, and one launched by a lead in main often does not, so
no helper counts on them. When rules do not load, read the rule file by hand. A session started in the main checkout
moves into an existing worktree with Claude Code's `EnterWorktree` tool and
the worktree's path, with no restart; `just worktree-new` makes one (#2967).
Plain git commands pass there, though Claude Code may refuse a compound one
it cannot check, so split it; every helper agent is told so at its start
(`scripts/subagent-start.sh`, #2976). Worktrees branch from fresh `origin/main` into
`.claude/worktrees/`, and the worktree lease still holds each one to one
session. After its PR merges, the session removes its own worktree and
simulator with `just worktree-done`, which refuses unless the PR is merged, the
tree is clean and nothing is unpushed; the worktree guard checks the same and
refuses the usual removal commands on another session's worktree, with some
unusual command shapes as known limits. `just worktree-rm <name>` is the manual
removal; it and forced removals skip the merged, clean and pushed check. The
graphify graph lives only in the main checkout:
query it there by path for architecture answers, never a full-repo read.

Read `just worktrees` before touching a worktree you did not create: a clean
tree at main is not evidence it is free. The lease and what the guard allows
are in [`worktrees.md`](worktrees.md).

## 3. Route

| You are | Do |
|---|---|
| Opening a session | The model and effort `.claude/settings.json` opens it on |
| Reaching work on the sensitive surfaces | Stay on the opening level, which is the role table's; never ask Jon to type `/effort`, and never a new chat |
| Handing work down | Settled work goes to a subagent (step 5); a new chat only frees the session (#1836) |

A session opens on `.claude/settings.json`. Every spawn's model and effort
come from the role table in
`~/.claude/CLAUDE.md` (Subagents, review and lane size), named on each Agent
call; this repo keeps no levels of its own. Review
rounds, lane size and the one approved plan per change are in the same file's
"Speed and reliability"; here only the local detail.

`/effort` saves to settings, so a session never runs it. Fast mode costs twice the standard
rate: use it only when latency matters, never to save money.

Superpowers is disabled on purpose, because its "invoke a skill for anything"
posture fights the tiers; four of its skills stay as globals
(`test-driven-development`, `requesting-code-review`, `receiving-code-review`,
`using-git-worktrees`), and `/speckit-*` is deprecated.

**The sensitive surfaces** are where wrong is silent: what crosses the FFI
bridge (positional bincode, #846), store migrations, the wire-pinned blobs and
auth. They are judged by type, not folder: a changed item deriving Facet (view
types, `Event` and its sub-events, effect operations), a `BLOB_VERSION` line,
the store or the FFI source, as `just review-route` names them (#2922); auth
is judged by hand. That work, including
any bridge change that is not additive (`.claude/rules/sensitive-surfaces.md`),
runs at the role table's level for sensitive surfaces with `reviewer` paired
from the core diff. A plan comment, direction
call or unexplained bug counts only when its work is on that list (#1985).

## 4. Plan

| You are | Do |
|---|---|
| Holding a Tier 2 or Tier 3 issue | Write the plan comment on the issue before the first commit and wait for Jon's approval, unless it is the next Tier 2 child of an epic whose body names it, off the sensitive surfaces, naming no choice Jon has not made |
| Holding a plan comment older than a merge it waited on | Rewrite it against main before building: platforms, line numbers and blockers move (#2792) |
| Holding a Tier 1 fix | No plan comment; build it |

Match ceremony to scope; if unsure, go one tier lighter and drift up. Before
researching or planning a topic, search issues and merged PRs for it, since
it may have shipped already (#2934).

- **Tier 1, just do it**: bug fixes, copy, style, renames, lint, one-file
  refactors, docs.
- **Tier 2, plan comment** (the default for feature work): a component or
  screen on existing patterns, an endpoint on established conventions, a field
  on a model. A new screen gets a SwiftUI sketch and Jon's word on it first
  (`docs/design-workflow.md`), and starts by reading an existing screen, which
  loads the UI rules.
- **Tier 3, lightweight spec** (architectural): net-new top-level features,
  Crux core or non-additive bridge changes, auth or schema changes. The plan
  comment, then one `specs/<feature>.md` of 100 to 200 lines riding as the
  PR's first commit, never its own PR. The spec is deleted once the screens
  ship, unless an open issue cites it (`specs/README.md`).

A feature ships as one PR across the stack, core commits first, then iOS:
a merged core with no caller is shell-dead (#1374). On the sensitive
surfaces, `reviewer` reviews the core commits before the screens commits
start, and lead-only work goes up a tier.

A plan comment is about 150 words on the shipping skill's
[Plan comments](../.claude/skills/intrada-shipping/SKILL.md#plan-comments)
skeleton; one that runs long means Tier 3. It names the issue's epic, and new
issues go under it the same turn with `just epic-add`; three or more ordered
issues with no epic are a new one ([Epics](how-the-work-runs.md#epics)).

Every plan names, per task, whether it is on the sensitive surfaces, where it runs, and its streams and serialisation points per `intrada-parallel-streams`. Worked tier
examples: [`reference.md`](reference.md).

## 5. Build

| You are | Do |
|---|---|
| Holding a settled plan on an existing pattern | One slice per `task` spawn, briefed by worktree, file and line; check its report against `reviewer` and the gate counts, never by re-reading its files (#1846) |
| About to run a gate | `test-runner`, never in the lead or inside `task`; a fan-out runs the suites once at the end |
| Needing facts from many files | `Explore`, at the role table's level for searches; its findings are leads, not facts |
| Looking at a file for the second time | grep, then a range; a screenshot is read once and described. The read guard denies a repeat whole-file read, by Read or `cat`, `sed`, `head` or `tail` (#1986), and one over 400 lines (#1844) |
| Warned at 150k context | Finish the slice in hand; the rest goes to `task` |
| At 200k | The rest of the unit goes to a subagent now |
| At 400k | `/compact` now, then finish the unit |
| Mid-task and the thread matters | `/compact`, not `/clear`: it keeps the decisions and drops the tool output |

Context length sets the bill (#1985, #1849).

- **One agent per vertical slice.** Fan out only on independent pieces; the
  lead integrates.
- **`task` takes a settled slice of at most ten hand-written files**
  (`.claude/agents/task.md`, #1888); the sensitive surfaces stay with the lead.
- **Name model and effort on every spawn**, from the role table; agent
  definitions carry no pins. `guard-spawn.sh` brings a spawn with no model
  named, on Fable, or above medium effort to Jon as a permission prompt.
- **Brief by file and line**, with acceptance criteria that can fail tomorrow,
  and have research agents mark observed against inferred.
- **A fan-out, a Workflow or a full-repo scan** is costed before it runs, and
  only when Jon asked.
- **Before editing a shared script**, check `just status` for in-flight work
  that touches it, or the clash shows only at the merge (#2198).

**Build and test.** `just check` and `just ios-test` skip a green HEAD
(delete `target/.check-stamp` or `ios/build/.ios-test-stamp` to force), and
`scripts/ios-sim-lock.sh` queues the simulator machine-wide (#1622, #1923).
The justfile and `ci.yml` change together. More in
[`ios-testing.md`](ios-testing.md).

**Code intelligence.** Run `just lsp-setup` once per machine and per worktree
that wants Swift; without it neither LSP answers. Swift's `references` returns
nothing, so callers stay a grep job. Restart Claude Code after. Installs:
[README](../README.md#prerequisites).

## 6. Review

Who reviews, and when, is the review step of the route in the
[shipping skill](../.claude/skills/intrada-shipping/SKILL.md#the-route).

## 7. Ship

The route from finished code to merge, in order, is in the
[shipping skill](../.claude/skills/intrada-shipping/SKILL.md#the-route).
Stacked PRs go two deep. Leaving for over an hour: finish or `/clear` first,
since the cache expires and the first turn back re-sends everything at write
price (#1842).

**Verification.** For UI, drive the simulator and show proof, or
say what a human must check by hand. "All green" never means only
`cargo test`. Report failures with the output, and never call a gate broken
without making it fail. Before fixing a CI failure, read the failing step's
last lines: a cause an issue gives is a guess until a log shows it. Delete
any scratch simulator you made when done. If
CI is green, local pty errors were host-only.

**Red main is fixed or switched off the same day.** The alert on the
`ci-red-main` issue names the failing job and step. Fix it, or, when a step
that runs only on main is itself broken, switch it off in a PR that day
(`if: false` with the issue number beside it) and open an issue to switch it
back on. Red left standing hides the next real break (#2747).

## 8. Measure

| You are | Do |
|---|---|
| Changing the harness | Add its row to [`harness-log.md`](harness-log.md) in the same PR: date, PR, what changed, with the number read once the draft opens, never guessed; a `/intrada-ship` retro fix is recorded in the lessons archive instead |
| Correcting a rule that failed | Edit that rule; never add a second beside it |
| Wanting session costs | `just usage`, or `just usage 14` for a fortnight; it ends with how many fixed lessons came back, the fixes that did not hold (#2968) |
| Switching a user hook off or on | Delete its line from `hooks` in `~/.claude/settings.json`, leaving the script; restore it from the dated backup in `~/.claude/backups/` (#1990) |

The numbers are **speed** (Jon's time from claim to merged PR), **rework**
(fix PRs, reverts, corrections) and **spend** (the constraint).

A rule lives in one layer: a gate (whose prose then goes), `CLAUDE.md` (near
450 words, #2595), a path-scoped rule, a skill, an agent pin, a hook, auto
memory (which does not load into subagents) or a doc. A setting that hurts
only its author is personal; anything else is checked in. When a hook's
threshold or name changes, grep the docs for it. Every user hook fails open
and ships with a test battery that is mutation-tested, not trusted green: each
mutation gets a fixture where only the mutated line decides, since a shadowed
line stays green (#2866), and for a small fix, reverting it and rerunning the
self-test is the cheapest mutation (#2845). A self-test's text check is a
substring match, so a printed command that is a prefix of another passes on
the longer one alone. Match the whole line, or run what is printed through
the fake and check what it shows, with the fake printing what the real tool
prints for each form of the command (#2969). Edit
rule files with `/memory`; tidy permission prompts with
`/fewer-permission-prompts`.

**A trim carries its ledger.** A cut to `CLAUDE.md`, a skill or an agent doc
lists each removed line in the PR with its new home, or says it is dropped.

## Troubleshooting

| Symptom | Cause, and what to do |
|---|---|
| A rule did not load | `paths:` rules fire on Read only, and only when the session's folder is the worktree, started there or moved in with `EnterWorktree`; a helper usually gets them the same way, less often when its lead sits in main, so no helper counts on them; read the rule file by hand, and `/context` shows what loaded |
| A skill will not resolve | Skills load at session start; restart |
| A hook change did nothing | `CLAUDE_CONFIG_DIR` points elsewhere (#1857) |
| `just check` says already green | The stamp matches HEAD; delete it |
| Unformatted code reaching CI | The format hook loads at session start; `just fmt` and `just ios-fmt` |
| PR hangs on a check that never reports | A renamed job left its old required context (#1542) |
| The spawn guard refused a subagent | It lifted the pin or asked for Fable; drop the override |
| Read denied as "already in context" after `/clear` | The guard reads the transcript (#1866); range-read |
| Edit denied in the main checkout | The session is not in its worktree; start one there with `cd <worktree> && claude`; see `worktrees.md` |
| `just worktree-done` refused | The PR is not merged, the tree has changes, or a commit is not in the merged PR; the message names which |
| `gh pr merge` or a push to main refused | Merging is Jon's, on GitHub |
| `xcodebuild` refused | Use the `just` recipe; for the installed version, `-version` is refused too, so read Xcode's Info.plist with `plutil -p` |
| A worktree session's command refused as too complex | A loop, a compound command, or text that looks like git (a `.github` path) tripped the guard; split it into plain separate commands. A loop over git, such as a mutation run, goes in a scratch script, written before the first try |
| Waiting on the simulator lock | Another run holds it; it queues |
| A dash on a changed line | Fix the line |
| `just claim` refused | The issue is taken or has two merged PRs (#1890): pick another, finish that PR, or rethink the approach. An open PR that only names it as a follow-up it filed is not a claim: read that PR, then post the claim comment and add the `in-flight` label by hand (#2988) |
| XCUITests fail, `SBMainWorkspace` busy | A simulator booted outside the recipes; ask before shutting it |
| First CI run cancelled or stale | `gh pr create --label` races itself; label later, or hand Jon `gh run rerun --failed` |
| Interrupted mid-tool | Retry another way and keep working |
| A gate looks like it ran nothing | Terse output is a pass; break an assertion to prove it |

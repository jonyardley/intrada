# Working with Claude Code on intrada

> The mechanics of driving this repo from Claude Code: what loads and what it
> costs, the layers a rule can live in, model and effort per activity, what a
> plan must say, delegation, isolation, guardrails and worked examples. The tier
> system in [`CLAUDE.md`](../CLAUDE.md) is normative and not restated. This
> file replaced the OMP-era guides and `model-guide.md` on 2026-09-08; why OMP
> was retired is in [`reference.md`](reference.md).
>
> Last reviewed: 2026-09-14, against the Claude 5 family (Fable 5.1, Opus 5,
> Sonnet 5, Haiku 4.5). Re-review at the next model generation.
>
> The generic version of this file, with the intrada names taken out, is
> [`agentic-primer.md`](agentic-primer.md): read that to set up a new repo,
> this to drive this one.

## When to do what

The rest of this file explains each row; this table is the one to read
mid-session when you need the next move. Every rule here is held by a hook or
a recipe named under "Guardrails already in place", and the number it was set
from is under "Usage guidelines".

| You are | Do | Explained under |
|---|---|---|
| Starting a unit of work | `just claim N`, `just worktree-new <name>`, prefix every shell command with `cd <worktree> && `, read the `.claude/rules/` files for the surfaces you will touch, and name the model and effort before the first edit | CLAUDE.md Always; Isolating concurrent work; Model and effort |
| Choosing a rung | Read the activity ladder row for what you are doing now, not the file count. Decisions go up the ladder, execution goes down it; `/model` and `/effort` switch in place | Model and effort |
| Holding a settled plan whose build follows a pattern in the repo | Hand each slice to `task` (Sonnet 5 `high`), one slice per spawn, briefed by worktree, file and line; check its report against `reviewer` and the gate counts, never by re-reading its files | Delegating |
| Holding a fully specified mechanical edit | `smol` (Haiku 4.5 `low`) | Delegating |
| About to run a gate | `test-runner`, never in the lead; a fan-out task skips the suites and the lead runs them once at the end | Delegating |
| Needing facts from many files | `Explore` with `model: haiku`, spawned from a session at `high` or below; its findings are leads, not facts | Delegating |
| About to open or update a PR | `/ship`: gates through `test-runner`, `reviewer` over the diff (Opus on a sensitive surface), then `just pr-open`; watch CI to a conclusion in the same turn and read mergeability | Shipping; Verification, the standard held |
| Looking at a file for the second time | grep, then a range read. The read guard denies the whole file, and it also denies the first read after `/clear` of a file the session read before it (#1866): range-read round that | Usage guidelines; Troubleshooting |
| Warned at 150k context | Finish the slice in hand; everything after it goes to `task` or `smol`, and this session checks and ships | Session controls |
| At 200k | The rest of the unit goes to a subagent now | Session controls |
| At 400k | `/compact` now, then carry on to the end of the unit | Session controls |
| Mid-task and the thread matters | `/compact`, not `/clear`: compact keeps the decisions and drops the tool output, and the path-scoped rules reload on the next Read | Session controls |
| The unit has shipped: PR green, issue closed, worktree removed | `/clear` in the same sitting | Usage guidelines |
| About to leave the session for over an hour | Finish the unit or `/clear` first; the first turn back re-sends the whole context at write price, and the cold nudge will say so | Usage guidelines |
| Needing a stronger rung for one decision | `/model` and `/effort` in this session, saying so, and back down at the boundary. A new chat frees the lead; it is never how a rung changes | Model and effort |
| Handing work to another session | An opener the new session pastes into a chat opened in the main checkout, naming the issue, activity, model, effort and stream; that session claims the issue and makes its own worktree. Never a worktree command for Jon to run | Plans ship their own resourcing |
| Thinking of a second stream | Read `intrada-parallel-streams` first: two by default, at most one touching `crates/`, a third only shell-only and announced | Isolating concurrent work |
| Asked "what's next" | Answer from `just status` and the session-start claims list, with no further reads | Usage guidelines |

## What loads, and what it costs

A session pays a fixed context bill before the first word, and a subagent pays
most of it again. Everything in the first column below is in that bill.

| Input | Path | When it loads |
|---|---|---|
| Project rules | `CLAUDE.md` | Every session and every subagent, in full. Under 200 lines by rule |
| Path-scoped rules | `.claude/rules/*.md` | When a file matching the rule's `paths:` globs is read with the Read tool. A `cat` through Bash does not count, and a file inside a worktree driven from the main checkout does not fire them either: read them by hand there. They reload the same way after compaction |
| Skills | `.claude/skills/*/SKILL.md` | The description every session; the body when invoked by name (`/ship`, `/intrada-parallel-streams`) |
| Agents | `.claude/agents/*.md` | The description every session; the body becomes the subagent's system prompt |
| Repo settings | `.claude/settings.json` | The model and effort a session opens on (Sonnet 5 medium, 1M window), permissions, the format-on-edit and git-hook-install hooks, and the plugins switched off for this repo |
| User rules | `~/.claude/CLAUDE.md`, `~/.claude/rules/` | Every session, before the project rules; project wins on conflict |
| Auto memory | `~/.claude/projects/<project>/memory/MEMORY.md` | Every session, first 200 lines. Not loaded into subagents. The one input that can carry a stale fact |
| User hooks | `~/.claude/settings.json`, `~/.claude/hooks/` | Text injected on every prompt (the turn reminder, and the context watch nudges, thresholds below), after a push (the CI-watch note) and in the day's first session (one usage line); before a tool runs, the worktree guard on every edit and command, the bash guard on every command, the read guard on every Read, the spawn guard on every subagent. Each is listed under "Guardrails already in place" |
| Plugins | `enabledPlugins` in user settings | Each plugin skill's description, every session. `.claude/settings.json` switches slack, atlassian, visual-explainer and frontend-design off here |
| Xcode tools | `.mcp.json` | xcodebuildmcp and the simulator workflow |

The `~/.claude/` rows above (user rules, auto memory, user hooks) move whole
if `CLAUDE_CONFIG_DIR` is set: a session on such a machine reads
`$CLAUDE_CONFIG_DIR/CLAUDE.md`, `$CLAUDE_CONFIG_DIR/settings.json` and
`$CLAUDE_CONFIG_DIR/projects/<project>/memory/` instead, and adding a hook to
`~/.claude/settings.json` there does nothing: caught during the #1849 epic
cross-check, where a hook shipped registered in the undocumented path only.

Run `/context` in a session to see exactly which files loaded.

## The layers, and which one a rule belongs in

A rule lives in exactly one of these, chosen by who reads it and when.

1. **Gates.** Branch protection, `permissions.deny`, the bash guard, CI, the git
   hooks. Enforced whether or not anyone read a rule, so the prose comes out in
   the same change that adds the gate. The comment density check, the dash
   check and the simulator lock are the models.
2. **`CLAUDE.md`.** Invariants every session needs: architecture, the tier
   system, the Always list. Paid on every request, so it stays under 200 lines.
3. **Path-scoped rules.** Offline-first, the UI and tone rules, the per-screen
   quality bar and the silent-failure hazards. Free until a covered file is
   read, then in context for the rest of the session.
4. **Skills, by name.** Workflows rather than surfaces: shipping and parallel
   streams. `/ship` is the pre-push funnel.
5. **Agents.** `reviewer`, `test-runner`, `smol`, `task`, with model and effort
   pinned in the definition. `Explore`, `fork` and `general-purpose` are
   built-in with no definition to pin ("Delegating" below).
6. **Hooks.** Repo: format on edit, install the git hooks, the session-start
   claims list. User: the worktree guard and its lease, the bash guard, the
   read guard, the spawn guard, the per-prompt reminder and context watch, the
   post-push CI note, the daily usage line, and the cold nudge on a launchd
   timer. Every user hook fails open and ships with a test battery that is
   mutation-tested, not trusted green.
7. **Auto memory.** What neither CLAUDE.md records: preferences, corrections,
   project state the code cannot show.
8. **Docs on demand.** `reference.md` for the why behind every rule, the specs,
   the roadmap. Cost nothing per turn.

The test for the personal versus project split, from the RIBA template review:
if this is set wrong, who does it hurt? Only the author, and it is personal
(`.claude/settings.local.json`, `~/.claude`). Anyone else, and it is project,
checked in, and enforced somewhere the personal layer cannot weaken it.

## Usage guidelines

Nine rules that balance speed, cost and quality, each with what holds it and
the number it was set from. Set on 2026-09-14 from the review in #1836: three
days, $1,225, 76 sessions, 44 PRs opened from Friday evening and 40 of them
merged.

| Guideline | What holds it | Set from |
|---|---|---|
| Open on Sonnet 5 medium. Go up by activity, not by task size, and name the rung and effort at every boundary | `.claude/settings.json`, the turn reminder | Monday at 65% Sonnet cost a quarter of Saturday for the same PR count |
| One unit per session: finish, `/clear`. Before a break over an hour, `/clear` | `context-watch.sh` on the next turn; a launchd timer (`cold-nudge.sh`, machine-local, #1842) nudges before that turn arrives | Seven sessions over four hours, every one Fable or Opus left open across a break; 194 cold turns cost $233 in the week before |
| Two streams by default; a third only shell-only, and say so when starting it | `intrada-parallel-streams` (#1839) | 34 simulator-busy hits across 20 sessions |
| The session makes and drives its worktree. Jon never runs a worktree command, and a handover is an opener pasted into a new chat in main | Always step 3; `guard-worktree.sh` denies the write in main, and #1840 makes the prefix automatic | Eleven corrections in three days, 51 denied writes in main |
| Settled work goes to subagents from one lead; a new chat is for freeing the session or going up a rung, never for work the lead could dispatch | `guard-spawn.sh`, the agent pins, #1838 | Twelve of 76 sessions were status or handover only |
| "What's next" and "what can run in parallel" are answered from `just status` and the session-start claims list, with no further reads | #1839 | Seven such sessions, each a fresh 80k to 200k context, all picking Tier 1 work |
| A correction edits the rule that failed; it never adds one. A rule a hook can enforce loses its prose in the same change | This section, "The layers" above | Three worktree rules that disagreed, six memories, and the mistake back the next morning |
| One harness slot a day; the rest is the app. Frozen until the #1849 re-measure on 2026-09-27, so the measurement is clean | Jon's call at planning | Nine of the 44 weekend PRs were harness work, and nine harness PRs merged on 2026-09-14 alone |
| Read a file once. A second look is a grep and a range, never the whole file again; a screenshot is read once and described | `read-guard.sh` denies a repeat and an unranged file over 400 lines (#1844); `rtk read` for the rest | 981 of 1,930 text reads over the weekend repeated a file already in the session; one core file was read 225 times |

A PR body says what the reviewer needs and stops. The weekend's median was 430
words with three over 1,000, against merges ten minutes after opening. Whether
that becomes a gate is decided by measuring the PRs after #1634 against its
586-word baseline, an experiment Jon set on 2026-09-10 with no gate on purpose.

## Model and effort

Match the **model** to how silently wrong the work can go, and the **effort** to
how much thinking beats typing. The silent-failure surfaces here are the FFI
bridge (positional bincode: wrong is a no-op, not a crash, #846), local GRDB
migrations (the device is the only copy of the user's data), the `ActiveSession`
crash-recovery blob (#1223, #1244, #1256) and auth. Those get the strongest
setup regardless of diff size. Everything else degrades gracefully because
failure is visible: a wrong layout is caught on the simulator, a wrong test
fails in CI.

| Model | $/MTok in/out | Use for |
|---|---|---|
| Fable 5.1 | 10 / 50 | Unrecoverable-if-wrong work; direction-setting; the worst debugging |
| Opus 5 | 5 / 25 | Default for judgement-dense feature work and reviews |
| Sonnet 5 | 2 / 10 | Conventional coding on non-sensitive surfaces; near-Opus on coding |
| Haiku 4.5 | 1 / 5 | Search, explore and report subagents |

Effort has five levels: `low`, `medium`, `high`, `xhigh`, `max`. This repo
opens at `medium`, and effort does not follow a model switch, so a session
that climbs the ladder sets `/effort` as well as `/model`. `xhigh` is the
documented sweet spot for lead-session coding on judgement-dense work; `high`
for patterned coding, planning, docs and review synthesis; `max` only when correctness beats cost
outright (a migration touching shipped data, a blob-graph change), since it can
overthink routine work; `low` and `medium` for mechanical work and gate
runners. `/model` and `/effort` change the running session, and both persist
to settings unless chosen as session-only. Fast mode (Opus only) is priced at
Fable's rate: use it when interactive latency genuinely matters, never as an
economy measure, and for sensitive work Fable at normal speed is better value.

**The activity ladder.** A piece of work passes through up to six
activities, and the rung follows the activity, not the file count. Read the
row for what you are doing now; a task that spans rows changes rung at the
boundary, saying so.

| Activity | Decide | Build or run | Runs in | Why this shape |
|---|---|---|---|---|
| Technical design: the `Event`/`Effect`/`ViewModel` shape, anything crossing the bridge, schema strategy, auth | Fable 5.1 `xhigh`; `max` for a migration or the `ActiveSession` blob graph | Opus 5 `high` against a reviewed spec that fixes the shape field by field, escalating any choice it leaves open | The lead, one session; the shape is written into the spec or issue before either side is wired | Fails silently (#846, #1223, #1345): the strongest rung decides, and no Sonnet tier |
| Visual design: a new flow against `design-principles.md`, a screen mocked in Claude Design | Opus 5 `high`; Fable 5.1 `high` only for a T-numbered decision or a flow with no precedent in the app | Opus 5 `high` for the mocks; Sonnet 5 `medium` for bookkeeping (`DesignSync`, tokens, filing the decision) | The lead, or a short session of its own | Fails visibly on the simulator, so a wrong call is cheap to see and cheap to redo |
| Planning: direction, slice plans, specs | Fable 5.1 `high` for direction (roadmap pivots, reversals, "should we build this at all", Tier 3 specs); Opus 5 `high` for a slice inside a settled direction | Sonnet 5 `medium` for the mechanics: issues from an agreed plan, handover openers | The lead, plan mode; on a model no weaker than the build it plans | A plan is where the judgement concentrates; the build that follows it is patterned |
| Tasks: writing the code | Opus 5 `xhigh` for core work with real judgement (new events and handlers in `intrada-core`, TDD-first) and judgement-dense screens | Sonnet 5 `high` for conventional Tier 2 on a non-sensitive surface, via `task`; Tier 1 trivia at Sonnet 5 `low` in the lead, or Haiku 4.5 via `smol` when fully specified | The judgement-dense half in the lead, since `task` is pinned to Sonnet and the spawn guard refuses a lift; the patterned half as a subagent from the session that planned it, once the plan is settled; a new session only to free the lead or on escalation | Sonnet is near Opus on patterned coding at a third of the cost; the pattern is already in the repo |
| Review | `reviewer` is pinned to Sonnet 5 `high`, which covers Tier 1 and screens-only diffs; a diff touching `crates/intrada-ffi`, a migration, `ActiveSession` or auth spawns `reviewer` with `model` set to Opus instead (Fable for Fable-written bridge or migration work), the one lever that beats a definition's pin | `/code-review` inline for a small Tier 2 on one file with no sensitive surface | `reviewer` as a subagent; the Opus or Fable review and `/code-review` in the lead; on a multi-surface slice, the core half before the screens half starts | Sonnet covers the non-sensitive majority (52 runs, 12 to 14 September, $77 on mostly Tier 1); the strongest rung stays reserved for the silent-failure surfaces |
| Gates and research | | `test-runner` (Haiku 4.5 `low`) for every gate; `Explore` with `model: haiku` for fan-out that reports facts back | A subagent, always | No judgement in the job; the gate or the lead's verification is the check |

The worst debugging (bincode wire breaks, silent no-ops, "green but wrong"
tests, which got past Opus-era sessions three times) is technical design
wearing a different hat: Fable 5.1 `xhigh`. A task with both design halves
splits, contract first at the top of the ladder: the two-PR rule again.

**What the rungs cost in money and time.** Measured over the fortnight to
2026-09-14 from the session transcripts on Jon's machine, main sessions only.
Latency is the gap between the previous transcript line and the model's reply,
so it includes thinking; medians are per request, not per task.

| Rung | Turns | $ per turn | Median s | p90 s |
|---|---|---|---|---|
| Fable 5.1 xhigh | 442 | 0.31 | 12.0 | 40.6 |
| Fable 5.1 high | 666 | 0.20 | 9.3 | 23.4 |
| Opus 5 xhigh | 918 | 0.16 | 5.7 | 18.9 |
| Opus 5 high | 1401 | 0.15 | 5.9 | 17.8 |
| Opus 5 medium | 1335 | 0.15 | 5.2 | 15.0 |
| Sonnet 5 high | 70 | 0.06 | 4.8 | 29.4 |
| Sonnet 5 medium | 483 | 0.05 | 3.4 | 8.2 |

Three things follow, and the ladder above is shaped by them:

- **On Opus, effort is close to free.** `medium`, `high` and `xhigh` cost
  within the noise of each other per turn, which is what context length
  dominating the bill predicts, and the median reply time is flat; p90 rises
  about a quarter from `medium` to `xhigh`. An Opus session sits at `high` by
  default and goes to `xhigh` for judgement-dense work without a cost case
  against it.
- **On Fable, effort is the lever.** `xhigh` costs half as much again per turn
  as `high` and lengthens the slow tail by three quarters, so it buys the
  deciding half of a task and hands the building half down. Fable at every
  measured effort is slower than Opus at every measured effort: use it where
  deliberation pays, not on interactive back-and-forth.
- **Sonnet is where the saving is.** A third of Opus per turn and the fastest
  reply. Sonnet `high` has a slow tail from a small sample, so treat its p90
  as unknown until the turn count is in the hundreds.

**Rules of thumb.** The sensitivity override applies to models: auth, bridge,
schema and migration work jumps a model-and-effort level whatever the file
count. Drop effort before dropping model. Decisions go up the ladder,
execution goes down it. A subagent's finding is a lead, not a fact: one on
2026-09-04 blamed the wrong commit, cited a line that pointed at a comment,
and said it could not run `git show` when it could. Brief research agents to
mark observed against inferred, and verify before acting.

**What it costs in practice.** Measured over the fortnight to 2026-09-13 from
the session transcripts on Jon's machine, all projects, weighted at API prices;
`just usage 14` prints the same table. Bullets naming a week come from
`just usage`, whose default window is seven days.

- **Context length drives usage more than the rung.** 52 of 154 main sessions
  passed 200k tokens, and the eight biggest sessions were 36% of the fortnight.
  `/compact` when mid-task. The status line shows the context in thousands,
  amber from 200k and red from 400k.
- **A parked session pays again.** The prompt cache lasts an hour; the first
  turn after a longer gap re-sends a context past 50k at write price. In the
  week to 2026-09-13 that was 194 turns and $233, and cache writes were 29% of
  that week's spend. The context watch names such a turn as it happens: finish
  the unit and `/clear` in the same sitting rather than leave a session to come
  back to.
- **Tool output is what fills the context.** 36 MB in the week to 2026-09-13,
  Read 31%, then git, sed, grep, just and cat at 5 to 10% each; `just usage`
  prints the table. Read once, keep ranges tight, and run gates in
  `test-runner`.
- **Per-turn prices are in the rung table above.** Fable's cache reads cost
  $0.25 per MTok against Opus's $0.50, which keeps the gap to Opus small at
  long context; dropping to Sonnet is what saves. This repo's
  `.claude/settings.json` opens sessions on Sonnet 5 medium with the 1M
  window, so name the rung the ladder gives the task and switch with `/model`
  and `/effort` before the work starts, saying so, not after.
- **Subagents were 26% of the fortnight's usage.** Their largest line was
  `task` spawned with `model: opus` over its Sonnet pin: keep the judgement
  in the lead and hand down settled work instead of lifting the agent. In the
  fortnight to 2026-09-14 (`just usage 14`, all projects) `task` runs at
  Sonnet `xhigh` and Opus `xhigh` were $353, all from spawns lifting the pin
  before the spawn guard existed; the guard now refuses both a model lift and
  an effort lift above a pinned definition, so the lead sets neither at the
  spawn for a pinned agent. It cannot see a built-in with no definition file
  at all (`Explore`, `fork`, `general-purpose`), so that cap is on the
  spawning session, not the guard: keep it at `high` or below there too.
  `task` runs grow as long as
  main sessions (59% of their turns past 200k), so brief one slice per spawn.

## Plans ship their own resourcing

Every plan (slice plan, spec phase breakdown, handover) names, per task:

1. **Which activity it is** (technical design, visual design, planning,
   tasks, review, gates and research), because the rung follows from the row.
2. **Model and effort**, from the ladder above. "Then build the screen" without
   "Sonnet 5, high" forces the next session to re-derive the routing, or
   default upward. An opener to a new session starts with the two commands
   themselves, `/model` and `/effort`, on their own lines above the pasted
   text, so the new chat never opens on the default by accident.
3. **Where it runs**: this session, a new session, or a subagent; one fresh
   session per task unless stated.
4. **Parallel streams**: two by default, at most one of them touching
   `crates/intrada-core` or `crates/intrada-ffi`; a third only if it touches
   `ios/` and no crate and has named the screens it owns (#1839); serialisation
   points named explicitly ("B after A"); one stream running iOS tests at a
   time, which costs little to queue behind (a fast-tier run is well under a
   minute either way, #1621).

A plan without activity, model, effort and stream annotations is incomplete, the same way
a phase without a test plan is.

## Delegating

| Job | Agent | Pinned | Because |
|---|---|---|---|
| Read-only research | `Explore` (built-in) | Pass `model: haiku` at the spawn; it has no definition to pin, so it inherits the spawning session's effort (below) | Reports facts back; a wrong answer is caught by the lead verifying it |
| Mechanical, fully specified edits | `smol` | Haiku 4.5, low | The decision is already made; the cheapest rung that types accurately |
| Run a gate and filter its log | `test-runner` | Haiku 4.5, low | No judgement; the gate itself is the check |
| Review a diff or a plan | `reviewer` | Sonnet 5, high; lifted to Opus (Fable for Fable-written work) on `crates/intrada-ffi`, a migration, `ActiveSession` or auth | Sonnet covers the non-sensitive majority; the strongest rung stays reserved for the silent-failure surfaces |
| Conventional Tier 2 slice | `task` | Sonnet 5, high | Non-sensitive surface, patterns already in the repo |

All four definitions live in `.claude/agents/`, so they are reviewed like code
and travel with the checkout. Pin model and effort in the definition rather than
at the spawn; the exceptions are `Explore`, `fork` and `general-purpose`, which
have no definition to pin, and lifting `reviewer` to Opus on the sensitive
surfaces, or to Fable for Fable-written work. Because a subagent with no
`effort:` in its definition inherits the parent session's effort, `Explore`,
`fork` and `general-purpose` should only be spawned from a session at `high`
or below (#1838).

`task` sits at high, not xhigh: the effort premium buys little on work that
follows a pattern already in the repo, and over the fortnight to 2026-09-13
context length dominated its cost either way (59% of its turns past 200k).

Four rules on top:

- **One agent per vertical slice.** Core and iOS are one job. Fan out only on
  genuinely independent pieces, and the lead integrates
  (`.claude/skills/intrada-parallel-streams/SKILL.md`).
- **A cheap agent needs acceptance criteria that can fail** tomorrow, not just
  now. A mechanical-edit agent once wired a binary path to a per-shell
  directory that was gone with the shell; every criterion it was given was
  true at the moment of checking.
- **The sensitivity override applies to subagents.** One touching the bridge, a
  migration, the blob or auth goes up a rung, or the lead keeps that slice.
- **Gates run through `test-runner`, never in the lead, and never inside
  `task`.** A failing suite prints thousands of lines that are re-sent on
  every later turn. `task` never runs `just check`, `just ios-test`,
  `just ios-test-full` or `cargo test` (`guard-bash.sh` denies them inside a
  `task` transcript); it confirms the build compiles and hands back a diff,
  and the lead runs `test-runner` once, at the end. `task` also stops and
  reports after 60 turns or its second failed build, and refuses a brief
  naming more than ten files as too large for one spawn (`.claude/agents/task.md`).
- **The lead verifies a task's result through `reviewer` and the gate
  counts, never by re-reading its files.** The report shape in
  `.claude/agents/task.md` is what the lead checks the work against. It opens
  a file the task touched only to act on a specific `reviewer` finding, and
  then by range, not the whole file.
- **Brief a task by file and line, not by area.** Naming the exact lines to
  change lets the task edit in place instead of opening a full read to
  relocate its own work; a `git grep` for a sentinel value beats reading a
  file end to end when the brief cannot cite a line number directly.

## Isolating concurrent work

Every session that edits works in its own worktree, whether or not another is
running, and makes it itself from the main checkout Jon started it in
(CLAUDE.md, Always step 3). Before a second stream, read
`.claude/skills/intrada-parallel-streams/SKILL.md` for the decoupled file set
and the serialisation points.

`just worktree-new <name>` branches from fresh `origin/main` and seeds the warm
`target/` and `ios/build` caches (#1205). Worktrees live at
`$INTRADA_WORKTREE_ROOT`, default `../intrada-worktrees`. `just worktree-rm
<name>` removes one and deletes its throwaway simulator. `graphify-out/` exists
only in the main checkout: query it there by path from a worktree
(`docs/reference.md`), never move the work there. Once you have a worktree,
edit only inside it.

`just worktrees` lists every worktree with its branch, uncommitted file count
and the session that holds its lease. Read it before touching a worktree you did
not create: a clean tree at main is not evidence that it is free. What the lease
claims, when it is taken and released, and what the guard allows in each of the
three places a session can be, are all in [`docs/worktrees.md`](worktrees.md).
In short: main lets anyone read and build but nobody edit, your own worktree is
yours entirely, and someone else's is readable and nothing more.

Run from a cmux terminal, `just worktree-new` prints the `cmux new-workspace`
command for the new worktree, because the sidebar shows the branch and PR of the
directory a session started in. It never starts a session itself (#1720): which
client you work in is your choice, not the recipe's.
`INTRADA_WORKTREE_CMUX=0` silences the suggestion.

**A session in the main checkout drives its worktree without restarting**, from
2026-09-12, and that is the only shape in use: Jon starts every session in
main, and a session never hands him a worktree command or a new chat to open
in a worktree (#1837; over 12 to 14 September the three rules that said
otherwise cost eleven corrections). The path-scoped rules in `.claude/rules/`
load only under the directory a session started in, so the session reads the
rules for the files it is about to touch by hand, or it is working blind.

The mechanism is the `cd` prefix, and the `EnterWorktree` tool is still banned
here: it marks the session isolated and the bash guard then refuses every
version control command, so that session can never commit. The shape the
prefix must take, the self-heal that adds it when the session holds exactly
one worktree lease (#1840), and what happens on a machine whose hooks predate
all this, are in [`docs/worktrees.md`](worktrees.md) and not repeated here.

A subagent inherits none of this. Its brief names the worktree by absolute
path, or its first edit lands in main and is denied, and `reviewer` diffs the
wrong tree (#1861). The four definitions in `.claude/agents/` say so, and the
brief still has to supply the path.

## Build and test control

The rules on driving iOS through the `just` recipes, running `just check`
before pushing, and the machine-global simulator are in `CLAUDE.md`. Beyond
those: `just check` and `just ios-test` skip on an already-green HEAD (delete
`target/.check-stamp` or `ios/build/.ios-test-stamp` to force a run);
`scripts/ios-sim-lock.sh` serialises `just ios-test`/`ios-test-full` runs
machine-wide, waiting rather than refusing while another agent's run holds it
(#1622), and each run shuts down the sim it booted when it finishes; `just
status` reads GitHub for what is in flight.
Simulator workflow in full: [`ios-testing.md`](ios-testing.md).

## Guardrails already in place

These run whether or not an agent read the rules.

- **Format on edit.** `.claude/settings.json` runs `rustfmt` on every `.rs` and
  `swift format` on every `.swift` the agent writes, skipping `ios/generated/`.
  Hooks load at session start, so run `just fmt` and `just ios-fmt` if
  unformatted code reaches CI.
- **Repo git hooks.** `scripts/install-git-hooks.sh` points `core.hooksPath` at
  `.githooks/`, run by a SessionStart hook. Pre-push refuses a push to a
  merged-PR branch and runs the comment-density and dash checks.
- **Dash check.** `scripts/check-dashes.sh` fails on em and en dashes in changed
  lines, in CI and pre-push. `SKIP_DASH_CHECK=1` for a justified case.
- **Link check.** `scripts/check-links.sh` fails on a markdown link added to a
  path that does not exist, and on a file the branch renames or deletes that
  something still links to, in `just hygiene` and CI. It reads the working
  tree, so an uncommitted link counts; the first half reads changed lines only,
  so a link already broken in a file nobody touched still ships. External URLs
  are skipped. `SKIP_LINK_CHECK=1` for a justified case.
- **Release name check.** `scripts/check-release-name.sh` fails when the app and
  the TestFlight lane stop composing the same Sentry release name, which would
  otherwise show up only as crashes filed under a release nobody created.
- **Permission deny list.** `.claude/settings.json` denies `gh pr merge`,
  `git push origin main`, `fly`, `just testflight` and the destructive
  simulator resets outright. Branch protection on `main` backs the first two
  server-side.
- **On Jon's machine, not in the repo**: `~/.claude/hooks/guard-bash.sh`
  matches the same denials inside compound commands, the turn reminder
  re-attaches the style rules on every prompt, and the post-push hook restates
  the CI-watch rule after every push. `guard-spawn.sh` refuses a spawn that
  lifts a subagent above the model in its definition (`reviewer` excepted, per
  the Review row of the activity ladder), `context-watch.sh` warns at 150k,
  hands the rest of the unit to a `task`/`smol` subagent at 200k and at 400k
  says to `/compact` now (`CONTEXT_WARN` and `CONTEXT_FIRM` stay overridable
  env vars), and `usage-daily.sh` opens the first session of each day with one
  line from `usage-report.py`. `read-guard.sh` denies a `Read` with no offset,
  limit or `pages` when this session's transcript already holds that exact
  path with an unchanged mtime, and, separately, denies one on a text file
  over 400 lines; images are exempt from both (#1844).

## Session controls

| Want | Do |
|---|---|
| The pre-push funnel | `/ship`: gates through `test-runner`, then `reviewer`, then the PR |
| Review the current diff | `/code-review` (the code-review plugin skill), at a chosen depth. Say "comment-policy violations are Blockers" or they survive as nits |
| Change rung mid-session | `/model`, `/effort`. Both persist unless chosen as session-only |
| See what loaded | `/context` lists the memory files and rules in this session |
| See what sessions cost | `just usage` (last 7 days by agent, model and effort, plus the biggest sessions); `just usage 14` for a fortnight |
| Read the context nudges | `context-watch.sh` nudges unprompted as context grows: at 150k finish the slice and hand the rest to a `task`/`smol` subagent (this session checks and ships), at 200k hand it over now, at 400k `/compact` |
| Keep the thread, drop the bulk | `/compact`. Decisions survive, tool output goes, the path-scoped rules reload on the next Read. Mid-task only |
| End the unit | `/clear`, once the PR is green, the issue closed and the worktree removed. The transcript file survives it, which is why the read guard can deny the next unit's first read (#1866) |
| Free the session or change the driver | A new chat, opened in the main checkout, from an opener that names the issue, activity, model, effort and stream. Never for a rung change alone: `/model` and `/effort` do that in place |
| Edit the rules files | `/memory` |
| Tidy up permission prompts | `/fewer-permission-prompts` |

The **Superpowers plugin is disabled**, deliberately: its "invoke a skill for
anything" posture fights the tier system. Four of its skills were kept as
standalone globals: `test-driven-development`, `requesting-code-review`,
`receiving-code-review` and `using-git-worktrees`. `/speckit-*` is deprecated.

## Verification, the standard held

- UI changes: drive the simulator and show proof, or say explicitly "I cannot
  reach the running app, here is what you need to verify by hand". Never claim
  "all green" when that means `cargo test` passed.
- Report failures faithfully with the actual output. No "should work".
- Never report a gate as broken without making it fail.
- After any push to an open PR, watch the run to a conclusion in the same turn.
  Read the PR's mergeability, not just the job list: a renamed job leaves the
  old required context "expected" for ever, which is a hang, not a failure,
  and is invisible in the checks list (#1542).

## Shipping

The gate funnel, Codecov expectations, the deferred-issue protocol and the PR
and issue body templates are in `.claude/skills/intrada-shipping/SKILL.md`,
which `/ship` follows. Stacked PRs are supported: open the child with base set
to the parent's branch, depth 2 maximum; CI runs on them because `ci.yml` has
no branch filter on `pull_request`. **Agents never merge.**

## Code intelligence

Run `just lsp-setup` once per machine, and once per worktree that wants Swift.
Without it neither LSP answers anything: `rust-analyzer` on PATH is a rustup
shim that fails unless the component is installed for the pinned toolchain, and
`sourcekit-lsp` never activates because its root markers live under `ios/`
while a session starts at the repo root. The recipe also does more than write
the build server: `xcode-build-server config` alone aims the index at Xcode's
default DerivedData, which this repo never writes to, so sourcekit-lsp reports
`No such module` on code that compiles. The recipe parses a real indexing
build's log instead, so diagnostics are honest, hover resolves and
jump-to-definition works across files. `references` still returns nothing,
which is a sourcekit-lsp limitation, so finding the callers of a Swift symbol
stays a grep job.

## Worked examples

Both are real issues and both start the same way: claim the issue and stop if
a PR already exists.

```bash
gh pr list --repo jonyardley/intrada --state open --search "<N>"
gh issue view <N> --json closedByPullRequestsReferences
```

### Small: #1426, a hand-rolled primitive

`ReflectionSheet.swift` builds an eyebrow by hand with `kerning(1.2)` where the
`Eyebrow` primitive uses `tracking(1.5)`, so the sheet's labels are visibly
tighter than the twenty other eyebrows in the app, and the hand-roll drops
`Eyebrow`'s un-uppercased `accessibilityLabel`, so VoiceOver reads the shouty
version.

Tier 1. One file, no bridge, no schema, no auth, so no override applies.

1. One session, no plan mode, no subagents. Sonnet 5 at `low` is the rung, but
   on a change this small the ceremony of moving there costs more than it saves.
2. Reading the file loads `.claude/rules/ios-ui.md`: reuse before creating,
   never hand-roll something that exists.
3. Replace the hand-roll with `Eyebrow`. Check the neighbouring `.badge`
   hand-roll on the same screen, and if it is a different primitive leave it and
   say so rather than widening the change silently.
4. Re-record the three affected references in one pass, then verify:
   ```bash
   just ios-snapshots-record ScreenSnapshotTests/testReflectionSheet
   just ios-fmt-check && just ios-test
   ```
   Recording is delete-then-run-twice by design, so a first-run failure is
   expected. Read the diff: the labels should get looser, not move.
5. Ship. Tier 1 trivia may skip the review agent but still runs the gates.

Opener:

```text
Claim #1426 and stop if a PR already exists, then fix it.

ReflectionSheet.swift hand-rolls an eyebrow at line 174; use the Eyebrow
primitive instead. There is a second hand-roll at line 87 on the badge font:
that is a different primitive, so leave it and flag it rather than widening
this change.

Re-record the affected references with just ios-snapshots-record, then
just ios-fmt-check and just ios-test. Tell me which snapshots moved and why
before opening the PR.
```

### Larger: #1512, a bound the shell should not own

`ClickSheet.swift` hard-codes `2...12` and `EntrySettingsSheet.swift` hard-codes
`3...10`, both mirroring constants in `crates/intrada-core/src/validation.rs`.
When the core's bound moves, the sheet keeps offering the old range and starts
sending values the core rejects, and the write is refused with nothing on
screen. That is the swallowed-update failure the offline-first rule exists to
prevent.

Tier 2 on file count, but projecting a bound through the `ViewModel` changes the
bridge contract, so the domain-sensitivity override puts it up a tier.

1. Contract before code, at the top of the ladder: Fable 5.1 at `xhigh`. Pin
   the `ViewModel` shape first, in one session, and write it down before either
   side is wired. `max` is reserved for migrations.
2. This touches `crates/`, so it is the one crate-touching stream this repo
   allows at a time. Do not fan out, and do not start a second stream that
   touches `crates/` while it is in flight; a shell-only stream elsewhere that
   has named its own screens may still run.
3. Core PR first. TDD is the default for `intrada-core`: write the failing test,
   then project the bounds. Extend the Rust `assert_round_trips` helper to the
   new view type before any screen reads it, because a stub-bridge test cannot
   catch a bincode wire break (#846).
4. Get the core PR reviewed before starting the screens. On a multi-surface
   slice, one review at the end is too late to be cheap.
5. Screens PR second. Read both bounds from the `ViewModel`, delete both
   hard-coded ranges, and add the test the issue asks for: the offered range
   matches the core's. A shell constant that merely repeats the number is not
   the fix.
6. Verify on the running app, not just in CI. A core type change means iOS
   tests mean nothing until `just ios-test-full` has run.
7. Ship as two PRs, core then screens, each independently reviewable.

Opener for the first session:

```text
Claim #1512 and stop if a PR already exists. Plan mode first, and do not
write code this session beyond the contract.

This changes the bridge contract, so use the strongest rung before you decide
anything. Pin the ViewModel shape that projects MIN_METRE_BEATS/MAX_METRE_BEATS
and the rep-target bound, and write it into the issue before either side is
wired. A refused write with nothing on screen is the failure the offline-first
rule exists to prevent.

This touches crates/, so it is the one crate-touching stream running. Do not
fan out, and do not start a second stream that touches crates/ while this is
in flight. Core PR only when we implement: TDD, and extend assert_round_trips
for the new view type before any screen reads it. Screens are a second PR
after this one is reviewed.
```

Opener for the screens session, once the core PR is reviewed:

```text
Screens half of #1512, core PR #<N> is merged. Grep each file for its
hard-coded range (2...12 in ClickSheet.swift, 3...10 in
EntrySettingsSheet.swift) and replace both with the bounds read from the
ViewModel; no need to open either file in full. A shell constant repeating
the number is not the fix.

Add the test the issue asks for: the offered range matches the core's, so
widening the core cannot silently leave a sheet behind. Then just ios-test-full,
because the core type changed. Re-record any snapshots the control changes
touch, and say which.

Report back per .claude/agents/task.md: diff --stat and changed symbols, the
gate and its counts, what you left out, what you could not verify.
```

## What slows us down

- Vague approval of large scope ("do the rest") instead of finishing the current
  slice.
- Asking an agent to skip verification to save time. The rework costs more.
- Burying deferred work in PR descriptions rather than tracked issues.
- Pushing to `main`. Never; always a feature branch and a PR.

## Troubleshooting

| Symptom | Cause |
|---|---|
| A rule did not load | Rules with `paths:` fire on the Read tool only; a `cat` via Bash does not count. `/context` shows what loaded |
| A skill will not resolve | Skills are discovered at session start; restart after adding one |
| `just check` says already green | Stamp matches HEAD and the tree is clean; delete the stamp |
| Unformatted Swift or Rust reaching CI | The format hook loads at session start; run `just fmt` and `just ios-fmt` |
| PR hangs on a check that never reports | A renamed job left its old required context "expected" (#1542) |
| A subagent ran on the wrong model or effort | Its definition has no `model:` or `effort:` pin (`Explore`, `fork`, `general-purpose`), so it inherited the session's; spawn those from `high` or below (#1838) |
| The spawn guard refused a subagent | The spawn lifted `model` or `effort` above the definition's pin. Only `reviewer` may go up, on a sensitive surface. Drop the override; keep the judgement in the lead instead |
| Read denied as "already in context" on the first read after `/clear` | The guard reads the transcript, which `/clear` does not restart (#1866). Range-read (`offset`, `limit`) until it is fixed |
| Read denied on a file over 400 lines | Grep for the symbol, then read the range. The guard never allows the whole file (#1844) |
| Edit or command denied in the main checkout | No worktree yet, or the `cd <worktree> && ` prefix is missing and the session holds more than one lease so it cannot be added for you. Make the worktree, prefix the command; the shapes the guard reads are in `worktrees.md` |
| Session start says another session holds this worktree | The lease is live; read `just worktrees` and pick another. A clean tree at main is not evidence it is free |
| A subagent edited the wrong tree, or `reviewer` reviewed main | The brief did not name the worktree by absolute path (#1861) |
| XCUITests refuse to launch, `SBMainWorkspace` busy | A simulator booted outside the `just` recipes. Ask before shutting it down; CI proves the UI tests meanwhile |
| A PR's first CI run shows cancelled or a stale failed context | `gh pr create --label` fires several events that cancel each other. Label after the first run starts, or hand Jon `gh run rerun --failed` |
| The session was interrupted mid-tool | Retry in a different form and keep working. An interrupted call is not a decision point to hand back |
| A gate looks like it ran nothing | Terse output is a pass. Break one assertion and watch it go red before reporting a gate broken (2026-09-04) |

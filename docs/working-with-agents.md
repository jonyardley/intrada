# Working with Claude Code on intrada

> How one unit of work runs in this repo, in the order it runs: claim,
> isolate, route, plan, build, review, ship, measure. Each section says what to
> do at that step, what holds it, and the number it was set from. The tier
> system in [`CLAUDE.md`](../CLAUDE.md) is normative and not restated. Harness,
> process, isolation, routing, rung, context, speed, rework and spend are
> defined in the glossary in [`reference.md`](reference.md). This file replaced
> the OMP-era guides and `model-guide.md` on 2026-09-08; why OMP was retired is
> in [`reference.md`](reference.md) too.
>
> Last reviewed: 2026-09-15, against the Claude 5 family (Fable 5.1, Opus 5,
> Sonnet 5, Haiku 4.5). Re-review at the next model generation.
>
> The generic version of this file, with the intrada names taken out, is
> [`agentic-primer.md`](agentic-primer.md): read that to set up a new repo,
> this to drive this one.

## The steps at a glance

| Step | The move | Held by |
|---|---|---|
| 1. Claim | `just claim N`, and stop if a PR already exists | `scripts/claim-issue.sh`, `just pr-open` |
| 2. Isolate | `just worktree-new <name>`, then `cd <worktree> && ` on every command | The worktree guard and its lease |
| 3. Route | Opus 5 `xhigh` in the lead; Fable 5.1 `high` only for the Fable list | `.claude/settings.json`, the agent pins, the spawn guard, the turn reminder |
| 4. Plan | Tier 2 and 3: a plan comment on the issue, approved by Jon | CLAUDE.md Workflow; `reviewer` checks the diff against it |
| 5. Build | Settled slices to `task`, gates to `test-runner`, each file read once | The read guard, the bash guard, the context watch |
| 6. Review | `reviewer` over the local diff, briefed with the worktree and the issue | `/ship` |
| 7. Ship | `just pr-open` as a draft, CI watched to a conclusion, a human merges | Branch protection, the deny list, the git hooks |
| 8. Measure | A dated row in `harness-log.md` per harness change; the Monday read | `just usage 7 --quality` |

Most rules below are held by a hook or a recipe named in their section. A rule
a hook can enforce loses its prose in the same change.

## 1. Claim

| You are | Do |
|---|---|
| Starting a unit of work | `just claim N`. It refuses and names the other branch when the issue is already claimed or an open PR references it; otherwise it labels the issue, comments the branch and moves the board to In progress |
| Asked "what's next" or "what can run in parallel" | Answer from `just status` and the session-start claims list, with no further reads |
| About to start a third fix on one issue | Stop and name the decision: two corrections on one point stop the change and ask if the approach is wrong before a third, and `just claim` refuses a third claim at two merged PRs (#1890) |

A claim is the check that the work is not already in flight:

```bash
gh pr list --repo jonyardley/intrada --state open --search "<N>"
gh issue view <N> --json closedByPullRequestsReferences
```

`just pr-open` refuses to open a PR whose title names an issue with no claim
on the current branch (#1702), the second way #1694 got built twice on
2026-09-11. The status rule was set from seven "what's next" sessions over 12
to 14 September, each a fresh 80k to 200k context, all picking Tier 1 work
(#1839).

## 2. Isolate

| You are | Do |
|---|---|
| About to edit anything | `just worktree-new <name>` from the main checkout, prefix every shell command with `cd <worktree> && `, and read the `.claude/rules/` files for the surfaces you will touch by hand |
| Handing work to another session | `just handover [N]` prints the opener: issue, model, effort, the claim, the branch and what it has touched. Paste it into a new chat opened in the main checkout; the opener names the worktree to carry on in when this branch already has the work, and tells that session to make its own when it does not. Never a worktree command for Jon |
| Thinking of a second stream | Read `intrada-parallel-streams` first: two by default, at most one touching `crates/`, a third only shell-only and announced |
| Spawning a subagent | Name the worktree by absolute path in the brief |

Every session that edits works in its own worktree, whether or not another is
running, and makes it itself from the main checkout Jon started it in
(CLAUDE.md, Always step 3). Over 12 to 14 September the three rules that said
otherwise cost eleven corrections (#1837), and `guard-worktree.sh` denied 51
writes in main, almost all a missing `cd` prefix, which the self-heal now adds
for you (#1840).

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
directory a session started in. It never starts a session itself (#1720).
`INTRADA_WORKTREE_CMUX=0` silences the suggestion.

**A session in the main checkout drives its worktree without restarting**, from
2026-09-12, and that is the only shape in use. The path-scoped rules in `.claude/rules/` load
only under the directory a session started in, so the session reads the rules
for the files it is about to touch by hand, or it is working blind. The
mechanism is the `cd` prefix, and the `EnterWorktree` tool is banned here: it
marks the session isolated and the bash guard then refuses every version
control command, so that session can never commit. The shape the prefix must
take, the self-heal that adds it when the session holds exactly one worktree
lease (#1840), and what happens on a machine whose hooks predate all this, are
in [`docs/worktrees.md`](worktrees.md).

A subagent inherits none of this. Its brief names the worktree by absolute
path, or its first edit lands in main and is denied, and `reviewer` diffs the
wrong tree (#1861). The three definitions in `.claude/agents/` say so, and the
brief still has to supply the path.

**Two streams by default** (#1839), set from 34 simulator-busy hits across 20
sessions. Before a second stream, read
`.claude/skills/intrada-parallel-streams/SKILL.md` for the decoupled file set
and the serialisation points.

## 3. Route

| You are | Do |
|---|---|
| Opening a session | Name the rung and effort before the first edit and at every boundary: Opus 5 `xhigh`, which the session opens on, unless the work is on the Fable list |
| Reaching work on the Fable list | `/model` and `/effort` in this session before the work starts, saying so, and back to Opus 5 `xhigh` at the boundary. A new chat frees the lead; it is never how a rung changes |
| Handing work down | Settled work goes to a subagent from one lead (step 5); a new chat is for freeing the session, never for work the lead could dispatch |

The new-chat rule was set from twelve of 76 sessions over 12 to 14 September
that did nothing but status or handover (#1836).

Three rungs, set on 2026-09-15 (#1897): Fable thinks, Opus builds, Haiku runs.
The lower rungs cost more of Jon's time in corrections and loops than they
saved in spend, read from the 11 to 14 September sessions (#1893), so nothing
with judgement in it runs below Opus.

Match the **model** to how silently wrong the work can go. The silent-failure
surfaces here are the FFI bridge (positional bincode: wrong is a no-op, not a
crash, #846), local GRDB migrations (the device is the only copy of the user's
data), the `ActiveSession` crash-recovery blob (#1223, #1244, #1256) and auth.
Everything else degrades gracefully because failure is visible: a wrong layout
is caught on the simulator, a wrong test fails in CI.

| Model | $/MTok in/out | Use for |
|---|---|---|
| Fable 5.1 | 10 / 50 | The Fable list, and nothing else |
| Opus 5 | 5 / 25 | Everything with judgement in it: the lead, `task`, `reviewer` |
| Haiku 4.5 | 1 / 5 | `test-runner` and `Explore`, nothing else |
| Sonnet 5 | 2 / 10 | Retired from this repo |

**The Fable list.** Switch the lead to Fable 5.1 `high` for these, with
`/model` and `/effort`, saying so, and back to Opus 5 `xhigh` at the boundary:

- The plan comment on a Tier 2 or Tier 3 issue.
- The shape and the build of anything crossing the bridge, a schema change or
  migration, the crash-recovery blob or auth. The sensitivity override puts this work on the
  list whatever the file count.
- A direction call: a roadmap pivot, a reversal, "should we build this at all".
- The bug nobody can explain: a bincode wire break, a silent no-op, a test
  green for the wrong reason.

**The rungs, with the human equivalent.**

| Rung | Human equivalent | Ask them for |
|---|---|---|
| Fable 5.1 `high`, thinks | The architect you pull into a design review. Sets the shape; builds only the sensitive surfaces on the Fable list | The plan comment, the bridge or migration shape, the direction call, the bug nobody can explain |
| Opus 5 `xhigh`, the lead | A strong senior engineer who owns the ticket end to end | The build, within a shape already agreed; they call the architect when the contract is in doubt |
| Opus 5 `high`, `task` | The same senior engineer working alone on a branch from a written ticket | One slice, reported back as a diff; you do not sit with them while they type |
| Opus 5 `high`, `reviewer` | A peer on the PR | Reads the diff, not the description; never merges |
| Haiku 4.5 `low`, `test-runner` | CI on your desk | Runs the gate, names what failed, has no opinion |
| Haiku 4.5, `Explore` | A new starter sent to find where something lives | File names and line numbers; a lead, not a fact |
| Sonnet 5, retired from the lead | A capable mid-level engineer who needs the pattern shown | Nothing that needs judgement: the time spent correcting is the rate saved |

**Effort** is how long you would let them think before answering: `low` is a
reply in the corridor, `medium` a minute at the desk, `high` worked through on
paper, `xhigh` slept on and back with the trade-offs, `max` a spike or a design
note. On Opus the longer think is nearly free per turn; on Fable it is the
bill. Tiers map the same way: Tier 1 is a fix you would just do, Tier 2 a
ticket with a plan attached, Tier 3 a design note before anyone builds.

Effort does not follow a model switch, so a session that changes rung sets
`/effort` as well as `/model`. Both change the running session, and both
persist to settings unless chosen as session-only. Fast mode (Opus only) is
priced at Fable's rate: use it when interactive latency genuinely matters,
never as an economy measure.

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

Three things follow, and the rungs above are shaped by them:

- **On Opus, effort is close to free.** `medium`, `high` and `xhigh` cost
  within the noise of each other per turn, which is what context length
  dominating the bill predicts, and the median reply time is flat; p90 rises
  about a quarter from `medium` to `xhigh`. The lead sits at `xhigh` for that
  reason, and `task` and `reviewer` at `high`.
- **On Fable, effort is the lever.** `xhigh` costs half as much again per turn
  as `high` and lengthens the slow tail by three quarters, so Fable works at
  `high`, and away from the sensitive surfaces it decides and hands the
  building back to Opus. Fable at every
  measured effort is slower than Opus at every measured effort: use it where
  deliberation pays, not on interactive back-and-forth.
- **Sonnet saved on paper only.** A third of Opus per turn and the fastest
  reply, but the corrections and loops on the lower rungs cost Jon more time
  than the rate saved (#1897), so it is retired from this repo.

**What a session carries.** A session pays a fixed context bill before the
first word, and a subagent pays most of it again. Everything in the first
column below is in that bill.

| Input | Path | When it loads |
|---|---|---|
| Project rules | `CLAUDE.md` | Every session and every subagent, in full. Under 200 lines by rule |
| Path-scoped rules | `.claude/rules/*.md` | When a file matching the rule's `paths:` globs is read with the Read tool. A `cat` through Bash does not count, and a file inside a worktree driven from the main checkout does not fire them either: read them by hand there. They reload the same way after compaction |
| Skills | `.claude/skills/*/SKILL.md` | The description every session; the body when invoked by name (`/ship`, `/intrada-parallel-streams`) |
| Agents | `.claude/agents/*.md` | The description every session; the body becomes the subagent's system prompt |
| Repo settings | `.claude/settings.json` | The model and effort a session opens on (Opus 5 `xhigh`, 1M window), permissions, the format-on-edit and git-hook-install hooks, and the plugins switched off for this repo |
| User rules | `~/.claude/CLAUDE.md`, `~/.claude/rules/` | Every session, before the project rules; project wins on conflict |
| Auto memory | `~/.claude/projects/<project>/memory/MEMORY.md` | Every session, first 200 lines. Not loaded into subagents. The one input that can carry a stale fact |
| User hooks | `~/.claude/settings.json`, `~/.claude/hooks/` | Text on every prompt (the turn reminder and the context watch), after a push (the CI-watch note) and in the day's first session (one usage line); before a tool runs, the worktree guard, the bash guard, the read guard and the spawn guard |
| Plugins | `enabledPlugins` in user settings | Each plugin skill's description, every session. `.claude/settings.json` switches slack, atlassian, visual-explainer and frontend-design off here |
| Xcode tools | `.mcp.json` | Xcode's tool server (`xcrun mcpbridge`) for driving the running app on an iOS 27 simulator, rendering previews and Apple docs search; builds, tests and project edits are denied in `.claude/settings.json` |

The `~/.claude/` rows above move whole if `CLAUDE_CONFIG_DIR` is set: a session
on such a machine reads `$CLAUDE_CONFIG_DIR/CLAUDE.md`,
`$CLAUDE_CONFIG_DIR/settings.json` and
`$CLAUDE_CONFIG_DIR/projects/<project>/memory/` instead, and adding a hook to
`~/.claude/settings.json` there does nothing (#1857). Run `/context` in a
session to see exactly which files loaded.

The **Superpowers plugin is disabled**, deliberately: its "invoke a skill for
anything" posture fights the tier system. Four of its skills were kept as
standalone globals: `test-driven-development`, `requesting-code-review`,
`receiving-code-review` and `using-git-worktrees`. `/speckit-*` is deprecated.

## 4. Plan

| You are | Do |
|---|---|
| Holding a Tier 2 or Tier 3 issue | Switch to Fable 5.1 `high` and write the plan comment on the issue before the first commit; wait for Jon's approval |
| Holding a Tier 1 fix | No plan comment; build it |
| Handing a task to a new session | `just handover [N]`, which starts the opener with the `/model` and `/effort` lines and names the issue; add the stream by hand when there is more than one |

A Tier 2 or Tier 3 plan lives on its issue as a comment of about 150 words:
done looks like, decisions taken, files and lines, tests, out of scope,
routing. Jon approves it, the build reads it, and `reviewer` is briefed with
the issue so it checks the diff against done looks like. A plan that runs long
is the signal the issue is really Tier 3, which adds a `specs/<feature>.md`
riding as the first commit of Phase A.

The plan names the epic the issue sits under, which `just claim` prints. Any
issue the plan creates that belongs to a body of work goes under its epic the
same turn, in working order, with `just epic-add`; three or more new issues with an order and no epic to
fit are a new epic ([Epics](roadmap.md#epics)).

Every plan (plan comment, spec phase breakdown, handover) names, per task:

1. **Whether it is on the Fable list**, because nothing else moves the rung
   off Opus.
2. **Model and effort**, from the rungs above. "Then design the migration"
   without "Fable 5.1, high" forces the next session to re-derive the routing,
   or stay on the default. An opener to a new session starts with the two
   commands themselves, `/model` and `/effort`, on their own lines above the
   pasted text, so the new chat never opens on the wrong rung by accident.
3. **Where it runs**: this session, a new session, or a subagent; one fresh
   session per task unless stated.
4. **Parallel streams**: two by default, at most one of them touching
   `crates/intrada-core` or `crates/intrada-ffi`; a third only if it touches
   `ios/` and no crate and has named the screens it owns (#1839); serialisation
   points named explicitly ("B after A"); one stream running iOS tests at a
   time, which costs little to queue behind (a fast-tier run is well under a
   minute either way, #1621).

A plan without model, effort and stream annotations is incomplete, the same way
a phase without a test plan is. Vague approval of large scope ("do the rest")
instead of finishing the current slice is what slows a plan down.

### Worked plans

Both are real issues and both start the same way: claim the issue and stop if
a PR already exists.

**Small: #1426, a hand-rolled primitive.** `ReflectionSheet.swift` builds an
eyebrow by hand with `kerning(1.2)` where the `Eyebrow` primitive uses
`tracking(1.5)`, so the sheet's labels are visibly tighter than the twenty
other eyebrows in the app, and the hand-roll drops `Eyebrow`'s un-uppercased
`accessibilityLabel`, so VoiceOver reads the shouty version.

Tier 1. One file, no bridge, no schema, no auth, so no override applies.

1. One session, no plan comment, no subagents. The lead's Opus 5 `xhigh` is the
   rung; nothing here is on the Fable list.
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

**Larger: #1512, a bound the shell should not own.** `ClickSheet.swift`
hard-codes `2...12` and `EntrySettingsSheet.swift` hard-codes `3...10`, both
mirroring constants in `crates/intrada-core/src/validation.rs`. When the core's
bound moves, the sheet keeps offering the old range and starts sending values
the core rejects, and the write is refused with nothing on screen. That is the
swallowed-update failure the offline-first rule exists to prevent.

Tier 2 on file count, but projecting a bound through the `ViewModel` changes the
bridge contract, so the domain-sensitivity override puts it up a tier.

1. Contract before code, on the Fable list: Fable 5.1 at `high`. Pin the
   `ViewModel` shape first, in one session, and write it into the plan comment
   before either side is wired.
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
Claim #1512 and stop if a PR already exists. Plan comment first, and do not
write code this session beyond the contract.

This changes the bridge contract, so switch to Fable 5.1 high (it is on the
Fable list) before you decide anything. Pin the ViewModel shape that projects MIN_METRE_BEATS/MAX_METRE_BEATS
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

## 5. Build

| You are | Do |
|---|---|
| Holding a settled plan whose build follows a pattern in the repo | Hand each slice to `task` (Opus 5 `high`), one slice per spawn, briefed by worktree, file and line; check its report against `reviewer` and the gate counts, never by re-reading its files |
| About to run a gate | `test-runner`, never in the lead and never inside `task`; a fan-out skips the suites and the lead runs them once at the end |
| Needing facts from many files | `Explore` with `model: haiku`; its findings are leads, not facts |
| Looking at a file for the second time | grep, then a range read. The read guard denies the whole file, whether the Read tool or a bare `cat`, `sed`, `head` or `tail` asks for it (#1986), and it also denies the first read after `/clear` of a file the session read before it (#1866): range-read round that |
| Warned at 150k context | Finish the slice in hand; everything after it goes to `task`, and this session checks and ships |
| At 200k | The rest of the unit goes to a subagent now |
| At 400k | `/compact` now, then carry on to the end of the unit |
| Mid-task and the thread matters | `/compact`, not `/clear`: compact keeps the decisions and drops the tool output, and the path-scoped rules reload on the next Read |

**Delegating.**

| Job | Agent | Pinned | Because |
|---|---|---|---|
| Read-only research | `Explore` (built-in) | Pass `model: haiku` at the spawn; it has no definition to pin, so it inherits the spawning session's effort | Reports facts back; a wrong answer is caught by the lead verifying it |
| Run a gate and filter its log | `test-runner` | Haiku 4.5, low | No judgement; the gate itself is the check |
| Conventional Tier 2 slice | `task` | Opus 5, high | Non-sensitive surface, patterns already in the repo |

The three definitions in `.claude/agents/` (`reviewer` is the third, step 6)
are reviewed like code and travel with the checkout. Pin model and effort in
the definition rather than at the spawn; the exceptions are `Explore`, `fork`
and `general-purpose`, which have no definition to pin, and lifting `reviewer`
to Fable on the sensitive surfaces. `guard-spawn.sh` refuses a spawn that lifts
model or effort above a definition's pin, `reviewer` excepted; it cannot see a
built-in with no definition file, so that cap is on the spawning session. The
lead opens at `xhigh`, so `fork` and `general-purpose` inherit Opus at `xhigh`
(#1838); `Explore` takes `model: haiku`, a model the `/effort` menu does not
offer `xhigh` for.
`task` sits at `high`, not `xhigh`: the effort premium buys little on work that
follows a pattern already in the repo, and over the fortnight to 2026-09-13
context length dominated its cost either way (59% of its turns past 200k).

Rules on top:

- **One agent per vertical slice.** Core and iOS are one job. Fan out only on
  genuinely independent pieces, and the lead integrates
  (`.claude/skills/intrada-parallel-streams/SKILL.md`).
- **Acceptance criteria must be able to fail tomorrow**, not just now. A
  mechanical-edit agent once wired a binary path to a per-shell directory that
  was gone with the shell; every criterion it was given was true at the moment
  of checking.
- **The sensitivity override applies to subagents.** A slice touching the
  bridge, a migration, the blob or auth stays with the lead on Fable.
- **Gates run through `test-runner`, never in the lead, and never inside
  `task`.** A failing suite prints thousands of lines that are re-sent on
  every later turn. `guard-bash.sh` denies `just check`, `just ios-test`,
  `just ios-test-full` and `cargo test` inside a `task` transcript; `task`
  confirms the build compiles and hands back a diff. It also stops and
  reports after 60 turns or its second failed build, and refuses a brief
  naming more than ten files (`.claude/agents/task.md`, #1888).
- **The lead verifies a task's result through `reviewer` and the gate
  counts, never by re-reading its files** (#1846). The report shape in
  `.claude/agents/task.md` is what the lead checks the work against; it opens a
  file the task touched only to act on a specific `reviewer` finding, and then
  by range.
- **Brief a task by file and line, not by area.** Naming the exact lines lets
  the task edit in place instead of opening a full read to relocate its own
  work; a `git grep` for a sentinel value beats reading a file end to end.
- **A subagent's finding is a lead, not a fact.** One on 2026-09-04 blamed the
  wrong commit, cited a line that pointed at a comment, and said it could not
  run `git show` when it could. Brief research agents to mark observed against
  inferred, and verify before acting.

**Read a file once.** A second look is a grep and a range, never the whole
file again; a screenshot is read once and described. `read-guard.sh` denies a
`Read` with no offset, limit or `pages` when this session's transcript already
holds that exact path with an unchanged mtime, and denies one on a text file
over 400 lines; images are exempt (#1844). Set from 981 of 1,930 text reads
over one weekend that repeated a file already in the session; one core file
was read 225 times. `rtk read` covers the rest.

**Context.** `context-watch.sh` warns at 150k, hands the rest of the unit to a
subagent at 200k and says `/compact` at 400k (`CONTEXT_WARN` and
`CONTEXT_FIRM` stay overridable env vars), set from 30 of 94 sessions passing
200k over 12 to 14 September (#1849), and set in #1848. The status line shows the context in
thousands, amber from 200k and red from 400k.

**Build and test control.** The rules on driving iOS through the `just`
recipes, running `just check` before pushing, and the machine-global simulator
are in `CLAUDE.md`. Beyond those: `just check` and `just ios-test` skip on an
already-green HEAD (delete `target/.check-stamp` or
`ios/build/.ios-test-stamp` to force a run); `scripts/ios-sim-lock.sh`
serialises `just ios-test`/`ios-test-full` runs machine-wide, waiting rather
than refusing while another agent's run holds it (#1622), and each run shuts
down the sim it booted when it finishes. Simulator workflow in full:
[`ios-testing.md`](ios-testing.md).

**Format on edit.** `.claude/settings.json` runs `rustfmt` on every `.rs` and
`swift format` on every `.swift` the agent writes, skipping `ios/generated/`.
Hooks load at session start, so run `just fmt` and `just ios-fmt` if
unformatted code reaches CI.

**Code intelligence.** Run `just lsp-setup` once per machine, and once per
worktree that wants Swift. Without it neither LSP answers anything:
`rust-analyzer` on PATH is a rustup shim that fails unless the component is
installed for the pinned toolchain, and `sourcekit-lsp` never activates because
its root markers live under `ios/` while a session starts at the repo root.
`xcode-build-server config` alone aims the index at Xcode's default
DerivedData, which this repo never writes to, so the recipe parses a real
indexing build's log instead: diagnostics are honest, hover resolves and
jump-to-definition works across files. `references` still returns nothing, a
sourcekit-lsp limitation, so finding the callers of a Swift symbol stays a grep
job.

## 6. Review

| You are | Do |
|---|---|
| Holding a Tier 2+ diff | `reviewer` over the local diff before the push, briefed with the worktree's absolute path and the issue number |
| Holding a diff on a sensitive surface | Spawn `reviewer` with `model` set to Fable: `crates/intrada-ffi`, `ios/generated/`, a migration, `ActiveSession` or auth |
| Holding a small Tier 2 on one file with no sensitive surface | `/code-review` inline in the lead may stand in. Say "comment-policy violations are Blockers" or they survive as nits |

`reviewer` is pinned to Opus 5 `high`, a peer who reads the diff, not the
description. It reads the issue named in its brief and checks the diff
delivers the plan comment's done looks like before anything else. Check for a
sensitive surface with
`git diff --stat origin/main...HEAD | grep -E 'intrada-ffi|ios/generated|LibraryStore.swift|domain/session.rs'`,
not the tier the author claimed; a hit lifts `reviewer` to Fable, the one
lever that beats a definition's pin.

It reports and yields: no GitHub writes, no waiting for a PR number, because
the review runs before the push (2026-09-07: one review parked 35 minutes, one
cancelled at its 45 minute limit). The lead posts the summary verbatim at PR
creation, saying which agent produced it, and writes the PR body while the
review runs. On a multi-surface slice, the core half is reviewed before the
screens half starts.

## 7. Ship

| You are | Do |
|---|---|
| About to open or update a PR | `/ship`: gates through `test-runner`, `reviewer` over the diff, then `just pr-open` as a draft; watch CI to a conclusion in the same turn and read mergeability |
| The PR is green and reviewed | `gh pr ready`, and `just project-status N "In review"` for the issue |
| The unit has shipped: PR green, issue closed, worktree removed | `/clear` in the same sitting |
| About to leave the session for over an hour | Finish the unit or `/clear` first; the first turn back re-sends the whole context at write price, and the cold nudge will say so |

The gate funnel, Codecov expectations, the deferred-issue protocol and the PR
and issue body templates are in `.claude/skills/intrada-shipping/SKILL.md`,
which `/ship` follows. Every deferred item becomes a tracked issue, not a line
in a PR description. Stacked PRs are supported: open the child with base set
to the parent's branch, depth 2 maximum; CI runs on them because `ci.yml` has
no branch filter on `pull_request`. **Agents never merge**, and never push to
`main`.

A PR body says what the reviewer needs and stops. The weekend's median was 430
words with three over 1,000, against merges ten minutes after opening. Whether
that becomes a gate is decided by measuring the PRs after #1634 against its
586-word baseline, an experiment Jon set on 2026-09-10 with no gate on purpose.

**Verification, the standard held.**

- UI changes: drive the simulator and show proof, or say explicitly "I cannot
  reach the running app, here is what you need to verify by hand". Never claim
  "all green" when that means `cargo test` passed. Asking an agent to skip
  verification to save time costs more in rework.
- Report failures faithfully with the actual output. No "should work".
- Never report a gate as broken without making it fail.
- After any push to an open PR, watch the run to a conclusion in the same turn.
  Read the PR's mergeability, not just the job list: a renamed job leaves the
  old required context "expected" for ever, which is a hang, not a failure,
  and is invisible in the checks list (#1542).

**What holds it at the push.**

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
  server-side, and `guard-bash.sh` on Jon's machine matches the same denials
  inside compound commands.
- **Post-push note.** A hook on Jon's machine restates the CI-watch rule after
  every push.

After the merge, CLAUDE.md step 5 closes the loop: close the issue, drop
`in-flight`, `just project-status N "Done"`, and `just worktree-rm` once
merged. Seven sessions over four hours, every one Fable or Opus left open
across a break, and 194 cold turns costing $233 in the week before, are why
the unit ends with `/clear`; a launchd timer (`cold-nudge.sh`, machine-local,
#1842) nudges before the cold turn arrives.

## 8. Measure

| You are | Do |
|---|---|
| Changing the harness | Add its row to [`harness-log.md`](harness-log.md) in the same PR: date, PR, what changed, the number it should move, first read |
| It is Monday | Read `just usage 7 --quality` against the log; a lever that has not moved its number after two reads is reverted, not patched |
| Correcting a rule that failed | Edit that rule; never add a second one beside it |
| Wanting to see what sessions cost | `just usage` (last seven days by agent, model and effort, plus the biggest sessions, from `usage-report.py`); `just usage 14` for a fortnight. `usage-daily.sh` opens the day's first session with one line of it |

A correction edits the rule that failed and never adds a second one beside
it: over 12 to 14 September three worktree rules disagreed, six memories
restated them, and the mistake was back the next morning. The usage
guidelines folded into these steps were set on 2026-09-14 from the review in
#1836: three days, $1,225, 76 sessions, 44 PRs opened from Friday evening and
40 of them merged.

The three numbers are **speed** (Jon's time from claim to merged PR),
**rework** (fix PRs, reverts and corrections) and **spend** (tokens and
dollars, the constraint). Where each is read and its baseline are in the log.
The log replaced a freeze on harness work on 2026-09-15: fifteen harness PRs
merged on 2026-09-14 with nothing saying what each should move, and the freeze
held the harness still instead of measuring it (#1897). The 2026-09-27
`just usage 14` read stays as the first fortnight check for #1849.

**What it costs in practice.** Measured over the fortnight to 2026-09-13 from
the session transcripts on Jon's machine, all projects, weighted at API prices;
`just usage 14` prints the same table. Bullets naming a week come from
`just usage`, whose default window is seven days.

- **Context length drives usage more than the rung.** 52 of 154 main sessions
  passed 200k tokens, and the eight biggest sessions were 36% of the fortnight.
- **A parked session pays again.** The prompt cache lasts an hour; the first
  turn after a longer gap re-sends a context past 50k at write price. In the
  week to 2026-09-13 that was 194 turns and $233, and cache writes were 29% of
  that week's spend. The context watch names such a turn as it happens.
- **Tool output is what fills the context.** 36 MB in the week to 2026-09-13,
  Read 31%, then git, sed, grep, just and cat at 5 to 10% each; `just usage`
  prints the table. In the fortnight to 2026-09-17 Read was 38% and 42% of
  text reads repeated a file the session already held, which is why the read
  guard now watches the Bash reads too (#1986). Read once, keep ranges tight,
  and run gates in `test-runner`.
- **Fable's cache reads** cost $0.25 per MTok against Opus's $0.50, which keeps
  the gap to Opus small at long context.
- **Subagents were 26% of the fortnight's usage.** Their largest line was
  `task` spawned with `model: opus` over what was then its Sonnet pin. In the
  fortnight to 2026-09-14 `task` runs at `xhigh` were $353, all from spawns
  lifting the pin before the spawn guard existed. `task` runs grow as long as
  main sessions (59% of their turns past 200k), so brief one slice per spawn.

**Where a harness change lives.** A rule lives in exactly one layer, chosen by
who reads it and when.

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
5. **Agents.** `reviewer`, `test-runner` and `task`, with model and effort
   pinned in the definition. `Explore`, `fork` and `general-purpose` are
   built-in with no definition to pin.
6. **Hooks.** Repo: format on edit, install the git hooks, the session-start
   claims list. User: the worktree guard and its lease, the bash guard, the
   read guard, the spawn guard, the turn reminder and context watch, the
   post-push CI note, the daily usage line, and the cold nudge on a launchd
   timer. Every user hook fails open and ships with a test battery that is
   mutation-tested, not trusted green. When a threshold or a name changes in a
   hook, grep the docs for the old value in the same change.
7. **Auto memory.** What neither CLAUDE.md records: preferences, corrections,
   project state the code cannot show.
8. **Docs on demand.** `reference.md` for the why behind every rule, the specs,
   the roadmap. Cost nothing per turn.

The test for the personal versus project split, from the RIBA template review:
if this is set wrong, who does it hurt? Only the author, and it is personal
(`.claude/settings.local.json`, `~/.claude`). Anyone else, and it is project,
checked in, and enforced somewhere the personal layer cannot weaken it. Edit
the rules files with `/memory`; tidy permission prompts with
`/fewer-permission-prompts`.

## Troubleshooting

| Symptom | Cause |
|---|---|
| A rule did not load | Rules with `paths:` fire on the Read tool only; a `cat` via Bash does not count. `/context` shows what loaded |
| A skill will not resolve | Skills are discovered at session start; restart after adding one |
| `just check` says already green | Stamp matches HEAD and the tree is clean; delete the stamp |
| Unformatted Swift or Rust reaching CI | The format hook loads at session start; run `just fmt` and `just ios-fmt` |
| PR hangs on a check that never reports | A renamed job left its old required context "expected" (#1542) |
| A subagent ran on the wrong model or effort | Its definition has no `model:` or `effort:` pin (`Explore`, `fork`, `general-purpose`), so it inherited the session's model or effort (#1838). Pass `model: haiku` to `Explore` |
| The spawn guard refused a subagent | The spawn lifted `model` or `effort` above the definition's pin. Only `reviewer` may go up, to Fable on a sensitive surface. Drop the override; keep the judgement in the lead instead |
| Read denied as "already in context" on the first read after `/clear` | The guard reads the transcript, which `/clear` does not restart (#1866). Range-read (`offset`, `limit`) until it is fixed |
| Read denied on a file over 400 lines | Grep for the symbol, then read the range. The guard never allows the whole file (#1844) |
| Edit or command denied in the main checkout | No worktree yet, or the `cd <worktree> && ` prefix is missing and the session holds more than one lease so it cannot be added for you. Make the worktree, prefix the command; the shapes the guard reads are in `worktrees.md` |
| Session start says another session holds this worktree | The lease is live; read `just worktrees` and pick another. A clean tree at main is not evidence it is free |
| A subagent edited the wrong tree, or `reviewer` reviewed main | The brief did not name the worktree by absolute path (#1861) |
| XCUITests refuse to launch, `SBMainWorkspace` busy | A simulator booted outside the `just` recipes. Ask before shutting it down; CI proves the UI tests meanwhile |
| A PR's first CI run shows cancelled or a stale failed context | `gh pr create --label` fires several events that cancel each other. Label after the first run starts, or hand Jon `gh run rerun --failed` |
| The session was interrupted mid-tool | Retry in a different form and keep working. An interrupted call is not a decision point to hand back |
| A gate looks like it ran nothing | Terse output is a pass. Break one assertion and watch it go red before reporting a gate broken (2026-09-04) |

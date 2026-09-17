# Working with Claude Code, from scratch

> A guide for an engineer starting with Claude Code, or starting a new repo
> with it. It is the generic half of a pair: the other half is a repo's own
> `docs/working-with-agents.md`, which names the recipes, hooks, thresholds and
> incident numbers for that codebase. Nothing here names a repo. Everything
> here was learned on one, mostly by getting it wrong first.

**The one thing to take away:** an agent does what your system lets it do, not
what your documents ask it to do. Every section below is about closing the gap
between those two.

## 1. Principles

1. **Enforcement is structural, not behavioural.** A rule in a document binds
   an agent only if it reads the document and chooses to obey. A rule as a
   test, a CI gate, a commit hook or a tool guard binds every agent every time.
   The most important move in agentic coding is promoting rules from the first
   kind to the second. Most rules that matter are judgement and cannot be
   gated, so prose never disappears; the skill is knowing which rules can
   become checks and promoting exactly those.
2. **Keep the leanest set of written rules.** Every always-on line is paid for
   on every request, and a fat rulebook rots. Enforce and delete: when a rule
   becomes a gate, its prose goes in the same change. A correction edits the
   rule that failed; it never adds a second rule beside it.
3. **Ground truth lives in the system.** Work status is labels on issues and
   open pull requests, read by a command, not a hand-edited status file. Any
   state a human must remember to update will drift, and a drifted document is
   worse than none because it is trusted.
4. **Match ceremony to the size of the change, with a sensitivity override.**
   Trivial fixes ship; ordinary feature work gets a short plan; architectural
   work gets a written spec. Anything touching auth, a schema, a migration or
   the contract between components escalates regardless of how small it looks,
   because those fail silently.
5. **Cheap models only where there is no judgement.** Running a gate or
   finding where something lives needs none, so the cheapest model does it.
   Everything with judgement in it runs on the middle model, and the strongest
   model thinks: the plan, anything that fails silently, the direction call,
   the bug nobody can explain. A cheap model on judgement work costs more of
   your time in corrections than it saves in spend. Encode it as
   configuration, pinned where the agent is defined, and name the rung out
   loud at every boundary.
6. **Verify against the real surface, and say so when you cannot.** "Green"
   means the thing ran: the app launched, the screen rendered, the test
   executed. Never quietly "the compiler was happy". If a surface cannot be
   verified, say so in one line and name what a human checks by hand. Silence
   reads as verified.
7. **The human owns the merge.** Agents branch, commit, push, open the PR and
   get CI green. A person merges. It is the last point where judgement is cheap
   and a bad change is still contained.
8. **Nothing unread stays in the tree.** Dead code is deleted, not parked
   behind a flag; a rule a machine now enforces is dead prose and goes too.
   Version control is the parking space.

## 2. The layers, and which one a rule belongs in

A rule lives in exactly one of these, chosen by who reads it and when. Cost is
in tokens per request; every always-on layer is paid by every session and,
mostly, again by every subagent.

| Layer | Loads | Belongs there |
|---|---|---|
| Gates: branch protection, CI, git hooks, tool guards, a permissions deny list | Never; they run | Anything that can be checked mechanically. The prose comes out when the gate goes in |
| The project rules file (`CLAUDE.md`) | Every session and subagent, in full | Invariants every session needs: architecture, the tier system, the always-do list. Under 200 lines |
| Path-scoped rules | When a file matching their globs is read with the Read tool | Rules for one surface: the persistence layer, the UI layer, the sensitive files. Free until relevant |
| Skills, by name | Description always; body when invoked | Workflows: shipping, running parallel streams. Not surfaces |
| Agent definitions | Description always; body becomes the subagent's system prompt | The three or four worker shapes you delegate to, with model and effort pinned |
| Hooks | Run on events; inject text or deny a call | Guards and nudges: the machine-checkable rules from the top row that live outside the repo |
| Auto memory | Every session, first 200 lines; not into subagents | Preferences, corrections, project state the code cannot show. The one layer that can carry a stale fact |
| Docs on demand | Nothing per turn | The why behind every rule, the incident write-ups, specs, the roadmap |

Two tests decide placement. **Who does it hurt if set wrong?** Only the author,
and it is personal (user settings, user hooks). Anyone else, and it is
project, checked in, and enforced somewhere the personal layer cannot weaken
it. **Who reads it and when?** A rule needed on every turn goes in the rules
file; one needed only when touching a surface goes path-scoped; one needed
only when doing a workflow goes in a skill; one nobody needs mid-task goes in
a doc the rules file links to.

Tie every rule to the incident that created it, by issue number or date, and
keep the story in the on-demand layer. A rule that cites the bug it prevents
is kept and trusted; one that reads as dogma is ignored or cargo-culted.

## 3. The session lifecycle

A session is a unit of work with a beginning and an end, not a place you live.
Context length drives cost more than the model does, and a long session
re-sends its whole history on every turn.

| You are | Do |
|---|---|
| Starting a unit | Claim the issue in the tracker so a second session cannot build it. Make a worktree of your own and drive it from where you started. Read the path-scoped rules for the surfaces you will touch. Name the model and effort before the first edit |
| Choosing a rung | The middle model by default; the strongest only for the short list it owns (section 4) |
| Holding ordinary feature work | A plan of about 150 words as a comment on the issue before the first commit, approved by the human (section 4) |
| Holding a settled plan | Hand the patterned slices to a builder subagent, one slice per spawn; keep the judgement in the lead (section 5) |
| About to run a gate | A gate-runner subagent, never the lead. A failing suite prints thousands of lines that are re-sent on every later turn |
| Looking at a file for the second time | Grep, then a range read. Never the whole file again |
| Context past the warning line | Finish the slice in hand; the rest of the unit goes to a subagent and this session checks and ships |
| Context past the firm line | Hand the rest over now |
| Context very long and the thread still matters | Compact: decisions survive, tool output goes. Mid-task only |
| The unit has shipped | Clear the session, in the same sitting |
| Leaving for over an hour | Finish or clear first. The prompt cache lapses; the first turn back re-sends the whole context at write price |
| Reaching work on the strongest model's list | Switch model and effort in place, say so, and drop back at the boundary. A new chat is for freeing the session, never for a rung change |
| Handing work on | An opener the next session pastes into a fresh chat, naming the issue, the model, the effort and any other stream running. That session claims the issue and makes its own worktree |
| Asked what is next | Answer from the tracker command and the session-start list, with no further reads |

**Clear versus compact versus new chat.** Clear ends a unit: nothing carries
over, and the next unit starts on a cold cache anyway. Compact is for a unit
that is not finished: the model summarises its own history, which keeps the
decisions and loses the bulk. A new chat is a clear plus a handover, and it
costs the handover: use it when the lead needs freeing for something else or
the driver is changing, never because the work needs a stronger model, which a
command changes in place.

Set the thresholds from measurement. One repo measured its main sessions,
found a third passed 200k tokens, and set the warning at 150k, the firm line
at 200k and the compaction nudge at 400k. Yours will differ; the shape will
not.

## 4. Model and effort

Match the **model** to how silently wrong the work can go, and the **effort**
to how much thinking beats typing. Failure that is visible (a wrong layout on
the simulator, a red test) degrades gracefully and is caught cheaply.
Failure that is silent (a serialisation contract that becomes a no-op, a
migration on the only copy of the user's data, auth) gets the strongest setup
regardless of diff size.

**Three rungs.** Measure your own, but one repo settled on the strongest model
thinking, the middle model building and the cheapest model running, after
corrections ran at much the same rate per prompt on every rung and the human
judged that the cheap rungs cost more of their time in loops than they saved
in spend.

| Rung | Human equivalent | Ask it for |
|---|---|---|
| Strongest model, high effort | The architect you pull into a design review. Sets the shape; builds only what fails silently | The plan, the contract or migration shape, the direction call, the bug nobody can explain |
| Middle model, the effort below max, the lead | A strong senior engineer who owns the ticket end to end | The build, within a shape already agreed |
| Middle model, high effort, the builder | The same engineer working alone from a written ticket | One slice, reported back as a diff |
| Middle model, high effort, the reviewer | A peer on the PR | Reads the diff, not the description; never merges |
| Cheapest model, low effort, the gate runner | CI on your desk | Runs the gate, names what failed, has no opinion |
| Cheapest model, the explorer | A new starter sent to find where something lives | File names and line numbers; a lead, not a fact |

**The strongest model's list** is short and written down: the plan, anything
that fails silently (the contract between components, a migration, auth), a
direction call, and the bug nobody can explain. A session opens on the middle
model and switches only for that list, in place, saying so.

**Effort** is how long you would let them think before answering: low is a
reply in the corridor, medium a minute at the desk, high worked through on
paper, the level above slept on and back with the trade-offs, max a spike or a
design note. One repo found the longer think nearly free per turn on the
middle model, since context length dominates the bill, while on the strongest
model it was the bill. Tiers map the same way: a fix you would just do, a
ticket with a plan attached, a design note before anyone builds.

**The plan comment.** Ordinary feature work starts with a plan of about 150
words on its issue: done looks like, decisions taken, files and lines, tests,
out of scope, routing. The human approves it, the build reads it, and the
reviewer checks the diff against its done looks like. A plan that runs long is
the signal the work is really architectural and needs a spec.

**Rules of thumb.** Effort does not follow a model switch; set both. A session
that says what the work needs in its first reply makes the routing a decision
rather than a default, and a wrong one is one command away rather than a
restart. Fast modes priced at the top tier's rate are for latency, never
economy.

## 5. Delegating

Four worker shapes cover almost everything. Define each once, in a file the
repo reviews like code, with model and effort pinned in the definition:

| Job | Shape | Rung | Because |
|---|---|---|---|
| Read-only research | The built-in explorer | Cheapest model, passed at the spawn since built-ins have no definition | Reports facts back; the lead verifies |
| Run a gate and filter its log | A gate runner | Cheapest model, low effort | No judgement; the gate is the check |
| Review a diff or a plan | A reviewer | Middle model, high effort; lifted to the strongest on sensitive surfaces | A peer who reads the diff; the strongest rung stays for what fails silently |
| One conventional slice on a safe surface | A builder | Middle model, high effort | Patterns already in the repo |

Rules that hold across all four:

- **One agent per vertical slice.** Splitting one slice across two agents
  (one on the core, one on the shell) fails because the second cannot see the
  invariant the first relied on. Fan out only on genuinely independent pieces,
  and the lead integrates.
- **A subagent starts blank.** No conversation history, no memory, no idea
  which worktree the lead is driving. The brief names the worktree by absolute
  path, the files and lines, the rule files to read, the gate to run and the
  report shape. A brief that names an area instead of a line makes the agent
  read whole files to find its own work.
- **The report is data, not prose.** Diff stat and changed symbols, the gate
  and its counts, what was left out or assumed, what could not be verified.
  The lead checks the report against a reviewer and the gate counts, and opens
  a file the subagent touched only to act on a named finding, by range.
- **Acceptance criteria must be able to fail tomorrow.** An agent satisfies
  the weakest reading of its instructions. A mechanical-edit agent once wired a
  binary to a per-shell directory that vanished with the shell; every
  criterion it was given was true at the moment of checking. "Works" is not
  the bar; "still works tomorrow" is.
- **A subagent's finding is a lead, not a fact.** One blamed the wrong commit,
  cited a line that pointed at a comment, and said it could not run a command
  it could. Brief research agents to mark observed against inferred, and
  verify before acting.
- **Pins beat spawns.** A definition with no model or effort inherits the
  parent's, which is how a builder ends up on the top model at top effort.
  Pin both in every definition. The built-ins have no definition: pass the
  cheapest model to the explorer at the spawn, and remember the others inherit
  the lead's model and effort. Have a guard refuse a spawn that lifts a pinned
  agent, with the reviewer as the one exception.
- **Gates run in the gate runner, never the lead**, and every fan-out task
  skips the suites; the lead runs them once at the end.

## 6. Worktrees and parallel streams

**One session, one worktree, made by that session.** The main checkout is
where every session starts and where nobody edits: it stays the clean base
every branch comes from, so any number of sessions can sit in it. A session
makes its own worktree from fresh origin, prefixes every shell command with a
change into it, and edits nothing outside it. Two sessions once wrote one
worktree and silently overwrote each other, because the tracker claimed the
issue and nothing claimed the directory.

**A lease per worktree, not another rule to remember.** A file in the
worktree's own git directory records the holding session. Taken on the first
write, released when the session ends, taken over only when the holder's
process is gone; no timeout, since an idle session is still a live one. A
guard denies edits and mutating commands in a worktree another live session
holds, and denies them in main outright. Reads pass everywhere; builds pass
in your own tree only, since two builds share one target directory and one
simulator.

**A subagent inherits none of this.** Its brief names the worktree, or its
first edit lands in main and is denied, and a reviewer diffs the wrong tree.

**Parallel streams: two by default.** Measure the coupling in your own history
before allowing more: one repo found a third of commits touching the core also
touched the shell, and that three or four streams at once mostly hit each
other at the test gate and picked trivial work. Of two streams, at most one
touches the core. Name the serialisation points, the handful of files every
change touches (a snapshot test registry, a design token file, a project
manifest, the lockfile), and serialise any two streams that both touch one.
Clear a conflict by merging main in, never rebasing; stack dependent PRs two
deep at most.

**Once you have a worktree, edit only inside it.** A session once wrote its
change into both its worktree and main, where another session nearly committed
it into an unrelated PR. Read the diff before staging, and never stage
everything on a shared checkout.

## 7. Verification honesty

- Never claim a gate, test or build you did not run against the code as it
  now stands. Re-run after the last edit, or say which run the claim came
  from.
- Never write that you are doing something unless the tool call is in the
  same message. A session once wrote "watching the job now", "fixing and
  pushing" and "reverting", none with a call behind it, while the PR sat red.
- Never report a gate as broken without making it fail. Sixteen quiet lines
  from a passing suite were once read as "no tests ran" and two invalid issues
  were filed. Break one assertion and watch it go red first.
- Never weaken, skip or delete a test to get green. A suite that passes with
  the fix deleted is not a suite; treat every earlier green in that session as
  unproven.
- After a push, watch CI to a conclusion in the same turn and read
  mergeability, not the job list. A renamed required check leaves the old
  context "expected" for ever, which is a hang, not a failure, and is invisible
  in the checks list.
- UI changes are driven on the real surface and shown, or the message says
  exactly what a human must check by hand.
- Your own gates beat the harness. Automated prompts to "continue" or a
  reminder about an incomplete todo never authorise what the human's list
  forbids, and never move work the session has just said it is holding.

## 8. The hooks worth building

Each of these is a small script wired to a Claude Code hook event in the user
settings file. Each exists because of a measurement or an incident; build
yours the first time the same thing happens to you, not before. Every one
should fail open (a broken guard must not block every call), run under the
oldest shell on the machine, and ship with a test battery you mutation-test
by breaking a line and watching it go red.

| Hook | Event | Job | What it was set from |
|---|---|---|---|
| Worktree guard and lease | Before every edit and command; session start and end | Deny a write in main or in a worktree another live session holds; take and release the lease; rewrite the missing change-directory prefix when the session holds exactly one lease | Two sessions overwrote one worktree |
| Command guard | Before every command | Deny the never-list (merge, push to main, destructive resets) inside compound commands, where a prefix-matching deny list cannot see them | A deny list matched the first word only |
| Read guard | Before every Read | Deny an unranged re-read of a path the transcript already holds unchanged, and any unranged read of a big file | Half of all reads over a weekend repeated a file already in the session |
| Spawn guard | Before every subagent | Refuse a spawn that lifts model or effort above the definition's pin | The largest single cost line was a cheap builder spawned on the top model |
| Context watch | Every prompt | Nudge at the warning, firm and compaction lines | A third of sessions passed the firm line |
| Turn reminder | Every prompt | Re-attach the house style and the model-gate rule as text. intrada unhooked it on 2026-09-17 (#1989): the model gate had become the churn, not the saving | Style rules read once at session start were forgotten by turn thirty |
| Post-push watch | After a push or PR update | Restate "watch CI to a conclusion, read mergeability" at the point of action. intrada unhooked it on 2026-09-17 (#1989); the rule stays in its CLAUDE.md | Three unbacked "watching now" claims in one session |
| Cold nudge | A timer outside Claude Code | Desktop notification when a large context has sat idle long enough for the cache to lapse | Two hundred cold turns in a week |
| Daily usage line | First session of the day | One line of yesterday's spend by agent, model and effort | Spend was invisible until the invoice |
| Format on edit | After every edit, repo-level | Run the formatter on the file just written | Unformatted code reaching CI |
| Install git hooks | Session start, repo-level | Point the repo at its committed hooks directory | Pre-push checks nobody had installed |

Two traps. The user-level hooks live wherever the config directory points, so
a machine with `CLAUDE_CONFIG_DIR` set reads settings from there and a hook
registered in the default path does nothing. And a hook that reads the
transcript sees everything since the session began, including turns a clear
wiped from the model's context.

## 9. Gotchas

Each cost at least an afternoon somewhere. Generalised; the repo doc carries
the specific one.

- **Path-scoped rules fire on the Read tool only.** A `cat` through the shell
  does not load them, and neither does reading a file inside a worktree driven
  from main. A session reads the rules for the files it is about to touch by
  hand, or it is working blind.
- **Subagents do not see memory or the session's worktree.** Everything they
  need goes in the brief.
- **Effort does not follow a model switch**, and an unpinned subagent inherits
  the parent's effort. Set both, pin both.
- **A transcript outlives a clear.** Anything that reads it (a repeat-read
  guard, a cold-session timer) has to reset at the clear marker or it judges
  the new unit by the old one.
- **The isolated-worktree tool can lock a session out of git.** A tool that
  marks the session isolated may make the command guard refuse every version
  control command. Make the worktree and prefix commands instead.
- **Interrupted tool calls are not decision points.** Retry in a different
  form and keep working; a menu of options back to the human is the failure.
- **Creating a PR with labels in the same call can cancel its own CI.**
  Several events fire and cancel each other, leaving a stale failed context.
  Label after the first run starts.
- **A shared simulator or device is machine-global.** Two test runs on it
  collide; serialise them with a machine-wide lock that waits, and never reset
  a device you did not boot.
- **A green run proves nothing about whose work is in the tree.** Read the
  diff before staging.
- **Terse output is a pass.** A quiet gate is not a broken one.
- **A cheap model on judgement work is a false saving.** The corrections cost
  more of your time than the rate saves; keep the cheapest models for jobs with
  no judgement in them.
- **A plan that lives in a chat is lost.** Put it on the issue as a comment:
  done looks like, decisions, files and lines, tests, out of scope, and the
  routing (model, effort, location, stream). Without the routing the next
  session re-derives it, or takes the default.
- **Twelve of seventy-six sessions did nothing but status or handover.** A
  new chat costs a full context load; answer "what's next" from the tracker
  command instead.
- **Documents disagree with hooks within a week.** When a threshold or a name
  changes in a script, grep the docs for the old value in the same change.
- **Log every harness change with the number it should move.** One repo
  merged fifteen harness changes in a day with nothing saying what each was
  for, then froze the harness to measure it. A dated row per change, read
  weekly against the usage report, with a change reverted when its number has
  not moved after two reads, keeps improvement continuous and measured.

## 10. What a newcomer should not copy

- **Do not copy a mature setup wholesale.** Almost every rule in a good one is
  scar tissue from a specific failure. Copied without the failure, it is cargo
  cult, and you will not know which rules are load-bearing. Start with almost
  nothing and add a rule the first time you get burned.
- **The scaffolding can waste more time than the agents save.** Worktree
  tooling, launchers, hooks and config are brittle and break quietly. A
  misconfigured tool once built empty working directories for days. Budget
  real time for the plumbing, and be suspicious of automation you cannot see
  failing.
- **Behavioural rules are only as strong as the tools that read them.**
  Introduce one tool that does not read your rules file and every prose rule
  is worthless the moment it touches your code. Prefer fewer tools, or push
  the rules into gates that bind regardless of the driver.
- **Agents will claim work they did not do.** Not malice, the failure mode.
  Build habits and checks that assume it; never accept a green you did not
  see produced.
- **Automation will push you past your own stopping points.** You need an
  explicit, higher rule that your gates and holds beat the harness.
- **This is front-loaded and can be over-engineered.** It pays off when the
  code has to be right in six months. For a weekend throwaway it is absurd.

## 11. Turning this into a starter kit, later

When a second repo needs the same setup, extract rather than rewrite. What to
lift, in order of how much it saves:

1. **The user-level hooks with their test batteries**, as a directory with an
   install script that writes the hook wiring into the user settings file and
   refuses to run if the config directory variable points elsewhere. The
   tests travel with the scripts or the scripts rot; run the batteries in the
   install.
2. **A rules-file skeleton** of under 60 lines: architecture placeholder, the
   tier system, the sensitivity override, the always-do list (claim, worktree,
   PR, human merges, close the loop), the style rules, and the two links to
   `working-with-agents.md` and this file.
3. **The three agent definitions** (gate runner, reviewer, builder) with
   model and effort pinned and the
   worktree paragraph in each, with the repo-specific hard stops (the
   sensitive paths) left as a clearly marked list to fill in.
4. **The recipes**: make a worktree from fresh origin, list worktrees with
   their leases, claim an issue and move the board, open a PR that refuses an
   unclaimed issue number, print the usage report.
5. **A `working-with-agents.md` template** with the section headings from
   the repo version and every number replaced by "measure this".

Sanitise as you go: no issue numbers, no dates, no people, no paths outside
the kit's own tree. Keep the kit in its own repository with a version stamp,
and treat a change to a hook in the source repo as a change to the kit in the
same week, or the two drift and the kit becomes a third rulebook. Do not
extract a rule the new repo has not yet needed; the kit is the enforcement
layer and the skeletons, not the scar tissue.

## How to actually start

Pick one real rule you keep breaking. Write it down in one line. The first time
an agent breaks it anyway, turn it into a check that fails. Delete the line.
Repeat. In a year you will have a system that fits your work exactly, because
every part of it was forged by a real failure, and you will understand every
rule because you were there when it was needed. That loop, incident to rule to
gate to prune, is the whole craft. The artefacts are its residue.

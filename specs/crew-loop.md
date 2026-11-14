# The crew loop

A script that builds ready issues unattended, one at a time, each in its own
Claude Code background session, and leaves a draft pull request for the
owner. Epic #2799, parent issue #2803, built in three slices: #2819 (park and
pick), #2820 (the session side) and #2821 (the loop itself). The full plan is
a private page linked from #2803.

## What the owner sees

The owner approves a plan by adding the `ready` label. After a run the issue
has either a draft pull request with both apps' screenshots and one evidence
line per "Done looks like" criterion, or a `needs-owner` label and one
question as a comment. Their phone tells them which. Nothing is ever marked
ready or merged by the crew: every merge stays the owner's.

## Starting and stopping

- Started by hand with `just crew`. There is no launchd job: the owner decides
  each time whether the crew runs.
- At each start the loop runs harness check 1 from `scripts/crew-qualify.sh`
  (a crew session's hooks see the crew marker and refuse `gh pr ready`). A
  failure stops the loop before any issue is touched.
- A stop file, checked before each issue, ends the run cleanly.
- One run at a time. The loop takes a `mkdir` lock before it touches any
  worktree or session, and releases it on every exit, Ctrl-C and a closed
  terminal included. A second run finds it held, logs that and ends, since it
  would count the first run's worktree as unfinished and stop its live
  session. A lock is cleared when its pid is no crew run, or when it has no
  pid 10 seconds after it was made. It is moved aside before it is removed,
  so two starters cannot each clear the other's fresh lock.
- The run keeps the Mac awake with `caffeinate -i`, watching the loop's own
  process, so a sleep never jumps the wall clock and parks an issue as a cap.
  A machine without `caffeinate` logs that and runs anyway.

## Caps

Set by the owner on 2026-11-12, all enforced by the script, since
`--max-budget-usd` works only in print mode and background sessions have no
turn limit:

| Cap | Figure | On reaching it |
|---|---|---|
| Wall clock per session | 3 hours | Stop the session and its process tree, park |
| Silence limit | 45 minutes with no write to the session's transcript, its subagents' or its background tasks' output | Stop, park |
| Issues per run | 3 | The loop ends |
| Daily spend | the last 24 hours of `just usage` past $150; an unreadable report counts as past | The loop ends before the next issue |

The silence limit is longer than the simulator queue, which waits up to 1800
seconds (`IOS_SIM_LOCK_TIMEOUT` in `scripts/ios-sim-lock.sh`), so a session
waiting its turn for the simulator is not mistaken for a stalled one.

## One issue, start to finish

1. **Pick.** First any crew worktree left by an earlier run whose issue is
   open, has no draft note, no `needs-owner` label, and no park note unless
   a later comment or the label's removal follows it, counting trusted logins
   only. One GraphQL read covers every such worktree, and an issue GitHub
   cannot read is skipped. Then the ready queue
   (`scripts/generate-status.sh --ready-json`, the READY section's order:
   horizon, Priority, then age), less any issue labelled `needs-owner`. Take
   the first for which `scripts/crew-pick.sh <issue>` exits 0; an issue is
   tried once a run.
2. **Worktree.** The script makes the worktree; the session runs
   `just claim` inside it, so a restart finds both and never claims twice.
3. **Session.** One `INTRADA_CREW=1 claude --bg` session on Opus at high
   effort, started inside the worktree, after stopping any live session
   already there. No session handles two issues. The loop reads the new
   session's id from `claude agents --json`: the session named `crew-N` whose
   `startedAt` is no earlier than the start, since `claude --bg` prints its id
   only as text. A start that never shows in the list stops any session named
   `crew-N` and ends the run without parking.
4. **Build.** The session posts its brief on the issue, builds with `task`,
   reviews with `reviewer`, runs the QA agent, and opens the draft with
   `just pr-open --draft`. `/ship` in loop mode ends at the draft.
5. **Watch.** The script polls `claude agents --json` each minute. Any wait
   (status `waiting`, or any `waitingFor` value) parks at once, since nobody
   answers a crew session. Two polls in a row reading blocked, done, failed,
   stopped or idle mean the turn ended, whatever the pid, since a finished
   session reads blocked with no pid. Then the newest
   `**Crew:` note since the session started decides: draft, parked, or the
   loop parks it. The session counts as having no process only when it reads
   working with a missing or dead pid on two polls in a row. Silence is read
   from the newest write to the session's transcript, its subagents' or its
   background tasks' output, since a lead waiting on `task` writes nothing
   itself. Stopping the loop with Ctrl-C, TERM or a closed terminal stops the
   session it is watching.
6. **Lessons.** The retro posts its lessons as an issue comment headed
   Lessons and labels the issue `lessons-pending`, never writing
   `docs/lessons.md`, so two drafts never conflict. The next brief carries
   `docs/lessons.md` from main plus every trusted Lessons comment under that
   label, open or closed, so a merged draft's lessons are not lost; the monthly prune
   folds them into the file and removes the label.
7. **Push.** The owner's phone is told the outcome, then the next issue
   starts.

Every step posts its note on the issue. A restart reads the last note and the
worktree before doing anything.

## Qualifying an issue (slice 1, `crew-pick.sh`)

An issue qualifies only when every rule below holds. One GraphQL read fetches
the issue, its labels, comments, label events and blockers.

- **Ready.** It is open and carries `ready`. An issue without it is passed
  over in silence: it is not in the queue.
- **A body it can trust.** The owner or the agent account opened the issue
  and made its last edit, since anyone's issue can be labelled and its author
  can edit it after. Otherwise the body is never read.
- **Plan.** Every issue, whatever its tier, has a plan comment: the newest
  trusted comment with both the `**Done looks like**` and `**Files**` headings of the
  shipping skill's skeleton, and non-empty Files, Tests and Routing too. A
  heading may read `**Files**:` or `**Files:**`, or stand alone with a list
  under it. No tier is guessed from the body, so an unplanned issue never
  qualifies.
- **Approved after the last edit.** The newest `ready` label event is later
  than the plan comment's last edit, so an edited plan needs the label again.
- **Evidence.** Every item, bulleted or numbered, under the body's "Done looks
  like" heading names a test, self-test, snapshot or screenshot as a whole
  word. A heading with no items skips the issue. With no such heading, the
  plan's `**Done looks like**` line is read instead, split at semicolons.
- **No open call.** No "Done looks like" line, in the body or the plan, reads
  "needs your call".
- **Mockup.** A screen issue, one whose Files name a path in a `Views/` or
  `ui/` folder, however shortened, has a `**Mockup**` line with a claude.ai
  canvas link and "approved the mockup on" a date, never "not approved".
- **No blocker.** No open issue in GitHub's "blocked by" relation.
- **Not sensitive.** `crew-park.sh` exits 0 on the whole plan comment.
- **A reply it can trust.** A reply missing the blocked-by list exits 2, as a
  failed read does, rather than reading as no blocker.

On a skip, `crew-pick.sh` exits 1 and posts one comment headed
**Crew skipped** listing every rule that failed; the issue stays skipped
while any of them holds. It posts none when a Crew
skipped comment already exists newer than both the last `ready` label event
and the plan's last edit, so a skipped issue costs one comment until the owner
edits the plan or re-adds the label. A failed GitHub read exits 2 and posts
nothing.

## The park step (slice 1, `crew-park.sh`)

`scripts/crew-park.sh <file>` reads the `paths:` list in the frontmatter of
`.claude/rules/sensitive-surfaces.md` at run time, the only list of sensitive
paths. It exits 3, printing each match, when the file names a matching path;
0 when it names none; 2 when the file or the list cannot be read, which the
loop treats as a park.

- A plan is read in full. Plans shorten paths, so a path counts when it
  matches a glob, is the tail of a glob's fixed part two folders or longer
  (`domain/session/active.rs`), names a crate's own folder (`intrada-ffi`),
  is the file name of a glob with no wildcard, or is a wider glob of its own
  (`crates/**`). Each errs towards parking.
- A diff (a file with `diff --git` lines) is read by its changed-file headers
  only, so a test or document that merely mentions a sensitive path does not
  park the diff.

It runs twice: the loop runs it on the plan before the session starts, and
the session runs it on its diff before the draft opens (`/ship loop`).

## Parking (slices 2 and 3)

An issue parks on any of:

- any wait, each with its own reason and question: `waitingFor` reading
  "permission prompt", "input needed", "sandbox request", "worker request" or
  "dialog open", a value the loop does not know, or status `waiting` alone;
- a session that failed (`state=failed`) with no draft or park note;
- a turn that ends in a question (blocked and idle);
- a session shown working with no process behind it;
- any cap above;
- two failed review, QA or CI rounds;
- a plan that needs a choice the owner has not made;
- a sensitive path in the diff (`crew-park.sh` exit 3).

Parking has two halves in `scripts/crew-loop.sh`. `stop-session <id>` reads
the session's process tree, stops it, kills what is left and releases the
simulator lock only when one of those processes held it; harness check 7 calls
it as `CREW_PARK_STEP`. The issue half then adds the
`needs-owner` label, posts the question as a comment, keeps the claim and
pushes to the owner's phone. Nothing waits on
a prompt; the loop takes the next issue. When the label cannot be added the
run ends, and a later run passes the issue over until a trusted comment
follows the park note or a trusted login removes the label. When GitHub cannot be read for the outcome, twice, the run ends
with the issue neither announced nor parked.

## The session side (slice 2, #2820)

- `.claude/commands/crew.md` is the brief: the loop starts the session with
  `/crew N`. A background session expands a command in its first prompt
  (proved by hand with a stand-in command; the #2821 row of
  `docs/harness-log.md`).
- `.claude/agents/qa.md` on Opus, since ticking a criterion against a
  screenshot is judgement. Read-only on code: it launches both apps with
  `just ios-run` and `just android-run`, each screenshot kept in the
  worktree, leaves the gates to `test-runner`, and reports one line per
  criterion with its evidence; the session posts it. It never uses Xcode's
  tool server, which asks for approval once per new worktree; taps are
  proved by UI tests.
- Loop mode in `.claude/commands/ship.md` (`/ship loop`): the retro as a
  Lessons comment, the gates, `reviewer`, `qa`, the park step on the diff,
  the draft and the CI watch, and stop.

### Notes on the issue

Every step ends with one comment headed `**Crew: <step>**`, posted only on
success, the record a restart and the loop read: the newest such comment is
the last step done. A restart finding `draft`, or `parked` with `needs-owner`
still on, does nothing; `parked` with the label removed carries on from the
step that parked, with the owner's answer.

| Heading | Posted when |
|---|---|
| `**Crew: brief**` | Before building: the plan it follows, the slices, the criteria |
| `**Crew: built**` | The slices are committed |
| `**Crew: reviewed**` | The self-review leaves no blocker: rounds and verdict |
| `**Crew: QA**` | Every criterion passes: the `qa` report, one line each |
| `**Crew: draft**` | The draft is open and CI is green on the last commit |
| `**Crew: parked**` | The issue parks: what is done, and one question |

The Lessons comment is headed `**Lessons**`, outside this sequence.

### Trusted comments

The repository is public, so the crew reads lessons, plans, notes and answers
only from the owner's login, the repository's owner, and the agent account its
sessions act as, `ostinato-agent` (`scripts/crew-trusted.sh`). The loop, the
pick and the brief all filter by author, and a comment with no author is not
trusted. The brief, `qa` and `reviewer` read the issue through
`crew-loop.sh issue N`, which prints the body only under the pick's body rule,
and the lessons through `crew-loop.sh lessons`. A Crew skipped comment counts
only from a trusted login, and so does a Claimed comment in `just claim`. The owner's own sessions post as the owner, so an
author does not prove the owner wrote it: removing `needs-owner` stays the
real answered signal.

## The owner's phone

Every outcome raises a Mac notification, then a short print-mode session sends
it with Claude's own push tool, the only route found to the phone without an
outside service. That tool holds a push back while the owner is active at a
terminal (the #2821 row of `docs/harness-log.md`), so it reaches the phone when
they are away. Whether it lands is the owner's to confirm on the first run
with the owner away.

## Out of scope

More than one issue at a time, any merge, a planning step that writes plans,
and the crew's own GitHub login (#2811).

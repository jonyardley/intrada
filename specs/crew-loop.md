# The crew loop

A script that builds ready issues overnight, one at a time, each in its own
Claude Code background session, and leaves a draft pull request for the
owner. Epic #2799, parent issue #2803, built in three slices: #2819 (park and
pick), #2820 (the session side) and #2821 (the loop itself). The full plan is
a private page linked from #2803.

## What the owner sees

The owner approves a plan by adding the `ready` label. Next morning the issue
has either a draft pull request with both apps' screenshots and one evidence
line per "Done looks like" criterion, or a `needs-owner` label and one
question as a comment. Their phone tells them which. Nothing is ever marked
ready or merged by the crew: every merge stays the owner's.

## Starting and stopping

- Started by hand with `just crew`. There is no launchd job: the owner decides
  each night whether the crew runs.
- At each start the loop runs harness check 1 from `scripts/crew-qualify.sh`
  (a crew session's hooks see the crew marker and refuse `gh pr ready`). A
  failure stops the loop before any issue is touched.
- A stop file, checked before each issue, ends the night cleanly.

## Caps

Set by the owner on 2026-11-12, all enforced by the script, since
`--max-budget-usd` works only in print mode and background sessions have no
turn limit:

| Cap | Figure | On reaching it |
|---|---|---|
| Wall clock per session | 3 hours | Stop the session and its process tree, park |
| Silence limit | 45 minutes with no new transcript line | Stop, park |
| Issues per night | 3 | The loop ends |
| Daily spend | the day's `just usage` total past $150 | The loop ends before the next issue |

The silence limit is longer than the simulator queue, which waits up to 1800
seconds (`IOS_SIM_LOCK_TIMEOUT` in `scripts/ios-sim-lock.sh`), so a session
waiting its turn for the simulator is not mistaken for a stalled one.

## One issue, start to finish

1. **Pick.** Read the ready queue (`just status`, the READY section, ordered
   by horizon, Priority, then age) and take the first issue for which
   `scripts/crew-pick.sh <issue>` exits 0.
2. **Worktree.** The script makes the worktree; the session runs
   `just claim` inside it, so a restart finds both and never claims twice.
3. **Session.** One `INTRADA_CREW=1 claude --bg` session on Opus at high
   effort, started inside the worktree. No session handles two issues.
4. **Build.** The session posts its brief on the issue, builds with `task`,
   reviews with `reviewer`, runs the QA agent, and opens the draft with
   `just pr-open --draft`. `/ship` in loop mode ends at the draft.
5. **Watch.** The script polls `claude agents --json` for the session's state
   and checks the caps.
6. **Lessons.** The retro posts its lessons as an issue comment headed
   Lessons and labels the issue `lessons-pending`, never writing
   `docs/lessons.md`, so two drafts never conflict. The next brief carries
   `docs/lessons.md` from main plus every Lessons comment under that label,
   open or closed, so a merged draft's lessons are not lost; the monthly prune
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
- **Plan.** Every issue, whatever its tier, has a plan comment: the newest
  comment with both the `**Done looks like**` and `**Files**` headings of the
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

The loop runs it twice: on the plan before the session starts, and on the
diff before the draft opens.

## Parking (slices 2 and 3)

An issue parks on any of:

- a permission prompt (`state=blocked`, `waitingFor="permission prompt"`);
- a turn that ends in a question (blocked and idle);
- a session shown working with no process behind it;
- any cap above;
- two failed review or QA rounds;
- a sensitive path in the diff (`crew-park.sh` exit 3).

Parking stops the session and kills its whole process tree, adds the
`needs-owner` label, posts the question as a comment, keeps the claim,
releases the simulator lock and pushes to the owner's phone. Nothing waits on
a prompt; the loop takes the next issue.

## The session side (slice 2, #2820)

- `.claude/commands/crew.md` is the brief: the loop starts the session with
  `/crew N`. Whether a background session expands a command in its first
  prompt is proved in #2821; if not, the prompt names the file instead.
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

Every step ends with one comment headed `**Crew: <step>**`, the record a
restart and the loop read: the newest such comment is the last step done.

| Heading | Posted when |
|---|---|
| `**Crew: brief**` | Before building: the plan it follows, the slices, the criteria |
| `**Crew: built**` | The slices are committed |
| `**Crew: reviewed**` | The self-review is clean: rounds and verdict |
| `**Crew: QA**` | The `qa` report, one line per criterion |
| `**Crew: draft**` | The draft is open and CI has a conclusion |
| `**Crew: parked**` | The issue parks: what is done, and one question |

The Lessons comment is headed `**Lessons**`, outside this sequence.

## Rabbit hole

How a script pushes to the phone. It is proved first, in #2821, before the
loop depends on it; until then the outcome is only on the issue.

## Out of scope

More than one issue at a time, any merge, a planning step that writes plans,
and the crew's own GitHub login (#2811).

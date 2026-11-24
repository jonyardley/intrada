# Lessons

> What sessions learned, so the next one does not relearn it (#2801). Every
> session start prints the lessons below. The retro that writes them, and
> what each lesson's outcome is, is the first step of the route in the
> [shipping skill](../.claude/skills/intrada-shipping/SKILL.md#the-route).

- A lesson is one line: the date, the issue, then what to do differently and
  why. **What to look for**, one of each kind the session has: a mistake or
  correction; a slow or costly step, read from the session's row in
  `just usage 1 --quality`; and a tool, recipe or doc that misled.
- A lesson already here or in [`lessons-archive.md`](lessons-archive.md) is
  not added again: its line ends "(seen again N)", counting up, and an
  archived one is copied back here with its count raised above the
  archive's. A fixed lesson that comes back means the fix did not hold.
- A "(issue #N)" lesson is archived with "issue:" in front by the pull
  request that closes its issue, or with "fix:" and the file when that
  pull request moves it into a doc; `just pr-ready` refuses while a lesson
  waits on that issue or on one already closed (#2968). A lesson that is both seen
  again and waiting ends "(issue #N) (seen again M)".
- **At most 30 lessons**; `just check` refuses more. The session that reaches
  the limit archives the oldest with no "seen again" and no open issue.
- The archive is never printed at session start and nothing is deleted from
  it. A worktree prints its own copy, so one behind main misses newer lessons.
- A local merge of main keeps both sides' lessons (#2937), and the lessons
  check refuses a line kept twice, bumped on two branches, or brought back
  from the archive.

## Lessons

- 2026-11-20, #2936: a union merge of the lessons file kept both versions of an edited rule line above the list, and the lessons check reads only the list, so the doubled line passed; read the rules after any merge that touched them (issue #2943).
- 2026-11-24, #2747: `just handover` run in a merged branch's worktree told the next session to carry on in that worktree; write the opener for a new issue from fresh main, never from the old branch (issue #2969).
- 2026-11-24, #2747: `gh issue view N --comments` prints only the comments, so on an issue with none it printed nothing and exited cleanly; read the body with `gh issue view N` as well (issue #2969).
- 2026-11-20, #2918: a session in the main checkout reaching into its worktree with a `cd` prefix was refused 76 times in five weeks and loaded no path rules; start the session inside the worktree instead, and let it remove its own with `just worktree-done` once merged (issue #2973) (seen again 4).
- 2026-11-24, #2747: the worktree guard refused compound commands, a loop, and one naming a `.github` path that ran no git; split them into plain separate commands (issue #2976) (seen again 2).

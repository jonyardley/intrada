# Lessons

> What sessions learned, so the next one does not relearn it (#2801). Every
> session start prints the lessons below. The retro that writes them, and
> what each lesson's outcome is, is the first step of the route in the
> [shipping skill](../.claude/skills/intrada-shipping/SKILL.md#the-route).

- A lesson is one line: the date, the issue, then what to do differently and
  why. **What to look for**, one of each kind the session has: a mistake or
  correction; a slow or costly step, read from the session's row in
  `just usage 1 --quality`; and a tool, recipe or doc that misled. A
  subagent has no row of its own there, since its cost counts in its lead's
  session, so it names its slow step from its own run or says it has none.
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
  from the archive. It also refuses a rules line above the list kept twice
  (#2943).

## Lessons

- fix: .claude/skills/intrada-shipping/SKILL.md, 2026-11-25, #2789: a session started outside the repo has no project agent types, so spawning `reviewer` fails with "agent type not found"; spawn a general-purpose agent told to follow `.claude/agents/reviewer.md`.
- 2026-11-25, #3004: the lead readied #3019 and #3026 with a bare `gh pr ready`, so the retro never ran until the owner asked; ready only through `just pr-ready`, which refuses a branch with no lessons commit, since prose cannot stop the bare command (issue #3033).

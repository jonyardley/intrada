# Lessons

> What sessions learned, so the next one does not relearn it (#2801). Every
> session start prints the lessons below. Each pull request adds two or three lessons
> under Lessons, newest last, from the `/ship` retro step.

- A lesson is one line: the date, the issue, then what to do
  differently and why.
- **What to look for**, one lesson of each kind where the session has one: a
  mistake or correction; a slow or costly step, read from the session's row
  in `just usage 1 --quality` (peak context and cost); and a tool, recipe or
  doc that misled.
- **Before adding**, search this file and
  [`lessons-archive.md`](lessons-archive.md). A lesson already there is not
  added again: its line ends "(seen again N)", counting up, and an archived
  one is copied back here.
- **At most 30 lessons**; `just check` refuses more. The session that reaches
  the limit moves the oldest lessons with no "seen again" to the archive.
- **Monthly prune**: the session doing the first Monday usage read of each
  month puts any lesson seen again three times to the owner as a rule, and
  archives what is stale or done. A lesson becomes a rule only with the
  owner's yes, and then moves to the archive with "rule:" and the rule's
  file in front.
- The archive is never printed at session start and nothing is deleted from
  it.
- A worktree prints its own copy, so one behind main misses newer lessons.
- Two branches adding lessons at once conflict at the end of the file; keep
  both sides.

## Lessons

- 2026-11-12, #2801: `just claim` refuses in the main checkout, so make the worktree first and claim from it, whatever order the session's brief gives.
- 2026-11-12, #2801: rebasing a branch of several commits onto a main that changed the same lines conflicts once per commit; squash it to one commit first and resolve once.
- 2026-11-12, #2801: in auto mode, the rtk hook (this machine only) rewrites some commands after review, so the first try is refused; resend the rewritten command exactly as the refusal quotes it.

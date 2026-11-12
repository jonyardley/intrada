# Lessons

> What sessions learned, so the next one does not relearn it (#2801). Every
> session start prints this file. Each pull request adds two or three lessons
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
  one moves back here.
- **At most 30 lessons**; `just check` refuses more. The session that reaches
  the limit moves the oldest lessons with no "seen again" to the archive.
- **Monthly prune**: the session doing the first Monday usage read of each
  month puts any lesson seen again three times to the owner as a rule, and
  archives what is stale or done. A lesson becomes a rule only with the
  owner's yes, and then leaves this file.
- The archive is never printed at session start and nothing is deleted from
  it.
- Two branches adding lessons at once conflict at the end of the file; keep
  both sides.

## Lessons

- 2026-11-12, #2801: `just claim` refuses in the main checkout, so make the worktree first and claim from it, whatever the handover's order says.
- 2026-11-12, #2801: `gh issue view N --comments` can print nothing; `--json body,comments` reads the same issue reliably.
- 2026-11-12, #2801: an epic has no plan comment of its own; the brief is its next open child's issue, so read that before planning.

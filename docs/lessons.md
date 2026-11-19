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

- 2026-11-12, #2801: `just claim` refuses in the main checkout, so make the worktree first and claim from it, whatever order the session's brief gives (seen again 1).
- 2026-11-12, #2801: rebasing a branch of several commits onto a main that changed the same lines conflicts once per commit; squash it to one commit first and resolve once.
- 2026-11-12, #2801: in auto mode, the rtk hook (this machine only) rewrites some commands after review, so the first try is refused; resend the rewritten command exactly as the refusal quotes it (seen again 1).
- 2026-11-12, #2820: a GitHub label description holds at most 100 characters, and `gh label create` refuses a longer one with a 422; count before sending.
- 2026-11-12, #2820: the #2803 plan page numbers the session side slice 3 and the spec calls it slice 2; name a slice by its issue number, never its position.
- 2026-11-13, #2799: the crew spec called the loop overnight, and a comparison page repeated it as if the owner had chosen it; a framing word in a spec is not a decision, so check it with the owner before building on it.
- 2026-11-13, #2791: a file removed with `git rm` stays staged, so a later `git add <dir>` commit for another layer sweeps it in; twice a store commit took core deletions. Commit a layer with `git commit -- <paths>`, or `git reset` first.
- 2026-11-13, #2791: one session removing a field from the core and both apps reached about 500k context before the shells were done; hand one shell's test repair to an agent as soon as the core is reviewed.
- 2026-11-13, #2791: `just ios-snapshots-record` wants the bare class name; `IntradaTests/Class` is read as a method, records nothing and still prints "0 passed, 0 failed".
- 2026-11-13, #2821: a finished `claude --bg` session reads state blocked with no pid, and one still running a background command reads working until it ends; read the turn's end from the state, never from the pid.
- 2026-11-13, #2821: on Claude Code 2.1.323 a call waiting on permission reaches the transcript only once the session is stopped, so read a stuck session's calls after the stop.
- 2026-11-13, #2821: a self-test whose default caps run to minutes turns one surviving mutant into a ten minute wait; keep every timing default in a test to seconds.
- 2026-11-14, #2866: lowering an agent's effort pin makes `guard-spawn.sh` refuse every spawn above it, so a pin that is lifted on some diffs needs an `effort-ceiling:` beside it, or the lift is blocked.
- 2026-11-14, #2866: a guard mutation that removes a line can stay green because the line it guards is shadowed (the first frontmatter match wins); give each mutation a fixture where only that line decides the result.
- 2026-11-15, #2869: a `--settings` flag that holds any sandbox list drops every sandbox list in the worktree's local settings (Claude Code 2.1.325), so a loop that passes `strictAllowlist` with its domains must pass osti's lists too; probe a sandbox change live, since the session starts without complaint.
- 2026-11-15, #2869: a sandbox path must be the real one: `/var/folders` and `/tmp` are links, and an allowance written through them is ignored; use `/private/var/...` and `/private/tmp/...`.
- 2026-11-15, #2869: inside Claude's sandbox Xcode cannot start its own for package manifests (`sandbox_apply: Operation not permitted`), even with the packages fetched beforehand; xcodebuild takes the switch as `-Key=YES`, not `-Key YES`.
- 2026-11-15, #2876: a workflow command property value holding a comma needs `%2C`, or the annotation title stops at the comma; assert the escaped string in the test.
- 2026-11-15, #2876: a scheduled job on the self-hosted Mac shares the gate's warm workspace, so a default checkout there would wipe `target/`; fetch only what it needs into a subfolder with `clean: false`.

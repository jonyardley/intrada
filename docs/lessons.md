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
- **Acting on a lesson** (#2934), at the same retro, each lesson gets one
  outcome:
  - **Fixed here**: a change of about 30 lines or fewer to the rule, doc,
    skill, script or recipe that would have prevented it, off the paths in
    `.claude/rules/sensitive-surfaces.md` and never app code, in its own
    commit in the same PR. The lesson goes straight to the archive with
    "fix:" and the changed file in front, so this file keeps only what is
    still open.
  - **Issue**: a bigger fix, or one on a sensitive surface, gets an issue,
    and the lesson stays here ending "(issue #N)" until that issue closes.
  - **Kept**: no clear fix, so it stays here for the monthly prune.
  A fixed lesson that comes back is copied back here with "(seen again N)":
  the fix did not hold.
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
- 2026-11-13, #2806: a fake gh that only writes its message to standard error hides how the real one fails; a failed `gh api` read prints the JSON body on standard output with no newline and the "gh:" message joined on, so run one real failing read before writing the fake.
- 2026-11-13, #2847: a self-test fixture passed with the code it tests removed, because the check refused it for another reason; remove each piece of a new check in turn and watch its own case fail.
- 2026-11-13, #2847: a session parked on a permission prompt for `rm -r` on a scratch folder; make a fresh `mktemp -d` folder per run and leave it for the job's own cleanup.
- 2026-11-13, #2847: `sed -E` on macOS rejected a backslash-heavy mutation and wrote an empty script, so the self-test's failure looked like a caught mutant; swap literal text with perl's `\Q...\E` and diff before running.
- 2026-11-14, #2866: lowering an agent's effort pin makes `guard-spawn.sh` refuse every spawn above it, so a pin that is lifted on some diffs needs an `effort-ceiling:` beside it, or the lift is blocked.
- 2026-11-14, #2866: a guard mutation that removes a line can stay green because the line it guards is shadowed (the first frontmatter match wins); give each mutation a fixture where only that line decides the result.
- 2026-11-15, #2869: a `--settings` flag that holds any sandbox list drops every sandbox list in the worktree's local settings (Claude Code 2.1.325), so a loop that passes `strictAllowlist` with its domains must pass osti's lists too; probe a sandbox change live, since the session starts without complaint.
- 2026-11-15, #2869: a sandbox path must be the real one: `/var/folders` and `/tmp` are links, and an allowance written through them is ignored; use `/private/var/...` and `/private/tmp/...`.
- 2026-11-15, #2869: inside Claude's sandbox Xcode cannot start its own for package manifests (`sandbox_apply: Operation not permitted`), even with the packages fetched beforehand; xcodebuild takes the switch as `-Key=YES`, not `-Key YES`.
- 2026-11-15, #2876: a workflow command property value holding a comma needs `%2C`, or the annotation title stops at the comma; assert the escaped string in the test.
- 2026-11-15, #2876: a scheduled job on the self-hosted Mac shares the gate's warm workspace, so a default checkout there would wipe `target/`; fetch only what it needs into a subfolder with `clean: false`.
- 2026-11-15, #2531: zsh does not split an unquoted expansion, so `${VAR:+--flag "$VAR"}` reaches the command as one argument; write the flag and its value as two words, and leave the flag off with a separate command when the value is empty.
- 2026-11-15, #2531: an agent's `just ios-test` built but ran no test, and it reported "compiles"; a build report is not a test run, so the lead's own gate run decides.
- 2026-11-15, #2531: GitHub allows at most 100 sub-issues per issue, so `just epic-add` refuses on the Android epic; a deferred issue for it stands alone until the epic is split.
- 2026-11-18, #2845: the plan said to reuse a helper in the status script, but the script defined it below the line that needed it; check where a shell function is defined before calling it from earlier in a script.
- 2026-11-18, #2845: for a one-line script fix, reverting the fixed line and rerunning the self-test showed both new assertions fail in one command, cheaper than any mutation tool.
- 2026-11-18, #2793: deleting whole snapshot classes by guess missed two split view references that also showed the changed words; grep every snapshot test file for each changed view, split views included, before deleting references.
- 2026-11-20, no issue: a claude.ai link from an earlier session can be a Claude Doc; the Artifact read returns only the type's instructions, so read the doc's tab through the Claude Docs connector with a search view.
- 2026-11-20, no issue: `wt switch -c name origin/main` prints usage and makes nothing; `just worktree-new name` makes the worktree from fresh origin/main.

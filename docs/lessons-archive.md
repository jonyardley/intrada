# Lessons archive

> Lessons pruned from [`lessons.md`](lessons.md), kept so their insight is not
> lost (#2801). No session start prints this file; search it when a problem
> looks familiar. The retro and the cap append here, newest
> last, each line as it stood, with "fix:" and the changed file in front when
> the retro fixed it (#2934), "issue:" when its issue closed, or "rule:" and the
> rule's file when it became one.

## Archived
- 2026-11-12, #2801: rebasing a branch of several commits onto a main that changed the same lines conflicts once per commit; squash it to one commit first and resolve once.
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
- fix: docs/worktrees.md, 2026-11-24, #2955: a helper launched into its own worktree from the main checkout can build and push, but the branch stays checked out in the builder's worktree, so a later helper runs `git switch --detach origin/<branch>` and pushes `HEAD:<branch>`, and `just pr-open` runs from the builder's worktree because it reads the branch name.
- fix: docs/ios-testing.md, 2026-11-24, #2926: two lanes timing builds on the same Mac skew each other, and the self-hosted CI runner shares that Mac; time builds only once the other lane's gates and CI are done.
- fix: docs/harness-log.md, 2026-11-20, #2934: the harness log's first read for the lessons file was due 2026-11-16 and is still blank, and nothing prompts it; check a harness change's read date before building on that change.

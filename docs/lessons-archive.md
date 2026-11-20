# Lessons archive

> Lessons pruned from [`lessons.md`](lessons.md), kept so their insight is not
> lost (#2801). No session start prints this file; search it when a problem
> looks familiar. The retro, the cap and the monthly prune append here, newest
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

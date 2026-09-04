---
name: intrada-parallel-streams
description: "Rules for running more than one coding-agent session against this repo at once: the decoupled file set a second stream may use, the serialisation points that must never be edited in parallel, worktree-per-agent mechanics, one-agent-per-vertical-slice, when to fan out, and the definition of done before requesting review. MUST read before starting a second concurrent stream, fanning work out to subagents, or coordinating parallel worktrees."
---

## Parallel work streams (agentic sessions)

Rules for running more than one Claude Code session against this repo at once.
Evidence base: coupling analysis of the last 400 commits (2026-08).

The claim protocol in Always(1) is what stops two streams building the same
issue; these rules stop two streams colliding in the same *files*. Both apply.

### Stream rules

- **Exactly one core+iOS vertical stream at a time.** 31% of core commits also
  touch `ios/`; two concurrent vertical features will collide.
- A **second stream** may run only in the decoupled set: `crates/intrada-api`,
  `docs/`, `specs/`, `design/`, or CI/tooling (`justfile`,
  `.github/workflows/`). An API task that needs a new domain field is a core
  change: it joins the vertical stream.
- **Serialisation points.** If your task and another live branch both touch one
  of these, serialise rather than parallelise:
  `crates/intrada-core/src/app.rs`, `crates/intrada-core/src/domain/session.rs`,
  `ios/IntradaTests/ScreenSnapshotTests.swift`,
  `ios/Intrada/DesignSystem/PreviewSupport.swift`, `ios/project.yml`, and
  `Cargo.lock` (never pair anything with a dependency bump).
- One git worktree per stream, branched from fresh `origin/main`. Follow the
  simulator safety rule under Commands. Close the second session when its task
  ships; do not keep it warm.
- **Clear a "conflicting" PR by merging main in, never by rebasing.**
  `git fetch origin main && git merge origin/main && git push`. Feature
  No tracked file is written by every PR any more, so a conflict now means two
  branches really did touch the same code.
- **Dependent PRs stack natively, depth 2 max.** Open the child PR with base =
  the parent's branch; GitHub retargets it to main when the parent merges.
  After the parent squash-merges, rebase the child:
  `git rebase --onto origin/main <parent-old-head>`. No stacking tooling.

### One agent per slice; fan out only on independent work

**A vertical slice is one agent's job.** Do not split core and iOS across two
agents working the same slice. In-session agent teams were tried on #1223 and
retired: on a slice coupled by a bridge contract the split caused the worst bug
in the PR, because the shell teammate couldn't see the core invariant it needed.
The measured post-mortem is in `docs/reference.md`.

**Fan out to worktrees when the pieces are genuinely independent** — no shared
contract in flight, no piece blocked on another's output. Good shapes: an audit
or migration sweep across many files, N independent approaches to one design
question, or unrelated tasks in the decoupled set. Bad shape: anything where two
agents would edit either side of one contract.

- **One git worktree per agent**, branched from fresh `origin/main`. Separate
  checkouts mean no shared-index hazard, so a bare `git commit` is safe — the
  explicit-pathspec rule existed only for the retired shared-checkout teams.
- **The lead integrates.** Fan-out agents report; they do not merge into each
  other's work. Reconcile in one place.
- **The simulator is machine-global.** Only one agent runs iOS tests at a time.
- **Choosing between `isolated: true` and a real worktree** (measured 2026-09-04):
  OMP's isolated workspace is an APFS clone with its own `.git`, not a git
  worktree. It carries `target/` (2.6 GB, so cargo starts warm) but **not**
  `ios/build`, so an iOS build there is cold, 5 to 10 minutes. Because
  `git-dir` equals `git-common-dir` in a clone, the graphify hooks' worktree
  guard does not fire, and every commit inside one kicks a full graph rebuild
  that is thrown away with the workspace. So: `isolated: true` for the
  decoupled set and core-only Rust; `just worktree-new` for anything touching
  `ios/`, which also gets the seeded `ios/build` caches (#1205).
- Mechanics for git worktrees: Claude Code's `using-git-worktrees` skill;
  OMP's native `isolated` option on `task` makes the field available per task
  item, it does not force isolation on a spawn that omits it.

**Contract before code applies to one agent as much as to several.** Pin the
Event/Effect/ViewModel shape for a slice before wiring either side — that
discipline is what makes bridge changes reviewable, not a handoff protocol.

### Definition of done (every stream, before requesting review)

- [ ] `just check` green locally; `just ios-fmt-check` too if `ios/` touched
- [ ] Tests shipped with the new code (see Testing)
- [ ] PR opened via the pre-push gate (checks + self-review); self-review
      comment posted
- [ ] Codecov compared against the PR's Coverage line (Tier 2+)
- [ ] Roadmap updated if a phase changed; deferred items tracked as issues
      (there is no status file to update)
- [ ] A human reviews and merges. Agents never merge.


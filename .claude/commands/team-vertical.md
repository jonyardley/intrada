---
description: Brief an agent team on a core+iOS vertical slice (one team = one stream)
argument-hint: <issue numbers or a one-line slice description>
---

Create an agent team for this vertical slice: $ARGUMENTS

Read the issue(s) with `gh issue view` before briefing the teammates, and
follow CLAUDE.md, Parallel work streams, Agent teams. Team shape:

- First, create a feature branch from fresh `origin/main`. The whole team
  works on that one branch and ships one PR.
- Teammate **core** owns `crates/intrada-core` (and any bindings regen).
  Works TDD (superpowers:test-driven-development). Before writing code, it
  **commits** the Event/Effect/ViewModel contract for the slice to the branch
  (a short `.md`, or the bare Rust type definitions) and then messages
  **ios** that it landed. A message-only contract has gone missing mid-session
  before (#1199); a committed file survives that and doubles as PR
  documentation.
- Teammate **ios** owns `ios/Intrada` and `ios/IntradaTests`. Builds from
  the design-system primitives (check `ios/Intrada/DesignSystem/` and
  `Views/Components/` before drawing anything new). May scaffold UI against
  stub data immediately, but waits for the committed core contract before
  wiring the store.
- Single-writer files: only **core** touches
  `crates/intrada-core/src/app.rs` and `domain/*`; only **ios** touches
  `ios/IntradaTests/ScreenSnapshotTests.swift` and `ios/project.yml`.
  Any change to a bridge-crossing type is messaged to the team before it
  lands.
- Gates before a teammate marks a task done: `just check` for core;
  `just ios-fmt-check` and `just ios-test` (the fast unit/snapshot tier —
  `ship` runs the full `ios-test-full` gate once, at the end) for ios. Use
  the test-runner subagent to keep the logs out of context. Never hand-run
  `just ios-test`/`ios-test-full` again after another teammate already ran
  one green at the current HEAD on a clean tree (#1199) — their green-stamp
  (#1192) already skips a same-HEAD rerun automatically, so doing it by hand
  only wastes a build. `just check` has no such stamp; re-running it is
  cheap (Rust-only) but still redundant once another teammate has run it
  green at the same HEAD.
- **Idle teammate drafts ship work.** Whoever finishes their task list first
  doesn't sit idle waiting for the other — it drafts the PR body, any
  deferred-issue text, and the roadmap edit as task notes for the lead
  (#1199), so the lead isn't doing all of that serially once both land.
- Finish with one PR via the `ship` skill. Its code-reviewer streams each
  Critical/Important finding to the lead as soon as that finding is
  confirmed, rather than holding everything for one batch report at the
  end, and can start reviewing whichever half of the slice landed first
  instead of waiting for both teammates to finish (#1199). Never merge; a
  human reviews.

If the slice is core-only or ios-only, collapse to one teammate plus a
reviewer teammate rather than forcing the two-role split.

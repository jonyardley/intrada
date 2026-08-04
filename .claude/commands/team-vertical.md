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
  Works TDD (superpowers:test-driven-development). Before writing code,
  it publishes the Event/Effect/ViewModel contract for the slice as a task
  note and messages it to **ios**.
- Teammate **ios** owns `ios/Intrada` and `ios/IntradaTests`. Builds from
  the design-system primitives (check `ios/Intrada/DesignSystem/` and
  `Views/Components/` before drawing anything new). May scaffold UI against
  stub data immediately, but waits for the core contract before wiring the
  store.
- Single-writer files: only **core** touches
  `crates/intrada-core/src/app.rs` and `domain/*`; only **ios** touches
  `ios/IntradaTests/ScreenSnapshotTests.swift` and `ios/project.yml`.
  Any change to a bridge-crossing type is messaged to the team before it
  lands.
- Gates before a teammate marks a task done: `just check` for core;
  `just ios-fmt-check` and `just ios-test` for ios. Use the test-runner
  subagent to keep the logs out of context.
- Finish with one PR via the `ship` skill. Never merge; a human reviews.

If the slice is core-only or ios-only, collapse to one teammate plus a
reviewer teammate rather than forcing the two-role split.

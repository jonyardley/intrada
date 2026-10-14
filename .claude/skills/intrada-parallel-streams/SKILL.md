---
name: intrada-parallel-streams
description: Running more than one Claude Code session or subagent against this repo at once: two streams by default and a third only when it is shell-only, up to four Android lanes beside them, the decoupled file set a second stream may use, the serialisation points never edited in parallel, one agent per vertical slice, worktree mechanics, and the definition of done. Read before starting a second stream or fanning out.
---

## Stream rules

`just claim` stops two streams building the same issue; these rules stop two
streams colliding in the same files.

- **Two streams by default, at most one touching `crates/intrada-core` or
  `crates/intrada-ffi`. A third only when it touches `ios/` and no crate, and
  the session says so when it starts one** (#1839): more streams queue on the
  simulator and drift to Tier 1 work. Each shell-only stream names the screens
  it owns before it starts. Editing runs concurrently, testing does not: test
  runs serialise on app launch whatever device they name (#1621;
  `docs/ios-testing.md`, "Running alongside another checkout (worktrees)").
- **Android runs up to four lanes of its own beside the iOS streams** (#2220),
  because its JVM tests need no device. The lanes and their order are in
  #2220. An Android lane needing a core change takes the one core slot shared
  with every iOS stream; it never runs a core change of its own alongside one.
- **Android serialisation points**, shared by every live Android lane:
  `android/app/src/test/kotlin/com/intrada/android/BridgeRoundTripTest.kt`
  and `Fixtures.kt` beside it, `android/app/src/main/kotlin/com/intrada/android/ui/Theme.kt`,
  the navigation files (`AppFrame.kt`, `AppTab.kt`, `MainActivity.kt`), and
  the Gradle build and version catalog files (a new library is a dependency
  change). A new screen keeps its wiring there to one line.
- A **stream touching neither the core crates nor `ios/`** keeps to the
  decoupled set: `docs/`, `specs/`, `design/`, or CI and tooling (`justfile`,
  `.github/workflows/`).
- **Core serialisation points**: `crates/intrada-core/src/app.rs`,
  `crates/intrada-core/src/domain/session/`, `Cargo.lock`. Serialise with any
  live branch touching one, and never pair anything with a dependency bump.
- **Shell serialisation points**, which govern every screen, so a stream
  touching one serialises with every other live stream:
  `ios/IntradaTests/SnapshotTestCase.swift`, the preview fixtures in
  `ios/Intrada/DesignSystem/` (`PreviewStores.swift`,
  `LibraryPreviewFixtures.swift`, `SessionPreviewFixtures.swift`),
  `ios/Intrada/DesignSystem/Theme.swift`, `ios/project.yml`. About half of
  shell-only pairs collide here, so check these files before assuming two are
  independent.
- **One worktree per session, made by that session** with
  `just worktree-new <name>`, which branches from fresh `origin/main` and
  seeds `ios/generated` and the Swift packages (#2191). The session stays in
  the main checkout and prefixes every shell command with `cd <worktree> && `
  (#1720, #1837; `docs/working-with-agents.md`). Close it when its task ships.
- **Once you have a worktree, edit only inside it.** A green run proves
  nothing about whose work is in the tree: read the diff before `git add`,
  and never `git add -A` on a shared checkout.
- **Clear a conflicting PR by merging main in, never rebasing**:
  `git fetch origin main && git merge origin/main && git push`.
- **Dependent PRs stack, depth 2 at most**, the child based on the parent's
  branch. After the parent squash-merges:
  `git rebase --onto origin/main <parent-old-head>`.

## One agent per slice

**A vertical slice is one agent's job.** Never split core and iOS across two
agents: the shell agent cannot see the core invariant it needs (#1223).

Fan out only for independent pieces: no shared contract in flight, nothing
blocked on another's output, such as a sweep across many files or unrelated
tasks in the decoupled set. One worktree per agent; the lead integrates and
runs `just check` and the suites once at the end; one agent runs iOS tests at
a time.

**Contract before code**, for one agent as much as several: pin the
Event, Effect and ViewModel shape before wiring either side.

## Definition of done, before requesting review

`/ship`, the `intrada-shipping` skill and the PR template checklist hold it.

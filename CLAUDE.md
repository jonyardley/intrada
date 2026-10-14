# intrada Development Guidelines

<!-- Loads into every session: invariants, gotchas, commands, about 430 words. Everything
else lives in the path-scoped rules, the skills and the docs below. Trial from 2026-10-13
(#2595); the undo rule is in docs/harness-log.md. -->

intrada is a **practice notebook** for musicians: plan a session, play it, score it. Two
native shells on one Rust core: SwiftUI in `ios/`, Compose in `android/`; every feature
ships on both. The web and Tauri shells were deleted (`docs/decisions.md`): **a request
implying web work? Confirm the platform first.** Direction: `docs/roadmap.md`; in flight: `just status`.

## Commands

```bash
just check            # CI's Rust and hygiene jobs, bar MSRV and coverage; run before pushing
just ios-run          # build + launch on simulator + screenshot (seeded; SEED=0 to persist)
just ios-test         # unit + snapshot; just ios-fmt-check for ios/
just android-run      # regen bindings + emulator + screenshot; android-test: JVM + snapshots
```

- **Drive iOS through the `just` recipes, never a bare `xcodebuild` or an MCP build, run or
  test call** (#1536, #1537); `RenderPreview` is the one exception. A passing run prints its
  counts; silence is never the evidence.
- **Read source with the Read tool, not `cat`**: path-scoped rules fire on Read.
- Mechanics: [`docs/reference.md`](docs/reference.md), [`docs/ios-testing.md`](docs/ios-testing.md).

## Architecture (non-negotiables)

```text
User → Events → crux_core (Rust) → Effects (Persistence, App, Render, Recognition, Sync) → Shell → I/O
```

1. **Core owns all logic.** No business rules, validation or domain state in Swift or Kotlin,
   only UI interaction state; if tempted, it is an `Event` or `Command` in `intrada-core`.
2. **The shell is a dumb pipe.** It renders the `ViewModel` and fulfils effects: local data
   through the shared `intrada-store`, crash recovery through an `AppEffect`.
3. **Typed bindings, no hand-written FFI.** Never hand-edit `ios/generated/` or
   `android/generated/`; fix the Rust type and regenerate both.

## Gotchas

- **JSON-only serde attrs break the bincode bridge silently** (#846); a stub-bridge test
  cannot catch it: use `LiveBridge`.
- **A field inside the crash-recovery snapshot invalidates every blob** (#1345): bump
  `ActiveSession::BLOB_VERSION` first, then re-pin; the shell's key follows it (#1116).

## Where the rest lives

- Code, comments and tests: `.claude/rules/code-and-tests.md`; offline-first, UI, quality
  bars and silent-failure surfaces: the other `.claude/rules/`. They do not load in a
  worktree: read them by hand before editing. Every rule binds either way.
- Claim, worktree, tiers, plan comments and routing (unplanned work: discuss first):
  [`docs/working-with-agents.md`](docs/working-with-agents.md); epics:
  [`docs/how-the-work-runs.md`](docs/how-the-work-runs.md#epics).
- Decisions whose records were removed: [`docs/decisions.md`](docs/decisions.md).
- Shipping: `/ship` and the `intrada-shipping` skill; more than one session: the
  `intrada-parallel-streams` skill.
- Writing: British English, no dashes, plain language, in PR bodies and issues too
  ([`docs/style-guide.md`](docs/style-guide.md), [`docs/tone-of-voice.md`](docs/tone-of-voice.md)).

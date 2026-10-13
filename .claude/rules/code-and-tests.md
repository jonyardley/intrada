---
paths:
  - "crates/**"
  - "ios/**/*.swift"
  - "android/**/*.kt"
---

# Code style and tests

Binds every code file whether or not this loaded (moved from `CLAUDE.md`,
#2595).

## Code style

- `cargo fmt` and `cargo clippy -- -D warnings` pass. No `unwrap()` without
  justification. Prefer established libraries over custom implementations.
  Read every compile error before fixing the first: `cargo check --all-targets`,
  and the full `just ios-test` error list.
- **Code with no reader gets deleted, not parked** (#1176): not
  `#[allow(dead_code)]`, not "inert until the feature returns", not a `pub`
  nobody calls. `git` is the parking space; keep the findings and name the
  recovery PR. A stub a test reads, or an API a shell calls, has a reader.
- **Comments: default to none.** The density gate counts them; the kind is
  yours. Only a one-line section header, a non-obvious WHY citing an issue or
  incident, or `HACK(#N)` / `FIXME(#N)`. Never one that restates the code or
  narrates the task. Over two lines, ask whether it should be a name or a type.
- **Core**: validation lives in `crates/intrada-core/src/validation.rs`. Writes
  commit locally with no refetch: a temp id for new entities,
  `*Updated { entity }`, `DeleteConfirmed`. Persistence is a core `Effect`
  driven by `Command`; the store executes typed effects and the core decides
  reads, writes and last-write-wins reconciliation.
- **Swift**: an `@Observable @MainActor` store, not `ObservableObject`; effect
  handlers run off the main actor and hop back. `try!`, force-unwraps and `as!`
  are banned like `unwrap()`. Singletons go to UserDefaults through an
  `AppEffect`, each blob under a versioned key with a Rust wire pin. Every
  colour, font, spacing and radius is a named token from `Theme.swift`.

## Tests

**Ship tests with new code**, test-first for non-UI Tier 2, all Tier 3 and
`intrada-core` changes: a test retrofit to pass agrees with the implementation
by construction (#1256). Non-trivial pure logic covers edge cases, None and
empty inputs. New iOS test files use Swift Testing; migrate an XCTest file only
when already touching it; XCUITest stays on XCTest.

- **Before asserting, ask what the value was one line earlier** (#1223); if
  nothing distinguishes the behaviour present from absent, delete the test.
  **Mutation-test by deleting the line, not inverting it** (#1423).
- **A parser or validator gets a table test against its consumer** from inputs
  a user would produce (#1256). Fixtures for a many-field type live in one
  place (Rust `fixture()` with struct update, Swift a fixture enum with default
  arguments).
- **Say so in the PR when you skip tests.** Tier 2+ carries a **Coverage** line
  naming expected gaps, then checks the Codecov comment against it (70% patch
  target, informational; `ios/` ignored).
- **UI verification means driving the app on the simulator**; if you cannot,
  name the hand check.

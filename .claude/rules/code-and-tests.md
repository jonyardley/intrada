---
paths:
  - "crates/**"
  - "ios/**/*.swift"
  - "android/**"
---

# Code style and tests

## Code style

- `cargo fmt` and `cargo clippy -- -D warnings` pass. No `unwrap()` without
  justification. Prefer established libraries over custom code. Read every
  compile error before fixing the first: `cargo check --all-targets`, and the
  full `just ios-test` error list.
- **Code with no reader gets deleted, not parked** (#1176): no
  `#[allow(dead_code)]`, no "inert until the feature returns", no `pub` nobody
  calls. `git` is the parking space: name the recovery PR. A stub a test reads, or an API a shell
  calls, has a reader.
- **Comments: default to none.** The density gate counts them; the kind is
  yours. Only a one-line section header, a non-obvious WHY citing an issue or
  incident, or `HACK(#N)` / `FIXME(#N)`. Never one that restates the code or
  narrates the task. Over two lines, ask whether it should be a name or a type.
- **Core**: validation lives in `crates/intrada-core/src/validation.rs`. Writes
  commit locally with no refetch: a temp id for new entities,
  `*Updated { entity }`, `DeleteConfirmed`. Persistence rules:
  `offline-first.md`.
- **Swift**: an `@Observable @MainActor` store, not `ObservableObject`; effect
  handlers run off the main actor and hop back. `try!`, force-unwraps and `as!`
  are banned like `unwrap()`. Tokens: `ios-ui.md`.

## Tests

**Ship tests with new code**, test-first for non-UI Tier 2, all Tier 3 and
`intrada-core` changes: a test retrofitted to pass agrees with the code by
construction (#1256). Cover edge cases, None and empty inputs. New iOS test
files use Swift Testing; migrate an XCTest file only when already touching it;
XCUITest stays on XCTest.

- **Before asserting, ask what the value was one line earlier** (#1223); if
  nothing distinguishes the behaviour present from absent, delete the test.
  **Mutation-test by deleting the line, not inverting it** (#1423).
- **A parser or validator gets a table test against its consumer** from inputs
  a user would produce (#1256); invented cases agree with your code by
  construction. Fixtures for a many-field type live in one place (Rust
  `fixture()` with struct update, Swift a fixture enum with default arguments).
- **Say so in the PR when you skip tests**, in the **Coverage** line the
  shipping skill describes.

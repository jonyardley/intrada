---
paths:
  - "crates/intrada-ffi/**"
  - "ios/generated/**"
  - "crates/intrada-core/src/**"
  - "crates/intrada-store/**"
  - "ios/Intrada/Core/SharedItemStore.swift"
---

# You are on a silent-failure surface

A wrong change here does not crash. It decodes into a plausible wrong value,
drops a write, or destroys the only copy of a user's data. Never spell out an
exploitable gap in a public PR body: say a gap exists and route the detail to
Jon. A diff is on this surface when it changes a type that crosses the bridge
(an item deriving Facet: view types, `Event` and its sub-events, effect
operations), a `BLOB_VERSION` line, the store or the FFI source, wherever the
file sits, and `just review-route` names them (#2922); auth is judged by hand. Before editing:

**Additive or not.** An additive bridge change adds a field, an event variant
or a view type and renames, removes, reorders or retypes nothing. It keeps its
tier, may be built by `task` or the lead, and ships with its LiveBridge
round-trip test. Everything else here is lead-only: the `ActiveSession`
crash-recovery blob, any type under a wire-pinned settings key (library sort,
profile, practice defaults, first run: bincode on the device, like the blob), store
migrations, auth, and any bridge change that is not additive.

1. **Lead-only work runs at the session's opening level**, which is the
   level the role table in `~/.claude/CLAUDE.md` gives sensitive surfaces
   (Subagents, review and lane size); nobody types `/effort`. If
   `.claude/settings.json` and that table's sensitive-surface row ever differ,
   the table wins and a PR corrects the settings file.
2. **Pair the `reviewer` agent on the core diff before the screens**, whoever
   built it, spawned at the model and effort that table gives a second reviewer.
3. **One PR across the stack**: core commits first, `reviewer` reads them alone,
   then the screens commits. Lead-only work goes up at least one tier in
   ceremony but is still one PR: a merged core with no caller is shell-dead
   (#1374).

The hazards, by file:

- **Bridge types (`intrada-ffi`, anything crossing the bridge).** The wire is
  positional bincode with no "absent". `deserialize_with`, `serialize_with`
  and `skip_serializing_if` on a non-trailing field are a silent no-op, not a
  crash (#846). Branch on `Deserializer::is_human_readable()` for JSON-only
  behaviour. Before a screen reads the type, cover it with the Rust
  `assert_round_trips` helper and `LiveBridge` (`LibraryBridgeTests` or
  `SessionBridgeTests`).
- **`ios/generated/`.** Never hand-edit; fix the Rust type and regenerate.
  UniFFI output fails under Swift 6.2 `MainActor`-default isolation
  (uniffi-rs#2818); the build recipe keeps the package non-MainActor-defaulted.
- **`domain/session/`.** A new field anywhere in the `ActiveSession` graph
  invalidates every crash-recovery blob on every device (#1345).
  `active_session_blob_wire_is_pinned` fails on purpose: bump
  `ActiveSession::BLOB_VERSION` first, then re-pin; the shell's key follows it
  (#1116). Never only re-pin.
- **`crates/intrada-store`.** Testers' notebooks open in it; its migrations
  are recorded in GRDB's `grdb_migrations` table: append-only, per
  `.claude/rules/offline-first.md`.

## Where stored and bridged types are built by hand

Research for a change to a stored or bridged type, and its lane brief, list
each place below that the change reaches:

- **Store codec**: `StoredPlay`, `stored_play` and the reader's `plays` and
  `play`, which also build a `Play` from a legacy row, in
  `crates/intrada-core/src/stored_session.rs`. Plays live inside the session
  row's JSON `entries` column (`crates/intrada-store/src/sessions.rs`), so a
  new play field needs no migration; a new session-level field is a real
  column and does (`.claude/rules/offline-first.md`).
- **Sync body pin**: `record_bodies_are_pinned`, `PINNED_SESSION` and
  `full_session()` in `crates/intrada-core/src/sync/tests.rs`.
- **Wire pins**: `active_session_blob_wire_is_pinned` in
  `crates/intrada-core/src/domain/session/tests.rs`, bumping
  `ActiveSession::BLOB_VERSION` first (the `domain/session/` hazard above);
  `every_stored_enum_text_is_pinned` in
  `crates/intrada-core/src/stored_session/tests.rs`; the settings blob pins in
  `domain/types.rs`, `practice_defaults.rs`, `profile.rs` and `first_run.rs`.
- **Bridge tests allowlist**: `scripts/bridge-tests-allowlist.txt`, checked by
  `just check`. A new bridged type needs a `LiveBridge` test naming it or a
  line there; the line comes off when the type gets its first test.
- **Rust construction sites**: `crates/intrada-store/src/tests.rs` and
  `crates/intrada-core/src/sample.rs` build `Play` in full. New intrada-core
  tests use `Play::fixture()` with struct update; it is `#[cfg(test)]
  pub(crate)`, so other crates cannot reach it.
- **Swift construction sites**: tests and previews calling a generated type's
  memberwise init, such as `ios/Intrada/DesignSystem/PreviewStores.swift` and
  files in `ios/IntradaTests`. Find them with the LSP's find references, or a
  word-boundary search such as `\bPlay\(` plus `.init(` calls, across
  `ios/Intrada` and `ios/IntradaTests`. Shared fixtures live in
  `ios/Intrada/DesignSystem/SessionPreviewFixtures.swift` and
  `LibraryPreviewFixtures.swift`.

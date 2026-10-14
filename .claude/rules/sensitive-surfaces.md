---
paths:
  - "crates/intrada-core/src/domain/session/**"
  - "crates/intrada-ffi/**"
  - "ios/generated/**"
  - "crates/intrada-store/**"
  - "ios/Intrada/Core/SharedItemStore.swift"
  - "android/generated/**"
  - "android/app/src/main/kotlin/com/intrada/android/core/CoreBridge.kt"
---

# You are on a silent-failure surface

A wrong change here does not crash. It decodes into a plausible wrong value,
drops a write, or destroys the only copy of a user's data. Never spell out an
exploitable gap in a public PR body: say a gap exists and route the detail to
Jon. Before editing:

**Additive or not.** An additive bridge change adds a field, an event variant
or a view type and renames, removes, reorders or retypes nothing. It keeps its
tier, may be built by `task` or the lead, and ships with its LiveBridge
round-trip test. Everything else here is lead-only: the `ActiveSession`
crash-recovery blob, any type under a wire-pinned settings key (library sort,
profile, practice defaults), store migrations, auth, and any bridge change
that is not additive.

1. **Lead-only work runs Opus 5.5 at `high`** (#2044, #2052), set with
   `/effort` before the first edit.
2. **Pair the `reviewer` agent on the core diff before the screens**, whoever
   built it.
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
- **`android/generated/` and `CoreBridge.kt`.** The same wire, read by a second
  decoder: extend `BridgeRoundTripTest` as well as the Swift `LiveBridge`
  tests. Generate only through `just android-gen`, which runs the
  `uniffi-bindgen` pinned inside `intrada-ffi`, never a global one, or the
  bridge skews silently.
- **`domain/session/`.** A new field anywhere in the `ActiveSession` graph
  invalidates every crash-recovery blob on every device (#1345).
  `active_session_blob_wire_is_pinned` fails on purpose: bump
  `ActiveSession::BLOB_VERSION` first, then re-pin; the shell's key follows it
  (#1116). Never only re-pin.
- **`crates/intrada-store`.** Testers' notebooks open in it; its migrations
  are recorded in GRDB's `grdb_migrations` table: append-only, per
  `offline-first.md`.

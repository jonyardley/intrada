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
tier, may be built by `task` or the lead, and ships as one PR with its LiveBridge
round-trip test. Everything else here is lead-only: the `ActiveSession`
crash-recovery blob, any type stored under a wire-pinned settings key (library
sort, profile, practice defaults: bincode on the device, like the blob), store
migrations, auth, and any bridge change that is not additive.

1. **Lead-only work runs at the sensitive surfaces' rung**: Opus 5.5 at `high`
   (#2044, #2052), set with `/effort` before the first edit. An additive change
   stays on the lead's usual effort.
2. **Pair the `reviewer` agent on the core diff before the screens**, on its
   Opus pin, whoever built it.
3. **One PR or two.** An additive change ships as one PR, the core commits
   first, and `reviewer` reads them on their own before the screens commits.
   Lead-only work goes up at least one tier in ceremony and ships as two PRs:
   core first, screens in the same working session, or the core PR waits (a
   merged core PR with no caller is shell-dead, #1374).

The hazards, by file:

- **Bridge types (`intrada-ffi`, anything crossing the bridge).** The wire is
  positional bincode with no "absent". `deserialize_with`, `serialize_with`
  and `skip_serializing_if` on a non-trailing field produce a silent no-op,
  not a crash (#846). Branch on `Deserializer::is_human_readable()` for
  JSON-only behaviour, and cover the type with a real-bridge round-trip
  (`LiveBridge` in `LibraryBridgeTests` or `SessionBridgeTests`).
- **`ios/generated/`.** Never hand-edit. Fix the Rust type and regenerate.
  UniFFI output fails under Swift 6.2 `MainActor`-default isolation
  (uniffi-rs#2818); the build recipe keeps the package non-MainActor-defaulted.
- **`android/generated/` and `CoreBridge.kt`.** The same wire as iOS, read by
  a second decoder: extend `BridgeRoundTripTest` to cover the changed
  payload, as well as the Swift `LiveBridge` tests. Never hand-edit the
  generated tree. Generate it only through `just android-gen`, which runs the
  `uniffi-bindgen` built inside `intrada-ffi` at its pinned uniffi version,
  never a globally installed one, or the bridge skews silently.
- **`domain/session/`.** A new field anywhere in the `ActiveSession` graph
  invalidates every crash-recovery blob on every device (#1345).
  `active_session_blob_wire_is_pinned` fails on purpose: bump
  `ActiveSession::BLOB_VERSION` first, then re-pin; the shell's key follows it
  (#1116). Never only re-pin.
- **`crates/intrada-store`.** Testers' notebooks open in it, and its
  migrations are recorded in GRDB's `grdb_migrations` table: append-only, per
  `.claude/rules/offline-first.md`.

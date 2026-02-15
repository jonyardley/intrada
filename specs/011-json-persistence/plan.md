# Implementation Plan: JSON File Persistence

**Branch**: `011-json-persistence` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-json-persistence/spec.md`

## Summary

Replace the SQLite persistence layer in the CLI shell with JSON file storage (`library.json`) and add localStorage persistence to the web shell (`intrada:library` key). The core's `StorageEffect` enum is unchanged — only the shell-side storage implementations change. Remove the `rusqlite` dependency entirely.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2, serde_json 1, web-sys (with Storage+Window features), dirs 5
**Storage**: JSON files (CLI: `~/.local/share/intrada/library.json`), localStorage (web: `intrada:library` key)
**Testing**: `cargo test` (unit tests in storage.rs, shell.rs; existing core tests unchanged)
**Target Platform**: CLI (macOS/Linux), WASM (browser via trunk)
**Project Type**: Workspace with 3 crates (core, cli, web)
**Performance Goals**: N/A — single-user local app, file I/O is negligible
**Constraints**: Atomic writes for CLI (temp file + rename). localStorage 5MB limit (log warning on failure, don't crash).
**Scale/Scope**: Single user, hundreds of items max. Full-file rewrite on each save is acceptable.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | Single-responsibility: `JsonStore` replaces `SqliteStore` with same public API. No dead code — SQLite code fully removed, not commented out. |
| II. Testing Standards | PASS | All existing CLI tests adapted to JSON backend. New tests for round-trip serialisation, missing file, malformed JSON. Tests use temp dirs, not real XDG path. |
| III. UX Consistency | PASS | No user-facing behaviour changes. CLI commands identical. Web app gains persistence (improvement, not inconsistency). |
| IV. Performance | PASS | Full-file rewrite is <1ms for expected data sizes. No N+1 or over-fetching concerns. |

**Post-Phase 1 re-check**: PASS — no new patterns introduced. `LibraryData` struct is a plain serde container, no abstractions.

## Project Structure

### Documentation (this feature)

```text
specs/011-json-persistence/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/
│   └── src/
│       └── domain/
│           └── types.rs          # Add LibraryData struct (shared serialisation unit)
├── intrada-cli/
│   ├── Cargo.toml                # Remove rusqlite, keep serde_json
│   └── src/
│       ├── storage.rs            # REWRITE: JsonStore replaces SqliteStore
│       ├── shell.rs              # UPDATE: use JsonStore instead of SqliteStore
│       └── main.rs               # MINOR: update store construction
└── intrada-web/
    ├── Cargo.toml                # Add web-sys with Storage+Window features
    └── src/
        ├── core_bridge.rs        # UPDATE: persist to/from localStorage (localStorage logic inlined here)
        └── data.rs               # KEEP: stub data still used for first-run seeding
```

**Workspace Cargo.toml**: Remove `rusqlite` from `[workspace.dependencies]`.

**Structure Decision**: Existing 3-crate workspace structure is preserved. No new crates. The `LibraryData` struct lives in `intrada-core` because both shells need it.

## File Change Summary

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` (workspace) | EDIT | Remove `rusqlite` from workspace dependencies |
| `crates/intrada-core/src/domain/types.rs` | EDIT | Add `LibraryData` struct |
| `crates/intrada-core/src/lib.rs` | EDIT | Re-export `LibraryData` |
| `crates/intrada-cli/Cargo.toml` | EDIT | Remove `rusqlite` dependency |
| `crates/intrada-cli/src/storage.rs` | REWRITE | `JsonStore` replacing `SqliteStore` |
| `crates/intrada-cli/src/shell.rs` | EDIT | Update type references from `SqliteStore` to `JsonStore` |
| `crates/intrada-cli/src/main.rs` | EDIT | Update store construction |
| `crates/intrada-web/Cargo.toml` | EDIT | Add `web-sys` with features, add `serde_json` |
| `crates/intrada-web/src/core_bridge.rs` | EDIT | Add localStorage read/write for all StorageEffect variants |
| `crates/intrada-web/src/app.rs` | EDIT | Load from localStorage on init instead of always using stub data |
| `crates/intrada-web/src/data.rs` | KEEP | Unchanged — stub data used for first-run seeding |

# Research: JSON File Persistence

**Feature**: 011-json-persistence | **Date**: 2026-02-14

## R1: Atomic file writes in Rust

**Decision**: Write to a temp file in the same directory, then `std::fs::rename()` to the target path.

**Rationale**: `rename()` is atomic on POSIX filesystems when source and target are on the same filesystem. Writing to a temp file in the same directory guarantees this. The `tempfile` crate is unnecessary — `std::fs::write` to a `.tmp` suffix then `rename` is sufficient for this use case.

**Alternatives considered**:
- `tempfile` crate: Adds a dependency for something achievable in 3 lines of std code.
- Write-in-place: Risk of data corruption if the process is killed mid-write.

## R2: localStorage API in WASM/Leptos

**Decision**: Use `web_sys::window().unwrap().local_storage().unwrap().unwrap()` directly via `web-sys` with the `Storage` feature. No additional crate needed.

**Rationale**: `web-sys` is already a transitive dependency of `leptos` and `wasm-bindgen`. Adding `gloo-storage` would introduce another dependency for a trivial wrapper around `localStorage.getItem()` / `setItem()`. Direct `web-sys` usage is 5-10 lines of code.

**Alternatives considered**:
- `gloo-storage` crate: Clean API but adds a dependency. The `gloo` ecosystem is in flux with version fragmentation.
- `wasm-bindgen` raw JS interop: Lower-level than needed when `web-sys` already exposes the API.

## R3: JSON pretty-print vs compact

**Decision**: Use `serde_json::to_string_pretty()` for CLI file output. Compact format for localStorage (via `serde_json::to_string()`).

**Rationale**: CLI users may inspect `library.json` manually — pretty-printed JSON is friendlier. localStorage has a 5MB limit, so compact is better. The serialisation format difference doesn't matter since both deserialise identically.

**Alternatives considered**:
- Compact everywhere: Harder for users to debug by inspecting the file.
- Pretty everywhere: Wastes localStorage space unnecessarily.

## R4: serde_json handling of unknown fields

**Decision**: No configuration needed. `serde_json::from_str` ignores unknown fields by default when deserialising into a struct. This is the standard serde behaviour.

**Rationale**: Schema evolution via `#[serde(default)]` on new optional fields plus default unknown-field-ignoring means old clients can read newer files and newer clients can read older files. No explicit `#[serde(deny_unknown_fields)]` should be added.

## R5: web-sys features needed

**Decision**: Enable `Storage`, `Window` features on `web-sys` in `intrada-web/Cargo.toml`.

**Rationale**: `web-sys` gates each Web API behind feature flags to minimise WASM binary size. We need `Window` (for `window()`) and `Storage` (for `local_storage()`).

**Alternatives considered**:
- Feature flags are already the standard pattern for web-sys usage. No real alternatives.

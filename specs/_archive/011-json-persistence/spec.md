# Feature Specification: JSON File Persistence

**Feature Branch**: `011-json-persistence`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "Replace SQLite persistence in the CLI shell with JSON file storage and add localStorage persistence to the web shell. Segmented file approach with separate files per data domain."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - CLI persists library data as JSON (Priority: P1)

A musician using the CLI adds pieces and exercises. Their data persists between sessions as a JSON file at `~/.local/share/intrada/library.json` instead of a SQLite database. All existing CLI commands (add, list, show, edit, delete, tag, untag, search) work exactly as before — only the storage backend changes.

**Why this priority**: This is the core change — replacing SQLite with JSON. Without this, nothing else matters.

**Independent Test**: Run `intrada add piece`, exit, run `intrada list` — the added piece appears. Inspect `~/.local/share/intrada/library.json` to confirm it's valid JSON containing the piece.

**Acceptance Scenarios**:

1. **Given** no existing data directory, **When** the user runs any CLI command, **Then** the data directory is created and `library.json` is written (empty library if no data yet)
2. **Given** a `library.json` with 3 pieces and 2 exercises, **When** the user runs `intrada list`, **Then** all 5 items are displayed
3. **Given** a `library.json` with existing data, **When** the user adds a new piece, **Then** the piece is appended and `library.json` is rewritten with the new piece included
4. **Given** a `library.json` with existing data, **When** the user edits an item, **Then** `library.json` is rewritten with the updated item
5. **Given** a `library.json` with existing data, **When** the user deletes an item, **Then** `library.json` is rewritten without that item
6. **Given** no `library.json` exists, **When** `LoadAll` is processed, **Then** an empty library is returned (no error)

---

### User Story 2 - Web shell persists library data to localStorage (Priority: P2)

A musician using the web app adds items. Their data persists across page reloads via the browser's localStorage. The web shell uses the same JSON format as the CLI, stored under domain-specific keys (e.g. `intrada:library`).

**Why this priority**: The web shell currently has no persistence — all data is lost on refresh. This makes the web app actually usable.

**Independent Test**: Open the web app, add a piece, refresh the page — the piece is still there. Check `localStorage.getItem("intrada:library")` in the browser console to confirm valid JSON.

**Acceptance Scenarios**:

1. **Given** no localStorage key exists, **When** the web app loads, **Then** the stub data is used as a starting library and persisted to localStorage
2. **Given** `intrada:library` exists in localStorage, **When** the web app loads, **Then** the persisted data is loaded (stub data is NOT used)
3. **Given** the web app is running, **When** the user adds/edits/deletes an item, **Then** the `intrada:library` key is updated immediately
4. **Given** corrupt or unparseable JSON in localStorage, **When** the web app loads, **Then** an empty library is used and the corrupt data is overwritten on next save

---

### Edge Cases

- What happens when `library.json` is malformed? → CLI prints an error and exits; web shell falls back to empty library.
- What happens when the filesystem is read-only or the data directory can't be created? → CLI returns an error via `anyhow`.
- What happens when localStorage is full (5MB limit)? → Web shell logs a warning to console; the in-memory state remains correct even if persistence fails.
- What happens when a future schema version adds new fields? → `#[serde(default)]` on new optional fields ensures old files load without migration.
- What happens when two CLI processes write simultaneously? → Last write wins (acceptable for a single-user app).
- What happens when `library.json` contains unknown fields from a newer version? → `serde_json` ignores unknown fields by default — the file loads fine.

## Clarifications

### Session 2026-02-14

- Q: Should the CLI use atomic writes (temp file + rename) to prevent corruption on interrupted writes? → A: Yes, atomic write via temp file + rename.
- Q: How should rusqlite be retained for the migrate command? → A: No migrate command needed — early development, no real user data at risk. Remove rusqlite entirely.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: CLI shell MUST read/write `library.json` at `~/.local/share/intrada/` (XDG data path, resolved via `dirs` crate as currently done)
- **FR-002**: Web shell MUST read/write to `localStorage` under the key `intrada:library`
- **FR-003**: Both shells MUST use the same JSON format: `{ "pieces": [...], "exercises": [...] }`
- **FR-004**: On `StorageEffect::LoadAll`, the shell MUST read and deserialise the library file/key. If absent, return an empty library
- **FR-005**: On `SavePiece`, `SaveExercise`, `UpdatePiece`, `UpdateExercise`, `DeleteItem`, the shell MUST update its in-memory collection and write the full library to storage. The CLI MUST use atomic writes (write to a temp file in the same directory, then rename) to prevent corruption on interrupted writes
- **FR-006**: The `StorageEffect` enum in `intrada-core` MUST NOT change
- **FR-007**: The `rusqlite` dependency MUST be removed from `intrada-cli` and the workspace entirely
- **FR-008**: Schema evolution MUST be handled via `#[serde(default)]` on new optional fields — no explicit migration system needed
- **FR-009**: The web shell MUST use stub data ONLY when no persisted data exists in localStorage
- **FR-010**: The JSON file approach MUST accommodate future domain files (e.g. `sessions.json`, `goals.json`) without structural changes — each domain gets its own file/key

### Key Entities

- **LibraryData**: Top-level JSON structure containing `pieces: Vec<Piece>` and `exercises: Vec<Exercise>`. This is the serialisation unit for `library.json` / `intrada:library`.
- **Piece**: Existing domain type (already has Serialize/Deserialize derives). No changes needed.
- **Exercise**: Existing domain type (already has Serialize/Deserialize derives). No changes needed.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All existing CLI tests continue to pass with the JSON backend
- **SC-002**: All existing web shell functionality works with localStorage persistence
- **SC-003**: Data round-trips correctly: CLI write → CLI read, web write → web read, CLI write → manual inspection of JSON
- **SC-004**: `rusqlite` no longer appears anywhere in `Cargo.lock`
- **SC-005**: Adding a new optional field to `Piece` or `Exercise` with `#[serde(default)]` loads old JSON files without error

# Data Model: JSON File Persistence

**Feature**: 011-json-persistence | **Date**: 2026-02-14

## Entities

### LibraryData (NEW)

Top-level serialisation unit for both `library.json` (CLI) and `intrada:library` (web localStorage).

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LibraryData {
    #[serde(default)]
    pub pieces: Vec<Piece>,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}
```

**Location**: `crates/intrada-core/src/domain/types.rs` (shared between shells)

**JSON format**:
```json
{
  "pieces": [
    {
      "id": "01HYK...",
      "title": "Clair de Lune",
      "composer": "Claude Debussy",
      "key": "Db Major",
      "tempo": { "marking": "Andante", "bpm": 66 },
      "notes": "Third movement of Suite bergamasque",
      "tags": ["impressionist", "piano"],
      "created_at": "2026-02-14T10:30:00Z",
      "updated_at": "2026-02-14T10:30:00Z"
    }
  ],
  "exercises": [
    {
      "id": "01HYK...",
      "title": "Hanon No. 1",
      "composer": "Charles-Louis Hanon",
      "category": "Technique",
      "key": "C Major",
      "tempo": { "marking": "Moderato", "bpm": 108 },
      "notes": "The Virtuoso Pianist — Exercise 1",
      "tags": ["technique", "warm-up"],
      "created_at": "2026-02-14T10:30:00Z",
      "updated_at": "2026-02-14T10:30:00Z"
    }
  ]
}
```

### Piece (UNCHANGED)

Existing struct in `crates/intrada-core/src/domain/piece.rs`. Already has `#[derive(Serialize, Deserialize)]`. No changes needed.

### Exercise (UNCHANGED)

Existing struct in `crates/intrada-core/src/domain/exercise.rs`. Already has `#[derive(Serialize, Deserialize)]`. No changes needed.

### Tempo (UNCHANGED)

Existing struct in `crates/intrada-core/src/domain/types.rs`. Already has `#[derive(Serialize, Deserialize)]`. No changes needed.

## Schema Evolution Strategy

New optional fields added to `Piece`, `Exercise`, or `LibraryData` MUST use `#[serde(default)]`:

```rust
// Example future addition:
pub struct Piece {
    // ... existing fields ...
    #[serde(default)]
    pub difficulty: Option<u8>,  // Added in v2 — old files load fine
}
```

Unknown fields in the JSON are silently ignored by serde_json (default behaviour). No `#[serde(deny_unknown_fields)]` anywhere.

## Storage Locations

| Shell | Path/Key | Format |
|-------|----------|--------|
| CLI | `~/.local/share/intrada/library.json` | Pretty-printed JSON |
| Web | `localStorage["intrada:library"]` | Compact JSON |

## Future Domain Files

The segmented approach supports future domains without structural changes:

| Domain | CLI File | Web Key | Load Strategy |
|--------|----------|---------|---------------|
| Library | `library.json` | `intrada:library` | Eager (on startup) |
| Sessions | `sessions.json` | `intrada:sessions` | On demand (future) |
| Goals | `goals.json` | `intrada:goals` | On demand (future) |

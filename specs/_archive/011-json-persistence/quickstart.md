# Quickstart: JSON File Persistence

**Feature**: 011-json-persistence | **Date**: 2026-02-14

## Verification Commands

```bash
# Run all tests (core + CLI + web compile check)
cargo test

# Run only CLI tests
cargo test -p intrada-cli

# Run only core tests
cargo test -p intrada-core

# Check for clippy warnings
cargo clippy -- -D warnings

# Verify rusqlite is gone from the dependency tree
cargo tree -p intrada-cli | grep -i sqlite && echo "FAIL: rusqlite still present" || echo "OK: rusqlite removed"

# Build web shell (requires trunk + wasm target)
cd crates/intrada-web && trunk build
```

## Manual Verification — CLI

```bash
# 1. Remove any existing data to start fresh
rm -f ~/.local/share/intrada/library.json

# 2. Add a piece
cargo run -p intrada-cli -- add piece "Clair de Lune" "Debussy" --key "Db Major"

# 3. Verify the JSON file was created
cat ~/.local/share/intrada/library.json | python3 -m json.tool

# 4. List items (should show the piece)
cargo run -p intrada-cli -- list

# 5. Add an exercise
cargo run -p intrada-cli -- add exercise "Hanon No. 1"

# 6. Verify both items in JSON
cat ~/.local/share/intrada/library.json | python3 -m json.tool

# 7. Delete the piece, verify it's removed from JSON
cargo run -p intrada-cli -- delete <piece-id>
cat ~/.local/share/intrada/library.json | python3 -m json.tool
```

## Manual Verification — Web

```bash
# 1. Start the web app
cd crates/intrada-web && trunk serve

# 2. Open http://localhost:8080 in browser

# 3. First load: stub data should appear and be persisted
#    Check: DevTools → Application → Local Storage → intrada:library

# 4. Add a new piece via the form

# 5. Refresh the page — the new piece should still be there
#    (plus the original stub data, minus any deleted items)

# 6. Clear localStorage and refresh — stub data should reappear
```

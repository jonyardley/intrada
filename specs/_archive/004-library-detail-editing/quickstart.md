# Quickstart: Library Add, Detail View & Editing

**Feature**: `004-library-detail-editing`
**Date**: 2026-02-14

## Prerequisites

- Rust stable toolchain (1.75+) with `wasm32-unknown-unknown` target
- trunk (`cargo install trunk`)
- tailwindcss v4 standalone CLI (see below)

## Setup

```bash
# Clone and checkout feature branch
git clone https://github.com/jonyardley/intrada.git
cd intrada
git checkout 004-library-detail-editing

# Install Tailwind CSS v4 standalone CLI (if not already installed)
# macOS:
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.18/tailwindcss-macos-arm64
chmod +x tailwindcss-macos-arm64
sudo mv tailwindcss-macos-arm64 /usr/local/bin/tailwindcss

# Linux:
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.18/tailwindcss-linux-x64
chmod +x tailwindcss-linux-x64
sudo mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss
```

## Development

```bash
# Run all tests (core + CLI)
cargo test --workspace

# Run clippy
cargo clippy -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Start the web dev server (auto-rebuilds on file changes)
cd crates/intrada-web
trunk serve --open
```

## Verification

### Automated checks
```bash
cargo test --workspace    # All tests pass (82+)
cargo clippy -- -D warnings  # No warnings
cargo fmt --all -- --check   # Formatting clean
cd crates/intrada-web && trunk build  # WASM build succeeds
```

### Manual verification (in browser at http://127.0.0.1:8080)

1. **Library list**: Page loads showing stub items (Clair de Lune, Hanon No. 1)
2. **Add piece**: Click "Add" > "Piece" > fill title + composer > Save > item appears in list
3. **Add exercise**: Click "Add" > "Exercise" > fill title > Save > item appears in list
4. **Validation**: Try submitting add form with empty title > inline error shown
5. **Detail view**: Click on any item > full details displayed > click Back > returns to list
6. **Edit**: From detail view, click Edit > change title > Save > title updated in detail and list
7. **Delete**: From detail view, click Delete > confirm > item removed from list
8. **Cancel**: From any form, click Cancel > returns to previous view without changes

## Architecture Notes

- **No persistence**: All data is in-memory. Page refresh resets to stub data.
- **Full-page views**: List, detail, add, and edit are separate full-page views — only one shown at a time.
- **Crux core unchanged**: All events (Add, Update, Delete) already exist. This feature only adds web UI.
- **Shell-side validation**: Forms validate before dispatching Crux events for inline error display.

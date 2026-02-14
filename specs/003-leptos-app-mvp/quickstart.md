# Quickstart: 003-leptos-app-mvp

**Date**: 2026-02-14
**Feature**: Leptos Web App MVP

## Prerequisites

- Rust stable toolchain (1.75+) with `wasm32-unknown-unknown` target
- trunk (`cargo install trunk` or `cargo binstall trunk`)
- Tailwind CSS v4 standalone CLI (downloaded from GitHub releases)

### Install Prerequisites

```bash
# Add WASM target (one-time)
rustup target add wasm32-unknown-unknown

# Install trunk (one-time)
cargo install trunk
# OR faster:
# cargo binstall trunk

# Install Tailwind CSS standalone CLI (one-time)
# macOS (Apple Silicon):
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64
chmod +x tailwindcss-macos-arm64
sudo mv tailwindcss-macos-arm64 /usr/local/bin/tailwindcss

# macOS (Intel):
# curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-x64

# Linux:
# curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
```

## Development Server

```bash
# From repository root:
cd crates/intrada-web
trunk serve
```

Open `http://127.0.0.1:8080` in your browser.

**Expected result**: A styled landing page with "Intrada" title and a mini library view showing 2 stub items (one piece, one exercise) with item count.

## Development Workflow

1. Edit `crates/intrada-web/src/main.rs` or `crates/intrada-web/input.css`
2. Save the file
3. Trunk auto-rebuilds and reloads the browser (within ~5 seconds)
4. See changes reflected immediately

## Verification Scenarios

### Scenario 1: Landing Page Renders (US1)

```
Steps:
1. Run `cd crates/intrada-web && trunk serve`
2. Open http://127.0.0.1:8080
3. Verify: Page shows "Intrada" title with styled content
4. Verify: Tailwind CSS classes are applied (visible styling)
```

### Scenario 2: Auto-Rebuild on Change (US2)

```
Steps:
1. With trunk serve running, open src/main.rs
2. Change a visible text string (e.g., page description)
3. Save the file
4. Verify: Browser reloads automatically within 5 seconds
5. Verify: Changed text appears in browser
```

### Scenario 3: Crux ViewModel Rendering (US3)

```
Steps:
1. Load the landing page
2. Verify: Mini library view displays "2 item(s)" (or similar count)
3. Verify: Item titles visible (e.g., "Clair de Lune", "Hanon No. 1")
4. Verify: Item types shown (piece, exercise)
```

### Scenario 4: Interactive Update (US3 - acceptance scenario 2)

```
Steps:
1. Load the landing page with stub data displayed
2. Click a button that triggers a Crux event (e.g., "Add Sample Item")
3. Verify: Item count increases and new item appears without page reload
4. Reload the page
5. Verify: View resets to original stub data (2 items)
```

### Scenario 5: Noscript Fallback (FR-010)

```
Steps:
1. Disable JavaScript in browser settings
2. Navigate to http://127.0.0.1:8080
3. Verify: A message displays indicating JavaScript is required
```

### Scenario 6: CLI Unaffected (SC-003)

```
Steps:
1. From repository root, run: cargo test
2. Verify: All 82 existing tests pass
3. Run: cargo clippy -- -D warnings
4. Verify: No new warnings
```

### Scenario 7: WASM Build (FR-011)

```
Steps:
1. From crates/intrada-web, run: trunk build
2. Verify: Build completes without errors
3. Verify: dist/ directory created with WASM + HTML + CSS files
```

### Scenario 8: Lighthouse Accessibility (SC-004)

```
Steps:
1. Run trunk serve and open page in Chrome
2. Open DevTools > Lighthouse > Accessibility audit
3. Verify: Score is 90 or above
```

## Build Commands

| Command | Directory | Purpose |
|---------|-----------|---------|
| `trunk serve` | `crates/intrada-web/` | Dev server with auto-reload |
| `trunk build` | `crates/intrada-web/` | Debug build |
| `trunk build --release` | `crates/intrada-web/` | Production build |
| `cargo test` | Repository root | Run all workspace tests |
| `cargo clippy` | Repository root | Lint all workspace crates |
| `cargo fmt --all -- --check` | Repository root | Check formatting |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `error: target wasm32-unknown-unknown not found` | Run `rustup target add wasm32-unknown-unknown` |
| `trunk: command not found` | Run `cargo install trunk` |
| Tailwind classes not applied | Ensure `tailwindcss` binary is on PATH; check `@source` directive in `input.css` |
| Slow rebuilds | Set `RUSTFLAGS="--cfg=erase_components"` before `trunk serve` |
| Port 8080 already in use | Configure port in `Trunk.toml` under `[serve]` section |
| Panics show unhelpful messages | Ensure `console_error_panic_hook::set_once()` is called in `main()` |

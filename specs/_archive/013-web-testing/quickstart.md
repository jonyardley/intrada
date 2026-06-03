# Quickstart: Web UI Testing & E2E Test Infrastructure

**Feature**: 013-web-testing | **Date**: 2026-02-15

## Prerequisites

- Rust stable (1.75+) with `wasm32-unknown-unknown` target
- `wasm-pack` CLI (`cargo install wasm-pack`)
- Node.js 20+ (for Playwright E2E tests)
- Chrome/Chromium (for headless WASM and E2E tests)

## Running Tests

### 1. Rust Unit Tests (helpers + validation)

```bash
# Run all web crate unit tests
cargo test -p intrada-web

# Run with output
cargo test -p intrada-web -- --nocapture
```

These tests verify pure functions (`parse_tags`, `parse_tempo`, `parse_tempo_display`, `validate_library_form`) and run in the standard Rust test harness — no browser required.

### 2. WASM Integration Tests (localStorage)

```bash
# Run WASM tests in headless Chrome
cd crates/intrada-web
wasm-pack test --headless --chrome -- --test wasm
```

These tests verify localStorage persistence logic (library data round-trips, session data round-trips, stub data seeding) inside a real browser environment.

**Troubleshooting**:
- If Chrome is not found: install Chrome or set `CHROME_BIN` environment variable
- If `wasm-pack` is not installed: `cargo install wasm-pack`

### 3. E2E Tests (Playwright)

```bash
# Build the app first
cd crates/intrada-web
trunk build

# Install E2E dependencies
cd ../../e2e
npm ci
npx playwright install --with-deps chromium

# Run E2E tests (serves dist/ automatically)
npx playwright test

# Run with UI for debugging
npx playwright test --ui
```

### All Tests Together

```bash
# Unit + WASM + E2E
cargo test -p intrada-web && \
  (cd crates/intrada-web && wasm-pack test --headless --chrome -- --test wasm) && \
  (cd crates/intrada-web && trunk build) && \
  (cd e2e && npm ci && npx playwright test)
```

## CI/CD

All three test layers run automatically on pull requests via GitHub Actions:

| Job | Command | Estimated Time |
|-----|---------|----------------|
| `test` | `cargo test` (includes web unit tests) | Existing (~1 min) |
| `wasm-test` | `wasm-pack test --headless --chrome` | ~1-2 min |
| `e2e` | `npx playwright test` (after artifact download) | ~1-2.5 min |

## File Structure

```
crates/intrada-web/
├── src/
│   ├── helpers.rs          # Pure functions (unit tested)
│   ├── validation.rs       # Form validation (unit tested)
│   └── core_bridge.rs      # localStorage bridge (WASM tested)
└── tests/
    └── wasm.rs             # WASM integration tests

e2e/
├── package.json            # Playwright dependency
├── playwright.config.ts    # Playwright configuration
└── tests/
    └── smoke.spec.ts       # Smoke test (page loads, content visible)
```

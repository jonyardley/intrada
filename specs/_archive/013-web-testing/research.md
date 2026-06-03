# Research: Web UI Testing & E2E Test Infrastructure

**Feature**: 013-web-testing | **Date**: 2026-02-15

## R1: Web Crate Testability Analysis

**Decision**: Focus unit tests on pure functions in `helpers.rs` and `validation.rs`; use `wasm-bindgen-test` for browser-dependent code in `core_bridge.rs`.

**Rationale**: The web crate contains two categories of code: pure functions that can be tested with standard `cargo test`, and browser-dependent code that requires WASM APIs (localStorage, `web_sys::window()`). Testing each with the appropriate tool maximises coverage with minimal friction.

**Findings**:

### Pure Functions (testable with `#[test]`)

| File | Function | Inputs/Outputs | Complexity |
|------|----------|----------------|------------|
| `helpers.rs` | `parse_tags(input: &str) -> Vec<String>` | Comma-separated string → vec | Low |
| `helpers.rs` | `parse_tempo(marking: &str, bpm_str: &str) -> Option<Tempo>` | Two strings → optional Tempo | Medium |
| `helpers.rs` | `parse_tempo_display(tempo: &Option<String>) -> (String, String)` | Formatted string → (marking, bpm) | Medium |
| `validation.rs` | `validate_library_form(ItemType, &FormData) -> HashMap<String, String>` | Form data → validation errors | High |

### Browser-Dependent Code (requires `wasm-bindgen-test`)

| File | Function | Browser Dependency |
|------|----------|--------------------|
| `core_bridge.rs` | `load_library_data()` | localStorage read, `web_sys::window()` |
| `core_bridge.rs` | `load_sessions_data()` | localStorage read |
| `core_bridge.rs` | `process_effects()` | localStorage write (via save functions) |
| `core_bridge.rs` | `get_local_storage()` | `web_sys::window()`, `.local_storage()` |
| `data.rs` | `create_stub_data()` | None (uses `Utc::now()` + `ulid::Ulid::new()`) |

**Note**: `data.rs::create_stub_data()` is mostly pure but uses `Utc::now()` and `ulid::Ulid::new()`, making assertions on exact values impractical. Tests should verify structure (field presence, non-empty) rather than exact values.

### Leptos Components (not directly testable)

Leptos 0.8.x CSR does not provide component-level test utilities. The 16 components and 6 views cannot be unit-tested in isolation without a full reactive runtime. This is an accepted limitation — component behaviour is verified through E2E tests instead.

**Alternatives considered**:
- Testing components via `leptos::ssr` rendering: Not viable for CSR-only apps
- Using `leptos_test` or similar: No stable crate exists for Leptos 0.8.x CSR component testing

---

## R2: wasm-bindgen-test Setup

**Decision**: Use `wasm-bindgen-test` 0.3.x with `wasm-pack test --headless --chrome` for browser-dependent integration tests.

**Rationale**: `wasm-bindgen-test` is the standard WASM testing framework in the Rust ecosystem, maintained alongside `wasm-bindgen`. It runs tests in a real browser environment, providing access to localStorage and DOM APIs.

**Setup requirements**:

1. Add to `Cargo.toml`:
   ```toml
   [dev-dependencies]
   wasm-bindgen-test = "0.3"
   ```

2. Create `tests/wasm.rs`:
   ```rust
   use wasm_bindgen_test::*;
   wasm_bindgen_test_configure!(run_in_browser);
   ```

3. Run with: `wasm-pack test --headless --chrome -- --test wasm`

**Test isolation**: Each test must clear localStorage before running. Use a helper:
```rust
fn clear_local_storage() {
    if let Some(storage) = web_sys::window().unwrap().local_storage().unwrap() {
        storage.clear().unwrap();
    }
}
```

**Thread-local state**: `core_bridge.rs` uses `thread_local!` for `LIBRARY` and `SESSIONS`. Tests sharing the same thread will share this state. Each test must reset these via the public `load_library_data()` / `load_sessions_data()` functions after clearing localStorage.

**CI integration**: Add a new job to `ci.yml`:
```yaml
wasm-test:
  name: WASM Tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown
    - uses: Swatinem/rust-cache@v2
      with:
        shared-key: "wasm"
    - uses: aspect-build/rules_js/.github/actions/setup-wasm-pack@main
      # Or: cargo install wasm-pack
    - run: wasm-pack test --headless --chrome -- --test wasm
      working-directory: crates/intrada-web
```

**Estimated CI time**: ~1-2 minutes (compile + run in headless Chrome).

**Alternatives considered**:
- `wasm-pack test --headless --firefox`: Works but Chrome is pre-installed on GitHub Actions runners
- Running in Node.js (`--node`): Does not provide localStorage or DOM APIs
- Custom test harness with `wasm-bindgen`: Unnecessary complexity over `wasm-bindgen-test`

---

## R3: E2E Testing Tool Evaluation

**Decision**: Recommend **Playwright** for E2E testing.

**Rationale**: Playwright is a mature, well-maintained browser automation framework that the Leptos project itself uses for E2E testing. It has first-class GitHub Actions support, fast headless execution, and excellent developer experience.

### Tool Comparison

| Criterion | Playwright | Cypress | Selenium |
|-----------|-----------|---------|----------|
| **Leptos/WASM compatibility** | Excellent — used by Leptos project | Good — works with any web app | Good — works with any web app |
| **GitHub Actions support** | Native — `playwright install --with-deps` | Requires custom setup | Requires WebDriver management |
| **Headless execution** | Built-in, fast | Built-in | Requires driver config |
| **Community maturity** | High — Microsoft-backed, active | High — large community | Very high — longest history |
| **Setup complexity** | Low — npm + install browsers | Medium — needs config | High — driver management |
| **Speed** | Fast (~1-2s per test) | Medium (~2-3s per test) | Slow (~3-5s per test) |
| **Language** | TypeScript/JavaScript | TypeScript/JavaScript | Multiple (Java, Python, JS) |
| **CI time estimate** | ~1-2.5 min total | ~2-3 min total | ~3-5 min total |

### Recommended Setup

**Directory structure**:
```
e2e/
├── package.json
├── playwright.config.ts
└── tests/
    └── smoke.spec.ts
```

**Workflow**: The existing `wasm-build` job already runs `trunk build`. Add artifact upload there, then a new `e2e` job downloads the artifact and serves it with `npx serve`:

```yaml
# In wasm-build job, add:
- uses: actions/upload-artifact@v4
  with:
    name: web-dist
    path: crates/intrada-web/dist/

# New e2e job:
e2e:
  name: E2E Tests
  runs-on: ubuntu-latest
  needs: wasm-build
  steps:
    - uses: actions/checkout@v4
    - uses: actions/download-artifact@v4
      with:
        name: web-dist
        path: dist/
    - uses: actions/setup-node@v4
      with:
        node-version: 20
    - working-directory: e2e
      run: npm ci
    - working-directory: e2e
      run: npx playwright install --with-deps chromium
    - working-directory: e2e
      run: npx playwright test
```

**Estimated CI time**: ~1-2.5 minutes (download artifact + install Playwright + run tests).

**Total CI time increase**: WASM tests (~1-2 min) + E2E tests (~1-2.5 min) = ~2-4.5 min total, within the 5-minute budget (FR-017).

**Alternatives considered**:
- **fantoccini** (Rust WebDriver client): Lower community adoption, requires separate WebDriver binary management, less tooling for assertions and debugging
- **headless_chrome** (Rust): Direct Chrome DevTools Protocol, very low-level, no test framework built-in
- **thirtyfour** (Rust Selenium): Requires separate WebDriver, more complex setup than Playwright
- **Cypress**: Strong but heavier setup, doesn't auto-wait as well for WASM-loaded content

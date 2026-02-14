# Research: 003-leptos-app-mvp

**Date**: 2026-02-14
**Feature**: Leptos Web App MVP
**Status**: Complete

## 1. Leptos Framework Version & CSR Setup

**Decision**: Use Leptos 0.8.x (latest stable) with `features = ["csr"]`

**Rationale**: Leptos 0.8.x is the current stable release series. The `csr` feature flag enables client-side rendering mode which compiles to WASM and runs entirely in the browser. This is the simplest deployment model — no server runtime needed.

**Key findings**:
- Entry point: `leptos::mount::mount_to_body(App)` (changed from 0.7.x `mount_to_body`)
- Signals: `RwSignal` is arena-allocated and `Copy`; `ArcRwSignal` is reference-counted for collections
- `.get()` clones value and subscribes effects; `.update()` mutates in-place (more efficient)
- `console_error_panic_hook` recommended for WASM debugging (better panic messages in browser console)
- `RUSTFLAGS="--cfg=erase_components"` improves compile times during development

**Alternatives considered**:
- Leptos 0.7.x: Older, Crux examples reference 0.7.8 but 0.8.x is the current stable
- Yew/Dioxus: Other Rust WASM frameworks, but spec requires Leptos specifically
- SSR mode: Deferred per spec; CSR-only is simpler for MVP

## 2. Trunk Build Tool

**Decision**: Use trunk 0.21.x (latest stable) as the WASM build/dev server

**Rationale**: Trunk is the recommended build tool for Leptos CSR apps. It handles WASM compilation, asset bundling, and dev server with auto-reload. No Node.js required.

**Key findings**:
- `trunk serve` compiles to WASM, starts dev server, watches for changes, does full rebuild + page reload via WebSocket
- `trunk build --release` for production builds
- Requires `index.html` in crate root with `<link data-trunk ...>` tags for asset processing
- Trunk.toml configuration supports `[build]`, `[watch]`, `[serve]` sections
- `trunk serve` does NOT do true HMR — it's full rebuild + auto-reload (still fast for small apps)
- `rust-toolchain.toml` can pin wasm32-unknown-unknown target for reproducible builds

**Alternatives considered**:
- cargo-leptos: Designed for SSR mode, overkill for CSR-only
- wasm-pack + manual server: More setup, less integrated DX
- Vite with wasm-pack: Requires Node.js, adds complexity

## 3. Tailwind CSS v4 Integration

**Decision**: Use Tailwind CSS v4 standalone CLI with trunk hooks

**Rationale**: Tailwind v4 is the current stable (released Jan 2025). It uses a CSS-first configuration approach — no JavaScript config file needed. The standalone CLI binary avoids any Node.js dependency, keeping the stack pure Rust + native tools.

**Key findings**:
- Tailwind v4 config is CSS-based: `@import 'tailwindcss'; @source "./src/**/*.rs";`
- The `@source` directive tells Tailwind to scan Rust source files for class names
- Standalone CLI available as platform-specific binary from GitHub releases
- Two integration approaches with trunk:
  - **Approach A**: Trunk built-in `<link data-trunk rel="tailwind-css" href="input.css" />` — uses trunk's native tailwind support
  - **Approach B**: Trunk.toml `[[hooks]]` — runs tailwind CLI as pre-build step
- Approach A is simpler but may lag behind Tailwind v4 features; Approach B gives full control
- **Decision**: Start with Approach A (trunk built-in); fall back to Approach B if v4 compatibility issues arise

**Alternatives considered**:
- Tailwind v3: Older, requires JS config file, more complex setup
- PostCSS pipeline: Requires Node.js
- Hand-written CSS: Doesn't meet spec requirement for utility-first CSS
- Styler/Leptos-style: Rust-native CSS-in-code, but spec explicitly requires Tailwind

## 4. Crux Web Shell Architecture

**Decision**: Leptos component wraps `Core<Intrada>`, uses `RwSignal<ViewModel>` for reactive state, effect loop handles Storage stubs

**Rationale**: This mirrors the proven CLI shell pattern (`Shell` struct in `intrada-cli/src/shell.rs`) adapted for the browser. Leptos signals provide the reactive bridge between Crux's ViewModel and the DOM.

**Key findings**:
- Existing Effect enum: `Render(Request<RenderOperation>)` and `Storage(Box<Request<StorageEffect>>)`
- CLI shell pattern: `core.process_event(event)` → iterate effects → match Storage/Render → `core.view()`
- Web shell equivalent:
  1. Create `Core<Intrada>` (can be in a Leptos `provide_context`)
  2. Hold `RwSignal<ViewModel>` for reactive UI updates
  3. On event: call `core.process_event(event)`, iterate returned effects
  4. For `Render(_)`: call `core.view()` and update the signal
  5. For `Storage(req)`: match on `req.operation`:
     - `LoadAll`: Return hardcoded stub data (≥2 items) — do NOT call `core.resolve()` since these are `notify_shell` effects with `RequestHandle::Never`
     - Save/Update/Delete: No-op (silently ignore)
- `intrada-core` has no WASM-incompatible deps (no rusqlite, no filesystem access)
- `crux_core` compiles to `wasm32-unknown-unknown` — designed for it

**Critical pattern**: In crux_core 0.17, `notify_shell` effects have `RequestHandle::Never`. The shell must NOT call `core.resolve()` on these. Instead, it performs the side effect (or stub) and the core continues. This differs from request/response capabilities where you resolve with a result.

**Stub data design**: At minimum 2 items — one Piece and one Exercise — with realistic field values to demonstrate the ViewModel rendering (title, composer/category, key, tags).

**Alternatives considered**:
- IndexedDB for browser persistence: Out of scope — spec says stub data only
- localStorage: Same — deferred to future feature
- Separate WASM bridge crate: Unnecessary complexity when Leptos can call Core directly

## 5. CI Pipeline: WASM Build Check

**Decision**: Add `wasm-build` job using `jetli/trunk-action` for fast trunk install, `dtolnay/rust-toolchain` with wasm32-unknown-unknown target

**Rationale**: The fastest CI approach downloads pre-built trunk binary (~2-3s) rather than compiling from source (~3-4min). The job validates that the web crate compiles to WASM without breaking.

**Key findings**:
- `jetli/trunk-action@v0.5.1` downloads pre-built trunk binary from GitHub releases (~2-3 seconds)
- Alternative: `cargo-binstall trunk` (~2-5 seconds) — also fast, more generic
- Avoid: `cargo install trunk` (~3-4 minutes from source)
- `dtolnay/rust-toolchain@stable` with `targets: wasm32-unknown-unknown` adds the WASM target
- `Swatinem/rust-cache@v2` with `shared-key: "wasm"` separates cache from native builds
- Tailwind standalone CLI needed in CI too — download from GitHub releases
- WASM clippy can catch WASM-incompatible code; scope to web crate with `-p intrada-web`
- Estimated warm-cache build time: 2-5 minutes total

**CI job structure**:
1. Checkout + rust-toolchain (with wasm32 target)
2. rust-cache (shared-key: "wasm")
3. Install trunk (jetli/trunk-action)
4. Install Tailwind CSS standalone binary
5. `trunk build` (validates WASM compilation)

**Alternatives considered**:
- cargo-binstall approach: Viable runner-up, slightly slower
- cargo install trunk: Too slow for CI
- Skip CI check: Risk of silent web crate breakage

## 6. Project Structure

**Decision**: New crate `crates/intrada-web/` alongside existing `intrada-core` and `intrada-cli`

**Rationale**: Follows the existing workspace pattern. The web crate depends on `intrada-core` (same as CLI) and contains Leptos components + trunk configuration.

**Key findings**:
- Workspace members in root `Cargo.toml` use `members = ["crates/*"]` — new crate auto-discovered
- Web crate needs: `leptos` (csr), `intrada-core` (path), `console_error_panic_hook`, `wasm-bindgen`
- File layout:
  ```
  crates/intrada-web/
  ├── Cargo.toml
  ├── Trunk.toml         # trunk config (port, watch settings)
  ├── index.html          # entry point with data-trunk links
  ├── input.css           # Tailwind CSS entry (@import 'tailwindcss')
  └── src/
      └── main.rs         # Leptos mount + App component + Crux shell
  ```
- Trunk expects `index.html` in the crate root directory, not in `src/`
- `trunk serve` must be run from the `crates/intrada-web/` directory

**Alternatives considered**:
- Top-level `web/` directory: Breaks existing `crates/*` convention
- Monolithic single crate: Violates workspace separation principle
- Separate repository: Loses workspace dep sharing and monorepo benefits

## 7. Developer Experience (DX)

**Decision**: `trunk serve` with auto-reload, `console_error_panic_hook`, clear README instructions

**Rationale**: Spec defines "Good DX" as fast feedback loops, clear error messages, and minimal setup steps.

**Key findings**:
- `trunk serve` watches `src/`, `index.html`, and CSS files — rebuilds on save
- Rebuild + reload cycle: ~2-5 seconds for small apps (within SC-002's 5-second budget)
- `console_error_panic_hook::set_once()` in main provides readable Rust panics in browser console
- `RUSTFLAGS="--cfg=erase_components"` speeds up Leptos compilation
- Trunk.toml can configure port, watch patterns, proxy settings
- `rust-toolchain.toml` in web crate dir can pin the wasm32 target for `rustup target add` auto-install

**Setup steps for developer** (target: under 3 minutes per SC-001):
1. Install trunk: `cargo install trunk` (one-time)
2. `cd crates/intrada-web && trunk serve`
3. Open browser to `http://127.0.0.1:8080`

**Alternatives considered**:
- cargo-leptos: SSR-focused, adds unnecessary complexity
- Custom watch script: Reinvents what trunk provides
- Docker dev environment: Adds setup friction

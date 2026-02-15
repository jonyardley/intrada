# Research: API Sync

**Feature**: 021-api-sync
**Date**: 2026-02-15
**Purpose**: Resolve technical unknowns before Phase 1 design

## R1: HTTP Approach — Shell-Side vs Core-Side

### Decision: Shell-side HTTP (Approach A)

### Rationale

The Crux architecture in intrada uses a pure core (`intrada-core`) that emits `StorageEffect` variants via `Command::notify_shell()`. The web shell (`intrada-web`) processes these effects in `process_effects()` and performs platform-specific I/O (currently localStorage reads/writes).

**Approach A (shell-side HTTP)** replaces the localStorage handlers in `process_effects()` with HTTP fetch calls to the REST API. The core's `StorageEffect` enum and `Event` enum remain unchanged — the core still emits `SavePiece(Piece)`, the shell just sends it to `POST /api/pieces` instead of writing to localStorage.

**Approach B (core-side HTTP capability)** would add a new `Http` effect variant to the `Effect` enum and use `crux_http` or a custom HTTP capability in the core. This would violate the Architecture Integrity constitution principle (pure core, no I/O) and require significant refactoring of the Effect/Event system.

### Alternatives Considered

| Approach | Pros | Cons |
|----------|------|------|
| A: Shell-side HTTP | No core changes, preserves purity, reuses existing patterns, minimal new code | Shell handler becomes async, slightly more complex shell |
| B: Core-side HTTP capability | Core explicitly models HTTP requests as effects | Breaks pure core principle, adds `crux_http` dependency, requires new Effect variants, significant refactoring |

### Conclusion

Approach A is strongly preferred. It aligns with the constitution (V. Architecture Integrity), requires no changes to `intrada-core`, and follows the existing pattern where the shell is responsible for all I/O.

---

## R2: Async Patterns in WASM Shell

### Decision: Use `wasm-bindgen-futures::spawn_local()` for async HTTP within `process_effects()`

### Rationale

The current `process_effects()` function is fully synchronous — it loops over effects, matches on `StorageEffect` variants, performs localStorage operations (which are synchronous Web APIs), and calls `view_model.set(core.view())` at the end.

HTTP fetch calls are inherently asynchronous in WASM. The function cannot simply `await` a fetch because it's not an async function, and making it async would require changes throughout the call chain.

The solution is to use `wasm-bindgen-futures::spawn_local()` to spawn async tasks for HTTP-backed storage effects. The pattern:

1. For effects that load data (`LoadAll`, `LoadSessions`): spawn an async task that fetches from the API, then feeds the result back to the core via `core.process_event(Event::DataLoaded { ... })` and recursively calls `process_effects()`.
2. For effects that write data (`SavePiece`, `UpdatePiece`, `DeleteItem`, etc.): spawn a fire-and-forget async task that sends the HTTP request. On failure, feed `Event::LoadFailed(error_message)` back to the core.
3. Session-in-progress effects (`SaveSessionInProgress`, `ClearSessionInProgress`): remain localStorage-based (per FR-008).

### Key Constraint

The `SharedCore` type is `SendWrapper<Rc<RefCell<Core<Intrada>>>>`. Since WASM is single-threaded, `spawn_local` tasks run on the same thread and can safely access `Rc<RefCell<...>>` — but the core and view_model references must be cloned (as `Rc`/`RwSignal`) into the async closure.

### Alternatives Considered

| Approach | Pros | Cons |
|----------|------|------|
| `spawn_local()` per effect | Simple, granular, works with existing sync loop | Need to clone references into closures |
| Make `process_effects()` fully async | Cleaner async flow | Requires async propagation through entire call chain, major refactor |
| Use `gloo-net` crate | Higher-level HTTP API | Additional dependency, less control |
| Use `web-sys` Fetch API directly | No extra dependencies | More boilerplate, manual `JsValue` wrangling |

### Conclusion

Use `spawn_local()` with `gloo-net` (or raw `web-sys` fetch). `gloo-net` provides a clean Rust API over the Fetch API and is a common choice for WASM HTTP. It adds minimal bundle size overhead.

---

## R3: HTTP Client Dependencies

### Decision: Use `gloo-net` for HTTP requests, `wasm-bindgen-futures` for async runtime

### Rationale

The `intrada-web` crate currently has no HTTP dependencies. The following additions are needed:

- **`wasm-bindgen-futures`** (already transitively available via wasm-bindgen ecosystem, but needs explicit dependency for `spawn_local()`): Provides `spawn_local()` to run async code from synchronous WASM contexts.
- **`gloo-net`**: Provides idiomatic Rust HTTP client for WASM. Wraps the browser Fetch API with proper error handling, JSON serialization, and `async`/`await` support.

Alternative: Using raw `web-sys` Fetch API requires manual `JsValue` conversions, `JsFuture` wrapping, and more boilerplate. `gloo-net` is the standard community choice and adds minimal bundle size (~15KB).

### New Dependencies

```toml
[dependencies]
gloo-net = { version = "0.6", features = ["http", "json"] }
wasm-bindgen-futures = "0.4"
serde = { workspace = true }  # Already available via intrada-core
```

### Alternatives Considered

| Option | Bundle Impact | Ergonomics | Maintenance |
|--------|---------------|------------|-------------|
| `gloo-net` | ~15KB | Excellent — typed requests/responses | Active, well-maintained |
| `reqwest` (wasm feature) | ~50KB+ | Good but heavy | Overkill for simple REST |
| Raw `web-sys` fetch | 0KB extra | Poor — manual JsValue handling | N/A |

---

## R4: API Base URL Configuration

### Decision: Compile-time environment variable via `option_env!()` with fallback

### Rationale

FR-011 requires the API base URL to be configurable. In a WASM app deployed to Cloudflare Workers, runtime environment variables are not available. The standard approach is:

1. Use `option_env!("INTRADA_API_URL")` at compile time (set in Trunk.toml or CI environment)
2. Fall back to a sensible default: `https://intrada-api.fly.dev` for production, or `http://localhost:8080` for local dev

Trunk supports environment variable injection via `trunk build` environment or `Trunk.toml` configuration.

### Configuration Approach

```rust
const API_BASE_URL: &str = match option_env!("INTRADA_API_URL") {
    Some(url) => url,
    None => "https://intrada-api.fly.dev",
};
```

For local development: `INTRADA_API_URL=http://localhost:8080 trunk serve`

---

## R5: Error Handling Strategy

### Decision: Map HTTP errors to `Event::LoadFailed(String)` for user-facing errors

### Rationale

The core already has `Event::LoadFailed(String)` which sets `model.last_error` and is rendered via `ViewModel.error`. This existing infrastructure is sufficient for displaying API errors.

Error categories to handle:
1. **Network errors** (server unreachable, timeout): "Unable to connect to the server. Please check your connection."
2. **Server errors** (500): "The server encountered an error. Please try again."
3. **Validation errors** (400): Display the server's error message directly (e.g., "Title must be between 1 and 500 characters")
4. **Not found** (404): "The requested item was not found."
5. **Deserialization errors**: "Received unexpected data from the server."

For write operations (save/update/delete), errors should also be fed back via `Event::LoadFailed` so the user sees feedback.

---

## R6: Loading State

### Decision: Add `is_loading` flag to ViewModel, set via new `Event` variants

### Rationale

FR-007 requires loading indicators during initial data load and form submissions. The current `ViewModel` has no loading state.

The approach is to add `is_loading: bool` to the `ViewModel` and `Model`, with new events:
- `Event::SetLoading(bool)` — or simply set the flag in the shell before dispatching async loads

However, since the core should stay pure, the simplest approach is a **shell-side loading signal** — a Leptos `RwSignal<bool>` provided via context, toggled by the shell's HTTP handlers. This avoids adding core events purely for UI state.

### Decision: Shell-side `RwSignal<bool>` for loading state

This is simpler and avoids polluting the core with UI-only concerns.

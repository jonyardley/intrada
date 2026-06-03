# Research: User Authentication

## Decision 1: Clerk JS SDK Integration in WASM/Trunk

**Decision**: Load `@clerk/clerk-js@5` via a `<script>` tag in `index.html` and bridge into Rust/WASM via `wasm_bindgen` inline JS bindings.

**Rationale**: Clerk's JS SDK is the only officially supported browser integration. The project uses Trunk for WASM builds with no JS bundler, so a CDN script tag is the simplest integration. A small JS helper on `window.__intrada_auth` exposes simplified async functions (`getToken`, `isSignedIn`, `signOut`) that Rust calls through `wasm_bindgen(inline_js)`.

**Alternatives Considered**:
- *npm-bundled Clerk SDK via Trunk plugin*: Rejected — project has no JS build toolchain for the web shell. Adding one increases complexity disproportionately.
- *Server-side redirect-only auth flow*: Rejected — loses Clerk's silent token refresh and client-side session management.
- *Custom OIDC implementation*: Rejected — no official Clerk Rust SDK exists, and hand-rolling OIDC is error-prone.

## Decision 2: JWT Validation Crate

**Decision**: Use `jsonwebtoken` v9 with RS256 algorithm. Fetch Clerk's JWKS from `{issuer}/.well-known/jwks.json` at server startup using `reqwest`, cache keys in `AppState` as `Arc<Vec<DecodingKey>>`.

**Rationale**: `jsonwebtoken` is the most widely used JWT crate in the Rust ecosystem, supports RS256 (Clerk's default), and has a stable API. Startup-time JWKS fetch is sufficient for a single-tenant app. Clerk rotates keys infrequently; a process restart picks up new keys.

**Alternatives Considered**:
- *`alcoholic_jwt`*: Has built-in JWKS support but is less maintained.
- *`biscuit`*: More complex API, focused on structured tokens.
- *Manual RSA with `ring`*: Too low-level; `jsonwebtoken` wraps this correctly.
- *Periodic JWKS rotation*: Deferred as enhancement. Not needed for initial launch.

## Decision 3: Auth-Optional Mode for Tests

**Decision**: When `CLERK_ISSUER_URL` env var is not set, the `AuthUser` extractor returns `AuthUser("")` (empty string matching the migration default). All existing API tests pass without changes.

**Rationale**: This is the zero-friction path. The auth middleware checks `AppState.auth_config: Option<AuthConfig>`. When `None`, requests pass through with a default empty user_id. Migration default `user_id = ''` matches, so existing test data is accessible. No test helper changes needed.

**Alternatives Considered**:
- *Feature flag (`--features auth`)*: Conditional compilation complicates CI and local dev.
- *Test-only JWT issuer with RSA key generation*: Adds test setup complexity with no benefit.
- *Mock middleware in tests*: Would require refactoring every test helper.

## Decision 4: Clerk SDK Version and Loading

**Decision**: Use `@clerk/clerk-js@5` from Clerk's CDN. The publishable key is compiled into the WASM build via `option_env!("CLERK_PUBLISHABLE_KEY")`.

**Rationale**: CDN loading avoids JS build toolchain. Major version pin (`@5`) balances stability with security patches. The publishable key is public by Clerk's design — safe to embed in client-side code.

**Alternatives Considered**:
- *Self-hosting Clerk JS bundle*: Adds maintenance burden for updates.
- *Pinning to specific minor version*: Would miss security patches.

## Decision 5: E2E Test Mocking Strategy

**Decision**: Stub `window.__intrada_auth` and `window.Clerk` via `page.addInitScript()` before page load. Block the Clerk CDN script via `page.route()` to prevent the real SDK from loading.

**Rationale**: The E2E mock fixture already intercepts all API requests at the browser network level, so auth headers are irrelevant to the mock. The auth gate in the web shell checks `window.__intrada_auth.isSignedIn()` — by stubbing this to return `true`, existing tests pass through the gate unchanged. Blocking the CDN script prevents network errors and speeds up tests.

**Alternatives Considered**:
- *Loading real Clerk SDK with test credentials*: Slower, flaky (depends on external service), and unnecessary for UI testing.
- *Skipping auth gate entirely in test builds*: Would require conditional compilation in the web shell, violating the "test what you ship" principle.

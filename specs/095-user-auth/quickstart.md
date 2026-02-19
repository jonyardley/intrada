# Quickstart: User Authentication Verification

## 1. Run API Tests (auth-optional mode)

```bash
cargo test -p intrada-api
```

All existing tests must pass without setting `CLERK_ISSUER_URL`. The auth middleware is skipped, and `AuthUser("")` is used for all queries.

## 2. Run All Unit Tests

```bash
cargo test
```

Verify `intrada-core` tests are unaffected (core crate has no auth changes).

## 3. Code Quality Checks

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

Both must pass cleanly.

## 4. Local API Server Without Auth

```bash
TURSO_DATABASE_URL=file:local.db TURSO_AUTH_TOKEN=unused cargo run --bin intrada-api
```

Verify:
```bash
curl http://localhost:3001/api/health    # → 200 OK
curl http://localhost:3001/api/items     # → 200 OK (auth skipped)
```

## 5. Local API Server With Auth

Prerequisites: Create a Clerk application at https://dashboard.clerk.com with Google OAuth enabled.

```bash
TURSO_DATABASE_URL=file:local.db \
TURSO_AUTH_TOKEN=unused \
CLERK_ISSUER_URL=https://<clerk-id>.clerk.accounts.dev \
cargo run --bin intrada-api
```

Verify:
```bash
curl -i http://localhost:3001/api/health  # → 200 OK (public)
curl -i http://localhost:3001/api/items   # → 401 Unauthorized
```

## 6. Web Shell With Auth

```bash
cd crates/intrada-web
CLERK_PUBLISHABLE_KEY=pk_test_... INTRADA_API_URL=http://localhost:3001 trunk serve
```

1. Open http://localhost:8080
2. Should see sign-in screen with "Sign in with Google" button
3. Click "Sign in with Google", complete OAuth flow
4. Should redirect to library view
5. Create an item — should appear in library
6. Click sign out — should return to sign-in screen
7. Refresh — should stay on sign-in screen

## 7. E2E Tests

```bash
cd e2e
npx playwright test
```

All 30+ existing tests must pass. The Clerk mock (injected via `page.addInitScript()`) simulates an authenticated state, bypassing the auth gate.

## 8. Deployment Verification

After deploying API to Fly.io and web shell to Cloudflare:

```bash
curl https://intrada-api.fly.dev/api/health   # → 200 OK
curl https://intrada-api.fly.dev/api/items    # → 401 Unauthorized
```

Open production URL:
1. See sign-in screen
2. Sign in with Google
3. See empty library (new user)
4. Create items, routines, sessions — all persist
5. Sign out and back in — data is still there
6. Open in a different browser/incognito — data is isolated

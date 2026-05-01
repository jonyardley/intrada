# Intrada — Setup & Configuration

This document covers the external accounts, secrets, and configuration needed to run and deploy Intrada.

## Architecture Overview

```
┌─────────────────────┐     HTTPS      ┌──────────────────┐     libsql     ┌──────────┐
│  Cloudflare Workers │ ─────────────→ │  Fly.io (Axum)   │ ────────────→ │  Turso   │
│  (static WASM app)  │   REST API     │  intrada-api     │               │  (SQLite) │
└─────────────────────┘                └──────────────────┘               └──────────┘
                                              │
                                              │  S3 API
                                              ▼
                                       ┌──────────────┐
                                       │ Cloudflare R2 │
                                       │ (photo store) │
                                       └──────────────┘
```

- **Frontend**: Leptos CSR + WASM, deployed as static files to Cloudflare Workers
- **API**: Axum 0.8 REST server, deployed to Fly.io via Docker
- **Database**: Turso (managed libsql/SQLite), accessed via HTTP
- **Object storage**: Cloudflare R2 for lesson photos, accessed via S3-compatible API

## 1. Cloudflare Workers (Frontend)

### Account setup

1. Create a [Cloudflare account](https://dash.cloudflare.com/sign-up)
2. Note your **Account ID** from the dashboard sidebar

### API token

1. Go to [API Tokens](https://dash.cloudflare.com/profile/api-tokens)
2. Create a token with the **"Edit Cloudflare Workers"** template
3. Save the token — you'll need it for GitHub Actions

### Configuration files

| File | Purpose |
|------|---------|
| `wrangler.toml` | Workers project config — app name, asset directory, SPA routing |
| `worker.js` | Minimal passthrough worker that serves static assets |

The first deploy creates the Workers project automatically. A `*.workers.dev` subdomain is assigned.

### GitHub Actions secrets

| Secret | Value | Where to find |
|--------|-------|---------------|
| `CLOUDFLARE_API_TOKEN` | API token from step above | Cloudflare dashboard → Profile → API Tokens |
| `CLOUDFLARE_ACCOUNT_ID` | Your account ID | Cloudflare dashboard sidebar |

Set these at: **GitHub repo → Settings → Secrets and variables → Actions**

## 2. Turso (Database)

### Account setup

1. Install the CLI: `brew install tursodatabase/tap/turso` (or see [docs](https://docs.turso.tech/cli/installation))
2. Sign up: `turso auth signup`

### Create the database

```bash
turso db create intrada
```

### Get credentials

```bash
# Database URL
turso db show intrada --url
# Output: libsql://intrada-<your-org>.turso.io

# Auth token
turso db tokens create intrada
# Output: eyJhbGci...
```

Save both values — you'll need them for Fly.io secrets and local development.

### Database schema

Migrations run automatically on server startup. Key tables:

- `items` — unified library items (pieces and exercises, with kind, title, composer, category, key, tempo, tags)
- `sessions` + `setlist_entries` — completed practice sessions with their entries
- `routines` + `routine_entries` — reusable practice routines

## 3. Fly.io (API Server)

### Account setup

1. Install the CLI: `brew install flyctl` (or see [docs](https://fly.io/docs/flyctl/install/))
2. Sign up: `fly auth signup`

### First deploy

```bash
# Launch creates the app on Fly.io (only needed once)
fly launch --no-deploy

# Set secrets
fly secrets set \
  TURSO_DATABASE_URL="libsql://intrada-<your-org>.turso.io" \
  TURSO_AUTH_TOKEN="<your-token>" \
  ALLOWED_ORIGIN="https://intrada.<your-account>.workers.dev"

# Deploy
fly deploy
```

### Configuration files

| File | Purpose |
|------|---------|
| `fly.toml` | App name, region, VM size, health check, auto-scaling |
| `Dockerfile` | Multi-stage build (cargo-chef → debian:bookworm-slim) |
| `.dockerignore` | Excludes unnecessary files from Docker build context |

### Machine config (cost-optimised)

| Setting | Value | Why |
|---------|-------|-----|
| VM size | `shared-cpu-1x` | Smallest available — sufficient for a JSON API |
| Memory | 256 MB (+256 MB swap) | Axum binary is lightweight |
| Auto-stop | `suspend` | Resumes in ~1s, zero cost while idle |
| Min machines | 0 | Scales to zero when no traffic |
| Region | `lhr` (London) | Single region, close to Turso |
| Health check | `GET /api/health` every 60s | Monitors server + database connectivity |

### Environment variables

| Variable | Set via | Value |
|----------|---------|-------|
| `TURSO_DATABASE_URL` | `fly secrets set` | `libsql://intrada-<org>.turso.io` |
| `TURSO_AUTH_TOKEN` | `fly secrets set` | Token from `turso db tokens create` |
| `ALLOWED_ORIGIN` | `fly secrets set` | Frontend URL (e.g. `https://intrada.xxx.workers.dev`) |
| `RUST_LOG` | `fly.toml` [env] | `info` (already configured) |
| `PORT` | `fly.toml` [env] | `8080` (production; defaults to `3001` locally) |

### Verify deployment

```bash
# Health check
curl https://intrada-api.fly.dev/api/health

# Expected: {"status":"ok","database":"ok"}
```

## 4. Cloudflare R2 (Photo Storage)

Lesson photos are stored in Cloudflare R2 (S3-compatible object storage). Without
R2 configured, the API starts normally but photo upload/delete endpoints return 500.

### Create the bucket

1. Go to [Cloudflare Dashboard → R2](https://dash.cloudflare.com/?to=/:account/r2)
2. Click **Create bucket**
3. Name it (e.g. `intrada-photos`), select a location hint close to your Fly.io region

### Enable public access

Photos are served directly from R2 to the iOS app. You need a public URL:

1. Open the bucket → **Settings** → **Public access**
2. Either:
   - **Custom domain** (recommended): add a subdomain like `photos.intrada.com` and configure the DNS record Cloudflare provides
   - **R2.dev subdomain**: enable the `*.r2.dev` URL (quick but not production-grade)
3. Note the public URL — this becomes `R2_PUBLIC_URL`

### Create an API token

1. Go to [R2 → Manage R2 API Tokens](https://dash.cloudflare.com/?to=/:account/r2/api-tokens)
2. Click **Create API token**
3. Permissions: **Object Read & Write** on the bucket you created
4. Save the **Access Key ID** and **Secret Access Key**

### Set Fly.io secrets

```bash
fly secrets set \
  R2_ACCOUNT_ID="<your-cloudflare-account-id>" \
  R2_ACCESS_KEY_ID="<access-key-from-step-above>" \
  R2_SECRET_ACCESS_KEY="<secret-key-from-step-above>" \
  R2_BUCKET_NAME="intrada-photos" \
  R2_PUBLIC_URL="https://photos.intrada.com"
```

Your Cloudflare Account ID is in the dashboard sidebar (same one used for Workers).

### Verify

```bash
# Check API logs after deploy
fly logs | grep R2

# Expected: "R2 photo storage configured"
# If missing: "R2 not configured — photo upload disabled (R2_ACCOUNT_ID must be set)"
```

### Local development

R2 is optional locally. Without the env vars, the API starts with photo upload
disabled. To test photos locally, add the R2 variables to your `.env` file.

## 5. Sentry (Error reporting + APM)

One Sentry project per surface. DSNs are public (Sentry's security model expects
them embedded in client code), but the Rust SDK reads them via env vars so
projects can be swapped without code changes.

### Projects

| Surface | Sentry platform | DSN delivery |
|---------|----------------|--------------|
| `intrada-api` | Rust | `SENTRY_DSN` env var (Fly secret) |
| `intrada-mobile` | Rust | `SENTRY_DSN_MOBILE` env var, baked at compile time via `option_env!` |
| `intrada-web` | Browser JavaScript | inlined in `crates/intrada-web/index.html` |

Each project tags events with `environment = development | production | ios`
(determined at runtime), and `release = $GIT_SHA` when set at build time.
Performance tracing is sampled at 10% (`traces_sample_rate = 0.1`).

### Set the API DSN as a Fly.io secret

```bash
fly secrets set SENTRY_DSN="https://...@...ingest.de.sentry.io/..." -a intrada-api
```

The API will pick it up on next deploy. Without it, Sentry init is a no-op.

### Set the mobile DSN locally

Add to your `.env` file at the repo root:

```
SENTRY_DSN_MOBILE=https://...@...ingest.de.sentry.io/...
```

`just ios-dev` will pick it up via `set dotenv-load`. Changing the value
triggers a rebuild thanks to `cargo:rerun-if-env-changed` in `build.rs`.

### Web DSN

The web DSN is inlined in `crates/intrada-web/index.html`. To swap projects,
edit that file directly. There is no `.env` value for it.

### Verifying

After the next deploy, trigger a test event from each surface (e.g. visit a
404 route, force a panic in a debug build) and confirm it appears in the
matching Sentry project's Issues tab.

## 6. CI/CD Pipeline (GitHub Actions)

The single workflow `.github/workflows/ci.yml` handles both CI and deployment:

```
push to PR:   test → clippy → fmt → wasm-build → wasm-test → e2e
push to main: all checks → deploy frontend (Cloudflare) + deploy API (Fly.io)
```

Both deploy jobs run in parallel after their required checks pass:
- **Frontend** (`deploy`): gates on all checks including E2E tests
- **API** (`deploy-api`): gates on test, clippy, fmt only (no WASM dependency — deploys faster)

The WASM app is built once (release mode) and reused for E2E tests and deployment.

### All GitHub Actions secrets

| Secret | Service | Required for |
|--------|---------|-------------|
| `CLOUDFLARE_API_TOKEN` | Cloudflare | Frontend deployment |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare | Frontend deployment |
| `FLY_API_TOKEN` | Fly.io | API deployment |

Set at: **GitHub repo → Settings → Secrets and variables → Actions**

### Generating the Fly.io deploy token

```bash
fly tokens create deploy -a intrada-api
```

Add the output as the `FLY_API_TOKEN` secret in GitHub Actions.

## 7. Local Development

### Prerequisites

- Rust stable (1.75+)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [just](https://github.com/casey/just) (`brew install just` or `cargo install just`)

### Quick start (recommended)

```bash
# 1. Set up environment
cp .env.example .env
# Edit .env with your Turso credentials

# 2. Start both API and web dev servers
just dev
# → API on :3001, web on :8080 (proxies /api/* to API)

# 3. Open http://localhost:8080
```

The `just dev` command starts both servers concurrently. The Trunk dev server on port 8080 proxies all `/api/*` requests to the API server on port 3001, so the browser only talks to one origin (no CORS issues).

### Manual setup (without just)

```bash
# Terminal 1: API server (requires Turso credentials)
export TURSO_DATABASE_URL="libsql://intrada-<your-org>.turso.io"
export TURSO_AUTH_TOKEN="<your-token>"
export ALLOWED_ORIGIN="http://localhost:8080"
export PORT=3001

cargo run -p intrada-api
# → API on http://localhost:3001

# Terminal 2: Web dev server
trunk serve --config crates/intrada-web/Trunk.toml
# → Web on http://localhost:8080 (proxies /api/* to :3001)
```

### Quick verification

```bash
# Health check (via API directly)
curl http://localhost:3001/api/health

# Health check (via Trunk proxy)
curl http://localhost:8080/api/health

# Create an item
curl -X POST http://localhost:3001/api/items \
  -H "Content-Type: application/json" \
  -d '{"kind":"piece","title":"Clair de Lune","composer":"Debussy","tags":[]}'

# List items
curl http://localhost:3001/api/items
```

### Seed data

To populate the API with realistic sample data:

```bash
just seed
# or: bash scripts/seed-dev-data.sh
```

See [README.md](README.md) for more seed options.

## Checklist

Use this when setting up from scratch:

- [ ] Cloudflare account created
- [ ] Cloudflare API token generated (Edit Workers permissions)
- [ ] `CLOUDFLARE_API_TOKEN` set in GitHub Actions secrets
- [ ] `CLOUDFLARE_ACCOUNT_ID` set in GitHub Actions secrets
- [ ] Turso CLI installed
- [ ] Turso database created (`turso db create intrada`)
- [ ] Database URL and auth token saved
- [ ] Fly.io CLI installed
- [ ] Fly.io app launched (`fly launch --no-deploy`)
- [ ] Fly.io secrets set (TURSO_DATABASE_URL, TURSO_AUTH_TOKEN, ALLOWED_ORIGIN)
- [ ] First deploy successful (`fly deploy`)
- [ ] Health check returns `{"status":"ok","database":"ok"}`
- [ ] R2 bucket created in Cloudflare
- [ ] R2 public access enabled (custom domain or r2.dev subdomain)
- [ ] R2 API token created (Object Read & Write)
- [ ] R2 secrets set on Fly.io (R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME, R2_PUBLIC_URL)
- [ ] `fly logs | grep R2` shows "R2 photo storage configured"
- [ ] Frontend deployed to Cloudflare Workers
- [ ] Frontend CORS requests to API work
- [ ] Sentry projects created (api, web, mobile)
- [ ] `SENTRY_DSN` set on Fly.io
- [ ] Web DSN inlined in `crates/intrada-web/index.html`
- [ ] Test event sent from each surface, visible in matching Sentry project

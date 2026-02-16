# Intrada — Setup & Configuration

This document covers the external accounts, secrets, and configuration needed to run and deploy Intrada.

## Architecture Overview

```
┌─────────────────────┐     HTTPS      ┌──────────────────┐     libsql     ┌──────────┐
│  Cloudflare Workers │ ─────────────→ │  Fly.io (Axum)   │ ────────────→ │  Turso   │
│  (static WASM app)  │   REST API     │  intrada-api     │               │  (SQLite) │
└─────────────────────┘                └──────────────────┘               └──────────┘
```

- **Frontend**: Leptos CSR + WASM, deployed as static files to Cloudflare Workers
- **API**: Axum 0.8 REST server, deployed to Fly.io via Docker
- **Database**: Turso (managed libsql/SQLite), accessed via HTTP

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

Migrations run automatically on server startup. Three tables are created:

- `pieces` — music pieces (title, composer, key, tempo, tags)
- `exercises` — practice exercises (title, composer, category, key, tempo, tags)
- `sessions` + `setlist_entries` — completed practice sessions with their entries

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

### Verify deployment

```bash
# Health check
curl https://intrada-api.fly.dev/api/health

# Expected: {"status":"ok","database":"ok"}
```

## 4. CI/CD Pipeline (GitHub Actions)

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

## 5. Local Development

### Frontend only (no API)

```bash
cd crates/intrada-web
trunk serve
# Open http://localhost:8080
```

Uses localStorage — no API or database needed.

### API server (requires Turso)

```bash
export TURSO_DATABASE_URL="libsql://intrada-<your-org>.turso.io"
export TURSO_AUTH_TOKEN="<your-token>"
export ALLOWED_ORIGIN="http://localhost:8080"

cargo run -p intrada-api
# Server starts on http://localhost:8080
```

### Quick verification

```bash
# Health check
curl http://localhost:8080/api/health

# Create a piece
curl -X POST http://localhost:8080/api/pieces \
  -H "Content-Type: application/json" \
  -d '{"title":"Clair de Lune","composer":"Debussy","tags":[]}'

# List pieces
curl http://localhost:8080/api/pieces
```

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
- [ ] Frontend deployed to Cloudflare Workers
- [ ] Frontend CORS requests to API work

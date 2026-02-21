# intrada Development Guidelines

> **Maintenance reminder**: Review this file for accuracy every 2 weeks or after any
> significant feature lands. Last reviewed: 2026-02-20.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics.

The product is organised around three activity pillars:
- **Plan** — decide what to practise (library, routines, goals, scheduling)
- **Practice** — play with intention (focus mode, timers, scoring, in-session UX)
- **Track** — see the process working (analytics, visualisation, insights)

See [`docs/roadmap.md`](docs/roadmap.md) for the current roadmap and
[`VISION.md`](VISION.md) for the research foundation.

## Project Structure

```text
crates/
  intrada-core/   # Pure Crux core — business logic, no I/O, no side effects
  intrada-web/    # Web shell — Leptos 0.8 CSR + WASM, Clerk auth UI
  intrada-api/    # REST API — Axum 0.8 + Turso (libsql), JWT validation
docs/             # Product roadmap (single source of truth for what's next)
e2e/              # Playwright E2E tests
specs/            # SpecKit design artifacts
```

## Tech Stack

- **Language**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, intrada-api requires 1.78+ for axum 0.8)
- **Core**: crux_core 0.17.0-rc2, serde 1, ulid 1, chrono 0.4, thiserror 1
- **API**: axum 0.8, tokio 1, libsql 0.9 (remote), tower-http 0.6 (CORS), reqwest 0.12, jsonwebtoken 10 (rust_crypto feature), tracing 0.1
- **Web**: leptos 0.8.x (CSR), leptos_router 0.8.x, gloo-net 0.6, web-sys 0.3, wasm-bindgen 0.2, send_wrapper 0.6, Tailwind CSS v4 (standalone CLI), trunk 0.21.x
- **Auth**: Clerk (managed auth, Google OAuth only), @clerk/clerk-js v5 (loaded via CDN)
- **Database**: Turso (managed libsql/SQLite) via HTTP protocol
- **Testing**: cargo test (unit/integration), wasm-bindgen-test 0.3, Playwright (E2E)
- **CI/CD**: GitHub Actions, deploy to Cloudflare Workers (web) + Fly.io (API)

## Authentication

- **Provider**: Clerk with Google OAuth sign-in
- **API auth**: JWT (RS256) validated against Clerk JWKS (`/.well-known/jwks.json`)
- **Key refresh**: Background tokio task refreshes JWKS keys every 60 minutes
- **User isolation**: All DB queries scope by `user_id` from JWT `sub` claim
- **Auth disabled mode**: When `CLERK_ISSUER_URL` is unset, `AuthUser("")` is returned — all data shares an empty user_id. This is for local dev/test only.
- **Frontend 401 retry**: API client retries once with a fresh Clerk token on 401

Key files: `intrada-api/src/auth.rs`, `intrada-web/src/clerk_bindings.rs`, `intrada-web/src/api_client.rs`

## Environment Variables

### API server (intrada-api)
- `TURSO_DATABASE_URL` — required, Turso database URL
- `TURSO_AUTH_TOKEN` — required, Turso auth token
- `CLERK_ISSUER_URL` — required in production (e.g. `https://clerk.myintrada.com`), omit to disable auth
- `ALLOWED_ORIGIN` — CORS origin (default: `http://localhost:8080`)
- `PORT` — server port (default: `3001`)
- `RUST_LOG` — tracing filter (default: `info`)

### Web build (intrada-web, compile-time)
- `CLERK_PUBLISHABLE_KEY` — Clerk publishable key, baked into WASM at build time
- `INTRADA_API_URL` — API base URL (default: `https://intrada-api.fly.dev`)

## Storage

- **All persistent data** (items, sessions, routines) is stored via the REST API in Turso
- **localStorage** is used ONLY for `intrada:session-in-progress` crash recovery
- No other localStorage keys are used — the old `intrada:library` and `intrada:sessions` keys were removed when the API was introduced

## Commands

```bash
cargo fmt --check          # must pass before commit — CI enforces this
cargo test                 # run all workspace tests
cargo clippy               # lint check
cargo test -p intrada-api  # API tests only (includes auth tests)
```

## Architecture Patterns

- **Crux core/shell split**: `intrada-core` contains zero I/O. All side effects are represented as enum variants and executed by the web shell. The core must compile on any Rust target without WASM dependencies.
- **Effect enum**: `AppEffect` carries all side-effect requests (HTTP API calls, localStorage). The shell processes each variant in `core_bridge.rs`.
- **API client**: `api_client.rs` has generic helpers (`get_json`, `post_json`, `put_json`, `delete`) with built-in 401 retry
- **Validation**: `intrada-core/src/validation.rs` is the single source of truth for all validation constants and rules
- **Database**: Positional column indexing (`row.get(0)`, etc.) with a `SELECT_COLUMNS` const to keep column order in one place
- **Migrations**: Sequential numbered migrations in `intrada-api/src/migrations.rs`, each must be a single SQL statement
- **Refresh-after-mutate**: Every write operation (create/update/delete) is followed by a full re-fetch from the API via `spawn_mutate()`. This keeps the Crux model as the single source of truth without client-side merge logic.

### State boundary

State is split between two systems. This is intentional — Crux owns *what the user has*,
Leptos owns *what the user is doing right now*.

| State kind | Where it lives | Examples |
|------------|---------------|----------|
| Domain data | Crux `Model` → `ViewModel` | Items, sessions, routines, active session progress, analytics |
| UI interaction | Leptos signals | Form field values, loading/submitting flags, timer ticks, drag state, tab selection |
| Crash recovery | localStorage | `intrada:session-in-progress` (single key, FR-008) |

**Rules:**
- Domain state must flow through `Event` → `Model` → `ViewModel`. Never store domain data in Leptos signals.
- UI state that has no meaning outside the current view stays in Leptos signals. Don't inflate the Crux model with ephemeral UI concerns.
- The `ViewModel` is the read-only projection that views consume. Views never mutate it directly.

## Code Style

- Rust stable, 2021 edition
- Follow standard Rust conventions
- `cargo fmt` and `cargo clippy -- -D warnings` must pass
- No `unwrap()` without justification

## Design System (Components-First)

All visual styling flows from the design token system defined in `intrada-web/input.css`.
Before writing any new UI code, check whether an existing token, utility, or component
already covers the pattern. If not, **create the abstraction first**, then use it.

### Colour tokens — never use raw Tailwind grays or named colours

| Token class     | Use for                              | Replaces           |
|-----------------|--------------------------------------|---------------------|
| `text-primary`  | Headings, titles, emphasis           | `text-white`        |
| `text-secondary`| Body text, descriptions              | `text-gray-300`     |
| `text-label`    | Form labels                          | `text-gray-200`     |
| `text-muted`    | Hints, captions, metadata            | `text-gray-400`     |
| `text-faint`    | Timestamps, very subtle text         | `text-gray-500`     |
| `text-accent-text` | Active nav, links, accent text    | `text-indigo-300`   |
| `text-danger-text`  | Error messages, delete actions   | `text-red-400`      |
| `text-success-text` | Positive status indicators       | `text-green-400`    |
| `text-warning-text` | Warning labels                   | `text-amber-400`    |
| `bg-surface-secondary` | Card backgrounds, skeletons   | `bg-white/5`        |
| `bg-surface-hover`  | Hover states                      | `bg-white/10`       |
| `border-border-default` | Separators, list borders       | `border-white/10`   |

### Typography utilities — prefer these over ad-hoc class strings

| Utility class    | Pattern                                         | Use for                          |
|------------------|-------------------------------------------------|----------------------------------|
| `card-title`     | `text-sm font-semibold text-secondary mb-3`     | Card subsection headings         |
| `section-title`  | `text-lg font-semibold text-primary mb-4`       | Section headings inside cards    |
| `field-label`    | `text-xs font-medium text-muted uppercase`      | Data labels, stat card titles    |
| `form-label`     | `block text-sm font-medium text-label mb-1`     | Form field labels                |
| `hint-text`      | `text-xs text-muted mb-1`                       | Helper text below fields         |
| `empty-text`     | `text-sm text-muted text-center py-4`           | No-data empty states             |
| `cta-link`       | Accent button as `<A>` link                     | "New Session", "Add Item" CTAs   |
| `action-link`    | `text-xs font-medium` + transition              | Edit/Delete text links           |

### Leptos components — reuse before creating new markup

| Component       | Purpose                                   |
|-----------------|-------------------------------------------|
| `PageHeading`   | Top-level page title (serif heading font) |
| `Card`          | Glassmorphism container                   |
| `StatCard`      | Metric display (title + value + subtitle) |
| `FieldLabel`    | `<dt>` label for definition lists         |
| `TextField`     | Form input with label + hint + error      |
| `TextArea`      | Form textarea with label + hint + error   |
| `Button`        | All clickable actions (Primary/Secondary/Danger/etc.) |
| `BackLink`      | Back-navigation link with arrow           |
| `TypeBadge`     | Piece/Exercise type pill                  |
| `Toast`         | Notification banner                       |
| `ErrorBanner`   | Persistent error display                  |

### Rules for new UI work

1. **Tokens first**: Every rendered colour MUST trace to a named token in `input.css`.
   Never use raw Tailwind colour classes (`text-gray-400`, `bg-red-500`, `text-indigo-300`).
2. **Utilities second**: If a styling pattern appears in 2+ places, create a `@utility`
   in `input.css` and document it in the table above.
3. **Components third**: If a pattern includes markup + logic, create a Leptos component
   in `components/` and re-export it from `components/mod.rs`.
4. **Design catalogue**: After creating any new component or utility, add a showcase
   entry to `views/design_catalogue.rs` so the full system is visible in one place.
5. **Spacing**: Use the spacing tokens (`p-card`, `p-card-compact`, `p-card-comfortable`,
   `space-y-6` for sections, `space-y-3`/`space-y-4` within cards, `gap-3` for grids).
6. **Font**: Use `font-heading` on page-level headings only. All other text uses the
   default system sans-serif.

Key files: `intrada-web/input.css` (tokens + utilities), `intrada-web/src/components/` (Leptos components), `views/design_catalogue.rs` (visual reference)

## Roadmap Alignment

Every piece of work should connect back to the product vision. Before starting
and after finishing, check alignment with the source-of-truth documents.

### Before starting work

1. **Identify the roadmap item.** Find the issue number in [`docs/roadmap.md`](docs/roadmap.md).
   If the work doesn't map to an existing item, pause and discuss whether it should.
2. **Check the pillar.** Know which pillar (Plan / Practice / Track) and horizon
   (Now / Next / Later) the work belongs to. Prefer `horizon:now` items over others.
3. **Check the project board.** The issue should be in Ready or In Progress on the
   [GitHub project board](https://github.com/users/jonyardley/projects/2). If it's
   still in Backlog, move it to Ready before starting.

### After completing work

1. **Update the roadmap.** If a feature is done, move it to "What's Built Today" in
   `docs/roadmap.md` and close the GitHub issue.
2. **Update CLAUDE.md.** If the work changes architecture, adds components, or
   introduces new patterns, update the relevant section of this file.
3. **Check for stale items.** Glance at the project board — are there issues that
   are now done, duplicated, or no longer relevant? Flag them.

### Periodic review (every 2 weeks)

These documents should stay in sync. When any one changes, check the others:

| Document | What to check |
|----------|---------------|
| [`docs/roadmap.md`](docs/roadmap.md) | Horizons still accurate? Closed issues removed? New work captured? |
| [`VISION.md`](VISION.md) | Still reflects current product direction? No stale phase references? |
| [`CLAUDE.md`](CLAUDE.md) | Tech stack, components, and patterns up to date? |
| [`docs/user-journey-to-be.drawio`](docs/user-journey-to-be.drawio) | Feature colours match pillar model? New features added? |
| [GitHub project board](https://github.com/users/jonyardley/projects/2) | Board status matches horizon labels? Priorities set? |

## Known Tech Debt

- Sessions and routines SQL is inline in route handlers (items has a dedicated `db/items.rs` module)
- Legacy `pieces` and `exercises` tables from early migrations still exist in the schema
- `dependabot.yml` needs `package-ecosystem` set to `"cargo"`

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

# intrada Development Guidelines

> Last reviewed: 2026-04-08.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics. Organised around three pillars:
**Plan** (library, routines), **Practice** (focus mode, timers, scoring),
**Track** (analytics, insights).

**Platform priority**: iOS is the primary channel. Web stays functional but
doesn't get active investment until iOS is in good shape. The Crux architecture
means core improvements benefit both shells, so prioritise iOS shell work and
defer web-only enhancements.

## Project Structure

```text
crates/
  intrada-core/   # Pure Crux core — business logic, no I/O
  intrada-web/    # Web shell — Leptos 0.8 CSR + WASM
  intrada-api/    # REST API — Axum 0.8 + Turso (libsql)
  shared/         # UniFFI bindings + CoreFfi/CoreJson bridges
  shared_types/   # Facet typegen — auto-generates Swift types
design/           # Pencil design system (intrada.pen)
docs/             # Product roadmap (single source of truth)
e2e/              # Playwright E2E tests
ios/Intrada/      # SwiftUI shell using CoreFfi (BCS bridge)
specs/            # Spec docs for major features (Tier 3 only — see Workflow)
```

## Tech Stack

- **Rust** stable (1.89.0 CI; MSRV 1.75+, intrada-api 1.78+)
- **Core**: crux_core 0.17.0-rc3, serde, ulid, chrono, thiserror
- **API**: axum 0.8, tokio, libsql 0.9 (Turso), tower-http (CORS), jsonwebtoken 10
- **Web**: leptos 0.8 (CSR), Tailwind CSS v4, trunk 0.21
- **iOS**: Swift 6.0, iOS 17.0+, SwiftUI, UniFFI (BCS bridge)
- **Auth**: Clerk (Google OAuth), JWT RS256 against JWKS
- **DB**: Turso (managed libsql/SQLite) via HTTP
- **E2E**: Playwright
- **CI/CD**: GitHub Actions → Cloudflare Workers (web) + Fly.io (API)

## Commands

```bash
cargo fmt --check          # must pass before commit
cargo test                 # all workspace tests
cargo clippy               # lint check
cargo test -p intrada-api  # API tests only
just typegen               # regenerate Swift types after core type changes
just ios-swift-check       # quick Swift compile check (~30s) — ALWAYS after .swift edits
just ios-smoke-test        # build + launch on sim (~15s) — after env/nav changes
just ios-preview-check     # validate #Preview blocks
```

Use `just ios-swift-check --clean` after switching branches or when CI/local diverge.

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User → Events → crux_core (Rust) → Effects (Http, KeyValue, Render) → Shell → I/O
```

1. **Core owns all logic.** HTTP requests built in core via `crux_http`. Core does
   all JSON serialization. Shells never understand domain types.
2. **Shells are dumb pipes.** Receive `HttpRequest` (URL, method, headers, bytes),
   return `HttpResponse`. No domain type imports.
3. **All types auto-generated.** `facet` typegen → Swift types from Rust. Zero
   hand-maintained type definitions in shells. Define in Rust + `derive(Facet)`.
4. **No hardcoded type names in scripts.** Everything flows from Rust via typegen.

### State boundary

| State kind | Where it lives |
|------------|---------------|
| Domain data | Crux `Model` → `ViewModel` (single source of truth) |
| UI interaction | Leptos signals (web) / SwiftUI `@State` (iOS) |
| Crash recovery | localStorage / UserDefaults (`intrada:session-in-progress` only) |

Domain state flows through `Event` → `Model` → `ViewModel`. Never store domain
data in shell-local state. UI-only state stays in shell signals/`@State`.

### Type generation

Pipeline: `facet` derive macros → `shared_types/build.rs` → Swift package with BCS.
Run `just typegen` after changing any `Facet`-derived type. Generated types are
NOT committed — they're rebuilt from Rust sources in CI and local builds.

**NEVER use `serde_repr`** on types in ViewModel or FFI traffic — causes byte-width
mismatch (i8 vs u32 variant indices), corrupting the BCS byte stream.

### Other patterns

- **Validation**: `intrada-core/src/validation.rs` is the single source of truth
- **DB**: Positional column indexing with `SELECT_COLUMNS` const
- **Migrations**: Sequential in `intrada-api/src/migrations.rs`, one SQL statement each
- **Mutate response**: Updates/deletes use API response directly (no re-fetch).
  Session creates use optimistic push (no re-fetch). Item creates re-fetch the
  full list (server assigns ID).

## Authentication

- Clerk + Google OAuth. JWT RS256 validated against JWKS.
- All DB queries scoped by `user_id` from JWT `sub` claim.
- When `CLERK_ISSUER_URL` unset: auth disabled (local dev only).
- Frontend retries once with fresh token on 401.
- Key files: `intrada-api/src/auth.rs`, `intrada-web/src/clerk_bindings.rs`

## Environment Variables

### API (intrada-api)
`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN` (required), `CLERK_ISSUER_URL` (required
in prod), `ALLOWED_ORIGIN` (default `http://localhost:8080`), `PORT` (default 3001)

### R2 photo storage (optional — API starts without it, photo endpoints return 500)
`R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET_NAME`,
`R2_PUBLIC_URL`. See `SETUP.md` §4 for provisioning steps.

### Web (compile-time)
`CLERK_PUBLISHABLE_KEY`, `INTRADA_API_URL` (default `https://intrada-api.fly.dev`)

## Design System Rules (Both Platforms)

Both platforms share the same dark-on-dark glassmorphism aesthetic. Cross-platform
visual parity is required — users should not be able to tell which platform they're on.

### Hierarchy: Tokens → Utilities/Modifiers → Components → Views

1. **Tokens first**: Every colour traces to a named token. Never use raw colours
   (`text-gray-400`, `.white`, `.indigo`). Web: `input.css`. iOS: `DesignSystem/Tokens/`.
2. **Reuse before creating**: Check existing components before building new markup.
   Web: `intrada-web/src/components/`. iOS: `ios/Intrada/Components/`.
3. **Design catalogue**: New components get a showcase entry.
   Web: `views/design_catalogue.rs`. iOS: `#Preview` blocks.
4. **Spacing tokens only**: `p-card`/`Spacing.card` (16), `p-card-compact`/
   `Spacing.cardCompact` (12), `p-card-comfortable`/`Spacing.cardComfortable` (24).

### iOS-specific rules

- Use `.navigationTitle()` on every screen (large on root, inline on pushed).
- `NavigationSplitView` on iPad for all list→detail screens.
- All content sections in `CardView`. All actions via `ButtonView(variant:)`.
- All empty states via `EmptyStateView`. No `ContentUnavailableView`.
- Destructive actions require `.confirmationDialog` with `titleVisibility: .visible`.
- No custom back buttons — use system NavigationStack back.
- `@Indirect` wrapper on generated types: access properties directly. In closures,
  add explicit type annotations for inference.
- Always read `Generated/SharedTypes/SharedTypes.swift` before writing code against
  generated types. Don't assume field types.

### Web-specific rules

- Never use raw Tailwind colour classes. Use token classes (`text-primary`, etc.).
- Typography utilities: `card-title`, `section-title`, `field-label`, `form-label`.
- New components go in `components/` and re-export from `components/mod.rs`.

## Code Style

- Rust stable, 2021 edition. `cargo fmt` + `cargo clippy -- -D warnings` must pass.
- No `unwrap()` without justification.
- Prefer well-established libraries over custom implementations.

## Workflow

Match ceremony to scope. Default to less. Escalate only when the work demands
it. The goal is a good outcome first time, not maximum process.

### Tier 1 — Just do it
Bug fixes, copy/text changes, style tweaks, renames, lint/clippy fixes,
single-file refactors, dependency bumps, doc updates.

No Plan mode, no spec doc. Read enough to confirm the change, make it,
verify, ship.

### Tier 2 — Plan mode only (default for most feature work)
New component/view following existing patterns, new API endpoint following
established conventions, adding a field to a model, new screen in existing
navigation, anything touching 2-4 focused files.

Use Plan mode to align on approach before coding. For UI work, design in
`design/intrada.pen` after the plan, before implementing. No spec doc.

### Tier 3 — Lightweight spec (rare; reserve for architectural work)
Net-new top-level features, Crux core / FFI bridge changes, auth or DB
schema changes, multi-week work spanning core + web + iOS, anything that
can't be described in three sentences.

Write ONE markdown doc in `specs/<feature>.md` — ~100-200 lines covering:
problem, proposed approach, key decisions, open questions. Get alignment,
then Pencil design (UI work), then Plan mode, then implement.

Do not run the SpecKit `/speckit-*` commands. They produce multi-file
spec/plan/tasks artifacts calibrated for multi-engineer teams; on a solo
project they add 1-3 hours of markdown overhead per feature without
qualitative benefit. Historical SpecKit folders under `specs/` are kept
as reference but are no longer the workflow.

### Decision rule
If unsure between tiers, go one tier lighter. Drift up if scope expands
during the work — don't pre-emptively over-spec.

### Always (regardless of tier)
1. Find the roadmap item in `docs/roadmap.md`. No item = discuss first.
2. Check priority on the [project board](https://github.com/users/jonyardley/projects/2).
3. UI work designs in `design/intrada.pen` before implementation.
4. Pre-push: `cargo fmt && cargo clippy` must pass.
5. After Swift/generated changes: `just ios-swift-check`.
6. Never push to main. Always a feature branch + PR.

### After completing work
1. Update `docs/roadmap.md`, close the GitHub issue.
2. Update this file if architecture/patterns changed.
3. Update Pencil if UI diverged from design.

## Pencil Design Workflow

All design in `design/intrada.pen` (single file). Required for new views and
significant UI changes. Mobile (375px) frames are primary; Desktop (1440px) frames
are optional until web gets active investment. Reuse design system components.
Colours must reference Pencil variables, not raw hex.

## Known Tech Debt

- Creates still re-fetch the full collection (server assigns ID)

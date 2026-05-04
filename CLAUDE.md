# intrada Development Guidelines

> Last reviewed: 2026-04-25.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics. Organised around three pillars:
**Plan** (library, routines), **Practice** (focus mode, timers, scoring),
**Track** (analytics, insights).

**Platform**: The Leptos shell (`crates/intrada-web`) is the single UI codebase
— it ships as both the web app (Cloudflare Workers) and the iOS app (Tauri 2
WKWebView host in `crates/intrada-mobile`). New features ship on both platforms
simultaneously.

## Project Structure

```text
crates/
  intrada-core/          # Pure Crux core — business logic, no I/O
  intrada-web/           # Leptos 0.8 CSR + WASM — UI shell (web + iOS)
  intrada-api/           # REST API — Axum 0.8 + Turso (libsql)
  intrada-mobile/        # Tauri 2 iOS host — wraps intrada-web in WKWebView
    src-tauri/           #   Rust host, tauri.conf.json, Swift plugins
design/                  # Pencil design system (intrada.pen)
docs/                    # Product roadmap (single source of truth)
e2e/                     # Playwright E2E tests
specs/                   # Spec docs for major features (Tier 3 only — see Workflow)
```

## Tech Stack

- **Rust** stable (1.89.0 CI; MSRV 1.75+, intrada-api 1.78+)
- **Core**: crux_core 0.17.0-rc3, serde, ulid, chrono, thiserror
- **API**: axum 0.8, tokio, libsql 0.9 (Turso), tower-http (CORS), jsonwebtoken 10
- **Web + iOS UI**: leptos 0.8 (CSR), Tailwind CSS v4, trunk 0.21
- **iOS host**: Tauri 2, iOS 17.0+, WKWebView, tauri-plugin-haptics, tauri-plugin-deep-link
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
just ios-dev               # start Tauri iOS dev session (sim) — runs trunk serve + tauri ios dev
just ios-build             # build Tauri iOS app for device (no TestFlight)
```

First-time iOS setup (run once after cloning or pulling this branch):
```bash
cargo install tauri-cli --version "^2" --locked   # Tauri CLI
brew install cocoapods                             # CocoaPods (required by Tauri iOS)
# Also requires: iOS Simulator runtime (Xcode → Settings → Platforms → iOS Simulator)
cd crates/intrada-mobile/src-tauri
cargo tauri ios init       # generates the Xcode project under src-tauri/gen/apple/
```

If you're forking this repo, update `bundle.iOS.developmentTeam` in
`crates/intrada-mobile/src-tauri/tauri.conf.json` to your own Apple Team ID
(find it at developer.apple.com → Membership, or Xcode → Settings → Accounts).

`just ios-dev` reads `INTRADA_API_URL` and `CLERK_PUBLISHABLE_KEY` from your
shell or a `.env` file at the repo root (the justfile uses `set dotenv-load`).
Without them set, the build will use defaults and Clerk auth won't work.

**Development security warning**: `just ios-dev` binds the Trunk dev server to
`0.0.0.0:8080` so the iOS simulator can reach it via the host's LAN IP.
Anyone on your Wi-Fi network can reach it (and the proxied `/api/`) while it's
running. Don't run `ios-dev` on public/untrusted Wi-Fi.

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User → Events → crux_core (Rust) → Effects (Http, KeyValue, Render) → Shell → I/O
```

1. **Core owns all logic.** HTTP requests built in core via `crux_http`. Core does
   all JSON serialization. Shells never understand domain types.
2. **Shells are dumb pipes.** Receive `HttpRequest` (URL, method, headers, bytes),
   return `HttpResponse`. No domain type imports.
3. **One UI codebase.** The Leptos shell talks to the core via `wasm-bindgen` —
   no typegen step. Domain types live in Rust and the WASM consumer uses them
   directly.

### State boundary

| State kind | Where it lives |
|------------|---------------|
| Domain data | Crux `Model` → `ViewModel` (single source of truth) |
| UI interaction | Leptos signals (web + iOS via Tauri) |
| Crash recovery | localStorage (`intrada:session-in-progress` only) |

Domain state flows through `Event` → `Model` → `ViewModel`. Never store domain
data in shell-local state. UI-only state stays in Leptos signals.

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

## Design System Rules

The Leptos shell uses a dark-on-dark glassmorphism aesthetic. Web and iOS run
the same UI codebase; iOS-specific look-and-feel is layered on with platform
gating (see iOS native-feel rules below).

### Hierarchy: Tokens → Utilities → Components → Views

1. **Tokens first**: Every colour traces to a named token. Never use raw colours
   (`text-gray-400`). Source: `crates/intrada-web/style/input.css`.
2. **Reuse before creating**: Check existing components before building new
   markup. Source: `crates/intrada-web/src/components/`.
3. **Design catalogue**: New components get a showcase entry in
   `views/design_catalogue.rs`.
4. **Spacing tokens only**: `p-card` (16), `p-card-compact` (12),
   `p-card-comfortable` (24).

### iOS native-feel rules (Leptos shell in Tauri WKWebView)

These rules apply when building or modifying views/components that will run
inside the Tauri iOS shell. Gate iOS-only CSS with `[data-platform="ios"]`
(set on `<html>` from `lib.rs` `setup` hook on iOS) — never with raw media
queries alone.

- **CSS reset**: `-webkit-touch-callout: none`, `-webkit-tap-highlight-color: transparent`,
  `-webkit-user-select: none` on chrome (not text content), `touch-action: manipulation`
  on interactive elements, `overscroll-behavior: none` on root.
- **Inputs**: `font-size: 16px` minimum (prevents iOS zoom-on-focus).
- **Safe areas**: `env(safe-area-inset-*)` on tab bar, headers, sticky bars.
  `viewport-fit=cover` already set in `index.html`.
- **Scroll**: only inner regions scroll; `overscroll-behavior: contain` on scroll containers.
- **View Transitions**: wrap route changes in `document.startViewTransition()` via wasm-bindgen.
- **Haptics**: use `tauri-plugin-haptics` — `selection` for tabs, `light` for taps,
  `success` for saves, `warning` for destructive confirms.
- **iPad**: all list→detail screens use `<SplitView>` (CSS-grid sidebar + detail pane,
  viewport-driven visibility). Build it before the view, not as a retrofit.
- **Typography**: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", system-ui`.
- **Animations**: Motion One spring config `stiffness: 300, damping: 30` ≈ iOS default.

### Web-specific rules

- Never use raw Tailwind colour classes. Use token classes (`text-primary`, etc.).
- Typography utilities: `card-title`, `section-title`, `field-label`, `form-label`.
- New components go in `components/` and re-export from `components/mod.rs`.

## Code Style

- Rust stable, 2021 edition. `cargo fmt` + `cargo clippy -- -D warnings` must pass.
- No `unwrap()` without justification.
- Prefer well-established libraries over custom implementations.

## Workflow

Match ceremony to scope. Default to less. Escalate only when work demands it.

### Tier 1 — Just do it
Bug fixes, copy/text changes, style tweaks, renames, lint/clippy fixes,
single-file refactors, dependency bumps, doc updates.

No Plan mode, no spec doc. Read enough to confirm the change, make it,
verify, ship.

### Tier 2 — Plan mode (default for feature work)
New component/view following existing patterns, new API endpoint following
established conventions, adding a field to an existing model, new screen
in existing navigation.

For UI work: Pencil design first (see Pencil Design Workflow below), then
Plan mode, then implement. For non-UI work: Plan mode, then implement.
No spec doc.

### Tier 3 — Lightweight spec (rare; architectural only)
Net-new top-level features, Crux core / FFI bridge changes, auth or DB
schema changes, multi-week work spanning core + web + iOS.

Write ONE markdown doc in `specs/<feature>.md` (~100-200 lines: problem,
approach, key decisions, open questions). Then Pencil for UI work, then
Plan mode, then implement.

Do not run `/speckit-*` slash commands. Historical SpecKit folders under
`specs/` are reference only.

### Domain sensitivity override
Changes to auth, Tauri plugin IPC contracts, DB schema, or migrations go
up at least one tier regardless of file count or apparent size.

### Decision rule
If unsure between tiers, go one tier lighter. Drift up if scope expands.

### Examples

| Task | Tier | Why |
|------|------|-----|
| Fix typo in a label | 1 | Trivial copy change |
| Bump a dependency with no API change | 1 | Dep bump |
| New "Recently practiced" view following existing list patterns | 2 | New view, established patterns |
| Refactor `intrada-core/src/practice/session.rs` (no FFI change) | 2 | Single file, non-trivial domain logic |
| Tweak retry backoff in `auth.rs` | 2 | Sensitivity override from Tier 1 |
| Add `notes` field to a piece (touches FFI + DB) | 3 | Override: FFI + schema |
| New auth provider | 3 | Auth + multi-crate |
| Migrate persistence layer | 3 | Architectural |

### Optional skills (Superpowers, opt-in only)

The [Superpowers](https://github.com/obra/superpowers) plugin provides ~14 auto-triggering skills + a methodology. We don't adopt it whole — its "always TDD, multi-stage subagent review, formal plans for everything" philosophy conflicts with the tier system above. But three of its skills are useful **when invoked deliberately**:

- **`test-driven-development`** — opt in for **non-UI Tier 2 / all Tier 3** work. Skip for visual / gesture / styling work where verification is on-device. The "watch the test fail before writing the code" discipline is what's valuable; the "delete code written before tests" rule is too strict for our pace.
- **`requesting-code-review`** — opt in **before opening any Tier 3 PR**, and for Tier 2 PRs touching auth / DB / FFI. Acts as a pre-flight checklist: does the diff match the spec, are tests passing, are there obvious quality issues. Cheaper than discovering them post-merge.
- **`using-git-worktrees`** — opt in when **two or more PR branches are in flight at once** (e.g., the recent #329 / #330 / #331 sequence). Prevents the rebase-conflict tangles that happen when squash-merges land while you're still working on the next branch.

**Do NOT enable** the rest of the methodology by default — `brainstorming`, `writing-plans`, `subagent-driven-development`, `executing-plans`, `finishing-a-development-branch`, `systematic-debugging`. Those collapse the tier system into one-size-fits-all heavyweight ceremony, which we explicitly don't want.

If you're unsure whether a skill applies, default to the tier system. The skills are sharper tools for specific situations, not replacements for "match ceremony to scope".

**Install (Claude Code):** `/plugin install superpowers@claude-plugins-official`

**Invoke selectively:** Tell the agent which skill to apply (e.g., "use test-driven-development for this"). Superpowers' default behaviour is to auto-trigger skills based on context — when invoking a single skill deliberately, also tell it to skip the others (e.g., "use just test-driven-development, no plan or subagent review needed").

**Re-evaluate** after the next 3 PRs that use any of these skills: did they catch a real issue, or did they add ceremony for its own sake? Expand scope, drop a skill, or trial another from the Superpowers set based on what we observe.

### Always
1. Find the roadmap item in `docs/roadmap.md`. No item = discuss first.
2. Check priority on the [project board](https://github.com/users/jonyardley/projects/2).
3. Never push to main. Always a feature branch + PR.

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

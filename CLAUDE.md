# intrada Development Guidelines

> **Maintenance reminder**: Review this file for accuracy every 2 weeks or after any
> significant feature lands. Last reviewed: 2026-04-03.

## Project Overview

intrada is a music practice companion app. Users sign in with Google (via Clerk),
manage a library of pieces and exercises, run timed practice sessions with scoring,
build reusable routines, and view analytics.

The product is organised around three activity pillars:
- **Plan** — decide what to practise (library, routines, scheduling)
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
  shared/         # Rust FFI crate — UniFFI bindings + CoreFfi/CoreJson bridges
  shared_types/   # Facet typegen — auto-generates Swift types with BCS serialization
design/           # Pencil design system file (intrada.pen)
docs/             # Product roadmap (single source of truth for what's next)
e2e/              # Playwright E2E tests
ios/              # iOS app — SwiftUI shell using CoreFfi (BCS bridge)
  Intrada/
    Core/           # Crux bridge (IntradaCore.swift — effect processor)
    DesignSystem/   # Tokens (Color, Font, Spacing) + ViewModifiers
    Components/     # Reusable SwiftUI components (one file per component)
    Views/          # Feature screens composed from components
    Navigation/     # Tab bar and navigation structure
    Config/         # App configuration (Clerk keys, API URL)
    Storage/        # UserDefaults wrappers (session crash recovery)
    Generated/      # Auto-generated types (SharedTypes, Serde, UniFFI)
specs/            # SpecKit design artifacts
```

## Tech Stack

- **Language**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, intrada-api requires 1.78+ for axum 0.8)
- **Core**: crux_core 0.17.0-rc3, serde 1, ulid 1, chrono 0.4, thiserror 1
- **API**: axum 0.8, tokio 1, libsql 0.9 (remote), tower-http 0.6 (CORS), reqwest 0.12, jsonwebtoken 10 (rust_crypto feature), tracing 0.1
- **Web**: leptos 0.8.x (CSR), leptos_router 0.8.x, gloo-net 0.6, web-sys 0.3, wasm-bindgen 0.2, send_wrapper 0.6, Tailwind CSS v4 (standalone CLI), trunk 0.21.x
- **Auth**: Clerk (managed auth, Google OAuth only), @clerk/clerk-js v5 (loaded via CDN)
- **Database**: Turso (managed libsql/SQLite) via HTTP protocol
- **iOS**: Swift 6.0, iOS 17.0+, SwiftUI, ClerkKit, UniFFI (BCS bridge)
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
just typegen               # regenerate Swift types after changing intrada-core types
just typegen-check         # verify generated types are up to date (CI use)
just ios                   # cross-compile for iOS + generate types + UniFFI bindings
just ios-swift-check       # quick Swift-only build validation (~30s, no Rust build)
just ios-smoke-test        # build + launch on sim, verify no crash (~15s after build)
just ios-preview-check     # validate SwiftUI previews compile
```

### iOS validation (IMPORTANT — always do this after Swift changes)

iOS has three levels of validation. **Always run level 1.** Run level 2 when
changing the app entry point, environment injection, or navigation structure.

| Level | Command | Time | Catches |
|-------|---------|------|---------|
| 1. Compile | `just ios-swift-check` | ~30s | Argument ordering, missing imports, type errors |
| 2. Smoke test | `just ios-smoke-test` | ~15s | Runtime crashes: missing `@Environment`, bad modifier ordering, crash-on-launch |
| 3. Preview | `just ios-preview-check` | ~30s | Broken `#Preview` providers |

```bash
just ios-swift-check       # ALWAYS after editing any .swift file
just ios-smoke-test        # after changing IntradaApp, environment, navigation
just ios-preview-check     # after changing #Preview blocks
```

**⚠️ Incremental build cache**: `just ios-swift-check` uses Xcode's incremental
build. The `just typegen`, `just ios`, `just ios-sim`, and `just ios-types` recipes
automatically invalidate the Xcode intermediate cache, so `ios-swift-check` is safe
to run immediately after them. Use `just ios-swift-check --clean` when:
- After switching branches
- When CI fails but local build passes
- After any manual changes to generated files

**When to run level 2 (smoke test):**
- After changing `IntradaApp.swift` or `ContentRouter`
- After adding/moving `.environment()` or `.toastOverlay()` modifiers
- After changing navigation structure (`MainTabView`, tab routing)
- After adding new `@Environment` reads in views
- Requires a prior `just ios-sim` build (Rust cross-compilation for simulator)

**Common runtime crash patterns to watch for:**
- `.toastOverlay()` or other modifiers that read `@Environment` must be applied
  **before** the `.environment()` that injects the value (SwiftUI reads environment
  from parent, not from sibling modifiers in the same chain)
- Every `@Environment(X.self)` must have a matching `.environment(x)` ancestor

## Architecture Patterns

### Crux capabilities pattern (IMPORTANT — follow this for all new work)

The project follows the standard Crux architecture where the **core requests
side-effects via capabilities** and **shells are dumb I/O executors**.

```text
User → Events → crux_core (Rust) → Effects (Http, KeyValue, Render) → Shell → I/O
                                 ← Responses ←                        ← Shell ←
```

Key principles:
1. **Core owns all logic.** HTTP requests are built inside the core using
   `crux_http`. The core serializes request bodies (JSON via serde) and
   deserializes response bodies. Shells never need to understand domain types
   for API communication.
2. **Shells are dumb pipes.** The iOS shell receives `HttpRequest` (URL, method,
   headers, body bytes) and returns `HttpResponse` (status, body bytes). The web
   shell does the same via `fetch`. Neither shell imports domain types for JSON.
3. **All types auto-generated.** `facet` typegen generates all Swift/Kotlin/TS
   types from Rust. There are **zero hand-maintained type definitions or Codable
   conformances** in shell projects. If you need a new type, define it in Rust
   with `derive(Facet)` and regenerate.
4. **No hardcoded type names in scripts.** Build scripts must not contain
   hardcoded enum Codable implementations or type-specific Swift code. Everything
   flows from Rust type definitions via the typegen pipeline.

### Other patterns

- **Crux core/shell split**: `intrada-core` contains zero I/O. All side effects are represented as capability operations (Http, KeyValue, Render) and executed by shells. The core must compile on any Rust target without WASM dependencies.
- **Effect enum**: Uses `crux_http::protocol::HttpRequest` for API calls and `KeyValue` for local storage. Shells process generic HTTP/storage operations.
- **Validation**: `intrada-core/src/validation.rs` is the single source of truth for all validation constants and rules
- **Database**: Positional column indexing (`row.get(0)`, etc.) with a `SELECT_COLUMNS` const to keep column order in one place
- **Migrations**: Sequential numbered migrations in `intrada-api/src/migrations.rs`, each must be a single SQL statement
- **Mutate response**: Updates and deletes use the API response (or optimistic confirmation) directly — no re-fetch. Creates still re-fetch the full list because the server assigns the canonical ID. The Crux model remains the single source of truth.

### Type generation (typegen)

All Swift types are **auto-generated** from the Rust domain types — there are zero
hand-written type definitions in the iOS project.

**Pipeline**: `facet` derive macros → `shared_types/build.rs` → `crux_core::type_generation::facet::TypeRegistry` → Swift package with BCS serialization.

**When to regenerate**: Run `just typegen` whenever you change structs/enums in
`intrada-core` that derive `facet::Facet` (domain types, events, effects, view model).
CI runs `just typegen-check` to enforce freshness.

**Key files**:

| File | Purpose |
|------|---------|
| `crates/shared_types/build.rs` | TypeRegistry that generates Swift from Rust types |
| `crates/shared_types/generated/swift/` | Auto-generated Swift package (committed to repo) |
| `ios/Intrada/Generated/SharedTypes/` | Copied by `build-ios.sh` (stripped of `import Serde`) |
| `ios/Intrada/Generated/Serde/` | BCS serialization runtime |
| `ios/Intrada/Generated/UniFFI/` | UniFFI bindings (`CoreFfi` class) |

**Serialization**: Bincode (via `crux_core::bridge::BincodeFfiFormat`) is used for all
Crux FFI traffic — Events, Effects, ViewModel, Request/Resolve. Auto-generated by
typegen. JSON serialization for the REST API is handled **entirely in Rust** via
`serde_json` inside `crux_http` — shells never perform JSON encoding/decoding of
domain types.

**⚠️ NEVER use `serde_repr` on types that appear in the ViewModel or FFI traffic.**
`facet` typegen generates standard variant-indexed serialization (u32 indices). Using
`serde_repr` causes a byte-width mismatch: Rust writes raw discriminant bytes (e.g. i8)
while Swift expects u32 variant indices, corrupting the entire byte stream. Use standard
`#[derive(Serialize, Deserialize)]` for all types crossing the FFI boundary.

### State boundary

State is split between two systems. This is intentional — Crux owns *what the user has*,
shells own *what the user is doing right now*.

| State kind | Where it lives | Examples |
|------------|---------------|----------|
| Domain data | Crux `Model` → `ViewModel` | Items, sessions, routines, active session progress, analytics |
| UI interaction (web) | Leptos signals | Form field values, loading/submitting flags, timer ticks, drag state, tab selection |
| UI interaction (iOS) | SwiftUI `@State` / `@Environment` | Form field values, loading flags, tab selection, toast state |
| Crash recovery | localStorage / UserDefaults | `intrada:session-in-progress` (single key, FR-008) |

**Rules:**
- Domain state must flow through `Event` → `Model` → `ViewModel`. Never store domain data in shell-local state (Leptos signals or SwiftUI `@State`).
- UI state that has no meaning outside the current view stays in shell-local state. Don't inflate the Crux model with ephemeral UI concerns.
- The `ViewModel` is the read-only projection that views consume. Views never mutate it directly.

## Code Style

- Rust stable, 2021 edition
- Follow standard Rust conventions
- `cargo fmt` and `cargo clippy -- -D warnings` must pass
- No `unwrap()` without justification

## Web Design System (Components-First)

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
| `TypeTabs`      | Piece/Exercise toggle switch              |
| `Toast`         | Notification banner                       |
| `ErrorBanner`   | Persistent error display                  |
| `FormFieldError`| Inline validation error display           |
| `Autocomplete`  | Dropdown suggestions for text inputs      |
| `TagInput`      | Chip-based multi-tag input with autocomplete |
| `SkeletonLine`  | Pulsing text-line placeholder             |
| `SkeletonBlock` | Pulsing rectangular placeholder           |
| `SkeletonItemCard` | Library item card skeleton             |
| `SkeletonCardList` | Generic list page skeleton (sessions, routines) |
| `LineChart`     | SVG line chart for analytics              |
| `TempoProgressChart` | Tempo progress visualization         |
| `ProgressRing`  | Circular progress indicator               |
| `LibraryItemCard` | Library item display card               |
| `SessionTimer`  | Active session countdown timer            |
| `SessionSummary`| End-of-session review with scoring        |
| `SetlistBuilder`| Drag-and-drop session item picker         |
| `SetlistEntryRow`| Single entry row in session builder      |
| `TransitionPrompt`| Focus-mode item transition overlay      |
| `DragHandle`    | Drag grip for reorderable lists           |
| `DropIndicator` | Visual drop target for drag-and-drop      |
| `RoutineLoader` | Load routine into session builder         |
| `RoutineSaveForm`| Save current session as routine          |
| `AppHeader`     | Top navigation bar with nav links         |
| `AppFooter`     | Bottom footer (desktop)                   |
| `BottomTabBar`  | Mobile bottom navigation tabs             |
| `WeekStrip`     | Weekly calendar strip navigator with swipe |
| `DayCell`       | Single day cell in week strip (date + dot) |

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

## iOS Design System (Components-First)

**Design consistency and reuse are critical.** The iOS app MUST look and feel like the
same product as the web app. Both platforms share the same dark-on-dark glassmorphism
aesthetic, warm indigo accents, and component vocabulary. Never build one-off UI —
always compose from the shared component library.

### Architecture layers (same hierarchy as web)

```text
1. Tokens       →  Color, Font, Spacing extensions (ios/Intrada/DesignSystem/Tokens/)
2. Modifiers    →  Reusable ViewModifiers for common patterns (ios/Intrada/DesignSystem/Modifiers/)
3. Components   →  SwiftUI views that use tokens internally (ios/Intrada/Components/)
4. Views        →  Feature screens composed entirely from components (ios/Intrada/Views/)
```

### Colour tokens — must match web token names

Every `Color` extension in the iOS design system maps 1:1 to a CSS custom property
in `input.css`. Use the **same token name** (e.g. `.textPrimary`, `.surfaceSecondary`,
`.accent`) so developers can cross-reference between platforms.

Never use raw SwiftUI colours (`.white`, `.gray`, `.indigo`, `.red`) in views or
components. Always use the named token.

### iOS components — mirror the web component library

| iOS Component     | Web Equivalent  | Purpose                                   |
|-------------------|-----------------|-------------------------------------------|
| `CardView`        | `Card`          | Glassmorphism container                   |
| `ButtonView`      | `Button`        | All tappable actions (Primary/Secondary/Danger) |
| `TextFieldView`   | `TextField`     | Form input with label + hint + error      |
| `TextAreaView`    | `TextArea`      | Form textarea with label + hint + error   |
| `StatCardView`    | `StatCard`      | Metric display (title + value + subtitle) |
| `PageHeading`     | `PageHeading`   | Serif page title                          |
| `TypeBadge`       | `TypeBadge`     | Piece/Exercise type pill                  |
| `Toast`           | `Toast`         | Notification banner                       |
| `ErrorBanner`     | `ErrorBanner`   | Persistent error display                  |
| `FormFieldError`  | `FormFieldError`| Inline validation error                   |
| `BackLink`        | `BackLink`      | Back-navigation with arrow                |
| `EmptyStateView`  | (inline)        | No-data empty states                      |
| `SkeletonLine`    | `SkeletonLine`  | Pulsing text placeholder                  |
| `SkeletonBlock`   | `SkeletonBlock` | Pulsing rectangular placeholder           |
| `TypeTabs`        | `TypeTabs`      | Piece/Exercise filter pill toggle         |
| `AutocompleteField` | `Autocomplete` | Text field with dropdown suggestions    |
| `TagInputView`    | `TagInput`      | Chip-based multi-tag input with autocomplete |
| `LibraryItemRow`  | `LibraryItemCard` | Library item list row (title, composer, badges) |
| `ScoreHistoryList` | (inline)       | Score history entries with colour-coded badges |
| `FlowLayout`      | (CSS flexbox)   | Wrapping layout for tag chips and badges   |
| `LibraryQueueRow`  | (new)           | Tap-to-queue library row with toggle state (accent bar + check/plus icon) |
| `SetlistEntryRow`  | `SetlistEntryRow` | Compact entry with drag handle, progressive disclosure for duration/intention/reps |
| `StickyBottomBar`  | (new)           | iPhone bottom bar — item count, total time, Start Session button |
| `ProgressRingView` | `ProgressRing`  | Circular countdown timer using `Circle().trim()` |
| `RepCounterView`   | (inline)        | Rep counter with Got it/Missed buttons, celebration state |
| `ScoreSelectorView` | (inline)       | 1–5 confidence score dot selector |

### iOS views — feature screens

| View | Purpose |
|------|---------|
| `LibraryView` | Root library screen — `NavigationSplitView` (sidebar + detail on iPad, stack on iPhone) |
| `LibraryListContent` | Scrollable item list with filter tabs, search, item count, skeleton/empty states |
| `ItemDetailView` | Full item detail — adaptive layout (single column iPhone, two-column iPad) |
| `AddItemView` | Create item form with client-side validation |
| `EditItemView` | Edit item form with pre-populated fields, PATCH-style double-optional updates |
| `LibrarySkeletonView` | List loading skeleton (5 placeholder rows) |
| `DetailSkeletonView` | Detail pane loading skeleton |
| `LibraryHelpers` | Shared helpers: `FilterTab`, `LibraryFormValidator`, date/tempo formatters |
| `PracticeTabRouter` | State-driven router — renders Idle/Building/Active/Summary based on `session_status` |
| `SessionBuilderView` | Main builder — adaptive iPhone (list + bottom bar) / iPad (split view) layout |
| `SessionBuilderListContent` | Scrollable library list with tap-to-queue rows and search filtering |
| `SetlistSheetContent` | Setlist editor — intention, drag-to-reorder entries, Start Session (sheet on iPhone, panel on iPad) |
| `ActivePracticeView` | Focus-mode active session — timer, progress ring, rep counter, pause overlay, iPhone + iPad layouts |
| `TransitionPromptSheet` | Between-item scoring sheet — score (1–5), tempo wheel picker, notes, Continue/Finish |
| `SessionSummaryView` | Post-session review — header stats, entry list with inline score/tempo/notes editing, Save/Discard |
| `SessionHistoryView` | Chronological session list with date grouping, session cards, swipe-to-delete, empty state |
| `SessionDetailView` | Past session detail — header stats, entry results (read-only) |
| `SessionEntryResultRow` | Shared entry result display — status icon, score/tempo/rep badges, editable or read-only |
| `RoutineListView` | Routines tab root — list of routine cards, empty state, iPad NavigationSplitView |
| `RoutineDetailView` | Routine detail — name, ordered item list, Edit button |
| `RoutineEditView` | Edit routine — rename, reorder, add/remove items, Save/Cancel |
| `RoutineSaveForm` | Collapsible save-as-routine form (name input), used in SetlistSheet and Summary |

### Rules for new iOS UI work

1. **Tokens first**: Every rendered colour MUST trace to a named `Color` extension in
   `DesignSystem/Tokens/`. Never use `.white`, `.indigo`, `.gray` directly.
2. **Modifiers second**: If a styling pattern appears in 2+ places, create a `ViewModifier`
   in `DesignSystem/Modifiers/`.
3. **Components third**: If a pattern includes layout + logic, create a SwiftUI view in
   `Components/`. One file per component.
4. **Xcode Previews**: Every component MUST have a `#Preview` block so the full system
   is visible during development.
5. **Cross-platform parity**: When building an iOS component, reference the matching web
   component for visual behaviour. The user should not be able to tell which platform
   they're on from the visual design.
6. **Dark mode only**: The app forces dark appearance. All tokens are defined for dark mode.
7. **Dynamic Type**: Components must support the system text size setting.
8. **`@Indirect` property wrapper**: All generated type fields use `@Indirect` (an enum
   wrapper for Sendable conformance). Access properties directly (`item.title`, not
   `item.title.value`). In closures, add explicit type annotations for type inference:
   `.map { (item: LibraryItemView) -> String in item.subtitle }` not `.map { $0.subtitle }`.
9. **Trust generated types, not assumptions**: Always read the actual generated type
   definitions in `ios/Intrada/Generated/SharedTypes/SharedTypes.swift` before writing
   code. Don't assume field types (e.g. `itemType` is `ItemKind`, not `String`). After
   writing code, do a **clean** build to catch stale-cache false positives.

Key files: `ios/Intrada/DesignSystem/` (tokens + modifiers), `ios/Intrada/Components/` (SwiftUI components)

### iOS UX pattern rules (canonical — see Pencil "iOS / UX Pattern Guide")

These rules apply to ALL iOS views. See #243 for the consistency sweep.

10. **System navigation titles**: Use `.navigationTitle()` on every screen. Large title
    on root views, inline on pushed views. Never use custom heading text as a title
    replacement (except Active Session focus mode which hides the bar entirely).
11. **NavigationSplitView on iPad**: Every list→detail screen MUST use
    `NavigationSplitView` on iPad (detect via `horizontalSizeClass == .regular`).
    Sidebar width: 320–420pt. No screen may ignore iPad.
12. **CardView for all content sections**: Every content section wraps in `CardView`.
    Dividers are only used INSIDE cards to separate items. Never use bare dividers
    as section separators.
13. **ButtonView for all actions**: Every tappable action uses `ButtonView(variant:)`.
    Never use raw `Button` with custom styling. Toolbar actions use `.toolbar { ToolbarItem }`.
14. **EmptyStateView for all empty states**: Use the `EmptyStateView` component.
    Never use custom VStack empty layouts or `ContentUnavailableView`.
15. **Spacing tokens only**: Use `Spacing.cardCompact` (12), `Spacing.card` (16),
    `Spacing.cardComfortable` (24). Never hardcode spacing values.
16. **Destructive confirmations**: All destructive actions use `.confirmationDialog`
    with `titleVisibility: .visible`. Never delete/discard without asking.
17. **No custom back buttons**: Never use manual BackLink components on iOS.
    NavigationStack provides system back buttons automatically.

## Roadmap Alignment

Every piece of work should connect back to the product vision. Before starting
and after finishing, check alignment with the source-of-truth documents.

### Before starting work

1. **Identify the roadmap item.** Find the issue number in [`docs/roadmap.md`](docs/roadmap.md).
   If the work doesn't map to an existing item, pause and discuss whether it should.
2. **Check the current focus.** iOS feature parity (#195–#201) is the current priority.
   After iOS parity, cross-platform features (Crux core + API) benefit both shells.
   Within the three pillars, prefer `horizon:now` items over others.
3. **Check the project board.** The issue should be in Ready or In Progress on the
   [GitHub project board](https://github.com/users/jonyardley/projects/2). If it's
   still in Backlog, move it to Ready before starting.
4. **Run SpecKit.** Always use SpecKit for feature design before implementation.
   Run `speckit.specify` to create the feature spec, then `speckit.plan` and
   `speckit.tasks` to generate the implementation plan and task breakdown.
   Specs live in `specs/{issue-number}-{slug}/`.
5. **Design in Pencil (if UI work).** After `speckit.specify` and before
   `speckit.plan`, mock new views or significant UI changes in
   `design/intrada.pen`. See [Pencil Design Workflow](#pencil-design-workflow)
   for when and how to use it.

### After completing work

1. **Update the roadmap.** If a feature is done, move it to "What's Built Today" in
   `docs/roadmap.md` and close the GitHub issue.
2. **Update CLAUDE.md.** If the work changes architecture, adds components, or
   introduces new patterns, update the relevant section of this file.
3. **Update Pencil.** If implementation diverged from the design, update the
   Pencil file. Add any new components to the design system and catalogue.
4. **Check for stale items.** Glance at the project board — are there issues that
   are now done, duplicated, or no longer relevant? Flag them.

### Periodic review (every 2 weeks)

These documents should stay in sync. When any one changes, check the others:

| Document | What to check |
|----------|---------------|
| [`docs/roadmap.md`](docs/roadmap.md) | Horizons still accurate? Closed issues removed? New work captured? |
| [`VISION.md`](VISION.md) | Still reflects current product direction? No stale phase references? |
| [`CLAUDE.md`](CLAUDE.md) | Tech stack, components, and patterns up to date? |
| [`design/intrada.pen`](design/intrada.pen) | Design system components match `input.css` tokens? Views match implemented UI? Catalogue up to date? |
| [`docs/user-journey-to-be.drawio`](docs/user-journey-to-be.drawio) | Feature colours match pillar model? New features added? |
| [GitHub project board](https://github.com/users/jonyardley/projects/2) | Board status matches horizon labels? Priorities set? |

## Pencil Design Workflow

`design/intrada.pen` is the single source of truth for visual design. It contains
the full design system (reusable components), app views (desktop + mobile), and a
Design Catalogue reference page.

### When to use Pencil

| Work type | Pencil needed? |
|-----------|---------------|
| New view or screen | **Yes** — design both desktop (1440px) and mobile (375px) before implementing |
| Significant UI change to existing view | **Yes** — mock the change first |
| New reusable component | **Yes** — add to design system + update catalogue frame |
| Visual bug fix | Optional — screenshot to confirm intent |
| Logic/API/backend changes | No |
| Copy or text changes | No |

### Where it fits in the SpecKit flow

```text
1. Identify roadmap item          (docs/roadmap.md)
2. speckit.specify                → spec.md
3. Design in Pencil (if UI work)  → new frames in design/intrada.pen
4. speckit.plan                   → plan.md (reference Pencil frames)
5. speckit.tasks                  → tasks.md
6. Implement
```

### Design file rules

1. **Single file.** All design work lives in `design/intrada.pen`. Don't create
   separate files per feature.
2. **Reuse components.** Always check the existing design system before creating
   ad-hoc markup. If a pattern is used in 2+ places, make it a reusable component.
3. **Desktop + Mobile.** New views need both a 1440px desktop frame and a 375px
   mobile frame.
4. **Naming.** Frames use the view name (e.g. "Library Desktop", "Add Item Mobile").
5. **Design Catalogue.** New components and tokens must be added to the catalogue
   frame so the full system is visible in one place.
6. **Token alignment.** Colours in the Pencil file must match the token variables
   defined in `intrada-web/input.css`. Never use raw hex values for design-system
   colours — reference the Pencil variables.

### Keeping design and code in sync

- After implementation, update the Pencil file if the final UI diverged from the mockup
- When adding a new Leptos component to `components/`, add the matching reusable
  component in Pencil and a showcase entry in the catalogue frame
- When updating tokens in `input.css`, update the corresponding Pencil variables
- The Design Catalogue frame should mirror `views/design_catalogue.rs`

Key files: `design/intrada.pen` (design system + views), `intrada-web/input.css`
(tokens), `intrada-web/src/views/design_catalogue.rs` (code catalogue)

## Known Tech Debt

- Creates still re-fetch the full collection (server assigns ID, client-side merge is fragile)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

## Active Technologies
- Swift 6.0, iOS 17.0+ + SwiftUI, ClerkKit, UniFFI (CoreFfi), BCS serialization (auto-generated) (001-ios-library)
- N/A (all persistence via Crux core HTTP effects → REST API → Turso) (001-ios-library)
- Swift 6.0, iOS 17.0+ + SwiftUI, UniFFI (CoreFfi), BCS serialization (auto-generated types) (196-ios-session-builder)
- N/A (all persistence via Crux core HTTP effects → REST API → Turso; crash recovery via UserDefaults) (196-ios-session-builder)
- Swift 6.0, iOS 17.0+ + SwiftUI, UniFFI (CoreFfi), BCS serialization (auto-generated SharedTypes) (197-ios-active-session)
- UserDefaults via `SessionStorage.swift` (crash recovery only, handled by existing effect processor) (197-ios-active-session)

## Recent Changes
- 001-ios-library: Added Swift 6.0, iOS 17.0+ + SwiftUI, ClerkKit, UniFFI (CoreFfi), BCS serialization (auto-generated)

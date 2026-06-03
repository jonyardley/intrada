# Quickstart: URL Routing for Web App Views

**Feature Branch**: `008-url-routing`
**Date**: 2026-02-14

## Prerequisites

- Rust stable (1.75+)
- trunk 0.21.x installed
- tailwindcss v4 CLI available

## Running the App

```bash
cd crates/intrada-web
trunk serve
```

Open `http://localhost:8080` in a browser.

## Verification Scenarios

### Scenario 1: Route-per-view (US1)

1. Open `http://localhost:8080/` — Library list view renders, URL shows `/`
2. Click on an item card — Detail view renders, URL changes to `/library/{ulid}`
3. Click "Edit" button — Edit form renders, URL changes to `/pieces/{ulid}/edit` or `/exercises/{ulid}/edit`
4. Click back link — Detail view renders, URL reverts to `/library/{ulid}`
5. Navigate to root — Library list renders at `/`
6. Click "Add" → "Piece" — Add piece form renders, URL shows `/pieces/new`
7. Click "Add" → "Exercise" — Add exercise form renders, URL shows `/exercises/new`

### Scenario 2: Browser history (US2)

1. Navigate: `/` → click item → `/library/{id}` → click edit → `/pieces/{id}/edit`
2. Press browser Back — returns to `/library/{id}` (detail view)
3. Press browser Back — returns to `/` (library list)
4. Press browser Forward — returns to `/library/{id}` (detail view)
5. Press browser Forward — returns to `/pieces/{id}/edit` (edit form)

### Scenario 3: Form submission history replacement (US2, FR-011)

1. Navigate to `/pieces/new` (add piece form)
2. Fill in required fields (title, composer) and submit
3. URL changes to `/` (library list) — note: uses `replace` so form entry is removed from history
4. Press browser Back — does NOT return to `/pieces/new`; returns to whatever was before the form

### Scenario 4: Deep linking and refresh (US3)

1. Navigate to an item detail: `/library/{ulid}`
2. Copy the URL
3. Open a new browser tab and paste the URL
4. The correct item detail view loads
5. Refresh the page (Ctrl+R / Cmd+R) — same view reloads

### Scenario 5: Not found (US4)

1. Navigate to `/nonexistent/path` — Not-found view renders with message and link to library
2. Navigate to `/library/invalid-ulid` — Application handles gracefully (not-found or redirect to library)
3. Click the "Back to Library" link — navigates to `/`

### Scenario 6: Accessibility preservation (FR-012)

1. Navigate using keyboard only (Tab, Enter, Space) through library list → detail → edit → back
2. Verify all links are focusable and announce their destination to screen readers
3. Verify `aria-current="page"` is set on active navigation links (provided by `<A>` component)

## Test Commands

```bash
# Run all workspace tests
cargo test --workspace

# Run clippy (zero warnings required)
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Build WASM bundle
cd crates/intrada-web && trunk build --release
```

## Route Table Reference

| Path | View | Parameters |
|---|---|---|
| `/` | Library List | — |
| `/library/:id` | Item Detail | `id`: ULID string |
| `/pieces/new` | Add Piece Form | — |
| `/exercises/new` | Add Exercise Form | — |
| `/pieces/:id/edit` | Edit Piece Form | `id`: ULID string |
| `/exercises/:id/edit` | Edit Exercise Form | `id`: ULID string |
| `/*` (fallback) | Not Found | — |

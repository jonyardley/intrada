# Quickstart: Glassmorphism UI & Responsive Layout

**Feature**: 016-glassmorphism-responsive
**Date**: 2026-02-15

## Prerequisites

- Rust stable (1.75+) with `wasm32-unknown-unknown` target
- Trunk 0.21.x (`cargo install trunk`)
- Tailwind CSS v4 standalone CLI (in project)
- Node.js 18+ with Playwright (for E2E tests)

## Build & Run

```bash
# From repo root
cd crates/intrada-web
trunk serve
# App available at http://localhost:8080
```

## Verification Steps

### 1. Automated Tests (MUST all pass)

```bash
# Core + shell tests (142 tests)
cargo test

# Clippy (zero warnings)
cargo clippy -- -D warnings

# E2E tests (14 tests — must pass unchanged)
cd e2e && npx playwright test
```

**Pass criteria**: All 142 unit tests pass, zero clippy warnings, all 14 E2E tests pass without modification.

### 2. Visual Verification — Desktop (1280px)

1. Open http://localhost:8080 in Chrome at 1280px width
2. **Gradient background**: Deep purple/indigo gradient visible behind all content, does not scroll with page
3. **Glass cards**: Library item cards are semi-transparent with visible blur effect and subtle white border
4. **Light text**: All text is white/light grey — no dark text on glass cards
5. **Header**: Uses glass treatment (semi-transparent with blur), horizontal nav links visible
6. **Footer**: Consistent styling with overall theme
7. **Buttons**: Styled for dark context (indigo primary, translucent secondary)
8. Navigate to Add Item, Detail, Sessions — verify consistent glass treatment on all pages

### 3. Visual Verification — Mobile (375px)

1. Open Chrome DevTools, set viewport to 375px width (iPhone SE)
2. **No horizontal scroll**: Content fits within viewport on all pages
3. **Bottom tab bar**: Persistent tab bar at bottom with Library and Sessions tabs; glass-styled
4. **Header nav hidden**: No horizontal nav links in header on mobile
5. **Touch targets**: All buttons and links are at least 44x44px
6. **Library list**: Cards stack in single column with appropriate padding
7. **Session timer**: Timer digits and control buttons are large and tappable
8. **Forms**: Full-width inputs, readable labels, easy-to-tap buttons

### 4. Visual Verification — Tablet (768px)

1. Set viewport to 768px width (iPad portrait)
2. **Library grid**: Items displayed in two-column grid
3. **Header nav**: Horizontal nav links visible (no bottom tab bar)
4. **Forms**: Comfortable width, not stretched to full screen
5. **Detail view**: Metadata uses available width efficiently

### 5. Accessibility & Edge Cases

1. **Contrast check**: Use Chrome DevTools accessibility audit — verify 4.5:1 contrast ratio for body text on glass cards
2. **Reduced motion**: Enable `prefers-reduced-motion: reduce` in Chrome DevTools (Rendering panel) — verify no transitions or animations play
3. **Backdrop-filter fallback**: In Chrome DevTools, disable `backdrop-filter` via CSS override — verify cards fall back to solid semi-opaque background with readable text
4. **Very wide screen**: Set viewport to 1920px — verify content is centered and constrained (not stretching to fill)
5. **Very small screen**: Set viewport to 320px — verify app is still usable, timer is readable

### 6. Performance Spot-Check

1. Open Chrome DevTools Performance panel
2. Record a scroll through the library list at 375px viewport
3. Verify frame rate stays above 30fps (ideally 60fps)
4. Check Layers panel — confirm no more than 3-4 compositing layers from backdrop-filter

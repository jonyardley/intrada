# Implementation Plan: Glassmorphism UI & Responsive Layout

**Branch**: `016-glassmorphism-responsive` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/016-glassmorphism-responsive/spec.md`

## Summary

Replace the current white-card-on-light-grey aesthetic with a glassmorphism design: deep purple/indigo gradient background with semi-transparent frosted-glass cards using backdrop blur. Simultaneously add responsive breakpoints for mobile (<640px), tablet (640-1024px), and desktop (>1024px), including a bottom tab bar for mobile navigation. This is a visual-only change — no behaviour, data, or routing modifications.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.8.x (CSR), crux_core 0.17.0-rc2, Tailwind CSS v4 (standalone CLI v4.1.18), trunk 0.21.x
**Storage**: N/A (no storage changes — visual only)
**Testing**: Playwright E2E tests (14 existing), cargo test (142 tests), cargo clippy
**Target Platform**: WASM (browser), responsive from 320px to 2560px+
**Project Type**: Crux workspace (intrada-core + intrada-web)
**Performance Goals**: No perceptible rendering lag from glass effects; `backdrop-blur-sm` on mobile, `backdrop-blur-md` on desktop
**Constraints**: All 14 E2E tests must pass unchanged; WCAG 2.1 AA contrast ratios; iOS Safari compatibility for fixed gradient
**Scale/Scope**: ~20 component/view files with Tailwind class changes, 1 new component (bottom tab bar), CSS config updates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | Class attribute updates are readable; no new logic complexity |
| II. Testing Standards | PASS | All 14 E2E tests must pass unchanged (FR-012); no new core logic to test |
| III. UX Consistency | PASS | This feature *establishes* a new consistent visual system across all views |
| IV. Performance | PASS | Research identified mobile backdrop-blur mitigations (sm on mobile, md on desktop) |
| V. Architecture Integrity | PASS | Pure visual change in shell only; zero changes to intrada-core |

### Post-Design Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | No dead code; glass utility classes are self-documenting |
| II. Testing Standards | PASS | E2E tests validate behaviour not styling; all pass unchanged |
| III. UX Consistency | PASS | Uniform glass treatment across all components, three-tier text colour hierarchy |
| IV. Performance | PASS | Moderate blur on mobile, higher on desktop; card lists use solid bg on mobile |
| V. Architecture Integrity | PASS | All changes in intrada-web shell; core untouched |

No violations. Complexity Tracking not needed.

## Project Structure

### Documentation (this feature)

```text
specs/016-glassmorphism-responsive/
├── plan.md              # This file
├── research.md          # Glassmorphism patterns, fallbacks, performance research
├── quickstart.md        # Verification steps
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── spec.md              # Feature specification
```

### Source Code (files to modify)

```text
crates/intrada-web/
├── index.html                          # Add viewport-fit=cover meta tag
├── input.css                           # Add @custom-variant and @utility definitions
├── src/
│   ├── app.rs                          # Fixed gradient background, responsive container
│   ├── components/
│   │   ├── app_header.rs               # Glass header, hide nav on mobile
│   │   ├── app_footer.rs               # Glass footer, light text
│   │   ├── bottom_tab_bar.rs           # NEW: mobile bottom tab bar component
│   │   ├── button.rs                   # Glass-themed button variants
│   │   ├── card.rs                     # Glass card (core change)
│   │   ├── page_heading.rs             # Light text colour
│   │   ├── library_item_card.rs        # Glass card, light text, responsive
│   │   ├── type_badge.rs               # Adjust badge colours for dark bg
│   │   ├── type_tabs.rs                # Glass tab styling
│   │   ├── back_link.rs                # Light link colour
│   │   ├── field_label.rs              # Light label colour
│   │   ├── text_field.rs               # Glass input styling
│   │   ├── text_area.rs                # Glass textarea styling
│   │   ├── form_field_error.rs         # Light error colour
│   │   ├── session_timer.rs            # Glass cards, responsive timer
│   │   ├── session_summary.rs          # Glass cards, light text
│   │   ├── setlist_builder.rs          # Glass styling, light text
│   │   ├── setlist_entry.rs            # Glass entry rows, light text
│   │   └── mod.rs                      # Export bottom_tab_bar
│   └── views/
│       ├── library_list.rs             # Responsive grid, light text, glass cards
│       ├── detail.rs                   # Light text, responsive metadata grid
│       ├── add_form.rs                 # Responsive form layout
│       ├── edit_form.rs                # Responsive form layout
│       ├── sessions.rs                 # Light text, glass cards
│       ├── session_new.rs              # (minimal — wraps components)
│       ├── session_active.rs           # (minimal — wraps components)
│       ├── session_summary.rs          # (minimal — wraps components)
│       └── not_found.rs                # Light text
```

**Structure Decision**: All changes are within the `intrada-web` shell crate. The core crate (`intrada-core`) is untouched. One new component (`bottom_tab_bar.rs`) is created; all other changes are class attribute modifications to existing files.

## Implementation Strategy

### Phase 1: Foundation (CSS Config + Background + Card)

Set up the glass infrastructure that everything else builds on:

1. **`input.css`**: Add `@custom-variant supports-backdrop`, `@utility pb-safe`, and any `@theme` overrides
2. **`index.html`**: Add `viewport-fit=cover` to viewport meta tag
3. **`app.rs`**: Replace `bg-gradient-to-b from-slate-50 to-slate-100` with fixed gradient wrapper + `min-h-screen` on main. Add `pb-20 sm:pb-0` for tab bar clearance
4. **`card.rs`**: Replace `bg-white rounded-xl shadow-sm border border-slate-200 p-6` with glass card pattern. This is the highest-leverage change — most views use `<Card>`.

### Phase 2: Core Components (Header + Footer + Buttons + Forms)

Restyle the structural components that appear on every page:

5. **`app_header.rs`**: Glass header, hide nav links on mobile (`sm:flex hidden`), light text
6. **`app_footer.rs`**: Glass footer or transparent, light text
7. **`bottom_tab_bar.rs`** (NEW): Create bottom tab bar component — fixed, glass-styled, `sm:hidden`, with Library and Sessions tabs. Wire into `app.rs`
8. **`mod.rs`**: Export `BottomTabBar`
9. **`button.rs`**: Restyle all four variants for dark glass context (translucent backgrounds, light text, adjusted hover/focus states)
10. **`text_field.rs`** + **`text_area.rs`**: Glass input styling (translucent bg, light text, border-white/20)
11. **`page_heading.rs`**: `text-white` instead of `text-slate-900`
12. **`back_link.rs`**: Light link colour (`text-gray-400 hover:text-white`)
13. **`field_label.rs`**: Light label colour (`text-gray-400`)
14. **`form_field_error.rs`**: `text-red-400` instead of `text-red-600`

### Phase 3: Content Components

Restyle the components that display library/session data:

15. **`library_item_card.rs`**: Light text hierarchy, responsive padding, tag colours for dark bg
16. **`type_badge.rs`**: Adjust badge colours for visibility on dark glass
17. **`type_tabs.rs`**: Glass tab container, adjusted active/inactive states
18. **`setlist_builder.rs`**: Light text, glass sub-cards
19. **`setlist_entry.rs`**: Light text, glass row styling
20. **`session_timer.rs`**: Light text, responsive timer display
21. **`session_summary.rs`**: Light text, glass entry cards

### Phase 4: Views (Responsive Layouts)

Add responsive breakpoints to view-level layouts:

22. **`library_list.rs`**: Responsive grid (`grid-cols-1 sm:grid-cols-2 lg:grid-cols-1`), light text, responsive padding
23. **`detail.rs`**: Light text, responsive metadata grid (already has `sm:grid-cols-2`)
24. **`add_form.rs`**: Responsive form width, responsive padding
25. **`edit_form.rs`**: Responsive form width, responsive padding
26. **`sessions.rs`**: Light text, responsive padding
27. **`not_found.rs`**: Light text

### Phase 5: Polish + Verification

28. Run all 14 E2E tests — must pass unchanged
29. Run `cargo test` (142 tests) + `cargo clippy`
30. Visual verification at 375px, 768px, and 1280px viewports
31. Test `prefers-reduced-motion` handling
32. Test backdrop-filter fallback (disable in devtools)

## Key Technical Patterns

### Glass Card (replaces all `<Card>` usage)
```
bg-white/10 supports-backdrop:backdrop-blur-md border border-white/15 rounded-xl shadow-lg p-6
```
Fallback: `bg-indigo-950/80` when `backdrop-filter` not supported.

### Fixed Gradient Background
```html
<div class="fixed inset-0 -z-10 bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950"></div>
```

### Text Colour Hierarchy
| Level | Class |
|-------|-------|
| Primary | `text-white` |
| Secondary | `text-gray-300` |
| Tertiary | `text-gray-400` |
| Links | `text-indigo-300 hover:text-indigo-200` |
| Errors | `text-red-400` |

### Responsive Breakpoints
| Tier | Prefix | Use |
|------|--------|-----|
| Mobile | (base) | Single column, bottom tab bar |
| Tablet | `sm:`/`md:` | Two-column grids, horizontal nav |
| Desktop | `lg:` | Max-width container, full layout |

### Bottom Tab Bar
```
fixed inset-x-0 bottom-0 z-50 h-16 bg-gray-900/60 backdrop-blur-md border-t border-white/10 pb-safe sm:hidden
```

### Motion Safety
All `transition-*` classes wrapped with `motion-safe:` prefix.

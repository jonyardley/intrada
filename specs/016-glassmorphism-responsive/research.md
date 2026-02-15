# Research: Glassmorphism UI & Responsive Layout

**Feature**: 016-glassmorphism-responsive
**Date**: 2026-02-15

## 1. Glassmorphism with Tailwind CSS v4

### Decision: Use native Tailwind v4 utilities for all glass effects

**Rationale**: Tailwind v4 provides all necessary utilities out of the box — `backdrop-blur-*`, `bg-color/opacity`, `border-color/opacity`. No plugins or custom CSS needed for the core effect.

**Concrete glass card pattern:**
```
bg-white/10 backdrop-blur-md border border-white/15 rounded-xl shadow-lg
```

**Alternatives considered:**
- Custom CSS with raw `backdrop-filter` — rejected (Tailwind utilities are more maintainable and consistent)
- Third-party glassmorphism library — rejected (unnecessary dependency, Tailwind covers it)

---

## 2. Backdrop-Filter Fallback Strategy

### Decision: Use `@supports` via Tailwind v4 `@custom-variant`

**Rationale**: `backdrop-filter` has 95%+ global browser support, but a fallback ensures the app remains usable on older browsers. Tailwind v4's `@custom-variant` makes this clean.

**Implementation in `input.css`:**
```css
@import "tailwindcss";

@custom-variant supports-backdrop {
  @supports (backdrop-filter: blur(1px)) {
    @slot;
  }
}
```

**Usage pattern**: Default to high-opacity solid background, layer glass effect when supported:
```
bg-indigo-950/80 supports-backdrop:bg-white/10 supports-backdrop:backdrop-blur-md
```

**Alternatives considered:**
- No fallback (rely on 95% support) — rejected (FR-003 requires explicit fallback)
- JavaScript feature detection — rejected (CSS-only is simpler and works without JS)

---

## 3. Text Colour Scheme on Dark Glass

### Decision: Light text hierarchy using white/grey scale

**Rationale**: Dark glassmorphism backgrounds (deep purple/indigo gradient with semi-transparent overlay) require light text for readability. A three-tier hierarchy provides visual structure.

**Colour mapping:**
| Purpose | Current Class | New Class |
|---------|--------------|-----------|
| Primary text (headings, titles) | `text-slate-900` | `text-white` |
| Secondary text (descriptions, metadata) | `text-slate-600`/`text-slate-700` | `text-gray-300` |
| Tertiary text (timestamps, hints) | `text-slate-400`/`text-slate-500` | `text-gray-400` |
| Accent/links | `text-indigo-600` | `text-indigo-300` |
| Error text | `text-red-600` | `text-red-400` |
| Form labels | `text-slate-700` | `text-gray-200` |

**Alternatives considered:**
- Keep dark text, increase card opacity to 80%+ — rejected (defeats the glassmorphism effect; cards become nearly opaque)
- Mixed dark/light approach — rejected (inconsistent and harder to maintain)

---

## 4. Fixed Gradient Background

### Decision: Fixed-position wrapper div (not `bg-fixed`)

**Rationale**: `background-attachment: fixed` is broken on iOS Safari. A `fixed inset-0 -z-10` wrapper works on all browsers.

**Pattern:**
```html
<div class="fixed inset-0 -z-10 bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950"></div>
<main class="relative z-0 min-h-screen">...</main>
```

**Note**: Tailwind v4 uses `bg-linear-to-br` (not `bg-gradient-to-br`).

**Alternatives considered:**
- `bg-fixed` on body — rejected (broken on iOS Safari)
- CSS custom property with `position: fixed` background — rejected (more complex, same outcome)

---

## 5. Bottom Tab Bar Implementation

### Decision: Fixed-position glass tab bar with safe area support

**Rationale**: Bottom tab bar is the chosen mobile navigation pattern (clarified in spec). Requires fixed positioning, glass treatment, and safe area padding for notched devices.

**Pattern:**
```html
<nav class="fixed inset-x-0 bottom-0 z-50 h-16
            bg-gray-900/60 backdrop-blur-md border-t border-white/10
            pb-safe sm:hidden">
  <!-- icon+label tab items -->
</nav>
```

**Custom utility in `input.css`:**
```css
@utility pb-safe {
  padding-bottom: env(safe-area-inset-bottom);
}
```

**Viewport meta tag** (required in `index.html`):
```html
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
```

**Content padding**: Main content needs `pb-20 sm:pb-0` to avoid content being hidden behind the tab bar.

**Alternatives considered:**
- Hamburger menu — rejected by user (bottom tab bar chosen for better one-handed mobile use)
- Slide-out drawer — rejected (more complex, less accessible)

---

## 6. Responsive Breakpoints

### Decision: Use Tailwind v4 default breakpoints, mobile-first

**Rationale**: The default breakpoints align well with the three-tier requirement.

**Mapping:**
| Tier | Width | Tailwind Prefix | Strategy |
|------|-------|-----------------|----------|
| Mobile | <640px | (base, unprefixed) | Single column, bottom tab bar, reduced padding |
| Tablet | 640px–1024px | `sm:` through `md:` | Two-column grids, horizontal nav, moderate padding |
| Desktop | >1024px | `lg:` and above | Current layout style, max-width constraint |

**Max-width variants** (`max-sm:`, `max-lg:`) available in v4 for exclusive targeting if needed.

**Alternatives considered:**
- Custom breakpoints — rejected (defaults are standard and well-tested)

---

## 7. Reduced Motion Handling

### Decision: Use `motion-safe:` prefix for all transitions

**Rationale**: Tailwind's `motion-safe:` variant is the additive pattern — transitions only apply when the user hasn't requested reduced motion. This is less code than undoing animations.

**Pattern:**
```
motion-safe:transition-colors motion-safe:duration-200
```

**Alternatives considered:**
- `motion-reduce:` (subtractive) — rejected (more verbose, requires undoing each animation)
- Custom media query — rejected (Tailwind variant is cleaner)

---

## 8. Backdrop-Filter Performance on Mobile

### Decision: Use moderate blur (sm/md) on mobile, higher on desktop

**Key findings:**
- `backdrop-blur-sm` (8px) is significantly cheaper than `backdrop-blur-xl` (24px)
- Each glass element creates a GPU compositing layer
- Fixed glass elements (header, tab bar) with scrolling content beneath are the biggest bottleneck
- Limit to 1-3 simultaneously visible glass elements with backdrop-filter on mobile

**Mitigation pattern:**
```
backdrop-blur-sm lg:backdrop-blur-md
```

**For scrollable card lists on mobile**: Use solid semi-transparent backgrounds without backdrop-blur to avoid per-card compositing overhead:
```
bg-white/10 lg:backdrop-blur-sm
```

**Alternatives considered:**
- Full backdrop-blur on all elements at all sizes — rejected (causes frame drops on mid-range Android devices)
- No backdrop-blur on mobile — rejected (too aggressive; moderate blur on fixed elements is fine)

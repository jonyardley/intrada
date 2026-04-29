# Design Refresh 2026

> Status: Plan. No code yet. Last updated 2026-04-29.

## Problem

The current visual language is the post-glassmorphism dark theme established in late 2025: warm-indigo accent, purple-tinted dark gradient background, white-on-dark cards at 12% opacity, rounded badges. It works but reads as generic SaaS dark mode — nothing about it says *practice tool* or feels distinctively intrada.

Six new design frames in `design/intrada.pen` (`NEW DESIGN - Library/Practice/Routines/Analytics`, `M3 — Piece Detail`, `NEW DESIGN - Add piece`) propose a tighter visual system: neutral-dark background, very subtle 3% white cards, a 4px gradient accent bar inset on the left of every list row and detail card, larger Source Serif headings, uppercase letter-spaced section labels, Inter as the body font.

This spec captures the delta and breaks the rollout into phases that can ship independently.

## Visual delta — what actually changes

### Token-level

| Token | Current | New | Why |
|-------|---------|-----|-----|
| Background gradient | purple-tinted (`#1A103A` → `#080818`) | neutral dark (`#111827` → `#0A0F1A`) | Removes the "dark indigo SaaS" feel — content reads cleaner |
| Body font | system-ui stack | **Inter** (with system fallback) | Distinct, recognisable, plays well with Source Serif |
| Card surface (rows) | white/12% | white/3% (`#FFFFFF08`) | Cards become whispers; the accent bar carries the weight |
| Section label colour | n/a (no pattern existed) | `#9CA3AF` 11–12px uppercase letter-spacing 1px | New IA pattern — visible structure without visual weight |
| Page heading size | `text-2xl` (24px) web, `text-3xl` (30px) iOS | **34px** weight 700 everywhere | Real iOS large-title scale, visually the page anchor |
| List row corner radius | varies | 10px | Tighter than current 12px |

Page padding becomes `[12, 20, 0, 20]` (12 top, 20 horizontal, content extends to tab bar). Section gap is 20px. Detail-card gap is 14px.

### New primitives (don't exist today)

1. **AccentRow** — list-row primitive with a 4px-wide gradient bar absolutely positioned on the left edge, clipped to the row's 10px radius. Two gradient variants ship for v1: gold (`#D4A050 → #B8860B`) and blue (`#6B9FD4 → #4A7AB8`). The bar is the signature element of the new design.
2. **InlineTypeIndicator** — 6px coloured dot + 12px label text. Replaces the boxed `TypeBadge` for in-row use (boxed badge stays for the Add Piece type-selector).
3. **SectionLabel** — uppercase 11–12px weight 600 letter-spacing 1px, colour `text-muted`. Used above any grouped content (DETAILS / RECENT ACTIVITY / NOTES / THIS WEEK).
4. **DetailGroup** — card containing one SectionLabel, a stack of label/value rows, and the inset accent bar. Padding `[12, 16, 12, 20]` — the extra 4px on the left accommodates the bar.
5. **CircularButton** — accent-filled round primary action (56px play, 44px secondary reset). Replaces the current pill-button for player controls.

### Components that change shape

- **PageHeading**: title to 34px / weight 700 (Source Serif). Trailing slot stays for `+` action.
- **TextField**: label uppercase 12px letter-spacing 1px, input height 48px, border removed (`stroke: 0`), background `#FFFFFF0D`, radius 12.
- **TextArea**: same form treatment as TextField, height 100px.
- **Button (primary)**: 48–52px height, 16–17px font weight 600. Accent fill, white text. Radius 12.
- **StatCard**: gains the inset accent gradient bar (gold or blue), value text uses accent colour.
- **TypeBadge**: keep for the Add Piece type-selector (boxed pill); deprecate inline use in favour of InlineTypeIndicator.
- **BottomTabBar**: minor — confirm active colour is `#A5AAFC` (it already is, post-PR #347).

### Components that stay (token swap only)

Toast/*, ErrorBanner, FormFieldError, Skeleton*, AppHeader, EmptyState, BottomSheet, SwipeActions, ContextMenu, PullToRefresh, ProgressRing, LineChart, TempoProgressChart, WeekStrip, TypeTabs, BackLink, Card.

### Components that get retired

None for v1. Some inline `TypeBadge` callsites get replaced by `InlineTypeIndicator` but the component itself stays.

## Phased rollout

Each phase is one PR unless noted. Phases are ordered by dependency, not by visual impact.

### Phase 0 — Foundation tokens (S)
- Update CSS variables: bg-gradient-top/bottom, surface-secondary (3%).
- Add Inter via @font-face or Google Fonts; add to `@theme` `--font-body`.
- No component changes — verify nothing visibly regresses.
- **Visual cost**: whole app loses the purple cast. Ship Phase 1 close behind so this doesn't sit in production looking half-done.

### Phase 1 — New primitives + design catalogue (M)
- `AccentRow` Leptos component (with `variant: Gold | Blue` prop).
- `InlineTypeIndicator` Leptos component.
- `SectionLabel` utility class in input.css.
- `DetailGroup` Leptos component.
- Catalogue entries for all four with Pencil-faithful examples.
- **No production view consumes them yet** — gate the visual change to the catalogue page so we can iterate before the full rollout.

### Phase 2 — Stat & action primitives (S)
- Update `StatCard` for inset accent bar variant.
- New `CircularButton` component (player controls).
- Tweaks to `Button` (size/weight to match Pencil).

### Phase 3 — Page rollout (one PR per pillar)
Order chosen by surface area, smallest first to validate the pattern:
- 3a — **Routines** (smallest, single list pattern)
- 3b — **Library** (list + search + tabs)
- 3c — **Sessions/Practice** (week strip + active session + summary)
- 3d — **Analytics** (stat cards + bar chart + recent activity)
- 3e — **Detail view** (Piece Detail — uses every new primitive)

### Phase 4 — Forms (M)
- Touches `TextField`, `TextArea`, `add_form.rs`, `edit_form.rs`, `routine_edit.rs`, `routine_save_form.rs`, session-summary inline inputs.
- Uppercase labels, border-less inputs, larger touch targets.

### Phase 5 — Catalogue & cleanup (S)
- Update every catalogue entry to the new tokens.
- Remove dead CSS tokens, deprecated badge variants, any leftover purple-tinted utilities.

### Phase 6 — Test sweep (S)
- Re-run E2E, fix any selector breakage from structural changes.
- Visual screenshot diff if we have it.

**Total**: ~10 PRs across ~2 weeks of focused work. Each PR is independently shippable and reversible.

## Open questions

1. **Accent-bar colour mapping** — gold or blue per row. Pencil shows alternation. Should it map to *type* (gold = piece, blue = exercise — strongest signal), to *recency*, or stay alternating-by-position (purely cosmetic)? **Default: map to type.**
2. **Inter hosting** — local `@font-face` (one extra ~150KB woff2 in the bundle) or Google Fonts CDN (one less network hop on cold start, privacy footprint)? **Default: local @font-face.**
3. **iOS-only vs cross-platform** — these designs are mobile frames (375×812). Desktop (≥sm) needs the same tokens but the layouts naturally diverge (no tab bar, wider cards, multi-column lists). Approach: tokens + primitives apply universally; layouts re-flow per breakpoint as today. **No desktop-specific Pencil frames yet — we're inferring.**
4. **SwiftUI shell** — ON HOLD per CLAUDE.md. Update its tokens too in Phase 0 (cheap, future-proofs the on-hold path) or leave? **Default: leave; document the divergence.**

## Out of scope

- Crux core / API / DB — no behaviour changes
- SwiftUI shell beyond optional token swap
- Icon library migration (#345) — independent track
- Liquid Glass for non-tab-bar surfaces (would need refraction WebKit doesn't support)
- Light mode (no Pencil frames; we're dark-mode-only today)

## Risks

- **Inter font weight mismatch**: every text element gets visually heavier. Mitigate by testing Phase 0 on iOS sim before merging — system fallback should keep the layout stable if Inter is slow to load.
- **Background gradient is loud**: removing the purple cast changes the *feel* of every screen instantly. Ship Phase 0 + Phase 1 close in time so users don't catch it half-shipped.
- **Accent bar everywhere reads as decoration**: if Phase 3a (Routines) lands and the bars feel busy rather than signature, reassess before 3b. Cheap pivot: drop saturation on the gradient or only show bars on certain row types.
- **E2E selector breakage**: most assertions are by text/role and survive. The grouped-list / library-list structural changes in Phase 3 may break a few. Budget half a day in Phase 6.
- **Catalogue drift**: easy to ship view changes and forget to update the catalogue. Phase 5 closes this; reviewers should reject Phase 3 PRs that don't update their respective catalogue section in the same PR.

## Acceptance

This refresh is "done" when:
1. All six Pencil frames render at parity in the iOS sim (visual diff against the Pencil screenshots).
2. Desktop ≥sm renders the same tokens with sensible re-flow (no broken layouts, no purple cast).
3. The design catalogue showcases every new primitive and reflects updated existing components.
4. CI green; no E2E flakes.
5. Pencil and code agree (no "Pencil shows X but code does Y" drift introduced by this work).

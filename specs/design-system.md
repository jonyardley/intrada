# Intrada Design System

> Living reference for design tokens, composite utilities, and component inventory.
> All tokens are defined in `crates/intrada-web/input.css` using Tailwind CSS v4's `@theme` directive.

## Colour Tokens

All colour values use the `oklch()` colour space to match Tailwind v4's internal representation.

### Surface Colours (Backgrounds)

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-surface-primary` | `oklch(100% 0 0 / 10%)` | white/10 | Glass cards (with backdrop-filter) |
| `--color-surface-secondary` | `oklch(100% 0 0 / 5%)` | white/5 | Subtle cards, library item cards |
| `--color-surface-chrome` | `oklch(15.6% 0.011 261.692 / 60%)` | gray-900/60 | Header, bottom tab bar |
| `--color-surface-fallback` | `oklch(25.7% 0.09 281.288 / 80%)` | indigo-950/80 | No-blur fallback for glass cards |
| `--color-surface-hover` | `oklch(100% 0 0 / 20%)` | white/20 | Hover states on glass surfaces |
| `--color-surface-input` | `oklch(100% 0 0 / 10%)` | white/10 | Form input backgrounds |

### Accent Colours (Indigo)

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-accent` | `oklch(51.1% 0.262 276.966)` | indigo-600 | Primary buttons, active tabs, score buttons |
| `--color-accent-hover` | `oklch(58.5% 0.233 277.117)` | indigo-500 | Button hover states |
| `--color-accent-text` | `oklch(78.5% 0.115 274.713)` | indigo-300 | Active navigation links |
| `--color-accent-focus` | `oklch(67.3% 0.182 276.935)` | indigo-400 | Focus rings, input focus borders |

### Danger Colours (Red)

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-danger` | `oklch(57.7% 0.245 27.325)` | red-600 | Danger buttons |
| `--color-danger-hover` | `oklch(63.7% 0.237 25.331)` | red-500 | Danger button hover |
| `--color-danger-text` | `oklch(70.4% 0.191 22.216)` | red-400 | Danger text, outline labels |
| `--color-danger-surface` | `oklch(63.7% 0.237 25.331 / 10%)` | red-500/10 | Danger button backgrounds (outline variant) |

### Border Colours

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-border-default` | `oklch(100% 0 0 / 10%)` | white/10 | Separators, list borders, header/tab bar borders |
| `--color-border-card` | `oklch(100% 0 0 / 15%)` | white/15 | Card borders (slightly more visible) |
| `--color-border-input` | `oklch(100% 0 0 / 20%)` | white/20 | Form input borders |

### Text Colours

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-text-primary` | `oklch(100% 0 0)` | white | Headings, titles, button labels |
| `--color-text-secondary` | `oklch(86.9% 0.022 252.894)` | gray-300 | Body text, descriptions |
| `--color-text-label` | `oklch(92.9% 0.013 255.508)` | gray-200 | Form labels |
| `--color-text-muted` | `oklch(70.4% 0.04 256.788)` | gray-400 | Hints, metadata, placeholders |
| `--color-text-faint` | `oklch(55.4% 0.046 257.417)` | gray-500 | Timestamps, tertiary info |

### Badge Colours

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-badge-piece-bg` | `oklch(60.6% 0.25 292.717 / 20%)` | violet-500/20 | Piece type badge background |
| `--color-badge-piece-text` | `oklch(81.1% 0.111 293.571)` | violet-300 | Piece type badge text |
| `--color-badge-exercise-bg` | `oklch(69.6% 0.17 162.48 / 20%)` | emerald-500/20 | Exercise type badge background |
| `--color-badge-exercise-text` | `oklch(84.5% 0.143 164.978)` | emerald-300 | Exercise type badge text |

### Chart Colours (SVG)

These tokens are referenced in SVG attributes via `var()` in `line_chart.rs`.

| Token | Value | Tailwind Equivalent | Usage |
|-------|-------|---------------------|-------|
| `--color-chart-line` | `oklch(67.3% 0.182 276.935)` | indigo-400 | Line stroke, data point fill |
| `--color-chart-area` | `oklch(67.3% 0.182 276.935 / 15%)` | indigo-400/15 | Area fill under the line |
| `--color-chart-point-stroke` | `oklch(25.7% 0.09 281.288)` | indigo-950 | Data point circle stroke |
| `--color-chart-grid` | `oklch(100% 0 0 / 10%)` | white/10 | Axis grid lines |
| `--color-chart-grid-mid` | `oklch(100% 0 0 / 5%)` | white/5 | Mid-point dashed grid line |
| `--color-chart-label` | `oklch(70.4% 0.04 256.788 / 70%)` | gray-400/70 | Axis label text |

## Radius Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-card` | `var(--radius-xl)` | Cards, containers |
| `--radius-button` | `var(--radius-lg)` | Buttons |
| `--radius-input` | `var(--radius-lg)` | Form inputs |
| `--radius-badge` | `var(--radius-md)` | Tag badges |
| `--radius-pill` | `9999px` | Type tabs, full-round elements |

## Composite Utilities

These `@utility` classes combine multiple CSS properties into single class names. Use them in Leptos `view!` macros like any Tailwind class.

### `card`

The primary container surface for the 2026 design language. Replaces the previous `glass-card` glassmorphism utility — see the historical-context comment in `input.css` for why.

```css
@utility card {
  background-color: var(--color-surface-faint);
  border-radius: var(--radius-surface);
  box-shadow: var(--shadow-card);
}
```

**Usage:** `<div class="card p-card sm:p-card-comfortable">` — add padding and any additional classes alongside.

**Surface family:** `card`, `accent-row`, `detail-group`, `stat-card-faint` all share `--color-surface-faint`, `--radius-surface` (10px), `--shadow-card`. The `accent-row` / `detail-group` / `stat-card-faint` variants layer a 4px gradient bar on top.

**Components using this:** `Card`, `StatCard` (classic variant).

### `glass-chrome`

Navigation chrome (header, bottom tab bar). Uses a neutral gray base instead of indigo.

```css
@utility glass-chrome {
  background-color: var(--color-surface-chrome);

  @supports (backdrop-filter: blur(1px)) {
    backdrop-filter: blur(12px);
  }
}
```

**Usage:** Add border and layout classes alongside: `<header class="glass-chrome border-b border-border-default">`

**Components using this:** `AppHeader`, `BottomTabBar`

### `input-base`

Shared form input styling for text inputs and textareas. Includes focus ring, placeholder colour, and consistent sizing.

```css
@utility input-base {
  width: 100%;
  border-radius: var(--radius-input);
  border: 1px solid var(--color-border-input);
  background-color: var(--color-surface-input);
  padding: 0.625rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-primary);

  &::placeholder { color: var(--color-text-muted); }
  &:focus {
    border-color: var(--color-accent-focus);
    outline: none;
    box-shadow: 0 0 0 1px var(--color-accent-focus);
  }
}
```

**Usage:** `<input class="input-base" />` or `<textarea class="input-base" />`

**Components using this:** `TextField`, `TextArea`, `SessionSummary` (entry notes + session notes)

## Custom Variants

### `supports-backdrop`

Progressive enhancement variant for `backdrop-filter` support.

```css
@custom-variant supports-backdrop {
  @supports (backdrop-filter: blur(1px)) { @slot; }
}
```

**Usage:** `supports-backdrop:bg-surface-secondary` — applies the class only when backdrop-filter is supported.

## Component Inventory

All components live in `crates/intrada-web/src/components/`.

### Layout Components

| Component | File | Description |
|-----------|------|-------------|
| `AppHeader` | `app_header.rs` | Top navigation bar with logo and desktop nav links. Uses `glass-chrome`. |
| `AppFooter` | `app_footer.rs` | Bottom footer with app info. |
| `BottomTabBar` | `bottom_tab_bar.rs` | Mobile bottom navigation (Library, Sessions, Analytics). Uses `glass-chrome`. Hidden on `sm:` and wider. |

### Container Components

| Component | File | Props | Description |
|-----------|------|-------|-------------|
| `Card` | `card.rs` | `children` | Generic content container. Uses the `card` utility (whisper-soft surface, subtle shadow). |
| `StatCard` | `stat_card.rs` | `title`, `value`, `subtitle?`, `bar?`, `tone?` | Metric display card for analytics. Classic variant uses the `card` utility; refresh variant adds a gradient accent bar. |
| `LibraryItemCard` | `library_item_card.rs` | `item: LibraryItemView` | Library list item with title, metadata, tags, type badge. Uses token-based surface colours. |

### Form Components

| Component | File | Props | Description |
|-----------|------|-------|-------------|
| `TextField` | `text_field.rs` | `id`, `label`, `value`, `field_name`, `errors`, `required?`, `placeholder?`, `hint?`, `input_type?` | Labelled text input with validation. Uses `input-base`. |
| `TextArea` | `text_area.rs` | `id`, `label`, `value`, `field_name`, `errors`, `rows?`, `placeholder?`, `hint?` | Labelled textarea with validation. Uses `input-base`. |
| `TypeTabs` | `type_tabs.rs` | `active`, `on_change?` | Piece/Exercise toggle switch. Pill-style segmented control. |
| `FieldLabel` | `field_label.rs` | `text` | Definition-term label for detail displays. |
| `FormFieldError` | `form_field_error.rs` | `field`, `errors`, `error_id` | Inline validation error message. |

### Button Components

| Component | File | Variants | Description |
|-----------|------|----------|-------------|
| `Button` | `button.rs` | `Primary`, `Secondary`, `Danger`, `DangerOutline` | Shared button with disabled/loading states. |

### Display Components

| Component | File | Props | Description |
|-----------|------|-------|-------------|
| `TypeBadge` | `type_badge.rs` | `item_type` | Coloured badge (violet for piece, emerald for exercise). |
| `PageHeading` | `page_heading.rs` | `text` | Page-level `<h2>` heading. |
| `BackLink` | `back_link.rs` | `label`, `href` | Back-navigation link with arrow. |
| `ErrorBanner` | `error_banner.rs` | (reads context) | Dismissible global error alert. |
| `LineChart` | `line_chart.rs` | `data: Vec<DailyPracticeTotal>` | SVG line chart for analytics. Uses `--color-chart-*` tokens. |

### Session Components

| Component | File | Description |
|-----------|------|-------------|
| `SessionTimer` | `session_timer.rs` | Live countdown/countup timer display. |
| `SessionSummary` | `session_summary.rs` | End-of-session review with scoring, notes, save/discard. |
| `SetlistBuilder` | `setlist_builder.rs` | Drag-and-drop session setlist builder. |
| `SetlistEntryRow` | `setlist_entry.rs` | Individual setlist entry row. |

## Design Patterns

### Glassmorphism (Legacy — chrome only)

The 2026 design language dropped glassmorphism for content surfaces in favour of the flat `card` family. The remaining glass-style surfaces are **chrome** (the app header / bottom tab bar via `glass-chrome`, plus the bottom sheet / context menu / grouped-list-row in iOS-feel mode), where the translucent + blur effect still earns its keep as a navigation cue.

For all other surfaces, use the `card` utility or one of its accent-bar siblings (`accent-row`, `detail-group`, `stat-card-faint`).

### Motion Safety

All CSS transitions are wrapped in `motion-safe:` to respect `prefers-reduced-motion`. This includes:
- `motion-safe:transition-colors` on interactive elements
- `motion-safe:transition-all motion-safe:duration-150` on score buttons

### Touch Targets

Interactive elements maintain a minimum `min-h-[44px]` for touch accessibility (WCAG 2.5.5).

### Responsive Strategy

The app uses a single breakpoint: `sm:` (640px).
- Below `sm:`: Mobile layout with `BottomTabBar`, compact padding
- At/above `sm:`: Desktop layout with header nav, wider padding

### Background

The app uses a fixed gradient background that stays behind all content:
```
bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950
```

## Usage Guidelines

### When to use which token

- **Adding a new card/container:** Use the `card` utility + padding classes (or an accent-bar sibling: `accent-row`, `detail-group`, `stat-card-faint`)
- **Adding a new form input:** Use `input-base` class
- **Referencing colours in Rust/Leptos:** Use Tailwind classes like `text-text-muted`, `bg-surface-primary`, `border-border-default`
- **Referencing colours in SVG attributes:** Use `var(--color-chart-*)` or `var(--color-*)` directly
- **Adding hover states on glass surfaces:** Use `hover:bg-surface-hover`

### Adding new tokens

1. Add the token to the `@theme` block in `input.css`
2. Use `oklch()` colour space values (check Tailwind v4 source for palette values)
3. Document the token in this file
4. The token automatically becomes available as a Tailwind utility (e.g., `bg-surface-new`)

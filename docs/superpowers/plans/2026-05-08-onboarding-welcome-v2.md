# Onboarding Welcome v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Iterate the welcome carousel with layered typography (label + anchor + continuation), per-card animated SVG marks, and a Library-not-AddItem CTA destination.

**Architecture:** Pure UI iteration on the existing `WelcomeCarousel`. New `WelcomeMark` component contains five tiny inline-SVG sub-components, one per card, each with a CSS-keyframe-driven draw-in animation. `WelcomeCard` prop API changes from `copy: String` to `label: Option<String> + anchor: String + continuation: Option<String>`. Card content array restructures to match. CTA destination changes from `/library/new` → `/`, label changes from "Add your first piece →" → "Get started →". Animations respect `prefers-reduced-motion`.

**Tech Stack:** Leptos 0.8 (CSR + WASM), Tailwind CSS v4 (with `@utility` and CSS keyframes), inline SVG with `stroke-dasharray` / `stroke-dashoffset` and CSS `transform` for animation.

**Spec:** [`specs/onboarding-welcome.md`](../../../specs/onboarding-welcome.md) — v2 section in card content table, visual treatment, flow.

---

## File map

| Action | File | Responsibility |
|--------|------|---------------|
| Modify | `crates/intrada-web/input.css` | Per-card SVG animation keyframes + reduced-motion gate (~70 new lines) |
| Create | `crates/intrada-web/src/components/welcome_mark.rs` | Single `WelcomeMark` component routing to five private mark sub-components (Line, Dots, Converge, Pulse, Chart) |
| Modify | `crates/intrada-web/src/components/welcome_card.rs` | Refactor props from `copy: String` → `label: Option<String> + anchor: String + continuation: Option<String>` |
| Modify | `crates/intrada-web/src/components/welcome_carousel.rs` | Replace `CARDS: [&str; 5]` with structured `CardContent` array; render `WelcomeMark` + new `WelcomeCard` props; CTA destination `/`; CTA label "Get started →" |
| Modify | `crates/intrada-web/src/components/mod.rs` | Register `welcome_mark` module + re-export `WelcomeMark` |
| Modify | `crates/intrada-web/src/views/design_catalogue.rs` | Update WelcomeCard showcase to new prop shape + add WelcomeMark showcase |
| Modify | `e2e/tests/welcome.spec.ts` | Update CTA test for new button label and `/` destination |

---

### Task 1: CSS animation keyframes + reduced-motion gate

**Files:**
- Modify: `crates/intrada-web/input.css` (append at end)

This task is pure CSS. The keyframes will be referenced by classes added in Task 2 (`.welcome-mark--line`, `--dots`, `--converge`, `--pulse`, `--chart`). Adding the CSS first means Task 2 can just attach class names without needing to edit CSS again.

- [ ] **Step 1: Append the welcome-mark animation block to `input.css`**

Open `crates/intrada-web/input.css` and append after the existing welcome-dots / toast blocks (the file ends with the toast keyframes; new block goes after the final `}`):

```css

/* ═══════════════════════════════════════════════════════════════════════
   Welcome mark animations — brand-vocabulary marks (lines + dots)
   ═══════════════════════════════════════════════════════════════════════
   Each card's WelcomeMark uses the same primitives as the brand logo
   (horizontal lines + violet/amber dots). Animations express the
   per-card concept by what the dots DO on the lines.

   Lines render in --color-text-faint with stroke-width 2 (matches the
   Icon component convention). Violet dots use --color-accent. Amber
   dots use --color-warm-accent.

   Animations run once on mount; mark stays at final state. Re-mounting
   the SVG (via card_index change) replays the animation. Collapses to
   instant final state under prefers-reduced-motion. */

.welcome-mark {
  width: 80px;
  height: 80px;
  display: block;
  /* The mark's lines and dots set their own colours via stroke/fill
     attributes referencing tokens; no inherited color needed. */
}

/* Shared timing curve — matches the iOS-default spring feel. */
:root {
  --welcome-mark-ease: cubic-bezier(0.65, 0, 0.35, 1);
}

/* ── Card 1: line draws in, then violet dot slides along it ────────── */
@keyframes welcome-mark-line-draw {
  from { stroke-dashoffset: 50; opacity: 0; }
  to   { stroke-dashoffset: 0; opacity: 1; }
}
@keyframes welcome-mark-dot-slide-1 {
  /* Dot starts off-line at the left, fades in once the line is drawn,
     then slides right to settle past centre. The translate values are
     in SVG user units (1:1 with px at 80 viewBox + 80px display). */
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(30px); }
}
.welcome-mark--opener line {
  stroke-dasharray: 50;
  animation: welcome-mark-line-draw 500ms ease-out forwards;
}
.welcome-mark--opener .welcome-dot {
  transform-box: fill-box;
  transform-origin: center;
  animation: welcome-mark-dot-slide-1 800ms var(--welcome-mark-ease) 200ms backwards;
}

/* ── Card 2: three lines draw staggered, dots snap on after ────────── */
@keyframes welcome-mark-dot-snap {
  0%   { transform: scale(0); opacity: 0; }
  60%  { transform: scale(1.2); opacity: 1; }
  100% { transform: scale(1); opacity: 1; }
}
.welcome-mark--capture line {
  stroke-dasharray: 56;
  animation: welcome-mark-line-draw 400ms ease-out backwards;
}
.welcome-mark--capture line:nth-of-type(1) { animation-delay: 0ms; }
.welcome-mark--capture line:nth-of-type(2) { animation-delay: 80ms; }
.welcome-mark--capture line:nth-of-type(3) { animation-delay: 160ms; }
.welcome-mark--capture .welcome-dot {
  transform-box: fill-box;
  transform-origin: center;
  animation: welcome-mark-dot-snap 400ms var(--welcome-mark-ease) backwards;
}
.welcome-mark--capture .welcome-dot:nth-of-type(1) { animation-delay: 350ms; }
.welcome-mark--capture .welcome-dot:nth-of-type(2) { animation-delay: 480ms; }
.welcome-mark--capture .welcome-dot:nth-of-type(3) { animation-delay: 610ms; }

/* ── Card 3: lines fade in, then violet dots slide horizontally to align ── */
@keyframes welcome-mark-line-fade {
  from { opacity: 0; }
  to   { opacity: 1; }
}
.welcome-mark--align line {
  animation: welcome-mark-line-fade 300ms ease-out backwards;
}
.welcome-mark--align line:nth-of-type(1) { animation-delay: 0ms; }
.welcome-mark--align line:nth-of-type(2) { animation-delay: 60ms; }
.welcome-mark--align line:nth-of-type(3) { animation-delay: 120ms; }
.welcome-mark--align .welcome-dot {
  transform-box: fill-box;
  transform-origin: center;
}
/* Each dot starts at its scattered position (set via SVG cx) and
   translates to the same final column. The keyframes encode the
   delta so dots end visually aligned. */
@keyframes welcome-mark-align-1 {
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(20px); }
}
@keyframes welcome-mark-align-2 {
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(-20px); }
}
@keyframes welcome-mark-align-3 {
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(10px); }
}
.welcome-mark--align .welcome-dot:nth-of-type(1) {
  animation: welcome-mark-align-1 700ms var(--welcome-mark-ease) 250ms backwards;
}
.welcome-mark--align .welcome-dot:nth-of-type(2) {
  animation: welcome-mark-align-2 700ms var(--welcome-mark-ease) 350ms backwards;
}
.welcome-mark--align .welcome-dot:nth-of-type(3) {
  animation: welcome-mark-align-3 700ms var(--welcome-mark-ease) 450ms backwards;
}

/* ── Card 4: line + violet dot pulses with rings; amber dot slides in ── */
@keyframes welcome-mark-ring-pulse {
  0%   { opacity: 0; transform: scale(1); }
  20%  { opacity: 0.8; }
  100% { opacity: 0; transform: scale(2.5); }
}
@keyframes welcome-mark-dot-pulse {
  0%   { transform: scale(0); opacity: 0; }
  40%  { transform: scale(1); opacity: 1; }
  60%  { transform: scale(1.15); }
  100% { transform: scale(1); opacity: 1; }
}
@keyframes welcome-mark-amber-enter {
  0%   { opacity: 0; transform: translateX(20px); }
  100% { opacity: 1; transform: translateX(0); }
}
.welcome-mark--practice line {
  stroke-dasharray: 50;
  animation: welcome-mark-line-draw 400ms ease-out forwards;
}
.welcome-mark--practice .welcome-dot--violet {
  transform-box: fill-box;
  transform-origin: center;
  animation: welcome-mark-dot-pulse 700ms var(--welcome-mark-ease) 300ms backwards;
}
.welcome-mark--practice .welcome-ring {
  transform-box: fill-box;
  transform-origin: center;
  animation: welcome-mark-ring-pulse 1000ms ease-out 500ms backwards;
}
.welcome-mark--practice .welcome-ring:nth-of-type(2) { animation-delay: 700ms; }
.welcome-mark--practice .welcome-dot--amber {
  transform-box: fill-box;
  transform-origin: center;
  animation: welcome-mark-amber-enter 600ms var(--welcome-mark-ease) 700ms backwards;
}

/* ── Card 5: three lines, violet dots stagger rightward as progress ── */
.welcome-mark--track line {
  animation: welcome-mark-line-fade 300ms ease-out backwards;
}
.welcome-mark--track line:nth-of-type(1) { animation-delay: 0ms; }
.welcome-mark--track line:nth-of-type(2) { animation-delay: 60ms; }
.welcome-mark--track line:nth-of-type(3) { animation-delay: 120ms; }
.welcome-mark--track .welcome-dot {
  transform-box: fill-box;
  transform-origin: center;
}
@keyframes welcome-mark-step-1 {
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(15px); }
}
@keyframes welcome-mark-step-2 {
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(28px); }
}
@keyframes welcome-mark-step-3 {
  0%   { opacity: 0; transform: translateX(0); }
  30%  { opacity: 1; transform: translateX(0); }
  100% { opacity: 1; transform: translateX(40px); }
}
.welcome-mark--track .welcome-dot:nth-of-type(1) {
  animation: welcome-mark-step-1 700ms var(--welcome-mark-ease) 250ms backwards;
}
.welcome-mark--track .welcome-dot:nth-of-type(2) {
  animation: welcome-mark-step-2 700ms var(--welcome-mark-ease) 400ms backwards;
}
.welcome-mark--track .welcome-dot:nth-of-type(3) {
  animation: welcome-mark-step-3 700ms var(--welcome-mark-ease) 550ms backwards;
}

/* ── Reduced-motion: snap to final state ─────────────────────────────── */
@media (prefers-reduced-motion: reduce) {
  .welcome-mark line,
  .welcome-mark .welcome-dot,
  .welcome-mark .welcome-ring {
    animation: none;
    stroke-dashoffset: 0;
    opacity: 1;
    transform: none;
  }
  /* Rings should be invisible at rest (they only exist as a transient
     pulse). Hide them entirely when motion is reduced. */
  .welcome-mark .welcome-ring {
    opacity: 0;
  }
  /* Dots in card 1, 3, 5 still need to land at their target X — apply
     the final translate without animation so they sit in alignment. */
  .welcome-mark--opener .welcome-dot { transform: translateX(30px); }
  .welcome-mark--align .welcome-dot:nth-of-type(1) { transform: translateX(20px); }
  .welcome-mark--align .welcome-dot:nth-of-type(2) { transform: translateX(-20px); }
  .welcome-mark--align .welcome-dot:nth-of-type(3) { transform: translateX(10px); }
  .welcome-mark--track .welcome-dot:nth-of-type(1) { transform: translateX(15px); }
  .welcome-mark--track .welcome-dot:nth-of-type(2) { transform: translateX(28px); }
  .welcome-mark--track .welcome-dot:nth-of-type(3) { transform: translateX(40px); }
}
```

- [ ] **Step 2: Verify CSS-only changes don't break the build**

Run:
```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

Both should pass — no Rust changed. (`fmt --check` is a no-op for CSS; `clippy` confirms no Rust regression.)

- [ ] **Step 3: Commit**

```bash
git add crates/intrada-web/input.css
git commit -m "style(onboarding): keyframes for welcome-mark per-card animations"
```

---

### Task 2: WelcomeMark component

**Files:**
- Create: `crates/intrada-web/src/components/welcome_mark.rs`
- Modify: `crates/intrada-web/src/components/mod.rs` (register module + re-export)

This task creates the new `WelcomeMark` component containing five private mark sub-components (one per card index). Each renders a small inline SVG with classes that match Task 1's CSS keyframes.

- [ ] **Step 1: Create `welcome_mark.rs`**

```rust
use leptos::prelude::*;

/// Animated SVG mark for a welcome carousel card.
///
/// Each card index gets a small mark composed of the brand's visual
/// vocabulary: horizontal lines (in `--color-text-faint`) plus dots
/// in violet (`--color-accent`) and amber (`--color-warm-accent`).
/// Animations express the per-card concept by what the dots do on
/// the lines (settle, snap, align, pulse, progress).
///
/// Indices map to:
/// - 0: opener — line draws, violet dot slides along it and settles
/// - 1: capture — three lines stack in, amber/violet/amber dots snap on
/// - 2: plan — three lines, scattered violet dots align in one column
/// - 3: practice — line + violet dot pulses with rings, amber enters
/// - 4: track — three lines, violet dots stagger rightward as progress
///
/// Out-of-range indices render an empty placeholder.
#[component]
pub fn WelcomeMark(card_index: usize) -> impl IntoView {
    match card_index {
        0 => view! { <MarkOpener /> }.into_any(),
        1 => view! { <MarkCapture /> }.into_any(),
        2 => view! { <MarkAlign /> }.into_any(),
        3 => view! { <MarkPractice /> }.into_any(),
        4 => view! { <MarkTrack /> }.into_any(),
        _ => view! { <span class="welcome-mark" aria-hidden="true"></span> }.into_any(),
    }
}

/// Card 1 (opener) — a single line draws in, then a violet dot
/// slides along it and settles past centre.
#[component]
fn MarkOpener() -> impl IntoView {
    view! {
        <svg
            class="welcome-mark welcome-mark--opener"
            viewBox="0 0 80 80"
            aria-hidden="true"
        >
            <line
                x1="15" y1="40" x2="65" y2="40"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <circle
                class="welcome-dot"
                cx="20" cy="40" r="5"
                fill="var(--color-accent)"
            />
        </svg>
    }
}

/// Card 2 (capture) — three lines, alternating amber/violet/amber
/// dots snapping onto each at varied positions.
#[component]
fn MarkCapture() -> impl IntoView {
    view! {
        <svg
            class="welcome-mark welcome-mark--capture"
            viewBox="0 0 80 80"
            aria-hidden="true"
        >
            <line
                x1="12" y1="22" x2="68" y2="22"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                x1="12" y1="40" x2="68" y2="40"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                x1="12" y1="58" x2="68" y2="58"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <circle
                class="welcome-dot"
                cx="50" cy="22" r="5"
                fill="var(--color-warm-accent)"
            />
            <circle
                class="welcome-dot"
                cx="32" cy="40" r="5"
                fill="var(--color-accent)"
            />
            <circle
                class="welcome-dot"
                cx="58" cy="58" r="5"
                fill="var(--color-warm-accent)"
            />
        </svg>
    }
}

/// Card 3 (plan) — three lines, three violet dots scattered at
/// different positions, sliding horizontally to align in one column.
#[component]
fn MarkAlign() -> impl IntoView {
    view! {
        <svg
            class="welcome-mark welcome-mark--align"
            viewBox="0 0 80 80"
            aria-hidden="true"
        >
            <line
                x1="12" y1="22" x2="68" y2="22"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                x1="12" y1="40" x2="68" y2="40"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                x1="12" y1="58" x2="68" y2="58"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            // Dots start at scattered cx values; CSS keyframes translate
            // each toward the same final column (≈ x=40):
            //   dot 1 starts at 20, translates +20 → ends at 40
            //   dot 2 starts at 60, translates -20 → ends at 40
            //   dot 3 starts at 30, translates +10 → ends at 40
            <circle
                class="welcome-dot"
                cx="20" cy="22" r="5"
                fill="var(--color-accent)"
            />
            <circle
                class="welcome-dot"
                cx="60" cy="40" r="5"
                fill="var(--color-accent)"
            />
            <circle
                class="welcome-dot"
                cx="30" cy="58" r="5"
                fill="var(--color-accent)"
            />
        </svg>
    }
}

/// Card 4 (practice) — single line with a pulsing violet dot at
/// centre, two concentric rings emanating outward, and an amber dot
/// sliding in from the right.
#[component]
fn MarkPractice() -> impl IntoView {
    view! {
        <svg
            class="welcome-mark welcome-mark--practice"
            viewBox="0 0 80 80"
            aria-hidden="true"
        >
            <line
                x1="15" y1="40" x2="65" y2="40"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            // Two transient rings — only visible during the pulse animation
            <circle
                class="welcome-ring"
                cx="40" cy="40" r="6"
                fill="none"
                stroke="var(--color-accent)"
                stroke-width="1.5"
            />
            <circle
                class="welcome-ring"
                cx="40" cy="40" r="6"
                fill="none"
                stroke="var(--color-accent)"
                stroke-width="1.5"
            />
            // Violet dot at centre — pulses
            <circle
                class="welcome-dot welcome-dot--violet"
                cx="40" cy="40" r="5"
                fill="var(--color-accent)"
            />
            // Amber dot — slides in from the right (CSS translates it from +20 → 0)
            <circle
                class="welcome-dot welcome-dot--amber"
                cx="58" cy="40" r="5"
                fill="var(--color-warm-accent)"
            />
        </svg>
    }
}

/// Card 5 (track) — three lines, three violet dots that slide
/// rightward in a staggered progression.
#[component]
fn MarkTrack() -> impl IntoView {
    view! {
        <svg
            class="welcome-mark welcome-mark--track"
            viewBox="0 0 80 80"
            aria-hidden="true"
        >
            <line
                x1="12" y1="22" x2="68" y2="22"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                x1="12" y1="40" x2="68" y2="40"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                x1="12" y1="58" x2="68" y2="58"
                stroke="var(--color-text-faint)"
                stroke-width="2"
                stroke-linecap="round"
            />
            // All three dots start at cx=18 (left of each line); CSS
            // keyframes translate them rightward by +15 / +28 / +40 so
            // the visual reads as left → middle → right progression.
            <circle
                class="welcome-dot"
                cx="18" cy="22" r="5"
                fill="var(--color-accent)"
            />
            <circle
                class="welcome-dot"
                cx="18" cy="40" r="5"
                fill="var(--color-accent)"
            />
            <circle
                class="welcome-dot"
                cx="18" cy="58" r="5"
                fill="var(--color-accent)"
            />
        </svg>
    }
}
```

- [ ] **Step 2: Register `welcome_mark` in `mod.rs`**

Open `crates/intrada-web/src/components/mod.rs`. In the alphabetical `pub mod` block (after `welcome_card` and before `welcome_carousel`), add:

```rust
pub mod welcome_mark;
```

In the alphabetical `pub use` block (after `welcome_card::WelcomeCard` and before `welcome_carousel::WelcomeCarousel`), add:

```rust
pub use welcome_mark::WelcomeMark;
```

- [ ] **Step 3: Verify build**

```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

Both must pass. The clippy check might flag the private `MarkOpener` / `MarkCapture` / `MarkAlign` / `MarkPractice` / `MarkTrack` as `dead_code` if it can't trace their references through the `WelcomeMark` match arm. They ARE used (each match arm calls one of them via the `view!` macro), so any warning is a Leptos-macro-expansion limitation, not a real dead path. If clippy still complains after the build is otherwise clean, add `#[allow(dead_code)]` to each Mark sub-component.

- [ ] **Step 4: Commit**

```bash
git add crates/intrada-web/src/components/welcome_mark.rs crates/intrada-web/src/components/mod.rs
git commit -m "feat(onboarding): add WelcomeMark — five animated SVG marks"
```

---

### Task 3: WelcomeCard prop refactor + design catalogue update

**Files:**
- Modify: `crates/intrada-web/src/components/welcome_card.rs` (full rewrite — small file)
- Modify: `crates/intrada-web/src/views/design_catalogue.rs` (update showcase)

This refactors `WelcomeCard` to the new prop shape and updates the design catalogue showcase in the same task because the catalogue's existing showcase will not compile against the new prop API. The changes must land together.

- [ ] **Step 1: Rewrite `welcome_card.rs`**

Replace the entire contents of `crates/intrada-web/src/components/welcome_card.rs` with:

```rust
use leptos::prelude::*;

/// A single card in the welcome carousel — layered typography with
/// an optional pillar label, anchor phrase, and continuation. The
/// animated mark and CTA button are passed in by the carousel as
/// children rendered above and below the text block respectively.
///
/// The opener (card 1) skips the label by passing `label = None`;
/// the final card uses `children` for the CTA.
#[component]
pub fn WelcomeCard(
    /// Optional pillar label rendered above the anchor (e.g. "CAPTURE").
    /// Pass `None` for cards that should skip the label.
    #[prop(optional, into)]
    label: Option<String>,
    /// Main anchor phrase rendered as a large serif heading.
    #[prop(into)]
    anchor: String,
    /// Optional softer continuation rendered below the anchor.
    #[prop(optional, into)]
    continuation: Option<String>,
    /// Optional CTA slot rendered below the continuation. Used on the
    /// final card for the "Get started →" button. The animated mark
    /// is rendered separately by the carousel above this card, not
    /// inside it.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center text-center px-card-comfortable max-w-md mx-auto">
            // Pillar label — small uppercase muted, skipped on opener.
            {label.map(|l| view! {
                <p class="field-label mb-3">{l}</p>
            })}

            // Anchor — large serif heading.
            <p class="page-title">{anchor}</p>

            // Continuation — softer body line, smaller and muted.
            {continuation.map(|c| view! {
                <p class="text-base text-muted mt-3 max-w-sm">{c}</p>
            })}

            // CTA slot — used on the final card.
            // mt-10 is intentional breathing room on the full-screen canvas
            // between continuation and CTA, no matching token in the spacing scale.
            {children.map(|c| view! {
                <div class="mt-10 w-full">{c()}</div>
            })}
        </div>
    }
}
```

- [ ] **Step 2: Update design catalogue showcase to use new prop API**

Open `crates/intrada-web/src/views/design_catalogue.rs`. Find the existing welcome-card section (around lines 1729-1760, the `<section id="welcome-card">` block).

Replace the two `<WelcomeCard copy="..." />` calls with new-API equivalents, and add a third showcase block for the WelcomeMark:

Locate this block:
```rust
                    <div class="mb-8 p-6 rounded-xl bg-surface-secondary">
                        <h3 class="card-title">"WelcomeCard (standalone)"</h3>
                        <div class="py-8">
                            <WelcomeCard copy="Knowing how to practise well is hard. I\u{2019}ve struggled with it. So I built this.".to_string() />
                        </div>
                    </div>

                    <div class="mb-8 p-6 rounded-xl bg-surface-secondary">
                        <h3 class="card-title">"WelcomeCard (with CTA — final card)"</h3>
                        <div class="py-8">
                            <WelcomeCard copy="Track your progress, achieve your goals.".to_string()>
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Hero
                                    full_width=true
                                >
                                    "Add your first piece \u{2192}"
                                </Button>
                            </WelcomeCard>
                        </div>
                    </div>
```

Replace with:
```rust
                    <div class="mb-8 p-6 rounded-xl bg-surface-secondary">
                        <h3 class="card-title">"WelcomeCard (opener — no label)"</h3>
                        <div class="py-8">
                            <WelcomeCard
                                anchor="Knowing how to practise well is hard.".to_string()
                                continuation="I\u{2019}ve struggled with it. So I built this.".to_string()
                            />
                        </div>
                    </div>

                    <div class="mb-8 p-6 rounded-xl bg-surface-secondary">
                        <h3 class="card-title">"WelcomeCard (with label + continuation)"</h3>
                        <div class="py-8">
                            <WelcomeCard
                                label="CAPTURE".to_string()
                                anchor="Build a library".to_string()
                                continuation="of pieces and exercises \u{2014} the things you\u{2019}re actually working on.".to_string()
                            />
                        </div>
                    </div>

                    <div class="mb-8 p-6 rounded-xl bg-surface-secondary">
                        <h3 class="card-title">"WelcomeCard (final — with CTA)"</h3>
                        <div class="py-8">
                            <WelcomeCard
                                label="TRACK".to_string()
                                anchor="Watch your progress".to_string()
                                continuation="Track every session, achieve your goals.".to_string()
                            >
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Hero
                                    full_width=true
                                >
                                    "Get started \u{2192}"
                                </Button>
                            </WelcomeCard>
                        </div>
                    </div>

                    <div class="mb-8 p-6 rounded-xl bg-surface-secondary">
                        <h3 class="card-title">"WelcomeMark — animated SVG per card"</h3>
                        <p class="text-sm text-muted mb-4">"Each card mounts a draw-in animation. Click each to remount and replay."</p>
                        <div class="grid grid-cols-5 gap-4 py-4">
                            <div class="flex flex-col items-center gap-2">
                                <WelcomeMark card_index=0 />
                                <span class="field-label">"Opener"</span>
                            </div>
                            <div class="flex flex-col items-center gap-2">
                                <WelcomeMark card_index=1 />
                                <span class="field-label">"Capture"</span>
                            </div>
                            <div class="flex flex-col items-center gap-2">
                                <WelcomeMark card_index=2 />
                                <span class="field-label">"Plan"</span>
                            </div>
                            <div class="flex flex-col items-center gap-2">
                                <WelcomeMark card_index=3 />
                                <span class="field-label">"Practice"</span>
                            </div>
                            <div class="flex flex-col items-center gap-2">
                                <WelcomeMark card_index=4 />
                                <span class="field-label">"Track"</span>
                            </div>
                        </div>
                    </div>
```

- [ ] **Step 3: Add `WelcomeMark` to the design catalogue's `use` block**

Find the existing `use crate::components::{ ... WelcomeCard, ...}` block at the top of `design_catalogue.rs` and add `WelcomeMark` alphabetically:

The existing line in the import (alphabetical, ends with `WelcomeCard`) — change to:
```rust
    StatCard, StatTone, StatusDot, StatusDotState, SwipeActions, TagInput, TempoProgressChart,
    TextArea, TextField, TransitionPrompt, TypeBadge, TypeTabs, WeekStrip, WelcomeCard,
    WelcomeMark,
```

- [ ] **Step 4: Verify build**

```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

At this point, **`welcome_carousel.rs` will fail to compile** because it still uses the old `<WelcomeCard copy=... />` API. That's expected — Task 4 fixes it. Do NOT commit yet.

Verify that the only compile error is in `welcome_carousel.rs`:
```bash
cargo check -p intrada-web 2>&1 | grep -E "^(error|warning)" | head -20
```
You should see errors that all point at `welcome_carousel.rs` lines that pass `copy=...`.

- [ ] **Step 5: Don't commit yet — proceed to Task 4**

The `WelcomeCard` API change breaks `WelcomeCarousel`. Both must land in the same commit (or sequential commits with the second following immediately) to keep the build green between commits. Continue to Task 4 with these changes uncommitted.

---

### Task 4: WelcomeCarousel — restructure card data + integrate new API + CTA changes

**Files:**
- Modify: `crates/intrada-web/src/components/welcome_carousel.rs`

This is the integration task. It replaces the flat `CARDS: [&str; 5]` with structured per-card data, updates the render block to use the new `WelcomeCard` API, mounts `WelcomeMark` above each card, changes the CTA destination from `/library/new` → `/`, and updates the CTA label from "Add your first piece →" → "Get started →".

- [ ] **Step 1: Replace the flat `CARDS` constant with structured `CardContent`**

Open `crates/intrada-web/src/components/welcome_carousel.rs`. Find the existing card content block (around line 18-26):

```rust
/// Card copy — these are the spec, not placeholders.
/// Edits should be deliberate (PR review, not drift).
const CARDS: [&str; CARD_COUNT] = [
    "Knowing how to practise well is hard. I\u{2019}ve struggled with it. So I built this.",
    "Build a library of pieces and exercises \u{2014} the things you\u{2019}re actually working on.",
    "Plan each session with intention. Decide where the effort goes before you pick up the instrument.",
    "Run focused, timed sessions with real-time reflection \u{2014} score what happened while it\u{2019}s still fresh.",
    "Track your progress, achieve your goals.",
];
```

Replace with:

```rust
/// Per-card content: optional pillar label, anchor phrase, continuation.
/// Card 1 (opener) deliberately omits the label.
///
/// These lines are the spec — not placeholders. Edits should be deliberate
/// (PR review, not drift). See specs/onboarding-welcome.md.
struct CardContent {
    label: Option<&'static str>,
    anchor: &'static str,
    continuation: Option<&'static str>,
}

const CARDS: [CardContent; CARD_COUNT] = [
    CardContent {
        label: None,
        anchor: "Knowing how to practise well is hard.",
        continuation: Some("I\u{2019}ve struggled with it. So I built this."),
    },
    CardContent {
        label: Some("CAPTURE"),
        anchor: "Build a library",
        continuation: Some(
            "of pieces and exercises \u{2014} the things you\u{2019}re actually working on.",
        ),
    },
    CardContent {
        label: Some("PLAN"),
        anchor: "Practise with intention",
        continuation: Some(
            "Plan each session. Decide where the effort goes before you pick up the instrument.",
        ),
    },
    CardContent {
        label: Some("PRACTICE"),
        anchor: "Focus, reflect, repeat",
        continuation: Some(
            "Run timed sessions with real-time reflection \u{2014} score what happened while it\u{2019}s still fresh.",
        ),
    },
    CardContent {
        label: Some("TRACK"),
        anchor: "Watch your progress",
        continuation: Some("Track every session, achieve your goals."),
    },
];
```

- [ ] **Step 2: Add `WelcomeMark` to the imports**

Find the existing `use crate::components::{ ... }` line at the top of `welcome_carousel.rs`:

```rust
use crate::components::{Button, ButtonSize, ButtonVariant, WelcomeCard};
```

Replace with:

```rust
use crate::components::{Button, ButtonSize, ButtonVariant, WelcomeCard, WelcomeMark};
```

- [ ] **Step 3: Change CTA destination from `/library/new` to `/`**

Find the `dismiss_cta` closure (around line 107-116). Update the navigate target:

```rust
    let dismiss_cta = {
        let navigate = navigate.clone();
        move || {
            // No explicit haptic here — Button::Primary fires haptic_light
            // on click internally. Adding another would double-tap on iOS.
            mark_welcome_seen();
            show.set(false);
            navigate("/", Default::default());
        }
    };
```

Note: only the navigate target changes (`"/library/new"` → `"/"`). The comment block above stays the same since it still applies.

- [ ] **Step 4: Replace the render block with new-API WelcomeCard + WelcomeMark**

Find the existing card content render block (around lines 250-275, inside the `<div class=card_content_class>`):

```rust
            // Card content — transitions on index change
            <div class=card_content_class>
                {move || {
                    let idx = card_index.get();
                    let copy = CARDS[idx].to_string();
                    let is_last = idx == CARD_COUNT - 1;

                    if is_last {
                        view! {
                            <WelcomeCard copy=copy>
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Hero
                                    full_width=true
                                    on_click=on_cta_cb
                                >
                                    "Add your first piece \u{2192}"
                                </Button>
                            </WelcomeCard>
                        }
                        .into_any()
                    } else {
                        view! {
                            <WelcomeCard copy=copy />
                        }
                        .into_any()
                    }
                }}
            </div>
```

Replace with:

```rust
            // Card content — transitions on index change.
            // The mark and the card share a vertical stack; both re-mount
            // when card_index changes so the SVG draw-in animation replays.
            <div class=card_content_class>
                {move || {
                    let idx = card_index.get();
                    let card = &CARDS[idx];
                    let is_last = idx == CARD_COUNT - 1;

                    if is_last {
                        view! {
                            <div class="flex flex-col items-center">
                                <div class="mb-8">
                                    <WelcomeMark card_index=idx />
                                </div>
                                <WelcomeCard
                                    label=card.label.map(str::to_string)
                                    anchor=card.anchor.to_string()
                                    continuation=card.continuation.map(str::to_string)
                                >
                                    <Button
                                        variant=ButtonVariant::Primary
                                        size=ButtonSize::Hero
                                        full_width=true
                                        on_click=on_cta_cb
                                    >
                                        "Get started \u{2192}"
                                    </Button>
                                </WelcomeCard>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="flex flex-col items-center">
                                <div class="mb-8">
                                    <WelcomeMark card_index=idx />
                                </div>
                                <WelcomeCard
                                    label=card.label.map(str::to_string)
                                    anchor=card.anchor.to_string()
                                    continuation=card.continuation.map(str::to_string)
                                />
                            </div>
                        }
                        .into_any()
                    }
                }}
            </div>
```

Two changes from the previous render block beyond the API rewrite:
1. CTA label `"Add your first piece \u{2192}"` → `"Get started \u{2192}"`
2. Each branch now wraps `WelcomeMark` + `WelcomeCard` in a vertical stack with `mb-8` between them

- [ ] **Step 5: Verify build**

```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

Both should pass — at this point all four changed files (input.css, welcome_mark.rs, welcome_card.rs, welcome_carousel.rs) compose together correctly.

If clippy flags `CardContent` fields as `dead_code`, that's a sign the const array isn't being read at compile time. The field reads happen at runtime through `&CARDS[idx]`, so they ARE used. If clippy still complains in a way you can't satisfy, add `#[allow(dead_code)]` to the struct.

- [ ] **Step 6: Manual smoke test (optional but recommended)**

```bash
trunk serve --port 8080
```

Open `http://localhost:8080`, open DevTools, clear `localStorage`, reload, sign in. Expected:
1. Welcome carousel appears.
2. Card 1 shows the line-drawing animation; copy reads "Knowing how to practise well is hard." with continuation "I've struggled with it. So I built this." — no label.
3. Tap to advance: card 2 shows dots-popping animation; "CAPTURE" label, "Build a library" anchor, continuation about pieces/exercises.
4. Cards 3-4 advance similarly.
5. Card 5: "TRACK" label, "Watch your progress", "Track every session, achieve your goals.", and a "Get started →" button.
6. Click "Get started →" → routes to `/` (NOT `/library/new`).

Then clear localStorage, reload, and toggle macOS Settings → Accessibility → Display → Reduce Motion ON. Re-test: carousel should still work, but animations should snap to final state instantly with no fade or draw-in.

- [ ] **Step 7: Commit Tasks 3 + 4 together**

```bash
git add crates/intrada-web/src/components/welcome_card.rs \
        crates/intrada-web/src/components/welcome_carousel.rs \
        crates/intrada-web/src/views/design_catalogue.rs
git commit -m "feat(onboarding): layered card typography + animated marks + library CTA

WelcomeCard prop API: copy: String → label: Option<String> +
anchor: String + continuation: Option<String>. Carousel now mounts
WelcomeMark above each card and uses the structured CardContent
array for per-card label/anchor/continuation. CTA destination
changes from /library/new to /; CTA label changes from \"Add your
first piece →\" to \"Get started →\". Design catalogue showcase
updated to new prop shape and gains a WelcomeMark grid showing all
five animations.

See specs/onboarding-welcome.md v2."
```

---

### Task 5: E2E test updates

**Files:**
- Modify: `e2e/tests/welcome.spec.ts`

The first welcome test asserts that the CTA navigates to `/library/new`. With the new destination, the assertion needs to update — and the button-name selector needs to match "Get started" instead of "Add your first piece".

- [ ] **Step 1: Update the CTA test**

Open `e2e/tests/welcome.spec.ts`. Find the first test (`"shows welcome on first visit, tap through to CTA"`) and update the final block. Look for:

```typescript
    // Use regex to match the button regardless of the → Unicode arrow suffix
    const cta = page.getByRole("button", { name: /Add your first piece/ });
    await expect(cta).toBeVisible();

    // Click CTA — should navigate to /library/new
    await cta.click();
    await expect(carousel).not.toBeVisible();
    await expect(page).toHaveURL(/\/library\/new/);
  });
```

Replace with:

```typescript
    // Use regex to match the button regardless of the → Unicode arrow suffix
    const cta = page.getByRole("button", { name: /Get started/ });
    await expect(cta).toBeVisible();

    // Click CTA — should navigate to / (the Library home)
    await cta.click();
    await expect(carousel).not.toBeVisible();
    await expect(page).toHaveURL(/\/$/);
  });
```

Two changes:
1. Button-name regex `/Add your first piece/` → `/Get started/`
2. URL regex `/\/library\/new/` → `/\/$/` (root path — same regex used by the Skip test below)

- [ ] **Step 2: Run the welcome E2E suite**

```bash
cd e2e && npx playwright test tests/welcome.spec.ts --reporter=line
```

Expected: 3 tests pass.

If the CTA test fails, common causes:
- Button text rendered with extra whitespace — try `name: /Get started/` (already a regex).
- URL regex too strict — `/\/$/` matches a trailing slash. If Playwright reports the URL as `http://localhost:8080/` and the regex doesn't match, expand to `/\/$|^\//`.
- Carousel doesn't dismiss — verify the `show.set(false)` and `mark_welcome_seen()` are being called (this would fail in app code, not test code, but worth checking).

- [ ] **Step 3: Run a sample of the broader E2E suite to confirm no regression**

```bash
cd e2e && npx playwright test tests/add-item.spec.ts tests/detail.spec.ts tests/navigation.spec.ts --reporter=line
```

Expected: all pass. The shared fixture's `setupWelcomeMock` already primes the welcome-seen flag for these tests, so they should be unaffected.

- [ ] **Step 4: Commit**

```bash
git add e2e/tests/welcome.spec.ts
git commit -m "test(onboarding): welcome CTA now navigates to / not /library/new"
```

---

## Post-implementation checklist

After all 5 tasks are complete:

- [ ] Run full test suite: `cargo test && cd e2e && npx playwright test`
- [ ] Run formatting + lint: `cargo fmt --check && cargo clippy -p intrada-web -- -D warnings`
- [ ] Manual on iOS simulator (`just ios-dev`) — confirm haptics still fire on advance + Skip + CTA, the SVG mark animations play smoothly inside WKWebView, and safe-area insets render correctly
- [ ] Verify design catalogue at `http://localhost:8080/design` (debug build): WelcomeCard showcase shows all three variants (opener / with-label / with-CTA) and the WelcomeMark grid plays animations on each entry
- [ ] Verify reduced-motion behaviour: with macOS reduce-motion ON, the carousel still works but all marks snap to final state and card transitions are instant

## Notes for the implementer

**On `transform-box: fill-box`:** SVG elements default to using the SVG viewport for transforms, which makes per-element scaling/rotation behave unexpectedly. `transform-box: fill-box` makes the transform origin relative to the element's own bounding box. This is required for the dot-pop, ring-pulse, and bar-rise animations to scale around their own centres, not the SVG origin.

**On animation replay:** The animations rely on the SVG element being a fresh DOM node when card_index changes. Leptos's `view! { ... }` macro inside the `move ||` reactive block typically creates new nodes when the rendered structure changes meaningfully — and since `WelcomeMark`'s output structure changes per index (different SVG paths), this should work. If you observe the animation NOT replaying when a card is revisited (e.g., swipe forward then back to the same card), the fix is to add a unique `data-key` attribute that changes per visit (e.g., a `RwSignal<u32>` counter incremented on every transition_to call). Test in `trunk serve` before assuming the simple approach is broken.

**On the order of Tasks 3 & 4:** They must be committed together OR sequentially with no other commits in between. Task 3 alone leaves `welcome_carousel.rs` uncompilable; Task 4 alone fails because the new `WelcomeCard` API doesn't exist yet. The plan structures them as two tasks for review clarity but the commits should land together. Task 4 Step 7 commits ALL of Tasks 3 + 4 in one commit.

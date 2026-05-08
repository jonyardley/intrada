# Onboarding Welcome Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a five-card welcome carousel shown once to new users after sign-in, before they reach the empty Library.

**Architecture:** Pure web-shell change — no Crux core, API, or DB modifications. A `WelcomeCarousel` component renders five typographic cards as a full-screen overlay inside `AuthenticatedApp`. Visibility is gated on a `RwSignal<bool>` driven from a single `localStorage` flag (`intrada:welcome-seen`). Touch swipe and tap navigation advance through cards. The final card's CTA routes to `/library/new`; Skip routes to `/`.

**Tech Stack:** Leptos 0.8 (CSR), Tailwind CSS v4, `web_sys` (localStorage, touch events), `gloo-timers` (transition delay), `wasm-bindgen` (haptics bridge)

**Spec:** [`specs/onboarding-welcome.md`](../../../specs/onboarding-welcome.md)

---

## File map

| Action | File | Responsibility |
|--------|------|---------------|
| Create | `crates/intrada-web/src/components/welcome_card.rs` | Single card visual: copy text + optional CTA children |
| Create | `crates/intrada-web/src/components/welcome_carousel.rs` | Overlay container: card state machine, progress dots, skip, touch gestures, localStorage, navigation |
| Modify | `crates/intrada-web/src/components/mod.rs` | Register + re-export the two new components |
| Modify | `crates/intrada-web/input.css` | Welcome-specific CSS: card transition animation, progress dot styles |
| Modify | `crates/intrada-web/src/app.rs` | Mount carousel in `AuthenticatedApp` as sibling of `<main>` |
| Modify | `crates/intrada-web/src/views/design_catalogue.rs` | Debug-only showcase entry with localStorage reset button |
| Create | `e2e/tests/welcome.spec.ts` | Playwright E2E: 3 scenarios (full flow, skip, primed localStorage) |

---

### Task 1: WelcomeCard component

**Files:**
- Create: `crates/intrada-web/src/components/welcome_card.rs`
- Modify: `crates/intrada-web/src/components/mod.rs`

- [ ] **Step 1: Create `welcome_card.rs`**

```rust
use leptos::prelude::*;

/// A single card in the welcome carousel — typographic copy with optional
/// CTA slot for the final card.
#[component]
pub fn WelcomeCard(
    /// The card copy text.
    #[prop(into)]
    copy: String,
    /// Optional CTA slot (used on the final card for the "Add your first piece" button).
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center text-center px-6 max-w-md mx-auto">
            <p class="font-heading text-[1.75rem] leading-9 font-semibold text-primary tracking-tight">
                {copy}
            </p>
            {children.map(|c| view! {
                <div class="mt-10 w-full">
                    {c()}
                </div>
            })}
        </div>
    }
}
```

- [ ] **Step 2: Register in `mod.rs`**

Add these two lines to `crates/intrada-web/src/components/mod.rs`:

In the `pub mod` section (alphabetical — after `week_strip`):
```rust
pub mod welcome_card;
pub mod welcome_carousel;
```

In the `pub use` section (after `week_strip`):
```rust
pub use welcome_card::WelcomeCard;
pub use welcome_carousel::WelcomeCarousel;
```

Note: `welcome_carousel` module is registered now but the file won't exist until Task 2. If the compiler blocks, create an empty placeholder file at `crates/intrada-web/src/components/welcome_carousel.rs` with:
```rust
// Placeholder — implemented in Task 2.
```

- [ ] **Step 3: Commit**

```bash
cargo fmt --check
git add crates/intrada-web/src/components/welcome_card.rs crates/intrada-web/src/components/mod.rs
git commit -m "feat(onboarding): add WelcomeCard component"
```

---

### Task 2: Welcome CSS utilities

**Files:**
- Modify: `crates/intrada-web/input.css` (append at end, before the closing line)

- [ ] **Step 1: Add welcome carousel styles to `input.css`**

Append the following block at the end of `crates/intrada-web/input.css` (after line 2182):

```css
/* ═══════════════════════════════════════════════════════════════════════
   Welcome carousel — first-run onboarding overlay
   ═══════════════════════════════════════════════════════════════════════ */

/* Card content transition — fade-and-rise when the card changes.
   Respects prefers-reduced-motion. */
.welcome-card-content {
  @media (prefers-reduced-motion: no-preference) {
    transition: opacity 200ms ease-out, transform 200ms ease-out;
  }
}

.welcome-card-content--entering {
  opacity: 0;
  transform: translateY(12px);
}

.welcome-card-content--visible {
  opacity: 1;
  transform: translateY(0);
}

/* Progress dots row */
.welcome-dots {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

/* Individual progress dot — a small tappable circle. The min 44px touch
   target is on the button element; the visible dot is the inner span. */
.welcome-dot {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.welcome-dot-inner {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-text-faint);
  @media (prefers-reduced-motion: no-preference) {
    transition: background 200ms ease;
  }
}

.welcome-dot--active .welcome-dot-inner {
  background: var(--color-text-primary);
}
```

- [ ] **Step 2: Commit**

```bash
cargo fmt --check
git add crates/intrada-web/input.css
git commit -m "style(onboarding): add welcome carousel CSS utilities"
```

---

### Task 3: WelcomeCarousel component

**Files:**
- Create: `crates/intrada-web/src/components/welcome_carousel.rs`

This is the largest task — the full carousel with state management, rendering, localStorage, touch gestures, transitions, navigation, and haptics.

- [ ] **Step 1: Create `welcome_carousel.rs` with full implementation**

```rust
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::prelude::*;
use web_sys::TouchEvent;

use crate::components::{Button, ButtonSize, ButtonVariant, WelcomeCard};
use intrada_web::haptics;

/// localStorage key for the "welcome seen" flag.
pub const WELCOME_SEEN_KEY: &str = "intrada:welcome-seen";

const CARD_COUNT: usize = 5;

/// Horizontal distance (px) to recognise a swipe gesture.
const SWIPE_THRESHOLD: f64 = 30.0;

/// Card copy — these are the spec, not placeholders.
/// Edits should be deliberate (PR review, not drift).
const CARDS: [&str; CARD_COUNT] = [
    "Knowing how to practise well is hard. I\u{2019}ve struggled with it. So I built this.",
    "Build a library of pieces and exercises \u{2014} the things you\u{2019}re actually working on.",
    "Plan each session with intention. Decide where the effort goes before you pick up the instrument.",
    "Run focused, timed sessions with real-time reflection \u{2014} score what happened while it\u{2019}s still fresh.",
    "Track your progress, achieve your goals.",
];

/// Check whether the welcome has already been seen on this device.
pub fn welcome_already_seen() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item(WELCOME_SEEN_KEY).ok())
        .flatten()
        .is_some()
}

/// Mark the welcome as seen in localStorage.
fn mark_welcome_seen() {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item(WELCOME_SEEN_KEY, "1");
    }
}

/// Clear the welcome-seen flag (dev use only).
#[cfg(debug_assertions)]
pub fn reset_welcome_seen() {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item(WELCOME_SEEN_KEY);
    }
}

/// Full-screen welcome carousel overlay — shown once to new users.
///
/// Renders five typographic cards with progress dots, Skip link, and a
/// primary CTA on the final card. Visibility is controlled by the parent
/// via the `show` signal — the carousel sets it to `false` when dismissed.
#[component]
pub fn WelcomeCarousel(
    /// The parent's visibility signal — set to `false` when the carousel
    /// is dismissed (Skip or CTA).
    show: RwSignal<bool>,
) -> impl IntoView {
    let card_index = RwSignal::new(0usize);
    let transitioning = RwSignal::new(false);
    let navigate = use_navigate();
    let carousel_ref = NodeRef::<leptos::html::Div>::new();

    // ── Card transition helpers ────────────────────────────────────────

    let transition_to = move |target: usize| {
        // Fade out current card
        transitioning.set(true);
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(50).await;
            card_index.set(target);
            transitioning.set(false);
        });
    };

    // ── Navigation actions ─────────────────────────────────────────────

    let advance = move || {
        let current = card_index.get_untracked();
        if current < CARD_COUNT - 1 {
            haptics::haptic_selection();
            transition_to(current + 1);
        }
    };

    let go_back = move || {
        let current = card_index.get_untracked();
        if current > 0 {
            haptics::haptic_selection();
            transition_to(current - 1);
        }
    };

    let dismiss_skip = {
        let navigate = navigate.clone();
        move || {
            haptics::haptic_light();
            mark_welcome_seen();
            show.set(false);
            navigate("/", Default::default());
        }
    };

    let dismiss_cta = {
        let navigate = navigate.clone();
        move || {
            haptics::haptic_success();
            mark_welcome_seen();
            show.set(false);
            navigate("/library/new", Default::default());
        }
    };

    // ── Touch/swipe handling ───────────────────────────────────────────
    // Follows the pattern from swipe_actions.rs — attach raw listeners
    // via Effect + Closure so we can call preventDefault on horizontal
    // swipes without blocking vertical scroll.

    let touch_start_x = RwSignal::new(None::<f64>);
    let touch_start_y = RwSignal::new(None::<f64>);

    Effect::new(move || {
        let Some(el) = carousel_ref.get() else {
            return;
        };

        let touchstart: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            if let Some(touch) = ev.touches().get(0) {
                touch_start_x.set(Some(touch.client_x() as f64));
                touch_start_y.set(Some(touch.client_y() as f64));
            }
        });

        let touchend: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            let Some(start_x) = touch_start_x.get_untracked() else {
                return;
            };
            let Some(start_y) = touch_start_y.get_untracked() else {
                return;
            };

            if let Some(touch) = ev.changed_touches().get(0) {
                let dx = touch.client_x() as f64 - start_x;
                let dy = touch.client_y() as f64 - start_y;

                // Only recognise horizontal swipes
                if dx.abs() > SWIPE_THRESHOLD && dx.abs() > dy.abs() {
                    if dx < 0.0 {
                        // Swipe left → advance
                        let current = card_index.get_untracked();
                        if current < CARD_COUNT - 1 {
                            haptics::haptic_selection();
                            transitioning.set(true);
                            // Use gloo Timeout (not leptos::task::spawn_local)
                            // because this closure runs outside the Leptos
                            // owner context — raw addEventListener callbacks
                            // don't have one. See CLAUDE.md gotcha.
                            gloo_timers::callback::Timeout::new(50, move || {
                                card_index.set(current + 1);
                                transitioning.set(false);
                            }).forget();
                        }
                    } else {
                        // Swipe right → go back
                        let current = card_index.get_untracked();
                        if current > 0 {
                            haptics::haptic_selection();
                            transitioning.set(true);
                            gloo_timers::callback::Timeout::new(50, move || {
                                card_index.set(current - 1);
                                transitioning.set(false);
                            }).forget();
                        }
                    }
                }
            }

            touch_start_x.set(None);
            touch_start_y.set(None);
        });

        let el_target: &web_sys::EventTarget = el.as_ref();
        let _ = el_target.add_event_listener_with_callback(
            "touchstart",
            touchstart.as_ref().unchecked_ref(),
        );
        let _ = el_target.add_event_listener_with_callback(
            "touchend",
            touchend.as_ref().unchecked_ref(),
        );

        // Leak intentionally — carousel lives for app lifetime or until
        // dismissed, and cleanup isn't worth the complexity.
        touchstart.forget();
        touchend.forget();
    });

    // ── Callbacks for Leptos event handlers ─────────────────────────────

    let on_skip = {
        let dismiss_skip = dismiss_skip.clone();
        Callback::new(move |_: ev::MouseEvent| {
            dismiss_skip();
        })
    };

    let on_cta = {
        let dismiss_cta = dismiss_cta.clone();
        Callback::new(move |_: ev::MouseEvent| {
            dismiss_cta();
        })
    };

    let on_card_click = Callback::new(move |_: ev::MouseEvent| {
        // Tap-to-advance on desktop (not on the final card — that has
        // the CTA button which handles its own click).
        let current = card_index.get_untracked();
        if current < CARD_COUNT - 1 {
            advance();
        }
    });

    // ── View ───────────────────────────────────────────────────────────

    let card_content_class = move || {
        if transitioning.get() {
            "welcome-card-content welcome-card-content--entering"
        } else {
            "welcome-card-content welcome-card-content--visible"
        }
    };

    view! {
        <div
            node_ref=carousel_ref
            class="fixed inset-0 z-10 flex flex-col items-center justify-center bg-linear-to-b from-[var(--color-bg-gradient-top)] to-[var(--color-bg-gradient-bottom)]"
            style="padding-top: env(safe-area-inset-top, 0); padding-bottom: env(safe-area-inset-bottom, 0);"
            on:click=on_card_click
            role="region"
            aria-label="Welcome"
        >
            // Skip link — top right
            <button
                class="absolute top-4 right-4 px-2 py-1 text-sm text-muted bg-transparent border-none cursor-pointer"
                style="top: calc(env(safe-area-inset-top, 16px) + 0.5rem);"
                on:click=move |ev: ev::MouseEvent| {
                    ev.stop_propagation(); // Don't trigger card-click advance
                    on_skip.run(ev);
                }
            >
                "Skip"
            </button>

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
                                    on_click=on_cta
                                >
                                    "Add your first piece \u{2192}"
                                </Button>
                            </WelcomeCard>
                        }.into_any()
                    } else {
                        view! {
                            <WelcomeCard copy=copy />
                        }.into_any()
                    }
                }}
            </div>

            // Progress dots
            <div
                class="welcome-dots absolute bottom-8"
                style="bottom: calc(env(safe-area-inset-bottom, 32px) + 1rem);"
            >
                {(0..CARD_COUNT).map(|i| {
                    let dot_class = move || {
                        if card_index.get() == i {
                            "welcome-dot welcome-dot--active"
                        } else {
                            "welcome-dot"
                        }
                    };
                    view! {
                        <button
                            class=dot_class
                            on:click=move |ev: ev::MouseEvent| {
                                ev.stop_propagation();
                                haptics::haptic_selection();
                                transition_to(i);
                            }
                            aria-label=format!("Card {}", i + 1)
                        >
                            <span class="welcome-dot-inner"></span>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
```

- [ ] **Step 2: Verify the `welcome_carousel` module is registered in `mod.rs`**

This was done in Task 1 Step 2. If you created a placeholder file, replace its contents now.

- [ ] **Step 3: Run cargo fmt + clippy**

```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

Fix any issues that arise. Common ones:
- Unused imports (remove them)
- `clone()` on Copy types (remove the clone)
- Missing `Send` bounds (shouldn't apply since this is WASM/CSR)

- [ ] **Step 4: Commit**

```bash
git add crates/intrada-web/src/components/welcome_carousel.rs
git commit -m "feat(onboarding): add WelcomeCarousel with swipe, dots, transitions"
```

---

### Task 4: Mount carousel in AuthenticatedApp

**Files:**
- Modify: `crates/intrada-web/src/app.rs:125-160`

- [ ] **Step 1: Add import for `WelcomeCarousel` and `welcome_already_seen`**

At the top of `app.rs`, add to the existing `use crate::components::{...}` import (line 14-16):

```rust
use crate::components::WelcomeCarousel;
```

And add a new import for the localStorage check:

```rust
use crate::components::welcome_carousel::welcome_already_seen;
```

- [ ] **Step 2: Add welcome signal + localStorage check in `AuthenticatedApp`**

Inside the `AuthenticatedApp` component function (after the `focus_mode` signal at line 131, before `provide_context` calls):

```rust
    // Welcome carousel — show for first-time users (localStorage gate)
    let show_welcome = RwSignal::new(!welcome_already_seen());
```

- [ ] **Step 3: Render `WelcomeCarousel` as a sibling of `<main>`**

Inside the `view!` block of `AuthenticatedApp`, immediately after the opening `<div class="relative z-0 min-h-screen text-primary">` (line 155) and before the Header `<Show>` block (line 157), add:

```rust
            // Welcome carousel overlay — shown once for first-time users.
            // Positioned fixed z-10, sits above the routed content. The
            // app underneath continues to mount and fetch data.
            <Show when=move || show_welcome.get()>
                <WelcomeCarousel show=show_welcome />
            </Show>
```

- [ ] **Step 4: Run cargo fmt + clippy**

```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

- [ ] **Step 5: Manual smoke test**

```bash
trunk serve
```

Open `http://localhost:8080` in a browser. Expected:
1. Sign in (or skip auth if `CLERK_PUBLISHABLE_KEY` is unset).
2. Welcome carousel appears as a full-screen overlay.
3. Tap/click advances through 5 cards.
4. Final card shows "Add your first piece →" button.
5. Click the CTA → routes to `/library/new`.
6. Reload → no carousel (localStorage flag is set).

Clear `localStorage.removeItem('intrada:welcome-seen')` in the browser console and reload to see it again.

- [ ] **Step 6: Commit**

```bash
git add crates/intrada-web/src/app.rs
git commit -m "feat(onboarding): mount WelcomeCarousel in AuthenticatedApp"
```

---

### Task 5: Design catalogue entry (dev affordance)

**Files:**
- Modify: `crates/intrada-web/src/views/design_catalogue.rs`

- [ ] **Step 1: Add import**

At the top of `design_catalogue.rs`, add `WelcomeCarousel` and `WelcomeCard` to the existing `use crate::components::{...}` import block (lines 9-19):

```rust
    WelcomeCard, WelcomeCarousel,
```

Also add the reset function import:

```rust
use crate::components::welcome_carousel::reset_welcome_seen;
```

- [ ] **Step 2: Add catalogue section**

Find the end of the last component showcase section in `design_catalogue.rs` and add a new section. Insert before the closing `</div>` of the catalogue:

```rust
                // ── Welcome Carousel ──────────────────────────────────────
                <section class="mb-16">
                    <h2 class="section-title">"Welcome Carousel"</h2>
                    <p class="text-sm text-muted mb-4">
                        "First-run onboarding overlay. Five typographic cards with progress dots, Skip, and CTA."
                    </p>

                    // Individual WelcomeCard showcase
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

                    // Reset button — clears localStorage so the carousel re-appears on reload
                    <div class="flex gap-4">
                        <Button
                            variant=ButtonVariant::Secondary
                            on_click=Callback::new(move |_: ev::MouseEvent| {
                                reset_welcome_seen();
                                // Reload the page so the carousel re-appears
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().reload();
                                }
                            })
                        >
                            "Reset welcome flag & reload"
                        </Button>
                    </div>
                </section>
```

- [ ] **Step 3: Add `web_sys` import if not already present**

Check the imports at the top of `design_catalogue.rs`. If `web_sys` is not imported, add:

```rust
use web_sys;
```

Note: it may already be imported via `wasm_bindgen::JsCast` (line 21). If `web_sys::window()` compiles, no extra import needed.

- [ ] **Step 4: Run cargo fmt + clippy**

```bash
cargo fmt --check
cargo clippy -p intrada-web -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add crates/intrada-web/src/views/design_catalogue.rs
git commit -m "feat(onboarding): add WelcomeCarousel to design catalogue with reset"
```

---

### Task 6: E2E tests

**Files:**
- Create: `e2e/tests/welcome.spec.ts`

- [ ] **Step 1: Create `welcome.spec.ts`**

The E2E fixture (`e2e/fixtures/api-mock.ts`) already mocks Clerk auth and API calls. Tests need to manage `localStorage` to control the welcome flag.

```typescript
import { test, expect } from "../fixtures/api-mock";

test.describe("welcome carousel", () => {
  test("shows welcome on first visit, tap through to CTA", async ({ page }) => {
    // Ensure localStorage is clean (no welcome-seen flag)
    await page.goto("/");

    // Carousel should be visible
    const carousel = page.getByRole("region", { name: "Welcome" });
    await expect(carousel).toBeVisible();

    // Card 1 content
    await expect(
      page.getByText("Knowing how to practise well is hard")
    ).toBeVisible();

    // Tap to advance through cards 2-4
    await carousel.click();
    await expect(
      page.getByText("Build a library of pieces and exercises")
    ).toBeVisible();

    await carousel.click();
    await expect(
      page.getByText("Plan each session with intention")
    ).toBeVisible();

    await carousel.click();
    await expect(
      page.getByText("Run focused, timed sessions")
    ).toBeVisible();

    await carousel.click();
    // Card 5 — final card with CTA
    await expect(
      page.getByText("Track your progress, achieve your goals")
    ).toBeVisible();

    const cta = page.getByRole("button", { name: "Add your first piece" });
    await expect(cta).toBeVisible();

    // Click CTA — should navigate to /library/new
    await cta.click();
    await expect(carousel).not.toBeVisible();
    await expect(page).toHaveURL(/\/library\/new/);
  });

  test("skip dismisses carousel and lands on library", async ({ page }) => {
    await page.goto("/");

    const carousel = page.getByRole("region", { name: "Welcome" });
    await expect(carousel).toBeVisible();

    // Click Skip
    await page.getByRole("button", { name: "Skip" }).click();

    await expect(carousel).not.toBeVisible();
    // Should be on the library page (root)
    await expect(page).toHaveURL(/\/$/);

    // Reload — carousel should NOT reappear (localStorage flag set)
    await page.reload();
    await expect(
      page.getByRole("region", { name: "Welcome" })
    ).not.toBeVisible();
  });

  test("does not show welcome when localStorage flag is set", async ({
    page,
  }) => {
    // Prime localStorage before navigating
    await page.addInitScript(() => {
      localStorage.setItem("intrada:welcome-seen", "1");
    });

    await page.goto("/");

    // Carousel should not appear
    await expect(
      page.getByRole("region", { name: "Welcome" })
    ).not.toBeVisible();

    // Library content should be visible
    await expect(page.getByRole("main")).toBeVisible();
  });
});
```

- [ ] **Step 2: Run E2E tests**

```bash
cd e2e && npx playwright test tests/welcome.spec.ts
```

Expected: all 3 tests pass. If they fail:
- Check that the Clerk mock in `fixtures/api-mock.ts` runs `setupClerkMock(page)` — the welcome carousel renders inside `AuthenticatedApp`, so auth must succeed.
- Check that `carousel.click()` advances cards. If tap-to-advance doesn't fire (the click might land on the Button on card 5), adjust the click target to avoid the CTA area.
- Check timing — if transitions cause flaky visibility, add `await page.waitForTimeout(100)` after each click.

- [ ] **Step 3: Commit**

```bash
git add e2e/tests/welcome.spec.ts
git commit -m "test(onboarding): E2E specs for welcome carousel"
```

---

## Post-implementation checklist

After all tasks are complete:

- [ ] Run full test suite: `cargo test && cd e2e && npx playwright test`
- [ ] Run formatting + lint: `cargo fmt --check && cargo clippy -- -D warnings`
- [ ] Manual test on iOS simulator: `just ios-dev` → sign in → verify carousel shows, haptics fire, safe-area insets look right
- [ ] Verify design catalogue entry at `http://localhost:8080/design` (debug build) — WelcomeCard renders, reset button works
- [ ] Create Pencil designs for the five cards + carousel chrome in `design/intrada.pen` (per CLAUDE.md Pencil workflow — required before merge but can run in parallel with code)
- [ ] Open GitHub issue: "Server-side welcome-seen persistence (part of #100)" with label `horizon:next`

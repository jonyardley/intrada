use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use web_sys::TouchEvent;

use crate::components::{Button, ButtonSize, ButtonVariant, WelcomeCard, WelcomeMark};
use intrada_web::haptics;

/// localStorage key for the "welcome seen" flag.
pub const WELCOME_SEEN_KEY: &str = "intrada:welcome-seen";

const CARD_COUNT: usize = 5;

/// Horizontal distance (px) to recognise a swipe gesture.
const SWIPE_THRESHOLD: f64 = 30.0;

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
        anchor: "Focus, reflect, move on",
        continuation: Some(
            "Run timed sessions. Score what happened while it\u{2019}s still fresh, then on to the next.",
        ),
    },
    CardContent {
        label: Some("TRACK"),
        anchor: "Watch the work add up",
        continuation: Some(
            "Music improves between sessions, not within them \u{2014} so the data shows what your ears can\u{2019}t hear yet.",
        ),
    },
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

fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|mq| mq.matches())
        .unwrap_or(false)
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
        if prefers_reduced_motion() {
            card_index.set(target);
            return;
        }
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

    let dismiss_skip = {
        let navigate = navigate.clone();
        move || {
            haptics::haptic_light();
            mark_welcome_seen();
            show.set(false);
            navigate("/library", Default::default());
        }
    };

    let dismiss_cta = {
        let navigate = navigate.clone();
        move || {
            // No explicit haptic here — Button::Primary fires haptic_light
            // on click internally. Adding another would double-tap on iOS.
            mark_welcome_seen();
            show.set(false);
            navigate("/library", Default::default());
        }
    };

    // ── Touch/swipe handling ───────────────────────────────────────────
    // Raw addEventListener via Effect+Closure (the swipe_actions.rs pattern).
    // No preventDefault is needed: the overlay is `fixed inset-0` with no
    // scrollable content beneath it during the welcome flow.

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
                        let current = card_index.get_untracked();
                        if current < CARD_COUNT - 1 {
                            haptics::haptic_selection();
                            if prefers_reduced_motion() {
                                card_index.set(current + 1);
                            } else {
                                transitioning.set(true);
                                leptos::task::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(50).await;
                                    card_index.set(current + 1);
                                    transitioning.set(false);
                                });
                            }
                        }
                    } else {
                        let current = card_index.get_untracked();
                        if current > 0 {
                            haptics::haptic_selection();
                            if prefers_reduced_motion() {
                                card_index.set(current - 1);
                            } else {
                                transitioning.set(true);
                                leptos::task::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(50).await;
                                    card_index.set(current - 1);
                                    transitioning.set(false);
                                });
                            }
                        }
                    }
                }
            }

            touch_start_x.set(None);
            touch_start_y.set(None);
        });

        let touchcancel: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            touch_start_x.set(None);
            touch_start_y.set(None);
        });

        let _ =
            el.add_event_listener_with_callback("touchstart", touchstart.as_ref().unchecked_ref());
        let _ = el.add_event_listener_with_callback("touchend", touchend.as_ref().unchecked_ref());
        let _ = el
            .add_event_listener_with_callback("touchcancel", touchcancel.as_ref().unchecked_ref());

        let touchstart = SendWrapper::new(touchstart);
        let touchend = SendWrapper::new(touchend);
        let touchcancel = SendWrapper::new(touchcancel);
        let el = SendWrapper::new(el);
        on_cleanup(move || {
            let _ = el.remove_event_listener_with_callback(
                "touchstart",
                touchstart.as_ref().unchecked_ref(),
            );
            let _ = el
                .remove_event_listener_with_callback("touchend", touchend.as_ref().unchecked_ref());
            let _ = el.remove_event_listener_with_callback(
                "touchcancel",
                touchcancel.as_ref().unchecked_ref(),
            );
        });
    });

    // ── Callbacks for Leptos event handlers ─────────────────────────────

    // `Callback` is needed for Button's `on_click` prop; plain closures
    // are used for Leptos `on:click` event attributes (which require FnMut).

    let on_cta_cb = Callback::new(move |_: ev::MouseEvent| {
        dismiss_cta();
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
            class="fixed inset-0 z-[2000] flex flex-col items-center justify-center bg-linear-to-b from-[var(--color-bg-gradient-top)] to-[var(--color-bg-gradient-bottom)]"
            style="padding-top: env(safe-area-inset-top, 0); padding-bottom: env(safe-area-inset-bottom, 0);"
            on:click=move |_: ev::MouseEvent| {
                // Tap-to-advance on desktop (not on the final card — that has
                // the CTA button which handles its own click).
                let current = card_index.get_untracked();
                if current < CARD_COUNT - 1 {
                    advance();
                }
            }
            role="region"
            aria-label="Welcome"
        >
            // Skip link — top right
            <button
                type="button"
                class="absolute top-4 right-4 px-2 py-1 text-sm text-muted bg-transparent border-none cursor-pointer"
                style="top: calc(env(safe-area-inset-top, 16px) + 0.5rem);"
                on:click=move |ev: ev::MouseEvent| {
                    ev.stop_propagation(); // Don't trigger card-click advance
                    dismiss_skip();
                }
            >
                "Skip"
            </button>

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

            // Progress dots
            <div
                class="welcome-dots absolute"
                style="bottom: calc(env(safe-area-inset-bottom, 32px) + 1rem);"
            >
                {(0..CARD_COUNT)
                    .map(|i| {
                        let is_active = move || card_index.get() == i;
                        let dot_class = move || {
                            if is_active() {
                                "welcome-dot welcome-dot--active"
                            } else {
                                "welcome-dot"
                            }
                        };
                        let aria_current = move || if is_active() { "true" } else { "false" };
                        view! {
                            <button
                                type="button"
                                class=dot_class
                                aria-current=aria_current
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
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

use leptos::portal::Portal;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, KeyboardEvent, TouchEvent};

use intrada_web::haptics::haptic_medium;

/// Long-press hold duration before the menu opens (ms). Matches iOS
/// "Default" haptic-touch speed. iOS 17's "Fast" is ~200ms; we use the
/// classic feel here.
const LONG_PRESS_MS: i32 = 500;
/// Maximum finger movement (px) during the hold before the gesture is
/// abandoned (treated as a scroll, not a long-press).
const LONG_PRESS_TOLERANCE: f64 = 8.0;
/// Menu width (kept in sync with CSS — used for viewport-edge clamping).
const MENU_WIDTH_PX: f64 = 240.0;
/// Approximate menu height (kept in sync with CSS — used for viewport-edge
/// clamping). Per-item height adds to this; rough estimate fine for clamp.
const MENU_HEIGHT_PX: f64 = 200.0;

/// A single action in a context menu.
#[derive(Clone)]
pub struct ContextMenuAction {
    pub label: String,
    pub destructive: bool,
    pub on_select: Callback<()>,
}

/// iOS-style long-press context menu (`UIContextMenuInteraction` feel).
///
/// Wraps a trigger element with long-press detection: holding for
/// ~500ms (without significant movement) opens a floating menu of
/// actions positioned near the touch point. Background dims, tap-outside
/// or selecting an action dismisses. Medium haptic fires on activation —
/// the same pattern native iOS uses.
///
/// On non-iOS devices the long-press behaviour is hidden entirely (the
/// gesture is touch-only); right-click or other desktop affordances would
/// be a future addition.
#[component]
pub fn ContextMenu(actions: Vec<ContextMenuAction>, children: Children) -> impl IntoView {
    let is_open = RwSignal::new(false);
    // Touch coordinates captured at touchstart; used to position the menu
    // and to abandon the gesture if the finger moves too far.
    let menu_x = RwSignal::new(0.0_f64);
    let menu_y = RwSignal::new(0.0_f64);
    let touch_start_x = RwSignal::new(None::<f64>);
    let touch_start_y = RwSignal::new(None::<f64>);

    let trigger_ref = NodeRef::<leptos::html::Div>::new();

    // Stored so the touch-handler closures can re-arm via signal sets, while
    // the actions list is captured into the menu render.
    let actions = StoredValue::new(actions);

    let close = Callback::new(move |_: ()| {
        is_open.set(false);
        touch_start_x.set(None);
        touch_start_y.set(None);
    });

    // Escape key dismisses the menu when open.
    Effect::new(move || {
        if !is_open.get() {
            return;
        }
        let on_keydown: Closure<dyn Fn(KeyboardEvent)> = Closure::new(move |ev: KeyboardEvent| {
            if ev.key() == "Escape" {
                close.run(());
            }
        });
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref());
        }
        on_keydown.forget();
    });

    // Long-press detection on the trigger. Uses touchstart + setTimeout to
    // schedule activation; touchmove past tolerance or touchend before the
    // timeout cancels.
    Effect::new(move || {
        let Some(el) = trigger_ref.get() else {
            return;
        };

        // setTimeout handle so we can cancel on early release / scroll.
        let timeout_handle = std::rc::Rc::new(std::cell::Cell::new(None::<i32>));

        let cancel_pending = {
            let timeout_handle = std::rc::Rc::clone(&timeout_handle);
            move || {
                if let Some(handle) = timeout_handle.take() {
                    if let Some(window) = web_sys::window() {
                        window.clear_timeout_with_handle(handle);
                    }
                }
            }
        };

        let touchstart: Closure<dyn Fn(TouchEvent)> = {
            let timeout_handle = std::rc::Rc::clone(&timeout_handle);
            let cancel_pending = cancel_pending.clone();
            Closure::new(move |ev: TouchEvent| {
                cancel_pending();
                if let Some(touch) = ev.touches().get(0) {
                    let x = touch.client_x() as f64;
                    let y = touch.client_y() as f64;
                    touch_start_x.set(Some(x));
                    touch_start_y.set(Some(y));

                    // Schedule activation after LONG_PRESS_MS.
                    let activate: Closure<dyn Fn()> = Closure::new(move || {
                        // Position menu, clamped to viewport edges so it
                        // doesn't render off-screen for taps near the right
                        // or bottom of the page.
                        let viewport_w = web_sys::window()
                            .and_then(|w| w.inner_width().ok())
                            .and_then(|v| v.as_f64())
                            .unwrap_or(393.0);
                        let viewport_h = web_sys::window()
                            .and_then(|w| w.inner_height().ok())
                            .and_then(|v| v.as_f64())
                            .unwrap_or(852.0);
                        let clamped_x = x.min(viewport_w - MENU_WIDTH_PX - 8.0).max(8.0);
                        let clamped_y = y.min(viewport_h - MENU_HEIGHT_PX - 8.0).max(8.0);
                        menu_x.set(clamped_x);
                        menu_y.set(clamped_y);
                        haptic_medium();
                        is_open.set(true);
                    });

                    if let Some(window) = web_sys::window() {
                        if let Ok(handle) = window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                activate.as_ref().unchecked_ref(),
                                LONG_PRESS_MS,
                            )
                        {
                            timeout_handle.set(Some(handle));
                        }
                    }
                    activate.forget();
                }
            })
        };

        let touchmove: Closure<dyn Fn(TouchEvent)> = {
            let cancel_pending = cancel_pending.clone();
            Closure::new(move |ev: TouchEvent| {
                let (Some(start_x), Some(start_y)) =
                    (touch_start_x.get_untracked(), touch_start_y.get_untracked())
                else {
                    return;
                };
                let Some(touch) = ev.touches().get(0) else {
                    return;
                };
                let dx = (touch.client_x() as f64 - start_x).abs();
                let dy = (touch.client_y() as f64 - start_y).abs();
                if dx > LONG_PRESS_TOLERANCE || dy > LONG_PRESS_TOLERANCE {
                    cancel_pending();
                    touch_start_x.set(None);
                    touch_start_y.set(None);
                }
            })
        };

        let touchend: Closure<dyn Fn(TouchEvent)> = {
            let cancel_pending = cancel_pending.clone();
            Closure::new(move |_: TouchEvent| {
                cancel_pending();
                touch_start_x.set(None);
                touch_start_y.set(None);
            })
        };

        let touchcancel: Closure<dyn Fn(TouchEvent)> = {
            let cancel_pending = cancel_pending.clone();
            Closure::new(move |_: TouchEvent| {
                cancel_pending();
                touch_start_x.set(None);
                touch_start_y.set(None);
            })
        };

        let opts = AddEventListenerOptions::new();
        opts.set_passive(true);

        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchstart",
            touchstart.as_ref().unchecked_ref(),
            &opts,
        );
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchmove",
            touchmove.as_ref().unchecked_ref(),
            &opts,
        );
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchend",
            touchend.as_ref().unchecked_ref(),
            &opts,
        );
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchcancel",
            touchcancel.as_ref().unchecked_ref(),
            &opts,
        );

        touchstart.forget();
        touchmove.forget();
        touchend.forget();
        touchcancel.forget();
    });

    let backdrop_class = move || {
        if is_open.get() {
            "context-menu-backdrop context-menu-backdrop--open"
        } else {
            "context-menu-backdrop"
        }
    };

    let menu_class = move || {
        if is_open.get() {
            "context-menu context-menu--open"
        } else {
            "context-menu"
        }
    };

    let menu_style = move || format!("left: {}px; top: {}px;", menu_x.get(), menu_y.get());

    view! {
        <div class="context-menu-trigger" node_ref=trigger_ref>
            {children()}
        </div>
        <Portal>
            <div
                class=backdrop_class
                on:click=move |_| close.run(())
                aria-hidden="true"
            ></div>
            <div
                class=menu_class
                style=menu_style
                role="menu"
            >
                {move || {
                    actions.with_value(|actions| {
                        actions.iter().map(|action| {
                            let label = action.label.clone();
                            let destructive = action.destructive;
                            let on_select = action.on_select;
                            let class = if destructive {
                                "context-menu-item context-menu-item--destructive"
                            } else {
                                "context-menu-item"
                            };
                            view! {
                                <button
                                    type="button"
                                    class=class
                                    role="menuitem"
                                    on:click=move |_| {
                                        on_select.run(());
                                        close.run(());
                                    }
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()
                    })
                }}
            </div>
        </Portal>
    }
}

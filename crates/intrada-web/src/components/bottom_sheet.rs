use leptos::portal::Portal;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, KeyboardEvent, TouchEvent};

use intrada_web::haptics::haptic_light;

/// Drag distance (px) past which a swipe-down releases the sheet to close.
const DRAG_DISMISS_THRESHOLD: f64 = 120.0;
/// Soft asymptote for upward drag resistance — the sheet doesn't move
/// proportionally past 0; pulling up hits a wall almost immediately.
const UPWARD_DRAG_RESISTANCE: f64 = 0.15;

/// iOS-style modal bottom sheet — slides up from the bottom of the viewport
/// over a dimmed backdrop. Matches the modern UISheetPresentationController
/// feel: rounded top corners, drag handle, swipe-down or backdrop-tap to
/// dismiss, optional Cancel button in a sheet-internal nav bar.
///
/// Renders into `<body>` via Portal so positioning is anchored to the viewport
/// and the sheet correctly overlays the app's `<main>` (which is itself
/// position: fixed on iOS).
#[component]
pub fn BottomSheet(
    /// Reactive open state. Sheet animates open when this becomes true.
    #[prop(into)]
    open: Signal<bool>,
    /// Fired when user dismisses (backdrop tap, swipe down past threshold,
    /// Cancel button, or Escape key on web). The parent is responsible for
    /// flipping `open` to false in response.
    on_close: Callback<()>,
    /// Optional title rendered in a small nav bar at the top of the sheet.
    /// When provided, a Cancel button is also shown on the left.
    #[prop(optional, into)]
    nav_title: Option<String>,
    /// Optional trailing-action label shown on the right of the nav bar
    /// (the iOS Mail-compose pattern: Cancel | Title | Send). Requires
    /// `nav_title` to be set.
    #[prop(optional, into)]
    nav_action_label: Option<String>,
    /// Click handler for the nav-bar trailing action.
    #[prop(optional, into)]
    on_nav_action: Option<Callback<()>>,
    /// Reactive disabled state for the trailing action.
    #[prop(optional, into)]
    nav_action_disabled: Option<Signal<bool>>,
    children: ChildrenFn,
) -> impl IntoView {
    // Stored so it can be cloned out of the Portal's Fn children closure
    // multiple times (Option<String> is moved otherwise).
    let nav_title = StoredValue::new(nav_title);
    let nav_action_label = StoredValue::new(nav_action_label);
    let nav_action_disabled = nav_action_disabled.unwrap_or_else(|| Signal::derive(|| false));

    // Live drag offset in px (positive = sheet pulled down). Drives the
    // sheet's transform during a touch drag. Snaps back to 0 or fires
    // on_close on touchend depending on whether threshold was crossed.
    let drag_offset = RwSignal::new(0.0_f64);
    let touch_start_y = RwSignal::new(None::<f64>);
    let dismiss_haptic_armed = RwSignal::new(false);

    let sheet_ref = NodeRef::<leptos::html::Div>::new();
    // Drag handle gets its own ref so the swipe-to-dismiss listener attaches
    // only to it, not the whole sheet. Putting the touch listener on the
    // entire sheet intercepts every touchmove inside the body and blocks
    // the form's own scroll — user can't scroll long forms because every
    // drag becomes a dismiss attempt.
    let handle_ref = NodeRef::<leptos::html::Div>::new();

    // Dismissal helper used by Cancel, backdrop, Escape, and swipe-end.
    // Wrapped as a Callback (Copy) so it can be shared across many closures
    // without ownership headaches.
    let close = Callback::new(move |_: ()| {
        on_close.run(());
        drag_offset.set(0.0);
        touch_start_y.set(None);
        dismiss_haptic_armed.set(false);
    });

    // Attach Escape-to-close at the document level when the sheet opens.
    Effect::new(move || {
        if !open.get() {
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

    // Wire touch handlers for swipe-down-to-dismiss. Attached to the handle
    // only — see the handle_ref doc comment for why. touchmove is non-passive
    // so we can preventDefault when taking over the gesture.
    Effect::new(move || {
        let Some(el) = handle_ref.get() else {
            return;
        };

        let touchstart: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            if let Some(touch) = ev.touches().get(0) {
                touch_start_y.set(Some(touch.client_y() as f64));
                drag_offset.set(0.0);
                dismiss_haptic_armed.set(false);
            }
        });

        let touchmove: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            let Some(start_y) = touch_start_y.get_untracked() else {
                return;
            };
            let Some(touch) = ev.touches().get(0) else {
                return;
            };
            let delta = touch.client_y() as f64 - start_y;
            let offset = if delta >= 0.0 {
                // Pulling down: 1:1 movement
                delta
            } else {
                // Pulling up: heavy resistance — sheet can't grow past its
                // open position
                delta * UPWARD_DRAG_RESISTANCE
            };
            ev.prevent_default();
            drag_offset.set(offset);

            // Haptic at the dismiss threshold so the user feels the
            // commitment point — same pattern as native iOS sheet swipe.
            if offset >= DRAG_DISMISS_THRESHOLD && !dismiss_haptic_armed.get_untracked() {
                haptic_light();
                dismiss_haptic_armed.set(true);
            } else if offset < DRAG_DISMISS_THRESHOLD && dismiss_haptic_armed.get_untracked() {
                dismiss_haptic_armed.set(false);
            }
        });

        let touchend: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            let offset = drag_offset.get_untracked();
            if offset >= DRAG_DISMISS_THRESHOLD {
                close.run(());
            } else {
                drag_offset.set(0.0);
                touch_start_y.set(None);
            }
        });

        let touchcancel: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            drag_offset.set(0.0);
            touch_start_y.set(None);
            dismiss_haptic_armed.set(false);
        });

        let passive_opts = AddEventListenerOptions::new();
        passive_opts.set_passive(true);
        let active_opts = AddEventListenerOptions::new();
        active_opts.set_passive(false);

        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchstart",
            touchstart.as_ref().unchecked_ref(),
            &passive_opts,
        );
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchmove",
            touchmove.as_ref().unchecked_ref(),
            &active_opts,
        );
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchend",
            touchend.as_ref().unchecked_ref(),
            &passive_opts,
        );
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "touchcancel",
            touchcancel.as_ref().unchecked_ref(),
            &passive_opts,
        );

        touchstart.forget();
        touchmove.forget();
        touchend.forget();
        touchcancel.forget();
    });

    let backdrop_class = move || {
        if open.get() {
            "bottom-sheet-backdrop bottom-sheet-backdrop--open"
        } else {
            "bottom-sheet-backdrop"
        }
    };

    let sheet_class = move || {
        if open.get() {
            "bottom-sheet bottom-sheet--open"
        } else {
            "bottom-sheet"
        }
    };

    // While dragging, transform follows the finger (no transition lag).
    // While idle (open or closed), CSS handles the slide via transition.
    let sheet_style = move || {
        let offset = drag_offset.get();
        let active = touch_start_y.get().is_some();
        if active && offset > 0.0 {
            format!("transform: translateY({offset}px); transition: none;")
        } else {
            String::new()
        }
    };

    view! {
        <Portal>
            <div
                class=backdrop_class
                on:click=move |_| close.run(())
                aria-hidden="true"
            ></div>
            <div
                class=sheet_class
                style=sheet_style
                node_ref=sheet_ref
                role="dialog"
                aria-modal="true"
            >
                <div
                    class="bottom-sheet-handle-area"
                    node_ref=handle_ref
                    aria-hidden="true"
                >
                    <div class="bottom-sheet-handle"></div>
                </div>
                {move || nav_title.get_value().map(|title| view! {
                    <div class="bottom-sheet-nav">
                        <button
                            class="bottom-sheet-cancel"
                            type="button"
                            on:click=move |_| close.run(())
                        >
                            "Cancel"
                        </button>
                        <h2 class="bottom-sheet-title">{title}</h2>
                        {match (nav_action_label.get_value(), on_nav_action) {
                            (Some(label), Some(handler)) => view! {
                                <button
                                    class="bottom-sheet-action"
                                    type="button"
                                    disabled=move || nav_action_disabled.get()
                                    on:click=move |_| handler.run(())
                                >
                                    {label}
                                </button>
                            }.into_any(),
                            _ => view! {
                                <span class="bottom-sheet-nav-spacer"></span>
                            }.into_any(),
                        }}
                    </div>
                })}
                <div class="bottom-sheet-body">
                    // NOTE: children stay mounted even when the sheet is
                    // closed (just translated off-screen). Tests querying
                    // page-wide for elements that also exist inside the
                    // sheet (e.g. "Cancel" button) need to scope to <main>
                    // to avoid matching the off-screen sheet contents.
                    {children()}
                </div>
            </div>
        </Portal>
    }
}

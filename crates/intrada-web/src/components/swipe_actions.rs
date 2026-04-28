use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, TouchEvent};

use intrada_web::haptics::haptic_light;

/// Width of the revealed action area (px) when the row is half-open.
const ACTION_WIDTH: f64 = 88.0;
/// Minimum horizontal movement (px) before we recognise a swipe and take
/// over the gesture from any underlying tap target. Below this threshold,
/// the touch passes through and a child link's tap still works.
const SWIPE_THRESHOLD: f64 = 8.0;
/// Movement past the half-open snap (px) at which a release fully commits
/// the destructive action without requiring a button tap — matches native
/// iOS UISwipeActionsConfiguration's "swipe to delete" behaviour.
const COMMIT_THRESHOLD: f64 = 200.0;
/// Resistance applied to over-swipes past the action width. Past
/// ACTION_WIDTH + this much extra finger movement, the row stops moving —
/// a soft cap rather than a hard stop.
const OVERSWIPE_DAMPING: f64 = 0.4;

/// iOS-style swipe-to-reveal action row. Wraps children with a touch-driven
/// swipe gesture: swiping left reveals a single trailing action (typically
/// Delete) on the right. A "full swipe" past `COMMIT_THRESHOLD` releases
/// the action without requiring the user to tap the button — same as
/// native UITableViewCell with a destructive trailing action.
///
/// The underlying child link / button still receives normal taps when the
/// gesture is below `SWIPE_THRESHOLD` of horizontal movement.
#[component]
pub fn SwipeActions(
    /// Fired when the destructive action is invoked, either by tapping the
    /// revealed button or by full-swiping past the commit threshold.
    on_delete: Callback<()>,
    /// Label shown on the revealed action button. Default "Delete".
    #[prop(optional, into)]
    delete_label: Option<String>,
    children: Children,
) -> impl IntoView {
    // Current horizontal translation of the row. Negative = pulled left
    // (revealing actions on the right).
    let translate_x = RwSignal::new(0.0_f64);
    let touch_start_x = RwSignal::new(None::<f64>);
    let touch_start_y = RwSignal::new(None::<f64>);
    // Latched once the user has clearly committed to a horizontal swipe;
    // before that we let touches fall through (so a tap on the row still
    // navigates / triggers its child action).
    let gesture_committed = RwSignal::new(false);
    // True while `translate_x` is past the commit threshold — drives the
    // single haptic tap when crossing.
    let commit_haptic_armed = RwSignal::new(false);
    // Snapped-open state. Once partially open, taps outside the action area
    // close the row (handled by an ancestor click listener — see CSS).
    let is_open = RwSignal::new(false);

    let label = StoredValue::new(delete_label.unwrap_or_else(|| "Delete".to_string()));

    let row_ref = NodeRef::<leptos::html::Div>::new();

    let close = Callback::new(move |_: ()| {
        translate_x.set(0.0);
        touch_start_x.set(None);
        touch_start_y.set(None);
        gesture_committed.set(false);
        commit_haptic_armed.set(false);
        is_open.set(false);
    });

    let trigger_delete = Callback::new(move |_: ()| {
        on_delete.run(());
        close.run(());
    });

    Effect::new(move || {
        let Some(el) = row_ref.get() else {
            return;
        };

        let touchstart: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            if let Some(touch) = ev.touches().get(0) {
                touch_start_x.set(Some(touch.client_x() as f64));
                touch_start_y.set(Some(touch.client_y() as f64));
                gesture_committed.set(false);
            }
        });

        let touchmove: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            let Some(start_x) = touch_start_x.get_untracked() else {
                return;
            };
            let Some(start_y) = touch_start_y.get_untracked() else {
                return;
            };
            let Some(touch) = ev.touches().get(0) else {
                return;
            };
            let dx = touch.client_x() as f64 - start_x;
            let dy = touch.client_y() as f64 - start_y;

            // Decide once whether this is a horizontal swipe or a vertical
            // scroll. If the user's first significant move is vertical, let
            // the page scroll normally and abandon the gesture.
            if !gesture_committed.get_untracked() {
                if dy.abs() > SWIPE_THRESHOLD && dy.abs() > dx.abs() {
                    touch_start_x.set(None);
                    touch_start_y.set(None);
                    return;
                }
                if dx.abs() < SWIPE_THRESHOLD {
                    return;
                }
                gesture_committed.set(true);
            }

            ev.prevent_default();

            // Compute target translation. If the row was already open and
            // the user starts at -ACTION_WIDTH, account for that so the
            // gesture feels continuous from the open state.
            let baseline = if is_open.get_untracked() {
                -ACTION_WIDTH
            } else {
                0.0
            };
            let raw = baseline + dx;

            // Clamp + dampen. Pulling further left than ACTION_WIDTH gets
            // resistance; pulling right past 0 also gets resistance.
            let translation = if raw <= -ACTION_WIDTH {
                let overshoot = (-ACTION_WIDTH) - raw; // positive
                -ACTION_WIDTH - overshoot * OVERSWIPE_DAMPING
            } else if raw > 0.0 {
                raw * OVERSWIPE_DAMPING
            } else {
                raw
            };
            translate_x.set(translation);

            // Haptic at commit threshold (full-swipe-to-delete).
            if translation <= -COMMIT_THRESHOLD && !commit_haptic_armed.get_untracked() {
                haptic_light();
                commit_haptic_armed.set(true);
            } else if translation > -COMMIT_THRESHOLD && commit_haptic_armed.get_untracked() {
                commit_haptic_armed.set(false);
            }
        });

        let touchend: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            let pos = translate_x.get_untracked();
            touch_start_x.set(None);
            touch_start_y.set(None);
            gesture_committed.set(false);
            commit_haptic_armed.set(false);
            // Snap decisions:
            //   pos <= -COMMIT_THRESHOLD  → full commit, fire delete
            //   pos <= -ACTION_WIDTH/2    → snap open (reveal action button)
            //   else                      → snap closed
            if pos <= -COMMIT_THRESHOLD {
                trigger_delete.run(());
            } else if pos <= -(ACTION_WIDTH / 2.0) {
                translate_x.set(-ACTION_WIDTH);
                is_open.set(true);
            } else {
                translate_x.set(0.0);
                is_open.set(false);
            }
        });

        let touchcancel: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            touch_start_x.set(None);
            touch_start_y.set(None);
            gesture_committed.set(false);
            commit_haptic_armed.set(false);
            // Snap back without firing — touchcancel means the browser
            // claimed the touch (e.g. for scroll). Don't commit destructive
            // actions on cancel.
            let pos = translate_x.get_untracked();
            if pos <= -(ACTION_WIDTH / 2.0) && pos > -COMMIT_THRESHOLD {
                translate_x.set(-ACTION_WIDTH);
                is_open.set(true);
            } else {
                translate_x.set(0.0);
                is_open.set(false);
            }
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

    let content_style = move || {
        let x = translate_x.get();
        let active = touch_start_x.get().is_some();
        if active {
            // Drive 1:1 with finger during active gesture, no transition.
            format!("transform: translateX({x}px); transition: none;")
        } else {
            // Snapped — let CSS handle the spring.
            format!("transform: translateX({x}px);")
        }
    };

    view! {
        <div class="swipe-row" node_ref=row_ref>
            <button
                type="button"
                class="swipe-row-action swipe-row-action--destructive"
                on:click=move |ev| {
                    ev.stop_propagation();
                    trigger_delete.run(());
                }
                tabindex=move || if is_open.get() { "0" } else { "-1" }
                aria-hidden=move || if is_open.get() { "false" } else { "true" }
            >
                {move || label.get_value()}
            </button>
            <div class="swipe-row-content" style=content_style>
                {children()}
            </div>
        </div>
    }
}

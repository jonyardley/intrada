use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, TouchEvent};

use intrada_web::haptics::haptic_light;

/// Pixels of pull required to commit to a refresh on release. Sized so the
/// indicator (32px tall) is fully clear of the safe area / status bar by the
/// time threshold is hit — feels like a deliberate gesture, not an accident.
const PULL_THRESHOLD: f64 = 110.0;
/// Asymptotic cap on pull translation past threshold. The rubber-band curve
/// below approaches but never reaches this — pulling harder past threshold
/// gives progressively less displacement, matching native iOS scroll-view
/// rubber-banding rather than a hard stop. Larger values = more give per
/// extra pull (less stiff); smaller = stiffer.
const MAX_OVERPULL: f64 = 150.0;
/// Multiplier on raw finger displacement before the rubber-band curve.
/// Slight amplification (>1) gives the visual "snappier than 1:1" feel
/// the user expects from a native iOS scroll view — content moves a hair
/// faster than the finger, which reads as "responsive" rather than "lazy".
const PULL_AMPLIFICATION: f64 = 1.15;

/// iOS-style elastic resistance curve. Linear (1:1) up to `threshold`, then
/// asymptotic past it: pulling harder still moves the content but with
/// diminishing returns, never exceeding `threshold + max_overpull`.
fn rubber_band(delta: f64, threshold: f64, max_overpull: f64) -> f64 {
    if delta <= threshold {
        delta
    } else {
        let overshoot = delta - threshold;
        threshold + (1.0 - 1.0 / (overshoot / max_overpull + 1.0)) * max_overpull
    }
}

/// Returns the scrollTop of the closest ancestor scroll container (`<main>`).
/// On iOS our scroll container is `<main>` (body is fixed); on other platforms
/// the body scrolls so we fall back to `window.scrollY`.
fn scroll_top() -> f64 {
    document()
        .query_selector("main")
        .ok()
        .flatten()
        .map(|el| el.scroll_top() as f64)
        .unwrap_or_else(|| window().scroll_y().unwrap_or(0.0))
}

/// Wraps content with iOS-style pull-to-refresh.
///
/// Takes over the gesture (preventDefault on touchmove) and translates the
/// wrapper transform 1:1 with the user's finger — no browser bounce, no
/// rate mismatch. The indicator sits above the content edge via position:
/// absolute, so it travels with the wrapper into the empty space that
/// opens up above the content as it slides down.
///
/// Hidden entirely on non-iOS platforms via CSS — the gesture is iOS-only.
#[component]
pub fn PullToRefresh(
    on_refresh: Callback<()>,
    #[prop(into)] is_refreshing: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let pull_distance = RwSignal::new(0.0_f64);
    let touch_start_y = RwSignal::new(None::<f64>);
    // True once we've fired the "you've crossed threshold" haptic for the
    // current gesture. Reset on touchend / drop-back-below-threshold so the
    // haptic fires again if the user pulls past, dips back, and re-crosses.
    let threshold_haptic_fired = RwSignal::new(false);

    let wrapper_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new(move || {
        let Some(el) = wrapper_ref.get() else {
            return;
        };

        let touchstart: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            // Touch handlers run as raw JS callbacks outside any Leptos owner
            // context — use .get_untracked()/.set() so signal access doesn't
            // try to register reactive subscriptions (which would just warn
            // and bail). Setters still propagate to subscribers normally.
            if scroll_top() > 0.0 {
                return;
            }
            if let Some(touch) = ev.touches().get(0) {
                touch_start_y.set(Some(touch.client_y() as f64));
                pull_distance.set(0.0);
            }
        });

        let touchmove: Closure<dyn Fn(TouchEvent)> = Closure::new(move |ev: TouchEvent| {
            let Some(start_y) = touch_start_y.get_untracked() else {
                return;
            };
            if scroll_top() > 0.0 {
                touch_start_y.set(None);
                pull_distance.set(0.0);
                return;
            }
            let Some(touch) = ev.touches().get(0) else {
                return;
            };
            let delta = touch.client_y() as f64 - start_y;
            if delta > 0.0 {
                // Take over from WKWebView's native rubber-band so we can
                // drive the wrapper transform — slightly amplified for the
                // "snappier than 1:1" iOS feel. Past threshold the rubber-
                // band curve provides elastic resistance.
                ev.prevent_default();
                let amplified = delta * PULL_AMPLIFICATION;
                let dist = rubber_band(amplified, PULL_THRESHOLD, MAX_OVERPULL);
                pull_distance.set(dist);

                // Fire a single light haptic on threshold crossing — same
                // pattern as native iOS UIRefreshControl ("you can release
                // now"). Re-arms if user dips back below threshold.
                if dist >= PULL_THRESHOLD && !threshold_haptic_fired.get_untracked() {
                    haptic_light();
                    threshold_haptic_fired.set(true);
                } else if dist < PULL_THRESHOLD && threshold_haptic_fired.get_untracked() {
                    threshold_haptic_fired.set(false);
                }
            } else {
                pull_distance.set(0.0);
            }
        });

        let touchend: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            let dist = pull_distance.get_untracked();
            let started = touch_start_y.get_untracked().is_some();
            if started && dist >= PULL_THRESHOLD && !is_refreshing.get_untracked() {
                on_refresh.run(());
            }
            touch_start_y.set(None);
            pull_distance.set(0.0);
            threshold_haptic_fired.set(false);
        });

        let touchcancel: Closure<dyn Fn(TouchEvent)> = Closure::new(move |_: TouchEvent| {
            touch_start_y.set(None);
            pull_distance.set(0.0);
            threshold_haptic_fired.set(false);
        });

        // touchmove must be non-passive so prevent_default works — that's
        // what lets us take over from the browser's bounce. The other three
        // can stay passive (small perf win, no preventDefault needed).
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

        // Listeners outlive the component; same pattern as use_drag_reorder.
        touchstart.forget();
        touchmove.forget();
        touchend.forget();
        touchcancel.forget();
    });

    // Drives the wrapper's transform — both content and indicator translate
    // together. Transition only applies when the user isn't actively pulling,
    // so the snap-back / refresh-lock animates smoothly without lagging
    // behind the finger during a drag.
    let wrapper_style = move || {
        let dist = if is_refreshing.get() {
            PULL_THRESHOLD
        } else {
            pull_distance.get()
        };
        let transition = if touch_start_y.get().is_some() {
            ""
        } else {
            " transition: transform 200ms ease;"
        };
        format!("transform: translateY({dist}px);{transition}")
    };

    let spinner_class = move || {
        if is_refreshing.get() {
            "pull-spinner-svg spinning"
        } else {
            "pull-spinner-svg"
        }
    };

    // iOS-style progressive spoke reveal. Each spoke "lights up" sequentially
    // as the user pulls — at ratio i/12 spoke i starts to fade in, fully
    // visible at (i+1)/12. Mimics native UIRefreshControl's "lines" spinner
    // drawing itself in. While refreshing, all spokes are at their base
    // opacity (the CSS rotation animation handles the spinning visual).
    //
    // Base opacity gradient (1.0 → 0.13) is the canonical iOS pattern: the
    // "head" at 12 o'clock is brightest, trailing spokes dim around the dial.
    const SPOKE_BASES: [f64; 12] = [
        1.0, 0.85, 0.7, 0.6, 0.5, 0.4, 0.3, 0.25, 0.2, 0.18, 0.15, 0.13,
    ];

    let spoke_opacity = move |idx: usize| -> String {
        let base = SPOKE_BASES[idx];
        if is_refreshing.get() {
            return format!("{base:.3}");
        }
        let ratio = (pull_distance.get() / PULL_THRESHOLD).min(1.0);
        // Spoke `idx` ramps from 0 → base opacity as ratio crosses idx/12 → (idx+1)/12.
        let visibility = ((ratio * 12.0) - idx as f64).clamp(0.0, 1.0);
        format!("{:.3}", visibility * base)
    };

    let spokes = (0..12usize)
        .map(|idx| {
            let rotation = idx * 30;
            let opacity_fn = move || spoke_opacity(idx);
            view! {
                <rect
                    x="11"
                    y="2"
                    width="2"
                    height="5"
                    rx="1"
                    opacity=opacity_fn
                    transform=format!("rotate({rotation} 12 12)")
                />
            }
        })
        .collect_view();

    view! {
        <div class="pull-to-refresh" node_ref=wrapper_ref style=wrapper_style>
            <div class="pull-to-refresh-indicator">
                <svg
                    class=spinner_class
                    viewBox="0 0 24 24"
                    width="24"
                    height="24"
                    aria-hidden="true"
                    fill="currentColor"
                >
                    {spokes}
                </svg>
            </div>
            {children()}
        </div>
    }
}

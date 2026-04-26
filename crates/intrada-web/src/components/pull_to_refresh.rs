use leptos::ev;
use leptos::prelude::*;

const PULL_THRESHOLD: f64 = 80.0;
const MAX_PULL: f64 = 120.0;

/// Wraps content with iOS-style pull-to-refresh.
///
/// Listens for touch/pointer pulls when the page is scrolled to top.
/// Past `PULL_THRESHOLD` the on_refresh callback fires; the spinner
/// stays visible while `is_refreshing` is true.
///
/// Hidden entirely on non-iOS platforms via CSS — the gesture is iOS-only.
#[component]
pub fn PullToRefresh(
    on_refresh: Callback<()>,
    #[prop(into)] is_refreshing: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let pull_distance = RwSignal::new(0.0_f64);
    let pointer_start_y = RwSignal::new(None::<f64>);

    let on_pointerdown = move |ev: ev::PointerEvent| {
        // Only respond to touch (not mouse), and only when at the top of the page
        if ev.pointer_type() != "touch" {
            return;
        }
        if window().scroll_y().unwrap_or(0.0) > 0.0 {
            return;
        }
        pointer_start_y.set(Some(ev.client_y() as f64));
        pull_distance.set(0.0);
    };

    let on_pointermove = move |ev: ev::PointerEvent| {
        let Some(start_y) = pointer_start_y.get() else {
            return;
        };
        // Bail if user has scrolled away from the top mid-drag
        if window().scroll_y().unwrap_or(0.0) > 0.0 {
            pointer_start_y.set(None);
            pull_distance.set(0.0);
            return;
        }
        let delta = ev.client_y() as f64 - start_y;
        if delta > 0.0 {
            pull_distance.set(delta.min(MAX_PULL));
        }
    };

    let on_pointerup = move |_: ev::PointerEvent| {
        if pointer_start_y.get().is_some()
            && pull_distance.get() >= PULL_THRESHOLD
            && !is_refreshing.get()
        {
            on_refresh.run(());
        }
        pointer_start_y.set(None);
        pull_distance.set(0.0);
    };

    let indicator_style = move || {
        if is_refreshing.get() {
            // Locked at threshold while refresh is in flight
            format!("opacity: 1; transform: translateY({PULL_THRESHOLD}px);")
        } else {
            let dist = pull_distance.get();
            let opacity = (dist / PULL_THRESHOLD).min(1.0);
            format!("opacity: {opacity}; transform: translateY({dist}px);")
        }
    };

    let spinner_class = move || {
        if is_refreshing.get() || pull_distance.get() >= PULL_THRESHOLD {
            "pull-spinner spinning"
        } else {
            "pull-spinner"
        }
    };

    view! {
        <div
            class="pull-to-refresh"
            on:pointerdown=on_pointerdown
            on:pointermove=on_pointermove
            on:pointerup=on_pointerup
            on:pointercancel=on_pointerup
        >
            <div class="pull-to-refresh-indicator" style=indicator_style>
                <div class=spinner_class></div>
            </div>
            {children()}
        </div>
    }
}

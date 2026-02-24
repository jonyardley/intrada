use leptos::prelude::*;

/// Circular SVG timer with digital readout centred inside.
///
/// When `planned_duration_secs` is provided the ring fills clockwise from the
/// top as elapsed time approaches the target, changing colour on completion.
/// When `None`, the ring displays as a decorative track with no progress fill —
/// giving a consistent circular timer appearance regardless of whether the user
/// set a duration.
#[component]
pub fn ProgressRing(
    /// Reactive signal for elapsed seconds (ticks every 1s from SessionTimer).
    elapsed_secs: RwSignal<u32>,
    /// Total planned duration in seconds, or `None` for a plain clock display.
    #[prop(optional)]
    planned_duration_secs: Option<u32>,
) -> impl IntoView {
    // SVG geometry: viewBox 0 0 200 200, circle at (100, 100), radius 88.
    // Stroke width 8 fits within viewBox with room to spare.
    let radius: f64 = 88.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;

    view! {
        <div class="progress-ring-wrapper">
            <svg class="progress-ring-svg" viewBox="0 0 200 200" aria-hidden="true">
                // Track circle (background ring — always visible)
                <circle
                    cx="100" cy="100" r=radius.to_string()
                    fill="none"
                    stroke="var(--color-progress-track)"
                    stroke-width="8"
                />
                // Progress circle (only rendered when a planned duration exists)
                {planned_duration_secs.map(|planned| {
                    view! {
                        <circle
                            cx="100" cy="100" r=radius.to_string()
                            fill="none"
                            stroke-width="8"
                            stroke-linecap="round"
                            stroke-dasharray=circumference.to_string()
                            stroke-dashoffset=move || {
                                let elapsed = elapsed_secs.get() as f64;
                                let progress = (elapsed / planned as f64).min(1.0);
                                let offset = circumference * (1.0 - progress);
                                format!("{offset:.2}")
                            }
                            stroke=move || {
                                if elapsed_secs.get() >= planned {
                                    "var(--color-progress-complete)"
                                } else {
                                    "var(--color-progress-fill)"
                                }
                            }
                            transform="rotate(-90 100 100)"
                            class="progress-ring-circle"
                        />
                    }
                })}
            </svg>
            // Digital timer centred inside the ring
            <div class="progress-ring-timer">
                <p class="text-4xl sm:text-6xl font-mono font-bold text-primary">
                    {move || {
                        let secs = elapsed_secs.get();
                        format!("{:02}:{:02}", secs / 60, secs % 60)
                    }}
                </p>
            </div>
        </div>
    }
}

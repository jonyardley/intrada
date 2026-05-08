use leptos::prelude::*;

/// Visual state to convey on the status dot.
///
/// `Building` = a session has been started in the planner but is not yet
/// running. Amber, no pulse — "you've got something in the kitchen".
///
/// `Live` = a practice session is actively running. Coral, pulses —
/// universal "recording / live" affordance.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatusDotState {
    Building,
    Live,
}

/// Tiny status dot — a coloured circle designed to overlay an icon (e.g.
/// the Practice tab's Clock) to signal that a practice session is in
/// flight without changing the underlying icon under the user's finger.
///
/// The caller positions this absolutely inside a `relative` parent. The
/// dot itself doesn't reserve layout space — it floats over the icon's
/// top-right.
///
/// Accessibility: not focusable on its own; rely on the parent affordance
/// to expose state via `aria-label` (e.g. "Practice — session in
/// progress").
///
/// State mapping (#272):
/// - `Building` → amber (`--color-warning`), static. "You've started
///   planning but not yet practising."
/// - `Live` → coral (`--color-danger`), gentle pulse. "Session is running
///   right now." Pulse keyframe is `status-dot-pulse` in input.css and is
///   suppressed under `prefers-reduced-motion`.
#[component]
pub fn StatusDot(state: Signal<Option<StatusDotState>>) -> impl IntoView {
    view! {
        {move || state.get().map(|s| {
            let class = match s {
                // 8px coloured dot with a subtle dark ring so it reads
                // against any icon underneath. Top-right placement
                // (`-top-0.5 -right-0.5`) so the dot sits in the icon's
                // negative space without clipping.
                StatusDotState::Building => "absolute -top-0.5 -right-0.5 \
                    h-2 w-2 rounded-full bg-[var(--color-warning)] \
                    ring-2 ring-[var(--color-bg-gradient-bottom)]",
                StatusDotState::Live => "absolute -top-0.5 -right-0.5 \
                    h-2 w-2 rounded-full bg-[var(--color-danger)] \
                    ring-2 ring-[var(--color-bg-gradient-bottom)] \
                    status-dot--pulse",
            };
            view! {
                <span
                    class=class
                    aria-hidden="true"
                />
            }
        })}
    }
}

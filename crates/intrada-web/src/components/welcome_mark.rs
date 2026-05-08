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

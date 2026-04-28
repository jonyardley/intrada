use std::collections::HashSet;

use chrono::{Datelike, Duration, NaiveDate};
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;

use crate::components::{Icon, IconName};
use intrada_web::haptics::haptic_selection;
use intrada_web::helpers::{day_abbrev, get_month_label, get_week_dates};

/// A single day cell in the week strip.
///
/// Shows the abbreviated day name, date number, optional session indicator dot,
/// and selected state highlight. Uses design tokens only — no raw Tailwind colours.
#[component]
pub fn DayCell(
    date: NaiveDate,
    day_abbrev: &'static str,
    is_selected: bool,
    has_sessions: bool,
    on_click: Callback<NaiveDate>,
) -> impl IntoView {
    let date_number = date.day();
    let aria_label = date.format("%A, %-d %B %Y").to_string();

    // Build class strings from design tokens
    let container_class = if is_selected {
        "flex flex-col items-center justify-center py-1 px-1 rounded-lg cursor-pointer \
         bg-accent-focus/15 ring-2 ring-accent-focus transition-colors min-w-0"
    } else {
        "flex flex-col items-center justify-center py-1 px-1 rounded-lg cursor-pointer \
         hover:bg-surface-hover transition-colors min-w-0"
    };

    let date_text_class = if is_selected {
        "text-xs font-medium text-accent-text"
    } else {
        "text-xs font-medium text-secondary"
    };

    let day_label_class = if is_selected {
        "text-xs text-accent-text"
    } else {
        "text-xs text-muted"
    };

    view! {
        <button
            class=container_class
            role="button"
            aria-label=aria_label
            aria-pressed=is_selected.to_string()
            on:click=move |_| on_click.run(date)
        >
            <span class=day_label_class>{day_abbrev}</span>
            <span class=date_text_class>{date_number}</span>
            {if has_sessions {
                Some(view! {
                    <span class="mt-0.5 w-1.5 h-1.5 rounded-full bg-warm-accent shadow-[0_0_4px_var(--color-warm-accent)]" aria-hidden="true"></span>
                })
            } else {
                // Invisible placeholder to maintain consistent height
                Some(view! {
                    <span class="mt-0.5 w-1.5 h-1.5 rounded-full" aria-hidden="true"></span>
                })
            }}
        </button>
    }
}

/// The full week strip component: month/year label, navigation arrows, and 7 day cells.
///
/// Layout: Card container, header row with left arrow + month label + right arrow,
/// row of 7 evenly distributed DayCell components.
#[component]
pub fn WeekStrip(
    week_start: NaiveDate,
    selected_date: Option<NaiveDate>,
    session_dates: HashSet<NaiveDate>,
    on_day_click: Callback<NaiveDate>,
    on_prev_week: Callback<()>,
    on_next_week: Callback<()>,
    /// Callback to jump back to the current week / today
    on_today: Callback<()>,
    /// Whether the strip is already showing the current week (hides the Today button)
    is_current_week: bool,
) -> impl IntoView {
    let prev_week_start = week_start - Duration::days(7);
    let next_week_start = week_start + Duration::days(7);
    let prev_dates = get_week_dates(prev_week_start);
    let current_dates = get_week_dates(week_start);
    let next_dates = get_week_dates(next_week_start);
    let week_end = current_dates[6];
    let month_label = get_month_label(week_start, week_end);

    /// Minimum movement (px) before we commit to either a horizontal swipe
    /// or a vertical scroll. Below this we don't translate the track —
    /// keeps taps from triggering a sub-pixel drag visual.
    const GESTURE_COMMIT_PX: f64 = 6.0;
    /// Threshold (as fraction of frame width) past which a release commits
    /// to navigating to the adjacent page rather than snapping back.
    const PAGE_SWIPE_RATIO: f64 = 0.25;
    /// Spring animation duration (ms) — kept in sync with the CSS
    /// transition value below so the deferred parent navigation lands
    /// after the visual snap completes.
    const SNAP_DURATION_MS: i32 = 320;

    // 3-page architecture: track holds [prev, current, next] side-by-side,
    // each page exactly 1/3 of the track width = 100% of the visible frame.
    // Default position translates the track so the middle (current) page
    // is centred in the frame.
    //
    // During an active drag, drag_offset (in px) is added to the centred
    // position so the user can see prev/next peeking in from the sides.
    //
    // On release past threshold, snap_target moves the track to the
    // chosen edge (prev or next centered) WITH a transition; once the
    // animation completes we call the parent's navigation callback,
    // which re-renders WeekStrip on the new week_start so its track
    // re-mounts at the centred default with the new "current" content.
    let pointer_start_x = RwSignal::new(0.0_f64);
    let pointer_start_y = RwSignal::new(0.0_f64);
    let frame_width = RwSignal::new(0.0_f64);
    let drag_offset = RwSignal::new(0.0_f64);
    let gesture_committed = RwSignal::new(false);
    // True once the user has moved far enough vertically that we've decided
    // this is a scroll, not a swipe. Stops further drag handling for this
    // gesture without losing the start coordinates.
    let gesture_abandoned = RwSignal::new(false);
    // -1 = snapping left (about to commit prev), 1 = snapping right (next),
    // 0 = idle / dragging.
    let snap_target = RwSignal::new(0i32);

    let frame_ref = NodeRef::<leptos::html::Div>::new();

    let handle_pointer_down = move |ev: PointerEvent| {
        if snap_target.get_untracked() != 0 {
            // Mid-snap animation, ignore — parent will re-mount us shortly.
            return;
        }
        pointer_start_x.set(ev.client_x() as f64);
        pointer_start_y.set(ev.client_y() as f64);
        drag_offset.set(0.0);
        gesture_committed.set(false);
        gesture_abandoned.set(false);
        // Cache the frame width once at gesture start so we can compute
        // the page-snap threshold against it without measuring per-move.
        if let Some(el) = frame_ref.get_untracked() {
            let rect = el.get_bounding_client_rect();
            frame_width.set(rect.width());
        }
    };

    let handle_pointer_move = move |ev: PointerEvent| {
        if snap_target.get_untracked() != 0 || gesture_abandoned.get_untracked() {
            return;
        }
        let dx = ev.client_x() as f64 - pointer_start_x.get_untracked();
        let dy = ev.client_y() as f64 - pointer_start_y.get_untracked();

        if !gesture_committed.get_untracked() {
            if dy.abs() > GESTURE_COMMIT_PX && dy.abs() > dx.abs() {
                gesture_abandoned.set(true);
                return;
            }
            if dx.abs() < GESTURE_COMMIT_PX {
                return;
            }
            gesture_committed.set(true);
        }

        drag_offset.set(dx);
    };

    let handle_pointer_cancel = move |_ev: PointerEvent| {
        if snap_target.get_untracked() != 0 {
            return;
        }
        gesture_committed.set(false);
        gesture_abandoned.set(false);
        drag_offset.set(0.0);
    };

    let handle_pointer_up = move |ev: PointerEvent| {
        if snap_target.get_untracked() != 0 {
            return;
        }
        let dx = ev.client_x() as f64 - pointer_start_x.get_untracked();
        let committed = gesture_committed.get_untracked();
        let abandoned = gesture_abandoned.get_untracked();
        gesture_committed.set(false);
        gesture_abandoned.set(false);

        let frame_w = frame_width.get_untracked();
        let threshold = frame_w * PAGE_SWIPE_RATIO;

        if !abandoned && committed && dx.abs() > threshold && frame_w > 0.0 {
            // Snap to the edge page first — animation drives the visual
            // commit. After SNAP_DURATION_MS we fire the parent callback
            // which re-renders this component on the new week_start.
            let direction = if dx < 0.0 { 1 } else { -1 };
            snap_target.set(direction);
            drag_offset.set(0.0);
            // iOS Calendar fires a subtle selection tick at commit.
            haptic_selection();

            let cb: Closure<dyn Fn()> = Closure::new(move || {
                if direction < 0 {
                    on_prev_week.run(());
                } else {
                    on_next_week.run(());
                }
                // Reset in case we're not unmounted (defensive).
                snap_target.set(0);
            });
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    SNAP_DURATION_MS,
                );
            }
            cb.forget();
        } else {
            // Snap back to centre via CSS transition.
            drag_offset.set(0.0);
        }
    };

    // Inline style for the track. Default centred at -33.333% (one page-
    // width from the left so the middle page is in the frame). During
    // drag, finger-tracked offset added; during snap, full-page shift.
    let track_style = move || {
        let snap = snap_target.get();
        let dx = drag_offset.get();
        let active = gesture_committed.get();

        if snap != 0 {
            // Animating to the chosen edge — let the CSS transition handle.
            let pct = if snap < 0 { 0.0 } else { -66.666 };
            format!("transform: translateX({pct}%);")
        } else if active {
            // 1:1 with finger, no transition.
            format!("transform: translateX(calc(-33.333% + {dx}px)); transition: none;")
        } else {
            // Centred / snapping back to centre.
            String::new()
        }
    };

    view! {
        <div
            class="pb-3 mb-2 border-b border-border-default"
            style="touch-action: pan-y;"
            on:pointerdown=handle_pointer_down
            on:pointermove=handle_pointer_move
            on:pointerup=handle_pointer_up
            on:pointercancel=handle_pointer_cancel
        >
            // Header: arrows + month label + today button.
            // On iOS, chevrons are hidden (swipe is the gesture) and the
            // Today button sheds its pill background to look like a plain
            // accent-text nav bar action — see week-strip.* CSS in input.css.
            <div class="week-strip-header flex items-center justify-between mb-2">
                <button
                    class="week-strip-nav p-1 rounded-lg hover:bg-surface-hover transition-colors text-muted hover:text-secondary"
                    aria-label="Previous week"
                    on:click=move |_| on_prev_week.run(())
                >
                    <Icon name=IconName::ChevronLeft class="w-4 h-4" />
                </button>
                <div class="flex items-center gap-2">
                    <span class="text-xs font-medium text-secondary">{month_label}</span>
                    {if !is_current_week {
                        Some(view! {
                            <button
                                class="week-strip-today text-xs font-medium text-accent-text hover:text-accent-text/80 \
                                       bg-accent-focus/10 hover:bg-accent-focus/20 px-2 py-0.5 rounded-full \
                                       transition-colors"
                                aria-label="Jump to today"
                                on:click=move |_| on_today.run(())
                            >
                                "Today"
                            </button>
                        })
                    } else {
                        None
                    }}
                </div>
                <button
                    class="week-strip-nav p-1 rounded-lg hover:bg-surface-hover transition-colors text-muted hover:text-secondary"
                    aria-label="Next week"
                    on:click=move |_| on_next_week.run(())
                >
                    <Icon name=IconName::ChevronRight class="w-4 h-4" />
                </button>
            </div>

            // 3-page track: [prev | current | next] side-by-side, each page
            // takes the full frame width. Track is translated -33.333% so the
            // current page sits in the visible frame; during drag, the
            // `track_style` closure adds the finger offset; on release past
            // threshold the track snaps to the adjacent page edge, then the
            // parent's nav callback re-mounts WeekStrip on the new week_start.
            <div class="week-strip-frame" node_ref=frame_ref>
                <div class="week-strip-track" style=track_style>
                    <div class="week-strip-page">
                        {prev_dates.into_iter().map(|date| {
                            let abbrev = day_abbrev(date.weekday());
                            let has_sessions = session_dates.contains(&date);
                            view! {
                                <DayCell
                                    date=date
                                    day_abbrev=abbrev
                                    is_selected=false
                                    has_sessions=has_sessions
                                    on_click=on_day_click
                                />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="week-strip-page">
                        {current_dates.into_iter().map(|date| {
                            let abbrev = day_abbrev(date.weekday());
                            let is_selected = selected_date == Some(date);
                            let has_sessions = session_dates.contains(&date);
                            view! {
                                <DayCell
                                    date=date
                                    day_abbrev=abbrev
                                    is_selected=is_selected
                                    has_sessions=has_sessions
                                    on_click=on_day_click
                                />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="week-strip-page">
                        {next_dates.into_iter().map(|date| {
                            let abbrev = day_abbrev(date.weekday());
                            let has_sessions = session_dates.contains(&date);
                            view! {
                                <DayCell
                                    date=date
                                    day_abbrev=abbrev
                                    is_selected=false
                                    has_sessions=has_sessions
                                    on_click=on_day_click
                                />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }
}

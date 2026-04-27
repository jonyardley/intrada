use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};
use leptos::prelude::*;
use web_sys::PointerEvent;

use crate::components::{Icon, IconName};
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
    let dates = get_week_dates(week_start);
    let week_end = dates[6];
    let month_label = get_month_label(week_start, week_end);

    /// Minimum horizontal distance (px) to recognise a swipe gesture.
    const SWIPE_THRESHOLD_PX: f64 = 50.0;

    // Swipe gesture state
    let pointer_start_x = RwSignal::new(0.0_f64);
    let pointer_start_y = RwSignal::new(0.0_f64);

    let handle_pointer_down = move |ev: PointerEvent| {
        pointer_start_x.set(ev.client_x() as f64);
        pointer_start_y.set(ev.client_y() as f64);
    };

    let handle_pointer_up = move |ev: PointerEvent| {
        let dx = ev.client_x() as f64 - pointer_start_x.get_untracked();
        let dy = ev.client_y() as f64 - pointer_start_y.get_untracked();
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        // Only trigger if horizontal swipe exceeds threshold and is more horizontal than vertical
        if abs_dx > SWIPE_THRESHOLD_PX && abs_dx > abs_dy {
            if dx < 0.0 {
                // Swipe left → next week
                on_next_week.run(());
            } else {
                // Swipe right → previous week
                on_prev_week.run(());
            }
        }
    };

    view! {
        <div
            class="pb-3 mb-2 border-b border-border-default"
            style="touch-action: pan-y;"
            on:pointerdown=handle_pointer_down
            on:pointerup=handle_pointer_up
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

            // Day cells row
            <div class="grid grid-cols-7 gap-1">
                {dates.into_iter().map(|date| {
                    let abbrev = day_abbrev(date.weekday());
                    let is_selected = selected_date == Some(date);
                    let has_sessions = session_dates.contains(&date);
                    let on_click = on_day_click;
                    view! {
                        <DayCell
                            date=date
                            day_abbrev=abbrev
                            is_selected=is_selected
                            has_sessions=has_sessions
                            on_click=on_click
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

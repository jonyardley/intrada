//! Tempo Progress Chart — SVG line chart showing achieved tempo over time
//! with an optional target BPM reference line and progress percentage.
//!
//! Consumes `TempoHistoryEntry` data from `ItemPracticeSummary` (tempo tracking #52).
//! Follows the same SVG viewBox pattern as `LineChart` but adapted for BPM data.

use intrada_core::TempoHistoryEntry;
use leptos::prelude::*;

// ── Data preparation helpers ────────────────────────────────────────────────

/// Parse a numeric BPM value from the item's tempo field.
///
/// Handles formats like `"120 BPM"`, `"120"`, `"♩ = 120"`, extracting the
/// first sequence of digits found. Returns `None` if no digits are present.
pub fn parse_target_bpm(tempo: &Option<String>) -> Option<u16> {
    tempo.as_ref().and_then(|t| {
        t.chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u16>()
            .ok()
    })
}

/// Compute the Y-axis range (min, max) for the chart, rounded to the nearest
/// 10 BPM with padding. Includes the target BPM in the range if present.
///
/// Returns `(y_min, y_max)` where both are multiples of 10, ensuring the data
/// and target line are visible within the chart area.
fn compute_y_range(entries: &[TempoHistoryEntry], target: Option<u16>) -> (u16, u16) {
    let data_min = entries.iter().map(|e| e.tempo).min().unwrap_or(60);
    let data_max = entries.iter().map(|e| e.tempo).max().unwrap_or(120);

    // Include target in range calculation
    let range_min = target.map_or(data_min, |t| data_min.min(t));
    let range_max = target.map_or(data_max, |t| data_max.max(t));

    // Use u32 for arithmetic to prevent u16 overflow on extreme values
    let range_min = range_min as u32;
    let range_max = range_max as u32;

    // Round down to nearest 10 with 10 BPM padding below
    let y_min = (range_min.saturating_sub(10)) / 10 * 10;
    // Round up to nearest 10 with 10 BPM padding above
    let y_max = ((range_max + 19) / 10) * 10;

    // Ensure minimum range of 20 BPM for visual clarity
    let (y_min, y_max) = if y_max - y_min < 20 {
        (y_min, y_min + 20)
    } else {
        (y_min, y_max)
    };

    (y_min as u16, y_max as u16)
}

/// Format an RFC3339 date string for X-axis chart labels.
///
/// Extracts month and day, producing labels like "Jan 15", "Feb 3".
fn format_chart_date(rfc3339: &str) -> String {
    // Extract "YYYY-MM-DD" portion
    let date_part = rfc3339.split('T').next().unwrap_or(rfc3339);
    let parts: Vec<&str> = date_part.split('-').collect();

    if parts.len() >= 3 {
        let month = match parts[1] {
            "01" => "Jan",
            "02" => "Feb",
            "03" => "Mar",
            "04" => "Apr",
            "05" => "May",
            "06" => "Jun",
            "07" => "Jul",
            "08" => "Aug",
            "09" => "Sep",
            "10" => "Oct",
            "11" => "Nov",
            "12" => "Dec",
            _ => parts[1],
        };
        // Strip leading zero from day
        let day = parts[2].trim_start_matches('0');
        format!("{} {}", month, day)
    } else {
        date_part.to_string()
    }
}

// ── Component ───────────────────────────────────────────────────────────────

/// A line chart that visualises tempo progress over time for a single library item.
///
/// Shows achieved BPM values as connected data points with an optional dashed
/// reference line at the target BPM. Includes a progress percentage when both
/// target and latest tempo are available.
#[component]
pub fn TempoProgressChart(
    /// Tempo history entries (expected in descending date order from the API).
    entries: Vec<TempoHistoryEntry>,
    /// Target BPM parsed from the item's tempo field (e.g., "120 BPM" → Some(120)).
    target_bpm: Option<u16>,
    /// Most recent achieved tempo for progress percentage calculation.
    latest_tempo: Option<u16>,
) -> impl IntoView {
    // Empty state: render nothing when no data
    if entries.is_empty() {
        return ().into_any();
    }

    // Reverse to chronological order (oldest first, left-to-right)
    let mut data: Vec<TempoHistoryEntry> = entries;
    data.reverse();

    let (y_min, y_max) = compute_y_range(&data, target_bpm);
    let y_range = (y_max - y_min) as f64;

    // Chart dimensions within viewBox (matches LineChart pattern)
    let width: f64 = 600.0;
    let height: f64 = 200.0;
    let padding_left: f64 = 45.0; // Slightly wider for "120" BPM labels
    let padding_right: f64 = 10.0;
    let padding_top: f64 = 10.0;
    let padding_bottom: f64 = 25.0;

    let chart_width = width - padding_left - padding_right;
    let chart_height = height - padding_top - padding_bottom;

    let n = data.len() as f64;
    let step_x = if n > 1.0 {
        chart_width / (n - 1.0)
    } else {
        0.0
    };

    // Map BPM value to Y coordinate (uses f64 to avoid u16 subtraction risk)
    let bpm_to_y = |bpm: u16| -> f64 {
        let ratio = (bpm as f64 - y_min as f64) / y_range;
        padding_top + chart_height - (ratio * chart_height)
    };

    // Build polyline points string
    let points: String = data
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let x = padding_left + i as f64 * step_x;
            let y = bpm_to_y(entry.tempo);
            format!("{:.1},{:.1}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Density-adaptive circle parameters (R6)
    let point_count = data.len();
    let show_visible_circles = point_count < 100;
    let circle_radius: f64 = if point_count < 50 {
        3.0
    } else if point_count < 100 {
        2.0
    } else {
        6.0 // invisible hit targets
    };
    let circle_fill = if show_visible_circles {
        "var(--color-chart-line)"
    } else {
        "transparent"
    };
    let circle_stroke = if show_visible_circles {
        "var(--color-chart-point-stroke)"
    } else {
        "none"
    };
    let circle_stroke_width = if show_visible_circles { "1.5" } else { "0" };

    // Data point circles with tooltips
    let circles: Vec<(f64, f64, u16, String)> = data
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let x = padding_left + i as f64 * step_x;
            let y = bpm_to_y(entry.tempo);
            let date_label = format_chart_date(&entry.session_date);
            (x, y, entry.tempo, date_label)
        })
        .collect();

    // Y-axis labels (min, mid, max)
    let y_mid = y_min + (y_max - y_min) / 2;
    let y_labels = vec![
        (bpm_to_y(y_min), format!("{}", y_min)),
        (bpm_to_y(y_mid), format!("{}", y_mid)),
        (bpm_to_y(y_max), format!("{}", y_max)),
    ];

    // X-axis date labels — show a sensible subset to avoid overlap
    let max_labels = 6;
    let last_idx = data.len() - 1;
    let label_step = if data.len() > max_labels {
        data.len() / max_labels
    } else {
        1
    };
    let x_labels: Vec<(f64, String)> = data
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            if *i == 0 || *i == last_idx {
                return true;
            }
            if label_step > 0 && *i % label_step == 0 {
                // Skip if too close to the forced last label to prevent overlap
                return last_idx - *i >= label_step / 2 + 1;
            }
            false
        })
        .map(|(i, entry)| {
            let x = padding_left + i as f64 * step_x;
            (x, format_chart_date(&entry.session_date))
        })
        .collect();

    // Target reference line Y position
    let target_y = target_bpm.map(bpm_to_y);

    // Progress percentage
    let progress_pct = match (latest_tempo, target_bpm) {
        (Some(latest), Some(target)) if target > 0 => {
            Some(((latest as f64 / target as f64) * 100.0).round() as u32)
        }
        _ => None,
    };

    let aria_label = format!(
        "Tempo progress chart showing {} data point{}",
        point_count,
        if point_count == 1 { "" } else { "s" }
    );

    let single_point = point_count == 1;

    view! {
        <div>
            // Progress percentage (US2)
            {progress_pct.map(|pct| {
                let target = target_bpm.unwrap_or(0);
                view! {
                    <p class="text-sm text-secondary mb-2">
                        <span class="text-accent-text font-semibold">
                            {format!("\u{266A} {}% of target", pct)}
                        </span>
                        <span class="text-muted ml-1">
                            {format!("({} BPM)", target)}
                        </span>
                    </p>
                }
            })}

            // SVG chart
            <svg
                viewBox="0 0 600 200"
                class="w-full h-auto"
                role="img"
                aria-label={aria_label}
                xmlns="http://www.w3.org/2000/svg"
            >
                // Grid lines — left axis
                <line
                    x1={format!("{:.1}", padding_left)}
                    y1={format!("{:.1}", padding_top)}
                    x2={format!("{:.1}", padding_left)}
                    y2={format!("{:.1}", padding_top + chart_height)}
                    stroke="var(--color-chart-grid)"
                    stroke-width="1"
                />
                // Grid lines — bottom axis
                <line
                    x1={format!("{:.1}", padding_left)}
                    y1={format!("{:.1}", padding_top + chart_height)}
                    x2={format!("{:.1}", width - padding_right)}
                    y2={format!("{:.1}", padding_top + chart_height)}
                    stroke="var(--color-chart-grid)"
                    stroke-width="1"
                />
                // Mid grid line (dashed)
                <line
                    x1={format!("{:.1}", padding_left)}
                    y1={format!("{:.1}", bpm_to_y(y_mid))}
                    x2={format!("{:.1}", width - padding_right)}
                    y2={format!("{:.1}", bpm_to_y(y_mid))}
                    stroke="var(--color-chart-grid-mid)"
                    stroke-width="1"
                    stroke-dasharray="4,4"
                />

                // Target reference line (US1 — dashed, warm gold)
                {target_y.map(|ty| {
                    view! {
                        <line
                            x1={format!("{:.1}", padding_left)}
                            y1={format!("{:.1}", ty)}
                            x2={format!("{:.1}", width - padding_right - 40.0)}
                            y2={format!("{:.1}", ty)}
                            stroke="var(--color-chart-secondary)"
                            stroke-width="1.5"
                            stroke-dasharray="6,4"
                        />
                        <text
                            x={format!("{:.1}", width - padding_right - 2.0)}
                            y={format!("{:.1}", ty + 3.0)}
                            text-anchor="end"
                            font-size="9"
                            fill="var(--color-chart-secondary)"
                        >
                            "Target"
                        </text>
                    }
                })}

                // Data line (polyline) — skip if single point
                {if !single_point {
                    Some(view! {
                        <polyline
                            points={points}
                            fill="none"
                            stroke="var(--color-chart-line)"
                            stroke-width="2"
                            stroke-linejoin="round"
                            stroke-linecap="round"
                        />
                    })
                } else {
                    None
                }}

                // Data point circles with tooltips (US3)
                // Density-adaptive: visible circles <100 pts, invisible hit targets ≥100
                {circles.into_iter().map(|(x, y, bpm, date_label)| {
                    view! {
                        <circle
                            cx={format!("{:.1}", x)}
                            cy={format!("{:.1}", y)}
                            r={format!("{:.0}", circle_radius)}
                            fill={circle_fill}
                            stroke={circle_stroke}
                            stroke-width={circle_stroke_width}
                        >
                            <title>{format!("{} \u{2014} {} BPM", date_label, bpm)}</title>
                        </circle>
                    }
                }).collect::<Vec<_>>()}

                // Y-axis labels
                {y_labels.into_iter().map(|(y, label)| {
                    view! {
                        <text
                            x={format!("{:.1}", padding_left - 5.0)}
                            y={format!("{:.1}", y + 3.0)}
                            text-anchor="end"
                            font-size="10"
                            fill="var(--color-chart-label)"
                        >
                            {label}
                        </text>
                    }
                }).collect::<Vec<_>>()}

                // X-axis date labels
                {x_labels.into_iter().map(|(x, label)| {
                    view! {
                        <text
                            x={format!("{:.1}", x)}
                            y={format!("{:.1}", padding_top + chart_height + 15.0)}
                            text-anchor="middle"
                            font-size="9"
                            fill="var(--color-chart-label)"
                        >
                            {label}
                        </text>
                    }
                }).collect::<Vec<_>>()}
            </svg>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(date: &str, bpm: u16) -> TempoHistoryEntry {
        TempoHistoryEntry {
            session_date: date.to_string(),
            tempo: bpm,
            session_id: "s".to_string(),
        }
    }

    // ── parse_target_bpm ─────────────────────────────────────────────

    #[test]
    fn parse_target_bpm_standard() {
        assert_eq!(parse_target_bpm(&Some("120 BPM".into())), Some(120));
    }

    #[test]
    fn parse_target_bpm_bare_number() {
        assert_eq!(parse_target_bpm(&Some("80".into())), Some(80));
    }

    #[test]
    fn parse_target_bpm_metronome_marking() {
        assert_eq!(parse_target_bpm(&Some("♩ = 132".into())), Some(132));
    }

    #[test]
    fn parse_target_bpm_lowercase() {
        assert_eq!(parse_target_bpm(&Some("100 bpm".into())), Some(100));
    }

    #[test]
    fn parse_target_bpm_no_digits() {
        assert_eq!(parse_target_bpm(&Some("allegro".into())), None);
    }

    #[test]
    fn parse_target_bpm_empty_string() {
        assert_eq!(parse_target_bpm(&Some(String::new())), None);
    }

    #[test]
    fn parse_target_bpm_none() {
        assert_eq!(parse_target_bpm(&None), None);
    }

    // ── compute_y_range ──────────────────────────────────────────────

    #[test]
    fn y_range_basic() {
        let entries = vec![
            entry("2026-01-01T00:00:00Z", 80),
            entry("2026-01-02T00:00:00Z", 100),
        ];
        let (lo, hi) = compute_y_range(&entries, None);
        assert!(lo <= 80, "lo={lo} should be <= 80");
        assert!(hi >= 100, "hi={hi} should be >= 100");
        assert!(hi - lo >= 20, "range should be at least 20");
    }

    #[test]
    fn y_range_includes_target() {
        let entries = vec![entry("2026-01-01T00:00:00Z", 80)];
        let (_lo, hi) = compute_y_range(&entries, Some(120));
        assert!(hi >= 120, "hi={hi} should include target 120");
    }

    #[test]
    fn y_range_minimum_20() {
        let entries = vec![entry("2026-01-01T00:00:00Z", 100)];
        let (lo, hi) = compute_y_range(&entries, None);
        assert!(hi - lo >= 20, "range {lo}..{hi} should be at least 20");
    }

    #[test]
    fn y_range_aligned_to_10() {
        let entries = vec![
            entry("2026-01-01T00:00:00Z", 73),
            entry("2026-01-02T00:00:00Z", 97),
        ];
        let (lo, hi) = compute_y_range(&entries, None);
        assert_eq!(lo % 10, 0, "lo={lo} should be a multiple of 10");
        assert_eq!(hi % 10, 0, "hi={hi} should be a multiple of 10");
    }

    #[test]
    fn y_range_no_overflow_high_bpm() {
        let entries = vec![entry("2026-01-01T00:00:00Z", 300)];
        let (lo, hi) = compute_y_range(&entries, Some(350));
        assert!(hi >= 350);
        assert!(lo <= 300);
    }

    // ── format_chart_date ────────────────────────────────────────────

    #[test]
    fn format_rfc3339() {
        assert_eq!(format_chart_date("2026-02-03T10:30:00Z"), "Feb 3");
    }

    #[test]
    fn format_date_only() {
        assert_eq!(format_chart_date("2026-12-25"), "Dec 25");
    }

    #[test]
    fn format_strips_leading_zero() {
        assert_eq!(format_chart_date("2026-01-05T00:00:00Z"), "Jan 5");
    }

    #[test]
    fn format_keeps_double_digit_day() {
        assert_eq!(format_chart_date("2026-03-15"), "Mar 15");
    }

    #[test]
    fn format_day_ten() {
        assert_eq!(format_chart_date("2026-06-10"), "Jun 10");
    }
}

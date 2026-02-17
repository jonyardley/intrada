use intrada_core::analytics::DailyPracticeTotal;
use leptos::prelude::*;

/// A responsive SVG line chart for displaying daily practice minutes.
///
/// Uses a fixed viewBox (`0 0 600 200`) with CSS responsive scaling.
/// Renders a `<polyline>` for the line, `<circle>` elements for data points,
/// and axis labels for day markers and minute scale.
#[component]
pub fn LineChart(data: Vec<DailyPracticeTotal>) -> impl IntoView {
    // Chart dimensions within viewBox
    let width: f64 = 600.0;
    let height: f64 = 200.0;
    let padding_left: f64 = 40.0;
    let padding_right: f64 = 10.0;
    let padding_top: f64 = 10.0;
    let padding_bottom: f64 = 25.0;

    let chart_width = width - padding_left - padding_right;
    let chart_height = height - padding_top - padding_bottom;

    let max_minutes = data.iter().map(|d| d.minutes).max().unwrap_or(1).max(1);
    // Round up to nearest 10 for clean y-axis
    let y_max = (max_minutes.div_ceil(10) * 10).max(10) as f64;

    let n = data.len() as f64;
    let step_x = if n > 1.0 { chart_width / (n - 1.0) } else { 0.0 };

    // Build polyline points string
    let points: String = data
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let x = padding_left + i as f64 * step_x;
            let y = padding_top + chart_height - (d.minutes as f64 / y_max * chart_height);
            format!("{:.1},{:.1}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Build filled area polygon (line + bottom)
    let area_points = {
        let first_x = padding_left;
        let last_x = padding_left + (data.len() as f64 - 1.0) * step_x;
        let bottom_y = padding_top + chart_height;
        format!(
            "{:.1},{:.1} {} {:.1},{:.1}",
            first_x, bottom_y, points, last_x, bottom_y
        )
    };

    // Data point circles
    let circles: Vec<(f64, f64, u32, String)> = data
        .iter()
        .enumerate()
        .filter(|(_, d)| d.minutes > 0)
        .map(|(i, d)| {
            let x = padding_left + i as f64 * step_x;
            let y = padding_top + chart_height - (d.minutes as f64 / y_max * chart_height);
            (x, y, d.minutes, d.date.clone())
        })
        .collect();

    // Y-axis labels (0, mid, max)
    let y_mid = y_max / 2.0;
    let y_labels = vec![
        (padding_top + chart_height, "0".to_string()),
        (
            padding_top + chart_height / 2.0,
            format!("{}m", y_mid as u32),
        ),
        (padding_top, format!("{}m", y_max as u32)),
    ];

    // X-axis day labels (every 7th day)
    let x_labels: Vec<(f64, String)> = data
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 7 == 0 || *i == data.len() - 1)
        .map(|(i, d)| {
            let x = padding_left + i as f64 * step_x;
            // Extract day part (dd) from "YYYY-MM-DD"
            let label = d
                .date
                .split('-')
                .nth(2)
                .unwrap_or(&d.date)
                .to_string();
            (x, label)
        })
        .collect();

    view! {
        <svg
            viewBox="0 0 600 200"
            class="w-full h-auto"
            role="img"
            aria-label="Practice history chart showing daily minutes over 28 days"
            xmlns="http://www.w3.org/2000/svg"
        >
            // Grid lines
            <line
                x1={format!("{:.1}", padding_left)}
                y1={format!("{:.1}", padding_top)}
                x2={format!("{:.1}", padding_left)}
                y2={format!("{:.1}", padding_top + chart_height)}
                stroke="rgba(255,255,255,0.1)"
                stroke-width="1"
            />
            <line
                x1={format!("{:.1}", padding_left)}
                y1={format!("{:.1}", padding_top + chart_height)}
                x2={format!("{:.1}", width - padding_right)}
                y2={format!("{:.1}", padding_top + chart_height)}
                stroke="rgba(255,255,255,0.1)"
                stroke-width="1"
            />
            // Mid grid line
            <line
                x1={format!("{:.1}", padding_left)}
                y1={format!("{:.1}", padding_top + chart_height / 2.0)}
                x2={format!("{:.1}", width - padding_right)}
                y2={format!("{:.1}", padding_top + chart_height / 2.0)}
                stroke="rgba(255,255,255,0.05)"
                stroke-width="1"
                stroke-dasharray="4,4"
            />

            // Filled area under the line
            <polygon
                points={area_points}
                fill="rgba(129, 140, 248, 0.15)"
            />

            // Line
            <polyline
                points={points}
                fill="none"
                stroke="rgb(129, 140, 248)"
                stroke-width="2"
                stroke-linejoin="round"
                stroke-linecap="round"
            />

            // Data point circles
            {circles.into_iter().map(|(x, y, minutes, date)| {
                view! {
                    <circle
                        cx={format!("{:.1}", x)}
                        cy={format!("{:.1}", y)}
                        r="3"
                        fill="rgb(129, 140, 248)"
                        stroke="rgb(30, 27, 75)"
                        stroke-width="1.5"
                    >
                        <title>{format!("{}: {}m", date, minutes)}</title>
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
                        fill="rgba(156, 163, 175, 0.7)"
                    >
                        {label}
                    </text>
                }
            }).collect::<Vec<_>>()}

            // X-axis labels
            {x_labels.into_iter().map(|(x, label)| {
                view! {
                    <text
                        x={format!("{:.1}", x)}
                        y={format!("{:.1}", padding_top + chart_height + 15.0)}
                        text-anchor="middle"
                        font-size="10"
                        fill="rgba(156, 163, 175, 0.7)"
                    >
                        {label}
                    </text>
                }
            }).collect::<Vec<_>>()}
        </svg>
    }
}

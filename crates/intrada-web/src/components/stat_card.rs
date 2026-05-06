use leptos::prelude::*;

use crate::components::accent_row::AccentBar;

/// Tone for the StatCard value text. Pairs with the inset accent-bar
/// variant so a stat's category reads at a glance — e.g. streak in
/// accent purple, weekly hours in warm gold, generic count in white.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatTone {
    #[default]
    Default,
    Accent,
    WarmAccent,
}

/// A small stat display card for analytics metrics (e.g., streak, weekly total).
///
/// Two visual modes:
/// - **Classic** (`bar=AccentBar::None` or omit): plain `.card` chrome
///   (the 2026 surface — whisper-soft + subtle shadow) with compact
///   padding. Used everywhere today.
/// - **2026 refresh** (`bar=AccentBar::Gold | Blue`): same surface plus
///   a 4px gradient bar inset on the left and the value text in the
///   matching tone. Used by the new Piece Detail stat row.
#[component]
pub fn StatCard(
    title: &'static str,
    #[prop(into)] value: String,
    #[prop(optional)] subtitle: Option<&'static str>,
    /// Optional inset gradient bar — opt in to the 2026 refresh variant.
    #[prop(optional)]
    bar: Option<AccentBar>,
    /// Optional tone for the value text. Defaults to white. Only takes
    /// effect in the refresh variant (when `bar` is set).
    #[prop(optional)]
    tone: StatTone,
) -> impl IntoView {
    if let Some(bar) = bar {
        // 2026 refresh variant
        let card_class = format!(
            "stat-card-faint{}",
            match bar {
                AccentBar::Gold => "",
                AccentBar::Blue => " stat-card-faint--blue",
                AccentBar::Teal => " stat-card-faint--teal",
                AccentBar::None => " stat-card-faint--no-bar",
            }
        );
        let value_class = format!(
            "stat-card-faint-value{}",
            match tone {
                StatTone::Default => "",
                StatTone::Accent => " stat-card-faint-value--accent",
                StatTone::WarmAccent => " stat-card-faint-value--warm",
            }
        );
        view! {
            <div class=card_class>
                <p class=value_class>{value}</p>
                <p class="stat-card-faint-label">{title}</p>
                {subtitle.map(|s| view! {
                    <p class="text-xs text-muted mt-0.5">{s}</p>
                })}
            </div>
        }
        .into_any()
    } else {
        // Classic variant — unchanged from previous design system
        view! {
            <div class="card p-card-compact text-center">
                <p class="field-label">{title}</p>
                <p class="text-2xl font-bold text-primary mt-1">{value}</p>
                {subtitle.map(|s| view! {
                    <p class="text-xs text-muted mt-0.5">{s}</p>
                })}
            </div>
        }
        .into_any()
    }
}

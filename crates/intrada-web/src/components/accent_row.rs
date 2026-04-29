use leptos::prelude::*;

/// Which accent bar (the 4px gradient inset on the left edge) to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccentBar {
    /// Gold gradient — maps to "piece" content in the 2026 design language.
    #[default]
    Gold,
    /// Blue gradient — maps to "exercise" content.
    Blue,
    /// No bar — for uniform-type lists where the bar would add noise.
    None,
}

impl AccentBar {
    fn modifier(self) -> &'static str {
        match self {
            Self::Gold => "",
            Self::Blue => " accent-row--blue",
            Self::None => " accent-row--no-bar",
        }
    }
}

/// List-row primitive from the 2026 refresh. Renders a 60px-tall card
/// with an optional 4px gradient bar inset on the left edge — the
/// signature visual element of the new design.
///
/// Use sparingly: bars earn their keep on **mixed-type lists** (library,
/// setlist builder) and detail-page sections. For uniform lists (session
/// history, all routines), pass `bar=AccentBar::None` so the bars don't
/// flatten into noise.
#[component]
pub fn AccentRow(
    children: Children,
    /// Which gradient to render in the inset bar (or `None` to omit it).
    #[prop(optional)]
    bar: AccentBar,
) -> impl IntoView {
    let class = format!("accent-row{}", bar.modifier());
    view! {
        <div class=class>
            {children()}
        </div>
    }
}

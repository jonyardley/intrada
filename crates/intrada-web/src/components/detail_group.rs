use leptos::prelude::*;

use crate::components::{accent_row::AccentBar, SectionLabel};

/// Card containing a SectionLabel + grouped rows + the inset 4px
/// gradient bar. The signature container of the 2026 refresh —
/// used for DETAILS / RECENT SESSIONS / NOTES on the Piece Detail
/// page and analogous surfaces.
///
/// Composes `<SectionLabel>` internally so callers don't need to
/// remember the label-then-rows layout.
#[component]
pub fn DetailGroup(
    /// Section label — rendered uppercase above the rows.
    #[prop(into)]
    label: String,
    /// Which gradient to render in the inset bar (or `None` to omit it).
    #[prop(optional)]
    bar: AccentBar,
    /// The rows / content of the group.
    children: Children,
) -> impl IntoView {
    let class = format!(
        "detail-group{}",
        match bar {
            AccentBar::Gold => "",
            AccentBar::Blue => " detail-group--blue",
            AccentBar::Teal => " detail-group--teal",
            AccentBar::None => " detail-group--no-bar",
        }
    );
    view! {
        <div class=class>
            <SectionLabel text=label />
            {children()}
        </div>
    }
}

/// A label/value row inside a `<DetailGroup>` — pairs a description on
/// the left with a value on the right. Used by the "Difficulty / Key /
/// Time Signature / Added" rows in the Piece Detail design.
#[component]
pub fn DetailRow(
    /// Left-hand description (the field name).
    #[prop(into)]
    label: String,
    /// Right-hand value. `Children` so callers can pass styled inline
    /// content (e.g. a key signature with a flat-symbol glyph) rather
    /// than a plain string.
    children: Children,
) -> impl IntoView {
    view! {
        <div class="detail-row">
            <span class="detail-row-label">{label}</span>
            <span class="detail-row-value">{children()}</span>
        </div>
    }
}

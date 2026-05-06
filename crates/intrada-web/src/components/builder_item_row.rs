use leptos::prelude::*;

use intrada_core::{ItemKind, LibraryItemView};
use intrada_web::types::ItemType;

use crate::components::{AccentBar, AccentRow, Icon, IconName, InlineTypeIndicator};

/// Library row used inside the session builder: title + subtitle + type badge,
/// with a toggle on the right that switches between "+ add" (idle) and
/// "✓ added" (already in the setlist). Whole row is clickable.
///
/// Visually mirrors `LibraryItemCard` (AccentRow with type-coloured bar) but
/// has no swipe actions or detail-link affordance — clicks toggle setlist
/// membership instead.
#[component]
pub fn BuilderItemRow(
    item: LibraryItemView,
    is_selected: Signal<bool>,
    on_toggle: Callback<String>,
) -> impl IntoView {
    let bar = match item.item_type {
        ItemKind::Piece => AccentBar::Gold,
        ItemKind::Exercise => AccentBar::Blue,
    };
    let item_id = item.id.clone();
    let title = item.title.clone();
    let subtitle = item.subtitle.clone();
    let inline_type = match item.item_type {
        ItemKind::Piece => ItemType::Piece,
        ItemKind::Exercise => ItemType::Exercise,
    };
    let aria_label = format!("{} ({})", title, item.item_type);

    view! {
        <button
            type="button"
            class="builder-row"
            aria-label=aria_label
            aria-pressed=move || is_selected.get().to_string()
            on:click={
                let id = item_id.clone();
                move |_| on_toggle.run(id.clone())
            }
        >
            <AccentRow bar=bar>
                <div class="flex-1 min-w-0 text-left">
                    <div class="text-sm font-medium text-primary truncate">{title}</div>
                    {(!subtitle.is_empty()).then(|| view! {
                        <div class="text-xs text-muted truncate">{subtitle.clone()}</div>
                    })}
                </div>
                <InlineTypeIndicator item_type=inline_type />
                <span class="builder-row-toggle" aria-hidden="true">
                    {move || {
                        if is_selected.get() {
                            view! {
                                <Icon name=IconName::CheckCircle class="w-5 h-5 text-accent-text" />
                            }.into_any()
                        } else {
                            view! {
                                <Icon name=IconName::Plus class="w-5 h-5 text-muted" />
                            }.into_any()
                        }
                    }}
                </span>
            </AccentRow>
        </button>
    }
}

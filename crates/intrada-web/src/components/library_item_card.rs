use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{ItemKind, LibraryItemView};

use crate::components::{
    AccentBar, AccentRow, ContextMenu, ContextMenuAction, Icon, IconName, InlineTypeIndicator,
    SwipeActions,
};
use intrada_web::types::ItemType;

/// A single library list row.
///
/// 2026 refresh shape: 60px AccentRow with the gradient bar mapped to
/// item type — gold for pieces, blue for exercises. Title + composer
/// (subtitle) on the left, InlineTypeIndicator + chevron on the right.
/// The richer metadata (key, tempo, tags) that used to render in this
/// row now lives on the detail page so the list reads at a glance.
#[component]
pub fn LibraryItemCard(
    item: LibraryItemView,
    /// Optional swipe-to-delete callback. When provided (typically in the
    /// library list on iOS), wraps the row in a SwipeActions container
    /// that reveals a trailing Delete action on left-swipe.
    #[prop(optional, into)]
    on_delete: Option<Callback<String>>,
) -> impl IntoView {
    let id = item.id.clone();
    let title = item.title.clone();
    let subtitle = item.subtitle.clone();
    let item_kind = item.item_type;

    let (bar, indicator_type) = match item_kind {
        ItemKind::Piece => (AccentBar::Gold, ItemType::Piece),
        ItemKind::Exercise => (AccentBar::Blue, ItemType::Exercise),
    };

    let href = format!("/library/{id}");
    let id_for_swipe = id.clone();
    let id_for_menu_delete = id.clone();
    let edit_href = format!("/library/{id}/edit");

    let row = view! {
        <A href=href attr:class="block no-underline">
            <AccentRow bar=bar>
                <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                    <span class="text-sm font-semibold text-primary truncate">{title}</span>
                    {if !subtitle.is_empty() {
                        Some(view! {
                            <span class="text-xs text-muted truncate">{subtitle}</span>
                        })
                    } else {
                        None
                    }}
                </div>
                <InlineTypeIndicator item_type=indicator_type />
                <Icon name=IconName::ChevronRight class="w-4 h-4 text-faint shrink-0" />
            </AccentRow>
        </A>
    };

    if let Some(cb) = on_delete {
        let cb_for_menu = cb;
        let menu_actions = vec![
            ContextMenuAction {
                label: "Edit".to_string(),
                destructive: false,
                on_select: Callback::new(move |_| {
                    let navigate = use_navigate();
                    navigate(&edit_href, NavigateOptions::default());
                }),
            },
            ContextMenuAction {
                label: "Delete".to_string(),
                destructive: true,
                on_select: Callback::new(move |_| {
                    cb_for_menu.run(id_for_menu_delete.clone());
                }),
            },
        ];

        view! {
            <li>
                <ContextMenu actions=menu_actions>
                    <SwipeActions on_delete=Callback::new(move |_| cb.run(id_for_swipe.clone()))>
                        {row}
                    </SwipeActions>
                </ContextMenu>
            </li>
        }
        .into_any()
    } else {
        view! {
            <li>{row}</li>
        }
        .into_any()
    }
}

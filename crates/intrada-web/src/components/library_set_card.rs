use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::SetView;

use crate::components::{
    AccentBar, AccentRow, ContextMenu, ContextMenuAction, Icon, IconName, SwipeActions,
};

/// A single library list row for a Set.
///
/// Sibling primitive to `<LibraryItemCard>` — same 60px AccentRow chrome,
/// but with the Teal bar (signalling Set content as distinct from gold
/// for Pieces / blue for Exercises). Title is the Set name, subtitle is
/// "X items" (no time estimate — Sets are recipes; durations come from
/// session-time defaults).
///
/// Tapping navigates to the Set Detail surface (`/library/sets/:id`)
/// for review-then-start. The context-menu Edit action goes to the
/// edit form (`/routines/:id/edit`) — that route stays as-is until the
/// fold-into-Library URL migration ships in a later PR.
#[component]
pub fn LibrarySetCard(
    set: SetView,
    /// Optional swipe-to-delete callback. When provided, wraps the row
    /// in a SwipeActions container that reveals a trailing Delete on
    /// left-swipe (matches LibraryItemCard).
    #[prop(optional, into)]
    on_delete: Option<Callback<String>>,
) -> impl IntoView {
    let id = set.id.clone();
    let name = set.name.clone();
    let entry_count = set.entry_count;
    let count_label = if entry_count == 1 {
        "1 item".to_string()
    } else {
        format!("{entry_count} items")
    };

    let href = format!("/library/sets/{id}");
    let id_for_swipe = id.clone();
    let id_for_menu_delete = id.clone();
    let edit_href = format!("/routines/{id}/edit");

    let row = view! {
        <A href=href attr:class="block no-underline">
            <AccentRow bar=AccentBar::Teal>
                <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                    <span class="text-sm font-semibold text-primary truncate">{name}</span>
                    <span class="text-xs text-muted truncate">{count_label}</span>
                </div>
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

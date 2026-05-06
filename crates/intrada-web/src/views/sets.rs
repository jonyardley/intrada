use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, ItemKind, SetEvent, SetView, ViewModel};

use crate::components::{
    AccentBar, AccentRow, ContextMenu, ContextMenuAction, EmptyState, Icon, IconName,
    PageAddButton, PageHeading, SkeletonCardList, SwipeActions,
};
use intrada_web::core_bridge::process_effects_with_core;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Management page for saved sets — lists all sets with edit/delete actions.
#[component]
pub fn SetsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <PageHeading
                text="Sets"
                subtitle="Save and reuse your favourite practice structures."
                trailing=Box::new(move || view! {
                    // Trailing + action mirrors the Library page heading.
                    // Navigates to the session builder (where sets are
                    // born — see the Save-as-Set flow).
                    <PageAddButton href="/sessions/new" aria_label="New Set" />
                }.into_any())
            />

            {move || {
                if is_loading.get() {
                    return view! {
                        <SkeletonCardList count=2 />
                    }.into_any();
                }

                let vm = view_model.get();

                if vm.sets.is_empty() {
                    view! {
                        <EmptyState
                            icon=IconName::ListChecks
                            title="No saved sets yet"
                            body="Save a setlist as a set when building a session or from the session summary."
                        >
                            <A href="/sessions/new" attr:class="cta-link">
                                "New Session"
                            </A>
                        </EmptyState>
                    }.into_any()
                } else {
                    view! {
                        <ul class="space-y-2 list-none p-0" role="list" aria-label="Saved sets">
                            {vm.sets.into_iter().map(|set| view! {
                                <li>
                                    <SetRow set=set />
                                </li>
                            }).collect::<Vec<_>>()}
                        </ul>
                        // Inline "add another" link mirrors the Pencil design — a
                        // discrete text link rather than a second hero button so the
                        // primary CTA stays in the header / empty state.
                        <div class="mt-4">
                            <A
                                href="/sessions/new"
                                attr:class="text-sm font-medium text-accent-text hover:text-accent-hover"
                            >
                                "Create New Set"
                            </A>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Build the meta line shown under the set title — "N items" or
/// "N pieces · M exercises" depending on the mix.
///
/// Pencil shows "3 pieces · 20 min" but `SetEntryView` carries no
/// duration today, so we surface the type breakdown instead. Total
/// duration is a #TODO once we model item duration in core.
fn set_meta_line(set: &SetView) -> String {
    let (pieces, exercises) =
        set.entries
            .iter()
            .fold((0usize, 0usize), |(p, e), entry| match entry.item_type {
                ItemKind::Piece => (p + 1, e),
                ItemKind::Exercise => (p, e + 1),
            });
    match (pieces, exercises) {
        (0, 0) => "Empty".to_string(),
        (p, 0) => format!("{} {}", p, if p == 1 { "piece" } else { "pieces" }),
        (0, e) => format!("{} {}", e, if e == 1 { "exercise" } else { "exercises" }),
        (p, e) => format!(
            "{} {} \u{00B7} {} {}",
            p,
            if p == 1 { "piece" } else { "pieces" },
            e,
            if e == 1 { "exercise" } else { "exercises" }
        ),
    }
}

/// A single set row — name + meta line, full-row tap to edit. The
/// Edit / Delete affordances live in the swipe gesture and long-press
/// context menu, matching the iOS list pattern. No accent bar (`bar=
/// AccentBar::None`) — the set list is uniform-type so bars would
/// flatten into noise instead of carrying signal.
#[component]
fn SetRow(set: SetView) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let id = set.id.clone();
    let id_for_swipe = set.id.clone();
    let id_for_menu_delete = set.id.clone();
    let name = set.name.clone();
    let meta = set_meta_line(&set);
    let edit_href = format!("/routines/{}/edit", id);
    let edit_href_for_menu = edit_href.clone();

    let core_for_gesture = core.clone();
    let direct_delete = Callback::new(move |set_id: String| {
        let event = Event::Set(SetEvent::DeleteSet { id: set_id });
        let effects = {
            let core_ref = core_for_gesture.borrow();
            core_ref.process_event(event)
        };
        process_effects_with_core(
            &core_for_gesture,
            effects,
            &view_model,
            &is_loading,
            &is_submitting,
        );
    });

    let menu_actions = vec![
        ContextMenuAction {
            label: "Edit".to_string(),
            destructive: false,
            on_select: Callback::new(move |_| {
                let navigate = use_navigate();
                navigate(&edit_href_for_menu, NavigateOptions::default());
            }),
        },
        ContextMenuAction {
            label: "Delete".to_string(),
            destructive: true,
            on_select: Callback::new(move |_| {
                direct_delete.run(id_for_menu_delete.clone());
            }),
        },
    ];

    view! {
        <ContextMenu actions=menu_actions>
            <SwipeActions on_delete=Callback::new(move |_| {
                direct_delete.run(id_for_swipe.clone());
            })>
                <A href=edit_href attr:class="block no-underline">
                    <AccentRow bar=AccentBar::None>
                        <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                            <span class="text-sm font-semibold text-primary truncate">{name}</span>
                            <span class="text-xs text-muted">{meta}</span>
                        </div>
                        <Icon name=IconName::ChevronRight class="w-4 h-4 text-faint shrink-0" />
                    </AccentRow>
                </A>
            </SwipeActions>
        </ContextMenu>
    }
}

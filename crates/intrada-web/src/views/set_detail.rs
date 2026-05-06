use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, ItemKind, SessionEvent, SetEvent, SetView, ViewModel};

use crate::components::{
    AccentBar, AccentRow, BackLink, Button, ButtonSize, ButtonVariant, Icon, IconName,
    InlineTypeIndicator, SkeletonBlock, SkeletonLine,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, ItemType, SharedCore};

/// Map an `ItemKind` from core into the `ItemType` enum used by
/// `<InlineTypeIndicator>`.
fn item_kind_to_type(kind: ItemKind) -> ItemType {
    match kind {
        ItemKind::Piece => ItemType::Piece,
        ItemKind::Exercise => ItemType::Exercise,
    }
}

/// Set Detail page — read-only review of a saved set with the entries
/// inline, plus a hero **Start Practice** CTA, an **Edit** trailing
/// action in the nav row, and a **Delete** at the bottom.
///
/// Pencil reference: the proposal frames placed at `dCQvy` / `K7RMu`
/// (set as a 4th tab in Library, set detail mirroring piece detail).
///
/// Route: `/library/sets/:id`. Tap a Set row in the Library list → here.
/// Start Practice dispatches `SessionEvent::StartBuilding` followed by
/// `SetEvent::LoadSetIntoSetlist`, then navigates to `/sessions/new`
/// where the populated builder takes over.
#[component]
pub fn SetDetailView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();

    let core_start = core.clone();
    let core_delete = core;

    let id_for_start = id.clone();
    let id_for_delete = id.clone();
    let id_for_edit = id.clone();
    let id_for_lookup = id;

    let on_start_practice = Callback::new(move |_| {
        // Two-step dispatch: StartBuilding (Idle → Building) then
        // LoadSetIntoSetlist (push entries onto the new building
        // setlist). If a session is already in progress, StartBuilding
        // surfaces an error via the global ErrorBanner — the user sees
        // it and resolves manually (resume / abandon).
        let core_ref = core_start.borrow();
        let mut effects = core_ref.process_event(Event::Session(SessionEvent::StartBuilding));
        effects.extend(
            core_ref.process_event(Event::Set(SetEvent::LoadSetIntoSetlist {
                set_id: id_for_start.clone(),
            })),
        );
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        drop(core_ref);
        let nav = use_navigate();
        nav(
            "/sessions/new",
            NavigateOptions {
                replace: false,
                ..Default::default()
            },
        );
    });

    let on_delete = Callback::new(move |_| {
        let event = Event::Set(SetEvent::DeleteSet {
            id: id_for_delete.clone(),
        });
        let core_ref = core_delete.borrow();
        let effects = core_ref.process_event(event);
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        drop(core_ref);
        let nav = use_navigate();
        nav(
            "/",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
    });

    view! {
        <div class="space-y-5">
            // Nav row — back link + trailing Edit action (iOS
            // UINavigationBar idiom; matches piece detail).
            <div class="flex items-center justify-between -mb-2">
                <BackLink label="Library" href="/".to_string() />
                <A
                    href=format!("/routines/{id_for_edit}/edit")
                    attr:class="text-sm font-medium text-accent-text hover:text-accent-hover"
                >
                    "Edit"
                </A>
            </div>

            {move || {
                let id = id_for_lookup.clone();
                let set = view_model.with(|vm| vm.sets.iter().find(|s| s.id == id).cloned());
                match (set, is_loading.get()) {
                    (Some(set), _) => render_loaded(set, on_start_practice, on_delete, is_submitting),
                    (None, true) => view! {
                        <div class="space-y-4 animate-pulse">
                            <SkeletonLine width="w-2/3" height="h-8" />
                            <SkeletonLine width="w-1/3" height="h-4" />
                            <SkeletonBlock height="h-32" />
                        </div>
                    }
                    .into_any(),
                    (None, false) => view! {
                        <div class="text-center py-8">
                            <p class="text-secondary mb-4">"Set not found."</p>
                            <A
                                href="/"
                                attr:class="text-accent-text hover:text-accent-hover font-medium"
                            >
                                "\u{2190} Back to Library"
                            </A>
                        </div>
                    }
                    .into_any(),
                }
            }}
        </div>
    }
}

fn render_loaded(
    set: SetView,
    on_start_practice: Callback<leptos::ev::MouseEvent>,
    on_delete: Callback<leptos::ev::MouseEvent>,
    is_submitting: IsSubmitting,
) -> leptos::prelude::AnyView {
    let entry_count = set.entry_count;
    let count_label = if entry_count == 1 {
        "1 item".to_string()
    } else {
        format!("{entry_count} items")
    };
    let entries = set.entries.clone();
    let name = set.name.clone();

    view! {
        // Inline heading (PageHeading takes static strings; set names
        // are dynamic). Mirrors DetailView's piece-detail header shape.
        <div class="space-y-1">
            <h2 class="page-title">{name}</h2>
            <p class="text-sm text-muted">{count_label}</p>
        </div>

        // Entries — read-only AccentRows (no drag handle, no remove).
        // Each entry shows the item title + InlineTypeIndicator dot/label
        // — the same shape used in the Library list, just inert here.
        <div class="space-y-3">
            <h3 class="section-title">"Entries"</h3>
            {if entries.is_empty() {
                view! {
                    <p class="text-sm text-muted">"This set has no entries yet."</p>
                }.into_any()
            } else {
                view! {
                    <div class="space-y-2">
                        {entries.into_iter().map(|entry| {
                            let bar = match entry.item_type {
                                ItemKind::Piece => AccentBar::Gold,
                                ItemKind::Exercise => AccentBar::Blue,
                            };
                            let inline_type = item_kind_to_type(entry.item_type);
                            view! {
                                <AccentRow bar=bar>
                                    <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                                        <span class="text-sm font-semibold text-primary truncate">
                                            {entry.item_title}
                                        </span>
                                    </div>
                                    <InlineTypeIndicator item_type=inline_type />
                                </AccentRow>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>

        // Hero CTA — Start Practice with this set. Dispatches the two-step
        // StartBuilding + LoadSet flow then navigates to the builder.
        <Button
            variant=ButtonVariant::Primary
            size=ButtonSize::Hero
            full_width=true
            on_click=on_start_practice
        >
            <span class="inline-flex items-center gap-1.5">
                <Icon name=IconName::Play class="w-4 h-4" />
                "Start Practice"
            </span>
        </Button>

        // Delete — destructive, de-emphasised. No confirm sheet for v1
        // because Sets are recipes (re-creating from any setlist is
        // cheap); revisit if accidental deletes turn out to be a real
        // pain point.
        <Button
            variant=ButtonVariant::DangerOutline
            disabled=Signal::derive(move || is_submitting.get())
            on_click=on_delete
        >
            "Delete Set"
        </Button>
    }
    .into_any()
}

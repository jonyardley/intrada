use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;
use leptos_router::NavigateOptions;

use intrada_core::{Event, ItemEvent, ItemKind, ViewModel};

use crate::components::{
    parse_target_bpm, AccentBar, BackLink, BottomSheet, Button, ButtonSize, ButtonVariant, Card,
    DetailGroup, DetailRow, Icon, IconName, InlineTypeIndicator, LinkButton, SkeletonBlock,
    SkeletonLine, StatCard, StatTone, TempoProgressChart,
};
use crate::views::EditLibraryItemForm;
use intrada_web::core_bridge::process_effects;
use intrada_web::helpers::format_date_short;
use intrada_web::types::{IsLoading, IsSubmitting, ItemType, SharedCore};

/// Format total practice time as the "2h 15m" / "45m" pattern Pencil uses.
fn format_total_practice(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Map an `ItemKind` from core into the `ItemType` enum used by
/// `<InlineTypeIndicator>` (the two enums are duplicated for FFI/typegen
/// reasons; see `crates/intrada-web/src/types.rs`).
fn item_kind_to_type(kind: ItemKind) -> ItemType {
    match kind {
        ItemKind::Piece => ItemType::Piece,
        ItemKind::Exercise => ItemType::Exercise,
    }
}

#[component]
pub fn DetailView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let navigate = use_navigate();

    let show_delete_confirm = RwSignal::new(false);
    let edit_sheet_open = RwSignal::new(false);
    let close_edit_sheet = Callback::new(move |_| edit_sheet_open.set(false));

    view! {
        <div class="detail-view space-y-5">
            // ── Nav row: back link on the left, Edit on the right ──
            // Edit lives here as the trailing nav action (matching the
            // Pencil reference and iOS UINavigationBar idiom). The
            // bottom action row keeps Delete only.
            <div class="flex items-center justify-between -mb-2">
                <BackLink label="Library" href="/".to_string() />
                <button
                    type="button"
                    class="text-sm font-medium text-accent-text hover:text-accent-hover"
                    on:click=move |_| edit_sheet_open.set(true)
                >
                    "Edit"
                </button>
            </div>

            {move || {
                // Reactively find item — re-runs when ViewModel updates after fetch
                let item = view_model
                    .get()
                    .items
                    .into_iter()
                    .find(|i| i.id == id);

                if let Some(item) = item {
                    let intrada_core::LibraryItemView {
                        id: item_id,
                        title,
                        subtitle,
                        item_type,
                        key,
                        tempo,
                        notes,
                        tags,
                        created_at,
                        updated_at: _,
                        practice,
                        latest_achieved_tempo: _,
                    } = item;

                    let indicator_type = item_kind_to_type(item_type);
                    let tempo_for_stats = tempo.clone();
                    let tempo_for_history = tempo.clone();
                    let id_for_delete = item_id.clone();
                    let core_for_delete = core.clone();
                    let navigate_for_delete = navigate.clone();

                    view! {
                        // Delete confirmation banner — kept here even though
                        // the trigger moved to the bottom of the page; this
                        // banner appears just below the nav row when active.
                        {move || {
                            if show_delete_confirm.get() {
                                let id_del = id_for_delete.clone();
                                let core_del = core_for_delete.clone();
                                let navigate_del = navigate_for_delete.clone();
                                Some(view! {
                                    <div class="rounded-lg bg-danger-surface border border-danger/20 p-4" role="alert">
                                        <p class="text-sm text-danger-text mb-3">
                                            "Are you sure you want to delete this item? This action cannot be undone."
                                        </p>
                                        <div class="flex gap-3">
                                            <Button
                                                variant=ButtonVariant::Danger
                                                loading=Signal::derive(move || is_submitting.get())
                                                on_click=Callback::new(move |_| {
                                                    let event = Event::Item(ItemEvent::Delete { id: id_del.clone() });
                                                    let core_ref = core_del.borrow();
                                                    let effects = core_ref.process_event(event);
                                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                    navigate_del("/", NavigateOptions { replace: true, ..Default::default() });
                                                })>
                                                {move || if is_submitting.get() { "Deleting\u{2026}" } else { "Confirm Delete" }}
                                            </Button>
                                            <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| { show_delete_confirm.set(false); })>
                                                "Cancel"
                                            </Button>
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            }
                        }}

                        // ── Hero block — title + composer + type ─────
                        // Title is the page anchor at 34px Source Serif
                        // (the .page-title utility). Composer + Inline-
                        // TypeIndicator sit beneath in a single row.
                        <div>
                            <h2 class="page-title">{title}</h2>
                            <div class="flex items-center gap-3 mt-2">
                                {if !subtitle.is_empty() {
                                    Some(view! {
                                        <span class="text-base text-secondary">{subtitle.clone()}</span>
                                    })
                                } else {
                                    None
                                }}
                                <InlineTypeIndicator item_type=indicator_type />
                            </div>
                        </div>

                        // ── Stats row (only when there's practice data) ──
                        // Three StatCard refresh variants — Total Practice
                        // (warm gold), Sessions (accent purple), Target BPM
                        // (warm gold). Mirrors Pencil's M3 Piece Detail.
                        {practice.as_ref().map(|p| {
                            let total_practice = format_total_practice(p.total_minutes);
                            let session_count_str = format!("{}", p.session_count);
                            let target_bpm = parse_target_bpm(&tempo_for_stats)
                                .map(|b| format!("{}", b))
                                .unwrap_or_else(|| "\u{2014}".to_string()); // em-dash for "no target"
                            view! {
                                <div class="grid grid-cols-3 gap-3">
                                    <StatCard
                                        title="Total Practice"
                                        value=total_practice
                                        bar=AccentBar::Gold
                                    />
                                    <StatCard
                                        title="Sessions"
                                        value=session_count_str
                                        bar=AccentBar::Blue
                                        tone=StatTone::Accent
                                    />
                                    <StatCard
                                        title="Target BPM"
                                        value=target_bpm
                                        bar=AccentBar::Gold
                                        tone=StatTone::WarmAccent
                                    />
                                </div>
                            }
                        })}

                        // ── DETAILS group ─────────────────────────────
                        <DetailGroup label="Details" bar=AccentBar::Gold>
                            {key.map(|k| view! {
                                <DetailRow label="Key">{k}</DetailRow>
                            })}
                            {tempo.map(|t| view! {
                                <DetailRow label="Tempo">{t}</DetailRow>
                            })}
                            <DetailRow label="Added">{format_date_short(&created_at)}</DetailRow>
                        </DetailGroup>

                        // ── NOTES group (if notes exist) ──────────────
                        {notes.map(|n| view! {
                            <DetailGroup label="Notes" bar=AccentBar::Blue>
                                <p class="text-sm text-secondary leading-relaxed whitespace-pre-wrap">{n}</p>
                            </DetailGroup>
                        })}

                        // ── Score history & Tempo progress (if data) ──
                        // Each rendered as its own DetailGroup so the
                        // chart/list inherits the inset accent-bar chrome.
                        {practice.as_ref().and_then(|p| {
                            (!p.score_history.is_empty()).then(|| {
                                let history = p.score_history.clone();
                                view! {
                                    <DetailGroup label="Score History" bar=AccentBar::Blue>
                                        <div class="space-y-1.5">
                                            {history.into_iter().map(|entry| {
                                                let display_date = format_date_short(&entry.session_date);
                                                view! {
                                                    <div class="flex items-center justify-between text-sm">
                                                        <span class="text-muted">{display_date}</span>
                                                        <span class="badge badge--accent">
                                                            {format!("{}/5", entry.score)}
                                                        </span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </DetailGroup>
                                }
                            })
                        })}

                        {practice.as_ref().and_then(|p| {
                            (!p.tempo_history.is_empty()).then(|| {
                                let target = parse_target_bpm(&tempo_for_history);
                                let entries = p.tempo_history.clone();
                                let latest = p.latest_tempo;
                                view! {
                                    <DetailGroup label="Tempo Progress" bar=AccentBar::Gold>
                                        <TempoProgressChart
                                            entries=entries
                                            target_bpm=target
                                            latest_tempo=latest
                                        />
                                    </DetailGroup>
                                }
                            })
                        })}

                        // ── Tags (if any) ─────────────────────────────
                        {(!tags.is_empty()).then(|| view! {
                            <DetailGroup label="Tags" bar=AccentBar::Blue>
                                <div class="flex flex-wrap gap-1.5">
                                    {tags.into_iter().map(|tag| view! {
                                        <span class="inline-flex items-center rounded-full border border-border-default px-2.5 py-0.5 text-xs text-muted">
                                            {tag}
                                        </span>
                                    }).collect::<Vec<_>>()}
                                </div>
                            </DetailGroup>
                        })}

                        // ── Hero CTA: Start Practice ──────────────────
                        // Links into the session builder with no item
                        // pre-selection wired up yet — that's a follow-up.
                        // For now it gets the user to the right place.
                        <LinkButton
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Hero
                            full_width=true
                            href="/sessions/new"
                        >
                            <Icon name=IconName::Play class="w-4 h-4" />
                            "Start Practice"
                        </LinkButton>

                        // ── Delete (destructive, de-emphasised) ───────
                        <Button
                            variant=ButtonVariant::DangerOutline
                            disabled=Signal::derive(move || is_submitting.get())
                            on_click=Callback::new(move |_| { show_delete_confirm.set(true); })
                        >
                            "Delete"
                        </Button>

                        // Edit sheet — Mail-compose nav pattern (Cancel
                        // | Edit Item | Save). Save triggers the form's
                        // submit via the shared form ref; the bottom
                        // "Save Changes" CTA in the form does the same.
                        {
                            let edit_form_ref = NodeRef::<leptos::html::Form>::new();
                            let on_save_edit = Callback::new(move |_| {
                                if let Some(form) = edit_form_ref.get() {
                                    let _ = form.request_submit();
                                }
                            });
                            let submitting_signal = Signal::derive(move || is_submitting.get());
                            view! {
                                <BottomSheet
                                    open=edit_sheet_open
                                    on_close=close_edit_sheet
                                    nav_title="Edit Item".to_string()
                                    nav_action_label="Save".to_string()
                                    on_nav_action=on_save_edit
                                    nav_action_disabled=submitting_signal
                                >
                                    <EditLibraryItemForm
                                        item_id=item_id.clone()
                                        in_sheet=true
                                        on_dismiss=close_edit_sheet
                                        form_ref=edit_form_ref
                                    />
                                </BottomSheet>
                            }
                        }
                    }.into_any()
                } else if is_loading.get() {
                    // Data still loading — show skeleton placeholder
                    view! {
                        <Card>
                            <div class="space-y-4 animate-pulse">
                                <div class="space-y-3">
                                    <SkeletonLine width="w-2/3" height="h-9" />
                                    <SkeletonLine width="w-1/2" height="h-5" />
                                </div>
                                <SkeletonBlock height="h-20" />
                                <SkeletonBlock height="h-20" />
                            </div>
                        </Card>
                    }.into_any()
                } else {
                    // Loading complete, item genuinely not found
                    view! {
                        <div class="text-center py-8">
                            <p class="text-secondary mb-4">"Item not found."</p>
                            <A href="/" attr:class="text-accent-text hover:text-accent-hover font-medium">
                                "← Back to Library"
                            </A>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

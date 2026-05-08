use std::collections::HashMap;

use leptos::prelude::*;

use intrada_core::{Event, McpToken, McpTokenEvent, ViewModel};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

use crate::components::{
    Button, ButtonVariant, Card, EmptyState, GroupedList, GroupedListRow, IconName, TextField,
};

/// Account-settings sub-page for managing MCP Personal Access Tokens —
/// the auth path used by AI clients (Claude Desktop, Cursor, custom MCP
/// agents) to act on the user's behalf via the `/api/mcp/*` endpoints
/// (Phase 3+).
///
/// Reached via the "MCP tokens" row on the Settings page. Loads the list
/// on mount, supports inline create/revoke. Newly-created tokens surface
/// the full bearer string once via a card at the top of the page; the
/// list endpoint never exposes more than the prefix.
#[component]
pub fn McpTokensView() -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    // Refresh on mount.
    {
        let core = core.clone();
        Effect::new(move |_| {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::McpToken(McpTokenEvent::LoadTokens));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        });
    }

    let tokens = Signal::derive(move || view_model.get().mcp_tokens);
    let tokens_loaded = Signal::derive(move || view_model.get().mcp_tokens_loaded);
    let tokens_loading = Signal::derive(move || view_model.get().mcp_tokens_loading);
    let just_created = Signal::derive(move || view_model.get().just_created_token);

    let create_form_open = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let errors = RwSignal::new(HashMap::<String, String>::new());

    // Hoist all event-dispatching callbacks above the view! so the iteration
    // closure inside the list doesn't have to capture `core` per-iteration
    // (which would force the view! closure to be FnOnce).
    let core_for_create = core.clone();
    let create_token = Callback::new(move |_: leptos::ev::MouseEvent| {
        let trimmed = new_name.get().trim().to_string();
        if trimmed.is_empty() {
            errors.set(HashMap::from([(
                "name".to_string(),
                "Give the token a name so you can recognise it later".to_string(),
            )]));
            return;
        }
        errors.set(HashMap::new());
        let core_ref = core_for_create.borrow();
        let effects = core_ref.process_event(Event::McpToken(McpTokenEvent::CreateToken {
            name: trimmed,
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        new_name.set(String::new());
        create_form_open.set(false);
    });

    let core_for_dismiss = core.clone();
    let dismiss_created = Callback::new(move |_: leptos::ev::MouseEvent| {
        let core_ref = core_for_dismiss.borrow();
        let effects = core_ref.process_event(Event::McpToken(McpTokenEvent::DismissCreatedToken));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    let core_for_revoke = core.clone();
    let revoke_token: Callback<String> = Callback::new(move |id: String| {
        let core_ref = core_for_revoke.borrow();
        let effects = core_ref.process_event(Event::McpToken(McpTokenEvent::RevokeToken { id }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    let open_create_form = Callback::new(move |_: leptos::ev::MouseEvent| {
        create_form_open.set(true);
        errors.set(HashMap::new());
    });

    let cancel_create_form = Callback::new(move |_: leptos::ev::MouseEvent| {
        create_form_open.set(false);
        new_name.set(String::new());
        errors.set(HashMap::new());
    });

    view! {
        <div class="max-w-md mx-auto py-comfortable space-y-comfortable pb-[env(safe-area-inset-bottom)]">
            <h1 class="page-title">"MCP tokens"</h1>
            <p class="text-sm text-secondary">
                "Personal Access Tokens let an AI client (Claude Desktop, Cursor, custom MCP agent) act on your behalf. Generate one for each device or app that needs access."
            </p>

            // Show-once card for the just-created token. Sits at the top so
            // the user can copy without losing it under the list.
            <Show when=move || just_created.get().is_some() fallback=|| view! { <></> }>
                <Card>
                    <div class="space-y-card-compact p-card-compact">
                        <div>
                            <h3 class="card-title">"Token created"</h3>
                            <p class="hint-text">
                                "Copy this token now — it won't be shown again. Paste it into your MCP client's Authorization header (Bearer) or its config."
                            </p>
                        </div>
                        <textarea
                            class="w-full min-h-[80px] font-mono text-xs px-card-compact py-card-compact rounded-md bg-surface-secondary border border-border-default text-primary select-all"
                            readonly
                            prop:value=move || {
                                just_created.get().map(|c| c.token).unwrap_or_default()
                            }
                        />
                        <div class="flex justify-end">
                            <Button variant=ButtonVariant::Primary on_click=dismiss_created>
                                "Done"
                            </Button>
                        </div>
                    </div>
                </Card>
            </Show>

            // Create form (collapsed by default; opens to a name input).
            <Show
                when=move || create_form_open.get() && just_created.get().is_none()
                fallback=move || {
                    view! {
                        <Show
                            when=move || just_created.get().is_none()
                            fallback=|| view! { <></> }
                        >
                            <Button
                                variant=ButtonVariant::Primary
                                full_width=true
                                on_click=open_create_form
                            >
                                "Create new token"
                            </Button>
                        </Show>
                    }
                }
            >
                <Card>
                    <div class="space-y-card-compact p-card-compact">
                        <TextField
                            id="mcp-token-name"
                            label="Name"
                            value=new_name
                            field_name="name"
                            errors=errors
                            placeholder="e.g. Claude Desktop on laptop"
                            input_type="text"
                        />
                        <div class="flex gap-card-compact justify-end">
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=cancel_create_form
                            >
                                "Cancel"
                            </Button>
                            <Button variant=ButtonVariant::Primary on_click=create_token>
                                "Create"
                            </Button>
                        </div>
                    </div>
                </Card>
            </Show>

            // Token list.
            <Show
                when=move || tokens_loaded.get() && !tokens.get().is_empty()
                fallback=move || {
                    let loading = tokens_loading.get();
                    let loaded = tokens_loaded.get();
                    view! {
                        <Show when=move || loaded && !loading fallback=|| view! { <></> }>
                            <EmptyState
                                icon=IconName::ListChecks
                                title="No tokens yet"
                                body="Create your first token above to connect an AI client."
                            />
                        </Show>
                    }
                }
            >
                <GroupedList aria_label="MCP tokens".to_string()>
                    {move || {
                        tokens
                            .get()
                            .into_iter()
                            .map(|t| {
                                view! {
                                    <GroupedListRow>
                                        <TokenRow token=t on_revoke=revoke_token />
                                    </GroupedListRow>
                                }
                            })
                            .collect_view()
                    }}
                </GroupedList>
            </Show>
        </div>
    }
}

#[component]
fn TokenRow(token: McpToken, on_revoke: Callback<String>) -> impl IntoView {
    let revoked = token.revoked_at.is_some();
    let last_used = match token.last_used_at {
        Some(dt) => format!("Last used {}", format_relative(dt)),
        None => "Never used".to_string(),
    };
    let created = format!("Created {}", format_relative(token.created_at));
    let prefix = token.prefix.clone();
    let token_id = token.id.clone();
    let click_revoke = Callback::new(move |_: leptos::ev::MouseEvent| {
        on_revoke.run(token_id.clone());
    });

    view! {
        <div class="flex items-center justify-between gap-card-compact px-card-compact py-card-compact min-h-[64px]">
            <div class="flex flex-col gap-1 min-w-0">
                <div class="flex items-baseline gap-card-compact">
                    <span class="text-sm font-medium text-primary truncate">
                        {token.name}
                    </span>
                    <span class="font-mono text-xs text-muted">{prefix} "…"</span>
                </div>
                <span class="text-xs text-secondary">{last_used} " · " {created}</span>
            </div>
            {move || {
                if revoked {
                    view! {
                        <span class="text-xs font-medium text-muted px-card-compact py-1 rounded-full bg-surface-secondary">
                            "Revoked"
                        </span>
                    }
                        .into_any()
                } else {
                    view! {
                        <Button variant=ButtonVariant::Danger on_click=click_revoke>
                            "Revoke"
                        </Button>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// Format an absolute timestamp as a coarse relative string ("2d ago",
/// "just now"). Refresh-on-mount means precision down to the minute is
/// fine; we don't need a ticking signal.
fn format_relative(when: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let delta = now.signed_duration_since(when);
    let secs = delta.num_seconds();
    if secs < 60 {
        return "just now".to_string();
    }
    let mins = delta.num_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = delta.num_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let days = delta.num_days();
    if days < 30 {
        return format!("{days}d ago");
    }
    when.format("%b %-d, %Y").to_string()
}

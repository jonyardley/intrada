use leptos::prelude::*;

use intrada_core::{Event, McpAuditEntry, McpAuditEvent, ViewModel};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

use crate::components::{BackLink, EmptyState, GroupedList, GroupedListRow};

/// Account-settings sub-page that shows the user's MCP audit log —
/// every successful write tool call attributed to one of their PATs.
/// Read-only; entries are immutable once written.
#[component]
pub fn McpAuditView() -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    // Refresh on mount.
    {
        let core = core.clone();
        Effect::new(move |_| {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::McpAudit(McpAuditEvent::LoadAudit));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        });
    }

    let entries = Signal::derive(move || view_model.get().mcp_audit);
    let loaded = Signal::derive(move || view_model.get().mcp_audit_loaded);
    let loading = Signal::derive(move || view_model.get().mcp_audit_loading);

    view! {
        <div class="max-w-md mx-auto py-card-comfortable space-y-section pb-[env(safe-area-inset-bottom)]">
            <BackLink label="Back to Settings" href="/settings".to_string() />

            <div class="space-y-card">
                <h1 class="page-title">"MCP activity"</h1>
                <p class="text-sm text-secondary">
                    "Every action an AI client took on your behalf. Read-only — entries can't be edited or deleted."
                </p>
            </div>

            <Show
                when=move || loaded.get() && !entries.get().is_empty()
                fallback=move || {
                    let l = loading.get();
                    let ld = loaded.get();
                    view! {
                        <Show when=move || ld && !l fallback=|| view! { <></> }>
                            <EmptyState
                                icon=icondata::LuClock
                                title="No activity yet"
                                body="When an AI client uses one of your tokens to create or edit something, it'll appear here."
                            />
                        </Show>
                    }
                }
            >
                <GroupedList aria_label="MCP audit entries".to_string()>
                    {move || {
                        entries
                            .get()
                            .into_iter()
                            .map(|e| {
                                view! {
                                    <GroupedListRow>
                                        <AuditEntryRow entry=e />
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
fn AuditEntryRow(entry: McpAuditEntry) -> impl IntoView {
    let when = format_relative(entry.created_at);
    // token_id is None → write via Clerk session (no PAT). Show "web app"
    // rather than falling through to "(deleted token)".
    let token_label = if entry.token_id.is_none() {
        "web app".to_string()
    } else {
        entry.token_name.clone().unwrap_or_else(|| {
            entry
                .token_prefix
                .clone()
                .unwrap_or_else(|| "(deleted token)".into())
        })
    };
    let tool_label = humanize_tool(&entry.tool);

    view! {
        <div class="flex flex-col gap-1 p-card min-h-[64px]">
            <div class="flex items-baseline justify-between gap-card-compact">
                <span class="text-sm font-medium text-primary truncate">{tool_label}</span>
                <span class="text-xs text-secondary shrink-0">{when}</span>
            </div>
            <div class="flex items-baseline gap-card-compact">
                <span class="text-xs text-secondary">"via "</span>
                <span class="font-mono text-xs text-muted truncate">{token_label}</span>
            </div>
        </div>
    }
}

/// Map the wire tool name (`create_item`) to a human label
/// (`Created an item`). The agent's tool descriptions use user-facing
/// terminology; this mirrors that for the audit log so the user reads
/// the same vocabulary they're used to.
//
// NOTE: keep in sync with `intrada-api/src/mcp/tools.rs::SINGLE_WRITE_TOOLS`
// + `BULK_IMPORT_ITEMS`. When a new write tool ships, add its label here;
// the catch-all below renders "Unknown action" which is safe-but-ugly
// and shouldn't be the steady state for any real tool.
fn humanize_tool(tool: &str) -> &'static str {
    match tool {
        "create_item" => "Created an item",
        "update_item" => "Updated an item",
        "delete_item" => "Deleted an item",
        "create_set" => "Created a routine",
        "update_set" => "Updated a routine",
        "bulk_import_items" => "Bulk-imported items",
        _ => "Unknown action",
    }
}

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

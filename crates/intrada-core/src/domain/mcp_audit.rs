//! MCP audit-log read-only surface for the shell.

use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::model::Model;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct McpAuditEntry {
    pub id: String,
    /// `None` for JWT-authenticated MCP writes (browser/iOS session, no PAT).
    /// The UI renders these as "(web app)" (#528).
    pub token_id: Option<String>,
    /// Joined from `mcp_tokens`. `None` when the token row was hard-deleted,
    /// or when `token_id` itself is `None` (JWT write). Both cases are
    /// handled by the UI.
    pub token_name: Option<String>,
    pub token_prefix: Option<String>,
    pub tool: String,
    pub args_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum McpAuditEvent {
    LoadAudit,
    AuditLoaded(Vec<McpAuditEntry>),
    LoadAuditFailed {
        message: String,
        /// Resolved from a 429's `Retry-After` header, if the rejection was
        /// one and named a resolvable wait (#1273).
        retry_after: Option<DateTime<Utc>>,
    },
}

pub fn handle_mcp_audit_event(event: McpAuditEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        McpAuditEvent::LoadAudit => {
            let now = chrono::Utc::now();
            if let Some(msg) = model.rate_limit_message(now) {
                model.surface_error(msg);
                return crux_core::render::render();
            }
            model.mcp_audit_loading = true;
            Command::all([
                crate::http::list_mcp_audit(&model.api_base_url),
                crux_core::render::render(),
            ])
        }
        McpAuditEvent::AuditLoaded(entries) => {
            model.mcp_audit = entries;
            model.mcp_audit_loaded = true;
            model.mcp_audit_loading = false;
            model.record_success();
            crux_core::render::render()
        }
        McpAuditEvent::LoadAuditFailed {
            message,
            retry_after,
        } => {
            model.mcp_audit_loading = false;
            model.note_retry_after(retry_after);
            model.surface_error(message);
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_sets_loading_flag() {
        let mut model = Model::test_default();
        let _ = handle_mcp_audit_event(McpAuditEvent::LoadAudit, &mut model);
        assert!(model.mcp_audit_loading);
        assert!(!model.mcp_audit_loaded);
    }

    #[test]
    fn loaded_replaces_list_and_clears_loading() {
        let mut model = Model::test_default();
        model.mcp_audit_loading = true;
        let entry = McpAuditEntry {
            id: "a".into(),
            token_id: Some("t".into()),
            token_name: Some("Claude Desktop".into()),
            token_prefix: Some("intrada_pat_xxxx".into()),
            tool: "create_item".into(),
            args_hash: "deadbeef".into(),
            created_at: Utc::now(),
        };
        let _ = handle_mcp_audit_event(McpAuditEvent::AuditLoaded(vec![entry.clone()]), &mut model);
        assert_eq!(model.mcp_audit, vec![entry]);
        assert!(model.mcp_audit_loaded);
        assert!(!model.mcp_audit_loading);
    }

    #[test]
    fn load_audit_declines_to_fire_while_rate_limited() {
        let mut model = Model::test_default();
        model.rate_limited_until = Some(Utc::now() + chrono::Duration::seconds(30));
        let mut cmd = handle_mcp_audit_event(McpAuditEvent::LoadAudit, &mut model);
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Http(_))));
        assert!(!model.mcp_audit_loading);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn load_audit_failed_notes_retry_after() {
        let mut model = Model::test_default();
        let until = Utc::now() + chrono::Duration::seconds(30);
        let _ = handle_mcp_audit_event(
            McpAuditEvent::LoadAuditFailed {
                message: "rate limited".to_string(),
                retry_after: Some(until),
            },
            &mut model,
        );
        assert_eq!(model.rate_limited_until, Some(until));
    }
}

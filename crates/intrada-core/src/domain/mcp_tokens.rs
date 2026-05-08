//! MCP Personal Access Token management.
//!
//! Mirrors the API's `mcp_tokens` table — the list view never carries the
//! full token (only the prefix), and the just-created response is the only
//! place the full bearer string appears.

use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::model::Model;

/// A single PAT in the user's account, as surfaced by the list endpoint.
/// Fields mirror `intrada-api`'s `TokenListItem`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct McpToken {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Response from the create endpoint. The `token` field is the only place the
/// full bearer string is ever returned — UI shows it once and never again.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreatedMcpToken {
    pub id: String,
    pub name: String,
    pub token: String,
    pub prefix: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum McpTokenEvent {
    /// Fetch the user's tokens.
    LoadTokens,
    TokensLoaded(Vec<McpToken>),
    LoadTokensFailed(String),

    /// Create a new PAT with the given name.
    CreateToken {
        name: String,
    },
    /// Server returned the full token. UI must show it once, then dismiss.
    TokenCreated(CreatedMcpToken),
    CreateTokenFailed(String),

    /// User dismissed the show-once modal — clear the just-created token
    /// from the model so it can't be re-displayed (and isn't kept in
    /// memory longer than necessary).
    DismissCreatedToken,

    /// Soft-revoke a PAT. The server stamps `revoked_at`; the model
    /// updates the corresponding entry in `mcp_tokens` so the list
    /// reflects the revocation without a full refetch.
    RevokeToken {
        id: String,
    },
    TokenRevoked {
        id: String,
        revoked_at: DateTime<Utc>,
    },
    RevokeTokenFailed {
        id: String,
        message: String,
    },
}

pub fn handle_mcp_token_event(event: McpTokenEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        McpTokenEvent::LoadTokens => {
            model.mcp_tokens_loading = true;
            Command::all([
                crate::http::list_mcp_tokens(&model.api_base_url),
                crux_core::render::render(),
            ])
        }

        McpTokenEvent::TokensLoaded(tokens) => {
            model.mcp_tokens = tokens;
            model.mcp_tokens_loaded = true;
            model.mcp_tokens_loading = false;
            crux_core::render::render()
        }

        McpTokenEvent::LoadTokensFailed(message) => {
            model.mcp_tokens_loading = false;
            model.last_error = Some(message);
            crux_core::render::render()
        }

        McpTokenEvent::CreateToken { name } => {
            crate::http::create_mcp_token(&model.api_base_url, &name)
        }

        McpTokenEvent::TokenCreated(created) => {
            // Push a list-shaped view of the new token into `mcp_tokens` so
            // the list immediately reflects it, and surface the full token
            // through `just_created_token` for the show-once modal.
            model.mcp_tokens.insert(
                0,
                McpToken {
                    id: created.id.clone(),
                    name: created.name.clone(),
                    prefix: created.prefix.clone(),
                    last_used_at: None,
                    created_at: created.created_at,
                    revoked_at: None,
                },
            );
            model.just_created_token = Some(created);
            crux_core::render::render()
        }

        McpTokenEvent::CreateTokenFailed(message) => {
            model.last_error = Some(message);
            crux_core::render::render()
        }

        McpTokenEvent::DismissCreatedToken => {
            model.just_created_token = None;
            crux_core::render::render()
        }

        McpTokenEvent::RevokeToken { id } => {
            crate::http::revoke_mcp_token(&model.api_base_url, &id)
        }

        McpTokenEvent::TokenRevoked { id, revoked_at } => {
            if let Some(token) = model.mcp_tokens.iter_mut().find(|t| t.id == id) {
                token.revoked_at = Some(revoked_at);
            }
            crux_core::render::render()
        }

        McpTokenEvent::RevokeTokenFailed { id: _, message } => {
            model.last_error = Some(message);
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_model() -> Model {
        Model::test_default()
    }

    fn sample_token(id: &str, revoked: bool) -> McpToken {
        McpToken {
            id: id.to_string(),
            name: format!("token-{id}"),
            prefix: "intrada_pat_xxxx".to_string(),
            last_used_at: None,
            created_at: Utc::now(),
            revoked_at: if revoked { Some(Utc::now()) } else { None },
        }
    }

    #[test]
    fn load_tokens_sets_loading_flag() {
        let mut model = fresh_model();
        let _cmd = handle_mcp_token_event(McpTokenEvent::LoadTokens, &mut model);
        assert!(model.mcp_tokens_loading);
        assert!(!model.mcp_tokens_loaded);
    }

    #[test]
    fn tokens_loaded_replaces_list_and_clears_loading() {
        let mut model = fresh_model();
        model.mcp_tokens_loading = true;
        let tokens = vec![sample_token("a", false), sample_token("b", false)];
        let _cmd = handle_mcp_token_event(McpTokenEvent::TokensLoaded(tokens.clone()), &mut model);
        assert_eq!(model.mcp_tokens, tokens);
        assert!(!model.mcp_tokens_loading);
        assert!(model.mcp_tokens_loaded);
    }

    #[test]
    fn token_created_prepends_to_list_and_stashes_full_token() {
        let mut model = fresh_model();
        model.mcp_tokens = vec![sample_token("existing", false)];
        let created = CreatedMcpToken {
            id: "new".to_string(),
            name: "fresh".to_string(),
            token: "intrada_pat_secret".to_string(),
            prefix: "intrada_pat_secr".to_string(),
            created_at: Utc::now(),
        };
        let _cmd = handle_mcp_token_event(McpTokenEvent::TokenCreated(created.clone()), &mut model);
        assert_eq!(model.mcp_tokens.len(), 2);
        assert_eq!(model.mcp_tokens[0].id, "new"); // newest first
        assert_eq!(model.just_created_token, Some(created));
    }

    #[test]
    fn dismiss_clears_just_created_token() {
        let mut model = fresh_model();
        model.just_created_token = Some(CreatedMcpToken {
            id: "x".to_string(),
            name: "x".to_string(),
            token: "intrada_pat_x".to_string(),
            prefix: "intrada_pat_x".to_string(),
            created_at: Utc::now(),
        });
        let _cmd = handle_mcp_token_event(McpTokenEvent::DismissCreatedToken, &mut model);
        assert!(model.just_created_token.is_none());
    }

    #[test]
    fn token_revoked_stamps_revoked_at_on_matching_entry() {
        let mut model = fresh_model();
        let token = sample_token("target", false);
        model.mcp_tokens = vec![sample_token("other", false), token.clone()];
        let revoked_at = Utc::now();
        let _cmd = handle_mcp_token_event(
            McpTokenEvent::TokenRevoked {
                id: "target".to_string(),
                revoked_at,
            },
            &mut model,
        );
        assert!(model
            .mcp_tokens
            .iter()
            .find(|t| t.id == "target")
            .unwrap()
            .revoked_at
            .is_some());
        assert!(model
            .mcp_tokens
            .iter()
            .find(|t| t.id == "other")
            .unwrap()
            .revoked_at
            .is_none());
    }
}

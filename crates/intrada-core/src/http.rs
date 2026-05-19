//! HTTP request builders for the intrada API.
//!
//! All request construction and response parsing happens here in the core.
//! The shell executes the raw HTTP requests and feeds responses back;
//! auth headers are added by the shell when processing `Effect::Http`.

use crux_core::Command;

use crate::app::{Effect, Event};
use crate::domain::account::{AccountEvent, AccountPreferences};
use crate::domain::goal::Goal;
use crate::domain::item::Item;
use crate::domain::mcp_audit::{McpAuditEntry, McpAuditEvent};
use crate::domain::mcp_tokens::{CreatedMcpToken, McpToken, McpTokenEvent};
use crate::domain::oauth::{OAuthEvent, OAuthFinalizeParams};
use crate::domain::session::PracticeSession;
use crate::domain::types::{
    CreateGoal, CreateItem, CreateSetRequest, LinkGoalItem, UpdateGoal, UpdateItem,
    UpdateSetRequest,
};

type Http = crux_http::command::Http<Effect, Event>;

// ── Fetch operations ────────────────────────────────────────────────────

pub fn fetch_items(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/items"))
        .expect_json::<Vec<Item>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(items) => Event::DataLoaded { items },
                None => Event::LoadFailed("Failed to parse items response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load items: {e}")),
        })
}

pub fn fetch_sessions(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/sessions"))
        .expect_json::<Vec<PracticeSession>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(sessions) => Event::SessionsLoaded { sessions },
                None => Event::LoadFailed("Failed to parse sessions response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load sessions: {e}")),
        })
}

pub fn fetch_sets(api_base_url: &str) -> Command<Effect, Event> {
    use crate::domain::set::Set;

    Http::get(format!("{api_base_url}/api/sets"))
        .expect_json::<Vec<Set>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(sets) => Event::SetsLoaded { sets },
                None => Event::LoadFailed("Failed to parse sets response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load sets: {e}")),
        })
}

// ── Item operations ─────────────────────────────────────────────────────

pub fn create_item(api_base_url: &str, item: &Item, temp_id: &str) -> Command<Effect, Event> {
    let create = CreateItem {
        title: item.title.clone(),
        kind: item.kind.clone(),
        composer: item.composer.clone(),
        key: item.key.clone(),
        tempo: item.tempo.clone(),
        notes: item.notes.clone(),
        tags: item.tags.clone(),
    };
    let temp_id = temp_id.to_string();
    Http::post(format!("{api_base_url}/api/items"))
        .body_json(&create)
        .expect("serialize CreateItem")
        .expect_json::<Item>()
        .build()
        .then_send(move |result| match result {
            Ok(response) => match response.body().cloned() {
                Some(item) => Event::ItemCreated {
                    temp_id: temp_id.clone(),
                    item,
                },
                None => Event::LoadFailed("create_item: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to save item: {e}")),
        })
}

pub fn update_item(api_base_url: &str, item: &Item) -> Command<Effect, Event> {
    let update = UpdateItem {
        title: Some(item.title.clone()),
        composer: Some(item.composer.clone()),
        key: Some(item.key.clone()),
        tempo: Some(item.tempo.clone()),
        notes: Some(item.notes.clone()),
        tags: Some(item.tags.clone()),
    };
    Http::put(format!("{api_base_url}/api/items/{}", item.id))
        .body_json(&update)
        .expect("serialize UpdateItem")
        .expect_json::<Item>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(item) => Event::ItemUpdated { item },
                None => Event::LoadFailed("update_item: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update item: {e}")),
        })
}

pub fn delete_item(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/items/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete item: {e}")),
        })
}

// ── Session operations ──────────────────────────────────────────────────

pub fn create_session(api_base_url: &str, session: &PracticeSession) -> Command<Effect, Event> {
    Http::post(format!("{api_base_url}/api/sessions"))
        .body_json(session)
        .expect("serialize PracticeSession")
        .build()
        .then_send(|result| match result {
            // Don't re-fetch: SaveSession already pushed the session into the
            // model optimistically and rebuilt practice_summaries. Re-fetching
            // could overwrite the optimistic data with a stale server response
            // before the write is visible, causing #247 (practice data not
            // updating after session save).
            Ok(_) => Event::SessionSaved,
            Err(e) => Event::LoadFailed(format!("Failed to save session: {e}")),
        })
}

pub fn delete_session(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/sessions/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete session: {e}")),
        })
}

// ── Set operations ──────────────────────────────────────────────────

pub fn create_set(api_base_url: &str, set: &crate::domain::set::Set) -> Command<Effect, Event> {
    let create = CreateSetRequest::from_set(set);
    Http::post(format!("{api_base_url}/api/sets"))
        .body_json(&create)
        .expect("serialize CreateSetRequest")
        .build()
        .then_send(|result| match result {
            // Success bumps the set_saves_committed counter (so SetSaveForm
            // can flip from optimistic→confirmed) AND triggers a refetch via
            // the SetSaveSucceeded handler — see app.rs (#449).
            Ok(_) => Event::SetSaveSucceeded,
            Err(e) => Event::LoadFailed(format!("Failed to save set: {e}")),
        })
}

pub fn update_set(api_base_url: &str, set: &crate::domain::set::Set) -> Command<Effect, Event> {
    use crate::domain::set::Set;

    let update = UpdateSetRequest::from_set(set);
    Http::put(format!("{api_base_url}/api/sets/{}", set.id))
        .body_json(&update)
        .expect("serialize UpdateSetRequest")
        .expect_json::<Set>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(set) => Event::SetUpdated { set },
                None => Event::LoadFailed("update_set: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update set: {e}")),
        })
}

pub fn delete_set(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/sets/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete set: {e}")),
        })
}

// ── Goal operations ───────────────────────────────────────────────────

pub fn fetch_goals(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/goals?status=all"))
        .expect_json::<Vec<Goal>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(goals) => Event::GoalsLoaded { goals },
                None => Event::LoadFailed("Failed to parse goals response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load goals: {e}")),
        })
}

pub fn fetch_goal(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/goals/{id}"))
        .expect_json::<Goal>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(goal) => Event::GoalLoaded { goal },
                None => Event::LoadFailed("Failed to parse goal response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load goal: {e}")),
        })
}

pub fn create_goal(
    api_base_url: &str,
    input: &CreateGoal,
    temp_id: &str,
) -> Command<Effect, Event> {
    let temp_id = temp_id.to_string();
    Http::post(format!("{api_base_url}/api/goals"))
        .body_json(input)
        .expect("serialize CreateGoal")
        .expect_json::<Goal>()
        .build()
        .then_send(move |result| match result {
            Ok(response) => match response.body().cloned() {
                Some(goal) => Event::GoalCreated {
                    temp_id: temp_id.clone(),
                    goal,
                },
                None => Event::LoadFailed("create_goal: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to save goal: {e}")),
        })
}

pub fn update_goal(api_base_url: &str, id: &str, input: &UpdateGoal) -> Command<Effect, Event> {
    Http::put(format!("{api_base_url}/api/goals/{id}"))
        .body_json(input)
        .expect("serialize UpdateGoal")
        .expect_json::<Goal>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(goal) => Event::GoalLoaded { goal },
                None => Event::LoadFailed("update_goal: server returned no body".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to update goal: {e}")),
        })
}

pub fn complete_goal(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    use crate::domain::goal::GoalStatus;
    use crate::domain::types::UpdateGoal;
    let input = UpdateGoal {
        status: Some(GoalStatus::Completed),
        ..Default::default()
    };
    Http::put(format!("{api_base_url}/api/goals/{id}"))
        .body_json(&input)
        .expect("serialize UpdateGoal for complete")
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::RefetchGoals,
            Err(e) => Event::LoadFailed(format!("Failed to complete goal: {e}")),
        })
}

pub fn delete_goal(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/goals/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete goal: {e}")),
        })
}

pub fn link_goal_item(
    api_base_url: &str,
    goal_id: &str,
    item: &LinkGoalItem,
) -> Command<Effect, Event> {
    Http::post(format!("{api_base_url}/api/goals/{goal_id}/items"))
        .body_json(item)
        .expect("serialize LinkGoalItem")
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::RefetchGoals,
            Err(e) => Event::LoadFailed(format!("Failed to link item to goal: {e}")),
        })
}

pub fn unlink_goal_item(
    api_base_url: &str,
    goal_id: &str,
    item_id: &str,
) -> Command<Effect, Event> {
    Http::delete(format!(
        "{api_base_url}/api/goals/{goal_id}/items/{item_id}"
    ))
    .build()
    .then_send(|result| match result {
        Ok(_) => Event::RefetchGoals,
        Err(e) => Event::LoadFailed(format!("Failed to unlink item from goal: {e}")),
    })
}

// ── Account operations ─────────────────────────────────────────────────

pub fn get_account_preferences(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/account/preferences"))
        .expect_json::<AccountPreferences>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(prefs) => Event::Account(AccountEvent::PreferencesLoaded(prefs)),
                None => Event::LoadFailed("Failed to parse preferences response".into()),
            },
            Err(e) => Event::LoadFailed(format!("Failed to load preferences: {e}")),
        })
}

pub fn save_account_preferences(
    api_base_url: &str,
    prefs: &AccountPreferences,
    previous: Option<AccountPreferences>,
) -> Command<Effect, Event> {
    Http::put(format!("{api_base_url}/api/account/preferences"))
        .body_json(prefs)
        .expect("serialize AccountPreferences")
        .expect_json::<AccountPreferences>()
        .build()
        .then_send(move |result| match result {
            Ok(response) => match response.body().cloned() {
                Some(saved) => Event::Account(AccountEvent::PreferencesSaved(saved)),
                None => Event::Account(AccountEvent::SavePreferencesFailed {
                    previous: previous.clone(),
                    message: "save_preferences: server returned no body".into(),
                }),
            },
            Err(e) => Event::Account(AccountEvent::SavePreferencesFailed {
                previous: previous.clone(),
                message: format!("Failed to save preferences: {e}"),
            }),
        })
}

pub fn delete_account(api_base_url: &str) -> Command<Effect, Event> {
    Http::delete(format!("{api_base_url}/api/account"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::Account(AccountEvent::AccountDeleted),
            Err(e) => Event::Account(AccountEvent::DeleteAccountFailed(format!(
                "Failed to delete account: {e}"
            ))),
        })
}

// ── MCP Personal Access Tokens ─────────────────────────────────────────

pub fn list_mcp_tokens(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/account/tokens"))
        .expect_json::<Vec<McpToken>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(tokens) => Event::McpToken(McpTokenEvent::TokensLoaded(tokens)),
                None => Event::McpToken(McpTokenEvent::LoadTokensFailed(
                    "list_mcp_tokens: empty body".into(),
                )),
            },
            Err(e) => Event::McpToken(McpTokenEvent::LoadTokensFailed(format!(
                "Failed to load tokens: {e}"
            ))),
        })
}

pub fn create_mcp_token(api_base_url: &str, name: &str) -> Command<Effect, Event> {
    #[derive(serde::Serialize)]
    struct Body<'a> {
        name: &'a str,
    }

    Http::post(format!("{api_base_url}/api/account/tokens"))
        .body_json(&Body { name })
        .expect("serialize CreateTokenRequest")
        .expect_json::<CreatedMcpToken>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(created) => Event::McpToken(McpTokenEvent::TokenCreated(created)),
                None => Event::McpToken(McpTokenEvent::CreateTokenFailed(
                    "create_mcp_token: empty body".into(),
                )),
            },
            Err(e) => Event::McpToken(McpTokenEvent::CreateTokenFailed(format!(
                "Failed to create token: {e}"
            ))),
        })
}

pub fn revoke_mcp_token(api_base_url: &str, id: &str) -> Command<Effect, Event> {
    let id_for_callback = id.to_string();
    Http::delete(format!("{api_base_url}/api/account/tokens/{id}"))
        .build()
        .then_send(move |result| match result {
            Ok(_) => Event::McpToken(McpTokenEvent::TokenRevoked {
                id: id_for_callback.clone(),
                revoked_at: chrono::Utc::now(),
            }),
            Err(e) => Event::McpToken(McpTokenEvent::RevokeTokenFailed {
                id: id_for_callback.clone(),
                message: format!("Failed to revoke token: {e}"),
            }),
        })
}

// ── MCP Audit Log ──────────────────────────────────────────────────────

pub fn list_mcp_audit(api_base_url: &str) -> Command<Effect, Event> {
    Http::get(format!("{api_base_url}/api/account/audit"))
        .expect_json::<Vec<McpAuditEntry>>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(entries) => Event::McpAudit(McpAuditEvent::AuditLoaded(entries)),
                None => Event::McpAudit(McpAuditEvent::LoadAuditFailed(
                    "list_mcp_audit: empty body".into(),
                )),
            },
            Err(e) => Event::McpAudit(McpAuditEvent::LoadAuditFailed(format!(
                "Failed to load audit log: {e}"
            ))),
        })
}

// ── OAuth Finalize ─────────────────────────────────────────────────────

#[derive(serde::Deserialize, Debug, Clone)]
struct OAuthFinalizeResponse {
    redirect_url: String,
}

pub fn oauth_finalize(api_base_url: &str, params: &OAuthFinalizeParams) -> Command<Effect, Event> {
    Http::post(format!("{api_base_url}/oauth/finalize"))
        .body_json(params)
        .expect("serialize OAuthFinalizeParams")
        .expect_json::<OAuthFinalizeResponse>()
        .build()
        .then_send(|result| match result {
            Ok(response) => match response.body().cloned() {
                Some(body) => Event::OAuth(OAuthEvent::ConsentFinalized {
                    redirect_url: body.redirect_url,
                }),
                None => Event::OAuth(OAuthEvent::ConsentFailed(
                    "oauth_finalize: empty body".into(),
                )),
            },
            Err(e) => Event::OAuth(OAuthEvent::ConsentFailed(format!(
                "Failed to finalize OAuth consent: {e}"
            ))),
        })
}

//! HTTP request builders for the intrada API.
//!
//! Auth headers are added by the shell when processing `Effect::Http`.

use crux_core::Command;

use crate::app::{Effect, Event};
use crate::domain::account::{AccountEvent, AccountPreferences};
use crate::domain::item::Item;
use crate::domain::mcp_audit::{McpAuditEntry, McpAuditEvent};
use crate::domain::mcp_tokens::{CreatedMcpToken, McpToken, McpTokenEvent};
use crate::domain::oauth::{OAuthEvent, OAuthFinalizeParams};
use crate::domain::session::PracticeSession;
use crate::domain::types::{CreateItem, CreateSetRequest, UpdateItem, UpdateSetRequest};

type Http = crux_http::command::Http<Effect, Event>;

/// crux_http panics building a request from a relative URL, and a panic
/// mid-`update` poisons the Model RwLock — bricking the core for the session.
/// So a base without an `http(s)://` scheme + host yields a soft `LoadFailed`.
fn require_absolute_base(api_base_url: &str) -> Option<Command<Effect, Event>> {
    let host = api_base_url
        .strip_prefix("http://")
        .or_else(|| api_base_url.strip_prefix("https://"));
    match host {
        Some(rest) if !rest.is_empty() => None,
        _ => Some(Command::event(Event::LoadFailed(
            "No API base URL configured".to_string(),
        ))),
    }
}

// ── Fetch operations ────────────────────────────────────────────────────

pub fn fetch_items(api_base_url: &str) -> Command<Effect, Event> {
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
    let create = CreateItem {
        title: item.title.clone(),
        kind: item.kind.clone(),
        composer: item.composer.clone(),
        key: item.key.clone(),
        modality: item.modality,
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
    let update = UpdateItem {
        title: Some(item.title.clone()),
        // Type-change is local-first only for now; the API doesn't persist `kind`
        // on update, so the online PATCH omits it (tracked for sync parity).
        kind: None,
        composer: Some(item.composer.clone()),
        key: Some(item.key.clone()),
        modality: Some(item.modality),
        tempo: Some(item.tempo.clone()),
        notes: Some(item.notes.clone()),
        tags: Some(item.tags.clone()),
        priority: Some(item.priority),
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
    Http::delete(format!("{api_base_url}/api/items/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete item: {e}")),
        })
}

// ── Session operations ──────────────────────────────────────────────────

pub fn create_session(api_base_url: &str, session: &PracticeSession) -> Command<Effect, Event> {
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
    Http::delete(format!("{api_base_url}/api/sessions/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete session: {e}")),
        })
}

// ── Set operations ──────────────────────────────────────────────────

pub fn create_set(
    api_base_url: &str,
    set: &crate::domain::set::Set,
    request_id: String,
) -> Command<Effect, Event> {
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
    let create = CreateSetRequest::from_set(set);
    Http::post(format!("{api_base_url}/api/sets"))
        .body_json(&create)
        .expect("serialize CreateSetRequest")
        .build()
        .then_send(move |result| match result {
            Ok(_) => Event::SetSaveSucceeded { request_id },
            Err(e) => Event::LoadFailed(format!("Failed to save set: {e}")),
        })
}

pub fn update_set(api_base_url: &str, set: &crate::domain::set::Set) -> Command<Effect, Event> {
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
    Http::delete(format!("{api_base_url}/api/sets/{id}"))
        .build()
        .then_send(|result| match result {
            Ok(_) => Event::DeleteConfirmed,
            Err(e) => Event::LoadFailed(format!("Failed to delete set: {e}")),
        })
}

// ── Account operations ─────────────────────────────────────────────────

pub fn get_account_preferences(api_base_url: &str) -> Command<Effect, Event> {
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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
    if let Some(cmd) = require_absolute_base(api_base_url) {
        return cmd;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Effect;
    use crate::domain::item::ItemKind;
    use crate::domain::set::{Set, SetEntry};
    use crate::domain::types::Tempo;
    use chrono::TimeZone;
    use crux_http::protocol::HttpRequest;
    use serde_json::{json, Value};

    const BASE: &str = "https://api.example.com";

    fn take_http(cmd: &mut Command<Effect, Event>) -> HttpRequest {
        for effect in cmd.effects() {
            if let Effect::Http(req) = effect {
                return req.operation.clone();
            }
        }
        panic!("expected an Http effect");
    }

    fn body_as_json(req: &HttpRequest) -> Value {
        serde_json::from_slice(&req.body).expect("body should parse as JSON")
    }

    fn fixed_time() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    fn sample_item() -> Item {
        Item {
            id: "01HX0000000000000000000000".into(),
            title: "Etude Op. 10 No. 1".into(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".into()),
            key: Some("C major".into()),
            modality: None,
            tempo: Some(Tempo {
                marking: Some("Allegro".into()),
                bpm: Some(132),
            }),
            notes: Some("focus on RH evenness".into()),
            tags: vec!["scale".into(), "warmup".into()],
            created_at: fixed_time(),
            updated_at: fixed_time(),
            priority: false,
        }
    }

    fn sample_set() -> Set {
        Set {
            id: "01HSET00000000000000000000".into(),
            name: "Warmup".into(),
            entries: vec![SetEntry {
                id: "01HENT00000000000000000000".into(),
                item_id: "01HX0000000000000000000000".into(),
                item_title: "Etude".into(),
                item_type: ItemKind::Piece,
                position: 0,
            }],
            created_at: fixed_time(),
            updated_at: fixed_time(),
        }
    }

    #[test]
    fn relative_base_url_emits_soft_error_not_panic() {
        for base in ["", "https://"] {
            let mut cmd = delete_item(base, "id");
            assert!(!cmd.effects().any(|e| matches!(e, Effect::Http(_))));
            assert!(cmd.events().any(|e| matches!(e, Event::LoadFailed(_))));
        }
    }

    // ── Fetch endpoints: URL + method ──────────────────────────────────

    #[test]
    fn fetch_items_is_get_to_items_collection() {
        let req = take_http(&mut fetch_items(BASE));
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/api/items");
        assert!(req.body.is_empty());
    }

    #[test]
    fn fetch_sessions_is_get_to_sessions_collection() {
        let req = take_http(&mut fetch_sessions(BASE));
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/api/sessions");
    }

    #[test]
    fn fetch_sets_is_get_to_sets_collection() {
        let req = take_http(&mut fetch_sets(BASE));
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/api/sets");
    }

    // ── Item create/update/delete ─────────────────────────────────────

    #[test]
    fn create_item_posts_create_dto_without_id_or_timestamps() {
        let item = sample_item();
        let req = take_http(&mut create_item(BASE, &item, "temp-1"));
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.example.com/api/items");

        let body = body_as_json(&req);
        assert_eq!(body["title"], "Etude Op. 10 No. 1");
        assert_eq!(body["kind"], "piece");
        assert_eq!(body["composer"], "Chopin");
        assert_eq!(body["key"], "C major");
        assert_eq!(body["tempo"]["marking"], "Allegro");
        assert_eq!(body["tempo"]["bpm"], 132);
        assert_eq!(body["notes"], "focus on RH evenness");
        assert_eq!(body["tags"], json!(["scale", "warmup"]));
        assert!(
            body.get("id").is_none() && body.get("created_at").is_none(),
            "create body must not include id or timestamps"
        );
    }

    #[test]
    fn update_item_puts_to_id_path_with_patch_body() {
        let mut item = sample_item();
        item.composer = None;
        let req = take_http(&mut update_item(BASE, &item));
        assert_eq!(req.method, "PUT");
        assert_eq!(
            req.url,
            "https://api.example.com/api/items/01HX0000000000000000000000"
        );

        let body = body_as_json(&req);
        assert_eq!(body["title"], "Etude Op. 10 No. 1");
        assert!(
            body["composer"].is_null(),
            "cleared optional fields serialize as JSON null"
        );
    }

    #[test]
    fn delete_item_is_delete_to_id_path_with_empty_body() {
        let req = take_http(&mut delete_item(BASE, "01HX0000000000000000000000"));
        assert_eq!(req.method, "DELETE");
        assert_eq!(
            req.url,
            "https://api.example.com/api/items/01HX0000000000000000000000"
        );
        assert!(req.body.is_empty());
    }

    // ── Session endpoints ─────────────────────────────────────────────

    #[test]
    fn create_session_posts_the_session_as_body() {
        let session = PracticeSession {
            id: "01HSESS0000000000000000000".into(),
            entries: vec![],
            session_notes: None,
            session_intention: Some("scales".into()),
            started_at: fixed_time(),
            completed_at: fixed_time(),
            total_duration_secs: 600,
            completion_status: crate::domain::session::CompletionStatus::Completed,
        };
        let req = take_http(&mut create_session(BASE, &session));
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.example.com/api/sessions");

        let body = body_as_json(&req);
        assert_eq!(body["id"], "01HSESS0000000000000000000");
        assert_eq!(body["session_intention"], "scales");
        assert_eq!(body["total_duration_secs"], 600);
    }

    #[test]
    fn delete_session_is_delete_to_id_path() {
        let req = take_http(&mut delete_session(BASE, "01HSESS0000000000000000000"));
        assert_eq!(req.method, "DELETE");
        assert_eq!(
            req.url,
            "https://api.example.com/api/sessions/01HSESS0000000000000000000"
        );
    }

    // ── Set endpoints ─────────────────────────────────────────────────

    #[test]
    fn create_set_posts_create_dto_shape() {
        let set = sample_set();
        let req = take_http(&mut create_set(BASE, &set, "req-test".to_string()));
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.example.com/api/sets");

        let body = body_as_json(&req);
        assert_eq!(body["name"], "Warmup");
        assert_eq!(body["entries"][0]["item_id"], "01HX0000000000000000000000");
        assert_eq!(body["entries"][0]["item_type"], "piece");
        assert!(
            body.get("id").is_none() && body["entries"][0].get("position").is_none(),
            "create set body strips server-owned fields"
        );
    }

    #[test]
    fn update_set_puts_to_id_path() {
        let set = sample_set();
        let req = take_http(&mut update_set(BASE, &set));
        assert_eq!(req.method, "PUT");
        assert_eq!(
            req.url,
            "https://api.example.com/api/sets/01HSET00000000000000000000"
        );
        let body = body_as_json(&req);
        assert_eq!(body["name"], "Warmup");
    }

    #[test]
    fn delete_set_is_delete_to_id_path() {
        let req = take_http(&mut delete_set(BASE, "01HSET00000000000000000000"));
        assert_eq!(req.method, "DELETE");
        assert_eq!(
            req.url,
            "https://api.example.com/api/sets/01HSET00000000000000000000"
        );
    }

    // ── Account / MCP / OAuth endpoints ──────────────────────────────

    #[test]
    fn get_account_preferences_is_get_to_preferences_path() {
        let req = take_http(&mut get_account_preferences(BASE));
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/api/account/preferences");
    }

    #[test]
    fn save_account_preferences_puts_full_prefs_body() {
        let prefs = AccountPreferences::default();
        let req = take_http(&mut save_account_preferences(BASE, &prefs, None));
        assert_eq!(req.method, "PUT");
        assert_eq!(req.url, "https://api.example.com/api/account/preferences");
        assert!(body_as_json(&req).is_object());
    }

    #[test]
    fn delete_account_is_delete_to_account_root() {
        let req = take_http(&mut delete_account(BASE));
        assert_eq!(req.method, "DELETE");
        assert_eq!(req.url, "https://api.example.com/api/account");
    }

    #[test]
    fn list_mcp_tokens_is_get_to_tokens_collection() {
        let req = take_http(&mut list_mcp_tokens(BASE));
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/api/account/tokens");
    }

    #[test]
    fn create_mcp_token_posts_name_only_body() {
        let req = take_http(&mut create_mcp_token(BASE, "claude-cli"));
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.example.com/api/account/tokens");

        let body = body_as_json(&req);
        let object = body.as_object().expect("name-only body is an object");
        assert_eq!(object.len(), 1, "create token body has exactly one field");
        assert_eq!(body["name"], "claude-cli");
    }

    #[test]
    fn revoke_mcp_token_is_delete_to_id_path() {
        let req = take_http(&mut revoke_mcp_token(BASE, "01HMCP0000000000000000000"));
        assert_eq!(req.method, "DELETE");
        assert_eq!(
            req.url,
            "https://api.example.com/api/account/tokens/01HMCP0000000000000000000"
        );
    }

    #[test]
    fn list_mcp_audit_is_get_to_audit_path() {
        let req = take_http(&mut list_mcp_audit(BASE));
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://api.example.com/api/account/audit");
    }

    #[test]
    fn oauth_finalize_posts_params_to_oauth_finalize() {
        let params = OAuthFinalizeParams {
            response_type: "code".into(),
            client_id: "client-abc".into(),
            redirect_uri: "https://app.example.com/cb".into(),
            state: Some("xyz".into()),
            scope: Some("read".into()),
            code_challenge: "challenge".into(),
            code_challenge_method: "S256".into(),
        };
        let req = take_http(&mut oauth_finalize(BASE, &params));
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.example.com/oauth/finalize");

        let body = body_as_json(&req);
        assert_eq!(body["response_type"], "code");
        assert_eq!(body["client_id"], "client-abc");
        assert_eq!(body["redirect_uri"], "https://app.example.com/cb");
        assert_eq!(body["code_challenge_method"], "S256");
    }

    // ── Base URL trimming ────────────────────────────────────────────

    #[test]
    fn trailing_slash_in_base_url_produces_double_slash() {
        // api_base_url is concatenated as-is; callers must pass it without a
        // trailing slash.
        let req = take_http(&mut fetch_items("https://api.example.com/"));
        assert_eq!(req.url, "https://api.example.com//api/items");
    }
}

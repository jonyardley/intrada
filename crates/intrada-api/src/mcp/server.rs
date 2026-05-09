//! Axum handler that dispatches JSON-RPC requests to MCP methods + tools.
//!
//! Auth: reuses the existing `AuthUser` extractor — same Bearer-PAT path
//! that protects `/api/items` etc. PATs in auth-disabled (dev) mode resolve
//! to the empty user_id, matching the rest of the API.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;

use crate::auth::{AuthSource, AuthUser};
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;

use super::handlers;
use super::protocol::{
    error_code, Capabilities, Icon, InitializeResult, JsonRpcRequest, JsonRpcResponse, ServerInfo,
    ToolContent, ToolsCallParams, ToolsCallResult, ToolsCapability, ToolsListResult,
    PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION,
};
use super::tools;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(handle))
}

/// Top-level JSON-RPC handler. Routes by `method` to either MCP-protocol
/// methods (`initialize`, `tools/list`, `tools/call`) or returns a
/// JSON-RPC `METHOD_NOT_FOUND` error. Notifications (no `id`) get a 204.
#[tracing::instrument(
    name = "mcp.request",
    skip_all,
    fields(jsonrpc.method = %req.method)
)]
async fn handle(
    State(state): State<AppState>,
    // Read for the `serverInfo.icons` URL on `initialize` — derived
    // from the request's `Host` so dev / preview / prod all advertise
    // a working URL without an extra env var.
    headers: HeaderMap,
    // `source` is read by `dispatch_tool` to attribute write-tool calls
    // in `mcp_audit_log`. PAT writes record the resolved token_id; JWT
    // writes (browser/iOS session) record token_id=NULL (#528); Disabled
    // (local dev) skips the audit row entirely. See `services::audit::record_mcp_write`.
    AuthUser { user_id, source }: AuthUser,
    Json(req): Json<JsonRpcRequest>,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    if req.jsonrpc != "2.0" {
        let id = req.id.unwrap_or(Value::Null);
        return Json(JsonRpcResponse::error(
            id,
            error_code::INVALID_REQUEST,
            "jsonrpc must be \"2.0\"",
        ))
        .into_response();
    }

    // Notifications: no `id`, no response. Common ones: `notifications/initialized`.
    let id = match req.id {
        Some(v) => v,
        None => return StatusCode::NO_CONTENT.into_response(),
    };

    let response = match req.method.as_str() {
        "initialize" => Json(JsonRpcResponse::success(
            id,
            serde_json::to_value(InitializeResult {
                protocol_version: PROTOCOL_VERSION,
                capabilities: Capabilities {
                    tools: ToolsCapability {
                        list_changed: false,
                    },
                },
                server_info: ServerInfo {
                    name: SERVER_NAME,
                    version: SERVER_VERSION,
                    icons: vec![Icon {
                        src: icon_url(&headers),
                        mime_type: Some("image/svg+xml"),
                        // SVG vector — "any" tells the client a single
                        // asset covers every render size.
                        sizes: Some(vec!["any"]),
                    }],
                },
            })
            .expect("InitializeResult serializes"),
        ))
        .into_response(),

        "tools/list" => Json(JsonRpcResponse::success(
            id,
            serde_json::to_value(ToolsListResult {
                tools: tools::catalogue(),
            })
            .expect("ToolsListResult serializes"),
        ))
        .into_response(),

        "tools/call" => {
            let params: ToolsCallParams = match serde_json::from_value(req.params) {
                Ok(p) => p,
                Err(e) => {
                    return Json(JsonRpcResponse::error(
                        id,
                        error_code::INVALID_PARAMS,
                        format!("Invalid tools/call params: {e}"),
                    ))
                    .into_response();
                }
            };
            dispatch_tool(&state, &user_id, &source, params, id).await
        }

        // Notifications-as-requests: some clients send them with an id. Treat
        // as no-op success rather than method-not-found.
        method if method.starts_with("notifications/") => {
            Json(JsonRpcResponse::success(id, Value::Null)).into_response()
        }

        other => Json(JsonRpcResponse::error(
            id,
            error_code::METHOD_NOT_FOUND,
            format!("Method not found: {other}"),
        ))
        .into_response(),
    };

    response
}

/// Dispatch a `tools/call` to the appropriate handler. Returns a successful
/// JSON-RPC response — tool-level errors (validation, not-found) are
/// embedded in the result with `is_error: true` per the MCP spec, not
/// surfaced as JSON-RPC errors (those are reserved for protocol-level
/// problems).
///
/// Wrapped in a `mcp.tool` span so Sentry dashboards can group call
/// volume + latency + error rate by tool name and (hashed) identity.
/// `tool.name` is recorded eagerly; `user.id.hash`, `token.id.hash`,
/// and `result` are recorded inside the body.
#[tracing::instrument(
    name = "mcp.tool",
    skip_all,
    fields(
        tool.name = %params.name,
        user.id.hash = tracing::field::Empty,
        token.id.hash = tracing::field::Empty,
        result = tracing::field::Empty,
    )
)]
async fn dispatch_tool(
    state: &AppState,
    user_id: &str,
    source: &AuthSource,
    params: ToolsCallParams,
    id: Value,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    // Identity attributes — hashed so Sentry never sees raw user IDs
    // or token IDs. `user.id.hash` is recorded for every call (empty
    // string in `AuthSource::Disabled` mode); `token.id.hash` only for
    // PAT-authenticated calls (Jwt + Disabled leave it empty).
    {
        let span = tracing::Span::current();
        span.record("user.id.hash", crate::telemetry::hash_id(user_id).as_str());
        if let AuthSource::Pat { token_id } = source {
            span.record(
                "token.id.hash",
                crate::telemetry::hash_id(token_id).as_str(),
            );
        }
    }

    let conn = state.conn();
    // Keep a clone available for the post-match audit recording —
    // each match arm moves `conn` into its closure.
    let conn_for_audit = conn.clone();
    // Hold onto a clone of the raw arguments so the audit-log row can
    // record `args_hash` even after the typed parse moves the JSON.
    let raw_args = params.arguments.clone();
    let tool_name = params.name.clone();
    let result = match params.name.as_str() {
        tools::LIST_ITEMS => {
            parse_and_run(params.arguments, |args| async move {
                handlers::list_items(&conn, user_id, args).await
            })
            .await
        }

        tools::GET_ITEM => {
            parse_and_run(params.arguments, |args| async move {
                handlers::get_item(&conn, user_id, args).await
            })
            .await
        }

        tools::LIST_SETS => {
            parse_and_run::<EmptyArgs, _, _>(params.arguments, |_| async move {
                handlers::list_sets(&conn, user_id).await
            })
            .await
        }

        tools::GET_SET => {
            parse_and_run(params.arguments, |args| async move {
                handlers::get_set(&conn, user_id, args).await
            })
            .await
        }

        tools::LIST_SESSIONS => {
            parse_and_run(params.arguments, |args| async move {
                handlers::list_sessions(&conn, user_id, args).await
            })
            .await
        }

        tools::GET_SESSION => {
            parse_and_run(params.arguments, |args| async move {
                handlers::get_session(&conn, user_id, args).await
            })
            .await
        }

        tools::GET_PRACTICE_SUMMARY => {
            parse_and_run(params.arguments, |args| async move {
                handlers::get_practice_summary(&conn, user_id, args).await
            })
            .await
        }

        // Phase 4 write tools.
        tools::CREATE_ITEM => {
            parse_and_run(params.arguments, |args| async move {
                handlers::create_item(&conn, user_id, args).await
            })
            .await
        }

        tools::UPDATE_ITEM => {
            parse_and_run(params.arguments, |args| async move {
                handlers::update_item(&conn, user_id, args).await
            })
            .await
        }

        tools::DELETE_ITEM => {
            parse_and_run(params.arguments, |args| async move {
                handlers::delete_item(&conn, user_id, args).await
            })
            .await
        }

        tools::CREATE_SET => {
            parse_and_run(params.arguments, |args| async move {
                handlers::create_set(&conn, user_id, args).await
            })
            .await
        }

        tools::UPDATE_SET => {
            parse_and_run(params.arguments, |args| async move {
                handlers::update_set(&conn, user_id, args).await
            })
            .await
        }

        tools::BULK_IMPORT_ITEMS => {
            // bulk_import handles its own audit recording — `dry_run=true`
            // must not produce an audit row, so the decision sits with
            // the handler that knows whether a write actually happened.
            let raw = raw_args.clone();
            let source = source.clone();
            parse_and_run(params.arguments, move |args| async move {
                handlers::bulk_import_items(&conn, &source, user_id, &raw, args).await
            })
            .await
        }

        unknown => {
            tracing::Span::current().record("result", "method_not_found");
            return Json(JsonRpcResponse::error(
                id,
                error_code::METHOD_NOT_FOUND,
                format!("Unknown tool: {unknown}"),
            ))
            .into_response();
        }
    };

    tracing::Span::current().record(
        "result",
        match &result {
            Ok(_) => "ok",
            Err(_) => "error",
        },
    );

    // Audit single-write tools after a successful execution. Read tools
    // and the bulk-import tool are excluded — bulk_import audits itself.
    // JWT writes are now included (#528): the row is recorded with token_id=NULL.
    if result.is_ok() && tools::SINGLE_WRITE_TOOLS.contains(&tool_name.as_str()) {
        services::audit::record_mcp_write(&conn_for_audit, source, user_id, &tool_name, &raw_args)
            .await;
    }

    match result {
        Ok(value) => Json(JsonRpcResponse::success(
            id,
            serde_json::to_value(ToolsCallResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string(&value).unwrap_or_else(|e| {
                        format!("{{\"error\":\"failed to serialize result: {e}\"}}")
                    }),
                }],
                is_error: false,
            })
            .expect("ToolsCallResult serializes"),
        ))
        .into_response(),

        Err(api_err) => Json(JsonRpcResponse::success(
            id,
            serde_json::to_value(ToolsCallResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string(&serde_json::json!({
                        "error": format_api_error(&api_err),
                    }))
                    .unwrap_or_else(|_| "{}".into()),
                }],
                is_error: true,
            })
            .expect("ToolsCallResult serializes"),
        ))
        .into_response(),
    }
}

#[derive(Deserialize, Default)]
struct EmptyArgs;

/// Parse `arguments` into the typed `A` and pass to the async closure. Any
/// parse failure becomes an `ApiError::Validation` so it's surfaced as a
/// tool-level error (`isError: true`) rather than a JSON-RPC error.
///
/// Missing/null `arguments` is normalised to `{}` so tools whose args are
/// all-optional don't require the agent to pass an explicit empty object;
/// tools with required fields still error cleanly via serde.
async fn parse_and_run<A, F, Fut>(args: Value, run: F) -> Result<Value, ApiError>
where
    A: serde::de::DeserializeOwned,
    F: FnOnce(A) -> Fut,
    Fut: std::future::Future<Output = Result<Value, ApiError>>,
{
    let args = if args.is_null() {
        Value::Object(Default::default())
    } else {
        args
    };
    let parsed: A = serde_json::from_value(args)
        .map_err(|e| ApiError::Validation(format!("Invalid arguments: {e}")))?;
    run(parsed).await
}

fn format_api_error(err: &ApiError) -> String {
    match err {
        ApiError::Validation(msg) => format!("Validation error: {msg}"),
        ApiError::NotFound(msg) => format!("Not found: {msg}"),
        ApiError::Unauthorized(msg) => format!("Unauthorized: {msg}"),
        ApiError::Internal(msg) => {
            // Don't leak internal details to the agent — just say "internal error".
            tracing::warn!(?msg, "MCP tool internal error");
            "Internal error".to_string()
        }
    }
}

/// Build the absolute URL to `/icon.svg` from the request headers, so
/// the MCP `serverInfo.icons` field always points at the same host the
/// client used to reach us. Same scheme/host derivation as the OAuth
/// discovery doc — see `routes/oauth.rs::discovery`. Behind a
/// TLS-terminating proxy (Fly.io / Cloudflare) we can't detect scheme
/// reliably, so we default to https; http only on local dev hosts.
fn icon_url(headers: &HeaderMap) -> String {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("intrada-api.fly.dev");
    let scheme = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    format!("{scheme}://{host}/icon.svg")
}

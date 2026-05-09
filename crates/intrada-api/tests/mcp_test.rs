mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn initialize_returns_protocol_version_and_server_info() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {},
            "id": 1
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let v: Value = common::json(&body);
    assert_eq!(v["jsonrpc"], "2.0");
    assert_eq!(v["id"], 1);
    let result = &v["result"];
    assert!(
        result["protocolVersion"].as_str().is_some(),
        "missing protocolVersion: {result:?}"
    );
    assert_eq!(result["serverInfo"]["name"], "intrada");
    assert!(result["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn initialize_advertises_server_icon_with_request_host() {
    // MCP spec 2025-11-25 (SEP-973): `serverInfo.icons` lets the
    // server advertise a logo. claude.ai's connector UI doesn't render
    // it today (anthropics/claude-ai-mcp#152), but locking in the
    // contract now means we don't quietly regress when they ship
    // support. URL is derived from the request's `Host` so dev /
    // preview / prod all advertise something fetchable.
    let app = common::setup_test_app().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/mcp")
        .header("host", "intrada-api.fly.dev")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {},
                "id": 1,
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    let v: Value = common::json(&body);

    let icons = v["result"]["serverInfo"]["icons"]
        .as_array()
        .expect("serverInfo.icons should be an array");
    assert_eq!(icons.len(), 1, "expected exactly one icon entry");
    let icon = &icons[0];
    assert_eq!(
        icon["src"], "https://intrada-api.fly.dev/icon.svg",
        "icon URL should be derived from the request's Host header"
    );
    assert_eq!(icon["mimeType"], "image/svg+xml");
    assert_eq!(icon["sizes"][0], "any");
}

#[tokio::test]
async fn icon_endpoint_serves_svg() {
    // The URL `serverInfo.icons[].src` points at must actually serve
    // something — this test guards the round-trip end-to-end so a
    // future router refactor that drops the asset route is caught
    // before deploy.
    let app = common::setup_test_app().await;
    let req = Request::builder()
        .method("GET")
        .uri("/icon.svg")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("image/svg+xml"),
    );
    let body = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    let body_str = std::str::from_utf8(&body).expect("icon should be valid utf-8");
    assert!(body_str.contains("<svg"), "expected SVG markup");
}

#[tokio::test]
async fn tools_list_returns_full_catalogue() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 2
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let v: Value = common::json(&body);
    let tools = v["result"]["tools"].as_array().expect("tools array");
    // 7 reads (Phase 3) + 6 writes (Phase 4) = 13.
    assert_eq!(tools.len(), 13, "expected 13 tools, got {}", tools.len());

    let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    // Reads.
    assert!(names.contains(&"list_items"));
    assert!(names.contains(&"get_item"));
    assert!(names.contains(&"list_sets"));
    assert!(names.contains(&"get_set"));
    assert!(names.contains(&"list_sessions"));
    assert!(names.contains(&"get_session"));
    assert!(names.contains(&"get_practice_summary"));
    // Writes.
    assert!(names.contains(&"create_item"));
    assert!(names.contains(&"update_item"));
    assert!(names.contains(&"delete_item"));
    assert!(names.contains(&"create_set"));
    assert!(names.contains(&"update_set"));
    assert!(names.contains(&"bulk_import_items"));

    // Every tool must declare an inputSchema (agents rely on this for
    // argument validation).
    for tool in tools {
        assert!(
            tool["inputSchema"].is_object(),
            "{} missing inputSchema",
            tool["name"]
        );
    }
}

#[tokio::test]
async fn tools_call_list_items_returns_items_as_json_text() {
    let app = common::setup_test_app().await;

    // Seed an item via the regular HTTP path.
    let (status, _) = common::post_json(
        app.clone(),
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Now ask via MCP.
    let (status, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "list_items",
                "arguments": {}
            },
            "id": 3
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let v: Value = common::json(&body);
    let result = &v["result"];
    assert_eq!(
        result["isError"],
        Value::Null,
        "expected isError omitted (=false) on success"
    );
    let content = result["content"].as_array().expect("content array");
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    let text = content[0]["text"].as_str().expect("text payload");
    let inner: Value = serde_json::from_str(text).expect("text is JSON");
    let items = inner.as_array().expect("items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Clair de Lune");
}

#[tokio::test]
async fn tools_call_list_items_filters_by_kind() {
    let app = common::setup_test_app().await;

    common::post_json(
        app.clone(),
        "/api/items",
        json!({"title": "Étude", "kind": "exercise", "tags": []}),
    )
    .await;
    common::post_json(
        app.clone(),
        "/api/items",
        json!({"title": "Sonata", "kind": "piece", "tags": []}),
    )
    .await;

    let (_, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {"name": "list_items", "arguments": {"kind": "exercise"}},
            "id": 4
        }),
    )
    .await;
    let v: Value = common::json(&body);
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Étude");
}

#[tokio::test]
async fn tools_call_get_item_unknown_returns_is_error() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "get_item",
                "arguments": {"id": "01HXNONE000000000000000000"}
            },
            "id": 5
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v: Value = common::json(&body);
    // Tool-level errors return JSON-RPC success with isError: true on result.
    assert!(v["error"].is_null(), "should not be a JSON-RPC error");
    assert_eq!(v["result"]["isError"], true);
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Not found"), "got: {text}");
}

#[tokio::test]
async fn tools_call_unknown_tool_returns_method_not_found() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {"name": "make_coffee", "arguments": {}},
            "id": 6
        }),
    )
    .await;
    let v: Value = common::json(&body);
    assert_eq!(v["error"]["code"], -32601);
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown tool"));
}

#[tokio::test]
async fn unknown_method_returns_method_not_found() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "nonexistent/method",
            "id": 7
        }),
    )
    .await;
    let v: Value = common::json(&body);
    assert_eq!(v["error"]["code"], -32601);
}

#[tokio::test]
async fn invalid_jsonrpc_version_returns_invalid_request() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "1.0",
            "method": "initialize",
            "id": 8
        }),
    )
    .await;
    let v: Value = common::json(&body);
    assert_eq!(v["error"]["code"], -32600);
}

#[tokio::test]
async fn notification_returns_no_content() {
    // No `id` → notification → 204 No Content per JSON-RPC spec.
    let app = common::setup_test_app().await;
    let (status, _) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn get_practice_summary_returns_aggregates() {
    let app = common::setup_test_app().await;

    // Seed a session with two entries.
    let session_payload = json!({
        "session_notes": null,
        "session_intention": null,
        "started_at": "2026-05-01T10:00:00Z",
        "completed_at": "2026-05-01T10:30:00Z",
        "total_duration_secs": 1800,
        "completion_status": "Completed",
        "entries": [
            {
                "id": "entry-001",
                "item_id": "i1",
                "item_title": "Etude 1",
                "item_type": "exercise",
                "position": 0,
                "duration_secs": 600,
                "status": "Completed",
                "notes": null,
                "score": 4
            },
            {
                "id": "entry-002",
                "item_id": "i2",
                "item_title": "Sonata",
                "item_type": "piece",
                "position": 1,
                "duration_secs": 1200,
                "status": "Completed",
                "notes": null,
                "score": 3
            }
        ]
    });
    let (status, _) = common::post_json(app.clone(), "/api/sessions", session_payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "get_practice_summary",
                "arguments": {
                    "start": "2026-05-01T00:00:00Z",
                    "end": "2026-05-02T00:00:00Z"
                }
            },
            "id": 9
        }),
    )
    .await;

    let v: Value = common::json(&body);
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let inner: Value = serde_json::from_str(text).unwrap();
    assert_eq!(inner["sessions_count"], 1);
    assert_eq!(inner["total_minutes"], 30);
    assert_eq!(inner["items_practiced"], 2);
    assert_eq!(inner["entries_count"], 2);
    // Average score: (4 + 3) / 2 = 3.5
    assert!((inner["average_score"].as_f64().unwrap() - 3.5).abs() < 1e-6);
    // Per-item array sorted by total_minutes descending — Sonata (20m) first.
    let items = inner["items"].as_array().unwrap();
    assert_eq!(items[0]["title"], "Sonata");
    assert_eq!(items[0]["total_minutes"], 20);
    assert_eq!(items[1]["title"], "Etude 1");
    assert_eq!(items[1]["total_minutes"], 10);
}

/// Sub-minute session durations must accumulate to whole minutes — i.e.
/// the aggregation must sum seconds first and divide once, not divide
/// per-session. Before the fix, two 90s sessions read as 2min total
/// instead of the correct 3min.
#[tokio::test]
async fn get_practice_summary_handles_sub_minute_sessions() {
    let app = common::setup_test_app().await;

    let make_session = |seq: u32, started: &str, completed: &str, secs: u64| -> serde_json::Value {
        json!({
            "started_at": started,
            "completed_at": completed,
            "total_duration_secs": secs,
            "completion_status": "Completed",
            "entries": [
                {
                    "id": format!("entry-{seq:03}"),
                    "item_id": "item-x",
                    "item_title": "Étude",
                    "item_type": "exercise",
                    "position": 0,
                    "duration_secs": secs,
                    "status": "Completed",
                    "notes": null
                }
            ]
        })
    };

    // Two 90-second sessions. Per-session integer division would give
    // 1 + 1 = 2 minutes; correct sum-first-then-divide gives 3.
    common::post_json(
        app.clone(),
        "/api/sessions",
        make_session(1, "2026-05-01T10:00:00Z", "2026-05-01T10:01:30Z", 90),
    )
    .await;
    common::post_json(
        app.clone(),
        "/api/sessions",
        make_session(2, "2026-05-01T11:00:00Z", "2026-05-01T11:01:30Z", 90),
    )
    .await;

    let (_, body) = common::post_json(
        app,
        "/api/mcp",
        json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "get_practice_summary",
                "arguments": {
                    "start": "2026-05-01T00:00:00Z",
                    "end": "2026-05-02T00:00:00Z"
                }
            },
            "id": 100
        }),
    )
    .await;

    let v: Value = common::json(&body);
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let inner: Value = serde_json::from_str(text).unwrap();
    assert_eq!(inner["sessions_count"], 2);
    assert_eq!(inner["total_minutes"], 3, "two 90s sessions = 3min total");
}

#[tokio::test]
async fn pat_authenticates_mcp_call() {
    // The PAT path must reach /api/mcp end-to-end. Existing PAT tests
    // only cover /api/items; this guards the MCP route specifically.
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "mcp-test"}),
    )
    .await;
    let v: Value = common::json(&body);
    let token = v["token"].as_str().unwrap().to_string();

    // Use the PAT to invoke an MCP method.
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/mcp")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "tools/list",
                "id": 1
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = http_body_util::BodyExt::collect(resp.into_body())
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    let v: Value = common::json(&body);
    assert!(v["result"]["tools"].is_array());
}

#[tokio::test]
async fn revoked_pat_cannot_reach_mcp() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "to-be-revoked"}),
    )
    .await;
    let v: Value = common::json(&body);
    let id = v["id"].as_str().unwrap().to_string();
    let token = v["token"].as_str().unwrap().to_string();

    // Revoke first, then try to use it.
    let (status, _) = common::delete(app.clone(), &format!("/api/account/tokens/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/mcp")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "tools/list",
                "id": 1
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "revoked PAT must not reach /api/mcp"
    );
}

#[tokio::test]
async fn cors_preflight_returns_permissive_headers_for_mcp() {
    // CORS decision (#481): permissive on /api/mcp/*, strict elsewhere.
    // A preflight OPTIONS from an arbitrary origin must succeed; the
    // strict-allowlist routes would reject the same origin.
    let app = common::setup_test_app().await;

    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/api/mcp")
        .header("Origin", "https://example.invalid")
        .header("Access-Control-Request-Method", "POST")
        .header(
            "Access-Control-Request-Headers",
            "authorization,content-type",
        )
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    let status = resp.status();
    let allow_origin = resp
        .headers()
        .get("access-control-allow-origin")
        .map(|v| v.to_str().unwrap().to_string());
    assert!(
        status.is_success(),
        "preflight should succeed; got {status}"
    );
    assert_eq!(
        allow_origin.as_deref(),
        Some("*"),
        "MCP preflight should advertise permissive origin; got {allow_origin:?}"
    );
}

// ── Phase 4 write tools ────────────────────────────────────────────────

/// Helper: drive an MCP `tools/call` against an authed `Router` using a PAT.
/// Returns the parsed response body.
async fn mcp_call_with_pat(
    app: axum::Router,
    token: &str,
    name: &str,
    arguments: Value,
    id: u32,
) -> Value {
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/mcp")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {"name": name, "arguments": arguments},
                "id": id
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    let body = http_body_util::BodyExt::collect(resp.into_body())
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    common::json(&body)
}

/// Helper: mint a PAT via the public API and return the bearer string.
async fn mint_pat(app: axum::Router, name: &str) -> String {
    let (_, body) = common::post_json(app, "/api/account/tokens", json!({"name": name})).await;
    let v: Value = common::json(&body);
    v["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_item_via_mcp_writes_and_audits() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "create-test").await;

    let response = mcp_call_with_pat(
        app.clone(),
        &token,
        "create_item",
        json!({"title": "Goldberg Variations", "kind": "piece", "composer": "J.S. Bach", "tags": []}),
        1,
    )
    .await;

    assert_eq!(response["result"]["isError"], Value::Null);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let item: Value = serde_json::from_str(text).unwrap();
    assert_eq!(item["title"], "Goldberg Variations");
    assert_eq!(item["kind"], "piece");

    // Audit log should have one row for this write.
    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let entries: Vec<Value> = common::json(&audit_body);
    assert_eq!(entries.len(), 1, "expected exactly one audit row");
    assert_eq!(entries[0]["tool"], "create_item");
    assert!(!entries[0]["args_hash"].as_str().unwrap().is_empty());
    // args_hash must NOT contain the literal title — it's a hash, not a copy.
    assert!(!entries[0]["args_hash"]
        .as_str()
        .unwrap()
        .contains("Goldberg"));
}

#[tokio::test]
async fn read_tools_do_not_audit() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "read-test").await;

    // Multiple read calls.
    for id in 0..3 {
        mcp_call_with_pat(app.clone(), &token, "list_items", json!({}), id).await;
    }

    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let entries: Vec<Value> = common::json(&audit_body);
    assert!(
        entries.is_empty(),
        "read tools must not produce audit rows; got {} entries",
        entries.len()
    );
}

#[tokio::test]
async fn update_item_via_mcp_writes_and_audits() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "update-test").await;

    // Seed an item via the regular HTTP path (not MCP, so this doesn't audit).
    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({"title": "Original", "kind": "piece", "composer": "Composer", "tags": []}),
    )
    .await;
    let v: Value = common::json(&body);
    let id = v["id"].as_str().unwrap().to_string();

    let response = mcp_call_with_pat(
        app.clone(),
        &token,
        "update_item",
        json!({"id": id, "title": "Renamed"}),
        2,
    )
    .await;
    assert_eq!(response["result"]["isError"], Value::Null);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let item: Value = serde_json::from_str(text).unwrap();
    assert_eq!(item["title"], "Renamed");

    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let entries: Vec<Value> = common::json(&audit_body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["tool"], "update_item");
}

#[tokio::test]
async fn delete_item_via_mcp_writes_and_audits() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "delete-test").await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({"title": "To delete", "kind": "exercise", "tags": []}),
    )
    .await;
    let v: Value = common::json(&body);
    let id = v["id"].as_str().unwrap().to_string();

    let response =
        mcp_call_with_pat(app.clone(), &token, "delete_item", json!({"id": id}), 3).await;
    assert_eq!(response["result"]["isError"], Value::Null);

    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let entries: Vec<Value> = common::json(&audit_body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["tool"], "delete_item");
}

#[tokio::test]
async fn bulk_import_dry_run_returns_preview_without_writing_or_auditing() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "bulk-dry-test").await;

    let response = mcp_call_with_pat(
        app.clone(),
        &token,
        "bulk_import_items",
        json!({
            "dry_run": true,
            "items": [
                {"title": "Bach: Cello Suite No. 1", "kind": "piece", "composer": "J.S. Bach", "tags": []},
                {"title": "Bach: Cello Suite No. 2", "kind": "piece", "composer": "J.S. Bach", "tags": []},
                {"title": "", "kind": "piece", "tags": []}  // invalid — empty title
            ]
        }),
        4,
    )
    .await;

    assert_eq!(response["result"]["isError"], Value::Null);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let preview: Value = serde_json::from_str(text).unwrap();
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["valid_count"], 2);
    assert_eq!(preview["invalid_count"], 1);
    let items = preview["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["valid"], true);
    assert_eq!(items[1]["valid"], true);
    assert_eq!(items[2]["valid"], false);

    // Library should be empty — preview must not write.
    let (_, items_body) = common::get(app.clone(), "/api/items").await;
    let items: Vec<Value> = common::json(&items_body);
    assert!(items.is_empty(), "dry_run must not write");

    // No audit row.
    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let audit: Vec<Value> = common::json(&audit_body);
    assert!(audit.is_empty(), "dry_run must not audit");
}

#[tokio::test]
async fn bulk_import_non_dry_run_writes_all_or_nothing_and_audits() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "bulk-write-test").await;

    // Refuse a write that contains any invalid item.
    let response = mcp_call_with_pat(
        app.clone(),
        &token,
        "bulk_import_items",
        json!({
            "dry_run": false,
            "items": [
                {"title": "Valid 1", "kind": "piece", "composer": "X", "tags": []},
                {"title": "", "kind": "piece", "tags": []}
            ]
        }),
        5,
    )
    .await;
    assert_eq!(response["result"]["isError"], true);

    // Library still empty.
    let (_, items_body) = common::get(app.clone(), "/api/items").await;
    let items: Vec<Value> = common::json(&items_body);
    assert!(
        items.is_empty(),
        "all-or-nothing: invalid item aborts write"
    );

    // No audit row for the failed write.
    let (_, audit_body) = common::get(app.clone(), "/api/account/audit").await;
    let audit: Vec<Value> = common::json(&audit_body);
    assert!(
        audit.is_empty(),
        "failed bulk_import must not audit; got {audit:?}"
    );

    // Now retry with valid items only.
    let response = mcp_call_with_pat(
        app.clone(),
        &token,
        "bulk_import_items",
        json!({
            "dry_run": false,
            "items": [
                {"title": "Bach Suite 1", "kind": "piece", "composer": "Bach", "tags": []},
                {"title": "Bach Suite 2", "kind": "piece", "composer": "Bach", "tags": []},
                {"title": "Bach Suite 3", "kind": "piece", "composer": "Bach", "tags": []}
            ]
        }),
        6,
    )
    .await;
    assert_eq!(response["result"]["isError"], Value::Null);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    assert_eq!(result["created_count"], 3);

    // Library should have 3 items.
    let (_, items_body) = common::get(app.clone(), "/api/items").await;
    let items: Vec<Value> = common::json(&items_body);
    assert_eq!(items.len(), 3);

    // Single audit row for the successful bulk import (not one-per-item).
    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let audit: Vec<Value> = common::json(&audit_body);
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0]["tool"], "bulk_import_items");
}

#[tokio::test]
async fn audit_log_endpoint_returns_newest_first() {
    let app = common::setup_test_app().await;
    let token = mint_pat(app.clone(), "audit-order").await;

    // Three sequential writes.
    for i in 0..3 {
        mcp_call_with_pat(
            app.clone(),
            &token,
            "create_item",
            json!({
                "title": format!("Item {i}"),
                "kind": "piece",
                "composer": "Test",
                "tags": []
            }),
            10 + i,
        )
        .await;
    }

    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let entries: Vec<Value> = common::json(&audit_body);
    assert_eq!(entries.len(), 3);
    // Newest first: created_at[0] >= created_at[1] >= created_at[2].
    let ts: Vec<&str> = entries
        .iter()
        .map(|e| e["created_at"].as_str().unwrap())
        .collect();
    assert!(
        ts[0] >= ts[1] && ts[1] >= ts[2],
        "audit list must be newest-first; got {ts:?}"
    );
}

#[tokio::test]
async fn audit_log_excludes_other_users() {
    // True isolation test — seed a foreign-user audit row directly via
    // the connection, then confirm the GET endpoint scoped to "" (the
    // disabled-mode user) doesn't see it.
    let (app, conn) = common::setup_test_app_with_conn(None, "http://localhost:3000").await;
    let token = mint_pat(app.clone(), "isolation").await;

    // Insert a foreign row attributed to user_id="other_user".
    intrada_api::db::audit::insert(
        &conn,
        "01HXFOREIGN0000000000000000",
        Some("01HXFOREIGNTOKEN0000000000"),
        "other_user",
        "create_item",
        "deadbeef".repeat(8).as_str(),
        chrono::Utc::now(),
    )
    .await
    .expect("insert foreign audit row");

    // Make one legitimate write as the disabled-mode user_id ("").
    mcp_call_with_pat(
        app.clone(),
        &token,
        "create_item",
        json!({"title": "Mine", "kind": "piece", "composer": "Me", "tags": []}),
        20,
    )
    .await;

    // The audit endpoint scoped to "" must return only the legitimate
    // write — never the seeded foreign row.
    let (_, audit_body) = common::get(app, "/api/account/audit").await;
    let entries: Vec<Value> = common::json(&audit_body);
    assert_eq!(entries.len(), 1, "must not include foreign-user rows");
    // Sanity: the entry belongs to the legitimate write (token id != foreign).
    assert_ne!(entries[0]["token_id"], "01HXFOREIGNTOKEN0000000000");
}

#[tokio::test]
async fn jwt_write_produces_audit_row_with_null_token_id() {
    // Verifies the #528 fix: writes attributed to AuthSource::Jwt record
    // a row with token_id = NULL, visible in the audit list as JSON null.
    //
    // We use the direct `record_mcp_write` path rather than a full HTTP
    // roundtrip (which would require a real Clerk JWT) so the test stays
    // self-contained. This exercises the exact code path that dispatch_tool
    // hits when source == AuthSource::Jwt.
    let (app, conn) = common::setup_test_app_with_conn(None, "http://localhost:3000").await;

    intrada_api::services::audit::record_mcp_write(
        &conn,
        &intrada_api::auth::AuthSource::Jwt,
        "", // disabled-mode user_id (empty string)
        "create_item",
        &serde_json::json!({"title": "JWT write test"}),
    )
    .await;

    let (status, audit_body) = common::get(app, "/api/account/audit").await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let entries: Vec<serde_json::Value> = common::json(&audit_body);
    assert_eq!(entries.len(), 1, "expected one audit row for JWT write");

    let entry = &entries[0];
    assert_eq!(entry["tool"], "create_item");
    // token_id must be JSON null — not a string, not missing.
    assert_eq!(
        entry["token_id"],
        serde_json::Value::Null,
        "JWT write must produce token_id = null in audit row; got {:?}",
        entry["token_id"]
    );
    // token_name and token_prefix are also null (no token to join).
    assert_eq!(entry["token_name"], serde_json::Value::Null);
}

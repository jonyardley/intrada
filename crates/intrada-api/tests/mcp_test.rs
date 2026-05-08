mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

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
    assert_eq!(tools.len(), 7, "expected 7 tools, got {}", tools.len());

    let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"list_items"));
    assert!(names.contains(&"get_item"));
    assert!(names.contains(&"list_sets"));
    assert!(names.contains(&"get_set"));
    assert!(names.contains(&"list_sessions"));
    assert!(names.contains(&"get_session"));
    assert!(names.contains(&"get_practice_summary"));

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

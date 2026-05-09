//! JSON-RPC 2.0 + minimal MCP protocol types.
//!
//! We hand-roll the protocol rather than depend on `rmcp` because:
//! 1. Per-request user context is awkward to plumb through `rmcp`'s
//!    session-factory closure.
//! 2. Read-only tools don't need streaming/SSE/sessions.
//! 3. Direct integration with the existing `AuthUser` extractor and
//!    `services::*` layer is cleaner.
//!
//! If we later need streaming or want to drop maintenance burden, swapping
//! to `rmcp` is a focused refactor — the on-the-wire protocol is the same.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP protocol version we advertise. Bumping this requires reviewing any
/// behaviour that changed between versions.
// Bumped to 2025-11-25 to advertise support for the `icons` field on
// `serverInfo` (added in that revision via SEP-973). The handler set
// we expose — `initialize`, `tools/list`, `tools/call`, plus
// `notifications/*` no-ops — is unchanged across all post-2024-11-05
// revisions, so declaring the latest is safe even though we don't
// implement any other 2025-11-25 additions today.
pub const PROTOCOL_VERSION: &str = "2025-11-25";

pub const SERVER_NAME: &str = "intrada";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Standard JSON-RPC 2.0 error codes.
pub mod error_code {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// Inbound JSON-RPC request. We accept both notifications (no `id`) and
/// regular requests.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    /// `None` means the message is a notification — no response is sent.
    #[serde(default)]
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

impl JsonRpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// `initialize` response payload — server announces protocol version,
/// capabilities, and identity.
#[derive(Debug, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: &'static str,
    pub capabilities: Capabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct Capabilities {
    /// Empty object signals "tools are supported"; absence would mean
    /// the client shouldn't bother calling `tools/list`.
    pub tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    /// We don't push `tools/list_changed` notifications today (tools are
    /// statically defined). `false` is the spec-compliant signal.
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: &'static str,
    pub version: &'static str,
    /// Server icons (MCP spec 2025-11-25, SEP-973). Skipped during
    /// serialization when empty so the response shape on older
    /// protocol revisions is unchanged. claude.ai's connector UI does
    /// not render this today (anthropics/claude-ai-mcp#152), but
    /// shipping it now means the icon lights up automatically the
    /// day they ship support — no further server change needed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<Icon>,
}

/// Icon descriptor per MCP spec 2025-11-25. `src` is required
/// (https:// or `data:` URI); the rest are advisory hints to the
/// client. `theme` ("light" / "dark") lets servers ship paired icons
/// when only one renders well on the user's chosen theme — we don't
/// today (single white-on-purple SVG renders fine on either).
#[derive(Debug, Serialize)]
pub struct Icon {
    pub src: String,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<&'static str>>,
}

/// `tools/list` response payload.
#[derive(Debug, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDefinition>,
}

/// A single tool definition surfaced via `tools/list`.
#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    /// JSON Schema describing the tool's `arguments` shape. Hand-written
    /// rather than derived because our argument types are small and the
    /// `description` strings on each property are tuned for the agent.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// `tools/call` request params.
#[derive(Debug, Deserialize)]
pub struct ToolsCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

/// `tools/call` response. Results are wrapped in a `content` array per the
/// MCP spec; for our reads-only tools we always return a single
/// JSON-stringified text item.
#[derive(Debug, Serialize)]
pub struct ToolsCallResult {
    pub content: Vec<ToolContent>,
    /// `true` means the tool ran but the operation failed (validation,
    /// not-found). Distinct from a JSON-RPC error which signals a
    /// protocol-level problem.
    #[serde(rename = "isError", skip_serializing_if = "is_false")]
    pub is_error: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

fn is_false(v: &bool) -> bool {
    !v
}

//! Tool catalogue surfaced via `tools/list`.
//!
//! Each tool has:
//! - `name` — what the agent calls
//! - `description` — what the agent reads to decide *whether* to call it.
//!   These descriptions are tuned for the agent (user-facing terminology
//!   like "piece" / "exercise" / "routine") even though wire-protocol
//!   names match the API (`items`, `sets`).
//! - `input_schema` — JSON Schema for `arguments`, validated at call time.

use serde_json::json;

use super::protocol::ToolDefinition;

pub const LIST_ITEMS: &str = "list_items";
pub const GET_ITEM: &str = "get_item";
pub const LIST_SETS: &str = "list_sets";
pub const GET_SET: &str = "get_set";
pub const LIST_SESSIONS: &str = "list_sessions";
pub const GET_SESSION: &str = "get_session";
pub const GET_PRACTICE_SUMMARY: &str = "get_practice_summary";

pub fn catalogue() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: LIST_ITEMS,
            description: "List the user's library items — pieces and exercises they're working on. Optionally filter by `kind` (\"piece\" or \"exercise\").",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "enum": ["piece", "exercise"],
                        "description": "Optional filter. Omit to return both pieces and exercises."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: GET_ITEM,
            description: "Fetch a single item (piece or exercise) by its id, including title, composer, key signature, tempo, notes, and tags.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The item's id (ULID)." }
                },
                "required": ["id"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: LIST_SETS,
            description: "List the user's routines — ordered groups of items they practice together as a sequence.",
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: GET_SET,
            description: "Fetch a single routine by its id, including its name and the ordered list of items inside it.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The routine's id (ULID)." }
                },
                "required": ["id"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: LIST_SESSIONS,
            description: "List the user's practice sessions, optionally filtered by start time (`session.started_at`). Each session has a start time, total duration, and a setlist of items practiced with per-item scores.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Optional RFC 3339 lower bound (inclusive); compared against session.started_at."
                    },
                    "end": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Optional RFC 3339 upper bound (inclusive); compared against session.started_at."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: GET_SESSION,
            description: "Fetch a single practice session by id with full per-item details (durations, scores, notes, intentions).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The session's id (ULID)." }
                },
                "required": ["id"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: GET_PRACTICE_SUMMARY,
            description: "Aggregate practice statistics over a date range — total minutes, session count, distinct items practiced, and average score. Use this for typical \"how was my week\" questions instead of fetching every session and aggregating client-side.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start": {
                        "type": "string",
                        "format": "date-time",
                        "description": "RFC 3339 lower bound (inclusive)."
                    },
                    "end": {
                        "type": "string",
                        "format": "date-time",
                        "description": "RFC 3339 upper bound (inclusive)."
                    }
                },
                "required": ["start", "end"],
                "additionalProperties": false
            }),
        },
    ]
}

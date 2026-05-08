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

// Phase 4 write tools.
pub const CREATE_ITEM: &str = "create_item";
pub const UPDATE_ITEM: &str = "update_item";
pub const DELETE_ITEM: &str = "delete_item";
pub const CREATE_SET: &str = "create_set";
pub const UPDATE_SET: &str = "update_set";
pub const BULK_IMPORT_ITEMS: &str = "bulk_import_items";

/// All write-tool names that the dispatcher should audit on success
/// (single writes only — `bulk_import_items` records audit internally
/// because the `dry_run=true` branch must NOT audit).
pub const SINGLE_WRITE_TOOLS: &[&str] = &[
    CREATE_ITEM,
    UPDATE_ITEM,
    DELETE_ITEM,
    CREATE_SET,
    UPDATE_SET,
];

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

        // ── Phase 4 write tools ─────────────────────────────────────────

        ToolDefinition {
            name: CREATE_ITEM,
            description: "Create a piece or exercise in the user's library. Returns the new item with its server-assigned id.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Display name for the item." },
                    "kind": {
                        "type": "string",
                        "enum": ["piece", "exercise"],
                        "description": "Whether this is a piece (e.g. \"Clair de Lune\") or an exercise (e.g. \"Hanon No. 1\")."
                    },
                    "composer": {
                        "type": ["string", "null"],
                        "description": "Optional composer/author."
                    },
                    "key": {
                        "type": ["string", "null"],
                        "description": "Optional key signature, e.g. \"D minor\"."
                    },
                    "tempo": {
                        "type": ["object", "null"],
                        "description": "Optional tempo target with marking and/or BPM.",
                        "properties": {
                            "marking": { "type": ["string", "null"] },
                            "bpm": { "type": ["integer", "null"] }
                        }
                    },
                    "notes": { "type": ["string", "null"] },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "default": []
                    }
                },
                "required": ["title", "kind"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: UPDATE_ITEM,
            description: "PATCH-style update of an item. Only fields you supply are changed; supply explicit `null` to clear an optional field. Use `get_item` first if you need the current state.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The item's id (ULID)." },
                    "title": { "type": "string" },
                    "composer": { "type": ["string", "null"] },
                    "key": { "type": ["string", "null"] },
                    "tempo": {
                        "type": ["object", "null"],
                        "properties": {
                            "marking": { "type": ["string", "null"] },
                            "bpm": { "type": ["integer", "null"] }
                        }
                    },
                    "notes": { "type": ["string", "null"] },
                    "tags": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["id"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: DELETE_ITEM,
            description: "Delete an item by id. The user can recreate from history if this was a mistake; for irreversible operations, prefer to ask the user first.",
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
            name: CREATE_SET,
            description: "Create a routine — a named, ordered list of items the user can practice together. Each entry references an existing item by id; create the items first if they don't exist.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "entries": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "item_id": { "type": "string" },
                                "item_title": { "type": "string" },
                                "item_type": {
                                    "type": "string",
                                    "enum": ["piece", "exercise"]
                                }
                            },
                            "required": ["item_id", "item_title", "item_type"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["name", "entries"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: UPDATE_SET,
            description: "Replace the contents of a routine (name + entries). PATCH-style for individual entries isn't supported; supply the full new list.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The routine's id (ULID)." },
                    "name": { "type": "string" },
                    "entries": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "item_id": { "type": "string" },
                                "item_title": { "type": "string" },
                                "item_type": {
                                    "type": "string",
                                    "enum": ["piece", "exercise"]
                                }
                            },
                            "required": ["item_id", "item_title", "item_type"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["id", "name", "entries"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: BULK_IMPORT_ITEMS,
            description: "Create many items at once (the killer tool for \"set me up with the Bach Cello Suites\"). Call with `dry_run: true` first to get a per-item validation preview, show that to the user for confirmation, then re-call with `dry_run: false` to actually write. Validation runs across all items pre-flight — if ANY item is invalid, NONE are written. Once validation passes, items are inserted sequentially; in the rare case a DB error happens mid-insert, the response surfaces how many items were written before the error so the agent can re-issue only the remainder.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string" },
                                "kind": { "type": "string", "enum": ["piece", "exercise"] },
                                "composer": { "type": ["string", "null"] },
                                "key": { "type": ["string", "null"] },
                                "tempo": {
                                    "type": ["object", "null"],
                                    "properties": {
                                        "marking": { "type": ["string", "null"] },
                                        "bpm": { "type": ["integer", "null"] }
                                    }
                                },
                                "notes": { "type": ["string", "null"] },
                                "tags": { "type": "array", "items": { "type": "string" }, "default": [] }
                            },
                            "required": ["title", "kind"]
                        }
                    },
                    "dry_run": {
                        "type": "boolean",
                        "default": false,
                        "description": "When true, validates each item and returns a per-item preview without writing. Always start with this, then re-call with `false` after the user has confirmed."
                    }
                },
                "required": ["items"],
                "additionalProperties": false
            }),
        },
    ]
}

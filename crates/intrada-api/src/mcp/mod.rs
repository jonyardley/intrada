//! MCP (Model Context Protocol) server.
//!
//! Per the spec at `specs/mcp-server.md`, this is Phase 3: a reads-only
//! tool surface that lets MCP clients (Claude Desktop, Cursor, custom
//! agents) act on the user's behalf.
//!
//! Lives as a module inside `intrada-api` rather than a separate crate
//! because it needs `AppState`, `AuthUser`, and `services::*` — extracting
//! those into a shared crate would be a much bigger refactor than the
//! Phase 3 scope justifies. If we ever want to publish the MCP server
//! independently, the surface here is structured so that move would be
//! local to this module.

mod handlers;
pub mod protocol;
mod server;
mod tools;

pub use server::router;

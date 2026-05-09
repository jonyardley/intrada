//! Public static assets served from the API root.
//!
//! Intentionally minimal — currently just `/icon.svg`, the server icon
//! advertised in the MCP `initialize` response's `serverInfo.icons`
//! field (MCP spec 2025-11-25). Anything heavier (favicon variants,
//! marketing images) belongs on the web origin / Cloudflare, not here.
//!
//! Why this lives in the API and not the web app: MCP clients only
//! know the API's URL (`intrada-api.fly.dev`); they don't know about
//! `myintrada.com`. The icon has to be fetchable from the same origin
//! that serves `/api/mcp`, otherwise we'd need an MCP-spec extension
//! the server doesn't have today (and CORS would still trip up
//! cross-origin asset fetches in some clients).

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::state::AppState;

/// 1024×1024 SVG with the same artwork as the web app's favicon. Inlined
/// at build time so the binary is fully self-contained — moving the
/// file requires updating one path here and re-running tests.
const ICON_SVG: &str = include_str!("../../static/icon.svg");

pub fn router() -> Router<AppState> {
    Router::new().route("/icon.svg", get(icon))
}

async fn icon() -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            // Long cache — the icon is content-addressable in spirit
            // (the bytes are pinned to the deployed commit). If we ever
            // change it, a redeploy bumps the response, and `immutable`
            // means well-behaved caches won't even revalidate.
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        ICON_SVG,
    )
        .into_response()
}

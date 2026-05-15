//! OAuth 2.1 + DCR endpoints (Phase 5 of #477).
//!
//! Mounted at the **root** of the API router (`.well-known/*` MUST be at
//! the root per RFC 8414). The user-facing consent step happens on the
//! web app — the API's `/oauth/authorize` validates params and redirects
//! the user's browser to `https://<web>/oauth/consent?...`. The web app
//! shows the consent UI gated by the existing Clerk auth flow, then
//! calls `POST /oauth/finalize` with the user's JWT to mint the code.
//!
//! All endpoints here have permissive CORS (per the same rationale as
//! `/api/mcp/*` in #481): claude.ai-style cross-origin OAuth flows are
//! the whole point of this surface, and PKCE provides the security.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde_json::json;
// `Form` extractor uses `axum::extract::Form`, included via the prelude above.

use crate::auth::{AuthSource, AuthUser};
use crate::error::ApiError;
use crate::services;
use crate::services::oauth::{
    AuthorizeParams, RegisterClientRequest, RegisterClientResponse, TokenRequest, TokenResponse,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/.well-known/oauth-authorization-server", get(discovery))
        .route("/oauth/register", post(register))
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/finalize", post(finalize))
        // Token endpoint accepts BOTH application/json and the canonical
        // application/x-www-form-urlencoded per RFC 6749. We default to
        // form-encoded (axum::Form) since most OAuth clients use it; JSON
        // is accepted as a fallback inside the handler.
        .route("/oauth/token", post(token))
}

// ── Discovery (RFC 8414) ───────────────────────────────────────────────

#[tracing::instrument(name = "oauth.discovery", skip_all)]
async fn discovery(headers: HeaderMap) -> Response {
    // Build the issuer URL from the request's `Host` header so the
    // discovery doc reflects whatever domain the client used to fetch
    // it. Behind a TLS-terminating proxy (Fly.io / Cloudflare) we can't
    // detect the scheme reliably, so we default to https — http is only
    // a thing in local dev where the client should know to look there.
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("intrada-api.fly.dev");
    let scheme = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    let issuer = format!("{scheme}://{host}");

    Json(json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/oauth/authorize"),
        "token_endpoint": format!("{issuer}/oauth/token"),
        "registration_endpoint": format!("{issuer}/oauth/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256"],
        // Public clients only — PKCE is the auth mechanism.
        "token_endpoint_auth_methods_supported": ["none"],
        "scopes_supported": ["mcp"],
    }))
    .into_response()
}

// ── DCR (RFC 7591) ─────────────────────────────────────────────────────

#[tracing::instrument(name = "oauth.register", skip_all, fields(client_name = %req.client_name))]
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterClientRequest>,
) -> Result<(StatusCode, Json<RegisterClientResponse>), ApiError> {
    let conn = state.conn();
    let response = services::oauth::register_client(&conn, req).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// ── Authorize (RFC 6749 §4.1.1) ────────────────────────────────────────

#[tracing::instrument(
    name = "oauth.authorize",
    skip_all,
    fields(client_id = %params.client_id)
)]
async fn authorize(
    State(state): State<AppState>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Response, ApiError> {
    // Validate up-front so an invalid client_id / redirect_uri / response_type
    // surfaces an error here rather than after the user has signed in. We
    // don't redirect errors back to the (possibly attacker-controlled)
    // redirect_uri until we've validated it's the registered one — RFC 6749 §4.1.2.1.
    state
        .with_transient_retry(|conn| {
            let params = params.clone();
            async move { services::oauth::validate_authorize_request(&conn, &params).await }
        })
        .await?;

    // Redirect to the web app's consent page. The web app gates on
    // Clerk auth (existing flow), shows consent UI, and on Allow calls
    // back to /oauth/finalize.
    //
    // The consent URL is derived from `WEB_BASE_URL` — defaults to
    // production. For local dev the operator overrides this.
    let web_base = state.web_base_url();
    let consent_url = format!(
        "{web_base}/oauth/consent?response_type={response_type}&client_id={client_id}&redirect_uri={redirect_uri}&state={state_param}&scope={scope}&code_challenge={code_challenge}&code_challenge_method={code_challenge_method}",
        response_type = urlencode(&params.response_type),
        client_id = urlencode(&params.client_id),
        redirect_uri = urlencode(&params.redirect_uri),
        state_param = urlencode(params.state.as_deref().unwrap_or("")),
        scope = urlencode(params.scope.as_deref().unwrap_or("")),
        code_challenge = urlencode(&params.code_challenge),
        code_challenge_method = urlencode(&params.code_challenge_method),
    );

    Ok(Redirect::to(&consent_url).into_response())
}

#[derive(Debug, serde::Deserialize)]
struct FinalizeRequest {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    code_challenge: String,
    code_challenge_method: String,
}

#[derive(Debug, serde::Serialize)]
struct FinalizeResponse {
    /// Full URL the web app should redirect the user's browser to —
    /// `redirect_uri?code=<code>&state=<state>`. Web app does
    /// `window.location = redirect_url` to complete the OAuth dance.
    redirect_url: String,
}

/// Called by the web app's consent page after the user clicks Allow.
/// Refuses anything other than `AuthSource::Jwt` (a real Clerk session).
///
/// PAT auth is rejected even though it's self-scoped — minting an OAuth
/// code requires interactive user consent, and a PAT is by definition
/// already a non-interactive credential. Allowing PAT-on-/finalize
/// would let a stolen PAT chain into an OAuth grant attributed to
/// "the user" without the user actually clicking Allow. Disabled mode
/// is rejected because there's no user to attribute the code to.
#[tracing::instrument(
    name = "oauth.finalize",
    skip_all,
    fields(client_id = %req.client_id)
)]
async fn finalize(
    State(state): State<AppState>,
    AuthUser { user_id, source }: AuthUser,
    Json(req): Json<FinalizeRequest>,
) -> Result<Json<FinalizeResponse>, ApiError> {
    match source {
        AuthSource::Jwt => {}
        AuthSource::Pat { .. } => {
            return Err(ApiError::Unauthorized(
                "/oauth/finalize requires a browser session (Clerk JWT), not a PAT".into(),
            ));
        }
        AuthSource::Disabled => {
            return Err(ApiError::Unauthorized(
                "/oauth/finalize requires a Clerk-authenticated user".into(),
            ));
        }
    }
    if user_id.is_empty() {
        // Defence in depth: AuthSource::Jwt with empty sub shouldn't happen
        // (Clerk always populates sub), but refuse anyway rather than
        // silently mint an empty-user token.
        return Err(ApiError::Unauthorized(
            "/oauth/finalize requires a non-empty user id".into(),
        ));
    }

    // Re-validate authorize params now that we know the user has consented
    // — guards against the consent page being rendered with stale or
    // tampered params (e.g. a swapped client_id).
    let conn = state.conn();
    let authorize_params = AuthorizeParams {
        response_type: req.response_type,
        client_id: req.client_id.clone(),
        redirect_uri: req.redirect_uri.clone(),
        state: req.state.clone(),
        scope: req.scope.clone(),
        code_challenge: req.code_challenge.clone(),
        code_challenge_method: req.code_challenge_method.clone(),
    };
    services::oauth::validate_authorize_request(&conn, &authorize_params).await?;

    let code = services::oauth::mint_auth_code(
        &conn,
        &user_id,
        &req.client_id,
        &req.redirect_uri,
        &req.code_challenge,
        &req.code_challenge_method,
        req.scope.clone(),
    )
    .await?;

    // Some OAuth clients register redirect_uris that already contain a
    // query string (e.g. `https://app.example/cb?token=xxx`). Use `&`
    // as the separator if so, `?` otherwise.
    let separator = if req.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    let mut redirect_url = format!(
        "{redirect_uri}{separator}code={code}",
        redirect_uri = req.redirect_uri,
        code = urlencode(&code),
    );
    if let Some(s) = &req.state {
        redirect_url.push_str(&format!("&state={}", urlencode(s)));
    }

    Ok(Json(FinalizeResponse { redirect_url }))
}

// ── Token endpoint (RFC 6749 §4.1.3) ───────────────────────────────────

/// RFC 6749 §3.2 — `application/x-www-form-urlencoded` is the canonical
/// content-type for the token endpoint. Most OAuth clients use it; JSON
/// is a non-standard extension that we don't accept here.
#[tracing::instrument(
    name = "oauth.token",
    skip_all,
    fields(client_id = %req.client_id, grant_type = %req.grant_type)
)]
async fn token(
    State(state): State<AppState>,
    Form(req): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let conn = state.conn();
    let response = services::oauth::exchange_code_for_token(&conn, req).await?;
    Ok(Json(response))
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Minimal URL-encoding for OAuth params we round-trip through
/// query strings. Avoids a `urlencoding` dep — we only encode a small
/// known surface (alphanumerics, hyphen, underscore, dot, tilde stay
/// raw per RFC 3986).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

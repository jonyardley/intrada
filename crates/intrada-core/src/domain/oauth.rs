//! OAuth consent flow (shell-side).
//!
//! Mirrors the API's `/oauth/finalize` shape. The web app's
//! `/oauth/consent` view collects the OAuth params from its URL query,
//! shows a consent UI gated by Clerk auth, and on Allow dispatches
//! `OAuthEvent::FinalizeConsent`. The handler POSTs to /oauth/finalize
//! (Clerk JWT is attached automatically by `core_bridge`), and on
//! success surfaces the redirect URL through the model so the view can
//! navigate the browser to the OAuth client's redirect_uri.

use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::model::Model;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct OAuthFinalizeParams {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum OAuthEvent {
    /// User clicked Allow on the consent screen.
    FinalizeConsent(OAuthFinalizeParams),
    /// API returned the redirect URL — the view picks this up via
    /// `view_model.oauth_redirect_url` and navigates the browser.
    ConsentFinalized {
        redirect_url: String,
    },
    ConsentFailed(String),
    /// View resets state when leaving the consent page so a stale
    /// redirect_url doesn't trigger a navigate next time.
    ResetConsent,
}

pub fn handle_oauth_event(event: OAuthEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        OAuthEvent::FinalizeConsent(params) => {
            model.oauth_in_flight = true;
            model.oauth_redirect_url = None;
            Command::all([
                crate::http::oauth_finalize(&model.api_base_url, &params),
                crux_core::render::render(),
            ])
        }
        OAuthEvent::ConsentFinalized { redirect_url } => {
            model.oauth_in_flight = false;
            model.oauth_redirect_url = Some(redirect_url);
            model.record_success();
            crux_core::render::render()
        }
        OAuthEvent::ConsentFailed(message) => {
            model.oauth_in_flight = false;
            model.surface_error(message);
            crux_core::render::render()
        }
        OAuthEvent::ResetConsent => {
            model.oauth_in_flight = false;
            model.oauth_redirect_url = None;
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finalize_sets_in_flight() {
        let mut model = Model::test_default();
        let _ = handle_oauth_event(
            OAuthEvent::FinalizeConsent(OAuthFinalizeParams {
                response_type: "code".into(),
                client_id: "x".into(),
                redirect_uri: "https://example.com/cb".into(),
                state: None,
                scope: None,
                code_challenge: "x".into(),
                code_challenge_method: "S256".into(),
            }),
            &mut model,
        );
        assert!(model.oauth_in_flight);
        assert!(model.oauth_redirect_url.is_none());
    }

    #[test]
    fn consent_finalized_surfaces_redirect_url() {
        let mut model = Model::test_default();
        model.oauth_in_flight = true;
        let _ = handle_oauth_event(
            OAuthEvent::ConsentFinalized {
                redirect_url: "https://example.com/cb?code=xxx&state=yyy".into(),
            },
            &mut model,
        );
        assert!(!model.oauth_in_flight);
        assert_eq!(
            model.oauth_redirect_url.as_deref(),
            Some("https://example.com/cb?code=xxx&state=yyy")
        );
    }

    #[test]
    fn reset_clears_redirect_url() {
        let mut model = Model::test_default();
        model.oauth_redirect_url = Some("stale".into());
        let _ = handle_oauth_event(OAuthEvent::ResetConsent, &mut model);
        assert!(model.oauth_redirect_url.is_none());
    }
}

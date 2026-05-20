use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::features::FeatureFlags;

use crate::auth::{AuthSource, AuthUser};
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_features))
}

const GOALS_ALLOWLIST_ENV: &str = "INTRADA_FEATURE_FLAG_GOALS_ALLOWLIST";

fn user_in_allowlist(env_var: &str, user_id: &str) -> bool {
    let Ok(raw) = std::env::var(env_var) else {
        return false;
    };
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|allowed| allowed == user_id)
}

async fn get_features(auth: AuthUser) -> Result<Json<FeatureFlags>, ApiError> {
    // Dev mode (no Clerk configured) opens every flag — no allowlist gymnastics
    // for solo local development.
    let goals = matches!(auth.source, AuthSource::Disabled)
        || user_in_allowlist(GOALS_ALLOWLIST_ENV, &auth.user_id);
    Ok(Json(FeatureFlags { goals }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_matches_exact_id() {
        std::env::set_var("INTRADA_TEST_FF_ALLOW_1", "user_abc, user_def , user_ghi");
        assert!(user_in_allowlist("INTRADA_TEST_FF_ALLOW_1", "user_def"));
        assert!(user_in_allowlist("INTRADA_TEST_FF_ALLOW_1", "user_abc"));
        assert!(!user_in_allowlist("INTRADA_TEST_FF_ALLOW_1", "user_xyz"));
        std::env::remove_var("INTRADA_TEST_FF_ALLOW_1");
    }

    #[test]
    fn allowlist_tolerates_empty_segments() {
        std::env::set_var("INTRADA_TEST_FF_ALLOW_3", ",,user_abc,,user_def,");
        assert!(user_in_allowlist("INTRADA_TEST_FF_ALLOW_3", "user_abc"));
        assert!(user_in_allowlist("INTRADA_TEST_FF_ALLOW_3", "user_def"));
        assert!(!user_in_allowlist("INTRADA_TEST_FF_ALLOW_3", ""));
        std::env::remove_var("INTRADA_TEST_FF_ALLOW_3");
    }

    #[test]
    fn allowlist_empty_or_unset_rejects() {
        std::env::set_var("INTRADA_TEST_FF_ALLOW_2", "");
        assert!(!user_in_allowlist("INTRADA_TEST_FF_ALLOW_2", "anyone"));
        std::env::remove_var("INTRADA_TEST_FF_ALLOW_2");
        assert!(!user_in_allowlist("INTRADA_TEST_FF_ALLOW_2", "anyone"));
    }
}

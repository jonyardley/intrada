use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::features::FeatureFlags;

use crate::auth::{AuthSource, AuthUser};
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_features))
}

fn flag_env(name: &str) -> String {
    format!("INTRADA_FEATURE_FLAG_{}_ALLOWLIST", name.to_uppercase())
}

fn user_in_allowlist(env_var: &str, user_id: &str) -> bool {
    let Ok(raw) = std::env::var(env_var) else {
        return false;
    };
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|allowed| allowed == user_id)
}

fn resolve(name: &str, auth: &AuthUser) -> bool {
    // Dev mode (no Clerk configured) opens every flag — no allowlist
    // gymnastics for solo local development.
    matches!(auth.source, AuthSource::Disabled) || user_in_allowlist(&flag_env(name), &auth.user_id)
}

async fn get_features(auth: AuthUser) -> Result<Json<FeatureFlags>, ApiError> {
    Ok(Json(FeatureFlags {
        goals: resolve("goals", &auth),
    }))
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
    fn flag_env_uppercases_and_wraps() {
        assert_eq!(flag_env("goals"), "INTRADA_FEATURE_FLAG_GOALS_ALLOWLIST");
        assert_eq!(
            flag_env("my_new_flag"),
            "INTRADA_FEATURE_FLAG_MY_NEW_FLAG_ALLOWLIST"
        );
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

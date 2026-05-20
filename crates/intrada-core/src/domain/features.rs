use serde::{Deserialize, Serialize};

/// Per-user feature flag state, resolved server-side. New flags need
/// `#[serde(default)]` so older clients don't fail to deserialise.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FeatureFlags {
    #[serde(default)]
    pub goals: bool,
}

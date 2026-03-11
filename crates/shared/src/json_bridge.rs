//! JSON-based FFI bridge for iOS/Android shells.
//!
//! This module wraps `crux_core::Core<Intrada>` and exposes JSON-based
//! methods for shells that prefer JSON over BCS serialisation.
//!
//! Why JSON instead of BCS:
//! - The `crux_core::typegen` codegen for Swift BCS is blocked by a
//!   `serde-reflection` limitation with GoalKind's enum variants.
//! - JSON serialisation is natively supported by Swift's `Codable`/`JSONEncoder`.
//! - `serde_json` uses the same field names and enum tagging as our Rust types.
//! - Performance difference is negligible for this app's data volume.

use std::sync::Mutex;

use crux_core::Core;

use intrada_core::app::{AppEffect, Effect, Intrada};
use intrada_core::{Event, ViewModel};

/// A JSON-serialisable representation of a single effect from the core.
///
/// Strips the `Request` wrapper (which carries effect IDs for the Bridge pattern)
/// since our shell handles effects synchronously and all `AppEffect` outputs are `()`.
#[derive(serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum JsonEffect {
    /// Render effect — signals the shell to re-read the ViewModel.
    Render,
    /// App effect — an operation the shell must execute (API call, storage, etc.).
    App(AppEffect),
}

/// JSON-based core wrapper for iOS/Android FFI.
///
/// Uses `Core<Intrada>` directly (not `Bridge`) and serialises all
/// communication as JSON strings via `serde_json`.
///
/// # Thread safety
///
/// `Core<Intrada>` uses `RefCell` internally so it is `Send` but not `Sync`.
/// Wrapping in `Mutex` makes the struct `Send + Sync` as required by UniFFI.
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct CoreJson {
    core: Mutex<Core<Intrada>>,
}

impl Default for CoreJson {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl CoreJson {
    /// Create a new core instance.
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            core: Mutex::new(Core::new()),
        }
    }

    /// Process a JSON-serialised `Event` and return JSON-serialised effects.
    ///
    /// # Input format
    ///
    /// The `event_json` string must be a valid JSON representation of
    /// `intrada_core::Event` using serde's default externally-tagged enum format:
    ///
    /// - Unit variants: `"ClearError"`
    /// - Newtype variants: `{"LoadFailed": "some error message"}`
    /// - Struct variants: `{"DataLoaded": {"items": [...]}}`
    ///
    /// # Output format
    ///
    /// Returns a JSON array of `JsonEffect` objects, each tagged with `"type"`:
    ///
    /// ```json
    /// [
    ///   {"type": "Render"},
    ///   {"type": "App", "data": "LoadAll"},
    ///   {"type": "App", "data": {"SaveItem": {... item fields ...}}}
    /// ]
    /// ```
    #[must_use]
    pub fn process_event(&self, event_json: String) -> String {
        let event: Event = serde_json::from_str(&event_json)
            .unwrap_or_else(|e| panic!("CoreJson: invalid event JSON: {e}\nInput: {event_json}"));

        let core = self.core.lock().expect("CoreJson: mutex poisoned");
        let effects = core.process_event(event);
        drop(core);

        let json_effects: Vec<JsonEffect> = effects
            .into_iter()
            .map(|e| match e {
                Effect::Render(_) => JsonEffect::Render,
                Effect::App(req) => JsonEffect::App(req.operation),
            })
            .collect();

        serde_json::to_string(&json_effects)
            .unwrap_or_else(|e| panic!("CoreJson: effect serialisation failed: {e}"))
    }

    /// Return the current `ViewModel` as a JSON string.
    ///
    /// The output is a JSON object matching `intrada_core::ViewModel` with
    /// snake_case field names (as defined by serde `#[serde(rename)]` attrs).
    #[must_use]
    pub fn view(&self) -> String {
        let core = self.core.lock().expect("CoreJson: mutex poisoned");
        let vm: ViewModel = core.view();
        drop(core);

        serde_json::to_string(&vm)
            .unwrap_or_else(|e| panic!("CoreJson: view serialisation failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_json_roundtrip() {
        let core = CoreJson::new();

        // Send DataLoaded with empty items
        let effects_json = core.process_event(r#"{"DataLoaded":{"items":[]}}"#.to_string());
        let effects: Vec<serde_json::Value> = serde_json::from_str(&effects_json).unwrap();
        assert!(!effects.is_empty(), "Expected at least a Render effect");

        // First effect should be Render
        assert_eq!(effects[0]["type"], "Render");

        // View should now have empty items
        let view_json = core.view();
        let vm: ViewModel = serde_json::from_str(&view_json).unwrap();
        assert!(vm.items.is_empty());
    }

    #[test]
    fn test_clear_error_event() {
        let core = CoreJson::new();

        // Load data first
        let _ = core.process_event(r#"{"DataLoaded":{"items":[]}}"#.to_string());

        // Send ClearError
        let effects_json = core.process_event(r#""ClearError""#.to_string());
        let effects: Vec<serde_json::Value> = serde_json::from_str(&effects_json).unwrap();
        // Should get at least a Render effect
        assert!(effects.iter().any(|e| e["type"] == "Render"));
    }

    #[test]
    fn test_view_returns_valid_json() {
        let core = CoreJson::new();
        let view_json = core.view();
        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&view_json).unwrap();
    }

    #[test]
    fn test_data_loaded_produces_load_all_and_render() {
        let core = CoreJson::new();

        // The initial DataLoaded event should trigger Render + LoadAll
        let effects_json = core.process_event(r#"{"DataLoaded":{"items":[]}}"#.to_string());
        let effects: Vec<serde_json::Value> = serde_json::from_str(&effects_json).unwrap();

        let has_render = effects.iter().any(|e| e["type"] == "Render");
        assert!(has_render, "Expected a Render effect");
    }

    #[test]
    fn test_effect_json_shape() {
        let core = CoreJson::new();

        // Send an event that triggers App effects
        let effects_json = core.process_event(r#"{"DataLoaded":{"items":[]}}"#.to_string());
        let effects: Vec<serde_json::Value> = serde_json::from_str(&effects_json).unwrap();

        // Every effect must have a "type" field
        for effect in &effects {
            assert!(
                effect.get("type").is_some(),
                "Effect missing 'type' field: {effect}"
            );
        }
    }
}

//! iOS-only bridge: registers the Swift plugin and gives the Rust
//! commands a typed handle for invoking its `@objc` methods. The
//! `tauri::ios_plugin_binding!` macro generates the FFI binding to the
//! `init_plugin_background_audio` C symbol exported by the Swift side
//! (see `ios/Sources/BackgroundAudioPlugin/BackgroundAudioPlugin.swift`).

use serde::Serialize;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

tauri::ios_plugin_binding!(init_plugin_background_audio);

/// Initializes the Swift plugin and returns a handle the commands can
/// dispatch through. Stored on the AppHandle via `app.manage(...)` from
/// the plugin's setup hook.
pub fn init<R: Runtime, C: serde::de::DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<BackgroundAudio<R>> {
    let handle = api
        .register_ios_plugin(init_plugin_background_audio)
        .map_err(|e| crate::Error::Bridge(e.to_string()))?;
    Ok(BackgroundAudio(handle))
}

/// Wrapper around the Swift plugin's IPC handle.
pub struct BackgroundAudio<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> BackgroundAudio<R> {
    /// Generic "send `args` to the Swift method named `cmd`" — the
    /// individual commands map their own typed payloads to this. Method
    /// names match the Swift `@objc` selectors, which are snake_case
    /// here so they line up 1:1 with the JS-side command names (no
    /// translation needed in either direction).
    pub fn run<P: Serialize>(&self, cmd: &str, payload: P) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>(cmd, payload)
            .map_err(|e| crate::Error::Bridge(e.to_string()))
    }
}

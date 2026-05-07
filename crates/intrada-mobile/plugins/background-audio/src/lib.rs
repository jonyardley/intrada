//! Intrada internal Tauri plugin — keeps the practice-session timer
//! running while the device is locked / the app is backgrounded.
//!
//! Phase B scaffold: commands accept the same shape that Phase C will
//! consume but currently return `Ok(())` without any native side-effects.
//! No `AVAudioSession` activation, no `MPNowPlayingInfoCenter` updates
//! yet — see `specs/background-audio-plugin.md` for the full plan.
//!
//! The plugin is iOS-only at runtime: on web the JS-side bindings are
//! a no-op (the Tauri `invoke` global is absent), on macOS / Android
//! the Rust commands resolve but do nothing. That's intentional — the
//! call sites in the shell don't need to gate on `data-platform`.

use serde::{Deserialize, Serialize};
use tauri::{plugin::TauriPlugin, Runtime};

/// Errors surfaceable from plugin commands. Phase B has no error paths
/// (every command returns `Ok`), but the type is here so Phase C can
/// surface AVAudioSession activation / interruption failures without
/// the JS bindings caring about the variant set growing.
#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(self.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Argument shape for `begin_session`. The shell passes the current item
/// title and the RFC3339 wall-clock anchor so the plugin can
/// (eventually) seed `MPNowPlayingInfoCenter` with the right metadata.
#[derive(Debug, Deserialize)]
pub struct BeginSessionArgs {
    pub title: String,
    pub started_at: String,
}

/// Argument shape for `set_now_playing` — fired on each item advance.
#[derive(Debug, Deserialize)]
pub struct NowPlayingArgs {
    pub title: String,
    pub position_label: String,
    pub started_at: String,
}

#[tauri::command]
async fn begin_session(_args: BeginSessionArgs) -> Result<()> {
    // Phase C will: AVAudioSession.setCategory(.playback, .mixWithOthers)
    // + setActive(true) + start silent loop + seed MPNowPlayingInfoCenter.
    // Phase B: no-op so the IPC roundtrip can be exercised end-to-end.
    Ok(())
}

#[tauri::command]
async fn set_now_playing(_args: NowPlayingArgs) -> Result<()> {
    // Phase C will: update MPNowPlayingInfoCenter title / subtitle /
    // elapsed and re-arm playbackState for the item-change visual cue.
    Ok(())
}

#[tauri::command]
async fn end_session() -> Result<()> {
    // Phase C will: stop silent loop + AVAudioSession.setActive(false,
    // notifyOthersOnDeactivation) + clear MPNowPlayingInfoCenter.
    Ok(())
}

/// Plugin entry point — wired into the Tauri Builder in
/// `intrada-mobile/src-tauri/src/lib.rs`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("background-audio")
        .invoke_handler(tauri::generate_handler![
            begin_session,
            set_now_playing,
            end_session
        ])
        .setup(|_app, _api| {
            // Phase C: register Swift plugin via `api.register_ios_plugin(...)`
            // so the Tauri runtime can route the commands across the FFI.
            Ok(())
        })
        .build()
}

//! Intrada internal Tauri plugin — keeps the practice-session timer
//! running while the device is locked / the app is backgrounded.
//!
//! Phase C (#309): Rust commands forward to the Swift implementation
//! via `PluginHandle::run_mobile_plugin`. The Swift side activates the
//! audio session, plays a silent loop to keep iOS from suspending the
//! app, and updates `MPNowPlayingInfoCenter` for the lock screen.
//!
//! On non-iOS platforms the commands resolve to `Ok(())` without doing
//! anything — the JS bindings call them unconditionally and we don't
//! want them to error on web / macOS / Android.
//!
//! Spec: `specs/background-audio-plugin.md`.

use serde::{Deserialize, Serialize};
#[cfg(target_os = "ios")]
use tauri::Manager;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};

#[cfg(target_os = "ios")]
mod mobile;

/// Errors surfaceable from plugin commands. Phase C surfaces native-side
/// failures (audio session activation, missing silent loop bundle,
/// AVAudioPlayer rejecting `play()`) so the shell can decide whether to
/// keep the wall-clock timer running web-only.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Tauri / Swift bridge surfaced an error string.
    #[error("background-audio: {0}")]
    Bridge(String),
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(self.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Argument shape for `begin_session`. Field names use snake_case to
/// match what JS sends; Swift's `Decodable` `parseArgs` accepts the
/// same shape.
#[derive(Debug, Serialize, Deserialize)]
pub struct BeginSessionArgs {
    pub title: String,
    pub started_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NowPlayingArgs {
    pub title: String,
    pub position_label: String,
    pub started_at: String,
}

#[tauri::command]
async fn begin_session<R: Runtime>(app: AppHandle<R>, args: BeginSessionArgs) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        let state = app.state::<mobile::BackgroundAudio<R>>();
        return state.run("begin_session", args);
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (app, args);
        Ok(())
    }
}

#[tauri::command]
async fn set_now_playing<R: Runtime>(app: AppHandle<R>, args: NowPlayingArgs) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        let state = app.state::<mobile::BackgroundAudio<R>>();
        return state.run("set_now_playing", args);
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (app, args);
        Ok(())
    }
}

#[tauri::command]
async fn end_session<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        let state = app.state::<mobile::BackgroundAudio<R>>();
        return state.run("end_session", ());
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = app;
        Ok(())
    }
}

/// Plugin entry point — wired into the Tauri Builder in
/// `intrada-mobile/src-tauri/src/lib.rs`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("background-audio")
        .invoke_handler(tauri::generate_handler![
            begin_session,
            set_now_playing,
            end_session
        ])
        .setup(|app, _api| {
            #[cfg(target_os = "ios")]
            {
                let handle = mobile::init(app, _api)?;
                app.manage(handle);
            }
            // Suppress unused-variable warning on non-iOS targets where
            // the setup hook is a no-op.
            #[cfg(not(target_os = "ios"))]
            {
                let _ = app;
            }
            Ok(())
        })
        .build()
}

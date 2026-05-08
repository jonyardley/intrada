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

/// Drop a Sentry breadcrumb tagged `background-audio` so any subsequent
/// failure (here or elsewhere in the session) carries the lifecycle
/// context. No-op when Sentry isn't initialised (simulator / dev /
/// non-iOS), so cheap to call unconditionally.
#[cfg(target_os = "ios")]
fn breadcrumb(command: &str, data: serde_json::Value) {
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some("background-audio".to_string()),
        message: Some(command.to_string()),
        level: sentry::Level::Info,
        data: data
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect(),
        ..Default::default()
    });
}

/// Capture a plugin-bridge failure as a Sentry message tagged with the
/// command name. Errors here mean the iOS audio session is likely not
/// active (timer will keep running web-side via the wall-clock derive,
/// but the lock-screen + background timer behaviour is broken) — we want
/// telemetry on the rate so we can prioritise fixes against real-device
/// data. No-op when Sentry isn't initialised.
#[cfg(target_os = "ios")]
fn capture_bridge_error(command: &str, err: &Error) {
    sentry::with_scope(
        |scope| {
            scope.set_tag("plugin", "background-audio");
            scope.set_tag("command", command);
        },
        || {
            sentry::capture_message(&format!("{err}"), sentry::Level::Error);
        },
    );
}

#[tauri::command]
async fn begin_session<R: Runtime>(app: AppHandle<R>, args: BeginSessionArgs) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        breadcrumb(
            "begin_session",
            serde_json::json!({
                "title": &args.title,
                "started_at": &args.started_at,
            }),
        );
        let state = app.state::<mobile::BackgroundAudio<R>>();
        let result = state.run("begin_session", args);
        if let Err(err) = &result {
            capture_bridge_error("begin_session", err);
        }
        result
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
        breadcrumb(
            "set_now_playing",
            serde_json::json!({
                "title": &args.title,
                "position_label": &args.position_label,
                "started_at": &args.started_at,
            }),
        );
        let state = app.state::<mobile::BackgroundAudio<R>>();
        let result = state.run("set_now_playing", args);
        if let Err(err) = &result {
            capture_bridge_error("set_now_playing", err);
        }
        result
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
        breadcrumb("end_session", serde_json::json!({}));
        let state = app.state::<mobile::BackgroundAudio<R>>();
        let result = state.run("end_session", ());
        if let Err(err) = &result {
            capture_bridge_error("end_session", err);
        }
        result
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

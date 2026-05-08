//! Intrada internal Tauri plugin — drives an ActivityKit Live Activity
//! for the duration of a practice session. Layers on top of the
//! background-audio plugin: same lifecycle (None → Some → None as the
//! WebView's `vm.active_session` changes), different surface.
//!
//! Phase B (#474): Rust commands accept the same shape Phase C will
//! consume but currently return `Ok(())` — the Swift side is a
//! placeholder that doesn't call ActivityKit yet. Spec:
//! `specs/live-activity-plugin.md`.
//!
//! On non-iOS platforms the commands resolve to `Ok(())` without doing
//! anything — the JS bindings call them unconditionally and we don't
//! want them to error on web / macOS / Android / iPad (where Live
//! Activities aren't supported).

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
/// failures (ActivityKit `Activity.request` rejection, user disabled
/// Live Activities in Settings, missing `NSSupportsLiveActivities`
/// plist key) so the shell can decide whether to keep going with just
/// the background-audio plugin.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Tauri / Swift bridge surfaced an error string.
    #[error("live-activity: {0}")]
    Bridge(String),
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(self.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Argument shape for `begin`. Field names use snake_case to match what
/// JS sends; Swift's `Decodable` accepts the same shape.
///
/// `planned_duration_secs` is optional because not every entry has a
/// planned duration set — the activity falls back to elapsed-only
/// rendering in that case.
#[derive(Debug, Serialize, Deserialize)]
pub struct BeginArgs {
    pub item_title: String,
    pub position_label: String,
    pub started_at: String,
    pub planned_duration_secs: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateArgs {
    pub item_title: String,
    pub position_label: String,
    pub started_at: String,
    pub planned_duration_secs: Option<u32>,
}

/// Drop a Sentry breadcrumb tagged `live-activity` so any subsequent
/// failure (here or elsewhere in the session) carries the lifecycle
/// context. No-op when Sentry isn't initialised (simulator / dev /
/// non-iOS), so cheap to call unconditionally.
#[cfg(target_os = "ios")]
fn breadcrumb(command: &str, data: serde_json::Value) {
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some("live-activity".to_string()),
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
/// command name. ActivityKit failures mean the Live Activity didn't
/// start / update — timer + lock-screen Now Playing (background-audio)
/// still work, so this isn't fatal, but we want telemetry on the rate.
#[cfg(target_os = "ios")]
fn capture_bridge_error(command: &str, err: &Error) {
    sentry::with_scope(
        |scope| {
            scope.set_tag("plugin", "live-activity");
            scope.set_tag("command", command);
        },
        || {
            sentry::capture_message(&format!("{err}"), sentry::Level::Error);
        },
    );
}

#[tauri::command]
async fn begin<R: Runtime>(app: AppHandle<R>, args: BeginArgs) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        // Strip `item_title` from breadcrumb data — piece/exercise titles
        // are user content that bypass `send_default_pii: false` if
        // attached explicitly. `position_label` ("Item N of M") is
        // generic; `planned_duration_secs` is metadata. Both are
        // sufficient for Sentry to triage failures by item position
        // without exposing what the user is practising.
        breadcrumb(
            "begin",
            serde_json::json!({
                "position_label": &args.position_label,
                "started_at": &args.started_at,
                "planned_duration_secs": args.planned_duration_secs,
            }),
        );
        let state = app.state::<mobile::LiveActivity<R>>();
        let result = state.run("begin", args);
        if let Err(err) = &result {
            capture_bridge_error("begin", err);
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
async fn update<R: Runtime>(app: AppHandle<R>, args: UpdateArgs) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        // See `begin` — `item_title` deliberately omitted from
        // breadcrumb data to avoid leaking user content into Sentry.
        breadcrumb(
            "update",
            serde_json::json!({
                "position_label": &args.position_label,
                "started_at": &args.started_at,
                "planned_duration_secs": args.planned_duration_secs,
            }),
        );
        let state = app.state::<mobile::LiveActivity<R>>();
        let result = state.run("update", args);
        if let Err(err) = &result {
            capture_bridge_error("update", err);
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
async fn end<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    #[cfg(target_os = "ios")]
    {
        breadcrumb("end", serde_json::json!({}));
        let state = app.state::<mobile::LiveActivity<R>>();
        let result = state.run("end", ());
        if let Err(err) = &result {
            capture_bridge_error("end", err);
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
    Builder::new("live-activity")
        .invoke_handler(tauri::generate_handler![begin, update, end])
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

use tauri::Manager;

// SENTRY_DSN_MOBILE is baked at compile time. Pair with the rerun-if-env-changed
// directive in build.rs so changing the env doesn't yield stale binaries.
const SENTRY_DSN: Option<&str> = option_env!("SENTRY_DSN_MOBILE");

#[tauri::command]
async fn close_splashscreen(app: tauri::AppHandle) {
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Note: data-platform="ios" is set by index.html based on
    // location.protocol === 'tauri:'. Setup-time eval here was racing page
    // load on physical devices and silently failed.

    let _sentry_guard = SENTRY_DSN.filter(|s| !s.is_empty()).map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: option_env!("GIT_SHA").map(Into::into),
                environment: Some(
                    if cfg!(debug_assertions) {
                        "development"
                    } else {
                        "production"
                    }
                    .into(),
                ),
                // Mobile host has no auto-instrumented transactions today,
                // so this is mostly moot — kept aligned with api/web for when
                // we add manual spans. See main.rs for the rationale.
                traces_sample_rate: 1.0,
                send_default_pii: false,
                ..Default::default()
            },
        ))
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_haptics::init())
        .plugin(tauri_plugin_deep_link::init())
        // Background audio: AVAudioSession + Now Playing for the active
        // practice session. Keeps the timer alive while the device is
        // locked. (#309 — fully shipped.)
        .plugin(tauri_plugin_background_audio::init())
        // Live Activity: Lock Screen + Dynamic Island for the active
        // practice session. Phase B scaffold of #474 — Swift side
        // resolves with no ActivityKit calls yet; Phase C wires the
        // real Activity<...>.request / update / end.
        .plugin(tauri_plugin_live_activity::init())
        .plugin(tauri_plugin_auth_session::init())
        .invoke_handler(tauri::generate_handler![close_splashscreen])
        .run(tauri::generate_context!())
        .expect("error while running intrada");
}

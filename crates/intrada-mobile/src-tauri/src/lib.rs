// SENTRY_DSN_MOBILE is baked at compile time. Pair with the rerun-if-env-changed
// directive in build.rs so changing the env doesn't yield stale binaries.
const SENTRY_DSN: Option<&str> = option_env!("SENTRY_DSN_MOBILE");

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
        .run(tauri::generate_context!())
        .expect("error while running intrada");
}

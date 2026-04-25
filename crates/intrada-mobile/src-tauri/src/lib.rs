#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_haptics::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|_app| {
            // Set data-platform="ios" so CSS [data-platform="ios"] selectors apply.
            // initializationScript is not available in Tauri 2 JSON config.
            #[cfg(target_os = "ios")]
            {
                use tauri::Manager;
                _app.get_webview_window("main")
                    .expect("main window not found")
                    .eval("document.documentElement.setAttribute('data-platform','ios')")?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running intrada");
}

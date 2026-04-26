#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Note: data-platform="ios" is set by index.html based on
    // location.protocol === 'tauri:'. Setup-time eval here was racing page
    // load on physical devices and silently failed.
    tauri::Builder::default()
        .plugin(tauri_plugin_haptics::init())
        .plugin(tauri_plugin_deep_link::init())
        .run(tauri::generate_context!())
        .expect("error while running intrada");
}

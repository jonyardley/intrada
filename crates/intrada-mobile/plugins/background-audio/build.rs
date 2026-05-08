// Tauri plugin build script: generates the permission JSON schema and
// the allowlist of commands the plugin exposes, and points Tauri at the
// iOS Swift package so the host Xcode project can pick it up at
// `cargo tauri ios init` / build time.
const COMMANDS: &[&str] = &["begin_session", "set_now_playing", "end_session"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}

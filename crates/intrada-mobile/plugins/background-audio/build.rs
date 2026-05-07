// Tauri plugin build script: generates the permission JSON schema + the
// allowlist of commands that the plugin exposes. Phase B commands are
// no-op stubs — Phase C will keep the same names and add the Swift impl.
const COMMANDS: &[&str] = &["begin_session", "set_now_playing", "end_session"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}

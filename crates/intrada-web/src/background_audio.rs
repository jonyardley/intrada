use wasm_bindgen::prelude::*;

// Calls Tauri plugin-background-audio commands via the Tauri IPC invoke
// bridge. All functions are no-ops outside a Tauri context (web browser,
// E2E tests). The try/catch in JS ensures plugin failures never surface
// as Rust errors — the timer continues working even if the audio session
// can't be activated. The Rust plugin captures failures into Sentry so
// we still get telemetry without surfacing UI banners users can't act on.
//
// Lifecycle orchestration lives in `session_lifecycle.rs` — a single
// Effect drives both this plugin and the live-activity plugin from the
// same `vm.active_session` transitions.
#[wasm_bindgen(inline_js = "
    function bg_audio_invoke(cmd, args) {
        try {
            const invoke =
                window.__TAURI__?.core?.invoke ??
                window.__TAURI_INTERNALS__?.invoke;
            if (invoke) invoke(cmd, args ?? {});
        } catch(e) {}
    }
    export function begin_session(title, started_at) {
        bg_audio_invoke('plugin:background-audio|begin_session', { args: { title, started_at } });
    }
    export function set_now_playing(title, position_label, started_at) {
        bg_audio_invoke('plugin:background-audio|set_now_playing', { args: { title, position_label, started_at } });
    }
    export function end_session() {
        bg_audio_invoke('plugin:background-audio|end_session');
    }
")]
extern "C" {
    /// Activate the iOS audio session + seed lock-screen Now Playing
    /// when a practice session starts. No-op outside Tauri.
    pub fn begin_session(title: &str, started_at: &str);
    /// Update lock-screen Now Playing on item advance. No-op outside Tauri.
    pub fn set_now_playing(title: &str, position_label: &str, started_at: &str);
    /// Release the audio session + clear Now Playing on session end.
    /// No-op outside Tauri.
    pub fn end_session();
}

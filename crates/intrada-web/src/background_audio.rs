use wasm_bindgen::prelude::*;

// Calls Tauri plugin-background-audio commands via the Tauri IPC invoke
// bridge. All functions are no-ops outside a Tauri context (web browser,
// E2E tests). The try/catch in JS ensures plugin failures never surface
// as Rust errors — the timer continues working even if the audio session
// can't be activated.
//
// Phase B: the plugin's Rust commands return Ok without side-effects.
// Phase C swaps in the Swift impl behind the same command names; no
// change needed on this side.
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
        bg_audio_invoke('plugin:background-audio|begin_session', { title, started_at });
    }
    export function set_now_playing(title, position_label, started_at) {
        bg_audio_invoke('plugin:background-audio|set_now_playing', { title, position_label, started_at });
    }
    export function end_session() {
        bg_audio_invoke('plugin:background-audio|end_session');
    }
")]
extern "C" {
    /// Activate the iOS audio session + seed lock-screen Now Playing
    /// when a practice session starts. Phase B: plugin command returns
    /// Ok with no native effect; Phase C wires AVAudioSession.
    pub fn begin_session(title: &str, started_at: &str);
    /// Update lock-screen Now Playing on item advance. Phase B: no-op.
    pub fn set_now_playing(title: &str, position_label: &str, started_at: &str);
    /// Release the audio session + clear Now Playing on session end.
    /// Phase B: no-op.
    pub fn end_session();
}

use wasm_bindgen::prelude::*;

// Calls Tauri plugin-haptics commands via the Tauri IPC invoke bridge.
// All functions are no-ops outside a Tauri context (web browser, E2E tests).
// The try/catch in JS ensures haptic failures never surface as Rust errors.
#[wasm_bindgen(inline_js = "
    function haptics_invoke(cmd, args) {
        try {
            const invoke =
                window.__TAURI__?.core?.invoke ??
                window.__TAURI_INTERNALS__?.invoke;
            if (invoke) invoke(cmd, args ?? {});
        } catch(e) {}
    }
    export function haptic_selection() {
        haptics_invoke('plugin:haptics|selection_feedback');
    }
    export function haptic_light() {
        haptics_invoke('plugin:haptics|impact_feedback', { style: 'light' });
    }
    export function haptic_medium() {
        haptics_invoke('plugin:haptics|impact_feedback', { style: 'medium' });
    }
    export function haptic_success() {
        haptics_invoke('plugin:haptics|notification_feedback', { type: 'success' });
    }
    export function haptic_warning() {
        haptics_invoke('plugin:haptics|notification_feedback', { type: 'warning' });
    }
")]
extern "C" {
    /// Fired on tab/segment selection changes.
    pub fn haptic_selection();
    /// Fired on standard button taps.
    pub fn haptic_light();
    /// Fired on significant actions.
    pub fn haptic_medium();
    /// Fired on successful saves/completions.
    pub fn haptic_success();
    /// Fired on destructive action confirms.
    pub fn haptic_warning();
}

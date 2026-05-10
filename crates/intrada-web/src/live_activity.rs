use wasm_bindgen::prelude::*;

// Calls Tauri plugin-live-activity commands via the Tauri IPC invoke
// bridge. All functions are no-ops outside a Tauri context (web browser,
// E2E tests, iPad — Live Activities are iPhone-only). The try/catch in
// JS ensures plugin failures never surface as Rust errors. The Rust
// plugin captures failures into Sentry so we still get telemetry
// without surfacing UI banners users can't act on.
//
// Mirrors the shape of `intrada-web/src/background_audio.rs` so both
// plugins can be driven from the same lifecycle Effect (see
// `mount_session_lifecycle` in `session_lifecycle.rs`).
#[wasm_bindgen(inline_js = "
    function live_activity_invoke(cmd, args) {
        try {
            const invoke =
                window.__TAURI__?.core?.invoke ??
                window.__TAURI_INTERNALS__?.invoke;
            if (invoke) invoke(cmd, args ?? {}).catch(function(e){ if (typeof console !== 'undefined') console.debug('plugin invoke failed:', cmd, e); });
        } catch(e) {}
    }
    export function begin(item_title, position_label, started_at, planned_duration_secs) {
        live_activity_invoke('plugin:live-activity|begin', {
            args: {
                item_title,
                position_label,
                started_at,
                planned_duration_secs: planned_duration_secs ?? null,
            },
        });
    }
    export function update(item_title, position_label, started_at, planned_duration_secs) {
        live_activity_invoke('plugin:live-activity|update', {
            args: {
                item_title,
                position_label,
                started_at,
                planned_duration_secs: planned_duration_secs ?? null,
            },
        });
    }
    export function end() {
        live_activity_invoke('plugin:live-activity|end');
    }
")]
extern "C" {
    /// Start a Live Activity for the active practice session. No-op
    /// outside Tauri / on iPad.
    pub fn begin(
        item_title: &str,
        position_label: &str,
        started_at: &str,
        planned_duration_secs: Option<u32>,
    );
    /// Update the Live Activity on item advance. No-op outside Tauri.
    pub fn update(
        item_title: &str,
        position_label: &str,
        started_at: &str,
        planned_duration_secs: Option<u32>,
    );
    /// End the Live Activity. No-op outside Tauri.
    pub fn end();
}

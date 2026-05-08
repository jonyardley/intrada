use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use intrada_core::ViewModel;

// Calls Tauri plugin-background-audio commands via the Tauri IPC invoke
// bridge. All functions are no-ops outside a Tauri context (web browser,
// E2E tests). The try/catch in JS ensures plugin failures never surface
// as Rust errors — the timer continues working even if the audio session
// can't be activated. The Rust plugin captures failures into Sentry so
// we still get telemetry without surfacing UI banners users can't act on.
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
    /// when a practice session starts. No-op outside Tauri.
    pub fn begin_session(title: &str, started_at: &str);
    /// Update lock-screen Now Playing on item advance. No-op outside Tauri.
    pub fn set_now_playing(title: &str, position_label: &str, started_at: &str);
    /// Release the audio session + clear Now Playing on session end.
    /// No-op outside Tauri.
    pub fn end_session();
}

/// Mount the background-audio lifecycle Effect on the global `ViewModel`
/// signal. Tracks `current_item_started_at` across renders so we can tell
/// session start (None → Some), item advance (anchor change), and session
/// end (Some → None) apart with a single Effect.
///
/// Must be called from a Leptos owner scope that lives for the duration of
/// the user's authenticated app session — i.e. `AuthenticatedApp`, not a
/// per-route component. Mounting this inside `<SessionTimer>` would leak
/// the audio session: when the user navigates away from `/sessions/active`
/// without finishing (e.g. taps a tab, hits Discard from `/sessions/new`)
/// the timer unmounts before the Effect can observe `Some → None`, so
/// `end_session()` never fires and the AVAudioSession stays active until
/// the OS reclaims it.
///
/// The Effect re-fires on every ViewModel push (coarser than ideal — any
/// unrelated VM mutation triggers it) but the anchor-equality guard makes
/// that idempotent.
pub fn mount_background_audio_lifecycle(view_model: RwSignal<ViewModel>) {
    let prev_anchor: RwSignal<Option<String>> = RwSignal::new(None);
    Effect::new(move |_| {
        let next = view_model.with(|vm| {
            vm.active_session.as_ref().map(|a| {
                (
                    a.current_item_title.clone(),
                    a.current_position,
                    a.total_items,
                    a.current_item_started_at.clone(),
                )
            })
        });
        let prev = prev_anchor.get_untracked();
        match (prev, next) {
            (None, Some((title, _pos, _total, started_at))) => {
                begin_session(&title, &started_at);
                prev_anchor.set(Some(started_at));
            }
            (Some(prev_anchor_val), Some((title, pos, total, started_at)))
                if prev_anchor_val != started_at =>
            {
                let position_label = format!("Item {} of {}", pos + 1, total);
                set_now_playing(&title, &position_label, &started_at);
                prev_anchor.set(Some(started_at));
            }
            (Some(_), None) => {
                end_session();
                prev_anchor.set(None);
            }
            _ => {}
        }
    });
}

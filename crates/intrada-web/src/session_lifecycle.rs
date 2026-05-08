use leptos::prelude::*;

use intrada_core::ViewModel;

use crate::{background_audio, live_activity};

/// Mount the session-lifecycle Effect on the global `ViewModel` signal.
/// Tracks `current_item_started_at` across renders so we can tell session
/// start (None → Some), item advance (anchor change), and session end
/// (Some → None) apart with a single Effect, then fans the transition
/// out to *both* native plugins:
///
/// - `background-audio` (#309) — keeps the timer alive while the device
///   is locked + shows a Now Playing card.
/// - `live-activity` (#474) — Lock Screen + Dynamic Island for the
///   active session. Layered on top, not a replacement.
///
/// One Effect, two side-effects: avoids two parallel observers racing
/// with divergent ordering.
///
/// # Where this must be mounted
///
/// Must be called from a Leptos owner scope that lives for the duration
/// of the user's authenticated app session — i.e. `AuthenticatedApp`,
/// not a per-route component. Mounting this inside `<SessionTimer>`
/// would leak both plugins' state: when the user navigates away from
/// `/sessions/active` without finishing (e.g. taps a tab, hits Discard
/// from `/sessions/new`) the timer unmounts before the Effect can
/// observe `Some → None`, so the `end` transition never fires and the
/// AVAudioSession + Live Activity stay active until the OS reclaims
/// them. (See #309 Phase D for the original observation.)
///
/// # Reactivity
///
/// The Effect re-fires on every ViewModel push (coarser than ideal —
/// any unrelated VM mutation triggers it) but the anchor-equality guard
/// makes that idempotent. If we ever care about the wasted work, a
/// Memo<Option<String>> over current_item_started_at would isolate the
/// dependency.
pub fn mount_session_lifecycle(view_model: RwSignal<ViewModel>) {
    let prev_anchor: RwSignal<Option<String>> = RwSignal::new(None);
    Effect::new(move |_| {
        let next = view_model.with(|vm| {
            vm.active_session.as_ref().map(|a| {
                (
                    a.current_item_title.clone(),
                    a.current_position,
                    a.total_items,
                    a.current_item_started_at.clone(),
                    a.current_planned_duration_secs,
                )
            })
        });
        let prev = prev_anchor.get_untracked();
        match (prev, next) {
            // Session start: vm.active_session went from None to Some.
            (None, Some((title, pos, total, started_at, planned))) => {
                let position_label = format!("Item {} of {}", pos + 1, total);
                background_audio::begin_session(&title, &started_at);
                live_activity::begin(&title, &position_label, &started_at, planned);
                prev_anchor.set(Some(started_at));
            }
            // Item advance: same session, new item anchor.
            (Some(prev_anchor_val), Some((title, pos, total, started_at, planned)))
                if prev_anchor_val != started_at =>
            {
                let position_label = format!("Item {} of {}", pos + 1, total);
                background_audio::set_now_playing(&title, &position_label, &started_at);
                live_activity::update(&title, &position_label, &started_at, planned);
                prev_anchor.set(Some(started_at));
            }
            // Session end: vm.active_session went from Some to None.
            (Some(_), None) => {
                background_audio::end_session();
                live_activity::end();
                prev_anchor.set(None);
            }
            _ => {}
        }
    });
}

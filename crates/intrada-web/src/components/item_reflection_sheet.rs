use leptos::prelude::*;

use intrada_core::{Event, ItemKind, SessionEvent};

use crate::components::{
    BottomSheet, Button, ButtonSize, ButtonVariant, InlineTypeIndicator, RatingChips,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, ItemType, SharedCore};
use intrada_web::validation::validate_achieved_tempo_input;

/// Snapshot of the item the user just completed — passed in by `SessionTimer`
/// when it opens the sheet. Carrying the values explicitly (rather than
/// re-reading the view model inside the sheet) keeps the sheet decoupled from
/// the active-session shape and avoids races where `current_position` has
/// already advanced.
#[derive(Clone, Debug)]
pub struct ItemReflectionTarget {
    pub entry_id: String,
    pub initial_score: Option<u8>,
    pub initial_tempo: Option<u16>,
    pub initial_notes: Option<String>,
}

/// Pencil reference: `hZfKR` ("Practice / Item Transition" interstitial).
/// Bottom sheet that captures self-rating + achieved tempo + notes for the
/// item the user just completed, then advances to the next item (or finishes
/// the session for the last item).
///
/// Open this on tap of the primary "Next Item" / "Finish Session" CTA — the
/// dispatch is deferred until the user hits Continue inside the sheet, or
/// fired immediately on Skip without capturing reflection data.
///
/// Dismissal semantics:
/// - **Continue** — captures reflection + advances.
/// - **Skip scoring** — discards captured reflection + advances.
/// - **Backdrop tap / swipe-down / Escape** — closes WITHOUT advancing.
///   The user is back on the same item, timer still running, CTA still
///   "Next Item" / "Finish Session". Re-tapping the CTA re-opens the sheet
///   pre-populated from the entry's persisted values (any in-progress edits
///   from the dismissed session are lost — there's no draft preservation).
///   This matches iOS Mail-compose "swipe to dismiss draft" behaviour.
#[component]
pub fn ItemReflectionSheet(
    open: RwSignal<bool>,
    /// Title of the next item, or `None` for the last item.
    #[prop(into)]
    next_item_title: Signal<Option<String>>,
    /// Type of the next item — drives the badge under the title. Ignored
    /// when `next_item_title` is None.
    #[prop(into)]
    next_item_type: Signal<Option<ItemKind>>,
    /// Snapshot of the just-completed entry, captured at the moment the
    /// sheet was opened. None when the sheet is closed.
    #[prop(into)]
    target: Signal<Option<ItemReflectionTarget>>,
    /// Position counter to render at the top of the sheet (e.g. "Item 2 of 3").
    #[prop(into)]
    position_label: Signal<String>,
    /// Fired on Continue (after the Update* events have been dispatched) and
    /// on Skip scoring. The parent advances the session — `NextItem` or
    /// `FinishSession` — based on its own state.
    on_advance: Callback<()>,
) -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<intrada_core::ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    // Local form state — reset when the sheet opens against a fresh target.
    let score = RwSignal::new(Option::<u8>::None);
    let tempo_str = RwSignal::new(String::new());
    let notes_str = RwSignal::new(String::new());
    let tempo_error = RwSignal::new(Option::<String>::None);

    // Re-seed local state each time a new target arrives. Without this the
    // sheet would carry the previous item's values into the next reflection.
    Effect::new(move |_| {
        if let Some(t) = target.get() {
            score.set(t.initial_score);
            tempo_str.set(t.initial_tempo.map(|n| n.to_string()).unwrap_or_default());
            notes_str.set(t.initial_notes.unwrap_or_default());
            tempo_error.set(None);
        }
    });

    let on_close = Callback::new(move |_| open.set(false));

    // Wrapped as a `Callback<bool>` so both Continue and Skip can call it
    // (Callbacks are Copy; a plain `move` closure with this many captures
    // is not, and would be moved into whichever handler ran first).
    //
    // Advance must run FIRST: `NextItem` / `FinishSession` is what flips
    // the just-completed entry's status to `Completed`. The core's
    // `UpdateEntryScore` / `Tempo` / `Notes` handlers gate on
    // `entry.status == Completed` — update-then-advance would be a no-op.
    let dispatch_advance = Callback::new(move |capture: bool| {
        open.set(false);
        on_advance.run(());

        if capture {
            if let Some(t) = target.get_untracked() {
                let entry_id = t.entry_id;

                // Score
                let event = Event::Session(SessionEvent::UpdateEntryScore {
                    entry_id: entry_id.clone(),
                    score: score.get_untracked(),
                });
                let core_ref = core.borrow();
                let effects = core_ref.process_event(event);
                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                drop(core_ref);

                // Tempo — only dispatch when the input parses cleanly. Mirrors
                // the summary screen's defensive parse.
                let raw = tempo_str.get_untracked();
                let parsed: Option<u16> = if raw.trim().is_empty() {
                    None
                } else {
                    raw.trim().parse().ok()
                };
                let event = Event::Session(SessionEvent::UpdateEntryTempo {
                    entry_id: entry_id.clone(),
                    tempo: parsed,
                });
                let core_ref = core.borrow();
                let effects = core_ref.process_event(event);
                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                drop(core_ref);

                // Notes
                let n = notes_str.get_untracked();
                let notes = if n.trim().is_empty() { None } else { Some(n) };
                let event = Event::Session(SessionEvent::UpdateEntryNotes { entry_id, notes });
                let core_ref = core.borrow();
                let effects = core_ref.process_event(event);
                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
            }
        }
    });

    let on_continue = Callback::new(move |_| {
        // Validate tempo before committing — same rule as summary input.
        let raw = tempo_str.get_untracked();
        if let Some(err) = validate_achieved_tempo_input(&raw) {
            tempo_error.set(Some(err));
            return;
        }
        tempo_error.set(None);
        dispatch_advance.run(true);
    });

    let on_skip = Callback::new(move |_| {
        dispatch_advance.run(false);
    });

    let header_label = move || -> &'static str {
        if next_item_title.with(|t| t.is_some()) {
            "Up Next"
        } else {
            "Last Item"
        }
    };

    let title_text = move || -> String {
        next_item_title
            .get()
            .unwrap_or_else(|| "Session complete".to_string())
    };

    let badge_view = move || {
        next_item_type.get().map(|kind| {
            let item_type = match kind {
                ItemKind::Piece => ItemType::Piece,
                ItemKind::Exercise => ItemType::Exercise,
            };
            view! {
                <div class="mt-1">
                    <InlineTypeIndicator item_type=item_type />
                </div>
            }
        })
    };

    view! {
        <BottomSheet open=open on_close=on_close>
            <div class="space-y-5">
                <p class="text-xs uppercase tracking-wider text-muted text-center">
                    {move || position_label.get()}
                </p>

                <div class="space-y-2">
                    <p class="text-xs uppercase tracking-wider text-muted">
                        {header_label}
                    </p>
                    <h3 class="text-xl font-bold text-primary font-heading">
                        {title_text}
                    </h3>
                    {badge_view}
                </div>

                <div class="border-t border-border-default pt-4 space-y-4">
                    // Rating chips — local signal, no dispatch yet (the
                    // sheet defers UpdateEntryScore until Continue/Skip).
                    <div>
                        <p class="text-sm text-secondary mb-2">"How did it go?"</p>
                        <RatingChips
                            selected=score
                            on_change=Callback::new(move |next: Option<u8>| score.set(next))
                        />
                    </div>

                    // Tempo (BPM) — number input matching summary screen
                    // styling. Inline error below input on parse failure.
                    <div>
                        <label class="text-sm text-secondary" for="reflection-tempo">
                            "Tempo (BPM)"
                        </label>
                        <input
                            id="reflection-tempo"
                            type="number"
                            inputmode="numeric"
                            placeholder="1\u{2013}500"
                            class="input-base mt-1"
                            class:input-error=move || tempo_error.get().is_some()
                            bind:value=tempo_str
                        />
                        {move || tempo_error.get().map(|err| view! {
                            <p class="text-xs text-danger mt-1">{err}</p>
                        })}
                    </div>

                    // Notes — single-line text input per Pencil. The summary
                    // screen has a multiline editor; this is the lightweight
                    // mid-session capture.
                    <div>
                        <label class="text-sm text-secondary" for="reflection-notes">
                            "Notes"
                        </label>
                        <input
                            id="reflection-notes"
                            type="text"
                            placeholder="A quick note (optional)"
                            class="input-base mt-1"
                            bind:value=notes_str
                        />
                    </div>
                </div>

                <div class="space-y-3 pt-2">
                    <Button
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Hero
                        full_width=true
                        on_click=on_continue
                    >
                        "Continue"
                    </Button>
                    <button
                        type="button"
                        class="w-full text-center text-sm text-muted hover:text-primary motion-safe:transition-colors"
                        on:click=move |_| on_skip.run(())
                    >
                        "Skip scoring"
                    </button>
                </div>
            </div>
        </BottomSheet>
    }
}

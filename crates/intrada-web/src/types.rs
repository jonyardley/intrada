use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::RwSignal;
use send_wrapper::SendWrapper;

use intrada_core::Intrada;

/// Wrapper around Core that is safe to use in Leptos reactive contexts (WASM is single-threaded).
pub type SharedCore = SendWrapper<Rc<RefCell<Core<Intrada>>>>;

/// Shell-side loading signal — avoids polluting the pure core with UI-only state.
/// Provided via Leptos context; toggled by async HTTP handlers in `process_effects()`.
pub type IsLoading = RwSignal<bool>;

/// Shell-side submitting signal — tracks whether a form mutation is in-flight.
/// Used to disable submit/delete buttons and prevent duplicate submissions (FR-010).
pub type IsSubmitting = RwSignal<bool>;

/// Identifies whether a library item is a Piece or an Exercise.
/// Used by the unified add/edit forms to drive tab state, validation, and submission logic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Piece,
    Exercise,
}

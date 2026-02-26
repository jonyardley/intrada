use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::{Get, GetUntracked, RwSignal, Set};
use send_wrapper::SendWrapper;

use intrada_core::Intrada;

/// Wrapper around Core that is safe to use in Leptos reactive contexts (WASM is single-threaded).
pub type SharedCore = SendWrapper<Rc<RefCell<Core<Intrada>>>>;

/// Shell-side loading signal — avoids polluting the pure core with UI-only state.
/// Provided via Leptos context; toggled by async HTTP handlers in `process_effects()`.
///
/// Newtype wrapper (not a type alias) so that `provide_context` / `expect_context`
/// use a distinct `TypeId` and don't collide with other `RwSignal<bool>` contexts.
#[derive(Clone, Copy)]
pub struct IsLoading(pub RwSignal<bool>);

impl IsLoading {
    pub fn new(val: bool) -> Self {
        Self(RwSignal::new(val))
    }
    pub fn get(&self) -> bool {
        self.0.get()
    }
    pub fn get_untracked(&self) -> bool {
        self.0.get_untracked()
    }
    pub fn set(&self, val: bool) {
        self.0.set(val);
    }
}

/// Shell-side submitting signal — tracks whether a form mutation is in-flight.
/// Used to disable submit/delete buttons and prevent duplicate submissions (FR-010).
///
/// Newtype wrapper for the same reason as [`IsLoading`].
#[derive(Clone, Copy)]
pub struct IsSubmitting(pub RwSignal<bool>);

impl IsSubmitting {
    pub fn new(val: bool) -> Self {
        Self(RwSignal::new(val))
    }
    pub fn get(&self) -> bool {
        self.0.get()
    }
    pub fn get_untracked(&self) -> bool {
        self.0.get_untracked()
    }
    pub fn set(&self, val: bool) {
        self.0.set(val);
    }
}

/// Identifies whether a library item is a Piece or an Exercise.
/// Used by the unified add/edit forms to drive tab state, validation, and submission logic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Piece,
    Exercise,
}

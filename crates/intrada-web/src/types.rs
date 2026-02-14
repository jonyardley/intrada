use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use send_wrapper::SendWrapper;

use intrada_core::Intrada;

/// Wrapper around Core that is safe to use in Leptos reactive contexts (WASM is single-threaded).
pub type SharedCore = SendWrapper<Rc<RefCell<Core<Intrada>>>>;

/// Identifies whether a library item is a Piece or an Exercise.
/// Used by the unified add/edit forms to drive tab state, validation, and submission logic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Piece,
    Exercise,
}

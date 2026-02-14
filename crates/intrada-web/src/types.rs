use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use send_wrapper::SendWrapper;

use intrada_core::Intrada;

/// Wrapper around Core that is safe to use in Leptos reactive contexts (WASM is single-threaded).
pub type SharedCore = SendWrapper<Rc<RefCell<Core<Intrada>>>>;

#[derive(Clone, Debug, PartialEq)]
pub enum ViewState {
    List,
    Detail(String),
    AddPiece,
    AddExercise,
    EditPiece(String),
    EditExercise(String),
}

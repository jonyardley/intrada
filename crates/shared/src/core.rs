use crux_core::bridge::{Bridge, EffectId};
use crux_core::Core;

use intrada_core::app::Intrada;

/// The main FFI interface used by iOS/Android shells.
///
/// Wraps a `Bridge<Intrada>` and exposes three methods:
/// - `update(&[u8]) -> Vec<u8>` — deserialises a serialised `Event`,
///   runs the core update loop, returns serialised effect requests
/// - `resolve(u32, &[u8]) -> Vec<u8>` — resolves an outstanding effect
///   by its ID, returns any new serialised effect requests
/// - `view() -> Vec<u8>` — returns the serialised `ViewModel`
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct CoreFFI {
    core: Bridge<Intrada>,
}

impl Default for CoreFFI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl CoreFFI {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            core: Bridge::new(Core::new()),
        }
    }

    /// Process a serialised `Event` and return serialised effect requests.
    #[must_use]
    pub fn update(&self, data: &[u8]) -> Vec<u8> {
        let mut effects = Vec::new();
        match self.core.update(data, &mut effects) {
            Ok(()) => effects,
            Err(e) => panic!("core update failed: {e}"),
        }
    }

    /// Resolve an outstanding effect by its numeric ID with a serialised response,
    /// returning any new serialised effect requests.
    #[must_use]
    pub fn resolve(&self, id: u32, data: &[u8]) -> Vec<u8> {
        let mut effects = Vec::new();
        match self.core.resolve(EffectId(id), data, &mut effects) {
            Ok(()) => effects,
            Err(e) => panic!("core resolve failed: {e}"),
        }
    }

    /// Return the serialised `ViewModel`.
    #[must_use]
    pub fn view(&self) -> Vec<u8> {
        let mut view_model = Vec::new();
        match self.core.view(&mut view_model) {
            Ok(()) => view_model,
            Err(e) => panic!("core view failed: {e}"),
        }
    }
}

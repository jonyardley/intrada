use crux_core::{
    bridge::{Bridge, EffectId},
    Core,
};

use crate::Intrada;

/// Error surfaced across the FFI boundary when the bridge fails to
/// (de)serialize an event/effect or resolve a request. The shell handles it as
/// a thrown error — the dumb-pipe contract bans `try!`/force-unwrap, so the
/// bridge returns a `Result` rather than panicking (cf. the crux `counter`
/// example, which panics and notes "in production handle the error properly").
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("core bridge error: {0}")]
    Bridge(String),
}

/// The single FFI surface the native shell talks to. Wraps the Crux `Bridge`
/// and exposes the three byte-buffer methods (bincode in, bincode out).
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

    /// Process a serialized `Event`; returns the serialized effect requests.
    pub fn update(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .update(data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    /// Resolve an outstanding effect by id; returns any follow-up requests.
    pub fn resolve(&self, id: u32, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .resolve(EffectId(id), data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    /// Serialize the current `ViewModel`.
    pub fn view(&self) -> Result<Vec<u8>, CoreError> {
        let mut view = Vec::new();
        self.core
            .view(&mut view)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_serializes_initial_view() {
        let core = CoreFFI::new();
        let view = core.view().expect("initial view should serialize");
        assert!(!view.is_empty(), "serialized ViewModel should be non-empty");
    }
}

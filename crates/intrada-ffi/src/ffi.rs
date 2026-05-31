use crux_core::{
    bridge::{Bridge, EffectId},
    Core,
};

use crate::Intrada;

// Returned (not panicked) so the shell handles it per the no-`try!` contract —
// the crux `counter` example panics but says to do this in production.
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("core bridge error: {0}")]
    Bridge(String),
}

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

    pub fn update(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .update(data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    pub fn resolve(&self, id: u32, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .resolve(EffectId(id), data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

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

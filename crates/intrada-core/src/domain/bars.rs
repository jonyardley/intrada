use serde::{Deserialize, Serialize};

/// "Bar 12" is `12..12`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BarRange {
    pub first: u16,
    pub last: u16,
}

impl BarRange {
    pub fn caption(&self) -> String {
        if self.first == self.last {
            format!("Bar {}", self.first)
        } else {
            format!("Bars {} to {}", self.first, self.last)
        }
    }
}

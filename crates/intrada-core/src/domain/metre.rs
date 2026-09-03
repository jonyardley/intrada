use serde::{Deserialize, Serialize};

/// A time signature, plus how the bar is counted. `groups` is what makes
/// "sounds on group starts" mean something in 7/8; `None` = undifferentiated.
/// Invariants (beats 2 to 12, unit 2, 4 or 8, groups summing to `beats`) are
/// enforced in `validation::validate_metre`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Metre {
    pub beats: u8,
    pub unit: u8,
    pub groups: Option<Vec<u8>>,
}

impl Default for Metre {
    fn default() -> Self {
        Metre {
            beats: 4,
            unit: 4,
            groups: None,
        }
    }
}

impl Metre {
    /// `achieved_tempo` means crotchet BPM whatever the click displayed, so a
    /// quaver or minim reading is converted here, rounding half away from zero:
    /// `♪ = 169` becomes 85, not 84 (specs/practice-instruments.md, question 2).
    pub fn crotchet_bpm(&self, displayed: u16) -> u16 {
        let unit = u32::from(self.unit.max(1));
        let crotchets = (u32::from(displayed) * 4 + unit / 2) / unit;
        u16::try_from(crotchets).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metre(beats: u8, unit: u8) -> Metre {
        Metre {
            beats,
            unit,
            groups: None,
        }
    }

    /// Inputs a musician would actually set, asserting the property the tempo
    /// trend needs: every stored value is a crotchet count comparable with
    /// every other, at the rounding the spec states.
    #[test]
    fn every_displayed_tempo_normalises_to_a_comparable_crotchet_bpm() {
        let table: [(Metre, u16, u16); 7] = [
            (metre(4, 4), 120, 120),
            (metre(3, 4), 66, 66),
            (metre(7, 8), 168, 84),
            (metre(7, 8), 169, 85),
            (metre(6, 8), 1, 1),
            (metre(2, 2), 60, 120),
            (metre(2, 2), 40, 80),
        ];
        for (metre, displayed, expected) in table {
            assert_eq!(
                metre.crotchet_bpm(displayed),
                expected,
                "{displayed} in {}/{}",
                metre.beats,
                metre.unit
            );
        }
    }

    #[test]
    fn the_default_metre_is_common_time() {
        assert_eq!(Metre::default(), metre(4, 4));
    }
}

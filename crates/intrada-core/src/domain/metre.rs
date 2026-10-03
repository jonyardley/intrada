use serde::{Deserialize, Serialize};

use crate::domain::practice_defaults::ClickStart;
use crate::validation;

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
        let unit = u32::from(self.unit);
        let crotchets = (u32::from(displayed) * 4 + unit / 2) / unit;
        u16::try_from(crotchets).unwrap_or(u16::MAX)
    }

    /// A stored crotchet tempo shown back in this metre's unit, at the same
    /// rounding, so the shell never converts a tempo (#1761).
    pub fn displayed_bpm(&self, crotchets: u16) -> u16 {
        let displayed = (u32::from(crotchets) * u32::from(self.unit) + 2) / 4;
        u16::try_from(displayed).unwrap_or(u16::MAX)
    }

    /// The click's band counted in this metre's unit: a 6/8 piece at dotted
    /// crotchet = 80 is quaver = 240, which the crotchet ceiling would refuse
    /// (#1499).
    pub fn click_tempo_band(&self) -> TempoBand {
        TempoBand {
            unit: self.unit,
            min: self.displayed_bpm(CLICK_TEMPO_MIN),
            max: self.displayed_bpm(CLICK_TEMPO_MAX),
        }
    }

    /// Where the click starts for an item: its own BPM in this metre's band,
    /// or the default, and whether that is still the item's number (#1942).
    pub fn click_seed(&self, target: Option<u16>) -> (u16, bool) {
        let band = self.click_tempo_band();
        let seed = target
            .unwrap_or(CLICK_TEMPO_DEFAULT)
            .clamp(band.min, band.max);
        (seed, target == Some(seed))
    }

    /// The groupings a musician would reach for; anything else stays
    /// ungrouped. Offered here, validated in `validation::validate_metre`.
    pub fn groupings(beats: u8) -> Vec<Vec<u8>> {
        let options: &[&[u8]] = match beats {
            5 => &[&[3, 2], &[2, 3]],
            6 => &[&[3, 3], &[2, 2, 2]],
            7 => &[&[3, 2, 2], &[2, 3, 2], &[2, 2, 3]],
            8 => &[&[3, 3, 2], &[3, 2, 3], &[2, 3, 3]],
            9 => &[&[3, 3, 3], &[2, 2, 2, 3]],
            10 => &[&[3, 3, 2, 2], &[2, 3, 2, 3]],
            11 => &[&[3, 3, 3, 2], &[2, 2, 3, 2, 2]],
            12 => &[&[3, 3, 3, 3], &[2, 2, 2, 2, 2, 2]],
            _ => &[],
        };
        options.iter().map(|g| g.to_vec()).collect()
    }

    pub fn click_presets(&self) -> Vec<ClickPresetOption> {
        let mut offered = vec![ClickPreset::EveryBeat];
        if self.groups.is_some() {
            offered.push(ClickPreset::GroupStarts);
        }
        offered.push(ClickPreset::Downbeat);
        if self.groups.is_none() && self.beats >= 4 {
            offered.push(ClickPreset::Backbeat);
        }
        offered
            .into_iter()
            .map(|preset| ClickPresetOption {
                preset,
                sounding: preset.sounding(self),
            })
            .collect()
    }
}

// ── Click ──
// Crotchet BPM. Narrower than `validation::MAX_BPM`: a target outside it
// arrives clamped rather than showing a tempo the steppers cannot reach.
pub const CLICK_TEMPO_MIN: u16 = 40;
pub const CLICK_TEMPO_MAX: u16 = 208;
pub const CLICK_TEMPO_STEP: u16 = 2;
pub const CLICK_TEMPO_DEFAULT: u16 = 96;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoBand {
    pub unit: u8,
    pub min: u16,
    pub max: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ClickPreset {
    EveryBeat,
    GroupStarts,
    Downbeat,
    Backbeat,
}

impl ClickPreset {
    /// Bitmask over `metre.beats`, LSB = beat 1.
    pub fn sounding(self, metre: &Metre) -> u16 {
        match self {
            ClickPreset::EveryBeat => ClickStart::EveryBeat.sounding(metre),
            ClickPreset::Backbeat => ClickStart::TwoAndFour.sounding(metre),
            ClickPreset::Downbeat => 1,
            ClickPreset::GroupStarts => metre.groups.as_ref().map_or(1, |groups| {
                groups
                    .iter()
                    .scan(0u16, |start, &group| {
                        let bit = 1 << *start;
                        *start += u16::from(group);
                        Some(bit)
                    })
                    .fold(0, |mask, bit| mask | bit)
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ClickPresetOption {
    pub preset: ClickPreset,
    pub sounding: u16,
}

/// One bar the click sheet can set, with the patterns it offers. The sheet's
/// groupings for a beat count are the rows with that `beats` and `Some` groups.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ClickBarOption {
    pub beats: u8,
    pub groups: Option<Vec<u8>>,
    pub presets: Vec<ClickPresetOption>,
}

/// The bars the click sheet names before "Other".
pub fn click_metre_presets() -> Vec<Metre> {
    vec![
        Metre {
            beats: 3,
            unit: 4,
            groups: None,
        },
        Metre::default(),
        Metre {
            beats: 6,
            unit: 8,
            groups: Some(vec![3, 3]),
        },
    ]
}

/// Every beat count the core accepts, ungrouped and in each offered grouping.
/// Patterns do not depend on the unit, so the bar is keyed without one.
pub fn click_bars() -> Vec<ClickBarOption> {
    (validation::MIN_METRE_BEATS..=validation::MAX_METRE_BEATS)
        .flat_map(|beats| {
            std::iter::once(None)
                .chain(Metre::groupings(beats).into_iter().map(Some))
                .map(move |groups| {
                    let metre = Metre {
                        beats,
                        unit: 4,
                        groups,
                    };
                    ClickBarOption {
                        presets: metre.click_presets(),
                        beats,
                        groups: metre.groups,
                    }
                })
        })
        .collect()
}

pub fn click_tempo_bands() -> Vec<TempoBand> {
    validation::METRE_UNITS
        .iter()
        .map(|&unit| {
            Metre {
                beats: 4,
                unit,
                groups: None,
            }
            .click_tempo_band()
        })
        .collect()
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
        let table: [(Metre, u16, u16); 13] = [
            (metre(4, 4), 120, 120),
            (metre(3, 4), 66, 66),
            (metre(7, 8), 168, 84),
            (metre(7, 8), 169, 85),
            (metre(6, 8), 1, 1),
            (metre(2, 2), 60, 120),
            (metre(2, 2), 40, 80),
            // The ends of the click's band in each unit.
            (metre(4, 4), 40, 40),
            (metre(4, 4), 208, 208),
            (metre(2, 2), 20, 40),
            (metre(2, 2), 104, 208),
            (metre(6, 8), 80, 40),
            (metre(6, 8), 416, 208),
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

    /// A stamp read back on the item-complete sheet counts in the unit of the
    /// click that made it (#1761 rule 6).
    #[test]
    fn a_stored_crotchet_tempo_reads_back_in_the_clicks_own_unit() {
        let table: [(Metre, u16, u16); 6] = [
            (metre(4, 4), 108, 108),
            (metre(7, 8), 84, 168),
            (metre(7, 8), 85, 170),
            (metre(2, 2), 120, 60),
            (metre(2, 2), 85, 43),
            (metre(2, 2), 416, 208),
        ];
        for (metre, crotchets, expected) in table {
            assert_eq!(
                metre.displayed_bpm(crotchets),
                expected,
                "{crotchets} crotchets in {}/{}",
                metre.beats,
                metre.unit
            );
        }
    }

    /// The band a stepper offers in each unit is the crotchet band read back
    /// in that unit; deleting the scaling caps 6/8 at 208 quavers.
    #[test]
    fn the_click_band_counts_in_each_beat_unit() {
        assert_eq!(
            click_tempo_bands(),
            vec![
                TempoBand {
                    unit: 2,
                    min: 20,
                    max: 104
                },
                TempoBand {
                    unit: 4,
                    min: 40,
                    max: 208
                },
                TempoBand {
                    unit: 8,
                    min: 80,
                    max: 416
                },
            ]
        );
    }

    #[test]
    fn the_click_starts_on_the_items_tempo_inside_the_band() {
        let table: [(Metre, Option<u16>, (u16, bool)); 8] = [
            (metre(4, 4), Some(120), (120, true)),
            (metre(4, 4), None, (96, false)),
            (metre(4, 4), Some(30), (40, false)),
            (metre(4, 4), Some(240), (208, false)),
            (metre(6, 8), Some(240), (240, true)),
            (metre(6, 8), None, (96, false)),
            (metre(2, 2), Some(60), (60, true)),
            (metre(2, 2), Some(120), (104, false)),
        ];
        for (metre, target, expected) in table {
            assert_eq!(
                metre.click_seed(target),
                expected,
                "{target:?} in {}/{}",
                metre.beats,
                metre.unit
            );
        }
    }

    fn grouped(beats: u8, groups: &[u8]) -> Metre {
        Metre {
            beats,
            unit: 8,
            groups: Some(groups.to_vec()),
        }
    }

    fn offered(metre: &Metre) -> Vec<(ClickPreset, u16)> {
        metre
            .click_presets()
            .into_iter()
            .map(|o| (o.preset, o.sounding))
            .collect()
    }

    #[test]
    fn each_bar_offers_the_patterns_a_musician_would_reach_for() {
        use ClickPreset::*;
        assert_eq!(
            offered(&metre(4, 4)),
            vec![(EveryBeat, 0b1111), (Downbeat, 1), (Backbeat, 0b1010)]
        );
        assert_eq!(
            offered(&metre(3, 4)),
            vec![(EveryBeat, 0b111), (Downbeat, 1)]
        );
        assert_eq!(
            offered(&grouped(7, &[3, 2, 2])),
            vec![
                (EveryBeat, 0b111_1111),
                (GroupStarts, 0b010_1001),
                (Downbeat, 1)
            ]
        );
        assert_eq!(
            offered(&grouped(9, &[2, 2, 2, 3])),
            vec![
                (EveryBeat, 0b1_1111_1111),
                (GroupStarts, 0b0_0101_0101),
                (Downbeat, 1)
            ]
        );
        assert_eq!(
            offered(&grouped(6, &[3, 3])),
            vec![
                (EveryBeat, 0b11_1111),
                (GroupStarts, 0b00_1001),
                (Downbeat, 1)
            ]
        );
    }

    #[test]
    fn every_offered_bar_and_pattern_passes_the_cores_validation() {
        use crate::domain::session::ClickState;
        for bar in click_bars() {
            for &unit in &validation::METRE_UNITS {
                let metre = Metre {
                    beats: bar.beats,
                    unit,
                    groups: bar.groups.clone(),
                };
                for option in &bar.presets {
                    let state = ClickState {
                        metre: metre.clone(),
                        sounding: option.sounding,
                    };
                    assert!(
                        validation::validate_click_state(&state).is_ok(),
                        "{metre:?} {option:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_bars_cover_every_beat_count_ungrouped_and_grouped() {
        let bars = click_bars();
        for beats in validation::MIN_METRE_BEATS..=validation::MAX_METRE_BEATS {
            let groupings: Vec<_> = bars
                .iter()
                .filter(|b| b.beats == beats)
                .map(|b| b.groups.clone())
                .collect();
            let expected: Vec<_> = std::iter::once(None)
                .chain(Metre::groupings(beats).into_iter().map(Some))
                .collect();
            assert_eq!(groupings, expected, "{beats} beats");
        }
        assert_eq!(Metre::groupings(7).len(), 3);
        assert!(Metre::groupings(4).is_empty());
    }

    #[test]
    fn every_metre_preset_is_a_bar_the_sheet_offers() {
        let bars = click_bars();
        for preset in click_metre_presets() {
            assert!(
                bars.iter()
                    .any(|b| b.beats == preset.beats && b.groups == preset.groups),
                "{preset:?}"
            );
        }
    }

    #[test]
    fn the_sheet_names_three_bars_and_groups_from_five_beats() {
        assert_eq!(
            click_metre_presets(),
            vec![metre(3, 4), metre(4, 4), grouped(6, &[3, 3])]
        );
        let table: Vec<(u8, Vec<Vec<u8>>)> = (2..=12).map(|b| (b, Metre::groupings(b))).collect();
        assert_eq!(
            table,
            vec![
                (2, vec![]),
                (3, vec![]),
                (4, vec![]),
                (5, vec![vec![3, 2], vec![2, 3]]),
                (6, vec![vec![3, 3], vec![2, 2, 2]]),
                (7, vec![vec![3, 2, 2], vec![2, 3, 2], vec![2, 2, 3]]),
                (8, vec![vec![3, 3, 2], vec![3, 2, 3], vec![2, 3, 3]]),
                (9, vec![vec![3, 3, 3], vec![2, 2, 2, 3]]),
                (10, vec![vec![3, 3, 2, 2], vec![2, 3, 2, 3]]),
                (11, vec![vec![3, 3, 3, 2], vec![2, 2, 3, 2, 2]]),
                (12, vec![vec![3, 3, 3, 3], vec![2, 2, 2, 2, 2, 2]]),
            ]
        );
    }

    #[test]
    fn the_steppers_move_two_beats_a_minute_from_ninety_six() {
        let limits = crate::model::LimitsView::default();
        assert_eq!(
            (limits.click_tempo_step, limits.click_tempo_default),
            (2, 96)
        );
    }

    #[test]
    fn the_click_tables_cross_the_bridge() {
        crate::domain::types::assert_round_trips(click_bars());
        crate::domain::types::assert_round_trips(click_tempo_bands());
    }

    #[test]
    fn the_default_metre_is_common_time() {
        assert_eq!(Metre::default(), metre(4, 4));
    }
}

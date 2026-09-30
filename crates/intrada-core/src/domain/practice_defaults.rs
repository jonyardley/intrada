//! The repetition target and metronome start a session item begins with,
//! set once by the musician and stored on the device
//! (`specs/practice-defaults.md`).

use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{AppEffect, Effect, Event};
use crate::domain::Metre;
use crate::model::Model;
use crate::validation;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ClickStart {
    #[default]
    EveryBeat,
    TwoAndFour,
}

impl ClickStart {
    /// Bitmask over `metre.beats`, LSB = beat 1. 2 and 4 only where the click
    /// sheet offers it, an ungrouped bar of four beats or more.
    pub fn sounding(self, metre: &Metre) -> u16 {
        let every_beat = (1u16 << metre.beats) - 1;
        match self {
            ClickStart::TwoAndFour if metre.groups.is_none() && metre.beats >= 4 => 0b1010,
            _ => every_beat,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeDefaults {
    pub rep_target: u8,
    pub click: ClickStart,
}

impl Default for PracticeDefaults {
    fn default() -> Self {
        PracticeDefaults {
            rep_target: validation::DEFAULT_REP_TARGET,
            click: ClickStart::EveryBeat,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PracticeDefaultsEvent {
    Save(PracticeDefaults),
    /// The shell replaying the stored blob at launch: clamped, never re-saved.
    Loaded(PracticeDefaults),
}

pub fn handle_practice_defaults_event(
    event: PracticeDefaultsEvent,
    model: &mut Model,
) -> Command<Effect, Event> {
    match event {
        PracticeDefaultsEvent::Save(defaults) => {
            if let Err(e) = validation::validate_rep_target(&Some(defaults.rep_target)) {
                model.raise_error(e.to_string());
                return crux_core::render::render();
            }
            model.practice_defaults = defaults;
            model.clear_error();
            Command::all([
                Command::notify_shell(AppEffect::SavePracticeDefaults(defaults)).into(),
                crux_core::render::render(),
            ])
        }
        PracticeDefaultsEvent::Loaded(defaults) => {
            model.practice_defaults = PracticeDefaults {
                rep_target: defaults
                    .rep_target
                    .clamp(validation::MIN_REP_TARGET, validation::MAX_REP_TARGET),
                ..defaults
            };
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Intrada;
    use crate::domain::types::assert_round_trips;
    use crux_core::App;

    fn fixture() -> PracticeDefaults {
        PracticeDefaults {
            rep_target: 5,
            click: ClickStart::TwoAndFour,
        }
    }

    fn send(model: &mut Model, event: PracticeDefaultsEvent) -> Command<Effect, Event> {
        Intrada.update(Event::PracticeDefaults(event), model)
    }

    fn emits_save(cmd: &mut Command<Effect, Event>) -> Option<PracticeDefaults> {
        cmd.effects().find_map(|e| match e {
            Effect::App(req) => match &req.operation {
                AppEffect::SavePracticeDefaults(d) => Some(*d),
                _ => None,
            },
            _ => None,
        })
    }

    fn metre(beats: u8, unit: u8, groups: Option<Vec<u8>>) -> Metre {
        Metre {
            beats,
            unit,
            groups,
        }
    }

    // ── Starting beats ──

    #[test]
    fn two_and_four_falls_back_to_every_beat_where_the_bar_cannot_hold_it() {
        let cases: &[(Metre, u16, u16)] = &[
            (metre(4, 4, None), 0b1111, 0b1010),
            (metre(5, 4, None), 0b11111, 0b1010),
            (metre(12, 8, None), 0b1111_1111_1111, 0b1010),
            (metre(3, 4, None), 0b111, 0b111),
            (metre(2, 4, None), 0b11, 0b11),
            (metre(2, 2, None), 0b11, 0b11),
            (metre(6, 8, Some(vec![3, 3])), 0b111111, 0b111111),
            (metre(7, 8, Some(vec![2, 2, 3])), 0b1111111, 0b1111111),
            (metre(4, 4, Some(vec![2, 2])), 0b1111, 0b1111),
        ];
        for (m, every, two_and_four) in cases {
            assert_eq!(
                ClickStart::EveryBeat.sounding(m),
                *every,
                "every beat in {m:?}"
            );
            assert_eq!(
                ClickStart::TwoAndFour.sounding(m),
                *two_and_four,
                "2 and 4 in {m:?}"
            );
        }
    }

    // ── Events ──

    #[test]
    fn save_updates_the_model_and_emits_the_save_effect() {
        let mut model = Model::default();
        let mut cmd = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
        assert_eq!(model.practice_defaults, fixture());
        assert_eq!(emits_save(&mut cmd), Some(fixture()));
    }

    #[test]
    fn save_accepts_both_ends_of_the_range() {
        for target in [validation::MIN_REP_TARGET, validation::MAX_REP_TARGET] {
            let mut model = Model::default();
            let defaults = PracticeDefaults {
                rep_target: target,
                ..fixture()
            };
            let mut cmd = send(&mut model, PracticeDefaultsEvent::Save(defaults));
            assert_eq!(model.practice_defaults, defaults);
            assert_eq!(emits_save(&mut cmd), Some(defaults));
        }
    }

    #[test]
    fn save_outside_the_range_keeps_the_last_defaults_and_writes_nothing() {
        for target in [
            validation::MIN_REP_TARGET - 1,
            validation::MAX_REP_TARGET + 1,
        ] {
            let mut model = Model::default();
            let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
            let mut cmd = send(
                &mut model,
                PracticeDefaultsEvent::Save(PracticeDefaults {
                    rep_target: target,
                    click: ClickStart::EveryBeat,
                }),
            );
            assert_eq!(model.practice_defaults, fixture(), "target {target}");
            assert!(model.last_error.is_some(), "target {target}");
            assert_eq!(emits_save(&mut cmd), None, "target {target}");
        }
    }

    /// The save effect is fire-and-forget, so no confirmation clears the
    /// banner: an accepted save must clear it itself (`Store+Feedback.swift`).
    #[test]
    fn an_accepted_save_clears_the_error_a_refused_one_left() {
        let mut model = Model::default();
        let _ = send(
            &mut model,
            PracticeDefaultsEvent::Save(PracticeDefaults {
                rep_target: 0,
                ..fixture()
            }),
        );
        assert!(model.last_error.is_some());
        let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
        assert_eq!(model.last_error, None);
    }

    #[test]
    fn loaded_sets_the_model_without_re_saving() {
        let mut model = Model::default();
        let mut cmd = send(&mut model, PracticeDefaultsEvent::Loaded(fixture()));
        assert_eq!(model.practice_defaults, fixture());
        assert_eq!(
            emits_save(&mut cmd),
            None,
            "restore never re-writes the blob"
        );
    }

    #[test]
    fn loaded_clamps_a_target_the_range_has_since_left_behind() {
        for (stored, expected) in [
            (0, validation::MIN_REP_TARGET),
            (200, validation::MAX_REP_TARGET),
        ] {
            let mut model = Model::default();
            let _ = send(
                &mut model,
                PracticeDefaultsEvent::Loaded(PracticeDefaults {
                    rep_target: stored,
                    ..fixture()
                }),
            );
            assert_eq!(model.practice_defaults.rep_target, expected);
            assert_eq!(model.practice_defaults.click, ClickStart::TwoAndFour);
        }
    }

    // ── View ──

    #[test]
    fn view_model_carries_the_defaults_and_the_limits_follow_them() {
        let mut model = Model::default();
        assert_eq!(
            Intrada.view(&model).limits.rep_target_default,
            validation::DEFAULT_REP_TARGET
        );
        let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
        let view = Intrada.view(&model);
        assert_eq!(view.practice_defaults, fixture());
        assert_eq!(view.limits.rep_target_default, 5);
    }

    // ── Wire ──

    const PINNED_DEFAULTS_HEX: &str = "0501000000";
    const PINNED_UNTOUCHED_DEFAULTS_HEX: &str = "0a00000000";

    /// Positional bincode: a new field breaks every stored blob (#1345).
    /// Bump `Store.practiceDefaultsKey`, then re-pin.
    #[test]
    fn practice_defaults_blob_wire_is_pinned() {
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
        for (defaults, pinned) in [
            (fixture(), PINNED_DEFAULTS_HEX),
            (PracticeDefaults::default(), PINNED_UNTOUCHED_DEFAULTS_HEX),
        ] {
            let mut bytes = Vec::new();
            BincodeFfiFormat::serialize(&mut bytes, &defaults).expect("serialize");
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(
                hex, pinned,
                "the practice defaults blob changed shape: bump Store.practiceDefaultsKey, then re-pin"
            );
            let back: PracticeDefaults =
                BincodeFfiFormat::deserialize(&bytes).expect("must decode on the FFI wire (#846)");
            assert_eq!(back, defaults);
        }
    }

    #[test]
    fn practice_defaults_events_round_trip_on_the_ffi_wire() {
        assert_round_trips(Event::PracticeDefaults(PracticeDefaultsEvent::Save(
            fixture(),
        )));
        assert_round_trips(Event::PracticeDefaults(PracticeDefaultsEvent::Loaded(
            PracticeDefaults::default(),
        )));
        assert_round_trips(AppEffect::SavePracticeDefaults(fixture()));
    }
}

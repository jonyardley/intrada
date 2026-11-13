//! The metronome start a session item begins with and the length a new
//! session starts at, set once by the musician and stored on the device
//! (#1915, #1736).

use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{AppEffect, Effect, Event};
use crate::domain::Metre;
use crate::model::Model;
use crate::validation;
use crux_core::bridge::{BincodeFfiFormat, FfiFormat};

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeDefaults {
    pub click: ClickStart,
    pub session_length_mins: Option<u16>,
}

impl PracticeDefaults {
    /// The blob is positional bincode, so a build reads only a blob of its
    /// own shape. The shell names its storage key by this number, so a shape
    /// change bumps it here and nowhere else (#2216). Version 1 is retired;
    /// version 2 is carried across once (#2791).
    pub const BLOB_VERSION: u32 = 3;
    pub const PREVIOUS_BLOB_VERSION: u32 = 2;

    /// Version 2 held a repetition target first, since removed (#2791).
    fn from_previous_blob(bytes: &[u8]) -> Option<Self> {
        #[derive(Deserialize)]
        struct Version2 {
            _rep_target: u8,
            click: ClickStart,
            session_length_mins: Option<u16>,
        }
        let old: Version2 = BincodeFfiFormat::deserialize(bytes).ok()?;
        Some(Self {
            click: old.click,
            session_length_mins: old.session_length_mins,
        })
    }

    fn clamped(self) -> Self {
        Self {
            session_length_mins: self
                .session_length_mins
                .map(validation::clamp_session_length),
            ..self
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
    /// The version 2 blob's bytes, sent once when no version 3 blob exists:
    /// clamped and saved in the new shape.
    Carried(Vec<u8>),
}

pub fn handle_practice_defaults_event(
    event: PracticeDefaultsEvent,
    model: &mut Model,
) -> Command<Effect, Event> {
    match event {
        PracticeDefaultsEvent::Save(defaults) => {
            if let Err(e) = validation::validate_session_length(&defaults.session_length_mins) {
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
            model.practice_defaults = defaults.clamped();
            crux_core::render::render()
        }
        PracticeDefaultsEvent::Carried(bytes) => {
            let Some(carried) = PracticeDefaults::from_previous_blob(&bytes) else {
                return crux_core::render::render();
            };
            let defaults = carried.clamped();
            model.practice_defaults = defaults;
            Command::all([
                Command::notify_shell(AppEffect::SavePracticeDefaults(defaults)).into(),
                crux_core::render::render(),
            ])
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
            click: ClickStart::TwoAndFour,
            session_length_mins: Some(25),
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

    fn bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
            .collect()
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

    /// The save effect is fire-and-forget, so no confirmation clears the
    /// banner: an accepted save must clear it itself (`Store+Feedback.swift`).
    #[test]
    fn an_accepted_save_clears_the_error_a_refused_one_left() {
        let mut model = Model::default();
        let _ = send(
            &mut model,
            PracticeDefaultsEvent::Save(PracticeDefaults {
                session_length_mins: Some(5),
                ..fixture()
            }),
        );
        assert!(model.last_error.is_some());
        let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
        assert_eq!(model.last_error, None);
    }

    #[test]
    fn a_session_length_off_the_stepper_refuses_the_whole_save() {
        for length in [5, 32, 125] {
            let mut model = Model::default();
            let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
            let mut cmd = send(
                &mut model,
                PracticeDefaultsEvent::Save(PracticeDefaults {
                    click: ClickStart::EveryBeat,
                    session_length_mins: Some(length),
                }),
            );
            assert_eq!(model.practice_defaults, fixture(), "length {length}");
            assert!(model.last_error.is_some(), "length {length}");
            assert_eq!(emits_save(&mut cmd), None, "length {length}");
        }
    }

    #[test]
    fn save_can_switch_the_session_length_off() {
        let mut model = Model::default();
        let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
        let off = PracticeDefaults {
            session_length_mins: None,
            ..fixture()
        };
        let mut cmd = send(&mut model, PracticeDefaultsEvent::Save(off));
        assert_eq!(model.practice_defaults.session_length_mins, None);
        assert_eq!(emits_save(&mut cmd), Some(off));
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
    fn loaded_brings_a_stored_session_length_back_onto_the_stepper() {
        for (stored, expected) in [
            (Some(5), Some(10)),
            (Some(32), Some(30)),
            (Some(300), Some(120)),
            (None, None),
        ] {
            let mut model = Model::default();
            let _ = send(
                &mut model,
                PracticeDefaultsEvent::Loaded(PracticeDefaults {
                    session_length_mins: stored,
                    ..fixture()
                }),
            );
            assert_eq!(
                model.practice_defaults.session_length_mins, expected,
                "stored {stored:?}"
            );
            assert_eq!(model.practice_defaults.click, ClickStart::TwoAndFour);
        }
    }

    // ── Carrying version 2 across (#2791) ──

    /// The two version 2 blobs pinned before #2791: what devices hold.
    #[test]
    fn a_version_2_blob_keeps_its_click_and_length_and_saves_them_as_version_3() {
        for (hex, expected) in [
            ("0501000000011900", fixture()),
            ("0a0000000000", PracticeDefaults::default()),
            (
                "0301000000010500",
                PracticeDefaults {
                    session_length_mins: Some(10),
                    ..fixture()
                },
            ),
        ] {
            let mut model = Model::default();
            let mut cmd = send(&mut model, PracticeDefaultsEvent::Carried(bytes(hex)));
            assert_eq!(model.practice_defaults, expected, "{hex}");
            assert_eq!(emits_save(&mut cmd), Some(expected), "{hex}");
        }
    }

    #[test]
    fn bytes_that_are_not_a_version_2_blob_change_and_write_nothing() {
        for hex in ["", "05", "0509000000"] {
            let mut model = Model::default();
            let mut cmd = send(&mut model, PracticeDefaultsEvent::Carried(bytes(hex)));
            assert_eq!(
                model.practice_defaults,
                PracticeDefaults::default(),
                "{hex}"
            );
            assert_eq!(model.last_error, None, "{hex}");
            assert_eq!(emits_save(&mut cmd), None, "{hex}");
        }
    }

    // ── View ──

    #[test]
    fn view_model_carries_the_defaults() {
        let mut model = Model::default();
        let _ = send(&mut model, PracticeDefaultsEvent::Save(fixture()));
        assert_eq!(Intrada.view(&model).practice_defaults, fixture());
    }

    // ── Wire ──

    const PINNED_DEFAULTS_HEX: &str = "01000000011900";
    const PINNED_UNTOUCHED_DEFAULTS_HEX: &str = "0000000000";
    const PINNED_BLOB_VERSION: u32 = 3;

    /// Positional bincode: a new field breaks every stored blob (#1345). The
    /// shell's key follows `BLOB_VERSION` (#2216); the test cannot tell a
    /// re-pin with a bump from one without, so the failure message is the protocol.
    #[test]
    fn practice_defaults_blob_wire_is_pinned() {
        for (defaults, pinned) in [
            (fixture(), PINNED_DEFAULTS_HEX),
            (PracticeDefaults::default(), PINNED_UNTOUCHED_DEFAULTS_HEX),
        ] {
            let mut bytes = Vec::new();
            BincodeFfiFormat::serialize(&mut bytes, &defaults).expect("serialize");
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(
                (PracticeDefaults::BLOB_VERSION, hex.as_str()),
                (PINNED_BLOB_VERSION, pinned),
                "the practice defaults blob changed shape: bump PracticeDefaults::BLOB_VERSION and PINNED_BLOB_VERSION, then re-pin the hex. Never only the hex."
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
        assert_round_trips(Event::PracticeDefaults(PracticeDefaultsEvent::Carried(
            bytes("0501000000011900"),
        )));
        assert_round_trips(AppEffect::SavePracticeDefaults(fixture()));
    }
}

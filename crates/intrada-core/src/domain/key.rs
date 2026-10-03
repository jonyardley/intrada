use serde::{Deserialize, Serialize};

use super::item::Modality;

const CIRCLE_MAJOR: [&str; 12] = [
    "C", "G", "D", "A", "E", "B", "Gb", "Db", "Ab", "Eb", "Bb", "F",
];
const CIRCLE_MINOR: [&str; 12] = [
    "A", "E", "B", "F#", "C#", "G#", "Eb", "Bb", "F", "C", "G", "D",
];

/// A stored key on the circle of fifths (#2074): `ring` counts clockwise from
/// C major at 12 o'clock; `spelling` is the tonic the picker writes back.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct KeyWheelSelection {
    pub ring: u8,
    pub modality: Modality,
    pub spelling: String,
}

/// A legacy freeform key ("F# major") is parsed so it still lights its wedge
/// and heals into tonic plus modality on the next save (#2074).
pub fn wheel_selection(key: &str, modality: Option<Modality>) -> Option<KeyWheelSelection> {
    modality
        .and_then(|mode| ring_for(key, mode))
        .or_else(|| parse_freeform(key))
}

/// One spoke of the picker's wheel: the circle's default spelling and, on the
/// three enharmonic spokes, the other one a second tap flips to (#2226).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyWedge {
    pub ring: u8,
    pub modality: Modality,
    pub primary: &'static str,
    pub alt: Option<&'static str>,
}

/// The twelve major spokes clockwise from 12 o'clock, then the twelve minor.
pub fn wheel() -> Vec<KeyWedge> {
    [
        (Modality::Major, &CIRCLE_MAJOR),
        (Modality::Minor, &CIRCLE_MINOR),
    ]
    .into_iter()
    .flat_map(|(modality, circle)| {
        circle
            .iter()
            .enumerate()
            .map(move |(ring, primary)| KeyWedge {
                ring: ring as u8,
                modality,
                primary,
                alt: enharmonic_alt(ring, modality),
            })
    })
    .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyTap {
    pub tonic: String,
    pub modality: Modality,
    pub flipped: bool,
}

/// Tapping the selected enharmonic spoke flips its spelling; any other tap
/// selects the spoke's default. `None` for a ring off the wheel.
pub fn next_on_tap(
    current_key: &str,
    current_modality: Option<Modality>,
    ring: u8,
    mode: Modality,
) -> Option<KeyTap> {
    let ring = usize::from(ring);
    let primary = match mode {
        Modality::Major => CIRCLE_MAJOR.get(ring)?,
        Modality::Minor => CIRCLE_MINOR.get(ring)?,
    };
    let flip_to = wheel_selection(current_key, current_modality)
        .filter(|sel| usize::from(sel.ring) == ring && sel.modality == mode)
        .and_then(|sel| {
            enharmonic_alt(ring, mode).map(|alt| {
                if sel.spelling == *primary {
                    alt
                } else {
                    primary
                }
            })
        });
    Some(KeyTap {
        tonic: flip_to.unwrap_or(primary).to_string(),
        modality: mode,
        flipped: flip_to.is_some(),
    })
}

fn enharmonic_alt(ring: usize, mode: Modality) -> Option<&'static str> {
    match (mode, ring) {
        (Modality::Major, 5) => Some("Cb"),
        (Modality::Major, 6) => Some("F#"),
        (Modality::Major, 7) => Some("C#"),
        (Modality::Minor, 5) => Some("Ab"),
        (Modality::Minor, 6) => Some("D#"),
        (Modality::Minor, 7) => Some("A#"),
        _ => None,
    }
}

fn ring_for(tonic: &str, mode: Modality) -> Option<KeyWheelSelection> {
    let spelling = normalise_tonic(tonic)?;
    let circle = match mode {
        Modality::Major => &CIRCLE_MAJOR,
        Modality::Minor => &CIRCLE_MINOR,
    };
    (0..12)
        .find(|&ring| circle[ring] == spelling || enharmonic_alt(ring, mode) == Some(&spelling))
        .map(|ring| KeyWheelSelection {
            ring: ring as u8,
            modality: mode,
            spelling,
        })
}

fn parse_freeform(raw: &str) -> Option<KeyWheelSelection> {
    let ascii = raw.trim().replace('\u{266F}', "#").replace('\u{266D}', "b");
    let lower = ascii.to_ascii_lowercase();
    let (mode, tonic) = match lower.strip_suffix("minor") {
        Some(tonic) => (Modality::Minor, tonic),
        None => (Modality::Major, lower.strip_suffix("major")?),
    };
    ring_for(&ascii[..tonic.len()], mode)
}

fn normalise_tonic(raw: &str) -> Option<String> {
    let mut chars = raw.trim().chars();
    let letter = chars.next()?.to_ascii_uppercase();
    if !('A'..='G').contains(&letter) {
        return None;
    }
    let accidental = match (chars.next(), chars.next()) {
        (None, _) => "",
        (Some('#'), None) => "#",
        (Some('b' | 'B'), None) => "b",
        _ => return None,
    };
    Some(format!("{letter}{accidental}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sel(ring: u8, modality: Modality, spelling: &str) -> Option<KeyWheelSelection> {
        Some(KeyWheelSelection {
            ring,
            modality,
            spelling: spelling.to_string(),
        })
    }

    #[test]
    fn stored_keys_land_on_their_wedge() {
        use Modality::{Major, Minor};
        let cases: &[(&str, Option<Modality>, Option<KeyWheelSelection>)] = &[
            ("C", Some(Major), sel(0, Major, "C")),
            ("A", Some(Minor), sel(0, Minor, "A")),
            ("F#", Some(Major), sel(6, Major, "F#")),
            ("Gb", Some(Major), sel(6, Major, "Gb")),
            ("Eb", Some(Minor), sel(6, Minor, "Eb")),
            ("Cb", Some(Major), sel(5, Major, "Cb")),
            ("Ab", Some(Minor), sel(5, Minor, "Ab")),
            ("f#", Some(Major), sel(6, Major, "F#")),
            (" db ", Some(Major), sel(7, Major, "Db")),
            ("F# major", None, sel(6, Major, "F#")),
            ("A minor", None, sel(0, Minor, "A")),
            ("c MAJOR", None, sel(0, Major, "C")),
            ("f# MINOR", None, sel(3, Minor, "F#")),
            ("C\u{266F} minor", None, sel(4, Minor, "C#")),
            ("B\u{266D} major", None, sel(10, Major, "Bb")),
            ("Gb major", None, sel(6, Major, "Gb")),
            ("F# major", Some(Minor), sel(6, Major, "F#")),
            ("", None, None),
            ("   ", None, None),
            ("", Some(Major), None),
            ("Lydian", None, None),
            ("H major", None, None),
            ("C## major", None, None),
            ("C dorian", None, None),
            ("F#", None, None),
            ("E#", Some(Major), None),
            ("\u{130} major", None, None),
        ];
        for (key, modality, expected) in cases {
            assert_eq!(
                &wheel_selection(key, *modality),
                expected,
                "{key:?} with {modality:?}"
            );
        }
    }

    fn tap(key: &str, modality: Option<Modality>, ring: u8, mode: Modality) -> (String, bool) {
        let t = next_on_tap(key, modality, ring, mode).expect("ring on the wheel");
        assert_eq!(t.modality, mode);
        (t.tonic, t.flipped)
    }

    #[test]
    fn a_second_tap_on_an_enharmonic_spoke_flips_its_spelling() {
        use Modality::{Major, Minor};
        let cases: &[(u8, Modality, &str, &str)] = &[
            (5, Major, "B", "Cb"),
            (6, Major, "Gb", "F#"),
            (7, Major, "Db", "C#"),
            (5, Minor, "G#", "Ab"),
            (6, Minor, "Eb", "D#"),
            (7, Minor, "Bb", "A#"),
        ];
        for &(ring, mode, primary, alt) in cases {
            assert_eq!(
                tap("", None, ring, mode),
                (primary.into(), false),
                "{ring} {mode:?}"
            );
            assert_eq!(
                tap(primary, Some(mode), ring, mode),
                (alt.into(), true),
                "{ring} {mode:?}"
            );
            assert_eq!(
                tap(alt, Some(mode), ring, mode),
                (primary.into(), true),
                "{ring} {mode:?}"
            );
        }
    }

    #[test]
    fn other_taps_select_the_spokes_default() {
        use Modality::{Major, Minor};
        assert_eq!(tap("C", Some(Major), 0, Major), ("C".into(), false));
        assert_eq!(tap("C", Some(Major), 1, Major), ("G".into(), false));
        assert_eq!(tap("F#", Some(Major), 6, Minor), ("Eb".into(), false));
        assert_eq!(tap("D#", Some(Minor), 7, Minor), ("Bb".into(), false));
        assert_eq!(tap("F# major", None, 6, Major), ("Gb".into(), true));
        assert_eq!(next_on_tap("C", Some(Major), 12, Major), None);
        assert_eq!(next_on_tap("A", Some(Minor), 12, Minor), None);
    }

    #[test]
    fn the_wheel_lists_every_spoke_in_order_with_its_enharmonic_spellings() {
        let wedges = wheel();
        assert_eq!(wedges.len(), 24);
        for (i, w) in wedges.iter().enumerate() {
            let mode = if i < 12 {
                Modality::Major
            } else {
                Modality::Minor
            };
            assert_eq!((w.ring as usize, w.modality), (i % 12, mode));
        }
        let alts: Vec<_> = wedges.iter().filter_map(|w| w.alt).collect();
        assert_eq!(alts, ["Cb", "F#", "C#", "Ab", "D#", "A#"]);
        assert_eq!(wedges[6].primary, "Gb");
        assert_eq!(wedges[18].primary, "Eb");
    }

    #[test]
    fn every_wedge_spelling_round_trips() {
        for mode in [Modality::Major, Modality::Minor] {
            let circle = match mode {
                Modality::Major => CIRCLE_MAJOR,
                Modality::Minor => CIRCLE_MINOR,
            };
            for (ring, tonic) in circle.iter().enumerate() {
                assert_eq!(
                    wheel_selection(tonic, Some(mode)),
                    sel(ring as u8, mode, tonic)
                );
                if let Some(alt) = enharmonic_alt(ring, mode) {
                    assert_eq!(wheel_selection(alt, Some(mode)), sel(ring as u8, mode, alt));
                }
            }
        }
    }
}

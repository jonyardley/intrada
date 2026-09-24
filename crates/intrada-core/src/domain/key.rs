use serde::{Deserialize, Serialize};

use super::item::Modality;

const CIRCLE_MAJOR: [&str; 12] = [
    "C", "G", "D", "A", "E", "B", "Gb", "Db", "Ab", "Eb", "Bb", "F",
];
const CIRCLE_MINOR: [&str; 12] = [
    "A", "E", "B", "F#", "C#", "G#", "Eb", "Bb", "F", "C", "G", "D",
];

/// A stored key placed on the circle of fifths: `ring` counts clockwise from
/// C major (A minor) at 12 o'clock, and `spelling` is the tonic as the key
/// picker writes it back.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct KeyWheelSelection {
    pub ring: u8,
    pub modality: Modality,
    pub spelling: String,
}

/// Structured keys resolve against their modality; a legacy freeform value
/// ("F# major", "c minor") is parsed so it still lights its wedge and heals
/// into tonic plus modality on the next save.
pub fn wheel_selection(key: &str, modality: Option<Modality>) -> Option<KeyWheelSelection> {
    modality
        .and_then(|mode| ring_for(key, mode))
        .or_else(|| parse_freeform(key))
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

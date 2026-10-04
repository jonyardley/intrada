use serde::{Deserialize, Serialize};

use super::item::Modality;

/// A key as one value everywhere (#2106): the written key, a chart's key, an
/// item's keys and a play's. The spelling is the letter and accidental, so C
/// sharp stays C sharp; counting and comparing go through `pitch_class`.
/// `mode: None` is a key written with no mode, and nothing guesses major.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Key {
    pub letter: Letter,
    pub accidental: Accidental,
    pub mode: Option<Modality>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
}

impl Letter {
    fn from_char(c: char) -> Option<Self> {
        Some(match c.to_ascii_uppercase() {
            'C' => Self::C,
            'D' => Self::D,
            'E' => Self::E,
            'F' => Self::F,
            'G' => Self::G,
            'A' => Self::A,
            'B' => Self::B,
            _ => return None,
        })
    }

    fn as_char(self) -> char {
        match self {
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::F => 'F',
            Self::G => 'G',
            Self::A => 'A',
            Self::B => 'B',
        }
    }

    fn natural_pitch(self) -> u8 {
        match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        }
    }
}

impl Key {
    pub const C_MAJOR: Key = Key {
        letter: Letter::C,
        accidental: Accidental::Natural,
        mode: Some(Modality::Major),
    };

    /// What musicians type: "Eb", "E flat major", "F-sharp minor", "C♯". A
    /// chord symbol ("F#m") and modes beyond major and minor (#830) are refused.
    pub fn parse(raw: &str) -> Option<Key> {
        let ascii = raw.replace('\u{266F}', "#").replace('\u{266D}', "b");
        let lower = ascii.trim().to_ascii_lowercase();
        let (rest, mode) = strip_mode(&lower);
        let mut chars = rest.trim_end().chars();
        let letter = Letter::from_char(chars.next()?)?;
        let sign = chars.as_str().trim_start();
        let sign = sign.strip_prefix('-').map_or(sign, str::trim_start);
        let accidental = match sign {
            "" => Accidental::Natural,
            "#" | "sharp" => Accidental::Sharp,
            "b" | "flat" => Accidental::Flat,
            _ => return None,
        };
        Some(Key {
            letter,
            accidental,
            mode,
        })
    }

    pub fn pitch_class(&self) -> u8 {
        let natural = self.letter.natural_pitch();
        match self.accidental {
            Accidental::Natural => natural,
            Accidental::Sharp => (natural + 1) % 12,
            Accidental::Flat => (natural + 11) % 12,
        }
    }

    /// Both spellings are one key: E flat major and D sharp major agree.
    pub fn same_key(&self, other: &Key) -> bool {
        self.pitch_class() == other.pitch_class() && self.mode == other.mode
    }

    /// The tonic as stored and as the wheel names it: "C", "Eb", "F#".
    pub fn spelling(&self) -> String {
        let sign = match self.accidental {
            Accidental::Natural => "",
            Accidental::Sharp => "#",
            Accidental::Flat => "b",
        };
        format!("{}{sign}", self.letter.as_char())
    }

    /// As the musician reads it: "E♭ major", "F♯ minor", "C".
    pub fn label(&self) -> String {
        let sign = match self.accidental {
            Accidental::Natural => "",
            Accidental::Sharp => "\u{266F}",
            Accidental::Flat => "\u{266D}",
        };
        let tonic = format!("{}{sign}", self.letter.as_char());
        match self.mode {
            Some(Modality::Major) => format!("{tonic} major"),
            Some(Modality::Minor) => format!("{tonic} minor"),
            None => tonic,
        }
    }
}

/// The trailing mode word, if any, with what comes before it.
fn strip_mode(lower: &str) -> (&str, Option<Modality>) {
    if let Some(rest) = lower.strip_suffix("major") {
        (rest, Some(Modality::Major))
    } else if let Some(rest) = lower.strip_suffix("minor") {
        (rest, Some(Modality::Minor))
    } else {
        (lower, None)
    }
}

/// The item's `key` and `modality` columns into a key (#2106). A mode word in
/// the text, written before the two were split (#2074), wins over the column.
/// Text the core cannot read is `None`; the shell keeps it in the column.
pub fn key_from_stored(text: Option<&str>, modality: Option<Modality>) -> Option<Key> {
    let key = Key::parse(text?)?;
    Some(Key {
        mode: key.mode.or(modality),
        ..key
    })
}

pub fn key_to_stored(key: &Key) -> (String, Option<Modality>) {
    (key.spelling(), key.mode)
}

// ── The wheel ─────────────────────────────────────────────────────────

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

/// The spoke a key lights. A key with no mode, or a spelling off the wheel
/// (E sharp major), lights none.
pub fn wheel_selection(key: &Key) -> Option<KeyWheelSelection> {
    ring_for(&key.spelling(), key.mode?)
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
    pub key: Key,
    pub flipped: bool,
}

/// Tapping the selected enharmonic spoke flips its spelling; any other tap
/// selects the spoke's default. `None` for a ring off the wheel.
pub fn next_on_tap(current: Option<&Key>, ring: u8, mode: Modality) -> Option<KeyTap> {
    let ring = usize::from(ring);
    let primary = match mode {
        Modality::Major => CIRCLE_MAJOR.get(ring)?,
        Modality::Minor => CIRCLE_MINOR.get(ring)?,
    };
    let flip_to = current
        .and_then(wheel_selection)
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
    let tonic = Key::parse(flip_to.unwrap_or(primary))?;
    Some(KeyTap {
        key: Key {
            mode: Some(mode),
            ..tonic
        },
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

fn ring_for(spelling: &str, mode: Modality) -> Option<KeyWheelSelection> {
    let circle = match mode {
        Modality::Major => &CIRCLE_MAJOR,
        Modality::Minor => &CIRCLE_MINOR,
    };
    (0..12)
        .find(|&ring| circle[ring] == spelling || enharmonic_alt(ring, mode) == Some(spelling))
        .map(|ring| KeyWheelSelection {
            ring: ring as u8,
            modality: mode,
            spelling: spelling.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Accidental::{Flat, Natural, Sharp};
    use Modality::{Major, Minor};

    fn key(letter: Letter, accidental: Accidental, mode: Option<Modality>) -> Option<Key> {
        Some(Key {
            letter,
            accidental,
            mode,
        })
    }

    /// Everything `is_key_label` accepted as a step, every stored key text the
    /// wheel and the freeform field wrote, and what musicians type instead.
    #[test]
    fn parse_reads_what_musicians_type() {
        let cases: &[(&str, Option<Key>)] = &[
            ("C", key(Letter::C, Natural, None)),
            ("F#", key(Letter::F, Sharp, None)),
            ("Bb", key(Letter::B, Flat, None)),
            ("F\u{266F}", key(Letter::F, Sharp, None)),
            ("B\u{266D}", key(Letter::B, Flat, None)),
            ("c", key(Letter::C, Natural, None)),
            (" D ", key(Letter::D, Natural, None)),
            ("b", key(Letter::B, Natural, None)),
            ("bb", key(Letter::B, Flat, None)),
            ("C major", key(Letter::C, Natural, Some(Major))),
            ("f# minor", key(Letter::F, Sharp, Some(Minor))),
            ("D\u{266F} major", key(Letter::D, Sharp, Some(Major))),
            ("G# major", key(Letter::G, Sharp, Some(Major))),
            ("Db minor", key(Letter::D, Flat, Some(Minor))),
            ("E flat major", key(Letter::E, Flat, Some(Major))),
            ("F sharp minor", key(Letter::F, Sharp, Some(Minor))),
            ("A flat", key(Letter::A, Flat, None)),
            ("c sharp", key(Letter::C, Sharp, None)),
            ("E-flat major", key(Letter::E, Flat, Some(Major))),
            ("F-sharp minor", key(Letter::F, Sharp, Some(Minor))),
            ("c MAJOR", key(Letter::C, Natural, Some(Major))),
            ("Gb major", key(Letter::G, Flat, Some(Major))),
            ("E#", key(Letter::E, Sharp, None)),
            ("Root position", None),
            ("1st inversion", None),
            ("Hands together", None),
            ("Variation 1", None),
            ("Am", None),
            ("F#m", None),
            ("C dorian", None),
            ("G mixolydian", None),
            ("C/E", None),
            ("H", None),
            ("H major", None),
            ("C## major", None),
            ("", None),
            ("   ", None),
            ("major", None),
            ("Lydian", None),
            ("E flatten major", None),
            ("F sharpish minor", None),
            ("r\u{e9} mineur", None),
            ("\u{130} major", None),
        ];
        for (raw, expected) in cases {
            assert_eq!(&Key::parse(raw), expected, "{raw:?}");
        }
    }

    #[test]
    fn enharmonic_spellings_are_one_key_and_keep_their_spelling() {
        let pairs = [
            ("Eb major", "D# major"),
            ("F# major", "Gb major"),
            ("C# minor", "Db minor"),
            ("B", "Cb"),
            ("E#", "F"),
        ];
        for (a, b) in pairs {
            let (a, b) = (Key::parse(a).unwrap(), Key::parse(b).unwrap());
            assert!(a.same_key(&b), "{a:?} and {b:?}");
            assert_ne!(a.spelling(), b.spelling());
        }
        let e_flat_major = Key::parse("Eb major").unwrap();
        assert!(!e_flat_major.same_key(&Key::parse("Eb minor").unwrap()));
        assert!(!e_flat_major.same_key(&Key::parse("Eb").unwrap()));
        assert!(!e_flat_major.same_key(&Key::parse("E major").unwrap()));
    }

    #[test]
    fn a_key_reads_with_its_signs() {
        assert_eq!(Key::parse("Eb major").unwrap().label(), "E\u{266D} major");
        assert_eq!(Key::parse("f# minor").unwrap().label(), "F\u{266F} minor");
        assert_eq!(Key::parse("c").unwrap().label(), "C");
        assert_eq!(Key::parse("c sharp").unwrap().spelling(), "C#");
    }

    /// What the wheel and the freeform field left in the two columns.
    #[test]
    fn stored_keys_read_back() {
        let cases: &[(Option<&str>, Option<Modality>, Option<Key>)] = &[
            (Some("C"), Some(Major), key(Letter::C, Natural, Some(Major))),
            (Some("Eb"), Some(Minor), key(Letter::E, Flat, Some(Minor))),
            (Some("f#"), Some(Major), key(Letter::F, Sharp, Some(Major))),
            (Some(" db "), Some(Major), key(Letter::D, Flat, Some(Major))),
            (Some("F#"), None, key(Letter::F, Sharp, None)),
            (Some("F# major"), None, key(Letter::F, Sharp, Some(Major))),
            (
                Some("F# major"),
                Some(Minor),
                key(Letter::F, Sharp, Some(Major)),
            ),
            (
                Some("B\u{266D} major"),
                None,
                key(Letter::B, Flat, Some(Major)),
            ),
            (Some("C dorian"), None, None),
            (Some(""), Some(Major), None),
            (None, Some(Major), None),
            (None, None, None),
        ];
        for (text, modality, expected) in cases {
            assert_eq!(
                &key_from_stored(*text, *modality),
                expected,
                "{text:?} {modality:?}"
            );
        }
    }

    #[test]
    fn every_wheel_spoke_round_trips_through_its_columns() {
        for wedge in wheel() {
            for spelling in std::iter::once(wedge.primary).chain(wedge.alt) {
                let tapped = key_from_stored(Some(spelling), Some(wedge.modality)).unwrap();
                let (text, modality) = key_to_stored(&tapped);
                assert_eq!(text, spelling);
                assert_eq!(key_from_stored(Some(&text), modality), Some(tapped));
                let sel = wheel_selection(&tapped).expect("a spoke lights");
                assert_eq!((sel.ring, sel.modality), (wedge.ring, wedge.modality));
                assert_eq!(sel.spelling, spelling);
            }
        }
    }

    #[test]
    fn a_key_with_no_mode_or_off_the_wheel_lights_no_spoke() {
        assert_eq!(wheel_selection(&Key::parse("F#").unwrap()), None);
        assert_eq!(wheel_selection(&Key::parse("E# major").unwrap()), None);
    }

    fn tap(current: Option<&str>, ring: u8, mode: Modality) -> (String, bool) {
        let current = current.map(|c| Key::parse(c).unwrap());
        let t = next_on_tap(current.as_ref(), ring, mode).expect("ring on the wheel");
        assert_eq!(t.key.mode, Some(mode));
        (t.key.spelling(), t.flipped)
    }

    #[test]
    fn a_second_tap_on_an_enharmonic_spoke_flips_its_spelling() {
        let cases: &[(u8, Modality, &str, &str)] = &[
            (5, Major, "B", "Cb"),
            (6, Major, "Gb", "F#"),
            (7, Major, "Db", "C#"),
            (5, Minor, "G#", "Ab"),
            (6, Minor, "Eb", "D#"),
            (7, Minor, "Bb", "A#"),
        ];
        for &(ring, mode, primary, alt) in cases {
            let word = if mode == Major { "major" } else { "minor" };
            assert_eq!(tap(None, ring, mode), (primary.into(), false));
            assert_eq!(
                tap(Some(&format!("{primary} {word}")), ring, mode),
                (alt.into(), true),
                "{ring} {mode:?}"
            );
            assert_eq!(
                tap(Some(&format!("{alt} {word}")), ring, mode),
                (primary.into(), true),
                "{ring} {mode:?}"
            );
        }
    }

    #[test]
    fn other_taps_select_the_spokes_default() {
        assert_eq!(tap(Some("C major"), 0, Major), ("C".into(), false));
        assert_eq!(tap(Some("C major"), 1, Major), ("G".into(), false));
        assert_eq!(tap(Some("F# major"), 6, Minor), ("Eb".into(), false));
        assert_eq!(tap(Some("D# minor"), 7, Minor), ("Bb".into(), false));
        assert_eq!(tap(Some("F#"), 6, Major), ("Gb".into(), false));
        assert_eq!(next_on_tap(None, 12, Major), None);
        assert_eq!(next_on_tap(None, 12, Minor), None);
    }

    #[test]
    fn the_wheel_lists_every_spoke_in_order_with_its_enharmonic_spellings() {
        let wedges = wheel();
        assert_eq!(wedges.len(), 24);
        for (i, w) in wedges.iter().enumerate() {
            let mode = if i < 12 { Major } else { Minor };
            assert_eq!((w.ring as usize, w.modality), (i % 12, mode));
        }
        let alts: Vec<_> = wedges.iter().filter_map(|w| w.alt).collect();
        assert_eq!(alts, ["Cb", "F#", "C#", "Ab", "D#", "A#"]);
        assert_eq!(wedges[6].primary, "Gb");
        assert_eq!(wedges[18].primary, "Eb");
    }
}

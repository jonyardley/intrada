//! Chord-chart parsing and deterministic scaffold derivation — pure music
//! theory, no I/O, no model access. The theory island: a wrong note here is a
//! real bug, so every rule is unit-testable to the pitch class.
//!
//! Phase A derives in the chart's own key only; the twelve-key step ladder
//! (#1083) is Phase C. Vocabulary is deliberately capped at the common jazz
//! qualities (see [`ChordQuality`]); anything else falls back to the chord's
//! arpeggio and is flagged.

use serde::{Deserialize, Serialize};

use super::item::Modality;

// ── Pitch classes ─────────────────────────────────────────────────────────
// A pitch class is a semitone offset from C, 0..=11 (C=0, C#=1, … B=11),
// octave-agnostic. Kept as a bare `u8` on the wire — the assertion currency
// for the derivation goldens.

const OCTAVE: u8 = 12;

fn letter_pc(c: char) -> Option<u8> {
    match c.to_ascii_uppercase() {
        'C' => Some(0),
        'D' => Some(2),
        'E' => Some(4),
        'F' => Some(5),
        'G' => Some(7),
        'A' => Some(9),
        'B' => Some(11),
        _ => None,
    }
}

// ── Chord model (bridge-crossing via `Item.chord_chart`) ──────────────────

/// The common jazz vocabulary v1 covers with a chord-scale. `Other` is any
/// symbol we recognise a root for but not the quality — it derives an arpeggio
/// fallback, flagged in the preview, rather than guessing a wrong scale.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ChordQuality {
    Maj7,
    Dom7,
    Min7,
    Min7b5,
    Dim7,
    MinMaj7,
    Six,
    Min6,
    Alt,
    // Phase C vocabulary. Suspensions have no 3rd (the sus tone stands in for
    // shells/guide-tones); Aug/Dom7Sharp5 carry a ♯5.
    Sus4,
    Sus2,
    Aug,
    Dom7Sharp5,
    Other,
}

/// A parsed chord symbol. `raw` is retained verbatim so an edited/unusual token
/// round-trips losslessly for display even when the theory reduces it to a
/// coarser quality.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ChordSymbol {
    /// Root pitch class, 0..=11.
    pub root: u8,
    pub quality: ChordQuality,
    /// Trailing tensions (e.g. `["b9"]`) — retained but unused by Phase A theory.
    pub extensions: Vec<String>,
    /// Slash-bass pitch class, when the token carried `/X`.
    pub bass: Option<u8>,
    pub raw: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ChartChord {
    pub symbol: ChordSymbol,
    pub beats: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Bar {
    pub chords: Vec<ChartChord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ChartSection {
    pub label: Option<String>,
    pub bars: Vec<Bar>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ChordChart {
    pub key: String,
    pub modality: Modality,
    /// Beats per bar. v1 assumes 4/4; the field lets richer metres land later.
    pub metre: u8,
    pub sections: Vec<ChartSection>,
}

impl ChordChart {
    /// Every chord across every section/bar, in reading order.
    pub fn changes(&self) -> Vec<&ChordSymbol> {
        self.sections
            .iter()
            .flat_map(|s| s.bars.iter())
            .flat_map(|b| b.chords.iter())
            .map(|c| &c.symbol)
            .collect()
    }
}

/// A parse failure, naming the first offending token and its 1-based bar so the
/// shell can surface it against the exact spot (never a partial chart).
#[derive(Debug, Clone, PartialEq)]
pub struct ChartParseError {
    /// 1-based bar index across the whole chart; 0 when the chart is empty.
    pub bar: usize,
    pub token: String,
    pub message: String,
}

impl std::fmt::Display for ChartParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bar == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "Bar {}: {}", self.bar, self.message)
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────

const DEFAULT_METRE: u8 = 4;

/// Parse a text chart in the bar-and-pipe grammar we own:
/// `[Section]` labels on their own or leading a bar line; bars delimited by
/// `|`, chords within a bar by whitespace. Strict on root + recognised quality;
/// lenient on trailing tensions.
pub fn parse_chart(
    raw: &str,
    key: &str,
    modality: Modality,
) -> Result<ChordChart, ChartParseError> {
    let mut sections: Vec<ChartSection> = Vec::new();
    let mut bar_no: usize = 0;

    for line in raw.lines() {
        let mut line = line.trim();
        if line.is_empty() {
            continue;
        }

        // A leading [Label] opens a new section; bars may follow on the line.
        if let Some(rest) = line.strip_prefix('[') {
            if let Some(end) = rest.find(']') {
                let label = rest[..end].trim().to_string();
                sections.push(ChartSection {
                    label: (!label.is_empty()).then_some(label),
                    bars: Vec::new(),
                });
                line = rest[end + 1..].trim();
                if line.is_empty() {
                    continue;
                }
            }
        }

        if !line.contains('|') {
            continue; // non-bar prose line; ignore rather than fail
        }

        if sections.is_empty() {
            sections.push(ChartSection {
                label: None,
                bars: Vec::new(),
            });
        }

        for raw_bar in line.split('|') {
            let raw_bar = raw_bar.trim();
            if raw_bar.is_empty() {
                continue;
            }
            bar_no += 1;

            let tokens: Vec<&str> = raw_bar.split_whitespace().collect();
            let mut chords = Vec::with_capacity(tokens.len());
            for tok in &tokens {
                chords.push(ChartChord {
                    symbol: parse_chord(tok, bar_no)?,
                    beats: 0, // assigned below once the count is known
                });
            }
            assign_beats(&mut chords, DEFAULT_METRE);

            sections
                .last_mut()
                .expect("a section exists")
                .bars
                .push(Bar { chords });
        }
    }

    if bar_no == 0 {
        return Err(ChartParseError {
            bar: 0,
            token: String::new(),
            message: "Add at least one bar of changes.".to_string(),
        });
    }

    Ok(ChordChart {
        key: key.to_string(),
        modality,
        metre: DEFAULT_METRE,
        sections,
    })
}

/// Split `metre` beats across a bar's chords as evenly as possible; any
/// remainder lands on the earlier chords.
fn assign_beats(chords: &mut [ChartChord], metre: u8) {
    let n = chords.len() as u8;
    if n == 0 {
        return;
    }
    let base = (metre / n).max(1);
    let extra = metre.saturating_sub(base * n);
    for (i, chord) in chords.iter_mut().enumerate() {
        chord.beats = base + u8::from((i as u8) < extra);
    }
}

fn parse_chord(token: &str, bar_no: usize) -> Result<ChordSymbol, ChartParseError> {
    let raw = token.to_string();
    let err = |message: String| ChartParseError {
        bar: bar_no,
        token: raw.clone(),
        message,
    };

    // A `/` before a non-letter is a tension (`C6/9`), not a slash bass (`Dm7/G`).
    let (head, bass) = match token.split_once('/') {
        Some((h, b)) if b.starts_with(|c: char| letter_pc(c).is_some()) => (h, Some(b)),
        _ => (token, None),
    };

    let mut chars = head.chars();
    let first = chars
        .next()
        .ok_or_else(|| err("Empty chord.".to_string()))?;
    let mut root =
        letter_pc(first).ok_or_else(|| err(format!("\"{first}\" isn't a note name (A–G).")))?;

    // Root accidentals: the char(s) right after the root letter. No quality
    // token starts with `#`/`b`, so a leading accidental is unambiguous — this
    // is what lets `Bb7`/`Eb9`/`Ab13` keep their flat root (a `b`-then-digit is
    // a flat root here, never a tension; tensions like `b5` only follow a
    // quality and are parsed below).
    let mut rest = chars.as_str();
    loop {
        let mut it = rest.chars();
        match it.next() {
            Some('#') | Some('♯') => {
                root = (root + 1) % OCTAVE;
                rest = it.as_str();
            }
            Some('b') | Some('♭') => {
                root = (root + OCTAVE - 1) % OCTAVE;
                rest = it.as_str();
            }
            _ => break,
        }
    }

    let bass_pc = match bass {
        Some(b) => Some(parse_bass(b, bar_no, &raw)?),
        None => None,
    };

    let (quality, extensions) = parse_quality(rest);

    Ok(ChordSymbol {
        root,
        quality,
        extensions,
        bass: bass_pc,
        raw,
    })
}

fn parse_bass(b: &str, bar_no: usize, raw: &str) -> Result<u8, ChartParseError> {
    let mut chars = b.chars();
    let first = chars.next().ok_or(ChartParseError {
        bar: bar_no,
        token: raw.to_string(),
        message: "Slash chord is missing its bass note.".to_string(),
    })?;
    let mut pc = letter_pc(first).ok_or(ChartParseError {
        bar: bar_no,
        token: raw.to_string(),
        message: format!("\"{first}\" isn't a bass note (A–G)."),
    })?;
    for c in chars {
        match c {
            '#' | '♯' => pc = (pc + 1) % OCTAVE,
            'b' | '♭' => pc = (pc + OCTAVE - 1) % OCTAVE,
            _ => {}
        }
    }
    Ok(pc)
}

/// Recognise a quality suffix; anything unmatched becomes [`ChordQuality::Other`]
/// (arpeggio fallback). Longest patterns first so `maj7` beats `maj`/`m`.
fn parse_quality(rest: &str) -> (ChordQuality, Vec<String>) {
    let s = rest.trim();
    // (pattern, quality) in priority order.
    // Longest / most-specific prefixes first; `maj*` before any bare `m*` so a
    // major chord is never read as minor.
    const TABLE: &[(&str, ChordQuality)] = &[
        ("m(maj7)", ChordQuality::MinMaj7),
        ("mmaj7", ChordQuality::MinMaj7),
        ("minmaj7", ChordQuality::MinMaj7),
        ("m7b5", ChordQuality::Min7b5),
        ("min7b5", ChordQuality::Min7b5),
        ("ø7", ChordQuality::Min7b5),
        ("ø", ChordQuality::Min7b5),
        ("dim7", ChordQuality::Dim7),
        ("°7", ChordQuality::Dim7),
        ("o7", ChordQuality::Dim7),
        ("°", ChordQuality::Dim7),
        ("dim", ChordQuality::Dim7),
        ("maj7", ChordQuality::Maj7),
        ("maj9", ChordQuality::Maj7),
        ("maj13", ChordQuality::Maj7),
        ("major7", ChordQuality::Maj7),
        ("ma7", ChordQuality::Maj7),
        ("maj", ChordQuality::Maj7),
        ("Δ7", ChordQuality::Maj7),
        ("Δ", ChordQuality::Maj7),
        ("m6", ChordQuality::Min6),
        ("min6", ChordQuality::Min6),
        ("-6", ChordQuality::Min6),
        ("m7", ChordQuality::Min7),
        ("m9", ChordQuality::Min7),
        ("m11", ChordQuality::Min7),
        ("m13", ChordQuality::Min7),
        ("min7", ChordQuality::Min7),
        ("min", ChordQuality::Min7),
        ("-7", ChordQuality::Min7),
        // `sus2` before `sus`/`sus4`; a 7/9/11/13-bearing sus is the dominant sus.
        ("7sus4", ChordQuality::Sus4),
        ("7sus", ChordQuality::Sus4),
        ("9sus4", ChordQuality::Sus4),
        ("9sus", ChordQuality::Sus4),
        ("11sus4", ChordQuality::Sus4),
        ("11sus", ChordQuality::Sus4),
        ("13sus4", ChordQuality::Sus4),
        ("13sus", ChordQuality::Sus4),
        ("sus2", ChordQuality::Sus2),
        ("sus4", ChordQuality::Sus4),
        ("sus", ChordQuality::Sus4),
        // Dominant ♯5 (`+7`/`aug7`) before the bare augmented triad.
        ("aug7", ChordQuality::Dom7Sharp5),
        ("7aug", ChordQuality::Dom7Sharp5),
        ("7#5", ChordQuality::Dom7Sharp5),
        ("7♯5", ChordQuality::Dom7Sharp5),
        ("7+5", ChordQuality::Dom7Sharp5),
        ("9#5", ChordQuality::Dom7Sharp5),
        ("+7", ChordQuality::Dom7Sharp5),
        ("aug", ChordQuality::Aug),
        ("+", ChordQuality::Aug),
        ("alt", ChordQuality::Alt),
        ("7alt", ChordQuality::Alt),
        ("6", ChordQuality::Six),
        // A bare 7/9/11/13 (no other quality marker) is a dominant.
        ("7", ChordQuality::Dom7),
        ("9", ChordQuality::Dom7),
        ("11", ChordQuality::Dom7),
        ("13", ChordQuality::Dom7),
    ];

    // Case-insensitive match on ASCII; unicode quality glyphs compared as-is.
    let lower = s.to_lowercase();
    for (pat, quality) in TABLE {
        let pat_cmp = pat.to_lowercase();
        if let Some(tail) = lower.strip_prefix(&pat_cmp) {
            let ext = tail.trim();
            let extensions = if ext.is_empty() {
                vec![]
            } else {
                vec![ext.to_string()]
            };
            return (*quality, extensions);
        }
    }

    if s.is_empty() {
        // Bare root / major triad → treat as a major-7 sound for shells/scale.
        (ChordQuality::Maj7, vec![])
    } else if lower.starts_with('-') || (lower.starts_with('m') && !lower.starts_with("ma")) {
        // A minor-ish token we don't recognise precisely → minor7 with the rest
        // as tensions. Excludes `ma…` so `maj`-family never falls to minor.
        (ChordQuality::Min7, vec![s.to_string()])
    } else {
        (ChordQuality::Other, vec![s.to_string()])
    }
}

// ── Theory tables ─────────────────────────────────────────────────────────

/// Semitone of the chord third above the root (major 3rd = 4, minor 3rd = 3).
fn third_interval(q: ChordQuality) -> Option<u8> {
    match q {
        ChordQuality::Maj7
        | ChordQuality::Dom7
        | ChordQuality::Six
        | ChordQuality::Alt
        | ChordQuality::Aug
        | ChordQuality::Dom7Sharp5 => Some(4),
        ChordQuality::Min7
        | ChordQuality::Min7b5
        | ChordQuality::Dim7
        | ChordQuality::MinMaj7
        | ChordQuality::Min6 => Some(3),
        // No 3rd; the sus tone stands in as the upper shell voice.
        ChordQuality::Sus4 => Some(5),
        ChordQuality::Sus2 => Some(2),
        ChordQuality::Other => None,
    }
}

/// Semitone of the chord seventh above the root. 6-chords have no seventh, so
/// their upper shell tone is the 6th (9 semitones) — a real chord tone.
fn seventh_interval(q: ChordQuality) -> Option<u8> {
    match q {
        ChordQuality::Maj7 | ChordQuality::MinMaj7 => Some(11),
        ChordQuality::Dom7
        | ChordQuality::Min7
        | ChordQuality::Min7b5
        | ChordQuality::Alt
        | ChordQuality::Sus4
        | ChordQuality::Dom7Sharp5 => Some(10),
        ChordQuality::Dim7 => Some(9),
        ChordQuality::Six | ChordQuality::Min6 => Some(9),
        // Bare triads carry no seventh; shells fall back.
        ChordQuality::Sus2 | ChordQuality::Aug => None,
        ChordQuality::Other => None,
    }
}

/// Full chord tones (pitch classes) for the arpeggio; `None` for `Other`, whose
/// tones we won't guess.
fn chord_tone_intervals(q: ChordQuality) -> Option<Vec<u8>> {
    let tones = match q {
        ChordQuality::Maj7 => vec![0, 4, 7, 11],
        ChordQuality::Dom7 => vec![0, 4, 7, 10],
        ChordQuality::Min7 => vec![0, 3, 7, 10],
        ChordQuality::Min7b5 => vec![0, 3, 6, 10],
        ChordQuality::Dim7 => vec![0, 3, 6, 9],
        ChordQuality::MinMaj7 => vec![0, 3, 7, 11],
        ChordQuality::Six => vec![0, 4, 7, 9],
        ChordQuality::Min6 => vec![0, 3, 7, 9],
        ChordQuality::Alt => vec![0, 4, 10], // root, M3, m7 (tensions omitted)
        ChordQuality::Sus4 => vec![0, 5, 7, 10],
        ChordQuality::Sus2 => vec![0, 2, 7],
        ChordQuality::Aug => vec![0, 4, 8],
        ChordQuality::Dom7Sharp5 => vec![0, 4, 8, 10],
        ChordQuality::Other => return None,
    };
    Some(tones)
}

/// The chord-scale (intervals from the root). `None` for `Other` → the caller
/// falls back to the arpeggio and flags it.
fn scale_intervals(q: ChordQuality) -> Option<Vec<u8>> {
    let scale = match q {
        ChordQuality::Maj7 | ChordQuality::Six => vec![0, 2, 4, 5, 7, 9, 11], // Ionian
        ChordQuality::Dom7 => vec![0, 2, 4, 5, 7, 9, 10],                     // Mixolydian
        ChordQuality::Min7 | ChordQuality::Min6 => vec![0, 2, 3, 5, 7, 9, 10], // Dorian
        ChordQuality::Min7b5 => vec![0, 1, 3, 5, 6, 8, 10],                   // Locrian
        ChordQuality::Dim7 => vec![0, 2, 3, 5, 6, 8, 9, 11], // whole-half diminished
        ChordQuality::MinMaj7 => vec![0, 2, 3, 5, 7, 9, 11], // melodic minor
        ChordQuality::Alt => vec![0, 1, 3, 4, 6, 8, 10],     // altered (super-locrian)
        ChordQuality::Sus4 => vec![0, 2, 4, 5, 7, 9, 10],    // Mixolydian (4 is the sus)
        ChordQuality::Sus2 => vec![0, 2, 4, 5, 7, 9, 11],    // Ionian
        ChordQuality::Aug | ChordQuality::Dom7Sharp5 => vec![0, 2, 4, 6, 8, 10], // whole-tone
        ChordQuality::Other => return None,
    };
    Some(scale)
}

/// The modal [`scale_intervals`], but a dominant is overridden by how it resolves
/// — the bounded ii–V–i / tritone-sub recognition (spec Phase C): to a minor a 4th
/// up → altered (Mixolydian's natural 9/13 are wrong there); down a semitone → a
/// tritone-sub → lydian-dominant. The major V–I is unchanged.
fn scale_for_change(sym: &ChordSymbol, next: Option<&ChordSymbol>) -> Option<Vec<u8>> {
    if sym.quality == ChordQuality::Dom7 {
        if let Some(next) = next {
            let up_a_fourth = (sym.root + 5) % OCTAVE;
            let down_a_semitone = (sym.root + OCTAVE - 1) % OCTAVE;
            if next.root == up_a_fourth && is_minor_target(next.quality) {
                return Some(vec![0, 1, 3, 4, 6, 8, 10]); // altered
            }
            if next.root == down_a_semitone {
                return Some(vec![0, 2, 4, 6, 7, 9, 10]); // lydian dominant
            }
        }
    }
    scale_intervals(sym.quality)
}

fn is_minor_target(q: ChordQuality) -> bool {
    matches!(
        q,
        ChordQuality::Min7 | ChordQuality::Min6 | ChordQuality::MinMaj7 | ChordQuality::Min7b5
    )
}

fn transpose(root: u8, intervals: &[u8]) -> Vec<u8> {
    intervals.iter().map(|i| (root + i) % OCTAVE).collect()
}

// ── Scaffold specs (core-internal in Phase A) ─────────────────────────────

/// Which scaffold exercise a spec is. Crosses the bincode bridge: it rides in
/// `ScaffoldSpecView` (the preview's per-row identity) and is the commit
/// payload — the shell sends the ticked `kind`s and the core re-derives, so no
/// `ScaffoldSpec` content ever crosses the wire (#1106).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ScaffoldKind {
    Melody,
    Shells,
    GuideToneLines,
    ScalesToChordTones,
    ConstrainedImprov,
}

/// Prefix of the reserved generated-from tag (spec Resolved #2). Kept on the item
/// so re-derivation reconciles by kind (rename-robust); hidden from the tag UI.
pub const SCAFFOLD_TAG_PREFIX: &str = "scaffold:";

impl ScaffoldKind {
    // Slug is independent of the renameable title, so reconciliation survives a rename.
    fn slug(self) -> &'static str {
        match self {
            ScaffoldKind::Melody => "melody",
            ScaffoldKind::Shells => "shells",
            ScaffoldKind::GuideToneLines => "guide-tone-lines",
            ScaffoldKind::ScalesToChordTones => "scales-to-chord-tones",
            ScaffoldKind::ConstrainedImprov => "constrained-improv",
        }
    }

    pub fn scaffold_tag(self) -> String {
        format!("{SCAFFOLD_TAG_PREFIX}{}", self.slug())
    }

    pub fn from_scaffold_tag(tag: &str) -> Option<ScaffoldKind> {
        let slug = tag.strip_prefix(SCAFFOLD_TAG_PREFIX)?;
        [
            ScaffoldKind::Melody,
            ScaffoldKind::Shells,
            ScaffoldKind::GuideToneLines,
            ScaffoldKind::ScalesToChordTones,
            ScaffoldKind::ConstrainedImprov,
        ]
        .into_iter()
        .find(|k| k.slug() == slug)
    }
}

pub fn is_scaffold_tag(tag: &str) -> bool {
    tag.starts_with(SCAFFOLD_TAG_PREFIX)
}

/// The derived per-change content of one exercise. `pitch_classes` is what the
/// exercise asks the player to sound over `chord`; `fallback` marks a change we
/// couldn't map (arpeggio substitute).
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeVoicing {
    pub chord: String,
    pub pitch_classes: Vec<u8>,
    pub fallback: bool,
}

/// A proposed exercise — pure data, no `Item` yet. Phase B's `CommitScaffold`
/// turns confirmed specs into real library items.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaffoldSpec {
    pub kind: ScaffoldKind,
    pub title: String,
    pub key: String,
    pub rationale: String,
    pub content: Vec<ChangeVoicing>,
    pub fallback_count: u8,
}

/// Derive the scaffold curriculum from a chart. Pure; deterministic; derives in
/// the chart's own key (no twelve-key ladder — Phase C).
pub fn derive_scaffold(chart: &ChordChart) -> Vec<ScaffoldSpec> {
    let changes = chart.changes();
    let key = chart.key.clone();

    vec![
        melody_spec(&key),
        shells_spec(&changes, &key),
        guide_tone_spec(&changes, &key),
        scales_spec(&changes, &key),
        improv_spec(&changes, &key),
    ]
}

fn melody_spec(key: &str) -> ScaffoldSpec {
    ScaffoldSpec {
        kind: ScaffoldKind::Melody,
        title: "Learn the melody".to_string(),
        key: key.to_string(),
        rationale: "Hear the tune before you build on it".to_string(),
        content: vec![], // placeholder — no notated content (copyright)
        fallback_count: 0,
    }
}

/// The upper shell tone: the seventh, or the sixth for 6-chords.
fn shell_tones(sym: &ChordSymbol) -> Option<(u8, u8)> {
    let third = third_interval(sym.quality)?;
    let seventh = seventh_interval(sym.quality)?;
    Some(((sym.root + third) % OCTAVE, (sym.root + seventh) % OCTAVE))
}

fn shells_spec(changes: &[&ChordSymbol], key: &str) -> ScaffoldSpec {
    let mut fallback_count = 0;
    let content = changes
        .iter()
        .map(|sym| match shell_tones(sym) {
            Some((third, seventh)) => ChangeVoicing {
                chord: sym.raw.clone(),
                pitch_classes: vec![third, seventh],
                fallback: false,
            },
            None => {
                fallback_count += 1;
                ChangeVoicing {
                    chord: sym.raw.clone(),
                    pitch_classes: vec![sym.root, (sym.root + 7) % OCTAVE],
                    fallback: true,
                }
            }
        })
        .collect();
    ScaffoldSpec {
        kind: ScaffoldKind::Shells,
        title: "Shells".to_string(),
        key: key.to_string(),
        rationale: "3rd + 7th of every chord — the voice-leading skeleton".to_string(),
        content,
        fallback_count,
    }
}

/// Circular (mod-12) semitone distance between two pitch classes, 0..=6.
fn pc_distance(a: u8, b: u8) -> u8 {
    let d = (a + OCTAVE - b) % OCTAVE;
    d.min(OCTAVE - d)
}

fn guide_tone_spec(changes: &[&ChordSymbol], key: &str) -> ScaffoldSpec {
    let mut fallback_count = 0;
    let mut content: Vec<ChangeVoicing> = Vec::with_capacity(changes.len());
    let mut prev: Option<u8> = None;

    for sym in changes {
        let voicing = match shell_tones(sym) {
            Some((third, seventh)) => {
                // Pick the guide tone (3rd or 7th) nearest the previous one, so
                // the line moves by the smallest step — 7→3 resolutions fall out
                // for free. Seed on the 7th (the classic descending line start).
                let chosen = match prev {
                    None => seventh,
                    Some(p) => {
                        if pc_distance(third, p) <= pc_distance(seventh, p) {
                            third
                        } else {
                            seventh
                        }
                    }
                };
                prev = Some(chosen);
                ChangeVoicing {
                    chord: sym.raw.clone(),
                    pitch_classes: vec![chosen],
                    fallback: false,
                }
            }
            None => {
                fallback_count += 1;
                // Keep the line going from the root; don't reset the anchor.
                let chosen = sym.root;
                prev = Some(chosen);
                ChangeVoicing {
                    chord: sym.raw.clone(),
                    pitch_classes: vec![chosen],
                    fallback: true,
                }
            }
        };
        content.push(voicing);
    }

    ScaffoldSpec {
        kind: ScaffoldKind::GuideToneLines,
        title: "Guide-tone lines".to_string(),
        key: key.to_string(),
        rationale: "Connect 3rds to 7ths across each change".to_string(),
        content,
        fallback_count,
    }
}

fn scales_spec(changes: &[&ChordSymbol], key: &str) -> ScaffoldSpec {
    let mut fallback_count = 0;
    let content = changes
        .iter()
        .enumerate()
        .map(|(i, sym)| {
            let next = changes.get(i + 1).copied();
            match scale_for_change(sym, next) {
                Some(scale) => ChangeVoicing {
                    chord: sym.raw.clone(),
                    pitch_classes: transpose(sym.root, &scale),
                    fallback: false,
                },
                None => {
                    fallback_count += 1;
                    // Arpeggio fallback: the chord tones we do know, else root+5th.
                    let intervals = chord_tone_intervals(sym.quality).unwrap_or_else(|| vec![0, 7]);
                    ChangeVoicing {
                        chord: sym.raw.clone(),
                        pitch_classes: transpose(sym.root, &intervals),
                        fallback: true,
                    }
                }
            }
        })
        .collect();
    ScaffoldSpec {
        kind: ScaffoldKind::ScalesToChordTones,
        title: "Scales to chord tones".to_string(),
        key: key.to_string(),
        rationale: "Run each chord-scale, landing on a chord tone (ii–V aware)".to_string(),
        content,
        fallback_count,
    }
}

fn improv_spec(changes: &[&ChordSymbol], key: &str) -> ScaffoldSpec {
    let mut fallback_count = 0;
    let content = changes
        .iter()
        .map(|sym| match chord_tone_intervals(sym.quality) {
            Some(intervals) => ChangeVoicing {
                chord: sym.raw.clone(),
                pitch_classes: transpose(sym.root, &intervals),
                fallback: false,
            },
            None => {
                fallback_count += 1;
                ChangeVoicing {
                    chord: sym.raw.clone(),
                    pitch_classes: transpose(sym.root, &[0, 7]),
                    fallback: true,
                }
            }
        })
        .collect();
    ScaffoldSpec {
        kind: ScaffoldKind::ConstrainedImprov,
        title: "Constrained improv".to_string(),
        key: key.to_string(),
        rationale: "Chord tones only, then rhythm — one ladder".to_string(),
        content,
        fallback_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Parser: roots + accidentals ──

    #[test]
    fn parses_natural_and_accidental_roots() {
        let chart = parse_chart("| C | F# | Bb |", "C", Modality::Major).unwrap();
        let roots: Vec<u8> = chart.changes().iter().map(|s| s.root).collect();
        assert_eq!(roots, vec![0, 6, 10]); // C, F#, Bb
    }

    #[test]
    fn rejects_a_non_note_root_naming_the_bar() {
        let err = parse_chart("| Cm7 | F7 | Hm7b5 |", "C", Modality::Major).unwrap_err();
        assert_eq!(err.bar, 3);
        assert_eq!(err.token, "Hm7b5");
        assert!(err.message.contains("isn't a note name"));
    }

    #[test]
    fn empty_chart_is_an_error_not_a_partial() {
        let err = parse_chart("   \n  \n", "C", Modality::Major).unwrap_err();
        assert_eq!(err.bar, 0);
    }

    // ── Parser: qualities ──

    fn quality_of(token: &str) -> ChordQuality {
        let sym = parse_chord(token, 1).unwrap();
        sym.quality
    }

    #[test]
    fn recognises_the_v1_vocabulary() {
        assert_eq!(quality_of("Cmaj7"), ChordQuality::Maj7);
        assert_eq!(quality_of("C7"), ChordQuality::Dom7);
        assert_eq!(quality_of("Cm7"), ChordQuality::Min7);
        assert_eq!(quality_of("Cm7b5"), ChordQuality::Min7b5);
        assert_eq!(quality_of("Cø7"), ChordQuality::Min7b5);
        assert_eq!(quality_of("Cdim7"), ChordQuality::Dim7);
        assert_eq!(quality_of("Cm(maj7)"), ChordQuality::MinMaj7);
        assert_eq!(quality_of("C6"), ChordQuality::Six);
        assert_eq!(quality_of("Cm6"), ChordQuality::Min6);
        assert_eq!(quality_of("C7alt"), ChordQuality::Alt);
        assert_eq!(quality_of("Calt"), ChordQuality::Alt);
    }

    #[test]
    fn bare_root_is_a_major_sound() {
        assert_eq!(quality_of("C"), ChordQuality::Maj7);
    }

    #[test]
    fn recognises_phase_c_vocabulary() {
        // Suspensions: any 7-bearing sus is the dominant sus; sus2 keeps its 2nd.
        assert_eq!(quality_of("C7sus4"), ChordQuality::Sus4);
        assert_eq!(quality_of("Csus4"), ChordQuality::Sus4);
        assert_eq!(quality_of("Csus"), ChordQuality::Sus4);
        assert_eq!(quality_of("C9sus4"), ChordQuality::Sus4);
        // A number-bearing sus (13sus/11sus) must stay a sus, not decay to the
        // bare-dominant it prefixes.
        assert_eq!(quality_of("C13sus"), ChordQuality::Sus4);
        assert_eq!(quality_of("C11sus4"), ChordQuality::Sus4);
        assert_eq!(quality_of("Csus2"), ChordQuality::Sus2);
        // Augmented: bare triad vs dominant ♯5.
        assert_eq!(quality_of("Caug"), ChordQuality::Aug);
        assert_eq!(quality_of("C+"), ChordQuality::Aug);
        assert_eq!(quality_of("C7#5"), ChordQuality::Dom7Sharp5);
        assert_eq!(quality_of("C+7"), ChordQuality::Dom7Sharp5);
        assert_eq!(quality_of("Caug7"), ChordQuality::Dom7Sharp5);
    }

    #[test]
    fn six_nine_is_a_major_six_not_a_broken_slash_bass() {
        // Regression: `C6/9` used to split on `/` and fail parsing a "9" bass;
        // the digit-after-slash guard keeps it a major 6 with a 9 tension.
        let sym = parse_chord("C6/9", 1).unwrap();
        assert_eq!((sym.root, sym.quality), (0, ChordQuality::Six));
        assert_eq!(sym.bass, None);
    }

    #[test]
    fn slash_bass_still_parses_after_the_six_nine_guard() {
        let sym = parse_chord("C/E", 1).unwrap();
        assert_eq!(
            (sym.root, sym.quality, sym.bass),
            (0, ChordQuality::Maj7, Some(4))
        );
    }

    #[test]
    fn new_qualities_derive_their_scale_and_tones() {
        let sus = parse_chord("C7sus4", 1).unwrap(); // Mixolydian, tones root/4/5/b7
        assert_eq!(
            scale_intervals(sus.quality),
            Some(vec![0, 2, 4, 5, 7, 9, 10])
        );
        assert_eq!(chord_tone_intervals(sus.quality), Some(vec![0, 5, 7, 10]));
        let aug7 = parse_chord("C7#5", 1).unwrap(); // whole-tone, tones root/M3/♯5/b7
        assert_eq!(scale_intervals(aug7.quality), Some(vec![0, 2, 4, 6, 8, 10]));
        assert_eq!(chord_tone_intervals(aug7.quality), Some(vec![0, 4, 8, 10]));
    }

    #[test]
    fn unknown_quality_falls_back_to_other() {
        assert_eq!(quality_of("Cadd9"), ChordQuality::Other);
    }

    // ── Context-aware scale selection (ii–V–i / tritone sub) ──

    #[test]
    fn minor_two_five_dominant_takes_the_altered_scale() {
        // Autumn Leaves' D7 → Gm7 (up a 4th to minor i): D altered, not Mixolydian.
        let specs = derive_scaffold(&autumn_leaves());
        let scales = &specs[3];
        let d7 = scales
            .content
            .iter()
            .find(|c| c.chord == "D7")
            .expect("D7 is in the changes");
        assert_eq!(d7.pitch_classes, transpose(2, &[0, 1, 3, 4, 6, 8, 10]));
    }

    #[test]
    fn major_five_one_dominant_stays_mixolydian() {
        let specs = derive_scaffold(&autumn_leaves()); // F7 → Bbmaj7 is a major V–I
        let scales = &specs[3];
        assert_eq!(scales.content[1].chord, "F7");
        assert_eq!(
            scales.content[1].pitch_classes,
            transpose(5, &[0, 2, 4, 5, 7, 9, 10])
        );
    }

    #[test]
    fn tritone_sub_dominant_takes_lydian_dominant() {
        // Db7 → Cmaj7 (down a semitone): tritone-sub, Db lydian-dominant.
        let chart = parse_chart("| Db7 | Cmaj7 |", "C", Modality::Major).unwrap();
        let scales = &derive_scaffold(&chart)[3];
        assert_eq!(scales.content[0].chord, "Db7");
        assert_eq!(
            scales.content[0].pitch_classes,
            transpose(1, &[0, 2, 4, 6, 7, 9, 10])
        );
    }

    #[test]
    fn keeps_tensions_as_extensions_without_changing_quality() {
        let sym = parse_chord("C7b9", 1).unwrap();
        assert_eq!(sym.quality, ChordQuality::Dom7);
        assert_eq!(sym.extensions, vec!["b9".to_string()]);
    }

    #[test]
    fn parses_a_slash_bass() {
        let sym = parse_chord("Dm7/G", 1).unwrap();
        assert_eq!(sym.root, 2); // D
        assert_eq!(sym.quality, ChordQuality::Min7);
        assert_eq!(sym.bass, Some(7)); // G
    }

    #[test]
    fn flat_root_is_kept_across_quality_and_tension() {
        // A flat root survives whether the quality starts with a letter or a
        // digit; a `b5` after the quality stays a tension on a natural root.
        assert_eq!(parse_chord("Bb", 1).unwrap().root, 10);
        assert_eq!(parse_chord("Bbmaj7", 1).unwrap().root, 10);
        // Regression: flat-root dominants (Bb7, Eb7, Ab7) — very common — must
        // keep their flat, not decay to a natural root + `Other` fallback.
        let bb7 = parse_chord("Bb7", 1).unwrap();
        assert_eq!((bb7.root, bb7.quality), (10, ChordQuality::Dom7));
        assert_eq!(parse_chord("Eb7", 1).unwrap().root, 3);
        assert_eq!(parse_chord("Ab7", 1).unwrap().root, 8);
        // `C7b5`: natural root, dominant, `b5` retained as a tension.
        let c7b5 = parse_chord("C7b5", 1).unwrap();
        assert_eq!((c7b5.root, c7b5.quality), (0, ChordQuality::Dom7));
        assert_eq!(c7b5.extensions, vec!["b5".to_string()]);
    }

    #[test]
    fn maj_family_is_major_never_minor() {
        // "maj…" starts with 'm' but must not fall to the minor branch.
        assert_eq!(quality_of("Cmaj9"), ChordQuality::Maj7);
        assert_eq!(quality_of("Cmaj"), ChordQuality::Maj7);
        assert_eq!(quality_of("Cma7"), ChordQuality::Maj7);
    }

    #[test]
    fn bare_number_extensions_pick_the_right_quality() {
        assert_eq!(quality_of("C9"), ChordQuality::Dom7);
        assert_eq!(quality_of("C13"), ChordQuality::Dom7);
        assert_eq!(quality_of("Cm9"), ChordQuality::Min7);
        assert_eq!(quality_of("Cm11"), ChordQuality::Min7);
    }

    // ── Parser: sections + beats ──

    #[test]
    fn splits_sections_and_bars() {
        let chart =
            parse_chart("[A]\n| Cm7 | F7 |\n[B]\n| Bbmaj7 |", "Bb", Modality::Major).unwrap();
        assert_eq!(chart.sections.len(), 2);
        assert_eq!(chart.sections[0].label, Some("A".to_string()));
        assert_eq!(chart.sections[0].bars.len(), 2);
        assert_eq!(chart.sections[1].label, Some("B".to_string()));
    }

    #[test]
    fn splits_beats_evenly_within_a_bar() {
        let chart = parse_chart("| Cm7 F7 |", "C", Modality::Major).unwrap();
        let beats: Vec<u8> = chart.sections[0].bars[0]
            .chords
            .iter()
            .map(|c| c.beats)
            .collect();
        assert_eq!(beats, vec![2, 2]);
    }

    #[test]
    fn a_label_can_lead_a_bar_line() {
        let chart = parse_chart("[A] | Cm7 | F7 |", "C", Modality::Major).unwrap();
        assert_eq!(chart.sections.len(), 1);
        assert_eq!(chart.changes().len(), 2);
    }

    // ── Derivation goldens: Autumn Leaves A-section, G minor ──
    // Changes aren't copyrightable (the iReal Pro model); the melody is.

    fn autumn_leaves() -> ChordChart {
        parse_chart(
            "| Cm7 | F7 | Bbmaj7 | Ebmaj7 | Am7b5 | D7 | Gm7 | Gm7 |",
            "G",
            Modality::Minor,
        )
        .unwrap()
    }

    #[test]
    fn derive_yields_the_five_generators_in_order() {
        let specs = derive_scaffold(&autumn_leaves());
        let kinds: Vec<ScaffoldKind> = specs.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            vec![
                ScaffoldKind::Melody,
                ScaffoldKind::Shells,
                ScaffoldKind::GuideToneLines,
                ScaffoldKind::ScalesToChordTones,
                ScaffoldKind::ConstrainedImprov,
            ]
        );
    }

    #[test]
    fn melody_is_a_titled_placeholder_with_no_content() {
        let specs = derive_scaffold(&autumn_leaves());
        let melody = &specs[0];
        assert_eq!(melody.title, "Learn the melody");
        assert!(melody.content.is_empty());
    }

    #[test]
    fn shells_are_third_and_seventh_to_the_pitch_class() {
        let specs = derive_scaffold(&autumn_leaves());
        let shells = &specs[1];
        // Cm7: 3rd Eb=3, 7th Bb=10. F7: 3rd A=9, 7th Eb=3.
        // Bbmaj7: 3rd D=2, 7th A=9. Ebmaj7: 3rd G=7, 7th D=2.
        // Am7b5: 3rd C=0, 7th G=7. D7: 3rd F#=6, 7th C=0.
        // Gm7: 3rd Bb=10, 7th F=5.
        let pcs: Vec<Vec<u8>> = shells
            .content
            .iter()
            .map(|c| c.pitch_classes.clone())
            .collect();
        assert_eq!(
            pcs,
            vec![
                vec![3, 10],
                vec![9, 3],
                vec![2, 9],
                vec![7, 2],
                vec![0, 7],
                vec![6, 0],
                vec![10, 5],
                vec![10, 5],
            ]
        );
        assert_eq!(shells.fallback_count, 0);
    }

    #[test]
    fn guide_tone_line_moves_by_smallest_step() {
        let specs = derive_scaffold(&autumn_leaves());
        let line: Vec<u8> = specs[2]
            .content
            .iter()
            .flat_map(|c| c.pitch_classes.clone())
            .collect();
        // Seed on Cm7's 7th (Bb=10); then nearest of {3rd,7th} each change.
        // Cm7:10 → F7{9,3}→9 → Bbmaj7{2,9}→9 → Ebmaj7{7,2}→7 →
        // Am7b5{0,7}→7 → D7{6,0}→6 → Gm7{10,5}→5 → Gm7{10,5}→5.
        assert_eq!(line, vec![10, 9, 9, 7, 7, 6, 5, 5]);
    }

    #[test]
    fn scales_pick_the_chord_scale_per_quality() {
        let specs = derive_scaffold(&autumn_leaves());
        let scales = &specs[3];
        // Cm7 → C Dorian: C D Eb F G A Bb = 0,2,3,5,7,9,10.
        assert_eq!(scales.content[0].pitch_classes, vec![0, 2, 3, 5, 7, 9, 10]);
        // F7 → F Mixolydian: F G A Bb C D Eb = 5,7,9,10,0,2,3.
        assert_eq!(scales.content[1].pitch_classes, vec![5, 7, 9, 10, 0, 2, 3]);
        // Bbmaj7 → Bb Ionian: Bb C D Eb F G A = 10,0,2,3,5,7,9.
        assert_eq!(scales.content[2].pitch_classes, vec![10, 0, 2, 3, 5, 7, 9]);
        assert_eq!(scales.fallback_count, 0);
    }

    #[test]
    fn constrained_improv_is_chord_tones_per_change() {
        let specs = derive_scaffold(&autumn_leaves());
        let improv = &specs[4];
        // Cm7 chord tones: C Eb G Bb = 0,3,7,10.
        assert_eq!(improv.content[0].pitch_classes, vec![0, 3, 7, 10]);
    }

    // ── Fallback ──

    #[test]
    fn out_of_vocab_chord_falls_back_to_arpeggio_and_is_flagged() {
        // `add9` stays out of vocab (major triad add 9, no mapped scale).
        let chart = parse_chart("| Cadd9 |", "C", Modality::Major).unwrap();
        let specs = derive_scaffold(&chart);
        let scales = &specs[3];
        assert!(scales.content[0].fallback);
        assert_eq!(scales.fallback_count, 1);
        // Arpeggio fallback for Other = root + perfect 5th.
        assert_eq!(scales.content[0].pitch_classes, vec![0, 7]);
    }

    #[test]
    fn seventh_less_triads_fall_back_on_shells() {
        // Sus2 and Aug have no 7th, so shells (3rd + 7th) can't voice them and
        // flag the fallback rather than inventing a seventh.
        let chart = parse_chart("| Csus2 | Caug |", "C", Modality::Major).unwrap();
        let shells = &derive_scaffold(&chart)[1];
        assert!(shells.content.iter().all(|c| c.fallback));
        assert_eq!(shells.fallback_count, 2);
    }

    // ── Bridge round-trip (#846) ──

    #[test]
    fn chord_chart_round_trips_on_the_ffi_bincode_wire() {
        // Includes every Phase C quality (sus4/sus2/aug/dom7♯5) so the new enum
        // variants are exercised on the bincode wire, not just the JSON one (#846).
        let chart = parse_chart(
            "[A] | Cmaj7 F7 | Dm7/G | Cø7 | Bbdim7 | Ealt | G7sus4 Dsus2 | Faug C7#5 |",
            "C",
            Modality::Major,
        )
        .unwrap();
        crate::domain::types::assert_round_trips(chart);
    }
}

//! On-device page recognition — the shell runs Vision (and, from phase C,
//! Foundation Models); the core decides what any of it means.
//!
//! Everything the shell returns is text with geometry. `read_fields` is the
//! only place a recognised line becomes a title, a composer or a tempo, and it
//! is a pure function so its tests need no device.

use crux_core::capability::Operation;
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::domain::types::Tempo;
use crate::validation::{MAX_BPM, MAX_COMPOSER, MAX_TEMPO_MARKING, MAX_TITLE, MIN_BPM};

// ── The effect contract ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum RecognitionOperation {
    /// Read a page the shell has already written to disk (phase A writes the
    /// bytes shell-side, so the core only ever names the file).
    ReadPage { photo_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum RecognitionOutput {
    Page(PageReading),
    /// No recognition available on this device. Not an error: the user types
    /// the fields, and the photo is still saved.
    Unsupported,
    Failed,
}

impl Operation for RecognitionOperation {
    type Output = RecognitionOutput;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PageReading {
    /// Recognised lines in reading order, with normalised geometry so the core
    /// can reason about position without ever seeing the image.
    pub lines: Vec<RecognisedLine>,
    /// Populated only where Foundation Models ran (phase C). `None` on most
    /// devices, and `read_fields` must produce a usable draft regardless.
    pub suggested: Option<SuggestedFields>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RecognisedLine {
    pub text: String,
    /// Normalised 0..1, origin top-left.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SuggestedFields {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub tempo_marking: Option<String>,
    pub bpm: Option<u16>,
    pub chart_text: Option<String>,
}

// ── The draft ───────────────────────────────────────────────────────

/// A proposal, exactly as `ScaffoldSpec` is: it becomes an `Item` only when the
/// user confirms. Every field carries where it came from, so the sheet can show
/// a low-confidence read differently from a clean one.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PhotoDraft {
    pub title: Option<TextDraftField>,
    pub composer: Option<TextDraftField>,
    pub tempo: Option<TempoDraftField>,
    /// Phase D. Always `None` today.
    pub chart_text: Option<TextDraftField>,
}

/// The spec pins this as `DraftField<T>`; it is split per payload type because
/// facet typegen emits no generics across the bridge.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TextDraftField {
    pub value: String,
    pub source: DraftSource,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoDraftField {
    pub value: Tempo,
    pub source: DraftSource,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum DraftSource {
    /// Geometry heuristics in the core. The floor, available everywhere.
    Recognised,
    /// The on-device model chose it and it survived the substring clamp.
    Suggested,
}

/// Below this a read is shown as uncertain rather than hidden (decision 7).
pub const LOW_CONFIDENCE: f32 = 0.5;

pub fn read_page(photo_id: String) -> Command<Effect, Event> {
    Command::request_from_shell(RecognitionOperation::ReadPage { photo_id })
        .then_send(Event::PhotoRead)
}

// ── Interpretation ──────────────────────────────────────────────────

/// Tempo markings we will name from a page, longest first so
/// "Allegro moderato" wins over "Allegro". Canonical spelling is ours, not the
/// page's: OCR reads `ALLEGRO` and the user should see `Allegro`.
const TEMPO_MARKINGS: &[&str] = &[
    "Allegro moderato",
    "Allegro vivace",
    "Molto allegro",
    "Andante moderato",
    "Adagio ma non troppo",
    "Larghetto",
    "Moderato",
    "Andantino",
    "Sostenuto",
    "Allegretto",
    "Grazioso",
    "Maestoso",
    "Marcato",
    "Adagietto",
    "Allegro",
    "Andante",
    "Moderate",
    "Adagio",
    "Largo",
    "Lento",
    "Grave",
    "Vivace",
    "Presto",
    "Prestissimo",
    "Ballad",
    "Swing",
    "Bossa",
    "Latin",
    "Rubato",
];

/// Credit prefixes, longest first. The `f32` is how much the match dampens the
/// line's own OCR confidence: a bare `by` is a weaker claim than `Music by`.
const CREDIT_PREFIXES: &[(&str, f32)] = &[
    ("words and music by", 1.0),
    ("music and lyrics by", 1.0),
    ("composed by", 1.0),
    ("music by", 1.0),
    ("written by", 0.9),
    ("by", 0.8),
];

/// A line this high up the page can be the title. Below it we are into the
/// first stave, where the largest text is a lyric, not a heading.
const TITLE_BAND: f32 = 0.4;

/// Deterministic. Prefers `suggested` where a field survives the substring
/// clamp (decision 5), otherwise falls back to geometry heuristics.
#[must_use]
pub fn read_fields(page: &PageReading) -> PhotoDraft {
    let heuristic = heuristic_draft(page);
    let Some(suggested) = page.suggested.as_ref() else {
        return heuristic;
    };

    let haystack = normalise(
        &page
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );

    PhotoDraft {
        title: clamped_text(suggested.title.as_deref(), &haystack, MAX_TITLE).or(heuristic.title),
        composer: clamped_text(suggested.composer.as_deref(), &haystack, MAX_COMPOSER)
            .or(heuristic.composer),
        tempo: clamped_tempo(suggested, &haystack).or(heuristic.tempo),
        chart_text: clamped_text(suggested.chart_text.as_deref(), &haystack, usize::MAX)
            .or(heuristic.chart_text),
    }
}

/// Decision 5, the whole of it: a suggested value the page does not literally
/// carry is discarded, so a model cannot invent a composer. Case- and
/// whitespace-insensitive, because OCR spacing is not the user's problem.
fn clamped_text(value: Option<&str>, haystack: &str, max: usize) -> Option<TextDraftField> {
    let value = value?.trim();
    if value.is_empty() || value.len() > max {
        return None;
    }
    if !haystack.contains(&normalise(value)) {
        return None;
    }
    Some(TextDraftField {
        value: value.to_string(),
        source: DraftSource::Suggested,
        confidence: 1.0,
    })
}

fn clamped_tempo(suggested: &SuggestedFields, haystack: &str) -> Option<TempoDraftField> {
    let marking = clamped_text(
        suggested.tempo_marking.as_deref(),
        haystack,
        MAX_TEMPO_MARKING,
    )
    .map(|f| f.value);
    let bpm = suggested
        .bpm
        .filter(|bpm| (MIN_BPM..=MAX_BPM).contains(bpm))
        .filter(|bpm| haystack.contains(&bpm.to_string()));

    Tempo::from_parts(marking, bpm).map(|value| TempoDraftField {
        value,
        source: DraftSource::Suggested,
        confidence: 1.0,
    })
}

fn normalise(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn heuristic_draft(page: &PageReading) -> PhotoDraft {
    PhotoDraft {
        title: heuristic_title(&page.lines),
        composer: heuristic_composer(&page.lines),
        tempo: heuristic_tempo(&page.lines),
        chart_text: None,
    }
}

/// The largest text in the top band, ignoring anything that has already
/// explained itself as a credit or a tempo.
fn heuristic_title(lines: &[RecognisedLine]) -> Option<TextDraftField> {
    lines
        .iter()
        .filter(|l| l.y < TITLE_BAND)
        .filter(|l| !l.text.trim().is_empty())
        .filter(|l| credit_prefix(&l.text).is_none())
        .filter(|l| marking_in(&l.text).is_none() && bpm_in(&l.text).is_none())
        .filter(|l| l.text.trim().len() <= MAX_TITLE)
        .max_by(|a, b| a.height.total_cmp(&b.height).then(b.y.total_cmp(&a.y)))
        .map(|l| TextDraftField {
            value: l.text.trim().to_string(),
            source: DraftSource::Recognised,
            confidence: l.confidence,
        })
}

fn heuristic_composer(lines: &[RecognisedLine]) -> Option<TextDraftField> {
    lines.iter().find_map(|l| {
        let (rest, damping) = credit_prefix(&l.text)?;
        let rest = rest.trim();
        (!rest.is_empty() && rest.len() <= MAX_COMPOSER).then(|| TextDraftField {
            value: rest.to_string(),
            source: DraftSource::Recognised,
            confidence: l.confidence * damping,
        })
    })
}

/// Returns what follows the credit prefix, and how far the prefix dampens
/// confidence. Matched only at the start of the line.
fn credit_prefix(text: &str) -> Option<(&str, f32)> {
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    CREDIT_PREFIXES.iter().find_map(|(prefix, damping)| {
        let rest = lower.strip_prefix(prefix)?;
        // Must be a word boundary: "Bydgoszcz Suite" is not a credit.
        if !rest.is_empty() && !rest.starts_with(|c: char| c.is_whitespace() || c == ':') {
            return None;
        }
        let rest = &trimmed[trimmed
            .char_indices()
            .nth(prefix.chars().count())
            .map_or(trimmed.len(), |(at, _)| at)..];
        Some((rest.trim_start_matches([':', ' ', '\t']), *damping))
    })
}

fn heuristic_tempo(lines: &[RecognisedLine]) -> Option<TempoDraftField> {
    let band: Vec<&RecognisedLine> = lines.iter().filter(|l| l.y < TITLE_BAND).collect();

    let marking = band
        .iter()
        .find_map(|l| marking_in(&l.text).map(|m| (m, l.confidence)));
    let bpm = band
        .iter()
        .find_map(|l| bpm_in(&l.text).map(|b| (b, l.confidence)));

    let confidence = match (&marking, &bpm) {
        (Some((_, a)), Some((_, b))) => a.min(*b),
        (Some((_, c)), None) | (None, Some((_, c))) => *c,
        (None, None) => return None,
    };

    Tempo::from_parts(marking.map(|(m, _)| m), bpm.map(|(b, _)| b)).map(|value| TempoDraftField {
        value,
        source: DraftSource::Recognised,
        confidence,
    })
}

fn marking_in(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    TEMPO_MARKINGS.iter().find_map(|marking| {
        let needle = marking.to_lowercase();
        lower
            .match_indices(&needle)
            .any(|(at, _)| {
                let before_ok = at == 0
                    || !lower[..at]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_alphanumeric);
                let after_ok = !lower[at + needle.len()..]
                    .chars()
                    .next()
                    .is_some_and(char::is_alphanumeric);
                before_ok && after_ok
            })
            .then(|| (*marking).to_string())
    })
}

/// Only a number that follows an `=` counts. `♩= 120` OCRs with the glyph
/// mangled or dropped, but the equals survives; a bare number near the top of a
/// page is a page number or a bar number at least as often as it is a tempo
/// (open question 3).
fn bpm_in(text: &str) -> Option<u16> {
    let (_, after) = text.rsplit_once('=')?;
    let digits: String = after
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse::<u16>()
        .ok()
        .filter(|bpm| (MIN_BPM..=MAX_BPM).contains(bpm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemKind;
    use crate::domain::types::{assert_round_trips, CreateItem};
    use crate::validation;

    fn line(text: &str, y: f32, height: f32) -> RecognisedLine {
        RecognisedLine {
            text: text.to_string(),
            x: 0.1,
            y,
            width: 0.8,
            height,
            confidence: 0.9,
        }
    }

    fn page(lines: Vec<RecognisedLine>) -> PageReading {
        PageReading {
            lines,
            suggested: None,
        }
    }

    /// A printed first page: title big at the top, credit under it, tempo above
    /// the first stave, then lyrics well down the page.
    fn printed_page() -> PageReading {
        page(vec![
            line("Autumn Leaves", 0.08, 0.09),
            line("Music by Joseph Kosma", 0.18, 0.03),
            line("Moderato  = 120", 0.27, 0.025),
            line("The falling leaves drift by my window", 0.55, 0.04),
        ])
    }

    // ── Title ───────────────────────────────────────────────────────

    #[test]
    fn title_is_the_largest_text_in_the_top_band() {
        let draft = read_fields(&printed_page());
        let title = draft.title.expect("a title");
        assert_eq!(title.value, "Autumn Leaves");
        assert_eq!(title.source, DraftSource::Recognised);
    }

    /// Deleting the top-band filter makes this pass on the lyric instead: the
    /// biggest text on a lead sheet is often a lyric line halfway down.
    #[test]
    fn title_ignores_larger_text_below_the_top_band() {
        let draft = read_fields(&page(vec![
            line("Prelude in C", 0.10, 0.04),
            line("VERSE ONE, SET HUGE", 0.60, 0.20),
        ]));
        assert_eq!(draft.title.expect("a title").value, "Prelude in C");
    }

    /// Deleting the credit filter makes this pass on the credit line.
    #[test]
    fn title_is_never_the_credit_line() {
        let draft = read_fields(&page(vec![
            line("Music by Erik Satie", 0.10, 0.09),
            line("Gymnopedie No. 1", 0.20, 0.04),
        ]));
        assert_eq!(draft.title.expect("a title").value, "Gymnopedie No. 1");
    }

    #[test]
    fn title_carries_the_lines_own_confidence() {
        let mut lines = vec![line("Blurry Title", 0.08, 0.09)];
        lines[0].confidence = 0.31;
        let draft = read_fields(&page(lines));
        assert_eq!(draft.title.expect("a title").confidence, 0.31);
    }

    #[test]
    fn a_page_with_no_text_reads_into_nothing() {
        let draft = read_fields(&page(vec![]));
        assert_eq!(draft, PhotoDraft::default());
    }

    // ── Composer ────────────────────────────────────────────────────

    #[test]
    fn composer_is_read_from_each_credit_form() {
        for (text, expected) in [
            ("Music by Joseph Kosma", "Joseph Kosma"),
            ("Words and Music by Cole Porter", "Cole Porter"),
            ("Composed by Clara Schumann", "Clara Schumann"),
            ("by Thelonious Monk", "Thelonious Monk"),
            ("Music by: Bill Evans", "Bill Evans"),
        ] {
            let draft = read_fields(&page(vec![line(text, 0.2, 0.03)]));
            assert_eq!(
                draft.composer.expect("a composer").value,
                expected,
                "reading {text:?}"
            );
        }
    }

    /// Deleting the word-boundary check reads "dgoszcz Suite" as a composer.
    #[test]
    fn a_word_merely_starting_with_by_is_not_a_credit() {
        let draft = read_fields(&page(vec![line("Bydgoszcz Suite", 0.1, 0.05)]));
        assert!(draft.composer.is_none());
        assert_eq!(draft.title.expect("a title").value, "Bydgoszcz Suite");
    }

    /// Deleting the damping factor makes both forms equally confident, which
    /// is the claim being made: a bare `by` is a weaker signal than `Music by`.
    #[test]
    fn a_bare_by_is_less_confident_than_an_explicit_credit() {
        let bare = read_fields(&page(vec![line("by Duke Ellington", 0.2, 0.03)]))
            .composer
            .expect("a composer")
            .confidence;
        let explicit = read_fields(&page(vec![line("Music by Duke Ellington", 0.2, 0.03)]))
            .composer
            .expect("a composer")
            .confidence;
        assert!(
            bare < explicit,
            "bare {bare} should be less confident than explicit {explicit}"
        );
    }

    #[test]
    fn a_credit_with_no_name_after_it_is_not_a_composer() {
        let draft = read_fields(&page(vec![line("Music by", 0.2, 0.03)]));
        assert!(draft.composer.is_none());
    }

    // ── Tempo ───────────────────────────────────────────────────────

    #[test]
    fn tempo_reads_the_marking_and_the_bpm_together() {
        let tempo = read_fields(&printed_page()).tempo.expect("a tempo");
        assert_eq!(tempo.value.marking.as_deref(), Some("Moderato"));
        assert_eq!(tempo.value.bpm, Some(120));
    }

    #[test]
    fn a_marking_is_named_in_our_spelling_not_the_pages() {
        let draft = read_fields(&page(vec![line("ALLEGRO", 0.2, 0.03)]));
        assert_eq!(
            draft.tempo.expect("a tempo").value.marking.as_deref(),
            Some("Allegro")
        );
    }

    #[test]
    fn a_longer_marking_wins_over_the_word_inside_it() {
        let draft = read_fields(&page(vec![line("Allegro moderato", 0.2, 0.03)]));
        assert_eq!(
            draft.tempo.expect("a tempo").value.marking.as_deref(),
            Some("Allegro moderato")
        );
    }

    /// Open question 3, answered conservatively. Deleting the `=` requirement
    /// reads the page number as a tempo.
    #[test]
    fn a_bare_number_is_not_a_bpm() {
        let draft = read_fields(&page(vec![
            line("Sonata in G", 0.08, 0.09),
            line("48", 0.02, 0.02),
        ]));
        assert!(draft.tempo.is_none());
    }

    #[test]
    fn a_bpm_outside_the_accepted_range_is_dropped() {
        for text in ["= 0", "= 900", "= 40000"] {
            let draft = read_fields(&page(vec![line(text, 0.2, 0.03)]));
            assert!(draft.tempo.is_none(), "reading {text:?}");
        }
    }

    #[test]
    fn a_mangled_note_glyph_still_yields_the_bpm() {
        let draft = read_fields(&page(vec![line("J= 88", 0.2, 0.03)]));
        assert_eq!(draft.tempo.expect("a tempo").value.bpm, Some(88));
    }

    #[test]
    fn tempo_takes_the_weaker_of_its_two_lines_confidences() {
        let mut lines = vec![line("Andante", 0.15, 0.03), line("= 72", 0.20, 0.03)];
        lines[0].confidence = 0.8;
        lines[1].confidence = 0.4;
        let tempo = read_fields(&page(lines)).tempo.expect("a tempo");
        assert_eq!(tempo.confidence, 0.4);
    }

    // ── The substring clamp (key decision 5) ────────────────────────

    fn suggested_page(suggested: SuggestedFields) -> PageReading {
        PageReading {
            suggested: Some(suggested),
            ..printed_page()
        }
    }

    fn no_suggestions() -> SuggestedFields {
        SuggestedFields {
            title: None,
            composer: None,
            tempo_marking: None,
            bpm: None,
            chart_text: None,
        }
    }

    /// The whole of decision 5: a ~3B model asked to extract will sometimes
    /// produce a plausible composer that is not on the page. Deleting the
    /// `haystack.contains` check lets "Cole Porter" through from a Kosma page.
    #[test]
    fn a_suggestion_the_page_does_not_carry_is_discarded() {
        let draft = read_fields(&suggested_page(SuggestedFields {
            composer: Some("Cole Porter".to_string()),
            ..no_suggestions()
        }));
        let composer = draft.composer.expect("the heuristic composer");
        assert_eq!(composer.value, "Joseph Kosma");
        assert_eq!(composer.source, DraftSource::Recognised);
    }

    #[test]
    fn a_suggestion_the_page_carries_is_preferred_over_the_heuristic() {
        let draft = read_fields(&suggested_page(SuggestedFields {
            composer: Some("Kosma".to_string()),
            ..no_suggestions()
        }));
        let composer = draft.composer.expect("a composer");
        assert_eq!(composer.value, "Kosma");
        assert_eq!(composer.source, DraftSource::Suggested);
    }

    #[test]
    fn the_clamp_ignores_case_and_ocr_spacing() {
        let draft = read_fields(&suggested_page(SuggestedFields {
            title: Some("autumn   leaves".to_string()),
            ..no_suggestions()
        }));
        assert_eq!(draft.title.expect("a title").source, DraftSource::Suggested);
    }

    #[test]
    fn a_suggested_bpm_the_page_does_not_print_is_discarded() {
        let draft = read_fields(&suggested_page(SuggestedFields {
            bpm: Some(144),
            ..no_suggestions()
        }));
        let tempo = draft.tempo.expect("the heuristic tempo");
        assert_eq!(tempo.value.bpm, Some(120));
        assert_eq!(tempo.source, DraftSource::Recognised);
    }

    #[test]
    fn suggestions_never_replace_a_field_they_left_empty() {
        let draft = read_fields(&suggested_page(no_suggestions()));
        assert_eq!(draft, read_fields(&printed_page()));
    }

    // ── Against the consumer ────────────────────────────────────────

    /// Pages as a user would photograph them, asserted against what the create
    /// form actually accepts — not against the heuristics that produced them.
    #[test]
    fn every_read_field_is_one_the_create_form_accepts() {
        let pages: Vec<(&str, PageReading)> = vec![
            ("printed lead sheet", printed_page()),
            (
                "hymnal page",
                page(vec![
                    line("Be Thou My Vision", 0.06, 0.07),
                    line("Words and Music by Traditional Irish", 0.16, 0.025),
                    line("Andante", 0.24, 0.02),
                ]),
            ),
            (
                "photocopied study",
                page(vec![
                    line("Etude Op. 10 No. 3", 0.09, 0.06),
                    line("by Frederic Chopin", 0.17, 0.03),
                    line("Lento ma non troppo  = 100", 0.25, 0.02),
                ]),
            ),
            (
                "chart with no credit at all",
                page(vec![
                    line("Blues in F", 0.07, 0.08),
                    line("F7 | Bb7 | F7 | F7", 0.30, 0.03),
                ]),
            ),
            (
                "skewed capture, title not first in reading order",
                page(vec![
                    line("Swing", 0.22, 0.02),
                    line("Music by Count Basie", 0.15, 0.03),
                    line("One O'Clock Jump", 0.05, 0.08),
                ]),
            ),
        ];

        for (name, page) in pages {
            let draft = read_fields(&page);
            let title = draft.title.as_ref().map(|f| f.value.clone());
            assert!(title.is_some(), "{name}: expected a title");

            let input = CreateItem {
                title: title.unwrap(),
                kind: ItemKind::Piece,
                composer: draft
                    .composer
                    .as_ref()
                    .map(|f| f.value.clone())
                    .or_else(|| Some("Unknown".to_string())),
                key: None,
                modality: None,
                tempo: draft.tempo.as_ref().map(|f| f.value.clone()),
                notes: None,
                tags: vec![],
            };
            validation::validate_create_item(&input)
                .unwrap_or_else(|e| panic!("{name}: the create form rejected the read: {e}"));

            if let Some(tempo) = draft.tempo.as_ref() {
                validation::validate_tempo(&tempo.value)
                    .unwrap_or_else(|e| panic!("{name}: tempo rejected: {e}"));
            }
        }
    }

    // ── The bridge wire (#846 class) ────────────────────────────────

    #[test]
    fn recognition_operation_round_trips_on_ffi_bincode_wire() {
        assert_round_trips(RecognitionOperation::ReadPage {
            photo_id: "01JB0000000000000000000000".to_string(),
        });
    }

    #[test]
    fn recognition_output_round_trips_on_ffi_bincode_wire() {
        assert_round_trips(RecognitionOutput::Page(suggested_page(SuggestedFields {
            title: Some("Autumn Leaves".to_string()),
            composer: Some("Kosma".to_string()),
            tempo_marking: Some("Moderato".to_string()),
            bpm: Some(120),
            chart_text: Some("F7 | Bb7".to_string()),
        })));
        assert_round_trips(RecognitionOutput::Unsupported);
        assert_round_trips(RecognitionOutput::Failed);
    }

    #[test]
    fn photo_draft_round_trips_on_ffi_bincode_wire() {
        assert_round_trips(read_fields(&printed_page()));
        assert_round_trips(PhotoDraft::default());
    }

    // ── Driving the app ─────────────────────────────────────────────

    mod wiring {
        use super::*;
        use crate::app::{Effect, Event, Intrada};
        use crate::domain::item::ItemEvent;
        use crate::model::{Model, PhotoRecognition, PhotoRecognitionStatus};
        use crux_core::App;

        const PHOTO: &str = "01JB0000000000000000000000";

        fn read_photo(model: &mut Model) -> crux_core::Command<Effect, Event> {
            Intrada.update(
                Event::Item(ItemEvent::ReadPhoto {
                    photo_id: PHOTO.to_string(),
                }),
                model,
            )
        }

        #[test]
        fn read_photo_requests_recognition_and_marks_the_photo_as_reading() {
            let mut model = Model::test_default();
            let mut cmd = read_photo(&mut model);

            assert!(cmd.effects().any(|e| matches!(e, Effect::Recognition(req)
            if req.operation == RecognitionOperation::ReadPage {
                photo_id: PHOTO.to_string()
            })));
            assert_eq!(
                model.photo_recognition,
                PhotoRecognition::Reading {
                    photo_id: PHOTO.to_string()
                }
            );
        }

        /// Offline-first invariant 1: recognition is on-device, end to end.
        #[test]
        fn reading_a_page_issues_no_http() {
            let mut model = Model::test_default();
            let mut cmd = read_photo(&mut model);
            assert!(!cmd.effects().any(|e| matches!(e, Effect::Http(_))));
        }

        #[test]
        fn an_unreadable_photo_id_surfaces_an_error_and_requests_nothing() {
            let mut model = Model::test_default();
            let mut cmd = Intrada.update(
                Event::Item(ItemEvent::ReadPhoto {
                    photo_id: "not-a-ulid".to_string(),
                }),
                &mut model,
            );
            assert!(model.last_error.is_some());
            assert_eq!(model.photo_recognition, PhotoRecognition::Idle);
            assert!(!cmd.effects().any(|e| matches!(e, Effect::Recognition(_))));
        }

        #[test]
        fn a_read_page_becomes_the_draft_the_form_is_filled_from() {
            let mut model = Model::test_default();
            let _ = read_photo(&mut model);
            let _ = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(printed_page())),
                &mut model,
            );

            let view = Intrada.view(&model).photo_recognition;
            assert_eq!(view.status, PhotoRecognitionStatus::Ready);
            assert_eq!(view.photo_id.as_deref(), Some(PHOTO));
            assert_eq!(
                view.draft.expect("a draft").title.expect("a title").value,
                "Autumn Leaves"
            );
        }

        /// Key decision 7, projected once in the core so no shell thresholds
        /// for itself. Deleting the `has_low_confidence` derivation leaves a
        /// blurry read looking as clean as a sharp one.
        #[test]
        fn a_weak_read_is_flagged_low_confidence_in_the_view() {
            let mut sharp = Model::test_default();
            let _ = read_photo(&mut sharp);
            let _ = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(printed_page())),
                &mut sharp,
            );
            assert!(!Intrada.view(&sharp).photo_recognition.has_low_confidence);

            let mut blurry = Model::test_default();
            let _ = read_photo(&mut blurry);
            let mut lines = printed_page().lines;
            lines[0].confidence = 0.2;
            let _ = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(page(lines))),
                &mut blurry,
            );
            assert!(Intrada.view(&blurry).photo_recognition.has_low_confidence);
        }

        /// Not an error: the photo is still saved and the user types the
        /// fields. Surfacing this as a banner would be the wrong story.
        #[test]
        fn a_device_without_recognition_is_an_outcome_not_an_error() {
            let mut model = Model::test_default();
            let _ = read_photo(&mut model);
            let _ = Intrada.update(Event::PhotoRead(RecognitionOutput::Unsupported), &mut model);

            assert_eq!(
                Intrada.view(&model).photo_recognition.status,
                PhotoRecognitionStatus::Unsupported
            );
            assert!(model.last_error.is_none());
        }

        #[test]
        fn a_failed_read_is_surfaced_as_failed_not_as_an_empty_draft() {
            let mut model = Model::test_default();
            let _ = read_photo(&mut model);
            let _ = Intrada.update(Event::PhotoRead(RecognitionOutput::Failed), &mut model);

            let view = Intrada.view(&model).photo_recognition;
            assert_eq!(view.status, PhotoRecognitionStatus::Failed);
            assert!(view.draft.is_none());
        }

        /// Deleting the `Reading` guard lets a read the user walked away from
        /// repopulate the form they have since left.
        #[test]
        fn a_late_read_is_dropped_once_the_user_has_moved_on() {
            let mut model = Model::test_default();
            let _ = read_photo(&mut model);
            let _ = Intrada.update(Event::DiscardPhotoDraft, &mut model);
            let _ = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(printed_page())),
                &mut model,
            );
            assert_eq!(model.photo_recognition, PhotoRecognition::Idle);
        }

        #[test]
        fn discarding_clears_a_draft_the_user_has_finished_with() {
            let mut model = Model::test_default();
            let _ = read_photo(&mut model);
            let _ = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(printed_page())),
                &mut model,
            );
            let _ = Intrada.update(Event::DiscardPhotoDraft, &mut model);

            let view = Intrada.view(&model).photo_recognition;
            assert_eq!(view.status, PhotoRecognitionStatus::Idle);
            assert!(view.draft.is_none());
            assert!(view.photo_id.is_none());
        }

        /// Nothing recognised is ever written without the user pressing Add
        /// (spec non-goal "no silent write").
        #[test]
        fn reading_a_page_never_writes_an_item() {
            let mut model = Model::test_default();
            let mut cmd = read_photo(&mut model);
            assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));

            let mut cmd = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(printed_page())),
                &mut model,
            );
            assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
            assert!(model.items.is_empty());
        }

        #[test]
        fn the_bridge_events_round_trip_on_ffi_bincode_wire() {
            assert_round_trips(Event::Item(ItemEvent::ReadPhoto {
                photo_id: PHOTO.to_string(),
            }));
            assert_round_trips(Event::PhotoRead(RecognitionOutput::Unsupported));
            assert_round_trips(Event::DiscardPhotoDraft);
        }

        #[test]
        fn the_view_projection_round_trips_on_ffi_bincode_wire() {
            let mut model = Model::test_default();
            let _ = read_photo(&mut model);
            let _ = Intrada.update(
                Event::PhotoRead(RecognitionOutput::Page(printed_page())),
                &mut model,
            );
            assert_round_trips(Intrada.view(&model).photo_recognition);
        }
    }
}

use std::ops::Range;

use super::section::BarRange;
use crate::validation::MAX_BPM;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotePointKind {
    Bars(BarRange),
    Tempo {
        bpm: u16,
    },
    Repetitions {
        count: u16,
        clean: bool,
        in_a_row: bool,
    },
    Section {
        name: String,
    },
}

/// A point read from a note; `span` is the byte range of the note it came
/// from, so the shell can highlight it without the note ever being rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotePoint {
    pub kind: NotePointKind,
    pub span: Range<usize>,
}

/// Below this a number after "at" is far more often a time or a count than a
/// metronome setting.
const MIN_NOTE_BPM: u16 = 20;
const MAX_REPETITIONS: u16 = 100;
const EN_DASH: char = '\u{2013}';
const CROTCHET_SIGN: char = '\u{2669}';

#[must_use]
pub fn read_note(note: &str, section_names: &[&str]) -> Vec<NotePoint> {
    let tokens = tokenise(note);
    let mut reader = Reader {
        consumed: vec![false; tokens.len()],
        tokens,
        points: Vec::new(),
    };
    reader.read_bars();
    reader.read_repetitions();
    reader.read_tempos();
    reader.read_bare_bar_ranges();
    reader.read_sections(section_names);
    let mut points = reader.points;
    points.sort_by_key(|p| p.span.start);
    points
}

// ── Tokens ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Word,
    Number,
    Symbol,
}

#[derive(Debug)]
struct Token<'a> {
    kind: TokenKind,
    text: &'a str,
    start: usize,
    end: usize,
}

impl Token<'_> {
    fn is_word(&self, word: &str) -> bool {
        self.kind == TokenKind::Word && self.text.eq_ignore_ascii_case(word)
    }

    fn is_any_word(&self, words: &[&str]) -> bool {
        words.iter().any(|w| self.is_word(w))
    }

    fn is_symbol(&self, c: char) -> bool {
        self.kind == TokenKind::Symbol && self.text.starts_with(c)
    }

    fn digits(&self) -> Option<u16> {
        match self.kind {
            TokenKind::Number => self.text.parse().ok(),
            _ => None,
        }
    }

    fn count(&self) -> Option<u16> {
        self.digits().or_else(|| number_word(self))
    }
}

fn tokenise(note: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut chars = note.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        let kind = if c.is_ascii_digit() {
            TokenKind::Number
        } else if c.is_alphabetic() {
            TokenKind::Word
        } else {
            TokenKind::Symbol
        };
        let mut end = start + c.len_utf8();
        if kind != TokenKind::Symbol {
            while let Some(&(i, next)) = chars.peek() {
                let same = match kind {
                    TokenKind::Number => next.is_ascii_digit(),
                    _ => next.is_alphabetic(),
                };
                if !same {
                    break;
                }
                end = i + next.len_utf8();
                chars.next();
            }
        }
        tokens.push(Token {
            kind,
            text: &note[start..end],
            start,
            end,
        });
    }
    tokens
}

fn number_word(token: &Token) -> Option<u16> {
    const WORDS: [&str; 20] = [
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
    ];
    let index = WORDS.iter().position(|w| token.is_word(w))?;
    u16::try_from(index + 1).ok()
}

fn is_dash(token: &Token) -> bool {
    token.is_symbol('-') || token.is_symbol(EN_DASH)
}

const RANGE_WORDS: [&str; 3] = ["to", "through", "thru"];
const REPETITION_WORDS: [&str; 6] = ["times", "reps", "repetitions", "goes", "runs", "passes"];
const NOT_A_TEMPO_AFTER: [&str; 15] = [
    "times",
    "x",
    "clean",
    "reps",
    "repetitions",
    "goes",
    "runs",
    "passes",
    "minutes",
    "mins",
    "min",
    "seconds",
    "secs",
    "bar",
    "bars",
];
const BEAT_UNITS: [&str; 9] = [
    "crotchet",
    "quaver",
    "minim",
    "semibreve",
    "semiquaver",
    "beat",
    "q",
    "h",
    "e",
];
const SECTION_CUES: [&str; 3] = ["section", "sections", "part"];

// ── Reader ──

struct Reader<'a> {
    tokens: Vec<Token<'a>>,
    consumed: Vec<bool>,
    points: Vec<NotePoint>,
}

impl Reader<'_> {
    fn token(&self, i: usize) -> Option<&Token<'_>> {
        self.tokens.get(i)
    }

    fn free(&self, i: usize) -> Option<&Token<'_>> {
        match self.consumed.get(i) {
            Some(false) => self.tokens.get(i),
            _ => None,
        }
    }

    fn push(&mut self, kind: NotePointKind, tokens: Range<usize>) {
        let (Some(first), Some(last)) = (
            self.tokens.get(tokens.start),
            self.tokens.get(tokens.end - 1),
        ) else {
            return;
        };
        let span = first.start..last.end;
        for flag in &mut self.consumed[tokens] {
            *flag = true;
        }
        self.points.push(NotePoint { kind, span });
    }

    fn bar_keyword_len(&self, i: usize) -> Option<usize> {
        let token = self.free(i)?;
        let dot_follows = self
            .token(i + 1)
            .is_some_and(|t| t.is_symbol('.') && t.start == token.end);
        if token.is_any_word(&["bar", "bars", "measure", "measures", "bb"]) {
            Some(if dot_follows { 2 } else { 1 })
        } else if token.is_any_word(&["b", "m"]) && dot_follows {
            Some(2)
        } else {
            None
        }
    }

    fn read_bars(&mut self) {
        for i in 0..self.tokens.len() {
            let Some(keyword_len) = self.bar_keyword_len(i) else {
                continue;
            };
            let first_at = i + keyword_len;
            let Some(first) = self.free(first_at).and_then(Token::count) else {
                continue;
            };
            let range_last = self
                .free(first_at + 1)
                .filter(|t| t.is_symbol('-') || t.is_symbol(EN_DASH) || t.is_any_word(&RANGE_WORDS))
                .and_then(|_| self.free(first_at + 2))
                .and_then(Token::count)
                .filter(|&last| last >= first);
            let (last, end) = match range_last {
                Some(last) => (last, first_at + 3),
                None => (first, first_at + 1),
            };
            self.push(NotePointKind::Bars(BarRange { first, last }), i..end);
        }
    }

    fn read_repetitions(&mut self) {
        for i in 0..self.tokens.len() {
            if let Some(found) = self.repetitions_from_count(i) {
                let (kind, end) = found;
                self.push(kind, i..end);
            } else if let Some(count) = self.times_sign_before_count(i) {
                self.push(
                    NotePointKind::Repetitions {
                        count,
                        clean: false,
                        in_a_row: false,
                    },
                    i..i + 2,
                );
            }
        }
    }

    fn repetitions_from_count(&self, i: usize) -> Option<(NotePointKind, usize)> {
        let count = self
            .free(i)
            .and_then(Token::count)
            .filter(|c| (1..=MAX_REPETITIONS).contains(c))?;
        if i > 0 && self.token(i - 1).is_some_and(is_dash) {
            return None;
        }
        let mut next = i + 1;
        let mut clean = false;
        let mut matched = false;
        if self.free(next).is_some_and(|t| t.is_word("clean")) {
            clean = true;
            matched = true;
            next += 1;
        }
        if self
            .free(next)
            .is_some_and(|t| t.is_word("x") || t.is_any_word(&REPETITION_WORDS))
        {
            matched = true;
            next += 1;
        }
        if !clean && matched && self.free(next).is_some_and(|t| t.is_word("clean")) {
            clean = true;
            next += 1;
        }
        let in_a_row = self.free(next).is_some_and(|t| t.is_word("in"))
            && self.free(next + 1).is_some_and(|t| t.is_word("a"))
            && self.free(next + 2).is_some_and(|t| t.is_word("row"));
        if in_a_row {
            matched = true;
            next += 3;
        }
        matched.then_some((
            NotePointKind::Repetitions {
                count,
                clean,
                in_a_row,
            },
            next,
        ))
    }

    fn times_sign_before_count(&self, i: usize) -> Option<u16> {
        let sign = self.free(i).filter(|t| t.is_word("x"))?;
        self.free(i + 1)
            .filter(|t| t.start == sign.end)
            .and_then(Token::digits)
            .filter(|c| (1..=MAX_REPETITIONS).contains(c))
    }

    fn tempo_at(&self, i: usize) -> Option<(u16, usize)> {
        let bpm = self
            .free(i)
            .and_then(Token::digits)
            .filter(|b| (MIN_NOTE_BPM..=MAX_BPM).contains(b))?;
        match self.token(i + 1) {
            Some(t) if t.is_word("bpm") => Some((bpm, i + 2)),
            Some(t)
                if t.is_any_word(&NOT_A_TEMPO_AFTER)
                    || t.is_symbol('%')
                    || t.is_symbol(':')
                    || (t.kind != TokenKind::Symbol && t.start == self.tokens[i].end) =>
            {
                None
            }
            _ => Some((bpm, i + 1)),
        }
    }

    fn tempo_cue(&self, i: usize) -> Option<(usize, usize)> {
        let token = self.free(i)?;
        if token.is_word("at") {
            return Some((i + 1, i + 1));
        }
        if token.is_symbol('@') {
            return Some((i, i + 1));
        }
        if token.is_any_word(&["up", "down"]) && self.free(i + 1).is_some_and(|t| t.is_word("to")) {
            return Some((i + 2, i + 2));
        }
        if token.is_any_word(&["tempo", "metronome"]) {
            let skip = self
                .free(i + 1)
                .is_some_and(|t| t.is_any_word(&["at", "on"]) || t.is_symbol('='));
            return Some((i, if skip { i + 2 } else { i + 1 }));
        }
        let is_unit = token.is_any_word(&BEAT_UNITS) || token.is_symbol(CROTCHET_SIGN);
        if is_unit && self.free(i + 1).is_some_and(|t| t.is_symbol('=')) {
            let dotted = i > 0 && self.free(i - 1).is_some_and(|t| t.is_word("dotted"));
            return Some((if dotted { i - 1 } else { i }, i + 2));
        }
        None
    }

    fn read_tempos(&mut self) {
        for i in 0..self.tokens.len() {
            let found = match self.tempo_cue(i) {
                Some((span_start, number_at)) => self
                    .tempo_at(number_at)
                    .map(|(bpm, end)| (bpm, span_start, end)),
                None => self
                    .tempo_at(i)
                    .filter(|&(_, end)| end == i + 2)
                    .map(|(bpm, end)| (bpm, i, end)),
            };
            let Some((bpm, span_start, mut end)) = found else {
                continue;
            };
            self.push(NotePointKind::Tempo { bpm }, span_start..end);
            while let Some((bpm, next_end)) = self.chained_tempo(end) {
                self.push(NotePointKind::Tempo { bpm }, end + 1..next_end);
                end = next_end;
            }
        }
    }

    fn chained_tempo(&self, connector_at: usize) -> Option<(u16, usize)> {
        let connector = self.free(connector_at)?;
        let joins = connector.is_any_word(&["then", "to", "and"])
            || [',', '-', EN_DASH, '/']
                .iter()
                .any(|&c| connector.is_symbol(c));
        if joins {
            self.tempo_at(connector_at + 1)
        } else {
            None
        }
    }

    fn read_bare_bar_ranges(&mut self) {
        for i in 0..self.tokens.len() {
            let first = self.free(i).and_then(Token::digits);
            let dash = self
                .free(i + 1)
                .is_some_and(|t| t.is_symbol('-') || t.is_symbol(EN_DASH));
            let last = self.free(i + 2).and_then(Token::digits);
            let (Some(first), true, Some(last)) = (first, dash, last) else {
                continue;
            };
            let joined_before = i > 0
                && self
                    .token(i - 1)
                    .is_some_and(|t| is_dash(t) || t.end == self.tokens[i].start);
            let joined_after = self.token(i + 3).is_some_and(|t| {
                t.is_symbol('-')
                    || t.is_symbol(EN_DASH)
                    || t.is_symbol('%')
                    || t.is_word("bpm")
                    || t.is_any_word(&NOT_A_TEMPO_AFTER)
                    || t.start == self.tokens[i + 2].end
            });
            if last >= first && !joined_before && !joined_after {
                self.push(NotePointKind::Bars(BarRange { first, last }), i..i + 3);
            }
        }
    }

    fn read_sections(&mut self, section_names: &[&str]) {
        let mut names: Vec<(&str, Vec<Token<'_>>)> = section_names
            .iter()
            .map(|name| (*name, tokenise(name)))
            .filter(|(_, tokens)| !tokens.is_empty())
            .collect();
        names.sort_by_key(|(_, tokens)| std::cmp::Reverse(tokens.len()));
        let mut i = 0;
        while i < self.tokens.len() {
            let found = names.iter().find_map(|(name, name_tokens)| {
                self.section_at(i, name_tokens).map(|r| (*name, r))
            });
            match found {
                Some((name, span)) => {
                    i = span.end;
                    self.push(
                        NotePointKind::Section {
                            name: name.to_string(),
                        },
                        span,
                    );
                }
                None => i += 1,
            }
        }
    }

    fn section_at(&self, i: usize, name: &[Token<'_>]) -> Option<Range<usize>> {
        let end = i + name.len();
        let matches = name.iter().enumerate().all(|(n, want)| {
            self.free(i + n).is_some_and(|got| {
                got.kind == want.kind && got.text.eq_ignore_ascii_case(want.text)
            })
        });
        if !matches {
            return None;
        }
        let is_alnum = |t: &Token| t.kind != TokenKind::Symbol;
        let glued_before = i > 0
            && self
                .token(i - 1)
                .is_some_and(|t| is_alnum(t) && t.end == self.tokens[i].start);
        let glued_after = self
            .token(end)
            .is_some_and(|t| is_alnum(t) && t.start == self.tokens[end - 1].end);
        if glued_before || glued_after {
            return None;
        }
        let cue_before = i > 0
            && self
                .free(i - 1)
                .is_some_and(|t| t.is_any_word(&SECTION_CUES));
        let cue_after = self.free(end).is_some_and(|t| t.is_any_word(&SECTION_CUES));
        let single_letter = name.len() == 1 && name[0].text.chars().count() == 1;
        if single_letter {
            let written_upper = !self.tokens[i].text.chars().any(char::is_lowercase);
            if cue_before {
                return Some(i - 1..end);
            }
            if cue_after && written_upper {
                return Some(i..end + 1);
            }
            return None;
        }
        Some(match (cue_before, cue_after) {
            (true, _) => i - 1..end,
            (false, true) => i..end + 1,
            (false, false) => i..end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bars(first: u16, last: u16) -> NotePointKind {
        NotePointKind::Bars(BarRange { first, last })
    }

    fn tempo(bpm: u16) -> NotePointKind {
        NotePointKind::Tempo { bpm }
    }

    fn reps(count: u16, clean: bool, in_a_row: bool) -> NotePointKind {
        NotePointKind::Repetitions {
            count,
            clean,
            in_a_row,
        }
    }

    fn section(name: &str) -> NotePointKind {
        NotePointKind::Section {
            name: name.to_string(),
        }
    }

    fn read<'a>(note: &'a str, sections: &[&str]) -> Vec<(NotePointKind, &'a str)> {
        read_note(note, sections)
            .into_iter()
            .map(|p| (p.kind, &note[p.span]))
            .collect()
    }

    const FORM: [&str; 3] = ["A", "B", "Coda"];

    #[test]
    fn reads_what_musicians_write() {
        let cases: Vec<(&str, Vec<(NotePointKind, &str)>)> = vec![
            (
                "left hand rushed in bar 12, got it at 84",
                vec![(bars(12, 12), "bar 12"), (tempo(84), "84")],
            ),
            (
                "bars 33 to 40 still messy",
                vec![(bars(33, 40), "bars 33 to 40")],
            ),
            (
                "coda from memory, 3 clean",
                vec![(section("Coda"), "coda"), (reps(3, true, false), "3 clean")],
            ),
            (
                "LH bar 5 at 60 then 72",
                vec![(bars(5, 5), "bar 5"), (tempo(60), "60"), (tempo(72), "72")],
            ),
            (
                "B section fingering in 21-24",
                vec![(section("B"), "B section"), (bars(21, 24), "21-24")],
            ),
            (
                "five clean in a row at crotchet = 84",
                vec![
                    (reps(5, true, true), "five clean in a row"),
                    (tempo(84), "crotchet = 84"),
                ],
            ),
            (
                "q=84, did it 3x",
                vec![(tempo(84), "q=84"), (reps(3, false, false), "3x")],
            ),
            (
                "played it three times, 84 bpm feels safe",
                vec![
                    (reps(3, false, false), "three times"),
                    (tempo(84), "84 bpm"),
                ],
            ),
            (
                "m. 12 needs work, b. 14 fine, bb. 20-22 rough",
                vec![
                    (bars(12, 12), "m. 12"),
                    (bars(14, 14), "b. 14"),
                    (bars(20, 22), "bb. 20-22"),
                ],
            ),
            (
                "dotted crotchet = 60 is plenty",
                vec![(tempo(60), "dotted crotchet = 60")],
            ),
            (
                "x3 then section A @92",
                vec![
                    (reps(3, false, false), "x3"),
                    (section("A"), "section A"),
                    (tempo(92), "@92"),
                ],
            ),
            (
                "bars 12 through 16",
                vec![(bars(12, 16), "bars 12 through 16")],
            ),
            ("pushed it up to 96", vec![(tempo(96), "96")]),
            (
                "metronome on 72 today",
                vec![(tempo(72), "metronome on 72")],
            ),
            (
                "ten in a row",
                vec![(reps(10, false, true), "ten in a row")],
            ),
            ("4 clean runs", vec![(reps(4, true, false), "4 clean runs")]),
            (
                "at 60, 66, 72",
                vec![(tempo(60), "60"), (tempo(66), "66"), (tempo(72), "72")],
            ),
        ];
        for (note, expected) in cases {
            assert_eq!(read(note, &FORM), expected, "note: {note:?}");
        }
    }

    #[test]
    fn empty_and_unremarkable_notes_read_nothing() {
        for note in [
            "",
            "   ",
            "a bit better",
            "felt good today",
            "84 felt rushed",
            "Bach is bouncy, back to basics",
            "A bit uneven",
            "a section of it was rough",
            "in A major this time",
            "a lot more relaxed, played it as a whole",
        ] {
            assert_eq!(read(note, &FORM), vec![], "note: {note:?}");
        }
    }

    #[test]
    fn a_bar_number_is_never_a_tempo() {
        assert_eq!(read("at bar 84", &[]), vec![(bars(84, 84), "bar 84")]);
        assert_eq!(read("bar 84 bpm", &[]), vec![(bars(84, 84), "bar 84")]);
    }

    #[test]
    fn number_words_count_for_bars_and_repetitions() {
        assert_eq!(
            read("bar five, then three clean", &[]),
            vec![
                (bars(5, 5), "bar five"),
                (reps(3, true, false), "three clean")
            ]
        );
    }

    #[test]
    fn a_number_word_is_not_a_tempo() {
        assert_eq!(read("got it at eighty", &[]), vec![]);
    }

    #[test]
    fn numbers_with_other_units_are_not_tempos() {
        for note in [
            "stopped at 30 minutes",
            "lesson at 21:00 tomorrow",
            "right at 90% of the time",
            "at 5 it fell apart",
        ] {
            assert_eq!(read(note, &[]), vec![], "note: {note:?}");
        }
    }

    #[test]
    fn a_backwards_range_keeps_only_the_first_bar() {
        assert_eq!(read("bars 16-12", &[]), vec![(bars(16, 16), "bars 16")]);
        assert_eq!(read("did 16-12 again", &[]), vec![]);
    }

    #[test]
    fn dates_are_not_bar_ranges() {
        assert_eq!(read("the 2026-10-04 run", &[]), vec![]);
        assert_eq!(read("12-16 times over", &[]), vec![]);
    }

    #[test]
    fn spans_are_byte_offsets_in_notes_with_non_ascii_text() {
        let note = "Coda \u{2013} bars 12\u{2013}16, \u{2669}=84 na\u{ef}ve";
        let points = read_note(note, &FORM);
        assert_eq!(
            points,
            vec![
                NotePoint {
                    kind: section("Coda"),
                    span: 0..4
                },
                NotePoint {
                    kind: bars(12, 16),
                    span: 9..21
                },
                NotePoint {
                    kind: tempo(84),
                    span: 23..29
                },
            ]
        );
    }

    #[test]
    fn single_letter_names_need_a_section_word() {
        let names = ["A", "B"];
        assert_eq!(
            read("section b sounded thin", &names),
            vec![(section("B"), "section b")]
        );
        assert_eq!(
            read("the A part is fine", &names),
            vec![(section("A"), "A part")]
        );
        assert_eq!(read("B was better", &names), vec![]);
        assert_eq!(read("2B section was messy", &names), vec![]);
        assert_eq!(read("section B2 again", &names), vec![]);
    }

    #[test]
    fn section_names_match_whole_words_case_insensitively() {
        let names = ["Main", "Coda", "Section A"];
        assert_eq!(
            read("coda 2 still shaky", &["Coda", "Coda 2"]),
            vec![(section("Coda 2"), "coda 2")]
        );
        assert_eq!(read("remain calm, codas are hard", &names), vec![]);
        assert_eq!(read("in the CODA", &names), vec![(section("Coda"), "CODA")]);
        assert_eq!(
            read("section a again", &names),
            vec![(section("Section A"), "section a")]
        );
    }

    #[test]
    fn a_section_name_is_not_read_out_of_a_bar_reference() {
        assert_eq!(read("b. 12", &["B"]), vec![(bars(12, 12), "b. 12")]);
    }
}

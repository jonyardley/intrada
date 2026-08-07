//! The criterion sentence, read back (A4). One dictated sentence *is* the gate
//! (decision 19b); tempo, keys and passes are pulled out of it and shown as
//! chips, so the form asks for nothing it was already told.
//!
//! Deliberately a small scanner rather than an LLM call (`specs/built-session.md`
//! "Resolution matching"). It reads the musician's own words — "three clean
//! passes at crotchet = 72, in all keys" — and, where it cannot, says so by
//! leaving the field `None` rather than guessing.

use crate::validation::{MAX_BPM, MAX_DRILL_KEYS, MAX_PASSES_TO_OPEN, MIN_BPM, MIN_PASSES_TO_OPEN};

/// What the sentence said, in the terms a gate is built from. A field the
/// sentence did not state is absent, never invented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCriterion {
    pub tempo_bpm: Option<u16>,
    pub keys: Vec<String>,
    pub passes_to_open: u8,
}

/// What "done" means when the sentence never says: enough reps to be a habit,
/// few enough to be one block. Read back as a chip, so it is editable, not
/// imposed.
pub const DEFAULT_PASSES: u8 = 3;

/// A key name every chromatic step, as a musician writes it. `all keys` opens
/// the whole set; the gate only ever counts how many, never which.
const ALL_KEYS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

pub fn parse_criterion(sentence: &str) -> ParsedCriterion {
    let words = tokenise(sentence);
    ParsedCriterion {
        tempo_bpm: tempo(&words),
        keys: keys(&words),
        passes_to_open: passes(&words).unwrap_or(DEFAULT_PASSES),
    }
}

/// Lowercased words, with punctuation dropped but `=`, `♩` and `,` kept as
/// their own tokens: "crotchet = 72" and "F, G" both hinge on them.
fn tokenise(sentence: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in sentence.chars() {
        if ch.is_alphanumeric() || ch == '#' {
            current.push(ch.to_ascii_lowercase());
            continue;
        }
        if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        if ch == '=' || ch == ',' || ch == '♩' {
            words.push(ch.to_string());
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// A count the sentence spoke rather than wrote. Stops at twelve: past that a
/// musician says the number.
fn spoken_number(word: &str) -> Option<u8> {
    Some(match word {
        "one" | "once" => 1,
        "two" | "twice" => 2,
        "three" | "thrice" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "eleven" => 11,
        "twelve" => 12,
        _ => return None,
    })
}

fn number(word: &str) -> Option<u16> {
    word.parse()
        .ok()
        .or_else(|| spoken_number(word).map(u16::from))
}

/// "at 72", "crotchet = 72", "♩ = 72", "72 bpm". A bare number is never a
/// tempo: "three clean" would become one.
fn tempo(words: &[String]) -> Option<u16> {
    for (index, word) in words.iter().enumerate() {
        let Some(value) = word.parse::<u16>().ok() else {
            continue;
        };
        if !(MIN_BPM..=MAX_BPM).contains(&value) {
            continue;
        }
        let follows = index
            .checked_sub(1)
            .and_then(|before| words.get(before))
            .map(String::as_str);
        let precedes = words.get(index + 1).map(String::as_str);
        // "in 12 keys" is a key count, not a tempo, however plausible the number.
        if follows == Some("in") && precedes == Some("keys") {
            continue;
        }
        if matches!(follows, Some("at" | "=" | "tempo")) || precedes == Some("bpm") {
            return Some(value);
        }
    }
    None
}

/// "three clean passes", "3 clean", "five times", "twice". The count sits
/// immediately before the word that says what is being counted.
fn passes(words: &[String]) -> Option<u8> {
    for (index, word) in words.iter().enumerate() {
        let counted = matches!(
            word.as_str(),
            "clean" | "pass" | "passes" | "time" | "times" | "rep" | "reps"
        );
        if !counted {
            continue;
        }
        let count = index
            .checked_sub(1)
            .and_then(|before| words.get(before))
            .and_then(|before| number(before));
        // Nothing countable in front of it: keep reading, and fall back to a
        // bare "twice" rather than giving up on the sentence.
        let Some(count) = count.filter(|count| *count > 0) else {
            continue;
        };
        return Some(
            u8::try_from(count)
                .unwrap_or(MAX_PASSES_TO_OPEN)
                .clamp(MIN_PASSES_TO_OPEN, MAX_PASSES_TO_OPEN),
        );
    }
    // A bare "twice" with nothing counted still says how many.
    words.iter().find_map(|word| match word.as_str() {
        "once" | "twice" | "thrice" => spoken_number(word),
        _ => None,
    })
}

/// "in F", "in B flat", "in all keys", "in 12 keys", "in F and G".
fn keys(words: &[String]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if words[index] != "in" {
            index += 1;
            continue;
        }
        // "in all keys" / "in every key" / "in 12 keys" — the whole set.
        match (
            words.get(index + 1).map(String::as_str),
            words.get(index + 2),
        ) {
            (Some("all" | "every"), _) => return ALL_KEYS.map(String::from).to_vec(),
            (Some(count), Some(noun)) if noun == "keys" => {
                if let Some(wanted) = number(count) {
                    let wanted = usize::from(wanted).clamp(1, MAX_DRILL_KEYS);
                    return ALL_KEYS[..wanted]
                        .iter()
                        .map(|key| key.to_string())
                        .collect();
                }
            }
            _ => {}
        }
        // "in the key of F", then a run of named keys joined by "and" or ",".
        let mut cursor = index + 1;
        while matches!(
            words.get(cursor).map(String::as_str),
            Some("the" | "key" | "of")
        ) {
            cursor += 1;
        }
        loop {
            let Some((key, next)) = key_at(words, cursor) else {
                break;
            };
            if !found.contains(&key) {
                found.push(key);
            }
            cursor = next;
            if matches!(words.get(cursor).map(String::as_str), Some("and" | ",")) {
                cursor += 1;
                continue;
            }
            break;
        }
        index = cursor.max(index + 1);
    }
    found.truncate(MAX_DRILL_KEYS);
    found
}

/// One key name at `at`, and where the reader gets to. "b flat" and "bb" are
/// the same key said two ways.
fn key_at(words: &[String], at: usize) -> Option<(String, usize)> {
    let word = words.get(at)?;
    let mut chars = word.chars();
    let letter = chars.next()?;
    if !matches!(letter, 'a'..='g') {
        return None;
    }
    let letter = letter.to_ascii_uppercase();
    match chars.as_str() {
        "" => match words.get(at + 1).map(String::as_str) {
            Some("flat") => Some((format!("{letter}b"), at + 2)),
            Some("sharp") => Some((format!("{letter}#"), at + 2)),
            _ => Some((letter.to_string(), at + 1)),
        },
        "b" => Some((format!("{letter}b"), at + 1)),
        "#" => Some((format!("{letter}#"), at + 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(sentence: &str) -> ParsedCriterion {
        parse_criterion(sentence)
    }

    #[test]
    fn the_worked_example_reads_back_whole() {
        // A4's own sentence.
        let parsed = parsed("Both hands together, bars 1–8, no stalls, at crotchet = 72.");
        assert_eq!(parsed.tempo_bpm, Some(72));
        assert_eq!(parsed.passes_to_open, DEFAULT_PASSES);
        assert!(parsed.keys.is_empty());
    }

    #[test]
    fn a_sentence_that_states_nothing_states_nothing() {
        let parsed = parsed("Make the intro breathe");
        assert_eq!(parsed.tempo_bpm, None, "a tempo is never invented");
        assert!(parsed.keys.is_empty());
        assert_eq!(
            parsed.passes_to_open, DEFAULT_PASSES,
            "only the count has a default, because a gate needs one"
        );
    }

    #[test]
    fn tempo_is_read_from_the_ways_a_musician_writes_it() {
        for sentence in [
            "clean at 72",
            "crotchet = 72",
            "♩ = 72",
            "72 bpm, hands together",
            "tempo 72",
        ] {
            assert_eq!(parsed(sentence).tempo_bpm, Some(72), "{sentence}");
        }
    }

    #[test]
    fn a_bare_number_is_not_a_tempo() {
        assert_eq!(
            parsed("three clean passes").tempo_bpm,
            None,
            "the count must not become the tempo"
        );
        assert_eq!(parsed("bars 1 to 8").tempo_bpm, None);
    }

    #[test]
    fn an_impossible_tempo_is_refused_rather_than_clamped() {
        assert_eq!(
            parsed("at 900").tempo_bpm,
            None,
            "a tempo the drill could not run is not a tempo the chip should claim"
        );
    }

    #[test]
    fn passes_are_read_from_digits_and_from_words() {
        assert_eq!(parsed("3 clean").passes_to_open, 3);
        assert_eq!(parsed("three clean passes").passes_to_open, 3);
        assert_eq!(parsed("five times through").passes_to_open, 5);
        assert_eq!(parsed("twice, hands apart").passes_to_open, 2);
    }

    #[test]
    fn passes_and_tempo_are_read_from_the_same_sentence() {
        let parsed = parsed("Three clean passes at 72, left hand alone");
        assert_eq!((parsed.passes_to_open, parsed.tempo_bpm), (3, Some(72)));
    }

    #[test]
    fn named_keys_are_read_and_normalised() {
        assert_eq!(parsed("clean in F").keys, vec!["F"]);
        assert_eq!(parsed("clean in B flat").keys, vec!["Bb"]);
        assert_eq!(parsed("clean in Bb").keys, vec!["Bb"]);
        assert_eq!(parsed("clean in F sharp").keys, vec!["F#"]);
        assert_eq!(parsed("clean in the key of G").keys, vec!["G"]);
    }

    #[test]
    fn a_run_of_keys_is_read_whole() {
        assert_eq!(parsed("clean in F, G and Bb").keys, vec!["F", "G", "Bb"]);
    }

    #[test]
    fn all_keys_opens_the_whole_set() {
        assert_eq!(parsed("clean in all keys").keys.len(), 12);
        assert_eq!(parsed("clean in every key").keys.len(), 12);
    }

    #[test]
    fn a_key_count_is_a_count_not_a_tempo() {
        let parsed = parsed("three clean in 12 keys");
        assert_eq!(parsed.keys.len(), 12);
        assert_eq!(parsed.tempo_bpm, None, "12 keys is not 12 bpm");
    }

    #[test]
    fn a_word_that_merely_starts_with_a_note_letter_is_not_a_key() {
        assert!(
            parsed("clean in bars 1 to 8").keys.is_empty(),
            "\"bars\" is not B, however it starts"
        );
        assert!(
            parsed("clean in").keys.is_empty(),
            "a dangling \"in\" reads nothing"
        );
    }

    #[test]
    fn a_repeated_key_is_named_once() {
        assert_eq!(parsed("clean in F and F").keys, vec!["F"]);
    }

    #[test]
    fn the_parse_never_exceeds_what_validation_will_accept() {
        let parsed = parsed("clean in all keys, 40 times at 72");
        assert!(parsed.keys.len() <= MAX_DRILL_KEYS);
        assert!(parsed.passes_to_open <= MAX_PASSES_TO_OPEN);
        assert!(parsed.passes_to_open >= MIN_PASSES_TO_OPEN);
    }
}

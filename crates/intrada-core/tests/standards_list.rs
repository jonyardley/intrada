//! The built-in jazz standards list (#2719) must hold only values a piece
//! accepts, so a pick can never fill in a field that fails.

use std::collections::HashSet;

use intrada_core::domain::Metre;
use intrada_core::validation::{validate_metre, MAX_COMPOSER, MAX_TITLE};
use intrada_core::Key;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Standard {
    title: String,
    #[serde(default)]
    also: Vec<String>,
    composers: Vec<String>,
    #[serde(deserialize_with = "present")]
    key: Option<String>,
    metre: Metre,
}

// A dropped "key" must fail, not read as one of the deliberate nulls.
fn present<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(d)
}

fn standards() -> Vec<Standard> {
    serde_json::from_str(include_str!("../data/standards.json")).expect("the list is valid JSON")
}

fn is_clean(s: &str) -> bool {
    !s.is_empty() && s.trim() == s
}

#[test]
fn every_key_parses_and_every_metre_validates() {
    for s in standards() {
        if let Some(key) = &s.key {
            assert!(Key::parse(key).is_some(), "{}: key {key:?}", s.title);
        }
        assert!(validate_metre(&s.metre).is_ok(), "{}: metre", s.title);
    }
}

#[test]
fn every_entry_fits_the_form() {
    for s in standards() {
        assert!(
            is_clean(&s.title) && s.title.chars().count() <= MAX_TITLE,
            "{:?}",
            s.title
        );
        assert!(!s.composers.is_empty(), "{}: no composer", s.title);
        assert!(
            s.composers.iter().all(|c| is_clean(c)),
            "{}: blank composer",
            s.title
        );
        let joined = s.composers.join(" and ");
        assert!(
            joined.chars().count() <= MAX_COMPOSER,
            "{}: composers too long",
            s.title
        );
        assert!(
            s.also.iter().all(|a| is_clean(a)),
            "{}: blank alternative title",
            s.title
        );
    }
}

#[test]
fn no_title_or_alternative_title_repeats() {
    let mut seen = HashSet::new();
    for s in standards() {
        for name in std::iter::once(&s.title).chain(&s.also) {
            assert!(seen.insert(name.to_lowercase()), "repeated: {name}");
        }
    }
}

//! The built-in jazz standards list (#2719) must hold only values the new
//! piece form would accept, so a pick can never fill in a field that fails.

use std::collections::HashSet;

use intrada_core::domain::Metre;
use intrada_core::validation::{validate_metre, MAX_COMPOSER, MAX_TITLE};
use intrada_core::Key;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Standard {
    title: String,
    #[serde(default)]
    also: Vec<String>,
    composers: Vec<String>,
    key: Option<String>,
    metre: String,
}

fn standards() -> Vec<Standard> {
    serde_json::from_str(include_str!("../data/standards.json")).expect("the list is valid JSON")
}

#[test]
fn every_key_parses_and_every_metre_validates() {
    for s in standards() {
        if let Some(key) = &s.key {
            assert!(Key::parse(key).is_some(), "{}: key {key:?}", s.title);
        }
        let (beats, unit) = s.metre.split_once('/').expect("beats/unit");
        let metre = Metre {
            beats: beats.parse().expect("beats"),
            unit: unit.parse().expect("unit"),
            groups: None,
        };
        assert!(
            validate_metre(&metre).is_ok(),
            "{}: metre {}",
            s.title,
            s.metre
        );
    }
}

#[test]
fn every_entry_fits_the_form() {
    for s in standards() {
        assert!(!s.title.trim().is_empty() && s.title.chars().count() <= MAX_TITLE);
        assert!(!s.composers.is_empty(), "{}: no composer", s.title);
        let joined = s.composers.join(" and ");
        assert!(joined.chars().count() <= MAX_COMPOSER, "{}", s.title);
        assert!(s.also.iter().all(|a| !a.trim().is_empty()), "{}", s.title);
    }
}

#[test]
fn titles_are_unique() {
    let mut seen = HashSet::new();
    for s in standards() {
        assert!(
            seen.insert(s.title.to_lowercase()),
            "duplicate: {}",
            s.title
        );
    }
}

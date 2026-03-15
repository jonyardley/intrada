use std::collections::HashMap;

use intrada_core::{
    MAX_ACHIEVED_TEMPO, MAX_BPM, MAX_COMPOSER, MAX_NOTES, MAX_TAG, MAX_TEMPO_MARKING, MAX_TITLE,
    MIN_ACHIEVED_TEMPO, MIN_BPM,
};

use crate::helpers::parse_tags;
use crate::types::ItemType;

/// Bundles form field values so `validate_library_form` takes two arguments
/// instead of eight.
pub struct FormData<'a> {
    pub title: &'a str,
    pub composer: &'a str,
    pub notes: &'a str,
    pub bpm_str: &'a str,
    pub tempo_marking: &'a str,
    pub tags_str: &'a str,
}

/// Unified validation for the library item form.
/// Uses limits exported by intrada-core so rules stay in sync.
pub fn validate_library_form(item_type: ItemType, data: &FormData<'_>) -> HashMap<String, String> {
    let mut errors = HashMap::new();

    // Title: required, 1..=MAX_TITLE chars
    let title = data.title.trim();
    if title.is_empty() {
        errors.insert("title".to_string(), "Title is required".to_string());
    } else if title.len() > MAX_TITLE {
        errors.insert(
            "title".to_string(),
            format!("Title must be between 1 and {MAX_TITLE} characters"),
        );
    }

    // Composer: required for Piece, optional for Exercise
    let composer = data.composer.trim();
    match item_type {
        ItemType::Piece => {
            if composer.is_empty() {
                errors.insert("composer".to_string(), "Composer is required".to_string());
            } else if composer.len() > MAX_COMPOSER {
                errors.insert(
                    "composer".to_string(),
                    format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
                );
            }
        }
        ItemType::Exercise => {
            if !composer.is_empty() && composer.len() > MAX_COMPOSER {
                errors.insert(
                    "composer".to_string(),
                    format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
                );
            }
        }
    }

    // Notes: optional, max MAX_NOTES
    let notes = data.notes.trim();
    if !notes.is_empty() && notes.len() > MAX_NOTES {
        errors.insert(
            "notes".to_string(),
            format!("Notes must not exceed {MAX_NOTES} characters"),
        );
    }

    // BPM: optional, MIN_BPM..=MAX_BPM
    let bpm_str = data.bpm_str.trim();
    if !bpm_str.is_empty() {
        match bpm_str.parse::<u16>() {
            Ok(bpm_val) if !(MIN_BPM..=MAX_BPM).contains(&bpm_val) => {
                errors.insert(
                    "bpm".to_string(),
                    format!("BPM must be between {MIN_BPM} and {MAX_BPM}"),
                );
            }
            Err(_) => {
                errors.insert(
                    "bpm".to_string(),
                    format!("BPM must be between {MIN_BPM} and {MAX_BPM}"),
                );
            }
            _ => {}
        }
    }

    // Tempo marking: optional, max MAX_TEMPO_MARKING
    let tempo_marking = data.tempo_marking.trim();
    if !tempo_marking.is_empty() && tempo_marking.len() > MAX_TEMPO_MARKING {
        errors.insert(
            "tempo_marking".to_string(),
            format!("Tempo marking must not exceed {MAX_TEMPO_MARKING} characters"),
        );
    }

    // Tags: each 1..=MAX_TAG chars
    let tags = parse_tags(data.tags_str);
    for tag in &tags {
        if tag.len() > MAX_TAG {
            errors.insert(
                "tags".to_string(),
                format!("Each tag must be between 1 and {MAX_TAG} characters"),
            );
            break;
        }
    }

    errors
}

/// Validate an achieved-tempo input string from the session summary UI.
/// Returns `Some(error_message)` if invalid, `None` if valid.
/// An empty/blank value is valid (user hasn't entered anything).
pub fn validate_achieved_tempo_input(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<u16>() {
        Ok(v) if (MIN_ACHIEVED_TEMPO..=MAX_ACHIEVED_TEMPO).contains(&v) => None,
        _ => Some(format!(
            "Tempo must be between {MIN_ACHIEVED_TEMPO} and {MAX_ACHIEVED_TEMPO}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_piece_data() -> FormData<'static> {
        FormData {
            title: "Clair de Lune",
            composer: "Claude Debussy",
            notes: "",
            bpm_str: "",
            tempo_marking: "",
            tags_str: "",
        }
    }

    fn valid_exercise_data() -> FormData<'static> {
        FormData {
            title: "Hanon No. 1",
            composer: "",
            notes: "",
            bpm_str: "",
            tempo_marking: "",
            tags_str: "",
        }
    }

    #[test]
    fn test_valid_piece_no_errors() {
        let errors = validate_library_form(ItemType::Piece, &valid_piece_data());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_exercise_no_errors() {
        let errors = validate_library_form(ItemType::Exercise, &valid_exercise_data());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_missing_title() {
        let data = FormData {
            title: "",
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("title"));
    }

    #[test]
    fn test_title_too_long() {
        let long_title = "x".repeat(MAX_TITLE + 1);
        let data = FormData {
            title: &long_title,
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("title"));
    }

    #[test]
    fn test_missing_composer_for_piece() {
        let data = FormData {
            composer: "",
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("composer"));
    }

    #[test]
    fn test_composer_optional_for_exercise() {
        let data = FormData {
            composer: "",
            ..valid_exercise_data()
        };
        let errors = validate_library_form(ItemType::Exercise, &data);
        assert!(!errors.contains_key("composer"));
    }

    #[test]
    fn test_oversized_composer() {
        let long_composer = "x".repeat(MAX_COMPOSER + 1);
        let data = FormData {
            composer: &long_composer,
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("composer"));
    }

    #[test]
    fn test_oversized_notes() {
        let long_notes = "x".repeat(MAX_NOTES + 1);
        let data = FormData {
            notes: &long_notes,
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("notes"));
    }

    #[test]
    fn test_invalid_bpm_non_numeric() {
        let data = FormData {
            bpm_str: "fast",
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("bpm"));
    }

    #[test]
    fn test_bpm_out_of_range() {
        let data = FormData {
            bpm_str: "999",
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("bpm"));
    }

    #[test]
    fn test_oversized_tempo_marking() {
        let long_marking = "x".repeat(MAX_TEMPO_MARKING + 1);
        let data = FormData {
            tempo_marking: &long_marking,
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("tempo_marking"));
    }

    #[test]
    fn test_tag_too_long() {
        let long_tag = "x".repeat(MAX_TAG + 1);
        let data = FormData {
            tags_str: &long_tag,
            ..valid_piece_data()
        };
        let errors = validate_library_form(ItemType::Piece, &data);
        assert!(errors.contains_key("tags"));
    }

    // --- validate_achieved_tempo_input tests ---

    #[test]
    fn test_achieved_tempo_empty_is_valid() {
        assert!(validate_achieved_tempo_input("").is_none());
        assert!(validate_achieved_tempo_input("  ").is_none());
    }

    #[test]
    fn test_achieved_tempo_valid_range() {
        assert!(validate_achieved_tempo_input("1").is_none());
        assert!(validate_achieved_tempo_input("120").is_none());
        assert!(validate_achieved_tempo_input("500").is_none());
    }

    #[test]
    fn test_achieved_tempo_out_of_range() {
        assert!(validate_achieved_tempo_input("0").is_some());
        assert!(validate_achieved_tempo_input("501").is_some());
    }

    #[test]
    fn test_achieved_tempo_non_numeric() {
        assert!(validate_achieved_tempo_input("fast").is_some());
        assert!(validate_achieved_tempo_input("-1").is_some());
    }
}

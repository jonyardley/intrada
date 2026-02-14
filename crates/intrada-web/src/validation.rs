use std::collections::HashMap;

use intrada_core::{
    MAX_BPM, MAX_CATEGORY, MAX_COMPOSER, MAX_NOTES, MAX_TAG, MAX_TEMPO_MARKING, MAX_TITLE, MIN_BPM,
};

use crate::helpers::parse_tags;
use crate::types::ItemType;

/// Bundles form field values so `validate_library_form` takes two arguments
/// instead of eight.
pub struct FormData<'a> {
    pub title: &'a str,
    pub composer: &'a str,
    pub category: &'a str,
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

    // Category: only validated for Exercise, optional max MAX_CATEGORY
    if item_type == ItemType::Exercise {
        let category = data.category.trim();
        if !category.is_empty() && category.len() > MAX_CATEGORY {
            errors.insert(
                "category".to_string(),
                format!("Category must be between 1 and {MAX_CATEGORY} characters"),
            );
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

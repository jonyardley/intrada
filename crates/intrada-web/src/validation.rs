use std::collections::HashMap;

use crate::helpers::parse_tags;
use crate::types::ItemType;

/// Unified validation for the library item form.
/// Applies validation rules based on `item_type`: Piece requires composer; Exercise does not.
#[allow(clippy::too_many_arguments)]
pub fn validate_library_form(
    item_type: ItemType,
    title: &str,
    composer: &str,
    category: &str,
    notes: &str,
    bpm_str: &str,
    tempo_marking: &str,
    tags_str: &str,
) -> HashMap<String, String> {
    let mut errors = HashMap::new();

    // Title: required, 1-500 chars
    let title = title.trim();
    if title.is_empty() {
        errors.insert("title".to_string(), "Title is required".to_string());
    } else if title.len() > 500 {
        errors.insert(
            "title".to_string(),
            "Title must be between 1 and 500 characters".to_string(),
        );
    }

    // Composer: required for Piece, optional for Exercise
    let composer = composer.trim();
    match item_type {
        ItemType::Piece => {
            if composer.is_empty() {
                errors.insert("composer".to_string(), "Composer is required".to_string());
            } else if composer.len() > 200 {
                errors.insert(
                    "composer".to_string(),
                    "Composer must be between 1 and 200 characters".to_string(),
                );
            }
        }
        ItemType::Exercise => {
            if !composer.is_empty() && composer.len() > 200 {
                errors.insert(
                    "composer".to_string(),
                    "Composer must be between 1 and 200 characters".to_string(),
                );
            }
        }
    }

    // Category: only validated for Exercise, optional max 100
    if item_type == ItemType::Exercise {
        let category = category.trim();
        if !category.is_empty() && category.len() > 100 {
            errors.insert(
                "category".to_string(),
                "Category must be between 1 and 100 characters".to_string(),
            );
        }
    }

    // Notes: optional, max 5000
    let notes = notes.trim();
    if !notes.is_empty() && notes.len() > 5000 {
        errors.insert(
            "notes".to_string(),
            "Notes must not exceed 5000 characters".to_string(),
        );
    }

    // BPM: optional, 1-400
    let bpm_str = bpm_str.trim();
    if !bpm_str.is_empty() {
        match bpm_str.parse::<u16>() {
            Ok(bpm_val) if !(1..=400).contains(&bpm_val) => {
                errors.insert(
                    "bpm".to_string(),
                    "BPM must be between 1 and 400".to_string(),
                );
            }
            Err(_) => {
                errors.insert(
                    "bpm".to_string(),
                    "BPM must be between 1 and 400".to_string(),
                );
            }
            _ => {}
        }
    }

    // Tempo marking: optional, max 100
    let tempo_marking = tempo_marking.trim();
    if !tempo_marking.is_empty() && tempo_marking.len() > 100 {
        errors.insert(
            "tempo_marking".to_string(),
            "Tempo marking must not exceed 100 characters".to_string(),
        );
    }

    // Tags: each 1-100 chars
    let tags = parse_tags(tags_str);
    for tag in &tags {
        if tag.len() > 100 {
            errors.insert(
                "tags".to_string(),
                "Each tag must be between 1 and 100 characters".to_string(),
            );
            break;
        }
    }

    errors
}

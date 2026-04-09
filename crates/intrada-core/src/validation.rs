use crate::domain::item::ItemKind;
use crate::domain::types::{CreateItem, Tempo, UpdateItem};
use crate::error::LibraryError;

/// Validation limits shared across shells (web, CLI).
pub const MAX_TITLE: usize = 500;
pub const MAX_COMPOSER: usize = 200;
pub const MAX_NOTES: usize = 5000;
pub const MAX_INTENTION: usize = 500;
pub const MAX_TAG: usize = 100;
pub const MAX_TEMPO_MARKING: usize = 100;
pub const MIN_BPM: u16 = 1;
pub const MAX_BPM: u16 = 400;
pub const MIN_SCORE: u8 = 1;
pub const MAX_SCORE: u8 = 5;
pub const DEFAULT_REP_TARGET: u8 = 5;
pub const MIN_REP_TARGET: u8 = 3;
pub const MAX_REP_TARGET: u8 = 10;
pub const MAX_REP_HISTORY: usize = 500;
pub const MAX_ROUTINE_NAME: usize = 200;
pub const MIN_PLANNED_DURATION_SECS: u32 = 60;
pub const MAX_PLANNED_DURATION_SECS: u32 = 3600;
pub const MIN_ACHIEVED_TEMPO: u16 = 1;
pub const MAX_ACHIEVED_TEMPO: u16 = 500;

pub fn validate_title(title: &str) -> Result<(), LibraryError> {
    if title.is_empty() || title.len() > MAX_TITLE {
        return Err(LibraryError::Validation {
            field: "title".to_string(),
            message: format!("Title must be between 1 and {MAX_TITLE} characters"),
        });
    }
    Ok(())
}

pub fn validate_create_item(input: &CreateItem) -> Result<(), LibraryError> {
    if input.title.is_empty() {
        return Err(LibraryError::Validation {
            field: "title".to_string(),
            message: "Title is required".to_string(),
        });
    }
    if input.title.len() > MAX_TITLE {
        return Err(LibraryError::Validation {
            field: "title".to_string(),
            message: format!("Title must be between 1 and {MAX_TITLE} characters"),
        });
    }
    // Composer is required for pieces, optional for exercises.
    match input.kind {
        ItemKind::Piece => {
            let composer = input.composer.as_deref().unwrap_or("");
            if composer.is_empty() {
                return Err(LibraryError::Validation {
                    field: "composer".to_string(),
                    message: "Composer is required".to_string(),
                });
            }
            if composer.len() > MAX_COMPOSER {
                return Err(LibraryError::Validation {
                    field: "composer".to_string(),
                    message: format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
                });
            }
        }
        ItemKind::Exercise => {
            if let Some(ref composer) = input.composer {
                if composer.is_empty() || composer.len() > MAX_COMPOSER {
                    return Err(LibraryError::Validation {
                        field: "composer".to_string(),
                        message: format!(
                            "Composer must be between 1 and {MAX_COMPOSER} characters"
                        ),
                    });
                }
            }
        }
    }
    if let Some(ref notes) = input.notes {
        if notes.len() > MAX_NOTES {
            return Err(LibraryError::Validation {
                field: "notes".to_string(),
                message: format!("Notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    validate_tags(&input.tags)?;
    if let Some(ref tempo) = input.tempo {
        validate_tempo(tempo)?;
    }
    Ok(())
}

pub fn validate_update_item(input: &UpdateItem) -> Result<(), LibraryError> {
    if let Some(ref title) = input.title {
        if title.is_empty() {
            return Err(LibraryError::Validation {
                field: "title".to_string(),
                message: "Title is required".to_string(),
            });
        }
        if title.len() > MAX_TITLE {
            return Err(LibraryError::Validation {
                field: "title".to_string(),
                message: format!("Title must be between 1 and {MAX_TITLE} characters"),
            });
        }
    }
    if let Some(Some(ref composer)) = input.composer {
        if composer.is_empty() || composer.len() > MAX_COMPOSER {
            return Err(LibraryError::Validation {
                field: "composer".to_string(),
                message: format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
            });
        }
    }
    if let Some(Some(ref notes)) = input.notes {
        if notes.len() > MAX_NOTES {
            return Err(LibraryError::Validation {
                field: "notes".to_string(),
                message: format!("Notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    if let Some(ref tags) = input.tags {
        validate_tags(tags)?;
    }
    if let Some(Some(ref tempo)) = input.tempo {
        validate_tempo(tempo)?;
    }
    Ok(())
}

pub fn validate_session_notes(notes: &Option<String>) -> Result<(), LibraryError> {
    if let Some(ref n) = notes {
        if n.len() > MAX_NOTES {
            return Err(LibraryError::Validation {
                field: "session_notes".to_string(),
                message: format!("Practice notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_entry_notes(notes: &Option<String>) -> Result<(), LibraryError> {
    if let Some(ref n) = notes {
        if n.len() > MAX_NOTES {
            return Err(LibraryError::Validation {
                field: "notes".to_string(),
                message: format!("Notes must not exceed {MAX_NOTES} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_entries_not_empty<T>(entries: &[T], context: &str) -> Result<(), LibraryError> {
    if entries.is_empty() {
        return Err(LibraryError::Validation {
            field: "entries".to_string(),
            message: format!("{context} must have at least one entry"),
        });
    }
    Ok(())
}

pub fn validate_tags(tags: &[String]) -> Result<(), LibraryError> {
    for tag in tags {
        if tag.is_empty() || tag.len() > MAX_TAG {
            return Err(LibraryError::Validation {
                field: "tags".to_string(),
                message: format!("Each tag must be between 1 and {MAX_TAG} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_intention(intention: &Option<String>) -> Result<(), LibraryError> {
    if let Some(ref text) = intention {
        if text.len() > MAX_INTENTION {
            return Err(LibraryError::Validation {
                field: "intention".to_string(),
                message: format!("Intention must not exceed {MAX_INTENTION} characters"),
            });
        }
    }
    Ok(())
}

pub fn validate_score(score: &Option<u8>) -> Result<(), LibraryError> {
    if let Some(s) = score {
        if !(MIN_SCORE..=MAX_SCORE).contains(s) {
            return Err(LibraryError::Validation {
                field: "score".to_string(),
                message: format!("Score must be between {MIN_SCORE} and {MAX_SCORE}"),
            });
        }
    }
    Ok(())
}

pub fn validate_rep_target(rep_target: &Option<u8>) -> Result<(), LibraryError> {
    if let Some(t) = rep_target {
        if !(MIN_REP_TARGET..=MAX_REP_TARGET).contains(t) {
            return Err(LibraryError::Validation {
                field: "rep_target".to_string(),
                message: format!(
                    "Rep target must be between {MIN_REP_TARGET} and {MAX_REP_TARGET}"
                ),
            });
        }
    }
    Ok(())
}

pub fn validate_planned_duration(planned_duration_secs: &Option<u32>) -> Result<(), LibraryError> {
    if let Some(d) = planned_duration_secs {
        if !(MIN_PLANNED_DURATION_SECS..=MAX_PLANNED_DURATION_SECS).contains(d) {
            return Err(LibraryError::Validation {
                field: "planned_duration_secs".to_string(),
                message: format!(
                    "Planned duration must be between {MIN_PLANNED_DURATION_SECS} and {MAX_PLANNED_DURATION_SECS} seconds"
                ),
            });
        }
    }
    Ok(())
}

pub fn validate_achieved_tempo(tempo: &Option<u16>) -> Result<(), LibraryError> {
    if let Some(t) = tempo {
        if !(MIN_ACHIEVED_TEMPO..=MAX_ACHIEVED_TEMPO).contains(t) {
            return Err(LibraryError::Validation {
                field: "achieved_tempo".to_string(),
                message: format!(
                    "Achieved tempo must be between {MIN_ACHIEVED_TEMPO} and {MAX_ACHIEVED_TEMPO} BPM"
                ),
            });
        }
    }
    Ok(())
}

pub fn validate_tempo(tempo: &Tempo) -> Result<(), LibraryError> {
    if tempo.marking.is_none() && tempo.bpm.is_none() {
        return Err(LibraryError::Validation {
            field: "tempo".to_string(),
            message: "Tempo must have at least a marking or BPM value".to_string(),
        });
    }
    if let Some(ref marking) = tempo.marking {
        if marking.len() > MAX_TEMPO_MARKING {
            return Err(LibraryError::Validation {
                field: "tempo".to_string(),
                message: format!("Tempo marking must not exceed {MAX_TEMPO_MARKING} characters"),
            });
        }
    }
    if let Some(bpm) = tempo.bpm {
        if !(MIN_BPM..=MAX_BPM).contains(&bpm) {
            return Err(LibraryError::Validation {
                field: "tempo".to_string(),
                message: format!("BPM must be between {MIN_BPM} and {MAX_BPM}"),
            });
        }
    }
    Ok(())
}

pub fn validate_routine_name(name: &str) -> Result<(), LibraryError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(LibraryError::Validation {
            field: "name".to_string(),
            message: "Routine name is required".to_string(),
        });
    }
    if trimmed.len() > MAX_ROUTINE_NAME {
        return Err(LibraryError::Validation {
            field: "name".to_string(),
            message: format!("Routine name must not exceed {MAX_ROUTINE_NAME} characters"),
        });
    }
    Ok(())
}

/// Validate rep field consistency: rep_count must be <= rep_target, and
/// rep_count, rep_target_reached, and rep_history all require rep_target.
pub fn validate_rep_consistency(
    rep_target: Option<u8>,
    rep_count: Option<u8>,
    rep_target_reached: Option<bool>,
    rep_history_present: bool,
) -> Result<(), LibraryError> {
    if let Some(target) = rep_target {
        if let Some(count) = rep_count {
            if count > target {
                return Err(LibraryError::Validation {
                    field: "rep_count".to_string(),
                    message: format!("rep_count ({count}) cannot exceed rep_target ({target})"),
                });
            }
        }
    } else {
        // If no target, count, reached, and history must also be absent
        if rep_count.is_some() || rep_target_reached.is_some() || rep_history_present {
            return Err(LibraryError::Validation {
                field: "rep_target".to_string(),
                message: "rep_count, rep_target_reached, and rep_history require rep_target"
                    .to_string(),
            });
        }
    }
    Ok(())
}

/// Validate a routine entry's required fields: item_id, item_title, and item_type.
pub fn validate_routine_entry_fields(item_id: &str, item_title: &str) -> Result<(), LibraryError> {
    if item_id.trim().is_empty() {
        return Err(LibraryError::Validation {
            field: "item_id".to_string(),
            message: "Entry item_id must not be empty".to_string(),
        });
    }
    if item_title.trim().is_empty() {
        return Err(LibraryError::Validation {
            field: "item_title".to_string(),
            message: "Entry item_title must not be empty".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_create_item tests (piece kind) ---

    #[test]
    fn test_valid_create_piece() {
        let input = CreateItem {
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: Some("C# minor".to_string()),
            tempo: Some(Tempo {
                marking: Some("Adagio sostenuto".to_string()),
                bpm: Some(60),
            }),
            notes: Some("First movement".to_string()),
            tags: vec!["classical".to_string(), "piano".to_string()],
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_empty_title() {
        let input = CreateItem {
            title: "".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_title_too_long() {
        let input = CreateItem {
            title: "x".repeat(501),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_no_composer() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_empty_composer() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_composer_too_long() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("x".repeat(201)),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_notes_too_long() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: Some("x".repeat(5001)),
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_notes_at_limit() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            tempo: None,
            notes: Some("x".repeat(5000)),
            tags: vec![],
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_minimal() {
        let input = CreateItem {
            title: "A".to_string(),
            kind: ItemKind::Piece,
            composer: Some("B".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        assert!(validate_create_item(&input).is_ok());
    }

    // --- validate_create_item tests (exercise kind) ---

    #[test]
    fn test_valid_create_exercise() {
        let input = CreateItem {
            title: "Scale Practice".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("Hanon".to_string()),
            key: Some("C major".to_string()),
            tempo: Some(Tempo {
                marking: Some("Moderato".to_string()),
                bpm: Some(100),
            }),
            notes: Some("Practice daily".to_string()),
            tags: vec!["technique".to_string()],
        };
        assert!(validate_create_item(&input).is_ok());
    }

    #[test]
    fn test_create_exercise_empty_title() {
        let input = CreateItem {
            title: "".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_title_too_long() {
        let input = CreateItem {
            title: "x".repeat(501),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_empty_composer() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_composer_too_long() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: Some("x".repeat(201)),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_notes_too_long() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: Some("x".repeat(5001)),
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_no_optional_fields() {
        let input = CreateItem {
            title: "Warm up".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        assert!(validate_create_item(&input).is_ok());
    }

    // --- validate_tags tests ---

    #[test]
    fn test_valid_tags() {
        let tags = vec!["classical".to_string(), "piano".to_string()];
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_empty_tag() {
        let tags = vec!["classical".to_string(), "".to_string()];
        let err = validate_tags(&tags).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tag_too_long() {
        let tags = vec!["x".repeat(101)];
        let err = validate_tags(&tags).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tag_at_limit() {
        let tags = vec!["x".repeat(100)];
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_empty_tags_vec() {
        let tags: Vec<String> = vec![];
        assert!(validate_tags(&tags).is_ok());
    }

    // --- validate_tempo tests ---

    #[test]
    fn test_valid_tempo_both_fields() {
        let tempo = Tempo {
            marking: Some("Allegro".to_string()),
            bpm: Some(120),
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_valid_tempo_marking_only() {
        let tempo = Tempo {
            marking: Some("Adagio".to_string()),
            bpm: None,
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_valid_tempo_bpm_only() {
        let tempo = Tempo {
            marking: None,
            bpm: Some(120),
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_tempo_neither_field() {
        let tempo = Tempo {
            marking: None,
            bpm: None,
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo must have at least a marking or BPM value");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_marking_too_long() {
        let tempo = Tempo {
            marking: Some("x".repeat(101)),
            bpm: None,
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo marking must not exceed 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_marking_at_limit() {
        let tempo = Tempo {
            marking: Some("x".repeat(100)),
            bpm: None,
        };
        assert!(validate_tempo(&tempo).is_ok());
    }

    #[test]
    fn test_tempo_bpm_zero() {
        let tempo = Tempo {
            marking: None,
            bpm: Some(0),
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_bpm_too_high() {
        let tempo = Tempo {
            marking: None,
            bpm: Some(401),
        };
        let err = validate_tempo(&tempo).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_tempo_bpm_at_limits() {
        let tempo_min = Tempo {
            marking: None,
            bpm: Some(1),
        };
        assert!(validate_tempo(&tempo_min).is_ok());

        let tempo_max = Tempo {
            marking: None,
            bpm: Some(400),
        };
        assert!(validate_tempo(&tempo_max).is_ok());
    }

    // --- validate_update_item tests ---

    #[test]
    fn test_valid_update_item_no_fields() {
        let input = UpdateItem::default();
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_update_item_empty_title() {
        let input = UpdateItem {
            title: Some("".to_string()),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_title_too_long() {
        let input = UpdateItem {
            title: Some("x".repeat(501)),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_empty_composer() {
        let input = UpdateItem {
            composer: Some(Some("".to_string())),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_clear_composer() {
        let input = UpdateItem {
            composer: Some(None),
            ..Default::default()
        };
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_update_item_notes_too_long() {
        let input = UpdateItem {
            notes: Some(Some("x".repeat(5001))),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_clear_notes() {
        let input = UpdateItem {
            notes: Some(None),
            ..Default::default()
        };
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_update_item_invalid_tags() {
        let input = UpdateItem {
            tags: Some(vec!["".to_string()]),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_invalid_tempo() {
        let input = UpdateItem {
            tempo: Some(Some(Tempo {
                marking: None,
                bpm: None,
            })),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo must have at least a marking or BPM value");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_clear_tempo() {
        let input = UpdateItem {
            tempo: Some(None),
            ..Default::default()
        };
        assert!(validate_update_item(&input).is_ok());
    }

    #[test]
    fn test_create_piece_with_invalid_tempo() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Bach".to_string()),
            key: None,
            tempo: Some(Tempo {
                marking: Some("x".repeat(101)),
                bpm: Some(120),
            }),
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo marking must not exceed 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_with_invalid_tags() {
        let input = CreateItem {
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Bach".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["good".to_string(), "".to_string()],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_with_invalid_tempo() {
        let input = CreateItem {
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: Some(Tempo {
                marking: None,
                bpm: Some(500),
            }),
            notes: None,
            tags: vec![],
        };
        let err = validate_create_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_invalid_tempo_bpm() {
        let input = UpdateItem {
            tempo: Some(Some(Tempo {
                marking: None,
                bpm: Some(0),
            })),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_item_tags_too_long() {
        let input = UpdateItem {
            tags: Some(vec!["x".repeat(101)]),
            ..Default::default()
        };
        let err = validate_update_item(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- validate_achieved_tempo tests ---

    #[test]
    fn test_achieved_tempo_none() {
        assert!(validate_achieved_tempo(&None).is_ok());
    }

    #[test]
    fn test_achieved_tempo_valid() {
        assert!(validate_achieved_tempo(&Some(120)).is_ok());
    }

    #[test]
    fn test_achieved_tempo_at_min() {
        assert!(validate_achieved_tempo(&Some(1)).is_ok());
    }

    #[test]
    fn test_achieved_tempo_at_max() {
        assert!(validate_achieved_tempo(&Some(500)).is_ok());
    }

    #[test]
    fn test_achieved_tempo_zero() {
        let err = validate_achieved_tempo(&Some(0)).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "achieved_tempo");
                assert_eq!(message, "Achieved tempo must be between 1 and 500 BPM");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_achieved_tempo_above_max() {
        let err = validate_achieved_tempo(&Some(501)).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "achieved_tempo");
                assert_eq!(message, "Achieved tempo must be between 1 and 500 BPM");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- validate_entries_not_empty tests ---

    #[test]
    fn test_entries_not_empty_ok() {
        assert!(validate_entries_not_empty(&[1, 2, 3], "Setlist").is_ok());
    }

    #[test]
    fn test_entries_empty_setlist() {
        let entries: Vec<i32> = vec![];
        let err = validate_entries_not_empty(&entries, "Setlist").unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "entries");
                assert_eq!(message, "Setlist must have at least one entry");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_entries_empty_routine() {
        let entries: Vec<i32> = vec![];
        let err = validate_entries_not_empty(&entries, "Routine").unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "entries");
                assert_eq!(message, "Routine must have at least one entry");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- validate_rep_consistency tests ---

    #[test]
    fn test_rep_consistency_all_none() {
        assert!(validate_rep_consistency(None, None, None, false).is_ok());
    }

    #[test]
    fn test_rep_consistency_valid() {
        assert!(validate_rep_consistency(Some(5), Some(3), Some(false), true).is_ok());
    }

    #[test]
    fn test_rep_consistency_count_at_target() {
        assert!(validate_rep_consistency(Some(5), Some(5), Some(true), true).is_ok());
    }

    #[test]
    fn test_rep_consistency_count_exceeds_target() {
        let err = validate_rep_consistency(Some(5), Some(6), None, false).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "rep_count");
                assert_eq!(message, "rep_count (6) cannot exceed rep_target (5)");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_rep_consistency_count_without_target() {
        let err = validate_rep_consistency(None, Some(3), None, false).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "rep_target");
                assert_eq!(
                    message,
                    "rep_count, rep_target_reached, and rep_history require rep_target"
                );
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_rep_consistency_reached_without_target() {
        let err = validate_rep_consistency(None, None, Some(true), false).unwrap_err();
        match err {
            LibraryError::Validation { field, .. } => {
                assert_eq!(field, "rep_target");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_rep_consistency_history_without_target() {
        let err = validate_rep_consistency(None, None, None, true).unwrap_err();
        match err {
            LibraryError::Validation { field, .. } => {
                assert_eq!(field, "rep_target");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // --- validate_routine_entry_fields tests ---

    #[test]
    fn test_routine_entry_fields_valid() {
        assert!(validate_routine_entry_fields("id1", "Sonata").is_ok());
    }

    #[test]
    fn test_routine_entry_fields_empty_item_id() {
        let err = validate_routine_entry_fields("", "Sonata").unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "item_id");
                assert_eq!(message, "Entry item_id must not be empty");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_routine_entry_fields_whitespace_item_id() {
        let err = validate_routine_entry_fields("  ", "Sonata").unwrap_err();
        match err {
            LibraryError::Validation { field, .. } => {
                assert_eq!(field, "item_id");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_routine_entry_fields_empty_item_title() {
        let err = validate_routine_entry_fields("id1", "").unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "item_title");
                assert_eq!(message, "Entry item_title must not be empty");
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

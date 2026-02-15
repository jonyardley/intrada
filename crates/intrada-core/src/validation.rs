use crate::domain::session::SetlistEntry;
use crate::domain::types::{CreateExercise, CreatePiece, Tempo, UpdateExercise, UpdatePiece};
use crate::error::LibraryError;

/// Validation limits shared across shells (web, CLI).
pub const MAX_TITLE: usize = 500;
pub const MAX_COMPOSER: usize = 200;
pub const MAX_CATEGORY: usize = 100;
pub const MAX_NOTES: usize = 5000;
pub const MAX_TAG: usize = 100;
pub const MAX_TEMPO_MARKING: usize = 100;
pub const MIN_BPM: u16 = 1;
pub const MAX_BPM: u16 = 400;

pub fn validate_create_piece(input: &CreatePiece) -> Result<(), LibraryError> {
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
    if input.composer.is_empty() {
        return Err(LibraryError::Validation {
            field: "composer".to_string(),
            message: "Composer is required".to_string(),
        });
    }
    if input.composer.len() > MAX_COMPOSER {
        return Err(LibraryError::Validation {
            field: "composer".to_string(),
            message: format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
        });
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

pub fn validate_create_exercise(input: &CreateExercise) -> Result<(), LibraryError> {
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
    if let Some(ref composer) = input.composer {
        if composer.is_empty() || composer.len() > MAX_COMPOSER {
            return Err(LibraryError::Validation {
                field: "composer".to_string(),
                message: format!("Composer must be between 1 and {MAX_COMPOSER} characters"),
            });
        }
    }
    if let Some(ref category) = input.category {
        if category.is_empty() || category.len() > MAX_CATEGORY {
            return Err(LibraryError::Validation {
                field: "category".to_string(),
                message: format!("Category must be between 1 and {MAX_CATEGORY} characters"),
            });
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

pub fn validate_update_piece(input: &UpdatePiece) -> Result<(), LibraryError> {
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
    if let Some(ref composer) = input.composer {
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

pub fn validate_update_exercise(input: &UpdateExercise) -> Result<(), LibraryError> {
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
    if let Some(Some(ref category)) = input.category {
        if category.is_empty() || category.len() > MAX_CATEGORY {
            return Err(LibraryError::Validation {
                field: "category".to_string(),
                message: format!("Category must be between 1 and {MAX_CATEGORY} characters"),
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
                message: format!("Session notes must not exceed {MAX_NOTES} characters"),
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

pub fn validate_setlist_not_empty(entries: &[SetlistEntry]) -> Result<(), LibraryError> {
    if entries.is_empty() {
        return Err(LibraryError::Validation {
            field: "entries".to_string(),
            message: "Setlist must contain at least one item".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_create_piece tests ---

    #[test]
    fn test_valid_create_piece() {
        let input = CreatePiece {
            title: "Moonlight Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: Some("C# minor".to_string()),
            tempo: Some(Tempo {
                marking: Some("Adagio sostenuto".to_string()),
                bpm: Some(60),
            }),
            notes: Some("First movement".to_string()),
            tags: vec!["classical".to_string(), "piano".to_string()],
        };
        assert!(validate_create_piece(&input).is_ok());
    }

    #[test]
    fn test_create_piece_empty_title() {
        let input = CreatePiece {
            title: "".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_piece(&input).unwrap_err();
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
        let input = CreatePiece {
            title: "x".repeat(501),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_empty_composer() {
        let input = CreatePiece {
            title: "Sonata".to_string(),
            composer: "".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_piece(&input).unwrap_err();
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
        let input = CreatePiece {
            title: "Sonata".to_string(),
            composer: "x".repeat(201),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_piece(&input).unwrap_err();
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
        let input = CreatePiece {
            title: "Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: Some("x".repeat(5001)),
            tags: vec![],
        };
        let err = validate_create_piece(&input).unwrap_err();
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
        let input = CreatePiece {
            title: "Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: Some("x".repeat(5000)),
            tags: vec![],
        };
        assert!(validate_create_piece(&input).is_ok());
    }

    #[test]
    fn test_create_piece_minimal() {
        let input = CreatePiece {
            title: "A".to_string(),
            composer: "B".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        assert!(validate_create_piece(&input).is_ok());
    }

    // --- validate_create_exercise tests ---

    #[test]
    fn test_valid_create_exercise() {
        let input = CreateExercise {
            title: "Scale Practice".to_string(),
            composer: Some("Hanon".to_string()),
            category: Some("Scales".to_string()),
            key: Some("C major".to_string()),
            tempo: Some(Tempo {
                marking: Some("Moderato".to_string()),
                bpm: Some(100),
            }),
            notes: Some("Practice daily".to_string()),
            tags: vec!["technique".to_string()],
        };
        assert!(validate_create_exercise(&input).is_ok());
    }

    #[test]
    fn test_create_exercise_empty_title() {
        let input = CreateExercise {
            title: "".to_string(),
            composer: None,
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
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
        let input = CreateExercise {
            title: "x".repeat(501),
            composer: None,
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
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
        let input = CreateExercise {
            title: "Scales".to_string(),
            composer: Some("".to_string()),
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
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
        let input = CreateExercise {
            title: "Scales".to_string(),
            composer: Some("x".repeat(201)),
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_empty_category() {
        let input = CreateExercise {
            title: "Scales".to_string(),
            composer: None,
            category: Some("".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "category");
                assert_eq!(message, "Category must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_category_too_long() {
        let input = CreateExercise {
            title: "Scales".to_string(),
            composer: None,
            category: Some("x".repeat(101)),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "category");
                assert_eq!(message, "Category must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_exercise_notes_too_long() {
        let input = CreateExercise {
            title: "Scales".to_string(),
            composer: None,
            category: None,
            key: None,
            tempo: None,
            notes: Some("x".repeat(5001)),
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
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
        let input = CreateExercise {
            title: "Warm up".to_string(),
            composer: None,
            category: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        };
        assert!(validate_create_exercise(&input).is_ok());
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

    // --- validate_update_piece tests ---

    #[test]
    fn test_valid_update_piece_no_fields() {
        let input = UpdatePiece::default();
        assert!(validate_update_piece(&input).is_ok());
    }

    #[test]
    fn test_update_piece_empty_title() {
        let input = UpdatePiece {
            title: Some("".to_string()),
            ..Default::default()
        };
        let err = validate_update_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_piece_title_too_long() {
        let input = UpdatePiece {
            title: Some("x".repeat(501)),
            ..Default::default()
        };
        let err = validate_update_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title must be between 1 and 500 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_piece_empty_composer() {
        let input = UpdatePiece {
            composer: Some("".to_string()),
            ..Default::default()
        };
        let err = validate_update_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_piece_notes_too_long() {
        let input = UpdatePiece {
            notes: Some(Some("x".repeat(5001))),
            ..Default::default()
        };
        let err = validate_update_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_piece_clear_notes() {
        let input = UpdatePiece {
            notes: Some(None),
            ..Default::default()
        };
        assert!(validate_update_piece(&input).is_ok());
    }

    #[test]
    fn test_update_piece_invalid_tags() {
        let input = UpdatePiece {
            tags: Some(vec!["".to_string()]),
            ..Default::default()
        };
        let err = validate_update_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_piece_invalid_tempo() {
        let input = UpdatePiece {
            tempo: Some(Some(Tempo {
                marking: None,
                bpm: None,
            })),
            ..Default::default()
        };
        let err = validate_update_piece(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "Tempo must have at least a marking or BPM value");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_piece_clear_tempo() {
        let input = UpdatePiece {
            tempo: Some(None),
            ..Default::default()
        };
        assert!(validate_update_piece(&input).is_ok());
    }

    // --- validate_update_exercise tests ---

    #[test]
    fn test_valid_update_exercise_no_fields() {
        let input = UpdateExercise::default();
        assert!(validate_update_exercise(&input).is_ok());
    }

    #[test]
    fn test_update_exercise_empty_title() {
        let input = UpdateExercise {
            title: Some("".to_string()),
            ..Default::default()
        };
        let err = validate_update_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert_eq!(message, "Title is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_exercise_empty_composer() {
        let input = UpdateExercise {
            composer: Some(Some("".to_string())),
            ..Default::default()
        };
        let err = validate_update_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "composer");
                assert_eq!(message, "Composer must be between 1 and 200 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_exercise_clear_composer() {
        let input = UpdateExercise {
            composer: Some(None),
            ..Default::default()
        };
        assert!(validate_update_exercise(&input).is_ok());
    }

    #[test]
    fn test_update_exercise_empty_category() {
        let input = UpdateExercise {
            category: Some(Some("".to_string())),
            ..Default::default()
        };
        let err = validate_update_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "category");
                assert_eq!(message, "Category must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_exercise_clear_category() {
        let input = UpdateExercise {
            category: Some(None),
            ..Default::default()
        };
        assert!(validate_update_exercise(&input).is_ok());
    }

    #[test]
    fn test_update_exercise_notes_too_long() {
        let input = UpdateExercise {
            notes: Some(Some("x".repeat(5001))),
            ..Default::default()
        };
        let err = validate_update_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "notes");
                assert_eq!(message, "Notes must not exceed 5000 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_exercise_invalid_tags() {
        let input = UpdateExercise {
            tags: Some(vec!["x".repeat(101)]),
            ..Default::default()
        };
        let err = validate_update_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tags");
                assert_eq!(message, "Each tag must be between 1 and 100 characters");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_exercise_invalid_tempo() {
        let input = UpdateExercise {
            tempo: Some(Some(Tempo {
                marking: None,
                bpm: Some(0),
            })),
            ..Default::default()
        };
        let err = validate_update_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_create_piece_with_invalid_tempo() {
        let input = CreatePiece {
            title: "Sonata".to_string(),
            composer: "Bach".to_string(),
            key: None,
            tempo: Some(Tempo {
                marking: Some("x".repeat(101)),
                bpm: Some(120),
            }),
            notes: None,
            tags: vec![],
        };
        let err = validate_create_piece(&input).unwrap_err();
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
        let input = CreatePiece {
            title: "Sonata".to_string(),
            composer: "Bach".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["good".to_string(), "".to_string()],
        };
        let err = validate_create_piece(&input).unwrap_err();
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
        let input = CreateExercise {
            title: "Scales".to_string(),
            composer: None,
            category: None,
            key: None,
            tempo: Some(Tempo {
                marking: None,
                bpm: Some(500),
            }),
            notes: None,
            tags: vec![],
        };
        let err = validate_create_exercise(&input).unwrap_err();
        match err {
            LibraryError::Validation { field, message } => {
                assert_eq!(field, "tempo");
                assert_eq!(message, "BPM must be between 1 and 400");
            }
            _ => panic!("Expected Validation error"),
        }
    }
}

use std::collections::HashSet;

use intrada_core::{LibraryItemView, Tempo};

/// Parse comma-separated tags string into Vec<String>.
/// Trims whitespace, filters empty entries.
pub fn parse_tags(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse tempo marking + BPM string into Option<Tempo>.
/// Returns None if both are empty.
pub fn parse_tempo(marking: &str, bpm_str: &str) -> Option<Tempo> {
    let marking_opt = {
        let m = marking.trim();
        if m.is_empty() {
            None
        } else {
            Some(m.to_string())
        }
    };
    let bpm_opt = bpm_str.trim().parse::<u16>().ok();
    Tempo::from_parts(marking_opt, bpm_opt)
}

/// Parse a formatted tempo display string back into (marking, bpm_str) for edit form pre-population.
/// Handles: "Allegro (132 BPM)", "Allegro", "132 BPM", None
pub fn parse_tempo_display(tempo: &Option<String>) -> (String, String) {
    let Some(t) = tempo else {
        return (String::new(), String::new());
    };

    // Pattern: "Marking (BPM_NUMBER BPM)"
    if let Some(paren_start) = t.rfind('(') {
        let marking = t[..paren_start].trim().to_string();
        let bpm_part = &t[paren_start + 1..];
        let bpm_str = bpm_part
            .trim_end_matches(')')
            .trim()
            .trim_end_matches("BPM")
            .trim()
            .to_string();
        return (marking, bpm_str);
    }

    // Pattern: "NUMBER BPM"
    if t.ends_with("BPM") {
        let bpm_str = t.trim_end_matches("BPM").trim().to_string();
        return (String::new(), bpm_str);
    }

    // Just a marking
    (t.clone(), String::new())
}

/// Collect all unique tags from library items, deduplicated case-insensitively
/// (preserving first-seen casing), sorted alphabetically.
pub fn unique_tags(items: &[LibraryItemView]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        for tag in &item.tags {
            let lower = tag.to_lowercase();
            if seen.insert(lower) {
                result.push(tag.clone());
            }
        }
    }
    result.sort_by_key(|a| a.to_lowercase());
    result
}

/// Extract unique composers from library items.
/// For pieces: extract `subtitle` (which is the composer).
/// For exercises: extract `subtitle` only when `category` is `None`
/// (otherwise subtitle is the category, not composer).
/// Deduplicates case-insensitively (preserving first-seen casing),
/// filters out empty strings, and sorts alphabetically.
pub fn unique_composers(items: &[LibraryItemView]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        // Pieces: subtitle is the composer. Exercises: subtitle is
        // composer only when category is None (otherwise subtitle = category).
        if item.item_type != "piece" && item.category.is_some() {
            continue;
        }
        let composer = &item.subtitle;
        if composer.is_empty() {
            continue;
        }
        let lower = composer.to_lowercase();
        if seen.insert(lower) {
            result.push(composer.clone());
        }
    }
    result.sort_by_key(|a| a.to_lowercase());
    result
}

/// Filter suggestions by input text. Case-insensitive matching with prefix
/// matches ranked before substring matches. Excludes values in `exclude` list.
/// Returns up to `max` results.
pub fn filter_suggestions(
    suggestions: &[String],
    input: &str,
    exclude: &[String],
    max: usize,
) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let exclude_lower: HashSet<String> = exclude.iter().map(|s| s.to_lowercase()).collect();

    let mut prefix_matches = Vec::new();
    let mut substring_matches = Vec::new();

    for suggestion in suggestions {
        let lower = suggestion.to_lowercase();
        if exclude_lower.contains(&lower) {
            continue;
        }
        if lower.starts_with(&input_lower) {
            prefix_matches.push(suggestion.clone());
        } else if lower.contains(&input_lower) {
            substring_matches.push(suggestion.clone());
        }
    }

    let mut result = prefix_matches;
    result.extend(substring_matches);
    result.truncate(max);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // T005: parse_tags tests
    #[test]
    fn test_parse_tags_empty_string() {
        assert!(parse_tags("").is_empty());
    }

    #[test]
    fn test_parse_tags_single() {
        assert_eq!(parse_tags("classical"), vec!["classical"]);
    }

    #[test]
    fn test_parse_tags_multiple() {
        assert_eq!(
            parse_tags("classical, piano, romantic"),
            vec!["classical", "piano", "romantic"]
        );
    }

    #[test]
    fn test_parse_tags_whitespace_trimming() {
        assert_eq!(
            parse_tags("  jazz ,  blues , funk  "),
            vec!["jazz", "blues", "funk"]
        );
    }

    #[test]
    fn test_parse_tags_trailing_comma() {
        assert_eq!(parse_tags("rock,"), vec!["rock"]);
    }

    #[test]
    fn test_parse_tags_empty_entries_filtered() {
        assert_eq!(parse_tags("a,,b,,,c"), vec!["a", "b", "c"]);
    }

    // T006: parse_tempo tests
    #[test]
    fn test_parse_tempo_both_empty() {
        assert!(parse_tempo("", "").is_none());
    }

    #[test]
    fn test_parse_tempo_marking_only() {
        let tempo = parse_tempo("Allegro", "").unwrap();
        assert_eq!(tempo.marking, Some("Allegro".to_string()));
        assert_eq!(tempo.bpm, None);
    }

    #[test]
    fn test_parse_tempo_bpm_only() {
        let tempo = parse_tempo("", "120").unwrap();
        assert_eq!(tempo.marking, None);
        assert_eq!(tempo.bpm, Some(120));
    }

    #[test]
    fn test_parse_tempo_both_present() {
        let tempo = parse_tempo("Allegro", "132").unwrap();
        assert_eq!(tempo.marking, Some("Allegro".to_string()));
        assert_eq!(tempo.bpm, Some(132));
    }

    #[test]
    fn test_parse_tempo_invalid_bpm() {
        // Invalid BPM string is treated as no BPM
        let tempo = parse_tempo("Andante", "fast");
        // "Andante" alone should still produce Some with marking
        assert!(tempo.is_some());
        let t = tempo.unwrap();
        assert_eq!(t.marking, Some("Andante".to_string()));
        assert_eq!(t.bpm, None);
    }

    // T007: parse_tempo_display tests
    #[test]
    fn test_parse_tempo_display_none() {
        let (marking, bpm) = parse_tempo_display(&None);
        assert!(marking.is_empty());
        assert!(bpm.is_empty());
    }

    #[test]
    fn test_parse_tempo_display_marking_only() {
        let (marking, bpm) = parse_tempo_display(&Some("Allegro".to_string()));
        assert_eq!(marking, "Allegro");
        assert!(bpm.is_empty());
    }

    #[test]
    fn test_parse_tempo_display_bpm_only() {
        let (marking, bpm) = parse_tempo_display(&Some("132 BPM".to_string()));
        assert!(marking.is_empty());
        assert_eq!(bpm, "132");
    }

    #[test]
    fn test_parse_tempo_display_full_format() {
        let (marking, bpm) = parse_tempo_display(&Some("Allegro (132 BPM)".to_string()));
        assert_eq!(marking, "Allegro");
        assert_eq!(bpm, "132");
    }

    // unique_tags tests
    fn make_item(
        item_type: &str,
        subtitle: &str,
        category: Option<&str>,
        tags: &[&str],
    ) -> LibraryItemView {
        LibraryItemView {
            id: String::new(),
            item_type: item_type.to_string(),
            title: String::new(),
            subtitle: subtitle.to_string(),
            category: category.map(|s| s.to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: String::new(),
            updated_at: String::new(),
            practice: None,
        }
    }

    #[test]
    fn test_unique_tags_empty() {
        assert!(unique_tags(&[]).is_empty());
    }

    #[test]
    fn test_unique_tags_dedup_case_insensitive() {
        let items = vec![
            make_item("piece", "", None, &["Classical", "jazz"]),
            make_item("piece", "", None, &["classical", "Jazz"]),
        ];
        let tags = unique_tags(&items);
        assert_eq!(tags, vec!["Classical", "jazz"]);
    }

    #[test]
    fn test_unique_tags_sorted_alphabetically() {
        let items = vec![
            make_item("piece", "", None, &["piano", "baroque"]),
            make_item("piece", "", None, &["classical"]),
        ];
        let tags = unique_tags(&items);
        assert_eq!(tags, vec!["baroque", "classical", "piano"]);
    }

    // unique_composers tests
    #[test]
    fn test_unique_composers_empty() {
        assert!(unique_composers(&[]).is_empty());
    }

    #[test]
    fn test_unique_composers_from_pieces() {
        let items = vec![
            make_item("piece", "Bach", None, &[]),
            make_item("piece", "Mozart", None, &[]),
            make_item("piece", "Bach", None, &[]), // duplicate
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Bach", "Mozart"]);
    }

    #[test]
    fn test_unique_composers_exercise_with_category_excluded() {
        let items = vec![
            make_item("exercise", "Technique", Some("Technique"), &[]),
            make_item("exercise", "Bach", None, &[]),
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Bach"]);
    }

    #[test]
    fn test_unique_composers_empty_subtitle_excluded() {
        let items = vec![
            make_item("piece", "", None, &[]),
            make_item("piece", "Mozart", None, &[]),
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Mozart"]);
    }

    #[test]
    fn test_unique_composers_case_insensitive_dedup() {
        let items = vec![
            make_item("piece", "Bach", None, &[]),
            make_item("piece", "bach", None, &[]),
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Bach"]);
    }

    // filter_suggestions tests
    #[test]
    fn test_filter_suggestions_empty_input() {
        let suggestions = vec!["Classical".to_string()];
        let result = filter_suggestions(&suggestions, "", &[], 8);
        // Empty input matches everything as prefix
        assert_eq!(result, vec!["Classical"]);
    }

    #[test]
    fn test_filter_suggestions_prefix_before_substring() {
        let suggestions = vec![
            "baroque".to_string(),
            "classical".to_string(),
            "neoclassical".to_string(),
        ];
        let result = filter_suggestions(&suggestions, "class", &[], 8);
        // "classical" is prefix match, "neoclassical" is substring
        assert_eq!(result, vec!["classical", "neoclassical"]);
    }

    #[test]
    fn test_filter_suggestions_case_insensitive() {
        let suggestions = vec!["Classical".to_string(), "Jazz".to_string()];
        let result = filter_suggestions(&suggestions, "cla", &[], 8);
        assert_eq!(result, vec!["Classical"]);
    }

    #[test]
    fn test_filter_suggestions_excludes() {
        let suggestions = vec![
            "classical".to_string(),
            "jazz".to_string(),
            "piano".to_string(),
        ];
        let exclude = vec!["jazz".to_string()];
        let result = filter_suggestions(&suggestions, "", &exclude, 8);
        assert_eq!(result, vec!["classical", "piano"]);
    }

    #[test]
    fn test_filter_suggestions_max_limit() {
        let suggestions: Vec<String> = (0..20).map(|i| format!("tag{i}")).collect();
        let result = filter_suggestions(&suggestions, "tag", &[], 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filter_suggestions_exclude_case_insensitive() {
        let suggestions = vec!["Classical".to_string(), "Jazz".to_string()];
        let exclude = vec!["CLASSICAL".to_string()];
        let result = filter_suggestions(&suggestions, "", &exclude, 8);
        assert_eq!(result, vec!["Jazz"]);
    }
}

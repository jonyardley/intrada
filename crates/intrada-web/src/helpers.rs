use intrada_core::Tempo;

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
}

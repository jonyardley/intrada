use intrada_core::domain::types::Tempo;

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
    let marking = marking.trim();
    let bpm_str = bpm_str.trim();

    if marking.is_empty() && bpm_str.is_empty() {
        return None;
    }

    let marking_opt = if marking.is_empty() {
        None
    } else {
        Some(marking.to_string())
    };

    let bpm_opt = if bpm_str.is_empty() {
        None
    } else {
        bpm_str.parse::<u16>().ok()
    };

    Some(Tempo {
        marking: marking_opt,
        bpm: bpm_opt,
    })
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

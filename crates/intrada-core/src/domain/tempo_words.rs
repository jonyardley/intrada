use super::types::Tempo;

const MINIM: u8 = 2;
const QUAVER: u8 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempoWords {
    pub text: String,
    pub spoken: String,
}

impl TempoWords {
    #[must_use]
    pub fn split(words: Option<Self>) -> (Option<String>, Option<String>) {
        words.map_or((None, None), |w| (Some(w.text), Some(w.spoken)))
    }
}

/// The click's readout. It names the note value the metre counts in, since
/// `♩ = 168` in 7/8 would be a lie the ear catches (T19). The minim is spelt
/// out: the bundled faces have no glyph for it.
#[must_use]
pub fn click_readout(bpm: u16, unit: u8) -> TempoWords {
    let (glyph, spoken) = match unit {
        MINIM => ("minim", format!("{bpm} minim beats per minute")),
        QUAVER => ("♪", format!("{bpm} quaver beats per minute")),
        _ => ("♩", format!("{bpm} beats per minute")),
    };
    TempoWords {
        text: format!("{glyph} = {bpm}"),
        spoken,
    }
}

/// An item's declared tempo: "Allegro · ♩ = 132". `None` when it has neither.
#[must_use]
pub fn tempo_line(tempo: Option<&Tempo>) -> Option<TempoWords> {
    let tempo = tempo?;
    let marking = tempo.marking.as_deref().filter(|m| !m.is_empty());
    let join = |bpm_part: Option<String>, separator: &str| {
        let parts: Vec<String> = marking
            .map(str::to_owned)
            .into_iter()
            .chain(bpm_part)
            .collect();
        (!parts.is_empty()).then(|| parts.join(separator))
    };
    Some(TempoWords {
        text: join(tempo.bpm.map(|bpm| format!("♩ = {bpm}")), " · ")?,
        spoken: join(tempo.bpm.map(|bpm| format!("{bpm} beats per minute")), ", ")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_click_names_the_note_value_it_counts_in() {
        let cases = [
            (60, 2, "minim = 60", "60 minim beats per minute"),
            (72, 4, "♩ = 72", "72 beats per minute"),
            (168, 8, "♪ = 168", "168 quaver beats per minute"),
            (100, 16, "♩ = 100", "100 beats per minute"),
        ];
        for (bpm, unit, text, spoken) in cases {
            let words = click_readout(bpm, unit);
            assert_eq!(words.text, text, "unit {unit}");
            assert_eq!(words.spoken, spoken, "unit {unit}");
        }
    }

    #[test]
    fn an_items_tempo_line_joins_its_marking_and_tempo() {
        let tempo = |marking: Option<&str>, bpm| Tempo {
            marking: marking.map(str::to_owned),
            bpm,
        };
        let cases = [
            (
                tempo(Some("Lent"), Some(70)),
                "Lent · ♩ = 70",
                "Lent, 70 beats per minute",
            ),
            (tempo(None, Some(96)), "♩ = 96", "96 beats per minute"),
            (tempo(Some("Largo"), None), "Largo", "Largo"),
            (tempo(Some(""), Some(80)), "♩ = 80", "80 beats per minute"),
        ];
        for (tempo, text, spoken) in cases {
            let words = tempo_line(Some(&tempo));
            assert_eq!(words.as_ref().map(|w| w.text.as_str()), Some(text));
            assert_eq!(words.as_ref().map(|w| w.spoken.as_str()), Some(spoken));
        }
    }

    #[test]
    fn no_tempo_or_an_empty_one_has_no_line() {
        assert_eq!(tempo_line(None), None);
        let empty = Tempo {
            marking: Some(String::new()),
            bpm: None,
        };
        assert_eq!(tempo_line(Some(&empty)), None);
    }
}

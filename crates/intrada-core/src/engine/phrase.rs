use super::note::Onset;

/// One step of a target phrase. Matched by pitch class, so any octave counts —
/// the spike's crude-on-purpose stance, inherited from the Swift gate drill.
/// `beat_offset_milli` is thousandths of a beat from the phrase start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseStep {
    pub pitch_classes: Vec<u8>,
    pub beat_offset_milli: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetPhrase {
    pub steps: Vec<PhraseStep>,
}

impl TargetPhrase {
    pub fn crotchets(pitch_classes: &[u8]) -> Self {
        Self {
            steps: pitch_classes
                .iter()
                .enumerate()
                .map(|(index, class)| PhraseStep {
                    pitch_classes: vec![*class],
                    beat_offset_milli: index as i64 * 1000,
                })
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Octave-tolerant, but **width-exact**: a one-note step needs a one-class
    /// onset. Matching on "contains the pitch class" alone let a five-note
    /// freeplay voicing satisfy 7 of the 8 gate-phrase steps (take 01).
    /// Octave doublings still match, since they add no distinct class.
    pub fn matches(&self, step: usize, onset: &Onset) -> bool {
        self.steps.get(step).is_some_and(|expected| {
            onset.distinct_pitch_classes() == expected.pitch_classes.len()
                && expected
                    .pitch_classes
                    .iter()
                    .all(|class| onset.has_pitch_class(*class))
        })
    }

    /// The phrase's own note density, which is what pause and abandon
    /// thresholds must scale against — a grid beat is the wrong unit for a
    /// phrase in quavers.
    pub fn step_spacing_milli(&self) -> i64 {
        let mut gaps: Vec<i64> = self
            .steps
            .windows(2)
            .map(|pair| pair[1].beat_offset_milli - pair[0].beat_offset_milli)
            .filter(|gap| *gap > 0)
            .collect();
        if gaps.is_empty() {
            return 1000;
        }
        gaps.sort_unstable();
        gaps[gaps.len() / 2]
    }

    /// How many onsets from `from_onset` match consecutively if the player is
    /// taken to be at `from_step`. The disambiguator for restart-versus-resume,
    /// which no timing signal can settle (see the findings note).
    pub fn consecutive_match_run(&self, onsets: &[Onset], from_step: usize) -> usize {
        onsets
            .iter()
            .enumerate()
            .take_while(|(offset, onset)| self.matches(from_step + offset, onset))
            .count()
    }
}

use serde::{Deserialize, Serialize};

/// What the input path can support, per design decision 7 (transport-tiered
/// scoring): never issue a precision verdict the input can't carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportTier {
    /// Wired MIDI, ±1-3ms.
    Wired,
    /// Bluetooth MIDI, ±10-20ms connection-interval jitter.
    Bluetooth,
    /// Microphone plus transcription, ±20ms at best.
    Acoustic,
}

/// One MIDI note event, timed from the click anchor. **Signed**: a note may
/// legitimately precede the anchor (anticipating beat 1, or playing during the
/// count-in), which `u64` cannot express.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteEvent {
    pub pitch: u8,
    pub velocity: u8,
    pub on: bool,
    pub t_us: i64,
}

/// Near-simultaneous note-ons collapsed into one musical event. A chord or a
/// rolled voicing is one attempt step, not five.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Onset {
    pub t_us: i64,
    pub pitches: Vec<u8>,
    pub peak_velocity: u8,
}

impl Onset {
    pub fn has_pitch_class(&self, pitch_class: u8) -> bool {
        self.pitches.iter().any(|p| p % 12 == pitch_class % 12)
    }

    pub fn distinct_pitch_classes(&self) -> usize {
        let mut mask: u16 = 0;
        for pitch in &self.pitches {
            mask |= 1 << (pitch % 12);
        }
        mask.count_ones() as usize
    }
}

/// Groups note-ons whose onsets fall within `window_us` of the group's first
/// note. Measured spread on real rolled voicings is ~34ms (take 01), so the
/// default window is 50ms — see the findings note for why that ceiling collides
/// with fast subdivisions.
pub fn cluster_onsets(events: &[NoteEvent], window_us: i64) -> Vec<Onset> {
    let mut ons: Vec<&NoteEvent> = events.iter().filter(|e| e.on).collect();
    ons.sort_by_key(|e| e.t_us);

    let mut clusters: Vec<Onset> = Vec::new();
    for event in ons {
        match clusters.last_mut() {
            Some(current) if event.t_us - current.t_us <= window_us => {
                current.pitches.push(event.pitch);
                current.peak_velocity = current.peak_velocity.max(event.velocity);
            }
            _ => clusters.push(Onset {
                t_us: event.t_us,
                pitches: vec![event.pitch],
                peak_velocity: event.velocity,
            }),
        }
    }
    clusters
}

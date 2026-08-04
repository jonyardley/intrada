use serde::{Deserialize, Serialize};

const US_PER_MINUTE_MILLI: i64 = 60_000_000_000;

/// The click grid, in microseconds from the anchor. `anchor_us` is bar 1 beat 1
/// — the first beat *after* the count-in, not engine start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClickGrid {
    pub bpm_milli: u32,
    pub beats_per_bar: u8,
    pub count_in_beats: u8,
}

/// A grid position. `beat_index` is 0-based from the anchor and goes negative
/// during the count-in; `bar`/`beat` are the musician's 1-based counting, so a
/// count-in beat reads as bar 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeatRef {
    pub beat_index: i64,
    pub bar: i64,
    pub beat: u8,
    pub offset_us: i64,
}

impl ClickGrid {
    pub fn new(bpm_milli: u32, beats_per_bar: u8, count_in_beats: u8) -> Self {
        Self {
            bpm_milli,
            beats_per_bar,
            count_in_beats,
        }
    }

    pub fn us_per_beat(&self) -> i64 {
        US_PER_MINUTE_MILLI / i64::from(self.bpm_milli.max(1))
    }

    /// Beats are expressed in thousandths throughout so the engine never
    /// carries a float across a boundary (engine spec §6: integers where an
    /// integer will do).
    pub fn beats_milli_to_us(&self, beats_milli: i64) -> i64 {
        beats_milli * self.us_per_beat() / 1000
    }

    pub fn beat_time_us(&self, beat_index: i64) -> i64 {
        beat_index * self.us_per_beat()
    }

    pub fn nearest_beat(&self, t_us: i64) -> BeatRef {
        let per_beat = self.us_per_beat();
        let beat_index = (t_us * 2 + per_beat.signum() * per_beat).div_euclid(per_beat * 2);
        let bpb = i64::from(self.beats_per_bar.max(1));
        BeatRef {
            beat_index,
            bar: beat_index.div_euclid(bpb) + 1,
            beat: (beat_index.rem_euclid(bpb) + 1) as u8,
            offset_us: t_us - self.beat_time_us(beat_index),
        }
    }

    /// Start of the count-in region. Anything at or after
    /// `count_in_cutoff_us` is body playing, including a note that anticipates
    /// beat 1 (two of the five real takes do this by 20-50ms).
    pub fn count_in_start_us(&self) -> i64 {
        -self.beat_time_us(i64::from(self.count_in_beats))
    }

    pub fn count_in_cutoff_us(&self) -> i64 {
        -self.us_per_beat() / 2
    }
}

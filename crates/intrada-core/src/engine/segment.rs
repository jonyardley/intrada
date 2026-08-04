use std::ops::Range;

use super::grid::ClickGrid;
use super::note::{cluster_onsets, NoteEvent, Onset};
use super::phrase::TargetPhrase;

/// Tunable thresholds. Everything time-shaped is a ratio of the target
/// phrase's own step spacing, never an absolute millisecond count, so the same
/// config works at 60 and 200 bpm and for quaver phrases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentConfig {
    pub chord_window_us: i64,
    /// Silence beyond this multiple of step spacing is a pause.
    pub pause_ratio_milli: i64,
    /// Silence beyond this multiple ends the attempt unresumed.
    pub abandon_ratio_milli: i64,
    pub max_consecutive_deviations: u32,
    /// How many steps must match before a start is believed. One matching note
    /// is not an attempt — in take 03 the scale run crosses the phrase's first
    /// note five times, and each crossing opened a spurious one-step attempt.
    pub min_start_run_steps: usize,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            chord_window_us: 50_000,
            pause_ratio_milli: 1_750,
            abandon_ratio_milli: 4_000,
            max_consecutive_deviations: 2,
            min_start_run_steps: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pause {
    pub after_step: u32,
    pub gap_us: i64,
}

/// Offsets of matched onsets against where the phrase wanted them. Per design
/// decision 6 the spread is the error and the mean is the feel, so both are
/// reported and neither is graded here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimingStats {
    pub count: u32,
    pub mean_offset_us: i64,
    pub stddev_offset_us: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptOutcome {
    /// Every step matched, in order.
    Completed,
    /// The player went back to step 0 before finishing.
    Restarted { at_step: u32 },
    /// Played a wrong note and stopped there.
    Collapsed { at_step: u32 },
    /// Left the phrase and kept playing something else.
    Diverged { at_step: u32 },
    /// Stopped or fell silent mid-phrase while still playing it correctly.
    /// Not evidence of failure — see the findings note.
    Abandoned { at_step: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub span: Range<usize>,
    pub start_us: i64,
    pub end_us: i64,
    pub matched_steps: u32,
    pub outcome: AttemptOutcome,
    pub deviation_onsets: Vec<usize>,
    pub pauses: Vec<Pause>,
    pub timing: TimingStats,
}

impl Attempt {
    /// A paused attempt's phrase timing is arithmetic about a phrase the player
    /// didn't play continuously, so it is never a scoring input.
    pub fn timing_is_scorable(&self) -> bool {
        self.outcome == AttemptOutcome::Completed && self.pauses.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segmentation {
    pub onsets: Vec<Onset>,
    /// Onsets played during the count-in, excluded from every attempt.
    pub count_in: Range<usize>,
    pub attempts: Vec<Attempt>,
    /// Playing attributed to no attempt: noodling before, between, or after.
    pub unattributed: Vec<Range<usize>>,
}

impl Segmentation {
    pub fn completed(&self) -> impl Iterator<Item = &Attempt> {
        self.attempts
            .iter()
            .filter(|a| a.outcome == AttemptOutcome::Completed)
    }
}

struct InProgress {
    first_onset: usize,
    first_t_us: i64,
    anchor_beat: i64,
    step: usize,
    last_onset: usize,
    last_t_us: i64,
    consecutive_deviations: u32,
    ended_on_deviation: bool,
    deviation_onsets: Vec<usize>,
    pauses: Vec<Pause>,
    offsets_us: Vec<i64>,
}

/// Splits a note stream into attempts against a target phrase. With no target
/// there are no attempts by construction — an attempt is defined by what it
/// was an attempt *at*.
pub fn segment(
    events: &[NoteEvent],
    grid: &ClickGrid,
    target: Option<&TargetPhrase>,
    config: &SegmentConfig,
) -> Segmentation {
    let onsets = cluster_onsets(events, config.chord_window_us);
    let cutoff = grid.count_in_cutoff_us();
    let body_start = onsets.partition_point(|o| o.t_us < cutoff);

    let Some(target) = target.filter(|t| !t.is_empty()) else {
        return Segmentation {
            count_in: 0..body_start,
            unattributed: span_if_nonempty(body_start..onsets.len()),
            attempts: Vec::new(),
            onsets,
        };
    };

    let spacing_milli = target.step_spacing_milli();
    let pause_us = grid.beats_milli_to_us(spacing_milli * config.pause_ratio_milli / 1000);
    let abandon_us = grid.beats_milli_to_us(spacing_milli * config.abandon_ratio_milli / 1000);

    let mut attempts: Vec<Attempt> = Vec::new();
    let mut unattributed: Vec<Range<usize>> = Vec::new();
    let mut current: Option<InProgress> = None;
    let mut noodle_start: Option<usize> = None;

    for index in body_start..onsets.len() {
        let onset = &onsets[index];
        let tail = contiguous_tail(&onsets[index..], abandon_us);

        if let Some(active) = current.take() {
            if onset.t_us - active.last_t_us > abandon_us {
                attempts.push(close(active, AttemptKind::Abandoned));
            } else {
                current = Some(active);
            }
        }

        let Some(active) = current.as_mut() else {
            if starts_here(tail, target, config) {
                current = Some(open(index, onset, grid, target));
                if let Some(start) = noodle_start.take() {
                    unattributed.extend(span_if_nonempty(start..index));
                }
            } else if noodle_start.is_none() {
                noodle_start = Some(index);
            }
            continue;
        };

        let gap_us = onset.t_us - active.last_t_us;
        let continues = target.matches(active.step, onset);
        let restarts = active.step > 0
            && starts_here(tail, target, config)
            && (!continues || prefers_restart(tail, target, active.step));

        if restarts {
            let active = current.take().expect("matched as_mut above");
            attempts.push(close(active, AttemptKind::Restarted));
            current = Some(open(index, onset, grid, target));
            continue;
        }

        if continues {
            if gap_us > pause_us {
                active.pauses.push(Pause {
                    after_step: active.step as u32,
                    gap_us,
                });
            }
            let expected_us = grid.beat_time_us(active.anchor_beat)
                + grid.beats_milli_to_us(target.steps[active.step].beat_offset_milli);
            active.offsets_us.push(onset.t_us - expected_us);
            active.step += 1;
            active.consecutive_deviations = 0;
            active.last_onset = index;
            active.last_t_us = onset.t_us;
            active.ended_on_deviation = false;

            if active.step == target.len() {
                let active = current.take().expect("matched as_mut above");
                attempts.push(close(active, AttemptKind::Completed));
            }
            continue;
        }

        active.consecutive_deviations += 1;
        active.deviation_onsets.push(index);
        active.last_onset = index;
        active.last_t_us = onset.t_us;
        active.ended_on_deviation = true;
        if active.consecutive_deviations >= config.max_consecutive_deviations {
            let active = current.take().expect("matched as_mut above");
            // Whether a divergence is noodling or a collapse is decided by what
            // follows it, not by the wrong notes themselves.
            let kept_playing = onsets
                .get(index + 1)
                .is_some_and(|next| next.t_us - onset.t_us <= abandon_us);
            let kind = if kept_playing {
                AttemptKind::Diverged
            } else {
                AttemptKind::Collapsed
            };
            attempts.push(close(active, kind));
            noodle_start = Some(index + 1);
        }
    }

    if let Some(active) = current.take() {
        let kind = if active.ended_on_deviation {
            AttemptKind::Collapsed
        } else {
            AttemptKind::Abandoned
        };
        attempts.push(close(active, kind));
    }
    if let Some(start) = noodle_start {
        unattributed.extend(span_if_nonempty(start..onsets.len()));
    }

    Segmentation {
        count_in: 0..body_start,
        attempts,
        unattributed,
        onsets,
    }
}

/// Spans of playing separated by silence longer than `gap_us`. The only
/// segmentation available with no target phrase — useful for logging off-piste
/// playing (decision 16), never for scoring.
pub fn rest_spans(onsets: &[Onset], gap_us: i64) -> Vec<Range<usize>> {
    let mut spans: Vec<Range<usize>> = Vec::new();
    for (index, onset) in onsets.iter().enumerate() {
        match spans.last_mut() {
            Some(current) if onset.t_us - onsets[current.end - 1].t_us <= gap_us => {
                current.end = index + 1;
            }
            _ => spans.push(index..index + 1),
        }
    }
    spans
}

enum AttemptKind {
    Completed,
    Restarted,
    Collapsed,
    Diverged,
    Abandoned,
}

fn open(index: usize, onset: &Onset, grid: &ClickGrid, target: &TargetPhrase) -> InProgress {
    // The anchor is where step 0 *should* have fallen, so an upbeat phrase
    // (non-zero first offset) doesn't skew every offset in the attempt.
    let step_zero_us = grid.beats_milli_to_us(target.steps[0].beat_offset_milli);
    let anchor_beat = grid.nearest_beat(onset.t_us - step_zero_us).beat_index;
    let expected_us = grid.beat_time_us(anchor_beat) + step_zero_us;
    InProgress {
        first_onset: index,
        first_t_us: onset.t_us,
        anchor_beat,
        step: 1,
        last_onset: index,
        last_t_us: onset.t_us,
        consecutive_deviations: 0,
        ended_on_deviation: false,
        deviation_onsets: Vec::new(),
        pauses: Vec::new(),
        offsets_us: vec![onset.t_us - expected_us],
    }
}

fn close(active: InProgress, kind: AttemptKind) -> Attempt {
    let at_step = active.step as u32;
    let outcome = match kind {
        AttemptKind::Completed => AttemptOutcome::Completed,
        AttemptKind::Restarted => AttemptOutcome::Restarted { at_step },
        AttemptKind::Collapsed => AttemptOutcome::Collapsed { at_step },
        AttemptKind::Diverged => AttemptOutcome::Diverged { at_step },
        AttemptKind::Abandoned => AttemptOutcome::Abandoned { at_step },
    };
    Attempt {
        span: active.first_onset..active.last_onset + 1,
        start_us: active.first_t_us,
        end_us: active.last_t_us,
        matched_steps: at_step,
        outcome,
        timing: timing_stats(&active.offsets_us),
        deviation_onsets: active.deviation_onsets,
        pauses: active.pauses,
    }
}

/// The tail up to the first silence long enough to end an attempt, so a
/// confirming run can't be assembled from notes a minute apart.
fn contiguous_tail(tail: &[Onset], abandon_us: i64) -> &[Onset] {
    let end = tail
        .windows(2)
        .position(|pair| pair[1].t_us - pair[0].t_us > abandon_us)
        .map_or(tail.len(), |index| index + 1);
    &tail[..end]
}

fn starts_here(tail: &[Onset], target: &TargetPhrase, config: &SegmentConfig) -> bool {
    let required = config.min_start_run_steps.clamp(1, target.len());
    target.consecutive_match_run(tail, 0) >= required
}

/// True when the tail matches the phrase better read as a fresh start than as a
/// continuation. Both readings are timing-identical, so only content decides —
/// and a continuation that explains everything available is never overruled,
/// since its run is capped by the steps remaining while a restart's is not.
fn prefers_restart(tail: &[Onset], target: &TargetPhrase, step: usize) -> bool {
    let continuing = target.consecutive_match_run(tail, step);
    if step + continuing >= target.len() || continuing >= tail.len() {
        return false;
    }
    target.consecutive_match_run(tail, 0) > continuing
}

fn timing_stats(offsets_us: &[i64]) -> TimingStats {
    if offsets_us.is_empty() {
        return TimingStats::default();
    }
    let count = offsets_us.len() as i64;
    let mean = offsets_us.iter().sum::<i64>() / count;
    // i128: squaring a whole-beat offset overflows i64 at slow tempi.
    let variance = offsets_us
        .iter()
        .map(|offset| {
            let delta = i128::from(offset - mean);
            delta * delta
        })
        .sum::<i128>()
        / i128::from(count);
    TimingStats {
        count: count as u32,
        mean_offset_us: mean,
        stddev_offset_us: (variance as f64).sqrt() as i64,
    }
}

fn span_if_nonempty(range: Range<usize>) -> Vec<Range<usize>> {
    if range.is_empty() {
        Vec::new()
    } else {
        vec![range]
    }
}

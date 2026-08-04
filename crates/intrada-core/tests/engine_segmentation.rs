//! Segmentation against the five real MIDI takes recorded in PR 2
//! (`tests/fixtures/midi_takes/README.md`). Each take is a named case from
//! `docs/rebuild-review.md` §6; the findings note
//! (`docs/segmentation-findings.md`) is written from what these assert.

mod midi_take;

use intrada_core::engine::{
    cluster_onsets, rest_spans, segment, AttemptOutcome, ClickGrid, NoteEvent, SegmentConfig,
    TargetPhrase,
};
use midi_take::MidiTake;

/// The gate drill phrase from the Swift spike: Dm7 shell then G7 shell,
/// arpeggiated one crotchet per beat. F C F C, B F B F.
fn gate_phrase() -> TargetPhrase {
    TargetPhrase::crotchets(&[5, 0, 5, 0, 11, 5, 11, 5])
}

#[test]
fn take_02_pause_mid_phrase_is_one_completed_attempt_with_a_pause() {
    let take = MidiTake::load("take-02-paused-mid-phrase-bluetooth.jsonl");
    let result = segment(
        &take.events,
        &take.grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    assert_eq!(result.attempts.len(), 1, "{:?}", result.attempts);
    let attempt = &result.attempts[0];
    assert_eq!(attempt.outcome, AttemptOutcome::Completed);
    assert_eq!(attempt.matched_steps, 8);
    assert_eq!(attempt.pauses.len(), 1);
    assert_eq!(attempt.pauses[0].after_step, 6);
    assert!(attempt.pauses[0].gap_us > 1_800_000);
    assert!(
        !attempt.timing_is_scorable(),
        "a paused attempt is complete but its phrase timing is not a scoring input"
    );
    assert!(result.unattributed.is_empty());
}

#[test]
fn take_04_restart_is_an_abandoned_attempt_then_a_clean_one() {
    let take = MidiTake::load("take-04-restart-bluetooth.jsonl");
    let result = segment(
        &take.events,
        &take.grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    assert_eq!(result.attempts.len(), 2, "{:?}", result.attempts);
    assert_eq!(
        result.attempts[0].outcome,
        AttemptOutcome::Restarted { at_step: 6 }
    );
    assert_eq!(result.attempts[1].outcome, AttemptOutcome::Completed);
    assert!(result.attempts[1].pauses.is_empty());
    assert!(result.attempts[1].timing_is_scorable());
    assert!(result.unattributed.is_empty());
}

/// Takes 02 and 04 open with the same six notes and pause for the same ~1.9s.
/// Only what follows the silence separates them, which is the spike's headline
/// finding: no gap threshold can tell a resume from a restart.
#[test]
fn takes_02_and_04_have_indistinguishable_pauses() {
    let paused = MidiTake::load("take-02-paused-mid-phrase-bluetooth.jsonl");
    let restarted = MidiTake::load("take-04-restart-bluetooth.jsonl");
    let config = SegmentConfig::default();

    let paused_gap = largest_gap_us(&paused.events, config.chord_window_us);
    let restarted_gap = largest_gap_us(&restarted.events, config.chord_window_us);
    assert!(
        (paused_gap - restarted_gap).abs() < 100_000,
        "gaps differ by {}us — the takes were meant to be timing-alike",
        (paused_gap - restarted_gap).abs()
    );

    let phrase = gate_phrase();
    assert_eq!(
        segment(&paused.events, &paused.grid, Some(&phrase), &config)
            .attempts
            .len(),
        1
    );
    assert_eq!(
        segment(&restarted.events, &restarted.grid, Some(&phrase), &config)
            .attempts
            .len(),
        2
    );
}

#[test]
fn take_03_noodling_ends_the_attempt_and_is_left_unattributed() {
    let take = MidiTake::load("take-03-noodle-after-start-bluetooth.jsonl");
    let result = segment(
        &take.events,
        &take.grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    assert_eq!(result.attempts.len(), 1, "{:?}", result.attempts);
    let attempt = &result.attempts[0];
    assert_eq!(attempt.outcome, AttemptOutcome::Diverged { at_step: 5 });
    assert_eq!(attempt.matched_steps, 5);

    let noodled: usize = result.unattributed.iter().map(|span| span.len()).sum();
    assert!(
        noodled > 15,
        "the scale run should be rejected wholesale, got {noodled} onsets"
    );
    assert!(result.completed().count() == 0);
}

#[test]
fn take_05_collapse_keeps_the_wrong_note_and_scores_nothing() {
    let take = MidiTake::load("take-05-collapse-bluetooth.jsonl");
    let result = segment(
        &take.events,
        &take.grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    assert_eq!(result.attempts.len(), 1, "{:?}", result.attempts);
    let attempt = &result.attempts[0];
    assert_eq!(attempt.outcome, AttemptOutcome::Collapsed { at_step: 5 });
    assert_eq!(attempt.deviation_onsets.len(), 1);
    assert!(!attempt.timing_is_scorable());
}

#[test]
fn take_01_freeplay_yields_no_attempts_without_a_target() {
    let take = MidiTake::load("take-01-freeplay-mixed-bluetooth.jsonl");
    let config = SegmentConfig::default();
    let result = segment(&take.events, &take.grid, None, &config);

    assert!(result.attempts.is_empty());
    assert_eq!(result.unattributed.len(), 1);
    assert_eq!(result.unattributed[0].len(), result.onsets.len());

    let spans = rest_spans(&result.onsets, take.grid.beats_milli_to_us(1_750));
    assert!(
        spans.len() > 1,
        "freeplay should still split into rest-separated spans for off-piste logging"
    );
}

/// The same freeplay take against a phrase the player never attempted.
/// Prescribing a phrase does not conjure attempts out of noodling — and this is
/// the case that caught both false-positive sources: a lone matching note
/// opening an attempt, and a wide voicing satisfying a single-note step.
#[test]
fn take_01_freeplay_yields_no_attempts_against_the_gate_phrase() {
    let take = MidiTake::load("take-01-freeplay-mixed-bluetooth.jsonl");
    let result = segment(
        &take.events,
        &take.grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    assert!(result.attempts.is_empty(), "{:?}", result.attempts);
    assert_eq!(result.unattributed[0].len(), result.onsets.len());
}

#[test]
fn a_voicing_never_satisfies_a_single_note_step_but_an_octave_does() {
    let phrase = TargetPhrase::crotchets(&[5]);
    let single = cluster_onsets(&[note_on(53, 0)], 50_000);
    let octaves = cluster_onsets(&[note_on(53, 0), note_on(65, 10_000)], 50_000);
    let voicing = cluster_onsets(&[note_on(53, 0), note_on(60, 10_000)], 50_000);

    assert!(phrase.matches(0, &single[0]));
    assert!(phrase.matches(0, &octaves[0]));
    assert!(!phrase.matches(0, &voicing[0]));
}

#[test]
fn chord_voicings_collapse_into_single_onsets() {
    let take = MidiTake::load("take-01-freeplay-mixed-bluetooth.jsonl");
    let note_ons = take.events.iter().filter(|e| e.on).count();
    let onsets = cluster_onsets(&take.events, 50_000);

    assert!(
        onsets.len() < note_ons,
        "the take's rolled voicings should collapse"
    );
    let widest = onsets.iter().map(|o| o.pitches.len()).max().unwrap_or(0);
    assert!(widest >= 5, "five-note voicings are present, got {widest}");
}

/// No real take contains count-in playing — the Swift recorder anchors bar 1
/// beat 1 after the count-in, and Jon waited. Synthetic, and flagged as such
/// in the findings note.
#[test]
fn count_in_playing_is_excluded_from_attempts() {
    let grid = ClickGrid::new(92_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let mut events = vec![
        note_on(60, -3 * per_beat),
        note_on(62, -2 * per_beat),
        note_on(64, -per_beat),
    ];
    for (step, class) in [5u8, 0, 5, 0, 11, 5, 11, 5].iter().enumerate() {
        events.push(note_on(48 + class, step as i64 * per_beat));
    }

    let result = segment(
        &events,
        &grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    assert_eq!(result.count_in.len(), 3);
    assert_eq!(result.attempts.len(), 1);
    assert_eq!(result.attempts[0].outcome, AttemptOutcome::Completed);
    assert_eq!(result.attempts[0].span.start, 3);
}

/// Both real takes that anticipate beat 1 do so by tens of milliseconds. That
/// note belongs to the attempt, not the count-in — and it needs a signed
/// timestamp to exist at all.
#[test]
fn a_note_anticipating_beat_one_belongs_to_the_attempt() {
    for name in [
        "take-01-freeplay-mixed-bluetooth.jsonl",
        "take-03-noodle-after-start-bluetooth.jsonl",
    ] {
        let take = MidiTake::load(name);
        assert!(
            take.events.iter().any(|e| e.on && e.t_us < 0),
            "{name} was recorded with a note before the anchor"
        );
        let result = segment(
            &take.events,
            &take.grid,
            Some(&gate_phrase()),
            &SegmentConfig::default(),
        );
        assert!(result.count_in.is_empty(), "{name}");
    }
}

/// Design decision 6: a stable offset is feel, spread is the error. The steady
/// on-the-beat passage in take 01 lays back consistently.
#[test]
fn steady_playing_reads_as_consistent_lay_back_not_error() {
    let take = MidiTake::load("take-01-freeplay-mixed-bluetooth.jsonl");
    let phrase = TargetPhrase::crotchets(&[9, 9, 9, 9, 9, 9, 9, 9]);
    let result = segment(
        &take.events,
        &take.grid,
        Some(&phrase),
        &SegmentConfig::default(),
    );

    let attempt = result
        .completed()
        .next()
        .expect("the repeated-A passage completes an eight-crotchet target");
    assert!(
        attempt.timing.mean_offset_us < -5_000,
        "expected a lay-back, got mean {}us",
        attempt.timing.mean_offset_us
    );
    assert!(
        attempt.timing.stddev_offset_us < attempt.timing.mean_offset_us.abs(),
        "spread {}us should be smaller than the offset it sits around",
        attempt.timing.stddev_offset_us
    );
}

fn note_on(pitch: u8, t_us: i64) -> NoteEvent {
    NoteEvent {
        pitch,
        velocity: 64,
        on: true,
        t_us,
    }
}

fn largest_gap_us(events: &[NoteEvent], chord_window_us: i64) -> i64 {
    let onsets = cluster_onsets(events, chord_window_us);
    onsets
        .windows(2)
        .map(|pair| pair[1].t_us - pair[0].t_us)
        .max()
        .unwrap_or(0)
}

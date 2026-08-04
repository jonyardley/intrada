//! Segmentation against the five real MIDI takes recorded in PR 2
//! (`tests/fixtures/midi_takes/README.md`). Each take is a named case from
//! `docs/rebuild-review.md` §6; the findings note
//! (`docs/segmentation-findings.md`) is written from what these assert.

mod midi_take;

use intrada_core::engine::{
    cluster_onsets, rest_spans, segment, AttemptOutcome, ClickGrid, NoteEvent, PhraseStep,
    SegmentConfig, TargetPhrase,
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
fn take_04_restart_is_a_closed_attempt_then_a_clean_one() {
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
    assert_eq!(
        spans.len(),
        4,
        "freeplay still splits into rest-separated spans for off-piste logging"
    );
    assert_eq!(spans.last().expect("four spans").end, result.onsets.len());
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

/// Design decision 6: the mean offset is feel, the spread is the error. This
/// pins the one clean repetition in the fixture set — the figures the findings
/// note quotes as the evidence that decision 6 is measurable over Bluetooth.
/// Negative is ahead of the click, so this rep pushes; it does not lay back.
#[test]
fn the_clean_repetition_pushes_consistently() {
    let take = MidiTake::load("take-04-restart-bluetooth.jsonl");
    let result = segment(
        &take.events,
        &take.grid,
        Some(&gate_phrase()),
        &SegmentConfig::default(),
    );

    let attempt = result.completed().next().expect("take 04's second rep");
    assert_eq!(attempt.timing.count, 8);
    assert!(
        (-30_000..-18_000).contains(&attempt.timing.mean_offset_us),
        "expected a consistent push around -24ms, got {}us",
        attempt.timing.mean_offset_us
    );
    assert!(
        attempt.timing.stddev_offset_us < 20_000,
        "expected a spread under 20ms, got {}us",
        attempt.timing.stddev_offset_us
    );
    assert!(
        attempt.timing.stddev_offset_us < attempt.timing.mean_offset_us.abs(),
        "the spread should be smaller than the offset it sits around"
    );
}

/// A repeated pitch class makes every continuation also look like a fresh
/// start. Two clean back-to-back reps of a one-note phrase must read as two
/// attempts, not as a restart on every note.
#[test]
fn a_phrase_of_one_repeated_pitch_class_does_not_restart_on_every_note() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let phrase = TargetPhrase::crotchets(&[0, 0, 0, 0]);
    let events: Vec<NoteEvent> = (0..8).map(|beat| note_on(60, beat * per_beat)).collect();

    let result = segment(&events, &grid, Some(&phrase), &SegmentConfig::default());

    assert_eq!(result.attempts.len(), 2, "{:?}", result.attempts);
    assert!(result
        .attempts
        .iter()
        .all(|a| a.outcome == AttemptOutcome::Completed));
}

/// The outcome says what the player did next, so a divergence that stops is a
/// collapse and one that keeps going is noodling — the wrong notes alone don't
/// tell them apart.
#[test]
fn stopping_and_carrying_on_are_different_outcomes() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let phrase = TargetPhrase::crotchets(&[0, 2, 4, 5]);
    let opening = [note_on(60, 0), note_on(62, per_beat)];

    let stopped: Vec<NoteEvent> = opening
        .iter()
        .copied()
        .chain([note_on(66, 2 * per_beat), note_on(68, 3 * per_beat)])
        .collect();
    let carried_on: Vec<NoteEvent> = stopped
        .iter()
        .copied()
        .chain((4..8).map(|beat| note_on(70, beat * per_beat)))
        .collect();

    let config = SegmentConfig::default();
    let stopped = segment(&stopped, &grid, Some(&phrase), &config);
    let carried_on = segment(&carried_on, &grid, Some(&phrase), &config);

    assert_eq!(
        stopped.attempts[0].outcome,
        AttemptOutcome::Collapsed { at_step: 2 }
    );
    assert_eq!(
        carried_on.attempts[0].outcome,
        AttemptOutcome::Diverged { at_step: 2 }
    );
}

/// Silence past the abandon threshold closes the attempt where it stood, and
/// what follows is judged on its own.
#[test]
fn silence_abandons_an_attempt_that_was_going_correctly() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let phrase = TargetPhrase::crotchets(&[0, 2, 4, 5, 7]);
    let events = vec![
        note_on(60, 0),
        note_on(62, per_beat),
        note_on(64, 2 * per_beat),
        note_on(60, 12 * per_beat),
        note_on(62, 13 * per_beat),
    ];

    let result = segment(&events, &grid, Some(&phrase), &SegmentConfig::default());

    assert_eq!(result.attempts.len(), 2, "{:?}", result.attempts);
    assert_eq!(
        result.attempts[0].outcome,
        AttemptOutcome::Abandoned { at_step: 3 }
    );
    assert_eq!(
        result.attempts[1].outcome,
        AttemptOutcome::Abandoned { at_step: 2 }
    );
    assert!(!result.attempts[0].timing_is_scorable());
}

/// Two notes an age apart are not a confirming run, however well they match.
#[test]
fn a_start_is_not_confirmed_across_a_long_silence() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let phrase = TargetPhrase::crotchets(&[0, 2, 4, 5]);
    let events = vec![note_on(60, 0), note_on(62, 120 * per_beat)];

    let result = segment(&events, &grid, Some(&phrase), &SegmentConfig::default());

    assert!(result.attempts.is_empty(), "{:?}", result.attempts);
    assert_eq!(
        result
            .unattributed
            .iter()
            .map(|span| span.len())
            .sum::<usize>(),
        2
    );
}

/// The thresholds are ratios of the phrase's own step spacing, so a quaver
/// phrase tolerates half the silence a crotchet phrase does. Nothing else in
/// the suite exercises a non-crotchet spacing.
#[test]
fn thresholds_scale_with_the_phrases_own_note_density() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let quavers = TargetPhrase {
        steps: [0u8, 2, 4, 5]
            .iter()
            .enumerate()
            .map(|(index, class)| PhraseStep {
                pitch_classes: vec![*class],
                beat_offset_milli: index as i64 * 500,
            })
            .collect(),
    };
    assert_eq!(quavers.step_spacing_milli(), 500);

    let gap_us = per_beat; // one beat: fine for crotchets, a pause for quavers
    let events = vec![
        note_on(60, 0),
        note_on(62, per_beat / 2),
        note_on(64, per_beat / 2 + gap_us),
        note_on(65, per_beat + gap_us),
    ];

    let quaver_result = segment(&events, &grid, Some(&quavers), &SegmentConfig::default());
    assert_eq!(quaver_result.attempts[0].pauses.len(), 1);

    let crotchet_events: Vec<NoteEvent> = [0u8, 2, 4, 5]
        .iter()
        .enumerate()
        .map(|(index, class)| note_on(60 + class, index as i64 * per_beat))
        .collect();
    let crotchet_result = segment(
        &crotchet_events,
        &grid,
        Some(&TargetPhrase::crotchets(&[0, 2, 4, 5])),
        &SegmentConfig::default(),
    );
    assert!(crotchet_result.attempts[0].pauses.is_empty());
}

/// An upbeat phrase is anchored by where its first note *should* have fallen,
/// not by the note itself, or every offset in the attempt inherits the skew.
#[test]
fn an_upbeat_phrase_is_not_offset_by_its_own_first_step() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let per_beat = grid.us_per_beat();
    let phrase = TargetPhrase {
        steps: [0u8, 2, 4, 5]
            .iter()
            .enumerate()
            .map(|(index, class)| PhraseStep {
                pitch_classes: vec![*class],
                beat_offset_milli: 500 + index as i64 * 1000,
            })
            .collect(),
    };
    let events: Vec<NoteEvent> = [0u8, 2, 4, 5]
        .iter()
        .enumerate()
        .map(|(index, class)| note_on(60 + class, per_beat / 2 + index as i64 * per_beat))
        .collect();

    let result = segment(&events, &grid, Some(&phrase), &SegmentConfig::default());

    let attempt = result
        .completed()
        .next()
        .expect("played exactly as written");
    assert_eq!(attempt.timing.mean_offset_us, 0);
    assert_eq!(attempt.timing.stddev_offset_us, 0);
}

#[test]
fn degenerate_input_segments_without_panicking() {
    let grid = ClickGrid::new(120_000, 4, 4);
    let config = SegmentConfig::default();
    let phrase = gate_phrase();
    let empty_phrase = TargetPhrase { steps: Vec::new() };
    let note_offs = vec![NoteEvent {
        pitch: 53,
        velocity: 0,
        on: false,
        t_us: 0,
    }];
    let unsorted = vec![note_on(60, 2_000_000), note_on(53, 0)];

    assert!(segment(&[], &grid, Some(&phrase), &config)
        .attempts
        .is_empty());
    assert!(segment(&note_offs, &grid, Some(&phrase), &config)
        .onsets
        .is_empty());
    assert!(segment(&unsorted, &grid, Some(&empty_phrase), &config)
        .attempts
        .is_empty());
    assert_eq!(
        segment(&unsorted, &grid, Some(&phrase), &config).onsets[0].t_us,
        0,
        "clustering sorts by time"
    );
    let stopped_clock = ClickGrid::new(0, 0, 0);
    assert_eq!(
        segment(&unsorted, &stopped_clock, Some(&phrase), &config)
            .onsets
            .len(),
        2,
        "a zero bpm grid must not overflow or panic"
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

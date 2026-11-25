use super::*;
use crate::app::{Event, Intrada};
use crate::domain::item::Item;
use crate::domain::key::Key;
use crate::model::ReflectionTempoView;
use crux_core::App;

fn t(secs: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_790_000_000 + secs, 0).expect("in range")
}

fn key(name: &str) -> Option<Key> {
    Key::parse(name)
}

fn piece(id: &str, written: Option<Key>) -> Item {
    Item {
        id: id.to_string(),
        title: "Nocturne".to_string(),
        kind: ItemKind::Piece,
        composer: None,
        key: written,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: t(0),
        updated_at: t(0),
        exercise_links: vec![],
        priority: false,
        chord_chart: None,
        variation_ids: vec![],
        keys: ["G major", "A major", "D major"]
            .iter()
            .filter_map(|k| key(k))
            .collect(),
        photo_id: None,
        metre: None,
    }
}

fn model_with(written: Option<Key>) -> Model {
    Model {
        items: vec![piece("p", written), piece("q", None)].into(),
        ..Default::default()
    }
}

fn played(key_name: Option<&str>, score: Option<u8>) -> Play {
    Play {
        key: key_name.and_then(key),
        score,
        seconds: 300,
        ..Play::fixture()
    }
}

fn saved(item_id: &str, plays: Vec<Play>, started: DateTime<Utc>) -> PracticeSession {
    let mut entry = SetlistEntry::fixture();
    entry.plan.item_id = item_id.to_string();
    entry.status = EntryStatus::Completed;
    entry.plays = plays;
    PracticeSession {
        entries: vec![entry],
        started_at: started,
        completed_at: started,
        ..PracticeSession::fixture(&format!("saved-{}", started.timestamp()))
    }
}

fn send(model: &mut Model, event: SessionEvent) {
    let _ = Intrada.update(Event::Session(event), model);
}

// ── The starting score (#3004) ──

#[test]
fn each_key_starts_from_its_last_score_else_the_others_averaged_else_five() {
    struct Case {
        name: &'static str,
        history: Vec<PracticeSession>,
        last: Option<u8>,
        start: u8,
    }
    let cases = [
        Case {
            name: "never played",
            history: vec![],
            last: None,
            start: 5,
        },
        Case {
            name: "played in G but never scored",
            history: vec![saved("p", vec![played(Some("G major"), None)], t(-100))],
            last: None,
            start: 5,
        },
        Case {
            name: "G scored",
            history: vec![saved("p", vec![played(Some("G major"), Some(7))], t(-100))],
            last: Some(7),
            start: 7,
        },
        Case {
            name: "only another key scored",
            history: vec![saved("p", vec![played(Some("A major"), Some(6))], t(-100))],
            last: None,
            start: 6,
        },
        Case {
            name: "two other keys, a half rounds up",
            history: vec![saved(
                "p",
                vec![
                    played(Some("A major"), Some(6)),
                    played(Some("D major"), Some(7)),
                ],
                t(-100),
            )],
            last: None,
            start: 7,
        },
        Case {
            name: "two other keys, a whole average",
            history: vec![saved(
                "p",
                vec![
                    played(Some("A major"), Some(6)),
                    played(Some("D major"), Some(8)),
                ],
                t(-100),
            )],
            last: None,
            start: 7,
        },
        Case {
            name: "the most recent session wins, whatever the list order",
            history: vec![
                saved("p", vec![played(Some("G major"), Some(8))], t(-100)),
                saved("p", vec![played(Some("G major"), Some(3))], t(-900)),
            ],
            last: Some(8),
            start: 8,
        },
        Case {
            name: "the later play wins inside a session",
            history: vec![saved(
                "p",
                vec![
                    played(Some("G major"), Some(4)),
                    played(Some("G major"), Some(9)),
                ],
                t(-100),
            )],
            last: Some(9),
            start: 9,
        },
        Case {
            name: "an unscored newer play leaves the older score",
            history: vec![
                saved("p", vec![played(Some("G major"), None)], t(-100)),
                saved("p", vec![played(Some("G major"), Some(6))], t(-900)),
            ],
            last: Some(6),
            start: 6,
        },
        Case {
            name: "each other key counts once, at its latest score",
            history: vec![
                saved("p", vec![played(Some("A major"), Some(9))], t(-100)),
                saved(
                    "p",
                    vec![
                        played(Some("A major"), Some(2)),
                        played(Some("D major"), Some(4)),
                    ],
                    t(-900),
                ),
            ],
            last: None,
            start: 7,
        },
        Case {
            name: "the other spelling is the same key",
            history: vec![saved(
                "p",
                vec![
                    played(Some("F# major"), Some(2)),
                    played(Some("Gb major"), Some(8)),
                ],
                t(-100),
            )],
            last: None,
            start: 8,
        },
        Case {
            name: "a variation in G is not plain G, and is left out of the suggestion",
            history: vec![saved(
                "p",
                vec![Play {
                    variation_ids: vec!["v-dot".to_string()],
                    ..played(Some("G major"), Some(4))
                }],
                t(-100),
            )],
            last: None,
            start: 5,
        },
        Case {
            name: "another item's scores are not this one's",
            history: vec![saved("q", vec![played(Some("G major"), Some(9))], t(-100))],
            last: None,
            start: 5,
        },
    ];
    for case in cases {
        let mut m = model_with(None);
        m.sessions = case.history.into();
        assert_eq!(
            (
                last_score(&m, "p", &plain("G major")),
                start_score(&m, "p", &plain("G major"))
            ),
            (case.last, case.start),
            "{}",
            case.name
        );
    }
}

fn plain(key_name: &str) -> PlayWay {
    PlayWay {
        key: key(key_name),
        variation_ids: vec![],
    }
}

fn with_variations(key_name: &str, ids: &[&str]) -> PlayWay {
    PlayWay {
        key: key(key_name),
        variation_ids: ids.iter().map(|id| (*id).to_string()).collect(),
    }
}

fn played_way(way: &PlayWay, score: u8) -> Play {
    Play {
        key: way.key,
        variation_ids: way.variation_ids.clone(),
        score: Some(score),
        seconds: 300,
        ..Play::fixture()
    }
}

#[test]
fn last_score_matches_the_key_and_the_variations_together() {
    struct Case {
        name: &'static str,
        scored: PlayWay,
        asked: PlayWay,
        last: Option<u8>,
    }
    let cases = [
        Case {
            name: "plain G for plain G",
            scored: plain("G major"),
            asked: plain("G major"),
            last: Some(6),
        },
        Case {
            name: "a variation's score is not carried to a plain play",
            scored: with_variations("G major", &["v-dot"]),
            asked: plain("G major"),
            last: None,
        },
        Case {
            name: "a plain score is not carried to a play with a variation",
            scored: plain("G major"),
            asked: with_variations("G major", &["v-dot"]),
            last: None,
        },
        Case {
            name: "the same variation in G",
            scored: with_variations("G major", &["v-dot"]),
            asked: with_variations("G major", &["v-dot"]),
            last: Some(6),
        },
        Case {
            name: "the same variations in another order",
            scored: with_variations("G major", &["v-dot", "v-legato"]),
            asked: with_variations("G major", &["v-legato", "v-dot"]),
            last: Some(6),
        },
        Case {
            name: "a different variation",
            scored: with_variations("G major", &["v-dot"]),
            asked: with_variations("G major", &["v-legato"]),
            last: None,
        },
        Case {
            name: "the same variation in another key",
            scored: with_variations("A major", &["v-dot"]),
            asked: with_variations("G major", &["v-dot"]),
            last: None,
        },
    ];
    for case in cases {
        let mut m = model_with(None);
        m.sessions = vec![saved("p", vec![played_way(&case.scored, 6)], t(-100))].into();
        assert_eq!(last_score(&m, "p", &case.asked), case.last, "{}", case.name);
    }
}

#[test]
fn the_suggestion_averages_the_other_keys_only() {
    struct Case {
        name: &'static str,
        scored: Vec<(PlayWay, u8)>,
        asked: PlayWay,
        start: u8,
    }
    let cases = [
        Case {
            name: "two other keys",
            scored: vec![(plain("A major"), 2), (plain("D major"), 8)],
            asked: plain("G major"),
            start: 5,
        },
        Case {
            name: "a variation's score is left out",
            scored: vec![
                (plain("A major"), 2),
                (with_variations("A major", &["v-dot"]), 9),
                (plain("D major"), 9),
            ],
            asked: plain("G major"),
            start: 6,
        },
        Case {
            name: "only variations scored falls back to the middle",
            scored: vec![(with_variations("A major", &["v-dot"]), 9)],
            asked: plain("G major"),
            start: 5,
        },
        Case {
            name: "a new variation reads the plain keys",
            scored: vec![
                (plain("G major"), 8),
                (with_variations("D major", &["v-dot"]), 2),
            ],
            asked: with_variations("G major", &["v-legato"]),
            start: 8,
        },
        Case {
            name: "a way scored before keeps its own score",
            scored: vec![
                (with_variations("G major", &["v-dot"]), 9),
                (plain("A major"), 2),
            ],
            asked: with_variations("G major", &["v-dot"]),
            start: 9,
        },
    ];
    for case in cases {
        let mut m = model_with(None);
        let plays = case
            .scored
            .iter()
            .map(|(way, score)| played_way(way, *score))
            .collect();
        m.sessions = vec![saved("p", plays, t(-100))].into();
        assert_eq!(
            start_score(&m, "p", &case.asked),
            case.start,
            "{}",
            case.name
        );
    }
}

#[test]
fn a_key_spelt_as_none_and_as_the_written_key_is_one_key() {
    let mut m = model_with(key("G major"));
    m.sessions = vec![saved("p", vec![played(None, Some(6))], t(-100))].into();

    assert_eq!(last_score(&m, "p", &plain("G major")), Some(6));
    assert_eq!(last_score(&m, "p", &PlayWay::default()), Some(6));
    assert_eq!(start_score(&m, "p", &plain("A major")), 6);
}

// ── The sheet's rows ──

fn sheet_open_in_g(history: Vec<PracticeSession>) -> (Model, String) {
    let mut m = model_with(None);
    m.sessions = history.into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: TempoReading::silent(),
        },
    );
    let play_id = active(&m).entries[0].plays[0].id.clone();
    (m, play_id)
}

fn active(m: &Model) -> &ActiveSession {
    let SessionStatus::Active(a) = &m.session_status else {
        panic!("expected Active");
    };
    a
}

fn finished_play(m: &Model) -> &Play {
    let entries = match &m.session_status {
        SessionStatus::Active(a) => &a.entries,
        SessionStatus::Summary(s) => &s.entries,
        _ => panic!("no session"),
    };
    &entries[0].plays[0]
}

fn row(m: &Model) -> ReflectionTempoView {
    Intrada
        .view(m)
        .active_session
        .and_then(|v| v.reflection)
        .expect("the sheet")
        .tempos
        .into_iter()
        .next()
        .expect("one row")
}

fn scored_in_g_and_a() -> Vec<PracticeSession> {
    vec![saved(
        "p",
        vec![
            played(Some("G major"), Some(7)),
            played(Some("A major"), Some(3)),
        ],
        t(-100),
    )]
}

fn in_a(play_id: &str) -> DraftWay {
    DraftWay {
        play_id: play_id.to_string(),
        key: key("A major"),
        variation_ids: vec![],
    }
}

#[test]
fn a_row_shows_last_times_score_for_its_key() {
    let (m, _) = sheet_open_in_g(scored_in_g_and_a());

    let row = row(&m);
    assert_eq!((row.last_score, row.start_score), (Some(7), 7));
}

#[test]
fn a_row_never_scored_suggests_the_other_keys() {
    let (m, _) = sheet_open_in_g(vec![saved(
        "p",
        vec![played(Some("A major"), Some(4))],
        t(-100),
    )]);

    let row = row(&m);
    assert_eq!((row.last_score, row.start_score), (None, 4));
}

#[test]
fn a_row_whose_key_is_changed_reads_that_keys_score() {
    let (mut m, play_id) = sheet_open_in_g(scored_in_g_and_a());

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: ReflectionAnswers {
                ways: vec![in_a(&play_id)],
                ..ReflectionAnswers::default()
            },
        },
    );

    let row = row(&m);
    assert_eq!((row.last_score, row.start_score), (Some(3), 3));
}

// ── An untouched score keeps last time's (decided 2026-11-25) ──

fn submit(m: &mut Model, answers: ReflectionAnswers) {
    send(
        m,
        SessionEvent::SubmitReflection {
            next_item_started_at: t(300),
            answers,
        },
    );
}

#[test]
fn an_untouched_key_keeps_last_times_score() {
    let (mut m, _) = sheet_open_in_g(scored_in_g_and_a());

    submit(&mut m, ReflectionAnswers::default());

    assert!(m.last_error.is_none());
    assert_eq!(finished_play(&m).score, Some(7));
}

#[test]
fn an_untouched_key_never_scored_saves_its_suggested_score() {
    let (mut m, _) = sheet_open_in_g(vec![saved(
        "p",
        vec![played(Some("A major"), Some(4))],
        t(-100),
    )]);

    submit(&mut m, ReflectionAnswers::default());

    assert!(m.last_error.is_none());
    assert_eq!(finished_play(&m).score, Some(4));
}

#[test]
fn an_untouched_item_never_scored_at_all_saves_the_middle_score() {
    let (mut m, _) = sheet_open_in_g(vec![]);

    submit(&mut m, ReflectionAnswers::default());

    assert_eq!(finished_play(&m).score, Some(DEFAULT_START_SCORE));
}

#[test]
fn an_untouched_key_changed_to_a_variation_does_not_keep_the_plain_score() {
    let (mut m, play_id) = sheet_open_in_g(scored_in_g_and_a());
    m.variations = vec![crate::domain::variation::Variation {
        id: "v-dot".to_string(),
        label: "Dotted rhythms".to_string(),
        updated_at: t(0),
        deleted_at: None,
    }]
    .into();

    submit(
        &mut m,
        ReflectionAnswers {
            ways: vec![DraftWay {
                play_id,
                key: key("G major"),
                variation_ids: vec!["v-dot".to_string()],
            }],
            ..ReflectionAnswers::default()
        },
    );

    let play = finished_play(&m);
    assert_eq!(
        (play.variation_ids.clone(), play.score),
        (vec!["v-dot".to_string()], Some(5)),
        "G (7) and A (3) average to 5; last time's plain 7 is not carried over"
    );
}

#[test]
fn a_score_set_today_wins_over_last_times() {
    let (mut m, play_id) = sheet_open_in_g(scored_in_g_and_a());

    submit(
        &mut m,
        ReflectionAnswers {
            marks: vec![DraftMark { play_id, score: 9 }],
            ..ReflectionAnswers::default()
        },
    );

    assert_eq!(finished_play(&m).score, Some(9));
}

#[test]
fn an_untouched_key_changed_on_the_sheet_keeps_that_keys_score() {
    let (mut m, play_id) = sheet_open_in_g(scored_in_g_and_a());

    submit(
        &mut m,
        ReflectionAnswers {
            ways: vec![in_a(&play_id)],
            ..ReflectionAnswers::default()
        },
    );

    let play = finished_play(&m);
    assert_eq!((play.key, play.score), (key("A major"), Some(3)));
}

#[test]
fn a_stray_play_dropped_at_close_gets_no_kept_score() {
    let mut m = model_with(None);
    m.sessions = scored_in_g_and_a().into();
    send(&mut m, SessionEvent::StartBuilding);
    for item_id in ["p", "q"] {
        send(
            &mut m,
            SessionEvent::AddToSetlist {
                item_id: item_id.to_string(),
            },
        );
    }
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    let entry_id = active(&m).entries[0].id.clone();
    send(
        &mut m,
        SessionEvent::SwitchPlay {
            entry_id,
            key: key("A major"),
            variation_ids: vec![],
            now: t(300),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(302),
            reading: TempoReading::silent(),
        },
    );
    let errors_before = m.error_seq;

    submit(&mut m, ReflectionAnswers::default());

    assert_eq!(m.error_seq, errors_before);
    assert_eq!(active(&m).current_index, 1);
    let plays = &active(&m).entries[0].plays;
    assert_eq!(plays.len(), 1, "the stray play is dropped");
    assert_eq!(plays[0].score, Some(7));
}

#[test]
fn a_skipped_sheet_writes_no_score() {
    let (mut m, _) = sheet_open_in_g(scored_in_g_and_a());

    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(300),
            next_item_started_at: t(300),
            reading: TempoReading::silent(),
        },
    );

    assert_eq!(finished_play(&m).score, None);
}

// ── A note per key (#3004) ──

fn noted(play_id: &str, note: &str) -> ReflectionAnswers {
    ReflectionAnswers {
        notes: vec![DraftNote {
            play_id: play_id.to_string(),
            note: note.to_string(),
        }],
        ..ReflectionAnswers::default()
    }
}

#[test]
fn a_keys_note_is_written_onto_its_play_trimmed() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);

    submit(&mut m, noted(&play_id, "  thumb under late \n"));

    assert!(m.last_error.is_none());
    assert_eq!(finished_play(&m).note.as_deref(), Some("thumb under late"));
}

#[test]
fn a_blank_note_writes_nothing() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);

    submit(&mut m, noted(&play_id, "   "));

    assert!(m.last_error.is_none());
    assert_eq!(finished_play(&m).note, None);
}

#[test]
fn a_note_too_long_refuses_the_submit() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    let errors_before = m.error_seq;

    submit(
        &mut m,
        noted(&play_id, &"x".repeat(crate::validation::MAX_PLAY_NOTE + 1)),
    );

    assert_eq!(m.error_seq, errors_before + 1);
    assert!(active(&m).reflection.is_some(), "the sheet stays open");
    assert!(
        m.last_error
            .as_deref()
            .is_some_and(|e| e.contains("Keep the note under 500 characters.")),
        "the note's own refusal"
    );
    assert_eq!(finished_play(&m).note, None);
}

#[test]
fn a_note_at_the_limit_is_kept() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    let note = "é".repeat(crate::validation::MAX_PLAY_NOTE);

    submit(&mut m, noted(&play_id, &note));

    assert_eq!(finished_play(&m).note.as_deref(), Some(note.as_str()));
}

#[test]
fn a_note_for_a_play_from_elsewhere_is_refused() {
    let (mut m, _) = sheet_open_in_g(vec![]);
    let errors_before = m.error_seq;

    submit(&mut m, noted("not-a-play", "steady"));

    assert_eq!(m.error_seq, errors_before + 1);
    assert!(active(&m).reflection.is_some());
}

#[test]
fn two_notes_for_one_play_are_refused() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    let mut answers = noted(&play_id, "steady");
    answers.notes.push(DraftNote {
        play_id,
        note: "rushed".to_string(),
    });
    let errors_before = m.error_seq;

    submit(&mut m, answers);

    assert_eq!(m.error_seq, errors_before + 1);
    assert_eq!(finished_play(&m).note, None);
}

#[test]
fn the_draft_keeps_a_note_trimmed_and_drops_a_blank_one() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, " steady "),
        },
    );
    let kept = active(&m)
        .reflection
        .as_ref()
        .expect("draft")
        .answers
        .notes
        .clone();
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, " "),
        },
    );

    assert_eq!(
        kept,
        [DraftNote {
            play_id,
            note: "steady".to_string()
        }]
    );
    assert!(active(&m)
        .reflection
        .as_ref()
        .expect("draft")
        .answers
        .notes
        .is_empty());
}

#[test]
fn a_draft_note_too_long_is_refused_but_the_rest_of_the_draft_records() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, "steady"),
        },
    );
    let errors_before = m.error_seq;

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: ReflectionAnswers {
                marks: vec![DraftMark {
                    play_id: play_id.clone(),
                    score: 8,
                }],
                ..noted(&play_id, &"x".repeat(crate::validation::MAX_PLAY_NOTE + 1))
            },
        },
    );

    assert_eq!(m.error_seq, errors_before + 1);
    assert!(m
        .last_error
        .as_deref()
        .is_some_and(|e| e.contains("Keep the note under 500 characters.")));
    let answers = &active(&m).reflection.as_ref().expect("draft").answers;
    assert_eq!(answers.notes[0].note, "steady", "the last good note stays");
    assert_eq!(answers.marks[0].score, 8, "the mark sent with it records");
}

#[test]
fn a_draft_after_a_refused_note_still_records() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, &"x".repeat(crate::validation::MAX_PLAY_NOTE + 1)),
        },
    );

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: ReflectionAnswers {
                marks: vec![DraftMark {
                    play_id: play_id.clone(),
                    score: 3,
                }],
                ..noted(&play_id, "thumb under")
            },
        },
    );

    let answers = &active(&m).reflection.as_ref().expect("draft").answers;
    assert_eq!(answers.notes[0].note, "thumb under");
    assert_eq!(answers.marks[0].score, 3);
}

#[test]
fn a_draft_the_core_does_not_keep_is_refused_aloud() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, "steady"),
        },
    );
    let errors_before = m.error_seq;

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted("not-a-play", "rushed"),
        },
    );

    assert_eq!(m.error_seq, errors_before + 1);
    assert_eq!(
        m.last_error.as_deref(),
        Some("Couldn't save the reflection.")
    );
    let answers = &active(&m).reflection.as_ref().expect("draft").answers;
    assert_eq!(answers.notes[0].note, "steady", "the kept draft stays");
}

#[test]
fn a_kept_draft_lifts_an_earlier_refusal() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted("not-a-play", "rushed"),
        },
    );
    assert!(m.last_error.is_some());

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, "steady"),
        },
    );

    assert_eq!(m.last_error, None);
}

#[test]
fn a_refused_draft_note_with_no_earlier_note_leaves_none() {
    let (mut m, play_id) = sheet_open_in_g(vec![]);

    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: noted(&play_id, &"x".repeat(crate::validation::MAX_PLAY_NOTE + 1)),
        },
    );

    let draft = active(&m).reflection.as_ref().expect("draft");
    assert!(draft.answers.notes.is_empty());
}

// ── The wire (#846) ──

#[test]
fn the_sheet_row_and_a_noted_draft_round_trip_on_the_ffi_bincode_wire() {
    use crate::domain::types::assert_round_trips;
    let (m, play_id) = sheet_open_in_g(scored_in_g_and_a());
    assert_round_trips(row(&m));
    assert_round_trips(Event::Session(SessionEvent::UpdateReflectionDraft {
        answers: noted(&play_id, "steady"),
    }));
    assert_round_trips(Play {
        note: Some("thumb under".to_string()),
        ..Play::fixture()
    });
}

#[test]
fn a_noted_play_round_trips_through_storage() {
    let session = saved(
        "p",
        vec![Play {
            note: Some("thumb under".to_string()),
            ..played(Some("G major"), Some(7))
        }],
        t(0),
    );

    let row = crate::stored_session::session_to_stored(&session).expect("writes");
    let read = crate::stored_session::session_from_stored(&row).expect("reads");

    assert_eq!(read.unreadable, Vec::<String>::new());
    assert_eq!(read.session, session);
}

#[test]
fn a_synced_session_from_before_notes_still_reads() {
    let session = saved("p", vec![played(Some("G major"), Some(7))], t(0));
    let mut json = serde_json::to_value(&session).expect("writes");
    json["entries"][0]["plays"][0]
        .as_object_mut()
        .expect("a play")
        .remove("note");

    let read: PracticeSession = serde_json::from_value(json).expect("reads without a note");

    assert_eq!(read, session);
}

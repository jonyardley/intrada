use super::*;
use crate::app::{Event, Intrada};
use crate::domain::bars::BarRange;
use crate::domain::item::Item;
use crate::domain::variation::Variation;
use crate::model::{ActiveSessionView, BuildingSetlistView};
use crux_core::App;

fn t(secs: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_790_000_000 + secs, 0).expect("in range")
}

fn piece(id: &str) -> Item {
    Item {
        id: id.to_string(),
        title: "Nocturne".to_string(),
        kind: ItemKind::Piece,
        composer: None,
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: t(0),
        updated_at: t(0),
        exercise_links: vec![],
        priority: false,
        chord_chart: None,
        variation_ids: vec![],
        keys: vec![],
        photo_id: None,
        metre: None,
    }
}

fn model() -> Model {
    Model {
        items: vec![piece("p"), piece("q"), piece("clair")].into(),
        variations: vec![Variation {
            id: "v-dot".to_string(),
            label: "Dotted".to_string(),
            updated_at: t(0),
            deleted_at: None,
        }]
        .into(),
        ..Default::default()
    }
}

fn send(model: &mut Model, event: SessionEvent) {
    let _ = Intrada.update(Event::Session(event), model);
}

fn building(item_ids: &[&str]) -> Model {
    let mut m = model();
    send(&mut m, SessionEvent::StartBuilding);
    for id in item_ids {
        send(
            &mut m,
            SessionEvent::AddToSetlist {
                item_id: id.to_string(),
            },
        );
    }
    m
}

fn entry_id(m: &Model, index: usize) -> String {
    entries(m)[index].id.clone()
}

fn entries(m: &Model) -> &[SetlistEntry] {
    match &m.session_status {
        SessionStatus::Building(b) => &b.entries,
        SessionStatus::Active(a) => &a.entries,
        SessionStatus::Summary(s) => &s.entries,
        SessionStatus::Idle => panic!("no session"),
    }
}

fn active(m: &Model) -> &ActiveSession {
    let SessionStatus::Active(a) = &m.session_status else {
        panic!("expected Active");
    };
    a
}

fn active_view(m: &Model) -> ActiveSessionView {
    Intrada.view(m).active_session.expect("active view")
}

fn building_view(m: &Model) -> BuildingSetlistView {
    Intrada.view(m).building_setlist.expect("building view")
}

fn set_duration(m: &mut Model, id: &str, secs: u32) {
    send(
        m,
        SessionEvent::SetEntryDuration {
            entry_id: id.to_string(),
            duration_secs: Some(secs),
        },
    );
}

/// As iOS sends it: the click's own state rides every reading.
fn sounding(bpm: u16) -> TempoReading {
    TempoReading {
        bpm,
        click_sounding: true,
        click: Some(ClickState {
            metre: crate::domain::metre::Metre::default(),
            sounding: 0b1111,
        }),
    }
}

/// The piece over twenty minutes, started at `t(0)`.
fn practising_p() -> Model {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 1200);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    m
}

// ── Time away (#2306) ──

fn away(m: &mut Model, left: i64, back: i64) {
    send(m, SessionEvent::WentAway { at: t(left) });
    send(m, SessionEvent::CameBack { at: t(back) });
}

fn practising_q() -> Model {
    let mut m = building(&["q"]);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    m
}

#[test]
fn away_under_a_minute_offers_nothing() {
    let mut m = practising_q();
    away(&mut m, 100, 159);

    assert_eq!(active_view(&m).record.away_offer, None);
    send(&mut m, SessionEvent::LeaveAwayOut);
    assert!(entries(&m)[0].plays[0].away.iter().all(|a| !a.left_out));
}

#[test]
fn a_real_gap_is_offered_in_minutes() {
    let mut m = practising_q();
    away(&mut m, 60, 420);

    let offer = active_view(&m).record.away_offer.expect("an offer");
    assert_eq!(offer.minutes, 6);
    assert_eq!(offer.label, "Away 6 minutes. Leave it out?");
}

#[test]
fn leaving_six_minutes_out_saves_the_play_six_minutes_shorter() {
    let mut m = practising_q();
    away(&mut m, 60, 420);
    send(&mut m, SessionEvent::LeaveAwayOut);

    assert_eq!(active_view(&m).record.away_offer, None);
    assert_eq!(
        active_view(&m).current_item_started_at,
        t(360).to_rfc3339(),
        "the item's clock drops the gap too"
    );
    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(600),
            next_item_started_at: t(600),
            reading: TempoReading::silent(),
        },
    );
    let entry = &entries(&m)[0];
    assert_eq!(entry.plays[0].seconds, 240);
    assert_eq!(entry.duration_secs, 240);
    let raw = &entry.plays[0].away[0];
    assert_eq!((raw.left_at, raw.back_at), (t(60), Some(t(420))));
}

#[test]
fn a_gap_kept_in_counts_as_practice() {
    let mut m = practising_q();
    away(&mut m, 60, 420);
    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(600),
            next_item_started_at: t(600),
            reading: TempoReading::silent(),
        },
    );

    assert_eq!(entries(&m)[0].plays[0].seconds, 600);
}

#[test]
fn an_away_open_across_a_crash_closes_at_the_last_saved_time() {
    let mut m = practising_q();
    send(&mut m, SessionEvent::WentAway { at: t(60) });
    let saved = active(&m).clone();

    let mut fresh = model();
    send(
        &mut fresh,
        SessionEvent::RecoverSession {
            session: saved,
            now: t(5000),
        },
    );

    let away = &entries(&fresh)[0].plays[0].away[0];
    assert_eq!(away.back_at, Some(t(60)));
}

// ── Intention (#2303) ──

fn set_focus(m: &mut Model, id: &str, kind: FocusKind, target: Option<u16>) {
    send(
        m,
        SessionEvent::SetFocus {
            entry_id: id.to_string(),
            focus: Some(IntentionFocus { kind, target }),
        },
    );
}

#[test]
fn a_focus_is_refused_without_what_it_needs() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    for (kind, target) in [
        (FocusKind::Tempo, Some(500)),
        (FocusKind::Tempo, Some(30)),
        (FocusKind::CleanReps, Some(0)),
        (FocusKind::Evenness, Some(3)),
    ] {
        set_focus(&mut m, &id, kind, target);
        assert!(m.last_error.is_some(), "{kind:?} {target:?}");
        assert_eq!(entries(&m)[0].plan.focus, None);
    }
}

#[test]
fn a_typed_intention_suggests_its_focus() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SetEntryIntention {
            entry_id: id.clone(),
            intention: Some("at 84".to_string()),
        },
    );

    let suggested = building_view(&m).entries[0]
        .record
        .suggested_focus
        .clone()
        .expect("a suggestion");
    assert_eq!(
        suggested.focus,
        IntentionFocus {
            kind: FocusKind::Tempo,
            target: Some(84),
        }
    );
    assert_eq!(suggested.label, "At 84");

    set_focus(&mut m, &id, FocusKind::Tempo, Some(84));
    assert_eq!(
        building_view(&m).entries[0].record.suggested_focus,
        None,
        "nothing left to suggest once it is the focus"
    );
}

fn finish_a_with(focus: (FocusKind, Option<u16>), reading: TempoReading) -> Model {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_focus(&mut m, &id, focus.0, focus.1);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading,
        },
    );
    m
}

#[test]
fn a_tempo_is_met_when_a_play_reached_the_target_with_the_click() {
    let m = finish_a_with((FocusKind::Tempo, Some(84)), sounding(84));

    let finish = active_view(&m).record.finish.expect("the sheet");
    assert_eq!(finish.intention_met_read, Some(IntentionMet::Yes));
    assert!(!finish.asks_intention);
}

#[test]
fn a_tempo_short_of_the_target_is_asked_not_read() {
    let m = finish_a_with((FocusKind::Tempo, Some(84)), sounding(80));

    let finish = active_view(&m).record.finish.expect("the sheet");
    assert_eq!(finish.intention_met_read, None);
    assert!(finish.asks_intention);
}

#[test]
fn an_answer_is_stored_only_when_asked_and_given() {
    let mut m = finish_a_with((FocusKind::Evenness, None), TempoReading::silent());
    let id = entry_id(&m, 0);
    assert!(active_view(&m).record.finish.expect("sheet").asks_intention);

    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(300),
            next_item_started_at: t(300),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::AnswerIntention {
            entry_id: id,
            answer: Some(IntentionMet::Partly),
        },
    );

    assert_eq!(entries(&m)[0].intention_met, Some(IntentionMet::Partly));
}

// ── Finish answers (#2307, #2308) ──

fn finished_with_note(note: &str) -> (Model, String) {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::UpdateEntryNotes {
            entry_id: id.clone(),
            notes: Some(note.to_string()),
        },
    );
    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(300),
            next_item_started_at: t(300),
            reading: TempoReading::silent(),
        },
    );
    (m, id)
}

fn span_of(note: &str, text: &str) -> NoteSpan {
    let start = note.find(text).expect("text in note");
    NoteSpan {
        start: start as u32,
        end: (start + text.len()) as u32,
    }
}

/// Notes a musician would type, confirmed through the event the sheet sends
/// (#1256): what each confirm stores on the entry.
#[test]
fn confirming_a_point_stores_what_the_note_said() {
    let bars = |first, last| NotePointKind::Bars(BarRange { first, last });
    let cases: Vec<(&str, &str, NotePointKind)> = vec![
        (
            "left hand rushed in bar 12, got it at 84",
            "bar 12",
            bars(12, 12),
        ),
        (
            "left hand rushed in bar 12, got it at 84",
            "84",
            NotePointKind::Tempo { bpm: 84 },
        ),
        ("B section fingering in 21-24", "21-24", bars(21, 24)),
        (
            "coda from memory, 3 clean",
            "3 clean",
            NotePointKind::Repetitions {
                count: 3,
                clean: true,
                in_a_row: false,
            },
        ),
        (
            "A1 five clean in a row at q=96",
            "q=96",
            NotePointKind::Tempo { bpm: 96 },
        ),
    ];
    for (note, text, kind) in cases {
        let (mut m, id) = finished_with_note(note);
        send(
            &mut m,
            SessionEvent::ConfirmNotePoint {
                entry_id: id,
                span: span_of(note, text),
            },
        );
        assert_eq!(
            entries(&m)[0].note_points,
            [NotePoint {
                kind,
                span: span_of(note, text),
            }],
            "{note:?} confirming {text:?}"
        );
    }
}

#[test]
fn the_sheet_offers_the_points_its_note_reads() {
    let note = "left hand rushed in bar 12, got it at 84";
    let mut m = building(&["p"]);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: ReflectionAnswers {
                note: note.to_string(),
                note_points: vec![span_of(note, "84")],
                ..ReflectionAnswers::default()
            },
        },
    );

    let offers: Vec<(String, bool)> = active_view(&m)
        .record
        .finish
        .expect("the sheet")
        .note_offers
        .into_iter()
        .map(|o| (o.label, o.confirmed))
        .collect();
    assert_eq!(
        offers,
        [
            ("Bar 12".to_string(), false),
            ("\u{2669} = 84".to_string(), true)
        ]
    );
}

#[test]
fn a_span_the_note_offers_nothing_at_stores_nothing() {
    let note = "left hand rushed in bar 12";
    let (mut m, id) = finished_with_note(note);
    send(
        &mut m,
        SessionEvent::ConfirmNotePoint {
            entry_id: id.clone(),
            span: span_of(note, "rushed"),
        },
    );
    send(
        &mut m,
        SessionEvent::ConfirmNotePoint {
            entry_id: id.clone(),
            span: span_of(note, "bar 12"),
        },
    );
    send(
        &mut m,
        SessionEvent::ConfirmNotePoint {
            entry_id: id,
            span: span_of(note, "bar 12"),
        },
    );

    assert_eq!(
        entries(&m)[0].note_points.len(),
        1,
        "once, and only a point"
    );
}

#[test]
fn how_it_felt_and_what_got_in_the_way_are_stored_on_the_entry() {
    let (mut m, id) = finished_with_note("fine");
    send(
        &mut m,
        SessionEvent::SetFelt {
            entry_id: id.clone(),
            felt: Some(Felt::Strained),
        },
    );
    for obstacle in [Obstacle::Memory, Obstacle::Tension, Obstacle::Memory] {
        send(
            &mut m,
            SessionEvent::ToggleObstacle {
                entry_id: id.clone(),
                obstacle,
            },
        );
    }

    let entry = &entries(&m)[0];
    assert_eq!(entry.felt, Some(Felt::Strained));
    assert_eq!(entry.got_in_the_way, [Obstacle::Tension]);
}

#[test]
fn finish_answers_are_refused_on_an_entry_never_finished() {
    let mut m = practising_p();
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SetFelt {
            entry_id: id,
            felt: Some(Felt::Comfortable),
        },
    );

    assert_eq!(entries(&m)[0].felt, None);
}

// ── Builder rules the core owns (#2393) ──

#[test]
fn an_unknown_variation_is_refused_and_plans_nothing() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id,
            variation_ids: vec!["v-gone".to_string()],
        },
    );

    assert!(m.last_error.is_some());
    assert!(entries(&m)[0].plan.variation_ids.is_empty());
}

fn with_piece_bpm(m: &mut Model, bpm: Option<u16>) {
    let item = m.items.iter_mut().find(|i| i.id == "p").expect("p");
    item.tempo = bpm.map(|bpm| crate::domain::types::Tempo {
        marking: None,
        bpm: Some(bpm),
    });
}

/// A tempo focus starts at the piece's own marked tempo, held to what the
/// click plays (#2406, decided with sections gone on 2026-11-08).
#[test]
fn a_numbered_focus_sent_without_a_target_starts_from_the_core() {
    for (kind, bpm, starts) in [
        (FocusKind::Tempo, Some(72), 72),
        (FocusKind::Tempo, Some(300), 208),
        (FocusKind::Tempo, None, 96),
        (FocusKind::CleanReps, Some(72), 1),
    ] {
        let mut m = building(&["p"]);
        with_piece_bpm(&mut m, bpm);
        let id = entry_id(&m, 0);
        set_focus(&mut m, &id, kind, None);
        assert!(m.last_error.is_none(), "{:?}", m.last_error);
        assert_eq!(
            entries(&m)[0].plan.focus.as_ref().and_then(|f| f.target),
            Some(starts),
            "{kind:?} {bpm:?}"
        );
    }
}

#[test]
fn each_focus_choice_carries_the_range_its_target_is_held_to() {
    let m = building(&["p"]);
    let ranges: Vec<Option<(u16, u16, u16)>> = building_view(&m)
        .focus_choices
        .iter()
        .map(|c| c.target.as_ref().map(|t| (t.min, t.max, t.step)))
        .collect();

    assert_eq!(ranges, [Some((40, 208, 2)), Some((1, 100, 1)), None, None]);
}

#[test]
fn a_target_at_each_end_of_its_range_is_taken() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    for (kind, target) in [
        (FocusKind::Tempo, 40),
        (FocusKind::Tempo, 208),
        (FocusKind::CleanReps, 1),
        (FocusKind::CleanReps, 100),
    ] {
        set_focus(&mut m, &id, kind, Some(target));
        assert!(m.last_error.is_none(), "{kind:?} {target}");
    }
    for (kind, target) in [
        (FocusKind::Tempo, 39),
        (FocusKind::Tempo, 209),
        (FocusKind::CleanReps, 101),
    ] {
        set_focus(&mut m, &id, kind, Some(target));
        assert!(m.last_error.is_some(), "{kind:?} {target}");
    }
}

#[test]
fn a_focus_shows_its_target_as_a_caption() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    for (kind, target, caption) in [
        (FocusKind::Tempo, Some(84), Some("\u{2669} = 84")),
        (FocusKind::CleanReps, Some(5), Some("5 clean in a row")),
        (FocusKind::FromMemory, None, None),
    ] {
        set_focus(&mut m, &id, kind, target);
        let focus = building_view(&m).entries[0].record.focus.clone();
        assert_eq!(
            focus.and_then(|f| f.target_caption).as_deref(),
            caption,
            "{kind:?}"
        );
    }
}

#[test]
fn twenty_clean_in_a_row_offers_a_focus_the_stepper_can_show() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SetEntryIntention {
            entry_id: id,
            intention: Some("20 clean in a row".to_string()),
        },
    );

    let view = building_view(&m);
    let suggested = view.entries[0]
        .record
        .suggested_focus
        .clone()
        .expect("a suggestion");
    assert_eq!(suggested.focus.kind, FocusKind::CleanReps);
    assert_eq!(suggested.focus.target, Some(20));
    let range = view
        .focus_choices
        .iter()
        .find(|c| c.kind == FocusKind::CleanReps)
        .and_then(|c| c.target.clone())
        .expect("a range");
    assert!((range.min..=range.max).contains(&20));
}

#[test]
fn a_typed_tempo_the_click_cannot_play_suggests_nothing() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SetEntryIntention {
            entry_id: id,
            intention: Some("A1 at 30".to_string()),
        },
    );

    assert_eq!(building_view(&m).entries[0].record.suggested_focus, None);
}

// ── Last time ──

fn saved_session_playing(item_id: &str, way: PlayWay, started: DateTime<Utc>) -> PracticeSession {
    let mut entry = SetlistEntry::fixture();
    entry.plan.item_id = item_id.to_string();
    entry.status = EntryStatus::Completed;
    entry.plays = vec![Play {
        variation_ids: way.variation_ids,
        seconds: 300,
        ..Play::fixture()
    }];
    PracticeSession {
        id: format!("saved-{}", started.timestamp()),
        entries: vec![entry],
        session_notes: None,
        started_at: started,
        completed_at: started,
        total_duration_secs: 300,
        completion_status: CompletionStatus::Completed,
        session_score: None,
        capture_version: Some(CAPTURE_VERSION),
    }
}

#[test]
fn last_time_is_offered_and_one_tap_plans_it() {
    let mut m = model();
    m.sessions = vec![
        saved_session_playing(
            "p",
            PlayWay {
                ..PlayWay::default()
            },
            t(-200_000),
        ),
        saved_session_playing(
            "p",
            PlayWay {
                key: None,
                variation_ids: vec!["v-dot".to_string()],
            },
            t(-100_000),
        ),
    ]
    .into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    let id = entry_id(&m, 0);

    let offers = building_view(&m).last_times;
    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].label, "Last time: Dotted");

    send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });
    let entry = &entries(&m)[0];
    assert_eq!(entry.plan.variation_ids, ["v-dot"]);
}

fn saved_with_focus(way: PlayWay, focus: IntentionFocus) -> PracticeSession {
    let mut saved = saved_session_playing("p", way, t(-100_000));
    saved.entries[0].plan.focus = Some(focus);
    saved
}

fn tempo_focus(target: u16) -> IntentionFocus {
    IntentionFocus {
        kind: FocusKind::Tempo,
        target: Some(target),
    }
}

fn building_p(m: &mut Model) -> String {
    send(m, SessionEvent::StartBuilding);
    send(
        m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    entry_id(m, 0)
}

#[test]
fn last_times_aim_is_offered_and_one_tap_plans_it_too() {
    let mut m = model();
    let way = PlayWay {
        key: None,
        variation_ids: vec!["v-dot".to_string()],
    };
    m.sessions = vec![saved_with_focus(way, tempo_focus(84))].into();
    let id = building_p(&mut m);

    let offers = building_view(&m).last_times;
    assert_eq!(offers[0].label, "Last time: Dotted · At 84");

    send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });
    let entry = &entries(&m)[0];
    assert_eq!(entry.plan.variation_ids, ["v-dot"]);
    assert_eq!(entry.plan.focus, Some(tempo_focus(84)));
}

#[test]
fn an_aim_the_builder_would_refuse_is_not_offered() {
    for focus in [tempo_focus(30), tempo_focus(220)] {
        let mut m = model();
        let way = PlayWay {
            variation_ids: vec!["v-dot".to_string()],
            ..PlayWay::default()
        };
        m.sessions = vec![saved_with_focus(way, focus.clone())].into();
        let id = building_p(&mut m);

        assert_eq!(
            building_view(&m).last_times[0].label,
            "Last time: Dotted",
            "{focus:?}"
        );
        send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });
        assert_eq!(entries(&m)[0].plan.focus, None, "{focus:?}");
    }
}

/// An aim set for a piece that was then skipped was never practised, so it
/// is not last time's (#2397).
#[test]
fn an_aim_from_a_piece_left_unplayed_is_not_offered() {
    let stray_tap = Play {
        seconds: 2,
        score: None,
        rep_count: None,
        ..Play::fixture()
    };
    assert!(stray_tap.is_incidental());
    for plays in [vec![], vec![stray_tap]] {
        let mut m = model();
        let mut saved = saved_with_focus(PlayWay::default(), tempo_focus(84));
        saved.entries[0].plays = plays;
        saved.entries[0].status = EntryStatus::Skipped;
        m.sessions = vec![saved].into();
        let id = building_p(&mut m);

        assert!(building_view(&m).last_times.is_empty());
        send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });
        assert_eq!(entries(&m)[0].plan.focus, None);
    }
}

#[test]
fn last_time_is_offered_only_while_nothing_is_planned() {
    let mut m = model();
    m.sessions = vec![saved_session_playing(
        "p",
        PlayWay {
            variation_ids: vec!["v-dot".to_string()],
            ..PlayWay::default()
        },
        t(-100_000),
    )]
    .into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    let id = entry_id(&m, 0);
    assert_eq!(building_view(&m).last_times.len(), 1);

    set_focus(&mut m, &id, FocusKind::FromMemory, None);
    assert!(building_view(&m).last_times.is_empty(), "a focus is a plan");

    send(
        &mut m,
        SessionEvent::SetFocus {
            entry_id: id.clone(),
            focus: None,
        },
    );
    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id.clone(),
            variation_ids: vec!["v-dot".to_string()],
        },
    );
    assert!(
        building_view(&m).last_times.is_empty(),
        "a variation is a plan"
    );

    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id,
            variation_ids: vec![],
        },
    );
    assert_eq!(
        building_view(&m).last_times.len(),
        1,
        "cleared, it offers again"
    );
}

// ── The wire (#846) ──

fn every_field_set() -> ActiveSession {
    let mut entry = SetlistEntry::fixture();
    entry.status = EntryStatus::Completed;
    entry.plan.focus = Some(IntentionFocus {
        kind: FocusKind::Tempo,
        target: Some(84),
    });
    entry.intention_met = Some(IntentionMet::Partly);
    entry.felt = Some(Felt::HardWork);
    entry.got_in_the_way = vec![Obstacle::Fingering, Obstacle::Tone];
    entry.note_points = vec![NotePoint {
        kind: NotePointKind::Repetitions {
            count: 5,
            clean: true,
            in_a_row: true,
        },
        span: NoteSpan { start: 3, end: 22 },
    }];
    entry.plays = vec![Play {
        away: vec![
            Away {
                left_at: t(60),
                back_at: Some(t(420)),
                left_out: true,
            },
            Away {
                left_at: t(500),
                back_at: None,
                left_out: false,
            },
        ],
        ..Play::fixture()
    }];
    ActiveSession {
        id: "s1".to_string(),
        entries: vec![entry],
        current_index: 0,
        current_item_started_at: t(0),
        session_started_at: t(0),
        reflection: Some(ReflectionDraft {
            now: t(600),
            reading: sounding(84),
            answers: ReflectionAnswers {
                felt: Some(Felt::Comfortable),
                got_in_the_way: vec![Obstacle::Notes],
                note_points: vec![NoteSpan { start: 0, end: 6 }],
                intention_met: Some(IntentionMet::Yes),
                ..ReflectionAnswers::default()
            },
        }),
    }
}

#[test]
fn the_v017_record_and_its_events_round_trip_on_the_ffi_bincode_wire() {
    use crate::domain::types::assert_round_trips;
    assert_round_trips(every_field_set());
    for event in [
        SessionEvent::SetFocus {
            entry_id: "e1".to_string(),
            focus: Some(IntentionFocus {
                kind: FocusKind::CleanReps,
                target: Some(5),
            }),
        },
        SessionEvent::ApplyLastTime {
            entry_id: "e1".to_string(),
        },
        SessionEvent::SetEntryVariations {
            entry_id: "e1".to_string(),
            variation_ids: vec!["v-dot".to_string()],
        },
        SessionEvent::WentAway { at: t(2) },
        SessionEvent::CameBack { at: t(3) },
        SessionEvent::LeaveAwayOut,
        SessionEvent::ConfirmNotePoint {
            entry_id: "e1".to_string(),
            span: NoteSpan { start: 1, end: 2 },
        },
        SessionEvent::SetFelt {
            entry_id: "e1".to_string(),
            felt: None,
        },
        SessionEvent::ToggleObstacle {
            entry_id: "e1".to_string(),
            obstacle: Obstacle::Rhythm,
        },
        SessionEvent::AnswerIntention {
            entry_id: "e1".to_string(),
            answer: Some(IntentionMet::NotYet),
        },
    ] {
        assert_round_trips(Event::Session(event));
    }
}

#[test]
fn the_v017_record_round_trips_through_storage() {
    let mut session = saved_session_playing("p", PlayWay::default(), t(0));
    let recorded = every_field_set();
    session.entries = recorded.entries;

    let row = crate::stored_session::session_to_stored(&session).expect("writes");
    let read = crate::stored_session::session_from_stored(&row).expect("reads");

    assert_eq!(read.unreadable, Vec::<String>::new());
    assert_eq!(read.session, session);
}

#[test]
fn each_felt_choice_is_stored_as_its_word_and_reads_back() {
    for (felt, word) in [
        (Felt::Comfortable, "comfortable"),
        (Felt::HardWork, "hard_work"),
        (Felt::Strained, "strained"),
    ] {
        let mut session = saved_session_playing("p", PlayWay::default(), t(0));
        session.entries[0].felt = Some(felt);

        let row = crate::stored_session::session_to_stored(&session).expect("writes");
        let read = crate::stored_session::session_from_stored(&row).expect("reads");

        assert!(
            row.entries.contains(&format!("\"felt\":\"{word}\"")),
            "{felt:?} stored as {word}: {}",
            row.entries
        );
        assert_eq!(read.unreadable, Vec::<String>::new());
        assert_eq!(read.session.entries[0].felt, Some(felt));
    }
}

#[test]
fn the_finish_sheet_offers_the_felt_choices_in_order_with_their_words() {
    let m = sheet_open_on("", ReflectionAnswers::default());
    let choices = active_view(&m)
        .record
        .finish
        .expect("the sheet")
        .felt_choices;

    assert_eq!(
        choices
            .iter()
            .map(|c| (c.felt, c.label.as_str()))
            .collect::<Vec<_>>(),
        [
            (Felt::Comfortable, "Comfortable"),
            (Felt::HardWork, "Hard work"),
            (Felt::Strained, "Strained"),
        ]
    );
}

#[test]
fn the_builder_offers_the_focus_kinds_in_order_with_their_words() {
    let m = building(&["p"]);
    let choices = building_view(&m).focus_choices;

    assert_eq!(
        choices
            .iter()
            .map(|c| (c.kind, c.label.as_str()))
            .collect::<Vec<_>>(),
        [
            (FocusKind::Tempo, "Tempo"),
            (FocusKind::CleanReps, "Clean in a row"),
            (FocusKind::FromMemory, "From memory"),
            (FocusKind::Evenness, "Evenness"),
        ]
    );
}

fn sheet_open_on(note: &str, answers: ReflectionAnswers) -> Model {
    let mut m = building(&["p"]);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::UpdateReflectionDraft {
            answers: ReflectionAnswers {
                note: note.to_string(),
                ..answers
            },
        },
    );
    m
}

#[test]
fn a_draft_drops_a_span_its_edited_note_no_longer_offers() {
    let note = "rushed in bar 12";
    let m = sheet_open_on(
        note,
        ReflectionAnswers {
            note_points: vec![NoteSpan { start: 0, end: 6 }, span_of(note, "bar 12")],
            ..ReflectionAnswers::default()
        },
    );

    let draft = &active(&m).reflection.as_ref().expect("draft").answers;
    assert_eq!(draft.note, note, "kept, not refused");
    assert_eq!(draft.note_points, [span_of(note, "bar 12")]);
}

#[test]
fn a_padded_draft_note_is_kept_trimmed_with_its_points() {
    let note = "rushed in bar 12";
    let m = sheet_open_on(
        "  rushed in bar 12\n",
        ReflectionAnswers {
            note_points: vec![span_of(note, "bar 12")],
            ..ReflectionAnswers::default()
        },
    );

    let draft = &active(&m).reflection.as_ref().expect("draft").answers;
    assert_eq!(draft.note, note);
    assert_eq!(draft.note_points, [span_of(note, "bar 12")]);
}

#[test]
fn a_draft_naming_an_obstacle_twice_is_refused() {
    let m = sheet_open_on(
        "fine",
        ReflectionAnswers {
            got_in_the_way: vec![Obstacle::Tone, Obstacle::Tone],
            ..ReflectionAnswers::default()
        },
    );

    let draft = &active(&m).reflection.as_ref().expect("draft").answers;
    assert!(draft.note.is_empty(), "refused whole");
}

#[test]
fn editing_the_note_drops_points_it_no_longer_reads() {
    let note = "left hand rushed in bar 12, got it at 84";
    let (mut m, id) = finished_with_note(note);
    for text in ["bar 12", "84"] {
        send(
            &mut m,
            SessionEvent::ConfirmNotePoint {
                entry_id: id.clone(),
                span: span_of(note, text),
            },
        );
    }
    let edited = "left hand rushed in bar 12, got it at 88";
    send(
        &mut m,
        SessionEvent::UpdateEntryNotes {
            entry_id: id,
            notes: Some(edited.to_string()),
        },
    );

    let kinds: Vec<NotePointKind> = entries(&m)[0]
        .note_points
        .iter()
        .map(|p| p.kind.clone())
        .collect();
    assert_eq!(
        kinds,
        [NotePointKind::Bars(BarRange {
            first: 12,
            last: 12
        })]
    );
}

#[test]
fn an_answer_without_an_intention_is_not_stored() {
    let (mut m, id) = finished_with_note("fine");
    send(
        &mut m,
        SessionEvent::AnswerIntention {
            entry_id: id,
            answer: Some(IntentionMet::Yes),
        },
    );

    assert_eq!(entries(&m)[0].intention_met, None);
}

#[test]
fn a_given_answer_survives_a_tempo_typed_after_the_sheet() {
    let mut m = finish_a_with((FocusKind::Tempo, Some(84)), sounding(72));
    let id = entry_id(&m, 0);
    assert!(active_view(&m).record.finish.expect("sheet").asks_intention);
    let play_id = entries(&m)[0].plays[0].id.clone();
    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(300),
            next_item_started_at: t(300),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::UpdateEntryTempo {
            entry_id: id.clone(),
            play_id,
            tempo: Some(84),
            user_set: true,
            click: None,
        },
    );
    send(
        &mut m,
        SessionEvent::AnswerIntention {
            entry_id: id,
            answer: Some(IntentionMet::NotYet),
        },
    );

    assert_eq!(entries(&m)[0].intention_met, Some(IntentionMet::NotYet));
    let shown = &Intrada.view(&m).summary.expect("summary").entries[0].record;
    assert_eq!(
        (shown.intention_met, shown.intention_met_read),
        (Some(IntentionMet::NotYet), false),
        "their own answer, not the read"
    );
}

#[test]
fn a_left_out_gap_is_not_taken_twice_after_a_resume_at_the_sheet() {
    let mut m = practising_q();
    away(&mut m, 300, 900);
    send(&mut m, SessionEvent::LeaveAwayOut);
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(1200),
            reading: TempoReading::silent(),
        },
    );
    assert_eq!(entries(&m)[0].plays[0].seconds, 600);
    let saved = active(&m).clone();
    let mut fresh = model();
    send(
        &mut fresh,
        SessionEvent::RecoverSession {
            session: saved,
            now: t(1260),
        },
    );
    let shown_start = active_view(&fresh).current_item_started_at;
    send(
        &mut fresh,
        SessionEvent::NextItem {
            now: t(1300),
            next_item_started_at: t(1300),
            reading: TempoReading::silent(),
        },
    );

    let entry = &entries(&fresh)[0];
    assert_eq!(
        (entry.plays[0].seconds, entry.duration_secs, shown_start),
        (600, 600, t(660).to_rfc3339())
    );
}

#[test]
fn a_gap_on_a_closed_play_is_not_taken_twice_after_a_resume() {
    let mut m = building(&["p"]);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    away(&mut m, 300, 900);
    send(&mut m, SessionEvent::LeaveAwayOut);
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SwitchPlay {
            entry_id: id,
            key: None,
            variation_ids: vec!["v-dot".to_string()],
            now: t(1200),
            reading: TempoReading::silent(),
        },
    );
    assert_eq!(entries(&m)[0].plays[0].seconds, 600);
    let saved = active(&m).clone();
    let mut fresh = model();
    send(
        &mut fresh,
        SessionEvent::RecoverSession {
            session: saved,
            now: t(1260),
        },
    );
    send(
        &mut fresh,
        SessionEvent::NextItem {
            now: t(1360),
            next_item_started_at: t(1360),
            reading: TempoReading::silent(),
        },
    );

    let entry = &entries(&fresh)[0];
    let plays: u64 = entry.plays.iter().map(|p| p.seconds).sum();
    assert_eq!((entry.duration_secs, plays), (700, 700));
}

#[test]
fn a_gap_inside_the_stamped_time_is_not_taken_twice_after_a_resume() {
    let mut m = practising_q();
    away(&mut m, 700, 900);
    send(&mut m, SessionEvent::LeaveAwayOut);
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(1200),
            reading: TempoReading::silent(),
        },
    );
    assert_eq!(entries(&m)[0].plays[0].seconds, 1000);
    let saved = active(&m).clone();
    let mut fresh = model();
    send(
        &mut fresh,
        SessionEvent::RecoverSession {
            session: saved,
            now: t(1260),
        },
    );
    let view = active_view(&fresh);
    send(
        &mut fresh,
        SessionEvent::NextItem {
            now: t(1300),
            next_item_started_at: t(1300),
            reading: TempoReading::silent(),
        },
    );

    assert_eq!(view.current_item_started_at, t(260).to_rfc3339());
    assert_eq!(entries(&fresh)[0].plays[0].seconds, 1000);
}

#[test]
fn the_sheet_holds_the_item_at_its_time_less_what_was_left_out() {
    let mut m = practising_q();
    away(&mut m, 300, 900);
    send(&mut m, SessionEvent::LeaveAwayOut);
    let before = active_view(&m).current_item_started_at;
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(1200),
            reading: TempoReading::silent(),
        },
    );

    assert_eq!(
        (before, active_view(&m).current_item_started_at),
        (t(600).to_rfc3339(), t(600).to_rfc3339())
    );
}

#[test]
fn a_gap_from_before_a_resume_mid_play_is_not_taken_off() {
    let mut m = practising_q();
    away(&mut m, 300, 900);
    send(&mut m, SessionEvent::LeaveAwayOut);
    let saved = active(&m).clone();
    let mut fresh = model();
    send(
        &mut fresh,
        SessionEvent::RecoverSession {
            session: saved,
            now: t(2000),
        },
    );
    send(
        &mut fresh,
        SessionEvent::NextItem {
            now: t(2100),
            next_item_started_at: t(2100),
            reading: TempoReading::silent(),
        },
    );

    assert_eq!(entries(&fresh)[0].plays[0].seconds, 100);
}

#[test]
fn a_span_confirmed_twice_in_a_draft_is_kept_once() {
    let note = "rushed in bar 12";
    let m = sheet_open_on(
        note,
        ReflectionAnswers {
            note_points: vec![span_of(note, "bar 12"), span_of(note, "bar 12")],
            ..ReflectionAnswers::default()
        },
    );

    let draft = &active(&m).reflection.as_ref().expect("draft").answers;
    assert_eq!(draft.note_points, [span_of(note, "bar 12")]);
}

// ── Linked exercises, keys and the scoring confirm (the rest of #2249) ──

fn exercise(id: &str, title: &str) -> Item {
    Item {
        title: title.to_string(),
        kind: ItemKind::Exercise,
        ..piece(id)
    }
}

/// Clair de Lune, with a linked warm-up when asked; written in D flat, kept
/// in D and C.
fn clair_with_links(linked: bool) -> Model {
    let mut m = model();
    let mut items: Vec<Item> = m.items.iter().cloned().collect();
    let clair = items.iter_mut().find(|i| i.id == "clair").expect("clair");
    clair.key = crate::domain::key::Key::parse("D flat major");
    clair.keys = vec![
        crate::domain::key::Key::parse("D major").expect("key"),
        crate::domain::key::Key::parse("C major").expect("key"),
    ];
    if linked {
        clair.exercise_links = crate::domain::link::links_to(&["ex-warm"], t(0));
    }
    items.extend([
        exercise("ex-other", "Left hand arpeggios"),
        exercise("ex-warm", "Five finger warm-up"),
    ]);
    m.items = items.into();
    m
}

fn building_clair(linked: bool) -> (Model, String) {
    let mut m = clair_with_links(linked);
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "clair".to_string(),
        },
    );
    let id = entries(&m)
        .iter()
        .find(|e| e.plan.item_id == "clair")
        .expect("clair entry")
        .id
        .clone();
    (m, id)
}

#[test]
fn adding_a_piece_brings_its_linked_exercises() {
    let (m, _) = building_clair(true);

    let items: Vec<&str> = entries(&m)
        .iter()
        .map(|e| e.plan.item_id.as_str())
        .collect();
    assert_eq!(items, ["ex-warm", "clair"]);
}

#[test]
fn the_key_picker_offers_the_written_key_then_the_pieces_own() {
    let (m, id) = building_clair(false);

    let view = building_view(&m);
    let keys = &view
        .entry_keys
        .iter()
        .find(|k| k.entry_id == id)
        .expect("clair keeps keys")
        .keys;
    let labels: Vec<(&str, Option<&str>)> = keys
        .iter()
        .map(|k| (k.label.as_str(), k.caption.as_deref()))
        .collect();
    assert_eq!(
        labels,
        [
            ("Written key", Some("D♭ major")),
            ("D major", None),
            ("C major", None)
        ]
    );
    assert_eq!(keys[0].key, None);
}

#[test]
fn a_planned_key_opens_the_first_play_and_variations_stay_a_plan() {
    let (mut m, id) = building_clair(false);
    let d = crate::domain::key::Key::parse("D major");
    send(
        &mut m,
        SessionEvent::SetEntryKey {
            entry_id: id.clone(),
            key: d,
        },
    );
    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id,
            variation_ids: vec!["v-dot".to_string()],
        },
    );
    assert_eq!(
        building_view(&m).entries[0].planned_label.as_deref(),
        Some("D major · Dotted")
    );
    assert_eq!(building_view(&m).entries[0].plan_tags, ["D major"]);

    send(&mut m, SessionEvent::StartSession { now: t(0) });

    let play = entries(&m)[0].open_play().expect("open play").clone();
    assert_eq!(play.key, d);
    assert!(play.variation_ids.is_empty());
}

fn finishing_clair_in_d() -> (Model, String, String) {
    let (mut m, id) = building_clair(false);
    send(
        &mut m,
        SessionEvent::SetEntryKey {
            entry_id: id.clone(),
            key: crate::domain::key::Key::parse("D major"),
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
    let play_id = entries(&m)[0].plays[0].id.clone();
    (m, id, play_id)
}

fn in_g(play_id: &str) -> DraftWay {
    DraftWay {
        play_id: play_id.to_string(),
        key: crate::domain::key::Key::parse("G major"),
        variation_ids: vec![],
    }
}

fn next(m: &mut Model) {
    send(
        m,
        SessionEvent::NextItem {
            now: t(300),
            next_item_started_at: t(300),
            reading: TempoReading::silent(),
        },
    );
}

#[test]
fn the_finish_sheet_offers_the_keys_to_confirm() {
    let (m, _, _) = finishing_clair_in_d();

    let finish = active_view(&m).record.finish.expect("the sheet");
    let keys: Vec<&str> = finish.keys.iter().map(|k| k.label.as_str()).collect();
    assert_eq!(keys, ["Written key", "D major", "C major"]);
}

#[test]
fn planned_in_d_scored_as_g_records_g() {
    let (mut m, id, play_id) = finishing_clair_in_d();
    let answers = ReflectionAnswers {
        ways: vec![in_g(&play_id)],
        ..ReflectionAnswers::default()
    };
    send(&mut m, SessionEvent::UpdateReflectionDraft { answers });
    assert_eq!(
        active(&m).reflection.as_ref().expect("draft").answers.ways,
        [in_g(&play_id)],
        "the change survives a crash before Next"
    );

    next(&mut m);
    let way = in_g(&play_id);
    send(
        &mut m,
        SessionEvent::UpdatePlayWay {
            entry_id: id,
            play_id: way.play_id,
            key: way.key,
            variation_ids: way.variation_ids,
        },
    );

    assert_eq!(entries(&m)[0].plays[0].key, way.key);
}

#[test]
fn a_draft_way_alone_changes_no_play() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    let answers = ReflectionAnswers {
        ways: vec![in_g(&play_id)],
        ..ReflectionAnswers::default()
    };
    send(&mut m, SessionEvent::UpdateReflectionDraft { answers });

    next(&mut m);

    assert_eq!(
        entries(&m)[0].plays[0].key,
        crate::domain::key::Key::parse("D major")
    );
}

#[test]
fn last_time_carries_its_key_into_the_plan() {
    let mut m = model();
    let mut saved = saved_session_playing(
        "p",
        PlayWay {
            ..PlayWay::default()
        },
        t(-100_000),
    );
    saved.entries[0].plays[0].key = crate::domain::key::Key::parse("G major");
    m.sessions = vec![saved].into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    let id = entry_id(&m, 0);
    assert_eq!(building_view(&m).last_times[0].label, "Last time: G major");

    send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });

    assert_eq!(
        entries(&m)[0].plan.key,
        crate::domain::key::Key::parse("G major")
    );
    assert!(
        building_view(&m).last_times.is_empty(),
        "a planned key is a plan"
    );
}

#[test]
fn the_practice_screen_offers_last_time_only_while_played_plain() {
    let mut m = model();
    m.sessions = vec![saved_session_playing(
        "p",
        PlayWay {
            variation_ids: vec!["v-dot".to_string()],
            ..PlayWay::default()
        },
        t(-100_000),
    )]
    .into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    let offer = active_view(&m)
        .record
        .last_time
        .expect("offered while plain");
    assert_eq!(offer.variation_ids, ["v-dot"]);

    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SwitchPlay {
            entry_id: id,
            key: offer.key,
            variation_ids: offer.variation_ids,
            now: t(60),
            reading: TempoReading::silent(),
        },
    );

    assert!(active_view(&m).record.last_time.is_none());
}

fn d_flat_major() -> Option<crate::domain::key::Key> {
    crate::domain::key::Key::parse("D flat major")
}

#[test]
fn a_way_sent_before_next_changes_no_play() {
    let (mut m, id, play_id) = finishing_clair_in_d();

    send(
        &mut m,
        SessionEvent::UpdatePlayWay {
            entry_id: id,
            play_id,
            key: None,
            variation_ids: vec![],
        },
    );

    let play = &entries(&m)[0].plays[0];
    assert_eq!(play.key, crate::domain::key::Key::parse("D major"));
}

#[test]
fn a_way_naming_a_play_from_elsewhere_is_refused() {
    let (mut m, id, _) = finishing_clair_in_d();
    next(&mut m);

    send(
        &mut m,
        SessionEvent::UpdatePlayWay {
            entry_id: id,
            play_id: "someone-elses-play".to_string(),
            key: None,
            variation_ids: vec![],
        },
    );

    assert!(m.last_error.is_some());
}

#[test]
fn the_written_key_is_stored_as_no_key_wherever_it_is_named() {
    let (mut m, id) = building_clair(false);
    send(
        &mut m,
        SessionEvent::SetEntryKey {
            entry_id: id,
            key: d_flat_major(),
        },
    );
    assert_eq!(entries(&m)[0].plan.key, None, "planning");

    let (mut m, id, play_id) = finishing_clair_in_d();
    let answers = ReflectionAnswers {
        ways: vec![DraftWay {
            key: d_flat_major(),
            ..in_g(&play_id)
        }],
        ..ReflectionAnswers::default()
    };
    send(&mut m, SessionEvent::UpdateReflectionDraft { answers });
    let drafted = &active(&m).reflection.as_ref().expect("draft").answers.ways;
    assert_eq!(drafted[0].key, None, "the draft");

    next(&mut m);
    send(
        &mut m,
        SessionEvent::UpdatePlayWay {
            entry_id: id,
            play_id,
            key: d_flat_major(),
            variation_ids: vec![],
        },
    );
    assert_eq!(entries(&m)[0].plays[0].key, None, "the confirm");
}

#[test]
fn a_planned_key_alone_hides_last_time_in_the_builder() {
    let mut m = model();
    m.sessions = vec![saved_session_playing(
        "p",
        PlayWay {
            variation_ids: vec!["v-dot".to_string()],
            ..PlayWay::default()
        },
        t(-100_000),
    )]
    .into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "p".to_string(),
        },
    );
    let id = entry_id(&m, 0);
    assert_eq!(building_view(&m).last_times.len(), 1);

    send(
        &mut m,
        SessionEvent::SetEntryKey {
            entry_id: id,
            key: crate::domain::key::Key::parse("G major"),
        },
    );

    assert!(building_view(&m).last_times.is_empty());
}

#[test]
fn the_finish_sheet_hides_last_time() {
    let mut m = model();
    let mut saved = saved_session_playing("q", PlayWay::default(), t(-100_000));
    saved.entries[0].plays[0].key = crate::domain::key::Key::parse("G major");
    m.sessions = vec![saved].into();
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "q".to_string(),
        },
    );
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    assert!(active_view(&m).record.last_time.is_some());

    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: TempoReading::silent(),
        },
    );

    assert!(active_view(&m).record.last_time.is_none());
}

fn key(name: &str) -> Option<crate::domain::key::Key> {
    crate::domain::key::Key::parse(name)
}

fn finish_row(m: &Model) -> crate::model::FinishRowView {
    active_view(m)
        .record
        .finish
        .expect("the sheet")
        .rows
        .into_iter()
        .next()
        .expect("one row")
}

fn draft(m: &mut Model, ways: Vec<DraftWay>) {
    let answers = ReflectionAnswers {
        ways,
        ..ReflectionAnswers::default()
    };
    send(m, SessionEvent::UpdateReflectionDraft { answers });
}

fn drafted(m: &Model) -> &[DraftWay] {
    &active(m).reflection.as_ref().expect("draft").answers.ways
}

fn active_play_mut(m: &mut Model) -> &mut Play {
    let SessionStatus::Active(a) = &mut m.session_status else {
        panic!("expected Active");
    };
    &mut a.entries[0].plays[0]
}

#[test]
fn a_finish_row_names_the_way_recorded_until_one_is_drafted() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    let row = finish_row(&m);
    assert_eq!(row.play_id, play_id);
    assert_eq!(row.label, "D major");
    assert_eq!(row.key, key("D major"));
    assert!(row.can_change);

    draft(&mut m, vec![in_g(&play_id)]);

    let row = finish_row(&m);
    assert_eq!(row.label, "G major");
    assert_eq!(row.key, key("G major"));
}

#[test]
fn a_finish_row_keeps_a_key_the_piece_no_longer_keeps() {
    let (mut m, _, _) = finishing_clair_in_d();
    active_play_mut(&mut m).key = key("G major");

    let row = finish_row(&m);
    assert_eq!(row.label, "G major");
    let finish = active_view(&m).record.finish.expect("the sheet");
    let keys: Vec<&str> = finish.keys.iter().map(|k| k.label.as_str()).collect();
    assert_eq!(keys, ["Written key", "D major", "C major", "G major"]);
}

#[test]
fn a_finish_row_reads_the_written_key_in_either_spelling_as_written() {
    let (mut m, _, _) = finishing_clair_in_d();
    active_play_mut(&mut m).key = key("C sharp major");

    let row = finish_row(&m);
    assert_eq!(row.key, None);
    assert_eq!(row.label, "No variation");
}

#[test]
fn nothing_to_change_on_a_plain_item_offers_no_change() {
    let mut m = building(&["q"]);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: TempoReading::silent(),
        },
    );

    let finish = active_view(&m).record.finish.expect("the sheet");
    assert!(finish.keys.is_empty(), "the written key alone is no choice");
    assert!(!finish.rows[0].can_change);
}

#[test]
fn a_drafted_way_back_to_the_recorded_one_leaves_the_draft() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    draft(&mut m, vec![in_g(&play_id)]);

    draft(
        &mut m,
        vec![DraftWay {
            key: key("D major"),
            ..in_g(&play_id)
        }],
    );

    assert!(drafted(&m).is_empty());
}

#[test]
fn a_play_keeps_a_variation_deleted_since_when_its_key_changes() {
    let (mut m, id, play_id) = finishing_clair_in_d();
    active_play_mut(&mut m).variation_ids = vec!["v-dot".to_string()];
    m.variations[0].deleted_at = Some(t(1));
    let way = DraftWay {
        variation_ids: vec!["v-dot".to_string()],
        ..in_g(&play_id)
    };

    draft(&mut m, vec![way.clone()]);
    assert_eq!(drafted(&m), std::slice::from_ref(&way));

    next(&mut m);
    send(
        &mut m,
        SessionEvent::UpdatePlayWay {
            entry_id: id,
            play_id: way.play_id,
            key: way.key,
            variation_ids: way.variation_ids,
        },
    );
    assert_eq!(entries(&m)[0].plays[0].key, key("G major"));
    assert!(m.last_error.is_none());
}

#[test]
fn a_deleted_variation_the_play_never_recorded_is_still_refused() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    draft(&mut m, vec![in_g(&play_id)]);
    m.variations[0].deleted_at = Some(t(1));

    draft(
        &mut m,
        vec![DraftWay {
            variation_ids: vec!["v-dot".to_string()],
            ..in_g(&play_id)
        }],
    );

    assert_eq!(drafted(&m), [in_g(&play_id)], "the earlier draft stands");
}

#[test]
fn the_key_row_names_a_planned_key_outside_the_pieces_own() {
    let (mut m, id) = building_clair(false);
    if let SessionStatus::Building(b) = &mut m.session_status {
        b.entries
            .iter_mut()
            .find(|e| e.id == id)
            .expect("clair")
            .plan
            .key = key("G major");
    }

    let view = building_view(&m);
    let keys = view
        .entry_keys
        .iter()
        .find(|k| k.entry_id == id)
        .expect("clair keeps keys");
    assert_eq!(keys.current_label, "G major");
    assert_eq!(keys.current, key("G major"));
    let labels: Vec<&str> = keys.keys.iter().map(|k| k.label.as_str()).collect();
    assert_eq!(labels, ["Written key", "D major", "C major", "G major"]);
}

#[test]
fn the_key_row_shows_for_a_planned_key_on_a_piece_that_keeps_none() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    if let SessionStatus::Building(b) = &mut m.session_status {
        b.entries[0].plan.key = key("G major");
    }

    let view = building_view(&m);
    let keys = view
        .entry_keys
        .iter()
        .find(|k| k.entry_id == id)
        .expect("a way back to the written key");
    assert_eq!(keys.current_label, "G major");
}

#[test]
fn the_key_row_reads_written_key_when_nothing_is_planned() {
    let (m, id) = building_clair(false);
    let view = building_view(&m);
    let keys = view
        .entry_keys
        .iter()
        .find(|k| k.entry_id == id)
        .expect("clair keeps keys");
    assert_eq!(keys.current_label, "Written key");
    assert_eq!(keys.current, None);
}

fn clair_mut(m: &mut Model) -> &mut Item {
    m.items.iter_mut().find(|i| i.id == "clair").expect("clair")
}

#[test]
fn a_recorded_variation_named_twice_is_refused() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    active_play_mut(&mut m).variation_ids = vec!["v-dot".to_string()];

    draft(
        &mut m,
        vec![DraftWay {
            variation_ids: vec!["v-dot".to_string(), "v-dot".to_string()],
            ..in_g(&play_id)
        }],
    );

    assert!(drafted(&m).is_empty());
}

#[test]
fn a_finish_row_spells_its_key_as_the_piece_keeps_it() {
    let (mut m, _, _) = finishing_clair_in_d();
    clair_mut(&mut m)
        .keys
        .push(key("G flat major").expect("key"));
    active_play_mut(&mut m).key = key("F sharp major");

    let row = finish_row(&m);
    assert_eq!(row.key, key("G flat major"));
    assert_eq!(row.label, "G\u{266d} major");
}

#[test]
fn enharmonic_keys_are_listed_once() {
    let item = piece("q");
    let (f_sharp, g_flat) = (
        key("F sharp major").expect("key"),
        key("G flat major").expect("key"),
    );

    let labels: Vec<String> = crate::view::session::key_choices(&item, &[f_sharp, g_flat])
        .into_iter()
        .map(|k| k.label)
        .collect();

    assert_eq!(labels, ["Written key", "F\u{266f} major"]);
}

#[test]
fn the_key_row_spells_a_planned_key_as_the_piece_keeps_it() {
    let (mut m, id) = building_clair(false);
    clair_mut(&mut m)
        .keys
        .push(key("G flat major").expect("key"));
    if let SessionStatus::Building(b) = &mut m.session_status {
        b.entries
            .iter_mut()
            .find(|e| e.id == id)
            .expect("clair")
            .plan
            .key = key("F sharp major");
    }

    let view = building_view(&m);
    let keys = view
        .entry_keys
        .iter()
        .find(|k| k.entry_id == id)
        .expect("clair keeps keys");
    assert_eq!(keys.current, key("G flat major"));
    assert_eq!(keys.current_label, "G\u{266d} major");
    assert_eq!(keys.keys.len(), 4, "listed once");
}

#[test]
fn a_plain_item_with_keys_of_its_own_can_change_its_rows() {
    let mut m = model();
    m.items.iter_mut().find(|i| i.id == "q").expect("q").keys = vec![key("D major").expect("key")];
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "q".to_string(),
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

    let row = finish_row(&m);
    assert!(row.can_change);
    assert_eq!(row.label, "No variation");
}

#[test]
fn no_key_row_when_the_only_key_kept_is_the_written_one() {
    let (mut m, id) = building_clair(false);
    let clair = clair_mut(&mut m);
    clair.key = key("D flat major");
    clair.keys = vec![key("C sharp major").expect("key")];

    let view = building_view(&m);
    assert!(view.entry_keys.iter().all(|k| k.entry_id != id));
}

// ── The sheet's one submit carries every answer (#2230) ──

#[test]
fn a_blank_submitted_note_leaves_the_note_already_there() {
    let mut m = finish_a_with((FocusKind::Evenness, None), TempoReading::silent());
    let entry_id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::UpdateEntryNotes {
            entry_id,
            notes: Some("left hand first".to_string()),
        },
    );

    send(
        &mut m,
        SessionEvent::SubmitReflection {
            next_item_started_at: t(300),
            answers: ReflectionAnswers {
                note: "  \n".to_string(),
                ..ReflectionAnswers::default()
            },
        },
    );

    assert_eq!(entries(&m)[0].notes.as_deref(), Some("left hand first"));
}

#[test]
fn a_submitted_note_is_stored_trimmed_and_blank_is_no_note() {
    for (typed, stored) in [
        ("  rushed in bar 12\n", Some("rushed in bar 12")),
        (" \n ", None),
    ] {
        let mut m = finish_a_with((FocusKind::Evenness, None), TempoReading::silent());

        send(
            &mut m,
            SessionEvent::SubmitReflection {
                next_item_started_at: t(300),
                answers: ReflectionAnswers {
                    note: typed.to_string(),
                    ..ReflectionAnswers::default()
                },
            },
        );

        assert_eq!(m.last_error, None);
        assert_eq!(entries(&m)[0].notes.as_deref(), stored, "typed {typed:?}");
    }
}

#[test]
fn one_submit_writes_the_way_the_point_the_felt_word_the_obstacle_and_the_aim() {
    let mut m = finish_a_with((FocusKind::Evenness, None), TempoReading::silent());
    let play_id = entries(&m)[0].plays[0].id.clone();
    let note = "rushed in bar 12";

    send(
        &mut m,
        SessionEvent::SubmitReflection {
            next_item_started_at: t(300),
            answers: ReflectionAnswers {
                note: note.to_string(),
                note_points: vec![span_of(note, "bar 12")],
                felt: Some(Felt::Strained),
                got_in_the_way: vec![Obstacle::Memory],
                intention_met: Some(IntentionMet::Partly),
                ways: vec![DraftWay {
                    play_id,
                    key: None,
                    variation_ids: vec!["v-dot".to_string()],
                }],
                ..ReflectionAnswers::default()
            },
        },
    );

    let entry = &entries(&m)[0];
    assert_eq!(m.last_error, None);
    assert_eq!(
        entry.plays[0].variation_ids,
        ["v-dot"],
        "the way changed by hand"
    );
    assert_eq!(entry.note_points.len(), 1, "read from the note as stored");
    assert_eq!(entry.felt, Some(Felt::Strained));
    assert_eq!(entry.got_in_the_way, [Obstacle::Memory]);
    assert_eq!(entry.intention_met, Some(IntentionMet::Partly));
}

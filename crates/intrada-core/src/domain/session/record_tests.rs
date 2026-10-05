use super::*;
use crate::app::{Event, Intrada};
use crate::domain::item::Item;
use crate::domain::section::{BarRange, ItemSection, SectionKind};
use crate::domain::variation::Variation;
use crate::model::{ActiveSessionView, BuildingSetlistView};
use crux_core::App;

fn t(secs: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_790_000_000 + secs, 0).expect("in range")
}

fn section(id: &str, name: &str, bars: Option<(u16, u16)>, position: usize) -> ItemSection {
    ItemSection {
        id: id.to_string(),
        name: name.to_string(),
        bars: bars.map(|(first, last)| BarRange { first, last }),
        kind: SectionKind::Form,
        target_bpm: None,
        position,
        updated_at: t(0),
        deleted_at: None,
    }
}

fn piece(id: &str, sections: Vec<ItemSection>) -> Item {
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
        sections,
        photo_id: None,
        metre: None,
    }
}

fn model() -> Model {
    Model {
        items: vec![
            piece(
                "p",
                vec![
                    section("s-a", "A1", Some((1, 8)), 0),
                    section("s-b", "B", Some((21, 24)), 1),
                    section("s-coda", "Coda", None, 2),
                ],
            ),
            piece("q", vec![]),
            piece(
                "clair",
                vec![
                    section("c-a1", "A1", Some((1, 14)), 0),
                    section("c-b", "B", Some((15, 26)), 1),
                    section("c-a2", "A2", Some((27, 50)), 2),
                    section("c-coda", "Coda", Some((51, 72)), 3),
                ],
            ),
        ]
        .into(),
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

fn seg(section_id: &str, planned_secs: u32) -> Segment {
    Segment {
        section_id: section_id.to_string(),
        planned_secs,
    }
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

fn set_segments(m: &mut Model, id: &str, segments: Vec<Segment>) {
    send(
        m,
        SessionEvent::SetSegments {
            entry_id: id.to_string(),
            segments,
        },
    );
}

fn minutes(m: &Model, index: usize) -> Vec<(String, u32)> {
    entries(m)[index]
        .segments
        .iter()
        .map(|s| (s.section_id.clone(), s.planned_secs / 60))
        .collect()
}

fn pair(id: &str, mins: u32) -> (String, u32) {
    (id.to_string(), mins)
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

/// A over B over twenty minutes, started at `t(0)`.
fn practising_a_then_b() -> Model {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 1200);
    set_segments(&mut m, &id, vec![seg("s-a", 0), seg("s-b", 0)]);
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    m
}

// ── Segments (#2315) ──

#[test]
fn twenty_minutes_split_into_a_and_b_starts_at_ten_and_ten() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 1200);
    set_segments(&mut m, &id, vec![seg("s-a", 0), seg("s-b", 0)]);

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    assert_eq!(minutes(&m, 0), [pair("s-a", 10), pair("s-b", 10)]);
}

#[test]
fn a_segment_given_minutes_keeps_them_and_the_rest_share_what_is_left() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 1200);
    set_segments(
        &mut m,
        &id,
        vec![seg("s-a", 0), seg("s-b", 720), seg("s-coda", 0)],
    );

    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 4), pair("s-b", 12), pair("s-coda", 4)]
    );
}

#[test]
fn segment_minutes_always_sum_to_the_planned_time() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 1200);
    set_segments(&mut m, &id, vec![seg("s-a", 600), seg("s-b", 300)]);
    assert_eq!(minutes(&m, 0), [pair("s-a", 10), pair("s-b", 10)]);

    set_duration(&mut m, &id, 900);
    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 8), pair("s-b", 7)],
        "a new planned time splits again"
    );
}

#[test]
fn segments_that_cannot_fit_are_refused_whole() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 600);
    set_segments(&mut m, &id, vec![seg("s-a", 0), seg("s-b", 0)]);

    set_segments(&mut m, &id, vec![seg("s-a", 600), seg("s-b", 0)]);

    assert!(m.last_error.is_some());
    assert_eq!(minutes(&m, 0), [pair("s-a", 5), pair("s-b", 5)]);
}

#[test]
fn a_segment_must_name_a_live_section_of_the_item_once() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    for bad in [
        vec![seg("s-elsewhere", 0)],
        vec![seg("s-a", 0), seg("s-a", 0)],
    ] {
        set_segments(&mut m, &id, bad);
        assert!(m.last_error.is_some());
        assert!(entries(&m)[0].segments.is_empty());
    }
}

#[test]
fn one_planned_section_is_one_segment_of_the_planned_time() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 900);
    send(
        &mut m,
        SessionEvent::SetEntryPlan {
            entry_id: id.clone(),
            section_ids: vec!["s-b".to_string()],
            variation_ids: vec![],
        },
    );

    assert_eq!(entries(&m)[0].segments, [seg("s-b", 900)]);
}

#[test]
fn the_first_play_opens_on_the_first_segment_and_its_clock_runs() {
    let m = practising_a_then_b();

    let open = entries(&m)[0].open_play().expect("open play");
    assert_eq!(open.section_id.as_deref(), Some("s-a"));
    let clock = active_view(&m).record.segment.expect("segment clock");
    assert_eq!(clock.label, "A1");
    assert_eq!(clock.ends_at, t(600).to_rfc3339());
    assert_eq!(clock.move_label.as_deref(), Some("On to B"));
    assert_eq!(
        clock.stay_label.as_deref(),
        Some("Stay on A1, 2 more minutes taken from B")
    );
}

#[test]
fn on_to_b_closes_a_and_opens_b_on_a_fresh_clock() {
    let mut m = practising_a_then_b();

    send(
        &mut m,
        SessionEvent::MoveToNextSegment {
            now: t(650),
            reading: TempoReading::silent(),
        },
    );

    let plays = &entries(&m)[0].plays;
    assert_eq!(plays.len(), 2);
    assert_eq!(plays[0].seconds, 650);
    assert_eq!(plays[1].section_id.as_deref(), Some("s-b"));
    let clock = active_view(&m).record.segment.expect("segment clock");
    assert_eq!(clock.label, "B");
    assert_eq!(clock.ends_at, t(650 + 600).to_rfc3339());
}

#[test]
fn stay_on_a_takes_two_minutes_from_b() {
    let mut m = practising_a_then_b();

    send(&mut m, SessionEvent::StayOnSegment);
    let clock = active_view(&m).record.segment.expect("segment clock");
    assert_eq!(clock.ends_at, t(720).to_rfc3339());

    send(
        &mut m,
        SessionEvent::MoveToNextSegment {
            now: t(720),
            reading: TempoReading::silent(),
        },
    );
    let clock = active_view(&m).record.segment.expect("segment clock");
    assert_eq!(clock.ends_at, t(720 + 480).to_rfc3339());
}

#[test]
fn the_last_segment_offers_nothing() {
    let mut m = practising_a_then_b();
    send(
        &mut m,
        SessionEvent::MoveToNextSegment {
            now: t(600),
            reading: TempoReading::silent(),
        },
    );

    let clock = active_view(&m).record.segment.expect("segment clock");
    assert_eq!((clock.move_label, clock.stay_label), (None, None));

    let before = active(&m).clone();
    send(&mut m, SessionEvent::StayOnSegment);
    send(
        &mut m,
        SessionEvent::MoveToNextSegment {
            now: t(700),
            reading: TempoReading::silent(),
        },
    );
    assert_eq!(active(&m), &before);
}

#[test]
fn ignoring_the_offer_never_stops_the_item() {
    let mut m = practising_a_then_b();
    send(
        &mut m,
        SessionEvent::NextItem {
            now: t(1500),
            next_item_started_at: t(1500),
            reading: TempoReading::silent(),
        },
    );

    let entry = &entries(&m)[0];
    assert_eq!(entry.duration_secs, 1500);
    assert_eq!(entry.plays[0].seconds, 1500);
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

fn set_focus(m: &mut Model, id: &str, kind: FocusKind, section: Option<&str>, target: Option<u16>) {
    send(
        m,
        SessionEvent::SetFocus {
            entry_id: id.to_string(),
            focus: Some(IntentionFocus {
                kind,
                section_id: section.map(str::to_string),
                target,
            }),
        },
    );
}

#[test]
fn a_focus_is_refused_without_what_it_needs() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    for (kind, section, target) in [
        (FocusKind::Tempo, None, Some(500)),
        (FocusKind::Tempo, None, Some(30)),
        (FocusKind::Tempo, Some("s-elsewhere"), Some(84)),
        (FocusKind::CleanReps, None, Some(0)),
        (FocusKind::Evenness, None, Some(3)),
    ] {
        set_focus(&mut m, &id, kind, section, target);
        assert!(m.last_error.is_some(), "{kind:?} {section:?} {target:?}");
        assert_eq!(entries(&m)[0].focus, None);
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
            intention: Some("A1 at 84".to_string()),
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
            section_id: Some("s-a".to_string()),
            target: Some(84),
        }
    );
    assert_eq!(suggested.label, "A1 at 84");

    set_focus(&mut m, &id, FocusKind::Tempo, Some("s-a"), Some(84));
    assert_eq!(
        building_view(&m).entries[0].record.suggested_focus,
        None,
        "nothing left to suggest once it is the focus"
    );
}

fn finish_a_with(focus: (FocusKind, Option<u16>), reading: TempoReading) -> Model {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_segments(&mut m, &id, vec![seg("s-a", 0)]);
    set_focus(&mut m, &id, focus.0, Some("s-a"), focus.1);
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
fn tempo_on_a_section_is_met_when_a_play_of_it_reached_the_target_with_the_click() {
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
    let cases: Vec<(&str, &str, NotePointKind, Option<&str>)> = vec![
        (
            "left hand rushed in bar 12, got it at 84",
            "bar 12",
            bars(12, 12),
            None,
        ),
        (
            "left hand rushed in bar 12, got it at 84",
            "84",
            NotePointKind::Tempo { bpm: 84 },
            None,
        ),
        (
            "B section fingering in 21-24",
            "21-24",
            bars(21, 24),
            Some("s-b"),
        ),
        (
            "coda from memory, 3 clean",
            "3 clean",
            NotePointKind::Repetitions {
                count: 3,
                clean: true,
                in_a_row: false,
            },
            Some("s-coda"),
        ),
        (
            "A1 five clean in a row at q=96",
            "q=96",
            NotePointKind::Tempo { bpm: 96 },
            Some("s-a"),
        ),
    ];
    for (note, text, kind, section) in cases {
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
                section_id: section.map(str::to_string),
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
    let mut m = practising_a_then_b();
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

fn step(m: &mut Model, id: &str, section_id: &str, minutes: i8) {
    send(
        m,
        SessionEvent::StepSegment {
            entry_id: id.to_string(),
            section_id: section_id.to_string(),
            minutes,
        },
    );
}

fn twelve_split_three() -> (Model, String) {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    set_segments(
        &mut m,
        &id,
        vec![seg("s-a", 0), seg("s-b", 0), seg("s-coda", 0)],
    );
    (m, id)
}

#[test]
fn a_variation_on_a_two_minute_item_split_into_three_keeps_its_sections() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_segments(
        &mut m,
        &id,
        vec![seg("s-a", 0), seg("s-b", 0), seg("s-coda", 0)],
    );
    set_duration(&mut m, &id, 120);
    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id,
            variation_ids: vec!["v-dot".to_string()],
        },
    );

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    let entry = &entries(&m)[0];
    assert_eq!(entry.planned_section_ids(), ["s-a", "s-b", "s-coda"]);
    assert_eq!(entry.planned_variation_ids, ["v-dot"]);
}

#[test]
fn a_variation_leaves_tuned_minutes_as_they_were() {
    let (mut m, id) = twelve_split_three();
    step(&mut m, &id, "s-a", 2);
    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id,
            variation_ids: vec!["v-dot".to_string()],
        },
    );

    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 6), pair("s-b", 2), pair("s-coda", 4)]
    );
}

#[test]
fn an_unknown_variation_is_refused_and_plans_nothing() {
    let (mut m, id) = twelve_split_three();
    send(
        &mut m,
        SessionEvent::SetEntryVariations {
            entry_id: id,
            variation_ids: vec!["v-gone".to_string()],
        },
    );

    assert!(m.last_error.is_some());
    assert!(entries(&m)[0].planned_variation_ids.is_empty());
}

#[test]
fn three_taps_on_a1_take_from_the_next_and_the_fourth_is_refused() {
    let (mut m, id) = twelve_split_three();
    for _ in 0..3 {
        step(&mut m, &id, "s-a", 1);
        assert!(m.last_error.is_none(), "{:?}", m.last_error);
    }
    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 7), pair("s-b", 1), pair("s-coda", 4)]
    );

    step(&mut m, &id, "s-a", 1);
    assert!(m.last_error.is_some());
    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 7), pair("s-b", 1), pair("s-coda", 4)]
    );
}

#[test]
fn the_last_segment_takes_from_the_one_before() {
    let (mut m, id) = twelve_split_three();
    step(&mut m, &id, "s-coda", 1);

    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 4), pair("s-b", 3), pair("s-coda", 5)]
    );
}

#[test]
fn a_minute_taken_off_goes_to_the_same_neighbour() {
    let (mut m, id) = twelve_split_three();
    step(&mut m, &id, "s-a", -1);

    assert_eq!(
        minutes(&m, 0),
        [pair("s-a", 3), pair("s-b", 5), pair("s-coda", 4)]
    );
}

#[test]
fn a_step_is_refused_without_a_neighbour_or_a_planned_time() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_segments(&mut m, &id, vec![seg("s-a", 0), seg("s-b", 0)]);
    step(&mut m, &id, "s-a", 1);
    assert!(m.last_error.is_some(), "no planned time");

    set_duration(&mut m, &id, 600);
    set_segments(&mut m, &id, vec![seg("s-a", 0)]);
    step(&mut m, &id, "s-a", 1);
    assert!(m.last_error.is_some(), "one segment");

    set_segments(&mut m, &id, vec![seg("s-a", 0), seg("s-b", 0)]);
    step(&mut m, &id, "s-coda", 1);
    assert!(m.last_error.is_some(), "not a segment");
    assert_eq!(minutes(&m, 0), [pair("s-a", 5), pair("s-b", 5)]);
}

#[test]
fn each_segment_says_which_taps_would_land() {
    let (mut m, id) = twelve_split_three();
    for _ in 0..3 {
        step(&mut m, &id, "s-a", 1);
    }

    let taps: Vec<(bool, bool)> = building_view(&m).entries[0]
        .record
        .segments
        .iter()
        .map(|s| (s.can_add_minute, s.can_take_minute))
        .collect();
    assert_eq!(taps, [(false, true), (true, false), (false, true)]);
}

#[test]
fn without_a_planned_time_no_tap_lands() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_segments(&mut m, &id, vec![seg("s-a", 0), seg("s-b", 0)]);

    assert!(building_view(&m).entries[0]
        .record
        .segments
        .iter()
        .all(|s| !s.can_add_minute && !s.can_take_minute));
}

fn with_target_bpm(m: &mut Model, section_id: &str, bpm: u16) {
    let mut items: Vec<Item> = m.items.iter().cloned().collect();
    for section in items.iter_mut().flat_map(|i| i.sections.iter_mut()) {
        if section.id == section_id {
            section.target_bpm = Some(bpm);
        }
    }
    m.items = items.into();
}

#[test]
fn a_numbered_focus_sent_without_a_target_starts_from_the_core() {
    let mut m = building(&["p"]);
    with_target_bpm(&mut m, "s-a", 72);
    with_target_bpm(&mut m, "s-b", 300);
    let id = entry_id(&m, 0);
    for (kind, section, starts) in [
        (FocusKind::Tempo, Some("s-a"), 72),
        (FocusKind::Tempo, Some("s-b"), 208),
        (FocusKind::Tempo, Some("s-coda"), 96),
        (FocusKind::Tempo, None, 96),
        (FocusKind::CleanReps, Some("s-a"), 1),
    ] {
        set_focus(&mut m, &id, kind, section, None);
        assert!(m.last_error.is_none(), "{:?}", m.last_error);
        assert_eq!(
            entries(&m)[0].focus.as_ref().and_then(|f| f.target),
            Some(starts),
            "{kind:?} {section:?}"
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
        set_focus(&mut m, &id, kind, None, Some(target));
        assert!(m.last_error.is_none(), "{kind:?} {target}");
    }
    for (kind, target) in [
        (FocusKind::Tempo, 39),
        (FocusKind::Tempo, 209),
        (FocusKind::CleanReps, 101),
    ] {
        set_focus(&mut m, &id, kind, None, Some(target));
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
        set_focus(&mut m, &id, kind, Some("s-a"), target);
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

// ── Last time and trouble spots ──

fn saved_session_playing(item_id: &str, way: PlayWay, started: DateTime<Utc>) -> PracticeSession {
    let mut entry = SetlistEntry::fixture();
    entry.item_id = item_id.to_string();
    entry.status = EntryStatus::Completed;
    entry.plays = vec![Play {
        section_id: way.section_id,
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
                section_id: Some("s-a".to_string()),
                ..PlayWay::default()
            },
            t(-200_000),
        ),
        saved_session_playing(
            "p",
            PlayWay {
                section_id: Some("s-b".to_string()),
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
    assert_eq!(offers[0].label, "Last time: B · Dotted");

    send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });
    let entry = &entries(&m)[0];
    assert_eq!(entry.planned_section_ids(), ["s-b"]);
    assert_eq!(entry.planned_variation_ids, ["v-dot"]);
}

#[test]
fn last_time_is_offered_only_while_nothing_is_planned() {
    let mut m = model();
    m.sessions = vec![saved_session_playing(
        "p",
        PlayWay {
            section_id: Some("s-b".to_string()),
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

    set_focus(&mut m, &id, FocusKind::FromMemory, None, None);
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
        SessionEvent::SetEntryPlan {
            entry_id: id.clone(),
            section_ids: vec![],
            variation_ids: vec!["v-dot".to_string()],
        },
    );
    assert!(
        building_view(&m).last_times.is_empty(),
        "a variation is a plan"
    );

    send(
        &mut m,
        SessionEvent::SetEntryPlan {
            entry_id: id.clone(),
            section_ids: vec!["s-a".to_string()],
            variation_ids: vec![],
        },
    );
    assert!(
        building_view(&m).last_times.is_empty(),
        "a section is a plan"
    );

    send(
        &mut m,
        SessionEvent::SetEntryPlan {
            entry_id: id,
            section_ids: vec![],
            variation_ids: vec![],
        },
    );
    assert_eq!(
        building_view(&m).last_times.len(),
        1,
        "cleared, it offers again"
    );
}

#[test]
fn a_trouble_spot_lands_nameless_in_score_order() {
    let mut m = practising_a_then_b();
    send(
        &mut m,
        SessionEvent::AddTroubleSpot {
            item_id: "p".to_string(),
            bars: BarRange {
                first: 12,
                last: 14,
            },
        },
    );

    let item = m.items.iter().find(|i| i.id == "p").expect("piece");
    let mut live: Vec<&ItemSection> = item
        .sections
        .iter()
        .filter(|s| s.deleted_at.is_none())
        .collect();
    live.sort_by_key(|s| s.position);
    let labels: Vec<(String, SectionKind)> = live.iter().map(|s| (s.label(), s.kind)).collect();
    assert_eq!(
        labels,
        [
            ("A1".to_string(), SectionKind::Form),
            ("Bars 12 to 14".to_string(), SectionKind::TroubleSpot),
            ("B".to_string(), SectionKind::Form),
            ("Coda".to_string(), SectionKind::Form),
        ]
    );
}

// ── The wire (#846) ──

fn every_field_set() -> ActiveSession {
    let mut entry = SetlistEntry::fixture();
    entry.status = EntryStatus::Completed;
    entry.segments = vec![seg("s-a", 600), seg("s-b", 600)];
    entry.focus = Some(IntentionFocus {
        kind: FocusKind::Tempo,
        section_id: Some("s-a".to_string()),
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
        section_id: Some("s-b".to_string()),
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
        segment: Some(SegmentClock {
            index: 1,
            started_at: t(600),
            allowance_secs: 480,
            taken_from_next_secs: 0,
            left_out_secs: 90,
        }),
    }
}

#[test]
fn the_v017_record_and_its_events_round_trip_on_the_ffi_bincode_wire() {
    use crate::domain::types::assert_round_trips;
    assert_round_trips(every_field_set());
    for event in [
        SessionEvent::SetSegments {
            entry_id: "e1".to_string(),
            segments: vec![seg("s-a", 0)],
        },
        SessionEvent::SetFocus {
            entry_id: "e1".to_string(),
            focus: Some(IntentionFocus {
                kind: FocusKind::CleanReps,
                section_id: None,
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
        SessionEvent::StepSegment {
            entry_id: "e1".to_string(),
            section_id: "s-a".to_string(),
            minutes: -1,
        },
        SessionEvent::AddSegment {
            entry_id: "e1".to_string(),
            section_id: "s-a".to_string(),
        },
        SessionEvent::RemoveSegment {
            entry_id: "e1".to_string(),
            section_id: "s-a".to_string(),
        },
        SessionEvent::AddTroubleSpot {
            item_id: "p".to_string(),
            bars: BarRange { first: 3, last: 4 },
        },
        SessionEvent::MoveToNextSegment {
            now: t(1),
            reading: sounding(84),
        },
        SessionEvent::StayOnSegment,
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
fn a_tempo_on_the_whole_piece_does_not_meet_one_on_a_section() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_focus(&mut m, &id, FocusKind::Tempo, Some("s-a"), Some(84));
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    send(
        &mut m,
        SessionEvent::SwitchPlay {
            entry_id: id,
            section_id: None,
            key: None,
            variation_ids: vec![],
            now: t(1),
            reading: TempoReading::silent(),
        },
    );
    send(
        &mut m,
        SessionEvent::PrepareReflection {
            now: t(300),
            reading: sounding(90),
        },
    );

    let finish = active_view(&m).record.finish.expect("the sheet");
    assert_eq!(finish.intention_met_read, None);
    assert!(finish.asks_intention);
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
            section_id: Some("s-b".to_string()),
            key: None,
            variation_ids: vec![],
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
fn segment_minutes_past_the_planned_time_are_refused() {
    let mut m = building(&["p"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 1200);
    set_segments(
        &mut m,
        &id,
        vec![seg("s-a", u32::MAX - 59), seg("s-b", 120), seg("s-coda", 0)],
    );

    assert!(m.last_error.is_some());
    assert!(entries(&m)[0].segments.is_empty());
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
fn a_segment_end_keeps_a_gap_left_out_before_a_switch() {
    let mut m = practising_a_then_b();
    away(&mut m, 60, 420);
    send(&mut m, SessionEvent::LeaveAwayOut);
    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SwitchPlay {
            entry_id: id,
            section_id: None,
            key: None,
            variation_ids: vec![],
            now: t(500),
            reading: TempoReading::silent(),
        },
    );

    let clock = active_view(&m).record.segment.expect("segment clock");
    assert_eq!(clock.ends_at, t(960).to_rfc3339());
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

// ── Adding and removing a section keeps the tuning (#2398) ──

fn add_section(m: &mut Model, id: &str, section_id: &str) {
    send(
        m,
        SessionEvent::AddSegment {
            entry_id: id.to_string(),
            section_id: section_id.to_string(),
        },
    );
}

fn remove_section(m: &mut Model, id: &str, section_id: &str) {
    send(
        m,
        SessionEvent::RemoveSegment {
            entry_id: id.to_string(),
            section_id: section_id.to_string(),
        },
    );
}

/// Clair de Lune's twelve minutes tuned to A1 7, B 1, A2 4.
fn clair_tuned() -> (Model, String) {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    set_segments(
        &mut m,
        &id,
        vec![seg("c-a1", 420), seg("c-b", 60), seg("c-a2", 240)],
    );
    assert_eq!(
        minutes(&m, 0),
        [pair("c-a1", 7), pair("c-b", 1), pair("c-a2", 4)]
    );
    (m, id)
}

#[test]
fn adding_the_coda_takes_its_share_from_the_largest_section() {
    let (mut m, id) = clair_tuned();
    add_section(&mut m, &id, "c-coda");

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    assert_eq!(
        minutes(&m, 0),
        [
            pair("c-a1", 4),
            pair("c-b", 1),
            pair("c-a2", 4),
            pair("c-coda", 3)
        ]
    );
}

#[test]
fn removing_a_section_gives_its_minutes_to_the_next() {
    let (mut m, id) = clair_tuned();
    remove_section(&mut m, &id, "c-b");

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    assert_eq!(minutes(&m, 0), [pair("c-a1", 7), pair("c-a2", 5)]);
}

#[test]
fn removing_the_last_section_gives_its_minutes_to_the_one_before() {
    let (mut m, id) = clair_tuned();
    remove_section(&mut m, &id, "c-a2");

    assert_eq!(minutes(&m, 0), [pair("c-a1", 7), pair("c-b", 5)]);
}

#[test]
fn a_two_minute_item_split_one_and_one_refuses_a_third_section() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 120);
    set_segments(&mut m, &id, vec![seg("c-a1", 0), seg("c-b", 0)]);
    add_section(&mut m, &id, "c-a2");

    assert_eq!(
        m.last_error.as_deref(),
        Some("Those minutes don't fit in this item's 2 min")
    );
    assert_eq!(minutes(&m, 0), [pair("c-a1", 1), pair("c-b", 1)]);
    assert!(!building_view(&m).entries[0].record.can_add_section);
}

#[test]
fn a_section_already_in_a_full_split_says_so_rather_than_the_minutes() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 120);
    set_segments(&mut m, &id, vec![seg("c-a1", 0), seg("c-b", 0)]);
    add_section(&mut m, &id, "c-b");

    assert_eq!(
        m.last_error.as_deref(),
        Some("Each section can be one segment only")
    );
}

#[test]
fn a_split_with_room_offers_another_section() {
    let (m, _) = clair_tuned();

    assert!(building_view(&m).entries[0].record.can_add_section);
}

#[test]
fn a_section_deleted_from_the_piece_mid_build_can_still_be_removed() {
    let (mut m, id) = clair_tuned();
    let clair = m.items.iter_mut().find(|i| i.id == "clair").expect("clair");
    clair.sections[1].deleted_at = Some(t(5));
    remove_section(&mut m, &id, "c-a1");

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    assert_eq!(minutes(&m, 0), [pair("c-b", 8), pair("c-a2", 4)]);
}

#[test]
fn a_tie_for_the_largest_gives_from_the_earliest() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    set_segments(
        &mut m,
        &id,
        vec![seg("c-a1", 300), seg("c-b", 300), seg("c-a2", 0)],
    );
    add_section(&mut m, &id, "c-coda");

    assert_eq!(
        minutes(&m, 0),
        [
            pair("c-a1", 2),
            pair("c-b", 5),
            pair("c-a2", 2),
            pair("c-coda", 3)
        ]
    );
}

#[test]
fn sections_added_one_by_one_to_a_fresh_split_share_evenly() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    add_section(&mut m, &id, "c-a1");
    add_section(&mut m, &id, "c-b");
    add_section(&mut m, &id, "c-a2");

    assert_eq!(
        minutes(&m, 0),
        [pair("c-a1", 4), pair("c-b", 4), pair("c-a2", 4)]
    );
}

#[test]
fn removing_from_an_untuned_split_shares_evenly_again() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    set_segments(
        &mut m,
        &id,
        vec![seg("c-a1", 0), seg("c-b", 0), seg("c-a2", 0)],
    );
    remove_section(&mut m, &id, "c-b");

    assert_eq!(minutes(&m, 0), [pair("c-a1", 6), pair("c-a2", 6)]);
}

#[test]
fn odd_seconds_stay_with_the_section_that_gives() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 750);
    set_segments(
        &mut m,
        &id,
        vec![seg("c-a1", 0), seg("c-b", 60), seg("c-a2", 240)],
    );
    add_section(&mut m, &id, "c-coda");

    let secs: Vec<u32> = entries(&m)[0]
        .segments
        .iter()
        .map(|s| s.planned_secs)
        .collect();
    assert_eq!(secs, [270, 60, 240, 180]);
}

#[test]
fn the_first_section_takes_the_whole_planned_time() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    add_section(&mut m, &id, "c-b");

    assert_eq!(minutes(&m, 0), [pair("c-b", 12)]);
}

#[test]
fn removing_the_only_section_leaves_no_split() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    set_duration(&mut m, &id, 720);
    add_section(&mut m, &id, "c-b");
    remove_section(&mut m, &id, "c-b");

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    assert!(entries(&m)[0].segments.is_empty());
}

#[test]
fn without_a_planned_time_sections_are_just_added_and_dropped() {
    let mut m = building(&["clair"]);
    let id = entry_id(&m, 0);
    add_section(&mut m, &id, "c-a1");
    add_section(&mut m, &id, "c-b");
    add_section(&mut m, &id, "c-a2");
    remove_section(&mut m, &id, "c-b");

    assert!(m.last_error.is_none(), "{:?}", m.last_error);
    assert_eq!(minutes(&m, 0), [pair("c-a1", 0), pair("c-a2", 0)]);
}

#[test]
fn a_section_already_in_the_split_is_not_added_twice() {
    let (mut m, id) = clair_tuned();
    add_section(&mut m, &id, "c-b");

    assert!(m.last_error.is_some());
    assert_eq!(
        minutes(&m, 0),
        [pair("c-a1", 7), pair("c-b", 1), pair("c-a2", 4)]
    );
}

#[test]
fn a_section_from_another_piece_is_refused() {
    let (mut m, id) = clair_tuned();
    add_section(&mut m, &id, "s-coda");

    assert!(m.last_error.is_some());
    assert_eq!(entries(&m)[0].segments.len(), 3);
}

#[test]
fn removing_a_section_not_in_the_split_is_refused() {
    let (mut m, id) = clair_tuned();
    remove_section(&mut m, &id, "c-coda");

    assert!(m.last_error.is_some());
    assert_eq!(entries(&m)[0].segments.len(), 3);
}

#[test]
fn sections_are_only_added_while_building() {
    let (mut m, id) = clair_tuned();
    send(&mut m, SessionEvent::StartSession { now: t(0) });
    add_section(&mut m, &id, "c-coda");

    assert!(m.last_error.is_some());
    assert_eq!(entries(&m)[0].segments.len(), 3);
}

// ── Drills, keys and the scoring confirm (the rest of #2249) ──

fn exercise(id: &str, title: &str) -> Item {
    Item {
        title: title.to_string(),
        kind: ItemKind::Exercise,
        ..piece(id, vec![])
    }
}

fn link(
    exercise_id: &str,
    section_id: Option<&str>,
    position: usize,
) -> crate::domain::item::ExerciseLink {
    crate::domain::item::ExerciseLink::new(
        exercise_id.to_string(),
        section_id.map(str::to_string),
        position,
        t(0),
    )
}

/// Clair de Lune with two drills for A2, one for A1 and, when asked, one
/// for the whole piece; written in D flat, kept in D and C.
fn clair_with_drills(whole_piece_drill: bool) -> Model {
    let mut m = model();
    let mut items: Vec<Item> = m.items.iter().cloned().collect();
    let clair = items.iter_mut().find(|i| i.id == "clair").expect("clair");
    clair.key = crate::domain::key::Key::parse("D flat major");
    clair.keys = vec![
        crate::domain::key::Key::parse("D major").expect("key"),
        crate::domain::key::Key::parse("C major").expect("key"),
    ];
    clair.exercise_links = vec![
        link("ex-arp", Some("c-a2"), 0),
        link("ex-pedal", Some("c-a2"), 1),
        link("ex-a1", Some("c-a1"), 2),
    ];
    if whole_piece_drill {
        clair.exercise_links.push(link("ex-warm", None, 3));
    }
    items.extend([
        exercise("ex-arp", "Left hand arpeggios"),
        exercise("ex-pedal", "Pedal changes"),
        exercise("ex-a1", "Voicing the melody"),
        exercise("ex-warm", "Five finger warm-up"),
    ]);
    m.items = items.into();
    m
}

fn building_clair(whole_piece_drill: bool) -> (Model, String) {
    let mut m = clair_with_drills(whole_piece_drill);
    send(&mut m, SessionEvent::StartBuilding);
    send(
        &mut m,
        SessionEvent::AddToSetlist {
            item_id: "clair".to_string(),
        },
    );
    let id = entries(&m)
        .iter()
        .find(|e| e.item_id == "clair")
        .expect("clair entry")
        .id
        .clone();
    (m, id)
}

fn add_segment(m: &mut Model, id: &str, section_id: &str) {
    send(
        m,
        SessionEvent::AddSegment {
            entry_id: id.to_string(),
            section_id: section_id.to_string(),
        },
    );
}

fn offered(m: &Model) -> Vec<(String, String, bool)> {
    building_view(m)
        .drill_offers
        .iter()
        .map(|d| {
            (
                d.section_id.clone(),
                d.exercise_id.clone(),
                d.added_entry_id.is_some(),
            )
        })
        .collect()
}

#[test]
fn adding_a_piece_brings_only_its_whole_piece_drills() {
    let (m, _) = building_clair(true);

    let items: Vec<&str> = entries(&m).iter().map(|e| e.item_id.as_str()).collect();
    assert_eq!(items, ["ex-warm", "clair"]);
}

#[test]
fn adding_a2_offers_its_two_drills_and_adds_neither() {
    let (mut m, id) = building_clair(false);
    assert!(offered(&m).is_empty(), "nothing planned, nothing offered");

    add_segment(&mut m, &id, "c-a2");

    assert_eq!(
        offered(&m),
        [
            ("c-a2".to_string(), "ex-arp".to_string(), false),
            ("c-a2".to_string(), "ex-pedal".to_string(), false),
        ]
    );
    assert_eq!(entries(&m).len(), 1);
}

#[test]
fn ticking_a_drill_puts_it_in_a_block_with_its_piece() {
    let (mut m, id) = building_clair(false);
    add_segment(&mut m, &id, "c-a2");

    send(
        &mut m,
        SessionEvent::AddDrill {
            entry_id: id.clone(),
            exercise_id: "ex-arp".to_string(),
        },
    );

    let items: Vec<&str> = entries(&m).iter().map(|e| e.item_id.as_str()).collect();
    assert_eq!(items, ["ex-arp", "clair"]);
    let group = entries(&m)[1].group_id.clone();
    assert!(group.is_some());
    assert_eq!(entries(&m)[0].group_id, group);
    assert!(offered(&m)[0].2, "the ticked drill reads as added");
}

#[test]
fn a_drill_joins_the_block_the_piece_already_heads() {
    let (mut m, id) = building_clair(true);
    add_segment(&mut m, &id, "c-a2");

    send(
        &mut m,
        SessionEvent::AddDrill {
            entry_id: id,
            exercise_id: "ex-pedal".to_string(),
        },
    );

    let items: Vec<&str> = entries(&m).iter().map(|e| e.item_id.as_str()).collect();
    assert_eq!(items, ["ex-warm", "ex-pedal", "clair"]);
    let groups: std::collections::HashSet<_> =
        entries(&m).iter().map(|e| e.group_id.clone()).collect();
    assert_eq!(groups.len(), 1);
}

#[test]
fn a_drill_for_a_section_not_planned_is_refused() {
    let (mut m, id) = building_clair(false);
    add_segment(&mut m, &id, "c-a2");

    send(
        &mut m,
        SessionEvent::AddDrill {
            entry_id: id,
            exercise_id: "ex-a1".to_string(),
        },
    );

    assert_eq!(entries(&m).len(), 1);
    assert!(m.last_error.is_some());
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
    add_segment(&mut m, &id, "c-a1");
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
        Some("A1 · D major · Dotted")
    );

    send(&mut m, SessionEvent::StartSession { now: t(0) });

    let play = entries(&m)[0].open_play().expect("open play").clone();
    assert_eq!(play.section_id.as_deref(), Some("c-a1"));
    assert_eq!(play.key, d);
    assert!(play.variation_ids.is_empty());
}

fn finishing_clair_in_d() -> (Model, String, String) {
    let (mut m, id) = building_clair(false);
    add_segment(&mut m, &id, "c-a1");
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
        section_id: Some("c-a1".to_string()),
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
fn the_finish_sheet_offers_the_sections_and_keys_to_confirm() {
    let (m, _, _) = finishing_clair_in_d();

    let finish = active_view(&m).record.finish.expect("the sheet");
    let sections: Vec<&str> = finish.sections.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(sections, ["A1", "B", "A2", "Coda"]);
    let keys: Vec<&str> = finish.keys.iter().map(|k| k.label.as_str()).collect();
    assert_eq!(keys, ["Written key", "D major", "C major"]);
}

#[test]
fn planned_a1_in_d_scored_as_g_records_g() {
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
            section_id: way.section_id,
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
fn a_way_naming_another_pieces_section_is_refused() {
    let (mut m, id, play_id) = finishing_clair_in_d();
    next(&mut m);

    send(
        &mut m,
        SessionEvent::UpdatePlayWay {
            entry_id: id,
            play_id,
            section_id: Some("s-a".to_string()),
            key: None,
            variation_ids: vec![],
        },
    );

    assert_eq!(entries(&m)[0].plays[0].section_id.as_deref(), Some("c-a1"));
    assert!(m.last_error.is_some());
}

#[test]
fn a_draft_way_for_a_section_not_on_the_piece_is_refused_whole() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    let answers = ReflectionAnswers {
        ways: vec![DraftWay {
            section_id: Some("s-a".to_string()),
            ..in_g(&play_id)
        }],
        felt: Some(Felt::Strained),
        ..ReflectionAnswers::default()
    };

    send(&mut m, SessionEvent::UpdateReflectionDraft { answers });

    let kept = &active(&m).reflection.as_ref().expect("draft").answers;
    assert!(kept.ways.is_empty());
    assert_eq!(kept.felt, None, "the felt word went with it");
}

#[test]
fn last_time_carries_its_key_into_the_plan() {
    let mut m = model();
    let mut saved = saved_session_playing(
        "p",
        PlayWay {
            section_id: Some("s-b".to_string()),
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
    assert_eq!(
        building_view(&m).last_times[0].label,
        "Last time: B · G major"
    );

    send(&mut m, SessionEvent::ApplyLastTime { entry_id: id });

    assert_eq!(
        entries(&m)[0].planned_key,
        crate::domain::key::Key::parse("G major")
    );
    assert!(
        building_view(&m).last_times.is_empty(),
        "a planned key is a plan"
    );
}

#[test]
fn the_practice_screen_offers_last_time_only_on_the_whole_piece_plain() {
    let mut m = model();
    m.sessions = vec![saved_session_playing(
        "p",
        PlayWay {
            section_id: Some("s-b".to_string()),
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
        .expect("offered on the whole piece");
    assert_eq!(offer.section_id.as_deref(), Some("s-b"));

    let id = entry_id(&m, 0);
    send(
        &mut m,
        SessionEvent::SwitchPlay {
            entry_id: id,
            section_id: offer.section_id,
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
            section_id: None,
            key: None,
            variation_ids: vec![],
        },
    );

    let play = &entries(&m)[0].plays[0];
    assert_eq!(play.section_id.as_deref(), Some("c-a1"));
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
            section_id: None,
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
    assert_eq!(entries(&m)[0].planned_key, None, "planning");

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
            section_id: Some("c-a1".to_string()),
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
            section_id: Some("s-b".to_string()),
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

#[test]
fn ticking_a_drill_already_in_the_session_forms_no_block() {
    let mut m = clair_with_drills(false);
    send(&mut m, SessionEvent::StartBuilding);
    for item_id in ["ex-arp", "clair"] {
        send(
            &mut m,
            SessionEvent::AddToSetlist {
                item_id: item_id.to_string(),
            },
        );
    }
    let id = entry_id(&m, 1);
    add_segment(&mut m, &id, "c-a2");

    send(
        &mut m,
        SessionEvent::AddDrill {
            entry_id: id,
            exercise_id: "ex-arp".to_string(),
        },
    );

    assert!(entries(&m).iter().all(|e| e.group_id.is_none()));
    assert!(offered(&m)[0].2, "it reads as ticked");
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
    assert_eq!(row.label, "A1 · D major");
    assert_eq!(row.section_id.as_deref(), Some("c-a1"));
    assert_eq!(row.key, key("D major"));
    assert!(row.can_change);

    draft(&mut m, vec![in_g(&play_id)]);

    let row = finish_row(&m);
    assert_eq!(row.label, "A1 · G major");
    assert_eq!(row.key, key("G major"));
}

#[test]
fn a_finish_row_on_the_whole_piece_plain_reads_whole_piece() {
    let (mut m, _, play_id) = finishing_clair_in_d();
    draft(
        &mut m,
        vec![DraftWay {
            play_id,
            section_id: None,
            key: None,
            variation_ids: vec![],
        }],
    );

    assert_eq!(finish_row(&m).label, "Whole piece");
}

#[test]
fn a_finish_row_keeps_a_key_the_piece_no_longer_keeps() {
    let (mut m, _, _) = finishing_clair_in_d();
    active_play_mut(&mut m).key = key("G major");

    let row = finish_row(&m);
    assert_eq!(row.label, "A1 · G major");
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
    assert_eq!(row.label, "A1");
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
            section_id: way.section_id,
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
    m.variations[0].deleted_at = Some(t(1));

    draft(
        &mut m,
        vec![DraftWay {
            variation_ids: vec!["v-dot".to_string()],
            ..in_g(&play_id)
        }],
    );

    assert!(drafted(&m).is_empty());
}

#[test]
fn the_key_row_names_a_planned_key_outside_the_pieces_own() {
    let (mut m, id) = building_clair(false);
    if let SessionStatus::Building(b) = &mut m.session_status {
        b.entries
            .iter_mut()
            .find(|e| e.id == id)
            .expect("clair")
            .planned_key = key("G major");
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
        b.entries[0].planned_key = key("G major");
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

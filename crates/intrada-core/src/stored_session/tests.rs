//! The legacy and unknown-value rows are shapes shipped builds wrote, copied
//! from the iOS store tests that pinned them (`LegacyEntryPlaysTests`,
//! `LibraryStoreTests`), never written to match this decoder (#1256). The bad
//! times, the clamps and the unknown play modality are new cases.

use super::*;
use crate::domain::key::Key;

const START: &str = "2026-09-01T10:00:00Z";

fn row(entries: &str) -> StoredSession {
    StoredSession {
        id: "s1".to_string(),
        started_at: START.to_string(),
        completed_at: "2026-09-01T10:10:00Z".to_string(),
        total_duration_secs: 600,
        completion_status: "completed".to_string(),
        session_notes: None,
        entries: entries.to_string(),
        session_score: None,
        capture_version: None,
    }
}

fn read(entries: &str) -> SessionRead {
    session_from_stored(&row(entries)).expect("reads")
}

fn only_entry(entries: &str) -> SetlistEntry {
    let mut read = read(entries).session.entries;
    assert_eq!(read.len(), 1);
    read.remove(0)
}

fn at(text: &str) -> DateTime<Utc> {
    text.parse().expect("a time")
}

// ── The fold of rows written before plays (#1739) ──

#[test]
fn a_legacy_entry_folds_into_one_play_carrying_everything() {
    let entry = only_entry(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"Major Scales","itemType":"exercise","position":0,"durationSecs":600,"status":"completed","score":7,"intention":"even tone","repTarget":10,"repCount":8,"repTargetReached":false,"repHistory":[{"action":"success","at":"2026-09-01T10:01:00Z"},{"action":"missed","at":"2026-09-01T10:02:00Z"}],"plannedDurationSecs":600,"achievedTempo":120,"clickPattern":{"metre":{"beats":3,"unit":4},"sounding":1},"groupId":"g1","variantId":"v-c"}]"#,
    );
    assert_eq!(entry.plays.len(), 1);
    let play = &entry.plays[0];
    assert_eq!(play.id, "e1-play");
    assert_eq!(play.started_at, at(START));
    assert_eq!(play.seconds, 600);
    assert_eq!(play.score, Some(7));
    assert_eq!((play.rep_target, play.rep_count), (Some(10), Some(8)));
    assert_eq!(play.achieved_tempo, Some(120));
    assert_eq!(
        play.click_pattern,
        Some(ClickState {
            metre: Metre {
                beats: 3,
                unit: 4,
                groups: None
            },
            sounding: 1
        })
    );
    assert_eq!(
        play.rep_history
            .as_ref()
            .map(|h| h.iter().map(|r| r.at).collect::<Vec<_>>()),
        Some(vec![at("2026-09-01T10:01:00Z"), at("2026-09-01T10:02:00Z")])
    );
    assert!(
        play.variation_ids.is_empty(),
        "a step is retired, not moved (#2246)"
    );
    assert_eq!(entry.group_id.as_deref(), Some("g1"));
    assert_eq!(entry.intention.as_deref(), Some("even tone"));
    assert_eq!(
        entry.planned_rep_target,
        Some(10),
        "the old target plans the entry"
    );
    assert!(entry.planned_variation_ids.is_empty());
}

/// A timed run-through with no mark: the status alone is the record.
#[test]
fn a_legacy_completed_entry_with_no_mark_still_folds() {
    let entry = only_entry(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"Clair de Lune","itemType":"piece","position":0,"durationSecs":420,"status":"completed"}]"#,
    );
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].seconds, 420);
}

const NOT_ATTEMPTED: &str = r#"[{"id":"e1","itemId":"i1","itemTitle":"Scales","itemType":"exercise","position":0,"durationSecs":0,"status":"not_attempted"}]"#;

#[test]
fn a_legacy_not_attempted_entry_keeps_no_play() {
    let entry = only_entry(NOT_ATTEMPTED);
    assert_eq!(entry.status, EntryStatus::NotAttempted);
    assert!(entry.plays.is_empty());
    assert!(
        read(NOT_ATTEMPTED).unreadable.is_empty(),
        "read, not defaulted"
    );
}

#[test]
fn a_legacy_entry_with_untimed_taps_dates_them_to_the_session_start() {
    let read = read(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"X","itemType":"piece","position":0,"durationSecs":0,"status":"completed","repTarget":5,"repCount":1,"repHistory":["success","missed"]}]"#,
    );
    assert!(
        read.unreadable.is_empty(),
        "both actions read, not defaulted"
    );
    let entry = &read.session.entries[0];
    let history = entry.plays[0].rep_history.clone().expect("a history");
    assert_eq!(
        history,
        vec![
            RepEvent {
                action: RepAction::Success,
                at: at(START),
                tempo: None,
                click_sounding: None
            },
            RepEvent {
                action: RepAction::Missed,
                at: at(START),
                tempo: None,
                click_sounding: None
            },
        ]
    );
}

/// The old skip froze rep state rather than discarding it, so these rows
/// exist: skipped, with repetitions genuinely banked.
#[test]
fn a_legacy_skipped_entry_that_banked_reps_keeps_its_play() {
    let entry = only_entry(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"Hanon No. 1","itemType":"exercise","position":0,"durationSecs":0,"status":"skipped","repTarget":10,"repCount":4,"repTargetReached":false}]"#,
    );
    assert_eq!(entry.status, EntryStatus::Skipped);
    assert_eq!(entry.plays.len(), 1);
    assert_eq!(entry.plays[0].rep_count, Some(4));
}

#[test]
fn a_legacy_skipped_entry_that_banked_nothing_keeps_no_play() {
    let entry = only_entry(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"Sight-reading","itemType":"exercise","position":0,"durationSecs":0,"status":"skipped"}]"#,
    );
    assert!(entry.plays.is_empty());
}

#[test]
fn plays_win_over_stale_legacy_keys() {
    let entry = only_entry(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"Major Scales","itemType":"exercise","position":0,"durationSecs":600,"status":"completed","score":2,"variantId":"v-stale","plays":[{"id":"p1","variationId":"v-c","startedAt":"2026-09-01T10:00:00Z","seconds":300,"score":8},{"id":"p2","variationId":"v-d","startedAt":"2026-09-01T10:05:00Z","seconds":300,"score":6}]}]"#,
    );
    assert_eq!(
        entry.plays.iter().map(|p| p.score).collect::<Vec<_>>(),
        vec![Some(8), Some(6)]
    );
    assert_eq!(entry.plays[1].started_at, at("2026-09-01T10:05:00Z"));
    assert!(entry.plays.iter().all(|p| p.variation_ids.is_empty()));
}

// ── Values the core cannot read (#949) ──

#[test]
fn unknown_values_fall_back_conservatively_and_are_listed() {
    let mut stored = row(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"X","itemType":"klingon","position":0,"durationSecs":0,"status":"quantum"},{"id":"e2","itemId":"i1","itemTitle":"X","itemType":"piece","position":1,"durationSecs":0,"status":"completed","repHistory":["warp"]}]"#,
    );
    stored.completion_status = "enlightenment".to_string();
    let read = session_from_stored(&stored).expect("reads");
    let session = read.session;

    assert_eq!(session.completion_status, CompletionStatus::Completed);
    let first = &session.entries[0];
    assert_eq!(first.status, EntryStatus::NotAttempted, "never completed");
    assert!(first.plays.is_empty(), "an unknown status invents no play");
    assert_eq!(first.item_type, ItemKind::Piece);
    let taps = session.entries[1].plays[0]
        .rep_history
        .clone()
        .expect("taps");
    assert_eq!(taps[0].action, RepAction::Missed, "never a success");
    assert_eq!(
        read.unreadable,
        vec![
            r#"unknown CompletionStatus on decode: "enlightenment""#,
            r#"unknown EntryStatus on decode: "quantum""#,
            r#"unknown ItemKind on decode: "klingon""#,
            r#"unknown RepAction on decode: "warp""#,
        ]
    );
}

#[test]
fn an_unknown_modality_keeps_the_key_without_a_mode() {
    let read = read(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"X","itemType":"piece","position":0,"durationSecs":60,"status":"completed","plays":[{"id":"p1","key":{"key":"Eb","modality":"lydian"},"startedAt":"2026-09-01T10:00:00Z","seconds":60},{"id":"p2","key":{"key":"Eb","modality":"major"},"startedAt":"2026-09-01T10:01:00Z","seconds":60}]}]"#,
    );
    let plays = &read.session.entries[0].plays;
    assert_eq!(
        plays[0].key,
        key_from_stored(Some("Eb"), None),
        "the spelling survives"
    );
    assert_eq!(
        plays[1].key,
        key_from_stored(Some("Eb"), Some(Modality::Major))
    );
    assert_eq!(
        read.unreadable,
        vec![r#"unknown Modality on decode: "lydian""#]
    );
}

#[test]
fn a_bad_time_inside_the_entries_takes_the_session_start_and_is_listed() {
    let read = read(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"X","itemType":"piece","position":0,"durationSecs":60,"status":"completed","plays":[{"id":"p1","startedAt":"yesterday","seconds":60}]}]"#,
    );
    assert_eq!(read.session.entries[0].plays[0].started_at, at(START));
    assert_eq!(read.unreadable, vec![r#"time on decode: "yesterday""#]);
}

#[test]
fn an_unreadable_entries_column_reads_as_no_entries_and_is_listed() {
    let read = read("not json");
    assert!(read.session.entries.is_empty());
    assert_eq!(read.unreadable.len(), 1);
    assert!(read.unreadable[0].starts_with("entries failed to decode"));
}

/// The note leaves the device in a report, so it never quotes what was typed.
#[test]
fn an_unreadable_entries_note_does_not_quote_the_row() {
    let read = read(r#"[{"id":"e1","position":"lighter thumb"}]"#);
    assert_eq!(read.unreadable.len(), 1);
    assert!(!read.unreadable[0].contains("lighter thumb"));
}

#[test]
fn a_session_time_that_does_not_parse_refuses_the_row() {
    let mut stored = row("[]");
    stored.completed_at = "soon".to_string();
    assert_eq!(
        session_from_stored(&stored),
        Err(StoredSessionError::Time {
            id: "s1".to_string(),
            column: "completed_at",
            raw: "soon".to_string()
        })
    );
}

#[test]
fn out_of_range_numbers_clamp() {
    let mut stored = row("[]");
    stored.total_duration_secs = -5;
    stored.session_score = Some(300);
    stored.capture_version = Some(-1);
    let read = session_from_stored(&stored).expect("reads");
    assert_eq!(read.session.total_duration_secs, 0);
    assert_eq!(read.session.session_score, Some(255));
    assert_eq!(read.session.capture_version, Some(0));
    assert_eq!(read.unreadable, vec!["total_duration_secs -5 is negative"]);
}

// ── Writing ──

fn current_session() -> PracticeSession {
    let tap = RepEvent {
        action: RepAction::Undo,
        at: at("2026-09-01T10:01:30.250Z"),
        tempo: Some(96),
        click_sounding: Some(true),
    };
    let play = Play {
        id: "p1".to_string(),
        section_id: Some("sec-b".to_string()),
        key: Key::parse("F# minor"),
        variation_ids: vec!["v-dotted".to_string()],
        started_at: at(START),
        seconds: 240,
        rep_target: Some(5),
        rep_count: Some(3),
        rep_history: Some(vec![tap]),
        tempo_changes: vec![TempoChange {
            at: at("2026-09-01T10:02:00Z"),
            tempo: 104,
            click_sounding: false,
        }],
        achieved_tempo: Some(104),
        click_pattern: Some(ClickState {
            metre: Metre {
                beats: 7,
                unit: 8,
                groups: Some(vec![2, 2, 3]),
            },
            sounding: 0b101,
        }),
        score: Some(4),
        away: Vec::new(),
    };
    let entry = SetlistEntry {
        item_type: ItemKind::Exercise,
        status: EntryStatus::Completed,
        duration_secs: 240,
        notes: Some("lighter thumb".to_string()),
        segments: vec![Segment {
            section_id: "sec-b".to_string(),
            planned_secs: 240,
        }],
        planned_variation_ids: vec!["v-dotted".to_string()],
        planned_rep_target: Some(5),
        plays: vec![play, Play::fixture()],
        ..SetlistEntry::fixture()
    };
    let skipped = SetlistEntry {
        id: "entry-2".to_string(),
        position: 1,
        status: EntryStatus::Skipped,
        ..SetlistEntry::fixture()
    };
    PracticeSession {
        id: "s1".to_string(),
        entries: vec![entry, skipped],
        session_notes: Some("tired".to_string()),
        started_at: at(START),
        completed_at: at("2026-09-01T10:10:00Z"),
        total_duration_secs: 600,
        completion_status: CompletionStatus::EndedEarly,
        session_score: Some(3),
        capture_version: Some(crate::domain::session::CAPTURE_VERSION),
    }
}

#[test]
fn a_current_session_reads_back_as_written() {
    let session = current_session();
    let stored = session_to_stored(&session).expect("writes");
    let read = session_from_stored(&stored).expect("reads");
    assert_eq!(read.session, session);
    assert!(read.unreadable.is_empty());
}

/// A build from before #2234 must still read a row written after it, so the
/// names and the absent-not-null optionals stay what Swift's encoder wrote.
#[test]
fn a_written_row_keeps_the_shape_the_swift_encoder_wrote() {
    let stored = session_to_stored(&current_session()).expect("writes");
    assert_eq!(stored.started_at, START);
    assert_eq!(stored.completion_status, "ended_early");
    let entries: serde_json::Value = serde_json::from_str(&stored.entries).expect("json");
    let skipped = &entries[1];
    assert_eq!(
        skipped,
        &serde_json::json!({
            "id": "entry-2", "itemId": "item-1", "itemTitle": "Item", "itemType": "piece",
            "position": 1, "durationSecs": 0, "status": "skipped",
            "plannedVariationIds": [], "plays": [], "segments": [], "gotInTheWay": [],
            "notePoints": []
        })
    );
    let play = &entries[0]["plays"][0];
    assert_eq!(
        play["key"],
        serde_json::json!({"key": "F#", "modality": "minor"})
    );
    assert_eq!(
        play["repHistory"][0],
        serde_json::json!({"action": "undo", "at": "2026-09-01T10:01:30.250Z", "tempo": 96, "clickSounding": true})
    );
    assert_eq!(
        play["clickPattern"],
        serde_json::json!({"metre": {"beats": 7, "unit": 8, "groups": [2, 2, 3]}, "sounding": 5})
    );
    assert_eq!(
        play["tempoChanges"][0],
        serde_json::json!({"at": "2026-09-01T10:02:00Z", "tempo": 104, "clickSounding": false})
    );
}

/// Every time so far was stored as chrono's serde wrote it.
#[test]
fn a_written_time_is_the_text_chronos_serde_writes() {
    let time = at("2026-09-01T10:01:30.123456789Z");
    assert_eq!(serde_json::to_value(time).expect("json"), time_text(&time));
}

/// Rows on the device are the only copy: a renamed string strands every
/// value written under the old one.
#[test]
fn every_stored_enum_text_is_pinned() {
    assert_eq!(
        [CompletionStatus::Completed, CompletionStatus::EndedEarly]
            .map(|s| completion_status_text(&s)),
        ["completed", "ended_early"]
    );
    assert_eq!(
        [
            EntryStatus::Completed,
            EntryStatus::Skipped,
            EntryStatus::NotAttempted
        ]
        .map(|s| entry_status_text(&s)),
        ["completed", "skipped", "not_attempted"]
    );
    assert_eq!(
        [RepAction::Missed, RepAction::Success, RepAction::Undo].map(rep_action_text),
        ["missed", "success", "undo"]
    );
    assert_eq!(
        [ItemKind::Piece, ItemKind::Exercise].map(|k| item_kind_text(&k)),
        ["piece", "exercise"]
    );
    assert_eq!(
        [Modality::Major, Modality::Minor].map(modality_text),
        ["major", "minor"]
    );
}

#[test]
fn every_stored_enum_text_reads_back() {
    for s in [CompletionStatus::Completed, CompletionStatus::EndedEarly] {
        assert_eq!(completion_status(completion_status_text(&s)), Some(s));
    }
    for s in [
        EntryStatus::Completed,
        EntryStatus::Skipped,
        EntryStatus::NotAttempted,
    ] {
        assert_eq!(entry_status(entry_status_text(&s)), Some(s));
    }
    for a in [RepAction::Missed, RepAction::Success, RepAction::Undo] {
        assert_eq!(rep_action(rep_action_text(a)), Some(a));
    }
    for k in [ItemKind::Piece, ItemKind::Exercise] {
        assert_eq!(item_kind(item_kind_text(&k)), Some(k));
    }
    for m in [Modality::Major, Modality::Minor] {
        assert_eq!(modality(modality_text(m)), Some(m));
    }
}

/// The shape v0.16 wrote (#2246): one planned section, before segments.
const V016_ENTRY: &str = r#"[{"id":"e1","itemId":"i1","itemTitle":"Nocturne","itemType":"piece","position":0,"durationSecs":900,"status":"completed","plannedDurationSecs":900,"plannedSectionIds":["sec-b"],"plannedVariationIds":["v-dotted"],"plays":[{"id":"e1-play","sectionId":"sec-b","variationIds":["v-dotted"],"startedAt":"2026-09-01T10:00:00Z","seconds":900,"tempoChanges":[]}]}]"#;

#[test]
fn a_v016_planned_section_reads_as_one_segment_of_the_planned_time() {
    let read = session_from_stored(&row(V016_ENTRY)).expect("reads");

    let entry = &read.session.entries[0];
    assert_eq!(
        entry.segments,
        [Segment {
            section_id: "sec-b".to_string(),
            planned_secs: 900,
        }]
    );
    assert_eq!(read.unreadable, Vec::<String>::new());
    let rewritten = session_to_stored(&read.session).expect("writes");
    assert!(
        !rewritten.entries.contains("plannedSectionIds"),
        "the old field is read, never written"
    );
}

#[test]
fn an_unknown_finish_answer_is_dropped_and_reported() {
    let read = session_from_stored(&row(
        r#"[{"id":"e1","itemId":"i1","itemTitle":"Nocturne","itemType":"piece","position":0,"durationSecs":60,"status":"completed","felt":"serene","gotInTheWay":["memory","weather"],"intentionMet":"mostly"}]"#,
    ))
    .expect("reads");

    let entry = &read.session.entries[0];
    assert_eq!(entry.felt, None);
    assert_eq!(entry.got_in_the_way, [Obstacle::Memory]);
    assert_eq!(entry.intention_met, None);
    assert_eq!(read.unreadable.len(), 3, "{:?}", read.unreadable);
}

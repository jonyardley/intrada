use super::*;
use crate::app::{Event, Intrada};
use crate::domain::item::Item;
use crate::domain::types::assert_round_trips;
use crate::model::{FitOffer, FitState, FitView};
use crux_core::App;

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(1_790_000_000, 0).expect("in range")
}

fn send(m: &mut Model, event: SessionEvent) {
    let _ = Intrada.update(Event::Session(event), m);
}

fn exercise(id: &str, title: &str) -> Item {
    let mut item = Item::fixture(id);
    item.title = title.to_string();
    item.kind = ItemKind::Exercise;
    item
}

/// Four items never played, so each is 10 usual minutes and ranks by title.
fn building(length_mins: Option<u16>) -> Model {
    let mut m = Model {
        items: vec![
            exercise("a", "Arpeggios"),
            exercise("b", "Broken chords"),
            exercise("c", "Chromatic scale"),
            exercise("d", "Dominant sevenths"),
        ]
        .into(),
        ..Default::default()
    };
    send(&mut m, SessionEvent::StartBuilding);
    send(&mut m, SessionEvent::SetSessionLength { length_mins });
    m
}

fn add_timed(m: &mut Model, item_id: &str, minutes: Option<u32>) {
    send(
        m,
        SessionEvent::AddToSetlist {
            item_id: item_id.to_string(),
        },
    );
    let entry_id = entries(m).last().expect("added").id.clone();
    send(
        m,
        SessionEvent::SetEntryDuration {
            entry_id,
            duration_secs: minutes.map(|mins| mins * 60),
        },
    );
}

fn entries(m: &Model) -> Vec<SetlistEntry> {
    match &m.session_status {
        SessionStatus::Building(b) => b.entries.clone(),
        other => panic!("not building: {other:?}"),
    }
}

fn planned(m: &Model) -> Vec<(String, Option<u32>)> {
    entries(m)
        .into_iter()
        .map(|e| (e.plan.item_id, e.plan.duration_secs.map(|s| s / 60)))
        .collect()
}

fn fit(m: &Model) -> Option<FitView> {
    Intrada.view(m).building_setlist.expect("building view").fit
}

fn row(id: &str, minutes: Option<u32>) -> (String, Option<u32>) {
    (id.to_string(), minutes)
}

#[test]
fn a_full_plan_with_one_more_item_reads_over_and_trims_back_to_the_length() {
    let mut m = building(Some(60));
    add_timed(&mut m, "a", Some(60));
    add_timed(&mut m, "b", Some(12));

    let over = fit(&m).expect("fit");
    assert_eq!(over.summary, "72 of 60 min");
    assert_eq!(over.state, FitState::Over);
    assert_eq!(over.offers, [FitOffer::TrimToFit, FitOffer::DropLastItem]);

    send(&mut m, SessionEvent::TrimToFit);

    assert_eq!(m.last_error, None);
    assert_eq!(planned(&m), [row("a", Some(50)), row("b", Some(10))]);
    let fitted = fit(&m).expect("fit");
    assert_eq!(fitted.used_mins, 60);
    assert_eq!(fitted.state, FitState::Exact);
    assert!(fitted.offers.is_empty());
}

#[test]
fn trim_leaves_a_plan_that_is_not_over_alone() {
    let mut m = building(Some(60));
    add_timed(&mut m, "a", Some(15));

    send(&mut m, SessionEvent::TrimToFit);

    assert_eq!(m.last_error, None);
    assert_eq!(planned(&m), [row("a", Some(15))]);
}

#[test]
fn a_short_plan_offers_fill_and_stretch_and_stretch_fills_the_length() {
    let mut m = building(Some(60));
    add_timed(&mut m, "a", Some(10));
    add_timed(&mut m, "b", Some(5));
    add_timed(&mut m, "c", None);

    let under = fit(&m).expect("fit");
    assert_eq!(under.summary, "15 of 60 min, plus 1 untimed");
    assert_eq!(under.state, FitState::Under);
    assert_eq!(
        under.offers,
        [FitOffer::FillWithSuggestions, FitOffer::StretchToFit]
    );

    send(&mut m, SessionEvent::StretchToFit);

    assert_eq!(
        planned(&m),
        [row("a", Some(40)), row("b", Some(20)), row("c", None)]
    );
}

#[test]
fn stretch_stops_at_sixty_minutes_a_row() {
    let mut m = building(Some(120));
    add_timed(&mut m, "a", Some(45));

    send(&mut m, SessionEvent::StretchToFit);

    assert_eq!(planned(&m), [row("a", Some(60))]);
    assert_eq!(
        fit(&m).expect("fit").offers,
        [FitOffer::FillWithSuggestions]
    );
}

#[test]
fn fill_adds_ranked_items_not_yet_planned_while_they_fit() {
    let mut m = building(Some(25));
    add_timed(&mut m, "a", Some(10));

    send(&mut m, SessionEvent::FillWithSuggestions { now: now() });

    assert_eq!(m.last_error, None);
    assert_eq!(planned(&m), [row("a", Some(10)), row("b", Some(10))]);
}

#[test]
fn fill_adds_nothing_when_nothing_fits_and_is_then_not_offered() {
    let mut m = building(Some(15));
    add_timed(&mut m, "a", Some(10));

    assert_eq!(fit(&m).expect("fit").offers, [FitOffer::StretchToFit]);

    send(&mut m, SessionEvent::FillWithSuggestions { now: now() });

    assert_eq!(planned(&m), [row("a", Some(10))]);
}

fn group_last_two_as_a_block(m: &mut Model) {
    let SessionStatus::Building(b) = &mut m.session_status else {
        panic!("not building");
    };
    let n = b.entries.len();
    for e in &mut b.entries[n - 2..] {
        e.plan.group_id = Some("g1".to_string());
    }
    b.entries[n - 1].plan.item_type = ItemKind::Piece;
}

#[test]
fn drop_last_item_drops_the_whole_block_when_the_last_row_is_a_grouped_piece() {
    let mut m = building(Some(30));
    add_timed(&mut m, "a", Some(20));
    add_timed(&mut m, "b", Some(10));
    add_timed(&mut m, "c", Some(15));
    group_last_two_as_a_block(&mut m);

    assert_eq!(
        fit(&m).expect("fit").offers,
        [FitOffer::TrimToFit, FitOffer::DropLastItem]
    );

    send(&mut m, SessionEvent::DropLastItem);

    assert_eq!(m.last_error, None);
    assert_eq!(planned(&m), [row("a", Some(20))]);
    assert_eq!(entries(&m)[0].position, 0);
}

#[test]
fn drop_last_item_drops_a_standalone_last_row() {
    let mut m = building(Some(20));
    add_timed(&mut m, "a", Some(20));
    add_timed(&mut m, "b", Some(10));

    send(&mut m, SessionEvent::DropLastItem);

    assert_eq!(planned(&m), [row("a", Some(20))]);
}

#[test]
fn drop_last_item_leaves_an_empty_plan_alone() {
    let mut m = building(Some(20));

    send(&mut m, SessionEvent::DropLastItem);

    assert_eq!(m.last_error, None);
    assert!(planned(&m).is_empty());
}

#[test]
fn without_a_length_there_is_no_fit_and_the_events_are_refused() {
    let mut m = building(None);
    add_timed(&mut m, "a", Some(10));

    assert_eq!(fit(&m), None);
    for event in [
        SessionEvent::TrimToFit,
        SessionEvent::StretchToFit,
        SessionEvent::FillWithSuggestions { now: now() },
    ] {
        m.last_error = None;
        send(&mut m, event);
        assert_eq!(m.last_error.as_deref(), Some("Set a session length first"));
        assert_eq!(planned(&m), [row("a", Some(10))]);
    }
}

#[test]
fn the_fit_events_are_refused_outside_the_builder() {
    let mut m = Model::default();
    for event in [
        SessionEvent::DropLastItem,
        SessionEvent::TrimToFit,
        SessionEvent::StretchToFit,
        SessionEvent::FillWithSuggestions { now: now() },
    ] {
        m.last_error = None;
        send(&mut m, event);
        assert_eq!(
            m.last_error.as_deref(),
            Some("A plan can only be fitted while building")
        );
        assert!(matches!(m.session_status, SessionStatus::Idle));
    }
}

#[test]
fn the_fit_events_and_view_round_trip_on_ffi_bincode_wire() {
    assert_round_trips(Event::Session(SessionEvent::TrimToFit));
    assert_round_trips(Event::Session(SessionEvent::StretchToFit));
    assert_round_trips(Event::Session(SessionEvent::DropLastItem));
    assert_round_trips(Event::Session(SessionEvent::FillWithSuggestions {
        now: now(),
    }));
    for state in [FitState::Under, FitState::Exact, FitState::Over] {
        assert_round_trips(state);
    }
    for offer in [
        FitOffer::FillWithSuggestions,
        FitOffer::StretchToFit,
        FitOffer::TrimToFit,
        FitOffer::DropLastItem,
    ] {
        assert_round_trips(offer);
    }
    assert_round_trips(FitView {
        used_mins: 72,
        length_mins: 60,
        state: FitState::Over,
        summary: "72 of 60 min, plus 2 untimed".to_string(),
        offers: vec![FitOffer::TrimToFit, FitOffer::DropLastItem],
        state_line: "12 min over".to_string(),
    });
}

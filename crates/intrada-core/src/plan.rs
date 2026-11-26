//! Today's plan from the time the musician has (#3007): a warm-up, then the
//! library by need (T42), fitted to the time. Pure and clock-injected.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::analytics::LocalClock;
use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::practice_defaults::PracticeDefaults;
use crate::model::LibraryItemView;
use crate::ranking::{self, PracticeRecords, Ranked};

/// About one key for every this many minutes of a row (#3008).
const MINUTES_PER_KEY: u32 = 3;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlanRowView {
    pub item_id: String,
    pub title: String,
    /// "Piece · a priority"; the warm-up row says only its kind.
    pub reason: String,
    pub minutes: u32,
    pub kind: ItemKind,
    /// Every key the item keeps, neediest first; the ticked ones are today's.
    pub keys: Vec<PlanKeyView>,
    /// Why the suggested keys were picked; empty with no keys.
    pub keys_reason: String,
}

/// One of a row's keys in today's plan (#3008).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlanKeyView {
    pub key: Key,
    pub label: String,
    pub ticked: bool,
    pub suggested: bool,
    pub score: Option<u8>,
    pub last_played: String,
}

/// The keys ticked for today's plan by item id, for the local day they were
/// ticked; never saved (#3008).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TodaysKeys {
    day: Option<NaiveDate>,
    by_item: HashMap<String, Vec<Key>>,
}

impl TodaysKeys {
    /// Empty `ticks` brings back the suggestion.
    pub(crate) fn set(&mut self, day: NaiveDate, item_id: String, ticks: Vec<Key>) {
        if self.day != Some(day) {
            *self = TodaysKeys {
                day: Some(day),
                by_item: HashMap::new(),
            };
        }
        if ticks.is_empty() {
            self.by_item.remove(&item_id);
        } else {
            self.by_item.insert(item_id, ticks);
        }
    }

    pub(crate) fn on(&self, day: NaiveDate) -> Option<&HashMap<String, Vec<Key>>> {
        (self.day == Some(day)).then_some(&self.by_item)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlanView {
    pub length_mins: u16,
    pub warm_up: Option<PlanRowView>,
    /// Some item is marked for the warm-up, so the empty hint can hide.
    pub has_warm_up_items: bool,
    pub warm_up_enabled: bool,
    pub rows: Vec<PlanRowView>,
    /// The warm-up and the rows together.
    pub total_mins: u32,
    pub start_title: String,
}

impl PlanView {
    /// The order a seeded session plays them in.
    pub(crate) fn played_rows(&self) -> impl Iterator<Item = &PlanRowView> {
        self.warm_up.iter().chain(&self.rows)
    }
}

/// `None` only for an empty library; what does not fit is skipped, never shrunk.
pub(crate) fn fill(
    items: &[LibraryItemView],
    records: &PracticeRecords,
    defaults: &PracticeDefaults,
    length_mins: u16,
    ticks: &TodaysKeys,
    clock: LocalClock,
) -> Option<PlanView> {
    let ticks = ticks.on(clock.today);
    let plan_row = |item: &LibraryItemView, reason: String| {
        let ticked = ticks.and_then(|t| t.get(&item.id)).map_or(&[][..], |t| t);
        row(item, reason, todays_keys(item, records, ticked, clock))
    };
    let ranked = ranking::rank_items(items, records, clock);
    let neediest = ranked.first()?;
    let length = u32::from(length_mins);
    let warm_up_on = defaults.start_with_warm_up;
    let chosen = ranked
        .iter()
        .find(|r| warm_up_on && r.item.include_in_warm_up && r.item.usual_minutes <= length);
    let mut total = chosen.map_or(0, |r| r.item.usual_minutes);
    let warm_up = chosen.map(|r| plan_row(r.item, kind_word(&r.item.item_type).to_string()));
    let mut rows = Vec::new();

    for candidate in &ranked {
        let item = candidate.item;
        if chosen.is_some_and(|c| c.item.id == item.id) {
            continue;
        }
        let spare_warm_up = warm_up_on && item.include_in_warm_up;
        if spare_warm_up && !item.priority && !candidate.is_due() {
            continue;
        }
        if total + item.usual_minutes <= length {
            total += item.usual_minutes;
            rows.push(plan_row(item, reason(candidate)));
        }
    }

    if warm_up.is_none() && rows.is_empty() {
        total += neediest.item.usual_minutes;
        rows.push(plan_row(neediest.item, reason(neediest)));
    }

    Some(PlanView {
        length_mins,
        warm_up,
        has_warm_up_items: items.iter().any(|i| i.include_in_warm_up),
        warm_up_enabled: defaults.start_with_warm_up,
        rows,
        total_mins: total,
        start_title: format!("Start session · {total} min"),
    })
}

fn row(item: &LibraryItemView, reason: String, keys: Vec<PlanKeyView>) -> PlanRowView {
    PlanRowView {
        item_id: item.id.clone(),
        title: item.title.clone(),
        reason,
        minutes: item.usual_minutes,
        kind: item.item_type.clone(),
        keys_reason: keys_reason(&keys, item.usual_minutes),
        keys,
    }
}

/// Every kept key in need order, the neediest few suggested; a tick for a key
/// the item no longer keeps is dropped, and no ticks left means the suggestion.
fn todays_keys(
    item: &LibraryItemView,
    records: &PracticeRecords,
    ticked: &[Key],
    clock: LocalClock,
) -> Vec<PlanKeyView> {
    let ranked = ranking::rank_keys(item, records, clock);
    let suggested = key_count(item.usual_minutes, ranked.len());
    let kept: Vec<Key> = ranked.iter().map(|k| k.key).collect();
    let ticked = crate::validation::kept_ticks(&kept, ticked);
    ranked
        .into_iter()
        .enumerate()
        .map(|(i, k)| PlanKeyView {
            ticked: if ticked.is_empty() {
                i < suggested
            } else {
                ticked.contains(&k.key)
            },
            suggested: i < suggested,
            key: k.key,
            label: k.label,
            score: k.score,
            last_played: k.last_played,
        })
        .collect()
}

/// Round to nearest, at least one, never more than the item keeps.
fn key_count(minutes: u32, kept: usize) -> usize {
    if kept == 0 {
        return 0;
    }
    (((minutes + MINUTES_PER_KEY / 2) / MINUTES_PER_KEY) as usize).clamp(1, kept)
}

fn keys_reason(keys: &[PlanKeyView], minutes: u32) -> String {
    let count = keys.iter().filter(|k| k.suggested).count() as u32;
    if count == 0 {
        return String::new();
    }
    let each = ((minutes + count / 2) / count).max(1);
    let unit = if each == 1 { "minute" } else { "minutes" };
    match count {
        1 => format!("The key that needs it most, about {each} {unit}"),
        n => format!(
            "The {} keys that need it most, about {each} {unit} each",
            number_word(n)
        ),
    }
}

fn number_word(n: u32) -> String {
    const WORDS: [&str; 11] = [
        "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    ];
    n.checked_sub(2)
        .and_then(|i| WORDS.get(i as usize))
        .map_or_else(|| n.to_string(), |w| (*w).to_string())
}

fn reason(ranked: &Ranked) -> String {
    format!(
        "{} · {}",
        kind_word(&ranked.item.item_type),
        ranked.reason()
    )
}

fn kind_word(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Piece => "Piece",
        ItemKind::Exercise => "Exercise",
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemPracticeSummary;
    use chrono::NaiveDate;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 27).expect("valid date")
    }

    fn clock() -> LocalClock {
        LocalClock {
            today: today(),
            utc_offset_minutes: 0,
        }
    }

    fn item(id: &str, kind: ItemKind, minutes: u32) -> LibraryItemView {
        let mut item = LibraryItemView::fixture(id, &format!("Title {id}"), kind);
        item.usual_minutes = minutes;
        item
    }

    /// Practised once with no mark, so 10 days is the interval: 25 days ago
    /// is cold, 15 going cold, 3 fresh.
    fn last_played(mut item: LibraryItemView, days_ago: i64) -> LibraryItemView {
        item.practice = Some(ItemPracticeSummary {
            session_count: 1,
            last_practiced_at: Some(
                (today() - chrono::Duration::days(days_ago))
                    .and_hms_opt(10, 0, 0)
                    .expect("valid time")
                    .and_utc()
                    .to_rfc3339(),
            ),
            ..ItemPracticeSummary::fixture()
        });
        item
    }

    fn starred(mut item: LibraryItemView) -> LibraryItemView {
        item.priority = true;
        item
    }

    fn warm_up(mut item: LibraryItemView) -> LibraryItemView {
        item.include_in_warm_up = true;
        item
    }

    /// A 5 minute warm-up, a 10 minute priority, a 15 minute cold piece that
    /// misses 20 minutes and a 5 minute exercise going cold that fits after it.
    fn library() -> Vec<LibraryItemView> {
        vec![
            warm_up(item("scales", ItemKind::Exercise, 5)),
            starred(item("sonata", ItemKind::Piece, 10)),
            last_played(item("nocturne", ItemKind::Piece, 15), 25),
            last_played(item("trills", ItemKind::Exercise, 5), 15),
            last_played(item("etude", ItemKind::Piece, 5), 3),
        ]
    }

    fn filled(items: &[LibraryItemView], length: u16) -> PlanView {
        fill(
            items,
            &PracticeRecords::new(),
            &PracticeDefaults::default(),
            length,
            &TodaysKeys::default(),
            clock(),
        )
        .expect("a plan")
    }

    fn ids(plan: &PlanView) -> Vec<&str> {
        plan.played_rows().map(|r| r.item_id.as_str()).collect()
    }

    #[test]
    fn an_empty_library_has_no_plan() {
        assert_eq!(
            fill(
                &[],
                &PracticeRecords::new(),
                &PracticeDefaults::default(),
                20,
                &TodaysKeys::default(),
                clock()
            ),
            None
        );
    }

    #[test]
    fn twenty_minutes_skips_what_does_not_fit_and_takes_what_does() {
        let plan = filled(&library(), 20);

        assert_eq!(ids(&plan), ["scales", "sonata", "trills"]);
        assert_eq!(plan.total_mins, 20);
        assert_eq!(plan.length_mins, 20);
    }

    #[test]
    fn thirty_minutes_makes_room_for_the_longer_cold_piece() {
        let plan = filled(&library(), 30);

        assert_eq!(ids(&plan), ["scales", "sonata", "nocturne"]);
        assert_eq!(plan.total_mins, 30);
    }

    #[test]
    fn the_warm_up_switch_off_gives_no_warm_up() {
        let defaults = PracticeDefaults {
            start_with_warm_up: false,
            ..PracticeDefaults::default()
        };
        let plan = fill(
            &library(),
            &PracticeRecords::new(),
            &defaults,
            20,
            &TodaysKeys::default(),
            clock(),
        )
        .expect("a plan");

        assert_eq!(plan.warm_up, None);
        assert!(!plan.warm_up_enabled);
        assert!(plan.has_warm_up_items);
    }

    #[test]
    fn with_the_warm_up_off_a_warm_up_item_fills_like_any_other() {
        let defaults = PracticeDefaults {
            start_with_warm_up: false,
            ..PracticeDefaults::default()
        };
        let items = vec![
            last_played(item("concerto", ItemKind::Piece, 30), 3),
            last_played(warm_up(item("scales", ItemKind::Exercise, 5)), 1),
        ];
        let plan = fill(
            &items,
            &PracticeRecords::new(),
            &defaults,
            20,
            &TodaysKeys::default(),
            clock(),
        )
        .expect("a plan");

        assert_eq!(ids(&plan), ["scales"]);
        assert_eq!(plan.total_mins, 5);
    }

    #[test]
    fn the_most_overdue_warm_up_item_is_the_one_chosen() {
        let items = vec![
            last_played(warm_up(item("recent", ItemKind::Exercise, 5)), 3),
            last_played(warm_up(item("stale", ItemKind::Exercise, 5)), 25),
        ];
        let plan = filled(&items, 20);

        assert_eq!(plan.warm_up.expect("a warm-up").item_id, "stale");
    }

    #[test]
    fn a_warm_up_item_too_long_for_the_length_gives_way_to_one_that_fits() {
        let items = vec![
            last_played(warm_up(item("long", ItemKind::Exercise, 30)), 25),
            last_played(warm_up(item("short", ItemKind::Exercise, 5)), 15),
        ];
        let plan = filled(&items, 20);

        assert_eq!(plan.warm_up.expect("a warm-up").item_id, "short");
    }

    #[test]
    fn a_warm_up_that_fills_the_length_stands_alone() {
        let items = vec![
            warm_up(item("scales", ItemKind::Exercise, 10)),
            starred(item("sonata", ItemKind::Piece, 10)),
            last_played(item("nocturne", ItemKind::Piece, 10), 25),
        ];
        let plan = filled(&items, 10);

        assert_eq!(ids(&plan), ["scales"]);
        assert!(plan.rows.is_empty());
        assert_eq!(plan.total_mins, 10);
    }

    #[test]
    fn no_warm_up_items_leaves_the_hint_to_show() {
        let items = vec![starred(item("sonata", ItemKind::Piece, 10))];
        let plan = filled(&items, 20);

        assert_eq!(plan.warm_up, None);
        assert!(!plan.has_warm_up_items);
    }

    #[test]
    fn a_starred_warm_up_item_wins_the_warm_up_over_a_colder_one() {
        let items = vec![
            last_played(warm_up(item("cold", ItemKind::Exercise, 5)), 25),
            last_played(starred(warm_up(item("starred", ItemKind::Exercise, 5))), 1),
        ];
        let plan = filled(&items, 20);

        assert_eq!(plan.warm_up.expect("a warm-up").item_id, "starred");
    }

    #[test]
    fn a_spare_starred_warm_up_item_takes_its_place_among_the_priorities() {
        let items = vec![
            last_played(starred(warm_up(item("first", ItemKind::Exercise, 5))), 25),
            last_played(starred(warm_up(item("spare", ItemKind::Exercise, 5))), 1),
            last_played(starred(item("sonata", ItemKind::Piece, 5)), 25),
        ];
        let plan = filled(&items, 20);

        assert_eq!(ids(&plan), ["first", "sonata", "spare"]);
    }

    #[test]
    fn a_spare_warm_up_item_fills_only_when_it_is_going_cold() {
        let items = vec![
            last_played(warm_up(item("chosen", ItemKind::Exercise, 5)), 25),
            last_played(warm_up(item("cold", ItemKind::Exercise, 5)), 15),
            last_played(warm_up(item("fresh", ItemKind::Exercise, 5)), 1),
            warm_up(item("new", ItemKind::Exercise, 5)),
        ];
        let plan = filled(&items, 30);

        assert_eq!(ids(&plan), ["chosen", "cold"]);
    }

    #[test]
    fn an_item_that_qualifies_twice_appears_once() {
        let items = vec![last_played(
            starred(warm_up(item("sonata", ItemKind::Piece, 5))),
            25,
        )];
        let plan = filled(&items, 30);

        assert_eq!(ids(&plan), ["sonata"]);
        assert_eq!(plan.total_mins, 5);
    }

    #[test]
    fn a_priority_comes_before_a_colder_item() {
        let items = vec![
            last_played(item("cold", ItemKind::Piece, 5), 60),
            last_played(starred(item("starred", ItemKind::Piece, 5)), 1),
        ];
        assert_eq!(ids(&filled(&items, 20)), ["starred", "cold"]);
    }

    #[test]
    fn going_cold_follows_cold_by_how_overdue_it_is() {
        let items = vec![
            last_played(item("going", ItemKind::Piece, 5), 15),
            last_played(item("cold", ItemKind::Piece, 5), 25),
        ];
        assert_eq!(ids(&filled(&items, 20)), ["cold", "going"]);
    }

    #[test]
    fn a_library_of_fresh_items_still_fills_the_length() {
        let items = vec![
            last_played(item("etude", ItemKind::Piece, 10), 1),
            last_played(item("arpeggios", ItemKind::Exercise, 5), 2),
            last_played(item("nocturne", ItemKind::Piece, 8), 3),
            last_played(item("trills", ItemKind::Exercise, 4), 4),
        ];
        let plan = filled(&items, 20);
        let longest = items.iter().map(|i| i.usual_minutes).max().unwrap_or(0);

        assert!(plan.total_mins <= 20);
        assert!(plan.total_mins + longest > 20, "{plan:?}");
    }

    #[test]
    fn a_fresh_row_says_when_it_was_last_played() {
        let items = vec![
            last_played(item("etude", ItemKind::Piece, 5), 3),
            item("new", ItemKind::Exercise, 5),
        ];
        let plan = filled(&items, 20);
        let reasons: Vec<&str> = plan.rows.iter().map(|r| r.reason.as_str()).collect();

        assert_eq!(
            reasons,
            [
                "Exercise · not practised yet",
                "Piece · practised 3 days ago"
            ]
        );
    }

    #[test]
    fn each_row_carries_its_kind() {
        let plan = filled(&library(), 20);
        let kinds: Vec<&ItemKind> = plan.played_rows().map(|r| &r.kind).collect();

        assert_eq!(
            kinds,
            [&ItemKind::Exercise, &ItemKind::Piece, &ItemKind::Exercise]
        );
    }

    #[test]
    fn a_fresh_library_still_offers_one_row() {
        let items = vec![last_played(item("etude", ItemKind::Piece, 10), 1)];
        let plan = filled(&items, 20);

        assert_eq!(ids(&plan), ["etude"]);
    }

    #[test]
    fn an_item_longer_than_the_length_stands_alone_when_nothing_fits() {
        let items = vec![starred(item("concerto", ItemKind::Piece, 40))];
        let plan = filled(&items, 20);

        assert_eq!(ids(&plan), ["concerto"]);
        assert_eq!(plan.total_mins, 40);
    }

    #[test]
    fn each_row_says_why_in_the_core_s_words() {
        let plan = filled(&library(), 20);
        let reasons: Vec<&str> = plan.played_rows().map(|r| r.reason.as_str()).collect();

        assert_eq!(
            reasons,
            [
                "Exercise",
                "Piece · a priority",
                "Exercise · going cold after 2 weeks"
            ]
        );
    }

    #[test]
    fn the_start_button_names_the_planned_total_not_the_length() {
        let plan = filled(&library(), 23);

        assert_eq!(plan.total_mins, 20);
        assert_eq!(plan.start_title, "Start session · 20 min");
    }

    #[test]
    fn the_plan_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(filled(&library(), 20));
    }

    // ── Today's keys (#3008) ──

    fn key(name: &str) -> Key {
        Key::parse(name).expect("a key")
    }

    fn scales() -> (LibraryItemView, PracticeRecords) {
        let mut scales = last_played(item("scales", ItemKind::Exercise, 9), 1);
        if let Some(practice) = scales.practice.as_mut() {
            practice.session_count = 6;
            practice.latest_score = Some(9);
        }
        let mut records = PracticeRecords::new();
        for (name, score, days) in [
            ("C major", Some(9), Some(1)),
            ("G major", Some(3), Some(14)),
            ("D major", Some(8), Some(20)),
            ("A major", Some(2), Some(2)),
            ("E major", None, None),
        ] {
            scales.keys.push(crate::model::ItemKeyView {
                key: key(name),
                label: key(name).label(),
                latest_score: score,
                caption: String::new(),
            });
            if let Some(days) = days {
                records
                    .entry("scales".to_string())
                    .or_default()
                    .keys
                    .entry(key(name))
                    .or_default()
                    .add_session(
                        (today() - chrono::Duration::days(days))
                            .and_hms_opt(10, 0, 0)
                            .expect("valid time")
                            .and_utc(),
                    );
            }
        }
        (scales, records)
    }

    fn scales_row(minutes: u32, ticks: &TodaysKeys) -> PlanRowView {
        let (mut scales, records) = scales();
        scales.usual_minutes = minutes;
        let plan = fill(
            &[scales],
            &records,
            &PracticeDefaults::default(),
            120,
            ticks,
            clock(),
        )
        .expect("a plan");
        let row = plan.played_rows().next().expect("a row").clone();
        row
    }

    fn labels(row: &PlanRowView) -> Vec<&str> {
        row.keys
            .iter()
            .filter(|k| k.ticked)
            .map(|k| k.label.as_str())
            .collect()
    }

    fn ticked(names: &[&str]) -> TodaysKeys {
        let mut ticks = TodaysKeys::default();
        ticks.set(
            today(),
            "scales".to_string(),
            names.iter().map(|n| key(n)).collect(),
        );
        ticks
    }

    #[test]
    fn about_one_key_for_every_three_minutes() {
        for (minutes, kept, expected) in [(2, 12, 1), (5, 12, 2), (9, 12, 3), (30, 12, 10)] {
            assert_eq!(key_count(minutes, kept), expected, "{minutes} minutes");
        }
        assert_eq!(key_count(30, 5), 5);
        assert_eq!(key_count(1, 5), 1);
        assert_eq!(key_count(9, 0), 0);
    }

    #[test]
    fn a_nine_minute_row_suggests_its_three_neediest_keys() {
        let row = scales_row(9, &TodaysKeys::default());

        assert_eq!(labels(&row), ["G major", "D major", "A major"]);
        let all: Vec<(&str, bool, Option<u8>, &str)> = row
            .keys
            .iter()
            .map(|k| {
                (
                    k.label.as_str(),
                    k.suggested,
                    k.score,
                    k.last_played.as_str(),
                )
            })
            .collect();
        assert_eq!(
            all,
            [
                ("G major", true, Some(3), "2 weeks ago"),
                ("D major", true, Some(8), "3 weeks ago"),
                ("A major", true, Some(2), "2 days ago"),
                ("C major", false, Some(9), "yesterday"),
                ("E major", false, None, "not practised yet"),
            ]
        );
        assert_eq!(
            row.keys_reason,
            "The three keys that need it most, about 3 minutes each"
        );
    }

    #[test]
    fn the_row_s_minutes_set_how_many_keys_are_suggested() {
        assert_eq!(labels(&scales_row(2, &TodaysKeys::default())), ["G major"]);
        assert_eq!(
            scales_row(2, &TodaysKeys::default()).keys_reason,
            "The key that needs it most, about 2 minutes"
        );
        assert_eq!(
            labels(&scales_row(5, &TodaysKeys::default())),
            ["G major", "D major"]
        );
        assert_eq!(labels(&scales_row(30, &TodaysKeys::default())).len(), 5);
    }

    #[test]
    fn ticks_change_the_labels_but_not_the_minutes() {
        let row = scales_row(9, &ticked(&["E major", "C major"]));

        assert_eq!(labels(&row), ["C major", "E major"]);
        assert_eq!(row.minutes, 9);
        assert!(row.keys.iter().take(3).all(|k| k.suggested));
    }

    #[test]
    fn a_tick_for_a_key_the_item_does_not_keep_is_dropped() {
        assert_eq!(
            labels(&scales_row(9, &ticked(&["B major", "C major"]))),
            ["C major"]
        );
        assert_eq!(
            labels(&scales_row(9, &ticked(&["B major"]))),
            ["G major", "D major", "A major"]
        );
    }

    #[test]
    fn yesterday_s_ticks_bring_back_the_suggestion() {
        let mut ticks = TodaysKeys::default();
        ticks.set(
            today() - chrono::Duration::days(1),
            "scales".to_string(),
            vec![key("C major")],
        );

        assert_eq!(
            labels(&scales_row(9, &ticks)),
            ["G major", "D major", "A major"]
        );
    }

    #[test]
    fn the_warm_up_row_carries_its_keys_too() {
        let (scales, records) = scales();
        let plan = fill(
            &[warm_up(scales)],
            &records,
            &PracticeDefaults::default(),
            20,
            &TodaysKeys::default(),
            clock(),
        )
        .expect("a plan");

        let warm_up = plan.warm_up.expect("a warm-up");
        assert_eq!(labels(&warm_up), ["G major", "D major", "A major"]);
    }

    #[test]
    fn an_item_with_no_keys_has_no_labels() {
        let plan = filled(&library(), 20);

        assert!(plan.played_rows().all(|r| r.keys.is_empty()));
        assert!(plan.played_rows().all(|r| r.keys_reason.is_empty()));
    }

    #[test]
    fn a_plan_with_keys_round_trips_on_ffi_bincode_wire() {
        let row = scales_row(9, &ticked(&["C major"]));
        assert!(!row.keys.is_empty());
        crate::domain::types::assert_round_trips(row.keys[0].clone());
        crate::domain::types::assert_round_trips(row);
    }
}

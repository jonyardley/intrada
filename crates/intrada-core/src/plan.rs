//! Today's plan from the time the musician has (#3007): a warm-up, priorities,
//! then what is going cold. Pure and clock-injected like `suggestion`.

use std::cmp::Reverse;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::analytics::LocalClock;
use crate::domain::item::ItemKind;
use crate::domain::practice_defaults::PracticeDefaults;
use crate::model::LibraryItemView;
use crate::suggestion::staleness_of;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlanRowView {
    pub item_id: String,
    pub title: String,
    /// "Piece · a priority"; the warm-up row says only its kind.
    pub reason: String,
    pub minutes: u32,
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

/// `None` only for an empty library. An item that does not fit is skipped,
/// never shrunk; when nothing fits, the first candidate stands alone.
pub(crate) fn fill(
    items: &[LibraryItemView],
    defaults: &PracticeDefaults,
    length_mins: u16,
    clock: LocalClock,
) -> Option<PlanView> {
    if items.is_empty() {
        return None;
    }
    let length = u32::from(length_mins);
    let most_overdue_first = |list: &mut Vec<&LibraryItemView>| {
        list.sort_by_cached_key(|i| {
            (
                Reverse(staleness_of(i, clock).overdue_key()),
                i.title.to_lowercase(),
                i.id.clone(),
            )
        });
    };

    let mut taken: HashSet<&str> = HashSet::new();
    let mut total = 0u32;

    let warm_up = if defaults.start_with_warm_up {
        let mut pool: Vec<&LibraryItemView> =
            items.iter().filter(|i| i.include_in_warm_up).collect();
        most_overdue_first(&mut pool);
        pool.iter().find(|i| i.usual_minutes <= length).map(|item| {
            taken.insert(item.id.as_str());
            total += item.usual_minutes;
            row(item, kind_word(&item.item_type).to_string())
        })
    } else {
        None
    };

    let mut cold: Vec<&LibraryItemView> = items
        .iter()
        .filter(|i| {
            let staleness = staleness_of(i, clock);
            !staleness.is_fresh() && staleness.days().is_some()
        })
        .collect();
    most_overdue_first(&mut cold);
    let candidates = crate::priorities::order_priorities(items, clock)
        .into_iter()
        .chain(cold);

    let mut rows = Vec::new();
    for item in candidates {
        if taken.contains(item.id.as_str()) {
            continue;
        }
        if total + item.usual_minutes > length {
            continue;
        }
        taken.insert(item.id.as_str());
        total += item.usual_minutes;
        rows.push(row(item, reason(item, clock)));
    }

    if warm_up.is_none() && rows.is_empty() {
        let mut rest: Vec<&LibraryItemView> = items
            .iter()
            .filter(|i| !taken.contains(i.id.as_str()))
            .collect();
        most_overdue_first(&mut rest);
        let pick = rest
            .iter()
            .find(|i| total + i.usual_minutes <= length)
            .or(rest.first());
        if let Some(item) = pick {
            total += item.usual_minutes;
            rows.push(row(item, reason(item, clock)));
        }
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

fn row(item: &LibraryItemView, reason: String) -> PlanRowView {
    PlanRowView {
        item_id: item.id.clone(),
        title: item.title.clone(),
        reason,
        minutes: item.usual_minutes,
    }
}

fn reason(item: &LibraryItemView, clock: LocalClock) -> String {
    let kind = kind_word(&item.item_type);
    if item.priority {
        format!("{kind} · a priority")
    } else {
        format!("{kind} · {}", staleness_of(item, clock).clause())
    }
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
        fill(items, &PracticeDefaults::default(), length, clock()).expect("a plan")
    }

    fn ids(plan: &PlanView) -> Vec<&str> {
        plan.played_rows().map(|r| r.item_id.as_str()).collect()
    }

    #[test]
    fn an_empty_library_has_no_plan() {
        assert_eq!(fill(&[], &PracticeDefaults::default(), 20, clock()), None);
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
        let plan = fill(&library(), &defaults, 20, clock()).expect("a plan");

        assert_eq!(plan.warm_up, None);
        assert!(!plan.warm_up_enabled);
        assert!(plan.has_warm_up_items);
        assert!(!ids(&plan).contains(&"scales"));
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
}

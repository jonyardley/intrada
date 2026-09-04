//! Ordering the starred set for one-tap "Practise your priorities" (#981).
//!
//! Least ready first means most overdue first, graded against each item's own
//! expected return interval rather than raw days: the same `overdue_key` the
//! Up next card ranks on, so the two surfaces cannot disagree about what has
//! been left behind. The goals-era ordering this replaces (#739, deleted by
//! #769) bucketed on per-item confidence and tempo targets, which
//! `specs/priority-items.md` removed on purpose.

use crate::analytics::LocalClock;
use crate::model::LibraryItemView;
use crate::suggestion::staleness_of;

/// The starred items, least ready first. Never practised ranks stalest of all
/// (`Staleness::overdue_key`), so a brand-new priority leads.
pub(crate) fn order_priorities(
    items: &[LibraryItemView],
    clock: LocalClock,
) -> Vec<&LibraryItemView> {
    let mut starred: Vec<&LibraryItemView> = items.iter().filter(|i| i.priority).collect();
    starred.sort_by_cached_key(|i| {
        (
            std::cmp::Reverse(staleness_of(i, clock).overdue_key()),
            i.title.to_lowercase(),
            i.id.clone(),
        )
    });
    starred
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemKind;
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

    /// A starred item last practised `days_ago`, or never when `None`. Mark and
    /// return count are fixed across the fixtures so the expected interval is
    /// identical and the tests vary only the gap.
    fn starred(id: &str, title: &str, days_ago: Option<i64>) -> LibraryItemView {
        let mut item = LibraryItemView::fixture(id, title, ItemKind::Piece);
        item.priority = true;
        item.practice = days_ago.map(|d| ItemPracticeSummary {
            session_count: 4,
            latest_score: Some(6),
            last_practiced_at: Some(
                (today() - chrono::Duration::days(d))
                    .and_hms_opt(10, 0, 0)
                    .expect("valid time")
                    .and_utc()
                    .to_rfc3339(),
            ),
            ..ItemPracticeSummary::fixture()
        });
        item
    }

    fn unstarred(id: &str, title: &str) -> LibraryItemView {
        LibraryItemView::fixture(id, title, ItemKind::Piece)
    }

    fn ordered_ids(items: &[LibraryItemView]) -> Vec<&str> {
        order_priorities(items, clock())
            .iter()
            .map(|i| i.id.as_str())
            .collect()
    }

    #[test]
    fn nothing_starred_orders_nothing() {
        let library = vec![unstarred("p1", "Prelude"), unstarred("p2", "Nocturne")];
        assert!(ordered_ids(&library).is_empty());
    }

    #[test]
    fn unstarred_items_are_left_out() {
        let library = vec![
            unstarred("p1", "Prelude"),
            starred("p2", "Nocturne", Some(3)),
        ];
        assert_eq!(ordered_ids(&library), ["p2"]);
    }

    #[test]
    fn the_longest_overdue_leads() {
        let library = vec![
            starred("fresh", "Prelude", Some(2)),
            starred("cold", "Nocturne", Some(60)),
        ];
        assert_eq!(ordered_ids(&library), ["cold", "fresh"]);
    }

    #[test]
    fn never_practised_leads_even_the_coldest() {
        let library = vec![
            starred("cold", "Nocturne", Some(400)),
            starred("new", "Prelude", None),
        ];
        assert_eq!(ordered_ids(&library), ["new", "cold"]);
    }

    #[test]
    fn an_equal_gap_falls_back_to_title() {
        // Ids run opposite to titles, and the titles order differently with and
        // without case folding, so neither tie-break clause can pass on the
        // other's behalf.
        let library = vec![
            starred("a", "Aubade", Some(30)),
            starred("z", "aria", Some(30)),
        ];
        assert_eq!(
            ordered_ids(&library),
            ["z", "a"],
            "same gap, so case-insensitive title order decides, not library order or id"
        );
    }
}

//! What needs practice, decided in one place (T42, #3015). Nothing outside
//! this module orders practice material: the plan takes its candidates, their
//! order and their reasons from here.

mod staleness;

use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::analytics::LocalClock;
use crate::domain::key::Key;
use crate::model::LibraryItemView;
use staleness::{assess, days_since, gap_phrase, grade, Staleness};

// ── Tuning ───────────────────────────────────────────────────────────

/// Every number the ranking reads (T42). Informed guesses from the spacing
/// research (`docs/research/space-layer.md`), not fitted values.
mod tuning {
    /// Days something never scored can sit before it is due.
    pub(super) const GAP_UNSCORED: u32 = 10;
    /// (highest score in the band, days it can sit).
    pub(super) const GAP_BY_SCORE: [(u8, u32); 4] = [(3, 5), (6, 9), (8, 16), (u8::MAX, 30)];
    /// (most sessions returned in, extra days), so each return lengthens the gap.
    pub(super) const RETURNS_BONUS: [(usize, u32); 4] = [(1, 0), (4, 2), (9, 5), (usize::MAX, 9)];
    /// Multiples of the gap: going cold past the first, cold past the second.
    pub(super) const GOING_COLD_AFTER: u32 = 1;
    pub(super) const COLD_AFTER: u32 = 2;
}

// ── Practice records ─────────────────────────────────────────────────

/// When one key or variation was last played, scored or not, and in how many
/// sessions. Its latest score is the one its view already carries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Played {
    pub(crate) last: Option<DateTime<Utc>>,
    pub(crate) sessions: usize,
}

impl Played {
    pub(crate) fn add_session(&mut self, at: DateTime<Utc>) {
        self.sessions += 1;
        self.last = self.last.max(Some(at));
    }
}

/// One item's keys, by the key as the item stores it, and its variations, by id.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ItemPlays {
    pub(crate) keys: HashMap<Key, Played>,
    pub(crate) variations: HashMap<String, Played>,
}

/// By item id; built once per render by `build_practice_records`.
pub(crate) type PracticeRecords = HashMap<String, ItemPlays>;

// ── Need ─────────────────────────────────────────────────────────────

/// The item itself, or one of its keys or variations, graded.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RankedTarget<'a> {
    /// `None` is the item itself.
    pub(crate) label: Option<&'a str>,
    pub(crate) score: Option<u8>,
    staleness: Staleness,
    /// Never played on its own, so graded on its item's record (T42).
    inherited: bool,
}

impl RankedTarget<'_> {
    /// Going cold or cold first, then never played, then fresh; within each,
    /// most overdue, then the lower score, then one actually played.
    fn need(&self) -> (u8, Reverse<u32>, Option<u8>, bool) {
        let tier = match (self.staleness.is_due(), self.staleness.days()) {
            (true, _) => 0,
            (false, None) => 1,
            (false, Some(_)) => 2,
        };
        (
            tier,
            Reverse(self.staleness.overdue_key()),
            self.score,
            self.inherited,
        )
    }
}

/// An item's keys and variations, neediest first, or the item alone when it
/// has neither. Never empty. One never played takes the item's own date,
/// returns and, unscored, its score.
pub(crate) fn rank_targets<'a>(
    item: &'a LibraryItemView,
    records: &PracticeRecords,
    clock: LocalClock,
) -> Vec<RankedTarget<'a>> {
    let plays = records.get(&item.id);
    let item_score = item.practice.as_ref().and_then(|p| p.latest_score);
    let target = |label: Option<&'a str>, score: Option<u8>, played: Option<&Played>| match played
        .filter(|p| p.last.is_some())
    {
        Some(played) => RankedTarget {
            label,
            score,
            staleness: grade(
                played.last.map(|at| days_since(at, clock)),
                score,
                played.sessions,
            ),
            inherited: false,
        },
        None => {
            let score = score.or(item_score);
            RankedTarget {
                label,
                score,
                staleness: assess(item.practice.as_ref(), score, clock),
                inherited: label.is_some(),
            }
        }
    };
    let keys = item.keys.iter().map(|k| {
        target(
            Some(&k.label),
            k.latest_score,
            plays.and_then(|p| p.keys.get(&k.key)),
        )
    });
    let variations = item.variations.iter().map(|v| {
        target(
            Some(&v.label),
            v.latest_score,
            plays.and_then(|p| p.variations.get(&v.id)),
        )
    });
    let mut targets: Vec<RankedTarget> = keys.chain(variations).collect();
    if targets.is_empty() {
        targets.push(target(None, None, None));
    }
    targets.sort_by_key(RankedTarget::need);
    targets
}

/// An item with the target that decided its place.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ranked<'a> {
    pub(crate) item: &'a LibraryItemView,
    neediest: RankedTarget<'a>,
}

/// The whole library, most in need first: starred, then most overdue, then
/// lower score, then title as the Library files it, then id.
pub(crate) fn rank_items<'a>(
    items: &'a [LibraryItemView],
    records: &PracticeRecords,
    clock: LocalClock,
) -> Vec<Ranked<'a>> {
    let mut ranked: Vec<Ranked> = items
        .iter()
        .filter_map(|item| {
            let neediest = rank_targets(item, records, clock).into_iter().next()?;
            Some(Ranked { item, neediest })
        })
        .collect();
    ranked.sort_by_cached_key(|r| {
        (
            Reverse(r.item.priority),
            r.neediest.need(),
            crate::view::library::title_sort_key(&r.item.title),
            r.item.id.clone(),
        )
    });
    ranked
}

// ── Reason ───────────────────────────────────────────────────────────

impl Ranked<'_> {
    /// Played, and going cold or cold on its neediest key or variation.
    pub(crate) fn is_due(&self) -> bool {
        self.neediest.staleness.is_due()
    }

    /// The rule that placed the item, mid-sentence: "a priority", "G major,
    /// scored 3, 2 weeks ago", "going cold after 3 weeks".
    pub(crate) fn reason(&self) -> String {
        if self.item.priority {
            return "a priority".to_string();
        }
        let Some(label) = self.neediest.label.filter(|_| !self.neediest.inherited) else {
            return self.neediest.staleness.clause();
        };
        let when = match self.neediest.staleness.days() {
            None => return format!("{label}, not practised yet"),
            Some(0) => "today".to_string(),
            Some(1) => "yesterday".to_string(),
            Some(d) => format!("{} ago", gap_phrase(d)),
        };
        match self.neediest.score {
            Some(score) => format!("{label}, scored {score}, {when}"),
            None => format!("{label}, practised {when}"),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemKind;
    use crate::model::{ItemKeyView, ItemPracticeSummary, VariationView};
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

    fn days_ago(days: i64) -> DateTime<Utc> {
        (today() - chrono::Duration::days(days))
            .and_hms_opt(10, 0, 0)
            .expect("valid time")
            .and_utc()
    }

    fn key(name: &str) -> Key {
        Key::parse(name).expect("a key")
    }

    fn item(id: &str, title: &str) -> LibraryItemView {
        LibraryItemView::fixture(id, title, ItemKind::Piece)
    }

    /// The item's own summary: played `days` ago in `sessions` sessions.
    fn practised(
        mut item: LibraryItemView,
        days: i64,
        score: Option<u8>,
        sessions: usize,
    ) -> LibraryItemView {
        item.practice = Some(ItemPracticeSummary {
            session_count: sessions,
            latest_score: score,
            last_practiced_at: Some(days_ago(days).to_rfc3339()),
            ..ItemPracticeSummary::fixture()
        });
        item
    }

    fn with_key(mut item: LibraryItemView, name: &str, score: Option<u8>) -> LibraryItemView {
        let key = key(name);
        item.keys.push(ItemKeyView {
            key,
            label: key.label(),
            latest_score: score,
            caption: String::new(),
        });
        item
    }

    fn played(records: &mut PracticeRecords, item_id: &str, name: &str, days: i64) {
        records
            .entry(item_id.to_string())
            .or_default()
            .keys
            .entry(key(name))
            .or_default()
            .add_session(days_ago(days));
    }

    fn order(items: &[LibraryItemView], records: &PracticeRecords) -> Vec<String> {
        rank_items(items, records, clock())
            .iter()
            .map(|r| r.item.id.clone())
            .collect()
    }

    /// Scales played in C major yesterday, scored 9, and in G major a
    /// fortnight ago, scored 3; a piece scored 8 a week ago.
    fn fortnight() -> (Vec<LibraryItemView>, PracticeRecords) {
        let scales = with_key(
            with_key(
                practised(item("scales", "Scales"), 1, Some(9), 6),
                "C major",
                Some(9),
            ),
            "G major",
            Some(3),
        );
        let piece = practised(item("piece", "Arabesque"), 7, Some(8), 3);
        let mut records = PracticeRecords::new();
        played(&mut records, "scales", "C major", 1);
        played(&mut records, "scales", "G major", 14);
        (vec![piece, scales], records)
    }

    #[test]
    fn a_low_key_a_fortnight_ago_beats_a_good_piece_a_week_ago() {
        let (items, records) = fortnight();
        let ranked = rank_items(&items, &records, clock());

        assert_eq!(ranked[0].item.id, "scales");
        assert_eq!(ranked[0].reason(), "G major, scored 3, 2 weeks ago");
        assert_eq!(ranked[1].reason(), "practised 7 days ago");
    }

    #[test]
    fn the_targets_of_an_item_come_neediest_first() {
        let (items, records) = fortnight();
        let labels: Vec<_> = rank_targets(&items[1], &records, clock())
            .iter()
            .map(|t| t.label)
            .collect();

        assert_eq!(labels, [Some("G major"), Some("C major")]);
    }

    #[test]
    fn an_item_with_no_keys_or_variations_is_its_own_target() {
        let piece = practised(item("piece", "Arabesque"), 7, Some(8), 3);
        let targets = rank_targets(&piece, &PracticeRecords::new(), clock());

        assert_eq!(targets.len(), 1);
        assert_eq!((targets[0].label, targets[0].score), (None, Some(8)));
    }

    #[test]
    fn a_key_scored_alone_moves_its_item() {
        // Both summaries read the same, so only the key can separate them.
        let exercise = with_key(
            practised(item("exercise", "Thirds"), 7, Some(9), 3),
            "D minor",
            Some(2),
        );
        let piece = practised(item("piece", "Arabesque"), 7, Some(9), 3);
        let mut records = PracticeRecords::new();
        played(&mut records, "exercise", "D minor", 7);

        assert_eq!(order(&[piece, exercise], &records), ["exercise", "piece"]);
    }

    #[test]
    fn an_unplayed_key_takes_its_items_record_and_does_not_pin_it_top() {
        let exercise = with_key(
            with_key(
                practised(item("exercise", "Thirds"), 1, Some(9), 6),
                "C major",
                Some(9),
            ),
            "F# minor",
            None,
        );
        let cold = practised(item("cold", "Arabesque"), 300, Some(2), 1);
        let mut records = PracticeRecords::new();
        played(&mut records, "exercise", "C major", 1);
        let items = [cold, exercise];
        let ranked = rank_items(&items, &records, clock());

        assert_eq!(ranked[0].item.id, "cold");
        assert_eq!(ranked[1].item.id, "exercise");
        assert!(
            !ranked[1].reason().contains("F♯ minor"),
            "{}",
            ranked[1].reason()
        );
    }

    #[test]
    fn an_unplayed_key_that_wins_speaks_for_the_item() {
        let exercise = with_key(
            practised(item("exercise", "Thirds"), 21, Some(9), 1),
            "F# minor",
            None,
        );
        let items = [exercise];

        assert_eq!(
            rank_items(&items, &PracticeRecords::new(), clock())[0].reason(),
            "practised 3 weeks ago"
        );
    }

    #[test]
    fn a_never_played_item_with_keys_is_not_practised_yet() {
        let exercise = with_key(item("exercise", "Thirds"), "F# minor", None);
        let items = [exercise];

        assert_eq!(
            rank_items(&items, &PracticeRecords::new(), clock())[0].reason(),
            "not practised yet"
        );
    }

    #[test]
    fn a_variation_counts_like_a_key() {
        let mut piece = practised(item("piece", "Arabesque"), 1, Some(9), 6);
        piece.variations.push(VariationView {
            id: "v1".to_string(),
            label: "Left hand".to_string(),
            latest_score: Some(4),
            score_history: Vec::new(),
            caption: String::new(),
        });
        let other = practised(item("other", "Aria"), 5, Some(9), 6);
        let mut records = PracticeRecords::new();
        records
            .entry("piece".to_string())
            .or_default()
            .variations
            .entry("v1".to_string())
            .or_default()
            .add_session(days_ago(20));
        let items = [other, piece];
        let ranked = rank_items(&items, &records, clock());

        assert_eq!(ranked[0].item.id, "piece");
        assert_eq!(ranked[0].reason(), "Left hand, scored 4, 3 weeks ago");
    }

    #[test]
    fn an_unscored_key_says_when_it_was_played() {
        let exercise = with_key(item("exercise", "Thirds"), "D minor", None);
        let mut records = PracticeRecords::new();
        played(&mut records, "exercise", "D minor", 3);
        let items = [exercise];

        assert_eq!(
            rank_items(&items, &records, clock())[0].reason(),
            "D minor, practised 3 days ago"
        );
    }

    #[test]
    fn starred_beats_more_overdue() {
        let mut starred = practised(item("starred", "Zephyr"), 1, Some(9), 6);
        starred.priority = true;
        let cold = practised(item("cold", "Arabesque"), 300, Some(2), 1);
        let items = [cold, starred];

        let ranked = rank_items(&items, &PracticeRecords::new(), clock());
        assert_eq!(ranked[0].item.id, "starred");
        assert_eq!(ranked[0].reason(), "a priority");
    }

    #[test]
    fn at_equal_overdue_the_lower_score_comes_first() {
        // 10 days on a 5 day gap and 18 days on a 9 day gap are both 200.
        let rough = practised(item("rough", "Zephyr"), 10, Some(2), 1);
        let middling = practised(item("middling", "Arabesque"), 18, Some(5), 1);

        assert_eq!(
            order(&[middling, rough], &PracticeRecords::new()),
            ["rough", "middling"]
        );
    }

    #[test]
    fn never_practised_comes_after_cold_and_before_fresh() {
        let cold = practised(item("cold", "Arabesque"), 60, None, 1);
        let going = practised(item("going", "Bourrée"), 15, None, 1);
        let fresh = practised(item("fresh", "Csárdás"), 1, None, 1);
        let new = item("new", "Zephyr");

        assert_eq!(
            order(&[fresh, new, going, cold], &PracticeRecords::new()),
            ["cold", "going", "new", "fresh"]
        );
    }

    #[test]
    fn a_starred_item_never_practised_follows_a_starred_cold_one() {
        let mut cold = practised(item("cold", "Arabesque"), 60, None, 1);
        cold.priority = true;
        let mut new = item("new", "Zephyr");
        new.priority = true;

        assert_eq!(
            order(&[new, cold], &PracticeRecords::new()),
            ["cold", "new"]
        );
    }

    #[test]
    fn equal_need_falls_back_to_title_as_the_library_files_it_then_id() {
        let items = [
            practised(item("b", "Etude"), 30, None, 1),
            practised(item("c", "Aria"), 30, None, 1),
            practised(item("a", "Étude"), 30, None, 1),
        ];

        assert_eq!(order(&items, &PracticeRecords::new()), ["c", "a", "b"]);
    }
}

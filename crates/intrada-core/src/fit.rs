//! The builder's plan against today's length (#3009): minutes used, offers,
//! the scaling trim and stretch share, and what fill would add.

use std::cmp::Reverse;

use crate::analytics::LocalClock;
use crate::domain::session::{BuildingSession, SetlistEntry};
use crate::model::{FitOffer, FitState, FitView, LibraryItemView};
use crate::ranking::{self, PracticeRecords};
use crate::validation::{MAX_PLANNED_DURATION_SECS, MIN_PLANNED_DURATION_SECS};

const MIN_ROW_MINS: u32 = MIN_PLANNED_DURATION_SECS / 60;
const MAX_ROW_MINS: u32 = MAX_PLANNED_DURATION_SECS / 60;

/// Each row's planned minutes, to the nearest whole minute; `None` is untimed.
pub(crate) fn row_minutes(entries: &[SetlistEntry]) -> Vec<Option<u32>> {
    entries
        .iter()
        .map(|e| e.plan.duration_secs.map(|secs| (secs + 30) / 60))
        .collect()
}

fn used(minutes: &[Option<u32>]) -> u32 {
    minutes.iter().flatten().sum()
}

/// Rounding error goes to the longest rows first, within each row's limits.
pub(crate) fn scale_to(minutes: &[Option<u32>], length_mins: u32) -> Vec<Option<u32>> {
    let total = u64::from(used(minutes));
    if total == 0 {
        return minutes.to_vec();
    }
    let mut scaled: Vec<Option<u32>> = minutes
        .iter()
        .map(|m| {
            m.map(|m| {
                let share = u64::from(m) * u64::from(length_mins) / total;
                u32::try_from(share)
                    .unwrap_or(MAX_ROW_MINS)
                    .clamp(MIN_ROW_MINS, MAX_ROW_MINS)
            })
        })
        .collect();

    let mut longest_first: Vec<usize> = (0..minutes.len())
        .filter(|&i| minutes[i].is_some())
        .collect();
    longest_first.sort_by_key(|&i| Reverse(minutes[i]));

    let mut gap = i64::from(length_mins) - i64::from(used(&scaled));
    while gap != 0 {
        let mut moved = false;
        for &i in &longest_first {
            let Some(m) = scaled[i].as_mut() else {
                continue;
            };
            if gap > 0 && *m < MAX_ROW_MINS {
                *m += 1;
                gap -= 1;
                moved = true;
            } else if gap < 0 && *m > MIN_ROW_MINS {
                *m -= 1;
                gap += 1;
                moved = true;
            }
            if gap == 0 {
                break;
            }
        }
        if !moved {
            break;
        }
    }
    scaled
}

/// What does not fit `room_mins` is skipped, as `plan::fill` does.
pub(crate) fn fill_suggestions<'a>(
    ranked: impl IntoIterator<Item = &'a LibraryItemView>,
    planned_ids: &[&str],
    room_mins: u32,
) -> Vec<(&'a LibraryItemView, u32)> {
    let mut room = room_mins;
    ranked
        .into_iter()
        .filter(|item| !planned_ids.contains(&item.id.as_str()))
        .filter_map(|item| {
            let minutes = item.usual_minutes.clamp(MIN_ROW_MINS, MAX_ROW_MINS);
            (minutes <= room).then(|| {
                room -= minutes;
                (item, minutes)
            })
        })
        .collect()
}

/// What Fill with suggestions would add to this build now; empty without a
/// length or without room.
pub(crate) fn suggestions_for<'a>(
    building: &BuildingSession,
    library: &'a [LibraryItemView],
    records: &PracticeRecords,
    clock: LocalClock,
) -> Vec<(&'a LibraryItemView, u32)> {
    let Some(length) = building.length_mins else {
        return Vec::new();
    };
    let room = u32::from(length).saturating_sub(used(&row_minutes(&building.entries)));
    if room == 0 {
        return Vec::new();
    }
    let planned: Vec<&str> = building
        .entries
        .iter()
        .map(|e| e.plan.item_id.as_str())
        .collect();
    let ranked = ranking::rank_items(library, records, clock);
    fill_suggestions(ranked.iter().map(|r| r.item), &planned, room)
}

/// The trailing run of one group, or the last entry alone.
pub(crate) fn last_unit(entries: &[SetlistEntry]) -> &[SetlistEntry] {
    let Some(last) = entries.last() else {
        return entries;
    };
    let start = match last.plan.group_id.as_deref() {
        Some(group) => entries
            .iter()
            .rposition(|e| e.plan.group_id.as_deref() != Some(group))
            .map_or(0, |i| i + 1),
        None => entries.len() - 1,
    };
    &entries[start..]
}

/// `None` without a length.
pub(crate) fn fit_view(
    entries: &[SetlistEntry],
    length_mins: Option<u16>,
    can_fill: bool,
) -> Option<FitView> {
    let length = length_mins?;
    let minutes = row_minutes(entries);
    let used_mins = used(&minutes);
    let untimed = minutes.iter().filter(|m| m.is_none()).count();
    let timed = || minutes.iter().flatten();
    let can_stretch = timed().any(|&m| m < MAX_ROW_MINS);
    let can_trim = timed().any(|&m| m > MIN_ROW_MINS);
    let can_drop = used(&row_minutes(last_unit(entries))) > 0;

    let state = match used_mins.cmp(&u32::from(length)) {
        std::cmp::Ordering::Less => FitState::Under,
        std::cmp::Ordering::Equal => FitState::Exact,
        std::cmp::Ordering::Greater => FitState::Over,
    };
    let offers = match state {
        FitState::Under => [
            can_fill.then_some(FitOffer::FillWithSuggestions),
            can_stretch.then_some(FitOffer::StretchToFit),
        ]
        .into_iter()
        .flatten()
        .collect(),
        FitState::Exact => Vec::new(),
        FitState::Over => [
            can_trim.then_some(FitOffer::TrimToFit),
            can_drop.then_some(FitOffer::DropLastItem),
        ]
        .into_iter()
        .flatten()
        .collect(),
    };
    let mut summary = format!("{used_mins} of {length} min");
    if untimed > 0 {
        summary.push_str(&format!(", plus {untimed} untimed"));
    }

    let state_line = match state {
        FitState::Over => format!("{} min over", used_mins - u32::from(length)),
        FitState::Under => format!("{} min left", u32::from(length) - used_mins),
        FitState::Exact => "Fits exactly".to_string(),
    };

    Some(FitView {
        used_mins,
        length_mins: length,
        state,
        summary,
        offers,
        state_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemKind;

    fn sum(minutes: &[Option<u32>]) -> u32 {
        used(minutes)
    }

    #[test]
    fn trim_brings_a_full_plan_with_one_more_piece_back_to_the_length() {
        let trimmed = scale_to(&[Some(30), Some(30), Some(12)], 60);
        assert_eq!(trimmed, [Some(25), Some(25), Some(10)]);
    }

    #[test]
    fn rounding_error_goes_to_the_longest_rows_and_still_sums_to_the_length() {
        let stretched = scale_to(&[Some(10), Some(5), Some(3)], 20);
        assert_eq!(stretched, [Some(12), Some(5), Some(3)]);

        let even = scale_to(&[Some(7), Some(7), Some(7)], 10);
        assert_eq!(even, [Some(4), Some(3), Some(3)]);
    }

    #[test]
    fn rounding_error_goes_to_the_longest_row_wherever_it_sits() {
        let stretched = scale_to(&[Some(3), Some(5), Some(10)], 20);
        assert_eq!(stretched, [Some(3), Some(5), Some(12)]);
    }

    #[test]
    fn trim_stops_at_one_minute_a_row() {
        let trimmed = scale_to(&[Some(1), Some(1), Some(1), Some(30)], 3);
        assert_eq!(trimmed, [Some(1), Some(1), Some(1), Some(1)]);
    }

    #[test]
    fn stretch_stops_at_sixty_minutes_a_row() {
        assert_eq!(scale_to(&[Some(50)], 90), [Some(60)]);
        assert_eq!(scale_to(&[Some(50), Some(10)], 120), [Some(60), Some(60)]);
        assert_eq!(scale_to(&[Some(50), Some(10)], 200), [Some(60), Some(60)]);
    }

    #[test]
    fn untimed_rows_are_left_alone() {
        let trimmed = scale_to(&[Some(30), None, Some(30)], 30);
        assert_eq!(trimmed, [Some(15), None, Some(15)]);
        assert_eq!(scale_to(&[None, None], 30), [None, None]);
    }

    #[test]
    fn the_total_lands_on_the_length_wherever_the_limits_allow() {
        let rows = [Some(13), Some(1), Some(44), Some(7), Some(60)];
        for length in 1..=400 {
            let expected = length.clamp(5, 300);
            assert_eq!(sum(&scale_to(&rows, length)), expected, "{length} minutes");
        }
    }

    fn item(id: &str, minutes: u32) -> LibraryItemView {
        let mut item = LibraryItemView::fixture(id, id, ItemKind::Exercise);
        item.usual_minutes = minutes;
        item
    }

    #[test]
    fn fill_skips_planned_items_and_what_does_not_fit() {
        let ranked = [item("a", 10), item("b", 20), item("c", 10), item("d", 5)];
        let picked: Vec<_> = fill_suggestions(&ranked, &["a"], 18)
            .into_iter()
            .map(|(i, m)| (i.id.as_str(), m))
            .collect();
        assert_eq!(picked, [("c", 10), ("d", 5)]);
    }

    #[test]
    fn fill_adds_nothing_when_nothing_fits() {
        let ranked = [item("a", 10), item("b", 20)];
        assert!(fill_suggestions(&ranked, &[], 9).is_empty());
        assert!(fill_suggestions(&ranked, &["a", "b"], 60).is_empty());
    }

    #[test]
    fn fill_takes_each_item_at_its_usual_minutes_within_a_row_s_limits() {
        let ranked = [item("long", 90), item("blank", 0)];
        let picked: Vec<_> = fill_suggestions(&ranked, &[], 61)
            .into_iter()
            .map(|(i, m)| (i.id.as_str(), m))
            .collect();
        assert_eq!(picked, [("long", 60), ("blank", 1)]);
    }

    fn entry(minutes: Option<u32>) -> SetlistEntry {
        let mut entry = SetlistEntry::fixture();
        entry.plan.duration_secs = minutes.map(|m| m * 60);
        entry
    }

    #[test]
    fn a_full_plan_with_one_more_piece_is_over_and_offers_trim_and_drop() {
        let fit = fit_view(&[entry(Some(60)), entry(Some(12))], Some(60), true).expect("fit");
        assert_eq!(fit.summary, "72 of 60 min");
        assert_eq!(fit.used_mins, 72);
        assert_eq!(fit.length_mins, 60);
        assert_eq!(fit.state, FitState::Over);
        assert_eq!(fit.offers, [FitOffer::TrimToFit, FitOffer::DropLastItem]);
    }

    fn block(
        group: &str,
        exercise_mins: Option<u32>,
        piece_mins: Option<u32>,
    ) -> [SetlistEntry; 2] {
        let mut related = entry(exercise_mins);
        related.plan.item_type = ItemKind::Exercise;
        related.plan.group_id = Some(group.to_string());
        let mut piece = entry(piece_mins);
        piece.plan.group_id = Some(group.to_string());
        [related, piece]
    }

    #[test]
    fn drop_is_offered_when_the_last_unit_is_a_block() {
        let [related, piece] = block("g1", Some(30), Some(40));
        let fit = fit_view(&[related, piece], Some(60), true).expect("fit");
        assert_eq!(fit.state, FitState::Over);
        assert_eq!(fit.offers, [FitOffer::TrimToFit, FitOffer::DropLastItem]);
    }

    #[test]
    fn drop_is_not_offered_when_the_last_unit_frees_no_timed_minutes() {
        let fit = fit_view(&[entry(Some(70)), entry(None)], Some(60), true).expect("fit");
        assert_eq!(fit.offers, [FitOffer::TrimToFit]);

        let [related, piece] = block("g1", None, None);
        let fit = fit_view(&[entry(Some(70)), related, piece], Some(60), true).expect("fit");
        assert_eq!(fit.offers, [FitOffer::TrimToFit]);
    }

    #[test]
    fn the_last_unit_is_the_trailing_run_of_one_group() {
        let [related, piece] = block("g1", Some(5), Some(10));
        let entries = [entry(Some(3)), related, piece];
        assert_eq!(last_unit(&entries).len(), 2);
        assert_eq!(last_unit(&entries[..1]).len(), 1);
        assert!(last_unit(&[]).is_empty());
    }

    #[test]
    fn a_short_plan_offers_fill_and_stretch() {
        let fit = fit_view(&[entry(Some(15))], Some(60), true).expect("fit");
        assert_eq!(fit.summary, "15 of 60 min");
        assert_eq!(fit.state, FitState::Under);
        assert_eq!(
            fit.offers,
            [FitOffer::FillWithSuggestions, FitOffer::StretchToFit]
        );
    }

    #[test]
    fn fill_is_not_offered_when_nothing_fits_and_stretch_needs_a_timed_row() {
        let fit = fit_view(&[entry(Some(15))], Some(60), false).expect("fit");
        assert_eq!(fit.offers, [FitOffer::StretchToFit]);

        let untimed = fit_view(&[entry(None), entry(None)], Some(60), false).expect("fit");
        assert_eq!(untimed.summary, "0 of 60 min, plus 2 untimed");
        assert!(untimed.offers.is_empty());
    }

    #[test]
    fn stretch_is_not_offered_when_every_timed_row_is_at_sixty() {
        let fit = fit_view(&[entry(Some(60)), entry(None)], Some(120), false).expect("fit");
        assert_eq!(fit.state, FitState::Under);
        assert!(fit.offers.is_empty());

        let fit = fit_view(&[entry(Some(45))], Some(120), false).expect("fit");
        assert_eq!(fit.offers, [FitOffer::StretchToFit]);
    }

    #[test]
    fn trim_is_not_offered_when_every_timed_row_is_at_one() {
        let rows = [entry(Some(1)), entry(Some(1)), entry(Some(1))];
        let fit = fit_view(&rows, Some(2), true).expect("fit");
        assert_eq!(fit.state, FitState::Over);
        assert_eq!(fit.offers, [FitOffer::DropLastItem]);
    }

    #[test]
    fn the_state_line_says_how_far_over_or_under_in_words() {
        let over = fit_view(&[entry(Some(72))], Some(60), true).expect("fit");
        assert_eq!(over.state_line, "12 min over");
        let under = fit_view(&[entry(Some(15))], Some(60), true).expect("fit");
        assert_eq!(under.state_line, "45 min left");
        let exact = fit_view(&[entry(Some(60))], Some(60), true).expect("fit");
        assert_eq!(exact.state_line, "Fits exactly");
    }

    #[test]
    fn an_exact_plan_offers_nothing() {
        let fit = fit_view(&[entry(Some(40)), entry(Some(20))], Some(60), true).expect("fit");
        assert_eq!(fit.state, FitState::Exact);
        assert!(fit.offers.is_empty());
    }

    #[test]
    fn no_length_means_no_fit() {
        assert_eq!(fit_view(&[entry(Some(15))], None, true), None);
    }

    #[test]
    fn untimed_rows_are_counted_in_the_summary() {
        let fit = fit_view(&[entry(Some(30)), entry(None)], Some(20), true).expect("fit");
        assert_eq!(fit.summary, "30 of 20 min, plus 1 untimed");
    }
}

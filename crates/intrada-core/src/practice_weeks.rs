//! The Practice tab's week strip (#2046). Days are counted on the one
//! `LocalClock` offset the streak and the Progress screen use, so a session
//! sits in the same week everywhere it is shown.

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

use crate::analytics::LocalClock;
use crate::domain::session::PracticeSession;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeWeekView {
    /// Seven, Monday first.
    pub days: Vec<PracticeDayView>,
    pub practised_days: usize,
    /// Index into `days` of the day the week shows when it comes into view.
    pub opening_day: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeDayView {
    /// `2026-05-27`: the key the shell remembers a tapped day by.
    pub date: String,
    pub weekday_initial: String,
    pub day_number: u32,
    /// `Wednesday 27 May`.
    pub full_date: String,
    /// `Today`, `Yesterday`, otherwise the full date.
    pub heading: String,
    pub is_today: bool,
    pub is_future: bool,
    /// Ids into `ViewModel::sessions`, newest first.
    pub session_ids: Vec<String>,
}

/// From the week of the earliest session to this week, oldest first. This
/// week alone when nothing came before it, so never empty.
pub fn compute_practice_weeks(
    sessions: &[PracticeSession],
    clock: LocalClock,
) -> Vec<PracticeWeekView> {
    let mut by_day: HashMap<NaiveDate, Vec<&PracticeSession>> = HashMap::new();
    for session in sessions {
        by_day
            .entry(clock.day_of(session.started_at))
            .or_default()
            .push(session);
    }
    for day_sessions in by_day.values_mut() {
        day_sessions.sort_by(|a, b| {
            b.started_at
                .cmp(&a.started_at)
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    let this_monday = monday_of(clock.today);
    let first_monday = by_day
        .keys()
        .min()
        .map(|d| monday_of(*d))
        .filter(|m| *m < this_monday)
        .unwrap_or(this_monday);

    first_monday
        .iter_weeks()
        .take_while(|monday| *monday <= this_monday)
        .map(|monday| week_view(monday, &by_day, clock.today))
        .collect()
}

fn monday_of(day: NaiveDate) -> NaiveDate {
    day.week(Weekday::Mon).first_day()
}

fn week_view(
    monday: NaiveDate,
    by_day: &HashMap<NaiveDate, Vec<&PracticeSession>>,
    today: NaiveDate,
) -> PracticeWeekView {
    let days: Vec<PracticeDayView> = monday
        .iter_days()
        .take(7)
        .map(|day| day_view(day, by_day.get(&day), today))
        .collect();
    PracticeWeekView {
        practised_days: days.iter().filter(|d| !d.session_ids.is_empty()).count(),
        opening_day: opening_day(&days),
        days,
    }
}

/// Today if practised, else the latest practice day before it, else today. A
/// past week has no today, so it opens on its latest practice day, else Sunday.
fn opening_day(days: &[PracticeDayView]) -> usize {
    let last = days
        .iter()
        .position(|d| d.is_today)
        .unwrap_or(days.len() - 1);
    (0..=last)
        .rev()
        .find(|&i| !days[i].session_ids.is_empty())
        .unwrap_or(last)
}

fn day_view(
    day: NaiveDate,
    sessions: Option<&Vec<&PracticeSession>>,
    today: NaiveDate,
) -> PracticeDayView {
    let full_date = day.format("%A %-d %B").to_string();
    let heading = match (today - day).num_days() {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        _ => full_date.clone(),
    };
    PracticeDayView {
        date: day.format("%Y-%m-%d").to_string(),
        weekday_initial: day.format("%a").to_string().chars().take(1).collect(),
        day_number: day.day(),
        full_date,
        heading,
        is_today: day == today,
        is_future: day > today,
        session_ids: sessions
            .map(|s| s.iter().map(|s| s.id.clone()).collect())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::compute_weekly_summary;
    use crate::domain::session::CompletionStatus;
    use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.from_utc_datetime(&day(y, m, d).and_hms_opt(h, min, 0).expect("valid time"))
    }

    fn clock(today: NaiveDate, utc_offset_minutes: i32) -> LocalClock {
        LocalClock {
            today,
            utc_offset_minutes,
        }
    }

    fn session(id: &str, started: DateTime<Utc>) -> PracticeSession {
        PracticeSession {
            id: id.to_string(),
            started_at: started,
            completed_at: started + chrono::Duration::minutes(20),
            total_duration_secs: 1200,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            entries: Vec::new(),
            session_score: None,
        }
    }

    /// A session at 10:00 UTC on each date, ids `s-<date>`.
    fn sessions_on(dates: &[NaiveDate]) -> Vec<PracticeSession> {
        dates
            .iter()
            .map(|d| session(&format!("s-{d}"), at(d.year(), d.month(), d.day(), 10, 0)))
            .collect()
    }

    fn week_containing(weeks: &[PracticeWeekView], date: &str) -> PracticeWeekView {
        weeks
            .iter()
            .find(|w| w.days.iter().any(|d| d.date == date))
            .cloned()
            .unwrap_or_else(|| panic!("no week holds {date}"))
    }

    fn dates(week: &PracticeWeekView) -> Vec<String> {
        week.days.iter().map(|d| d.date.clone()).collect()
    }

    #[test]
    fn a_week_runs_monday_to_sunday_whatever_the_weekday() {
        let expected: Vec<String> = (25..=31).map(|d| format!("2026-05-{d}")).collect();
        for today in [day(2026, 5, 25), day(2026, 5, 27), day(2026, 5, 31)] {
            let weeks = compute_practice_weeks(&[], clock(today, 0));
            assert_eq!(weeks.len(), 1, "today {today}");
            assert_eq!(dates(&weeks[0]), expected, "today {today}");
            let initials: String = weeks[0]
                .days
                .iter()
                .map(|d| d.weekday_initial.as_str())
                .collect();
            assert_eq!(initials, "MTWTFSS");
            let numbers: Vec<u32> = weeks[0].days.iter().map(|d| d.day_number).collect();
            assert_eq!(numbers, (25..=31).collect::<Vec<u32>>());
        }
    }

    #[test]
    fn a_day_lists_its_sessions_newest_first() {
        let sessions = vec![
            session("thu-morning", at(2026, 5, 28, 9, 0)),
            session("sat", at(2026, 5, 30, 10, 0)),
            session("thu-evening", at(2026, 5, 28, 18, 0)),
        ];
        let week = &compute_practice_weeks(&sessions, clock(day(2026, 5, 31), 0))[0];
        let ids: Vec<Vec<&str>> = week
            .days
            .iter()
            .map(|d| d.session_ids.iter().map(String::as_str).collect())
            .collect();
        assert_eq!(
            ids,
            vec![
                vec![],
                vec![],
                vec![],
                vec!["thu-evening", "thu-morning"],
                vec![],
                vec!["sat"],
                vec![],
            ]
        );
        assert_eq!(week.practised_days, 2);
    }

    #[test]
    fn a_session_lands_on_the_day_it_fell_on_at_todays_offset() {
        // (offset, UTC start, local date). BST is +60, New York in summer -240.
        let cases = [
            (0, at(2026, 5, 27, 23, 30), "2026-05-27"),
            (60, at(2026, 5, 27, 23, 30), "2026-05-28"),
            (-240, at(2026, 5, 28, 3, 0), "2026-05-27"),
        ];
        for (offset, started, expected) in cases {
            let weeks =
                compute_practice_weeks(&[session("s", started)], clock(day(2026, 5, 31), offset));
            let practised: Vec<&str> = weeks[0]
                .days
                .iter()
                .filter(|d| !d.session_ids.is_empty())
                .map(|d| d.date.as_str())
                .collect();
            assert_eq!(practised, vec![expected], "offset {offset}");
        }
    }

    #[test]
    fn sunday_night_and_monday_morning_fall_in_different_weeks() {
        let sessions = vec![
            session("sun", at(2026, 5, 24, 23, 55)),
            session("mon", at(2026, 5, 25, 0, 5)),
        ];
        let weeks = compute_practice_weeks(&sessions, clock(day(2026, 5, 27), 0));
        assert_eq!(weeks.len(), 2);
        assert_eq!(weeks[0].days[6].session_ids, vec!["sun"]);
        assert_eq!(weeks[1].days[0].session_ids, vec!["mon"]);

        // 23:30 UTC on the Sunday is 00:30 BST on the Monday: this week only.
        let bst = compute_practice_weeks(
            &[session("mon-bst", at(2026, 5, 24, 23, 30))],
            clock(day(2026, 5, 27), 60),
        );
        assert_eq!(bst.len(), 1);
        assert_eq!(bst[0].days[0].session_ids, vec!["mon-bst"]);
    }

    #[test]
    fn weeks_run_from_the_earliest_session_to_this_week() {
        let sessions = sessions_on(&[day(2026, 5, 28), day(2026, 5, 30)]);
        let weeks = compute_practice_weeks(&sessions, clock(day(2026, 6, 10), 0));
        let mondays: Vec<&str> = weeks.iter().map(|w| w.days[0].date.as_str()).collect();
        assert_eq!(mondays, vec!["2026-05-25", "2026-06-01", "2026-06-08"]);
        assert!(weeks[2]
            .days
            .iter()
            .any(|d| d.is_today && d.date == "2026-06-10"));
    }

    #[test]
    fn with_no_sessions_there_is_this_week_alone() {
        let weeks = compute_practice_weeks(&[], clock(day(2026, 5, 31), 0));
        assert_eq!(weeks.len(), 1);
        assert_eq!(weeks[0].days[0].date, "2026-05-25");
        assert_eq!(weeks[0].practised_days, 0);
    }

    #[test]
    fn sessions_after_this_week_are_not_shown() {
        let sessions = sessions_on(&[day(2026, 6, 2)]);
        let weeks = compute_practice_weeks(&sessions, clock(day(2026, 5, 31), 0));
        assert_eq!(weeks.len(), 1);
        assert_eq!(weeks[0].practised_days, 0);
    }

    #[test]
    fn each_week_opens_on_the_day_the_rule_picks() {
        // (today, practice dates, a date in the week looked at, day it opens on)
        let cases: [(NaiveDate, Vec<NaiveDate>, &str, &str); 6] = [
            // This week, today practised: today.
            (
                day(2026, 5, 30),
                vec![day(2026, 5, 28), day(2026, 5, 30)],
                "2026-05-30",
                "2026-05-30",
            ),
            // This week, today not practised: the most recent earlier practice day.
            (
                day(2026, 5, 31),
                vec![day(2026, 5, 28), day(2026, 5, 30)],
                "2026-05-31",
                "2026-05-30",
            ),
            // This week, no practice: today.
            (day(2026, 5, 31), vec![], "2026-05-31", "2026-05-31"),
            // This week, practice only on a later day: still today.
            (
                day(2026, 5, 27),
                vec![day(2026, 5, 29)],
                "2026-05-27",
                "2026-05-27",
            ),
            // A past week: its most recent practice day.
            (
                day(2026, 6, 10),
                vec![day(2026, 5, 28), day(2026, 5, 30)],
                "2026-05-28",
                "2026-05-30",
            ),
            // A past week with no practice: its Sunday.
            (
                day(2026, 6, 10),
                vec![day(2026, 5, 20)],
                "2026-05-28",
                "2026-05-31",
            ),
        ];
        for (today, practice, looked_at, expected) in cases {
            let weeks = compute_practice_weeks(&sessions_on(&practice), clock(today, 0));
            let week = week_containing(&weeks, looked_at);
            assert_eq!(
                week.days[week.opening_day].date, expected,
                "today {today}, week of {looked_at}"
            );
        }
    }

    #[test]
    fn the_practised_count_belongs_to_its_own_week() {
        let sessions = sessions_on(&[day(2026, 5, 28), day(2026, 5, 30)]);
        let weeks = compute_practice_weeks(&sessions, clock(day(2026, 6, 10), 0));
        assert_eq!(week_containing(&weeks, "2026-05-28").practised_days, 2);
        assert_eq!(week_containing(&weeks, "2026-06-10").practised_days, 0);
    }

    #[test]
    fn days_are_labelled_against_today() {
        let week = &compute_practice_weeks(&[], clock(day(2026, 5, 27), 0))[0];
        // (index, heading, full date, is today, is future)
        let cases = [
            (0, "Monday 25 May", "Monday 25 May", false, false),
            (1, "Yesterday", "Tuesday 26 May", false, false),
            (2, "Today", "Wednesday 27 May", true, false),
            (3, "Thursday 28 May", "Thursday 28 May", false, true),
            (6, "Sunday 31 May", "Sunday 31 May", false, true),
        ];
        for (index, heading, full_date, is_today, is_future) in cases {
            let d = &week.days[index];
            assert_eq!(d.heading, heading, "day {index}");
            assert_eq!(d.full_date, full_date, "day {index}");
            assert_eq!(
                (d.is_today, d.is_future),
                (is_today, is_future),
                "day {index}"
            );
        }
    }

    #[test]
    fn the_strip_agrees_with_the_weekly_summary() {
        // Near-midnight sessions either side of both week boundaries, at BST.
        let sessions = vec![
            session("last-sun-late", at(2026, 5, 17, 22, 30)),
            session("last-mon-early", at(2026, 5, 17, 23, 30)),
            session("last-wed", at(2026, 5, 20, 10, 0)),
            session("this-mon-early", at(2026, 5, 24, 23, 30)),
            session("this-tue", at(2026, 5, 26, 18, 0)),
        ];
        let clock = clock(day(2026, 5, 27), 60);
        let weeks = compute_practice_weeks(&sessions, clock);
        let summary = compute_weekly_summary(&sessions, clock);
        let count =
            |w: &PracticeWeekView| -> usize { w.days.iter().map(|d| d.session_ids.len()).sum() };
        let this_week = &weeks[weeks.len() - 1];
        let last_week = &weeks[weeks.len() - 2];
        assert_eq!(count(this_week), 2);
        assert_eq!(count(this_week), summary.session_count);
        assert_eq!(count(last_week), 2);
        assert_eq!(count(last_week), summary.prev_session_count);
    }

    #[test]
    fn practice_weeks_round_trip_on_ffi_bincode_wire() {
        let sessions = sessions_on(&[day(2026, 5, 28)]);
        let weeks = compute_practice_weeks(&sessions, clock(day(2026, 5, 31), 0));
        assert!(!weeks.is_empty());
        crate::domain::types::assert_round_trips(weeks);
    }
}

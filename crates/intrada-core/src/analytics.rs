//! Analytics computations. All functions are pure and take `today: NaiveDate`
//! (rather than reading the clock) so they're deterministic under test.

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::domain::item::{Item, ItemKind};
use crate::domain::session::PracticeSession;

// ── Analytics View Model Types ───────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    #[default]
    Same,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct NeglectedItem {
    pub item_id: String,
    pub item_title: String,
    /// Days since last practised; `None` means never practised.
    pub days_since_practice: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScoreChange {
    pub item_id: String,
    pub item_title: String,
    /// Latest score before this week; `None` if scored for the first time.
    pub previous_score: Option<u8>,
    pub current_score: u8,
    /// Signed change (current − previous); 0 for newly scored items.
    pub delta: i8,
    pub is_new: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct AnalyticsView {
    pub weekly_summary: WeeklySummary,
    pub streak: PracticeStreak,
    pub daily_totals: Vec<DailyPracticeTotal>,
    pub top_items: Vec<ItemRanking>,
    pub score_trends: Vec<ItemScoreTrend>,
    pub neglected_items: Vec<NeglectedItem>,
    pub score_changes: Vec<ScoreChange>,
}

/// Aggregated stats for the current and previous ISO weeks (Monday–Sunday).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct WeeklySummary {
    pub total_minutes: u32,
    pub session_count: usize,
    pub items_covered: usize,
    pub prev_total_minutes: u32,
    pub prev_session_count: usize,
    pub prev_items_covered: usize,
    pub time_direction: Direction,
    pub sessions_direction: Direction,
    pub items_direction: Direction,
    pub has_prev_week_data: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeStreak {
    pub current_days: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct DailyPracticeTotal {
    pub date: String,
    pub minutes: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ItemRanking {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub total_minutes: u32,
    pub session_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ItemScoreTrend {
    pub item_id: String,
    pub item_title: String,
    pub scores: Vec<ScorePoint>,
    pub latest_score: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScorePoint {
    pub date: String,
    pub score: u8,
}

// ── Computation Functions ────────────────────────────────────────────

pub fn compute_analytics(
    sessions: &[PracticeSession],
    items: &[Item],
    today: NaiveDate,
) -> AnalyticsView {
    AnalyticsView {
        weekly_summary: compute_weekly_summary(sessions, today),
        streak: compute_streak(sessions, today),
        daily_totals: compute_daily_totals(sessions, today),
        top_items: compute_top_items(sessions),
        score_trends: compute_score_trends(sessions),
        neglected_items: compute_neglected_items(sessions, items, today),
        score_changes: compute_score_changes(sessions, today),
    }
}

/// Uses ISO week numbering (Monday = start of week).
pub fn compute_weekly_summary(sessions: &[PracticeSession], today: NaiveDate) -> WeeklySummary {
    let today_iso_week = today.iso_week();
    let prev_week_date = today - chrono::Duration::days(7);
    let prev_iso_week = prev_week_date.iso_week();

    let mut total_secs: u64 = 0;
    let mut session_count: usize = 0;
    let mut current_item_ids: HashSet<String> = HashSet::new();

    let mut prev_secs: u64 = 0;
    let mut prev_session_count: usize = 0;
    let mut prev_item_ids: HashSet<String> = HashSet::new();

    for session in sessions {
        let session_date = session.started_at.date_naive();
        let session_week = session_date.iso_week();

        if session_week == today_iso_week {
            total_secs += session.total_duration_secs;
            session_count += 1;
            for entry in &session.entries {
                current_item_ids.insert(entry.item_id.clone());
            }
        } else if session_week == prev_iso_week {
            prev_secs += session.total_duration_secs;
            prev_session_count += 1;
            for entry in &session.entries {
                prev_item_ids.insert(entry.item_id.clone());
            }
        }
    }

    let total_minutes = (total_secs / 60) as u32;
    let prev_total_minutes = (prev_secs / 60) as u32;
    let items_covered = current_item_ids.len();
    let prev_items_covered = prev_item_ids.len();
    let has_prev_week_data = prev_session_count > 0;

    fn direction(current: usize, previous: usize) -> Direction {
        if current > previous {
            Direction::Up
        } else if current < previous {
            Direction::Down
        } else {
            Direction::Same
        }
    }

    WeeklySummary {
        total_minutes,
        session_count,
        items_covered,
        prev_total_minutes,
        prev_session_count,
        prev_items_covered,
        time_direction: direction(total_minutes as usize, prev_total_minutes as usize),
        sessions_direction: direction(session_count, prev_session_count),
        items_direction: direction(items_covered, prev_items_covered),
        has_prev_week_data,
    }
}

/// Counts backwards from `today` (or yesterday if today has no session) as long
/// as each day has at least one session.
pub fn compute_streak(sessions: &[PracticeSession], today: NaiveDate) -> PracticeStreak {
    if sessions.is_empty() {
        return PracticeStreak { current_days: 0 };
    }

    let session_dates: HashSet<NaiveDate> =
        sessions.iter().map(|s| s.started_at.date_naive()).collect();

    let mut current = today;
    if !session_dates.contains(&current) {
        current = today - chrono::Duration::days(1);
    }

    let mut streak: u32 = 0;
    while session_dates.contains(&current) {
        streak += 1;
        current -= chrono::Duration::days(1);
    }

    PracticeStreak {
        current_days: streak,
    }
}

/// Returns exactly 28 entries, oldest first (today − 27 days through today).
pub fn compute_daily_totals(
    sessions: &[PracticeSession],
    today: NaiveDate,
) -> Vec<DailyPracticeTotal> {
    let mut secs_by_date: HashMap<NaiveDate, u64> = HashMap::new();
    for session in sessions {
        let date = session.started_at.date_naive();
        *secs_by_date.entry(date).or_default() += session.total_duration_secs;
    }

    (0..28)
        .rev()
        .map(|days_ago| {
            let date = today - chrono::Duration::days(days_ago);
            let minutes = (secs_by_date.get(&date).copied().unwrap_or(0) / 60) as u32;
            DailyPracticeTotal {
                date: date.format("%Y-%m-%d").to_string(),
                minutes,
            }
        })
        .collect()
}

/// Top 10 items by total time practised.
pub fn compute_top_items(sessions: &[PracticeSession]) -> Vec<ItemRanking> {
    let mut items: HashMap<String, (String, ItemKind, u64, HashSet<String>)> = HashMap::new();

    for session in sessions {
        for entry in &session.entries {
            let record = items.entry(entry.item_id.clone()).or_insert_with(|| {
                (
                    entry.item_title.clone(),
                    entry.item_type.clone(),
                    0,
                    HashSet::new(),
                )
            });
            record.2 += entry.duration_secs;
            record.3.insert(session.id.clone());
        }
    }

    let mut rankings: Vec<ItemRanking> = items
        .into_iter()
        .map(
            |(item_id, (title, item_type, total_secs, session_ids))| ItemRanking {
                item_id,
                item_title: title,
                item_type,
                total_minutes: (total_secs / 60) as u32,
                session_count: session_ids.len(),
            },
        )
        .collect();

    rankings.sort_by(|a, b| b.total_minutes.cmp(&a.total_minutes));
    rankings.truncate(10);
    rankings
}

/// The 5 most recently scored items, each with a chronological score series.
pub fn compute_score_trends(sessions: &[PracticeSession]) -> Vec<ItemScoreTrend> {
    let mut scored: HashMap<String, (String, Vec<(NaiveDate, u8)>)> = HashMap::new();

    for session in sessions {
        let session_date = session.started_at.date_naive();
        for entry in &session.entries {
            if let Some(score) = entry.score {
                let record = scored
                    .entry(entry.item_id.clone())
                    .or_insert_with(|| (entry.item_title.clone(), Vec::new()));
                record.1.push((session_date, score));
            }
        }
    }

    if scored.is_empty() {
        return Vec::new();
    }

    let mut trends: Vec<ItemScoreTrend> = scored
        .into_iter()
        .map(|(item_id, (title, mut score_points))| {
            score_points.sort_by_key(|(date, _)| *date);

            let latest_score = score_points.last().map(|(_, s)| *s).unwrap_or(0);

            let scores = score_points
                .iter()
                .map(|(date, score)| ScorePoint {
                    date: date.format("%Y-%m-%d").to_string(),
                    score: *score,
                })
                .collect();

            ItemScoreTrend {
                item_id,
                item_title: title,
                scores,
                latest_score,
            }
        })
        .collect();

    trends.sort_by(|a, b| {
        let a_latest = a.scores.last().map(|s| s.date.as_str()).unwrap_or("");
        let b_latest = b.scores.last().map(|s| s.date.as_str()).unwrap_or("");
        b_latest.cmp(a_latest)
    });

    trends.truncate(5);
    trends
}

/// Items not practised in the last 14 days. Up to 5, ordered never-practised
/// first, then by longest gap.
pub fn compute_neglected_items(
    sessions: &[PracticeSession],
    items: &[Item],
    today: NaiveDate,
) -> Vec<NeglectedItem> {
    if items.is_empty() {
        return Vec::new();
    }

    let lookback_start = today - chrono::Duration::days(13); // 14 days inclusive

    let mut recently_practised: HashSet<String> = HashSet::new();
    let mut latest_dates: HashMap<String, NaiveDate> = HashMap::new();

    for session in sessions {
        let session_date = session.started_at.date_naive();
        for entry in &session.entries {
            latest_dates
                .entry(entry.item_id.clone())
                .and_modify(|d| {
                    if session_date > *d {
                        *d = session_date;
                    }
                })
                .or_insert(session_date);

            if session_date >= lookback_start && session_date <= today {
                recently_practised.insert(entry.item_id.clone());
            }
        }
    }

    let mut neglected: Vec<NeglectedItem> = Vec::new();

    for item in items {
        if recently_practised.contains(&item.id) {
            continue;
        }

        let days_since_practice = latest_dates
            .get(&item.id)
            .map(|d| (today - *d).num_days().max(0) as u32);

        neglected.push(NeglectedItem {
            item_id: item.id.clone(),
            item_title: item.title.clone(),
            days_since_practice,
        });
    }

    neglected.sort_by(
        |a, b| match (&a.days_since_practice, &b.days_since_practice) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a_days), Some(b_days)) => b_days.cmp(a_days),
        },
    );

    neglected.truncate(5);
    neglected
}

/// This week's latest score vs the latest before it, per item. Up to 5, largest
/// absolute delta first.
pub fn compute_score_changes(sessions: &[PracticeSession], today: NaiveDate) -> Vec<ScoreChange> {
    let today_iso_week = today.iso_week();

    let mut this_week: HashMap<String, (u8, NaiveDate, String)> = HashMap::new();
    let mut prev: HashMap<String, (u8, NaiveDate)> = HashMap::new();

    for session in sessions {
        let session_date = session.started_at.date_naive();
        for entry in &session.entries {
            if let Some(score) = entry.score {
                if session_date.iso_week() == today_iso_week {
                    let existing = this_week.get(&entry.item_id);
                    if !matches!(existing, Some(e) if session_date < e.1) {
                        this_week.insert(
                            entry.item_id.clone(),
                            (score, session_date, entry.item_title.clone()),
                        );
                    }
                } else {
                    let existing = prev.get(&entry.item_id);
                    if !matches!(existing, Some(e) if session_date < e.1) {
                        prev.insert(entry.item_id.clone(), (score, session_date));
                    }
                }
            }
        }
    }

    let mut changes: Vec<ScoreChange> = Vec::new();

    for (item_id, (current_score, _date, item_title)) in &this_week {
        let previous = prev.get(item_id);

        match previous {
            Some((prev_score, _)) if *prev_score == *current_score => {
                continue;
            }
            Some((prev_score, _)) => {
                changes.push(ScoreChange {
                    item_id: item_id.clone(),
                    item_title: item_title.clone(),
                    previous_score: Some(*prev_score),
                    current_score: *current_score,
                    delta: *current_score as i8 - *prev_score as i8,
                    is_new: false,
                });
            }
            None => {
                changes.push(ScoreChange {
                    item_id: item_id.clone(),
                    item_title: item_title.clone(),
                    previous_score: None,
                    current_score: *current_score,
                    delta: 0,
                    is_new: true,
                });
            }
        }
    }

    changes.sort_by(|a, b| b.delta.unsigned_abs().cmp(&a.delta.unsigned_abs()));

    changes.truncate(5);
    changes
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::{CompletionStatus, EntryStatus, PracticeSession, SetlistEntry};
    use chrono::{NaiveDate, TimeZone, Utc};

    fn make_session(
        id: &str,
        date: NaiveDate,
        total_secs: u64,
        entries: Vec<SetlistEntry>,
    ) -> PracticeSession {
        let started = Utc.from_utc_datetime(&date.and_hms_opt(10, 0, 0).expect("valid time"));
        let finished = started + chrono::Duration::seconds(total_secs as i64);
        PracticeSession {
            id: id.to_string(),
            started_at: started,
            completed_at: finished,
            total_duration_secs: total_secs,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
            entries,
            session_score: None,
        }
    }

    fn make_entry(
        item_id: &str,
        title: &str,
        item_type: ItemKind,
        duration_secs: u64,
        score: Option<u8>,
    ) -> SetlistEntry {
        SetlistEntry {
            id: format!("entry-{item_id}-{duration_secs}"),
            item_id: item_id.to_string(),
            item_title: title.to_string(),
            item_type,
            position: 0,
            duration_secs,
            status: EntryStatus::Completed,
            notes: None,
            score,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs: None,
            achieved_tempo: None,
            group_id: None,
        }
    }

    // ── Weekly Summary Tests ──────────────────────────────────────────

    #[test]
    fn test_weekly_summary_basic() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(); // Wednesday
        let mon = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(); // Monday (same week)
        let tue = NaiveDate::from_ymd_opt(2026, 2, 17).unwrap();

        let sessions = vec![
            make_session("s1", mon, 1800, vec![]),  // 30 min
            make_session("s2", tue, 2700, vec![]),  // 45 min
            make_session("s3", today, 600, vec![]), // 10 min
        ];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.total_minutes, 85); // 30 + 45 + 10
        assert_eq!(summary.session_count, 3);
    }

    #[test]
    fn test_weekly_summary_excludes_previous_week() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(); // Wednesday
        let last_week = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(); // previous Wed

        let sessions = vec![
            make_session("s1", last_week, 3600, vec![]), // 60 min (previous week)
            make_session("s2", today, 1200, vec![]),     // 20 min (this week)
        ];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.total_minutes, 20);
        assert_eq!(summary.session_count, 1);
    }

    #[test]
    fn test_weekly_summary_empty() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let summary = compute_weekly_summary(&[], today);
        assert_eq!(summary.total_minutes, 0);
        assert_eq!(summary.session_count, 0);
    }

    // ── Week-over-week comparison tests ────────────────────────────────

    #[test]
    fn test_weekly_summary_comparison_both_weeks() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(); // Wed, week 8
        let this_mon = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(); // Wed, week 7
        let last_thu = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();

        let sessions = vec![
            // This week: 2 sessions, 50 min, 2 items
            make_session(
                "s1",
                this_mon,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
            make_session(
                "s2",
                today,
                1200,
                vec![make_entry("p2", "Scales", ItemKind::Exercise, 1200, None)],
            ),
            // Last week: 3 sessions, 90 min, 1 item
            make_session(
                "s3",
                last_wed,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
            make_session(
                "s4",
                last_wed,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
            make_session(
                "s5",
                last_thu,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
        ];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.total_minutes, 50);
        assert_eq!(summary.session_count, 2);
        assert_eq!(summary.items_covered, 2);
        assert_eq!(summary.prev_total_minutes, 90);
        assert_eq!(summary.prev_session_count, 3);
        assert_eq!(summary.prev_items_covered, 1);
        assert_eq!(summary.time_direction, Direction::Down);
        assert_eq!(summary.sessions_direction, Direction::Down);
        assert_eq!(summary.items_direction, Direction::Up);
        assert!(summary.has_prev_week_data);
    }

    #[test]
    fn test_weekly_summary_this_week_only() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![make_session(
            "s1",
            today,
            1800,
            vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
        )];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.total_minutes, 30);
        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.items_covered, 1);
        assert_eq!(summary.prev_total_minutes, 0);
        assert_eq!(summary.prev_session_count, 0);
        assert_eq!(summary.prev_items_covered, 0);
        assert!(!summary.has_prev_week_data);
    }

    #[test]
    fn test_weekly_summary_last_week_only() {
        // Monday morning — no sessions this week, sessions last week
        let today = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(); // Monday
        let last_fri = NaiveDate::from_ymd_opt(2026, 2, 13).unwrap();

        let sessions = vec![make_session(
            "s1",
            last_fri,
            3600,
            vec![make_entry("p1", "Sonata", ItemKind::Piece, 3600, None)],
        )];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.total_minutes, 0);
        assert_eq!(summary.session_count, 0);
        assert_eq!(summary.items_covered, 0);
        assert_eq!(summary.prev_total_minutes, 60);
        assert_eq!(summary.prev_session_count, 1);
        assert_eq!(summary.prev_items_covered, 1);
        assert!(summary.has_prev_week_data);
        assert_eq!(summary.time_direction, Direction::Down);
    }

    #[test]
    fn test_weekly_summary_items_covered_counts_distinct() {
        // Multiple entries for same item counted once
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![
            make_session(
                "s1",
                today,
                1800,
                vec![
                    make_entry("p1", "Sonata", ItemKind::Piece, 900, None),
                    make_entry("p1", "Sonata", ItemKind::Piece, 900, None), // same item
                ],
            ),
            make_session(
                "s2",
                today,
                600,
                vec![
                    make_entry("p1", "Sonata", ItemKind::Piece, 300, None), // same item again
                    make_entry("p2", "Scales", ItemKind::Exercise, 300, None),
                ],
            ),
        ];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.items_covered, 2); // p1 + p2, not 4
    }

    #[test]
    fn test_weekly_summary_directions() {
        // Up when current > prev, Down when current < prev, Same when equal
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let sessions = vec![
            // This week: 2 sessions, 60 min, 1 item
            make_session(
                "s1",
                today,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
            make_session(
                "s2",
                today,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
            // Last week: 2 sessions, 30 min, 1 item (sessions same, time down, items same)
            make_session(
                "s3",
                last_wed,
                900,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 900, None)],
            ),
            make_session(
                "s4",
                last_wed,
                900,
                vec![make_entry("p2", "Scales", ItemKind::Exercise, 900, None)],
            ),
        ];

        let summary = compute_weekly_summary(&sessions, today);
        assert_eq!(summary.time_direction, Direction::Up); // 60 > 30
        assert_eq!(summary.sessions_direction, Direction::Same); // 2 == 2
        assert_eq!(summary.items_direction, Direction::Down); // 1 < 2
    }

    #[test]
    fn test_weekly_summary_week_boundary() {
        // Sunday 23:55 belongs to ending week, Monday 00:05 to new week
        let monday = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap(); // Previous week's Sunday

        let sun_session = {
            use chrono::TimeZone;
            let started = chrono::Utc.from_utc_datetime(&sunday.and_hms_opt(23, 55, 0).unwrap());
            PracticeSession {
                id: "sun".to_string(),
                started_at: started,
                completed_at: started + chrono::Duration::seconds(600),
                total_duration_secs: 600,
                completion_status: crate::domain::session::CompletionStatus::Completed,
                session_notes: None,
                session_intention: None,
                entries: vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, None)],
                session_score: None,
            }
        };

        let mon_session = make_session(
            "mon",
            monday,
            1200,
            vec![make_entry("p2", "Scales", ItemKind::Exercise, 1200, None)],
        );

        // From Monday's perspective
        let summary = compute_weekly_summary(&[sun_session, mon_session], monday);
        assert_eq!(summary.session_count, 1); // Only Monday's session in current week
        assert_eq!(summary.prev_session_count, 1); // Sunday's session in previous week
        assert_eq!(summary.items_covered, 1); // p2 only
        assert_eq!(summary.prev_items_covered, 1); // p1 only
    }

    // ── Streak Tests ──────────────────────────────────────────────────

    #[test]
    fn test_streak_consecutive_days() {
        // 3 consecutive days ending today
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let yesterday = NaiveDate::from_ymd_opt(2026, 2, 17).unwrap();
        let day_before = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap();

        let sessions = vec![
            make_session("s1", day_before, 1800, vec![]),
            make_session("s2", yesterday, 1800, vec![]),
            make_session("s3", today, 1800, vec![]),
        ];

        let streak = compute_streak(&sessions, today);
        assert_eq!(streak.current_days, 3);
    }

    #[test]
    fn test_streak_broken() {
        // gap in days resets streak
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let yesterday = NaiveDate::from_ymd_opt(2026, 2, 17).unwrap();
        // Skip Feb 16
        let three_days_ago = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();

        let sessions = vec![
            make_session("s1", three_days_ago, 1800, vec![]),
            make_session("s2", yesterday, 1800, vec![]),
            make_session("s3", today, 1800, vec![]),
        ];

        let streak = compute_streak(&sessions, today);
        assert_eq!(streak.current_days, 2); // Only yesterday + today
    }

    #[test]
    fn test_streak_no_sessions_today() {
        // sessions on yesterday and day before, no session today
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let yesterday = NaiveDate::from_ymd_opt(2026, 2, 17).unwrap();
        let day_before = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap();

        let sessions = vec![
            make_session("s1", day_before, 1800, vec![]),
            make_session("s2", yesterday, 1800, vec![]),
        ];

        let streak = compute_streak(&sessions, today);
        assert_eq!(streak.current_days, 2);
    }

    #[test]
    fn test_streak_empty() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let streak = compute_streak(&[], today);
        assert_eq!(streak.current_days, 0);
    }

    // ── Daily Totals Tests ────────────────────────────────────────────

    #[test]
    fn test_daily_totals_28_days() {
        // sessions across 5 different days within past 28 days
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![
            make_session("s1", today, 1800, vec![]), // 30 min
            make_session("s2", today - chrono::Duration::days(1), 2700, vec![]), // 45 min
            make_session("s3", today - chrono::Duration::days(5), 600, vec![]), // 10 min
            make_session("s4", today - chrono::Duration::days(10), 3600, vec![]), // 60 min
            make_session("s5", today - chrono::Duration::days(27), 900, vec![]), // 15 min (oldest in range)
        ];

        let totals = compute_daily_totals(&sessions, today);
        assert_eq!(totals.len(), 28);

        assert_eq!(totals[0].date, "2026-01-22"); // 27 days ago
        assert_eq!(totals[0].minutes, 15); // s5

        assert_eq!(totals[27].date, "2026-02-18"); // today
        assert_eq!(totals[27].minutes, 30); // s1

        assert_eq!(totals[26].minutes, 45); // yesterday
        assert_eq!(totals[22].minutes, 10); // 5 days ago
        assert_eq!(totals[17].minutes, 60); // 10 days ago

        assert_eq!(totals[25].minutes, 0); // 2 days ago, no session
    }

    #[test]
    fn test_daily_totals_multiple_sessions_same_day() {
        // 3 sessions on the same day
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![
            make_session("s1", today, 1800, vec![]), // 30 min
            make_session("s2", today, 1200, vec![]), // 20 min
            make_session("s3", today, 600, vec![]),  // 10 min
        ];

        let totals = compute_daily_totals(&sessions, today);
        assert_eq!(totals[27].minutes, 60); // 30 + 20 + 10
    }

    #[test]
    fn test_daily_totals_empty() {
        // empty sessions → 28 entries all 0
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let totals = compute_daily_totals(&[], today);
        assert_eq!(totals.len(), 28);
        assert!(totals.iter().all(|t| t.minutes == 0));
    }

    // ── Top Items Tests ───────────────────────────────────────────────

    #[test]
    fn test_top_items_ranking() {
        // 5 items with varying durations, verify sorted by total_minutes descending
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![make_session(
            "s1",
            today,
            9000,
            vec![
                make_entry("p1", "Sonata", ItemKind::Piece, 3600, None), // 60 min
                make_entry("p2", "Etude", ItemKind::Piece, 1800, None),  // 30 min
                make_entry("e1", "Scales", ItemKind::Exercise, 900, None), // 15 min
                make_entry("e2", "Arps", ItemKind::Exercise, 1500, None), // 25 min
                make_entry("p3", "Nocturne", ItemKind::Piece, 1200, None), // 20 min
            ],
        )];

        let ranking = compute_top_items(&sessions);
        assert_eq!(ranking.len(), 5);
        assert_eq!(ranking[0].item_id, "p1"); // 60 min
        assert_eq!(ranking[0].total_minutes, 60);
        assert_eq!(ranking[1].item_id, "p2"); // 30 min
        assert_eq!(ranking[2].item_id, "e2"); // 25 min
        assert_eq!(ranking[3].item_id, "p3"); // 20 min
        assert_eq!(ranking[4].item_id, "e1"); // 15 min
    }

    #[test]
    fn test_top_items_max_10() {
        // 15 items → only top 10 returned
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let entries: Vec<SetlistEntry> = (0..15)
            .map(|i| {
                make_entry(
                    &format!("item{i}"),
                    &format!("Item {i}"),
                    ItemKind::Piece,
                    (i + 1) as u64 * 60, // 1 min, 2 min, ..., 15 min
                    None,
                )
            })
            .collect();

        let total_secs: u64 = entries.iter().map(|e| e.duration_secs).sum();
        let sessions = vec![make_session("s1", today, total_secs, entries)];

        let ranking = compute_top_items(&sessions);
        assert_eq!(ranking.len(), 10);
        // Highest should be item14 (15 min)
        assert_eq!(ranking[0].item_id, "item14");
        assert_eq!(ranking[0].total_minutes, 15);
    }

    #[test]
    fn test_top_items_session_count() {
        // same item in 3 sessions → session_count is 3
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let yesterday = today - chrono::Duration::days(1);
        let day_before = today - chrono::Duration::days(2);

        let sessions = vec![
            make_session(
                "s1",
                today,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
            ),
            make_session(
                "s2",
                yesterday,
                1200,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1200, None)],
            ),
            make_session(
                "s3",
                day_before,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, None)],
            ),
        ];

        let ranking = compute_top_items(&sessions);
        assert_eq!(ranking.len(), 1);
        assert_eq!(ranking[0].session_count, 3);
        assert_eq!(ranking[0].total_minutes, 60); // (1800+1200+600)/60 = 60
    }

    #[test]
    fn test_top_items_empty() {
        let ranking = compute_top_items(&[]);
        assert!(ranking.is_empty());
    }

    // ── Score Trends Tests ────────────────────────────────────────────

    #[test]
    fn test_score_trends_basic() {
        // 3 sessions scoring the same item with 2, 3, 4
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let d1 = today - chrono::Duration::days(2);
        let d2 = today - chrono::Duration::days(1);
        let d3 = today;

        let sessions = vec![
            make_session(
                "s1",
                d1,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, Some(2))],
            ),
            make_session(
                "s2",
                d2,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, Some(3))],
            ),
            make_session(
                "s3",
                d3,
                1800,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, Some(4))],
            ),
        ];

        let trends = compute_score_trends(&sessions);
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].item_id, "p1");
        assert_eq!(trends[0].latest_score, 4);
        assert_eq!(trends[0].scores.len(), 3);
        // Chronological order
        assert_eq!(trends[0].scores[0].score, 2);
        assert_eq!(trends[0].scores[1].score, 3);
        assert_eq!(trends[0].scores[2].score, 4);
    }

    #[test]
    fn test_score_trends_max_5_items() {
        // 8 items scored → only 5 most recently scored returned
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let entries: Vec<SetlistEntry> = (0..8)
            .map(|i| {
                make_entry(
                    &format!("item{i}"),
                    &format!("Item {i}"),
                    ItemKind::Piece,
                    900,
                    Some(3),
                )
            })
            .collect();

        // Create sessions on different days so each item has a different "most recent" date
        let sessions: Vec<PracticeSession> = (0..8)
            .map(|i| {
                let date = today - chrono::Duration::days(i);
                make_session(
                    &format!("s{i}"),
                    date,
                    900,
                    vec![entries[i as usize].clone()],
                )
            })
            .collect();

        let trends = compute_score_trends(&sessions);
        assert_eq!(trends.len(), 5);
        // Most recent first: item0 (today), item1 (yesterday), ...
        assert_eq!(trends[0].item_id, "item0");
        assert_eq!(trends[4].item_id, "item4");
    }

    #[test]
    fn test_score_trends_excludes_unscored() {
        // mix of scored and unscored entries
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![make_session(
            "s1",
            today,
            3600,
            vec![
                make_entry("p1", "Sonata", ItemKind::Piece, 1800, Some(4)), // scored
                make_entry("p2", "Etude", ItemKind::Piece, 1800, None),     // unscored
            ],
        )];

        let trends = compute_score_trends(&sessions);
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].item_id, "p1");
    }

    #[test]
    fn test_score_trends_empty() {
        // sessions with no scored entries
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let sessions = vec![make_session(
            "s1",
            today,
            1800,
            vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, None)],
        )];

        let trends = compute_score_trends(&sessions);
        assert!(trends.is_empty());
    }

    // ── Neglected Items Tests ─────────────────────────────────────────

    fn make_item(id: &str, title: &str) -> Item {
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind: crate::domain::item::ItemKind::Piece,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            linked_exercise_ids: vec![],
            priority: false,
        }
    }

    #[test]
    fn test_neglected_items_basic() {
        // 10 items, 4 practised this week → 6 neglected, capped at 5
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items: Vec<Item> = (1..=10)
            .map(|i| make_item(&format!("p{i}"), &format!("Item {i}")))
            .collect();

        // Practice 4 items within last 14 days
        let sessions = vec![
            make_session(
                "s1",
                today,
                1800,
                vec![
                    make_entry("p1", "Item 1", ItemKind::Piece, 600, None),
                    make_entry("p2", "Item 2", ItemKind::Piece, 600, None),
                ],
            ),
            make_session(
                "s2",
                today - chrono::Duration::days(5),
                1200,
                vec![
                    make_entry("p3", "Item 3", ItemKind::Piece, 600, None),
                    make_entry("p4", "Item 4", ItemKind::Piece, 600, None),
                ],
            ),
        ];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert_eq!(neglected.len(), 5); // capped at 5 out of 6
        for n in &neglected {
            assert!(!["p1", "p2", "p3", "p4"].contains(&n.item_id.as_str()));
        }
    }

    #[test]
    fn test_neglected_items_never_practised_sort_first() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items = vec![
            make_item("p1", "Old Item"),
            make_item("p2", "Never Practised"),
        ];

        // p1 was practised 20 days ago, p2 never
        let sessions = vec![make_session(
            "s1",
            today - chrono::Duration::days(20),
            600,
            vec![make_entry("p1", "Old Item", ItemKind::Piece, 600, None)],
        )];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert_eq!(neglected.len(), 2);
        assert_eq!(neglected[0].item_id, "p2"); // never practised first
        assert!(neglected[0].days_since_practice.is_none());
        assert_eq!(neglected[1].item_id, "p1");
        assert_eq!(neglected[1].days_since_practice, Some(20));
    }

    #[test]
    fn test_neglected_items_ordered_by_days_descending() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items = vec![
            make_item("p1", "A"),
            make_item("p2", "B"),
            make_item("p3", "C"),
        ];

        let sessions = vec![
            make_session(
                "s1",
                today - chrono::Duration::days(20),
                600,
                vec![make_entry("p1", "A", ItemKind::Piece, 600, None)],
            ),
            make_session(
                "s2",
                today - chrono::Duration::days(30),
                600,
                vec![make_entry("p2", "B", ItemKind::Piece, 600, None)],
            ),
            make_session(
                "s3",
                today - chrono::Duration::days(15),
                600,
                vec![make_entry("p3", "C", ItemKind::Piece, 600, None)],
            ),
        ];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert_eq!(neglected.len(), 3);
        assert_eq!(neglected[0].item_id, "p2"); // 30 days
        assert_eq!(neglected[1].item_id, "p1"); // 20 days
        assert_eq!(neglected[2].item_id, "p3"); // 15 days
    }

    #[test]
    fn test_neglected_items_all_recent() {
        // All items practised within 14 days → empty result
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items = vec![make_item("p1", "A"), make_item("p2", "B")];

        let sessions = vec![make_session(
            "s1",
            today - chrono::Duration::days(5),
            1200,
            vec![
                make_entry("p1", "A", ItemKind::Piece, 600, None),
                make_entry("p2", "B", ItemKind::Piece, 600, None),
            ],
        )];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert!(neglected.is_empty());
    }

    #[test]
    fn test_neglected_items_empty_library() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let neglected = compute_neglected_items(&[], &[], today);
        assert!(neglected.is_empty());
    }

    #[test]
    fn test_neglected_items_deleted_item_not_included() {
        // Item in session but not in current items list → not in neglected
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items = vec![make_item("p1", "Existing")];

        // Session references p2 which is not in items
        let sessions = vec![make_session(
            "s1",
            today - chrono::Duration::days(20),
            600,
            vec![make_entry("p2", "Deleted", ItemKind::Piece, 600, None)],
        )];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert_eq!(neglected.len(), 1);
        assert_eq!(neglected[0].item_id, "p1"); // only existing item
        assert!(neglected[0].days_since_practice.is_none()); // never practised
    }

    #[test]
    fn test_neglected_items_13_days_not_neglected() {
        // Item practised 13 days ago → within 14-day window → not neglected
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items = vec![make_item("p1", "Recent")];

        let sessions = vec![make_session(
            "s1",
            today - chrono::Duration::days(13),
            600,
            vec![make_entry("p1", "Recent", ItemKind::Piece, 600, None)],
        )];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert!(neglected.is_empty());
    }

    #[test]
    fn test_neglected_items_14_days_is_neglected() {
        // Item practised exactly 14 days ago → outside 14-day window → neglected
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let items = vec![make_item("p1", "Old")];

        let sessions = vec![make_session(
            "s1",
            today - chrono::Duration::days(14),
            600,
            vec![make_entry("p1", "Old", ItemKind::Piece, 600, None)],
        )];

        let neglected = compute_neglected_items(&sessions, &items, today);
        assert_eq!(neglected.len(), 1);
        assert_eq!(neglected[0].item_id, "p1");
        assert_eq!(neglected[0].days_since_practice, Some(14));
    }

    // ── Score Changes Tests ───────────────────────────────────────────

    #[test]
    fn test_score_changes_improvement() {
        // Item scored 2 last week, 4 this week → delta +2
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(); // Wed, week 8
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(); // Wed, week 7

        let sessions = vec![
            make_session(
                "s1",
                last_wed,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(2))],
            ),
            make_session(
                "s2",
                today,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(4))],
            ),
        ];

        let changes = compute_score_changes(&sessions, today);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].item_id, "p1");
        assert_eq!(changes[0].previous_score, Some(2));
        assert_eq!(changes[0].current_score, 4);
        assert_eq!(changes[0].delta, 2);
        assert!(!changes[0].is_new);
    }

    #[test]
    fn test_score_changes_decrease() {
        // Item scored 4 last week, 3 this week → delta -1
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let sessions = vec![
            make_session(
                "s1",
                last_wed,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(4))],
            ),
            make_session(
                "s2",
                today,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
            ),
        ];

        let changes = compute_score_changes(&sessions, today);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].delta, -1);
    }

    #[test]
    fn test_score_changes_newly_scored() {
        // Item scored for first time this week
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![make_session(
            "s1",
            today,
            600,
            vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
        )];

        let changes = compute_score_changes(&sessions, today);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].previous_score, None);
        assert_eq!(changes[0].current_score, 3);
        assert_eq!(changes[0].delta, 0);
        assert!(changes[0].is_new);
    }

    #[test]
    fn test_score_changes_empty() {
        // No items scored this week
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let sessions = vec![make_session(
            "s1",
            last_wed,
            600,
            vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
        )];

        let changes = compute_score_changes(&sessions, today);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_score_changes_capped_at_5() {
        // More than 5 score changes → capped at 5, sorted by largest absolute delta
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let mut last_entries = Vec::new();
        let mut this_entries = Vec::new();
        for i in 1..=7 {
            last_entries.push(make_entry(
                &format!("p{i}"),
                &format!("Item {i}"),
                ItemKind::Piece,
                600,
                Some(1),
            ));
            this_entries.push(make_entry(
                &format!("p{i}"),
                &format!("Item {i}"),
                ItemKind::Piece,
                600,
                Some(1 + i as u8), // deltas: 1,2,3,4,5,6,7
            ));
        }

        let sessions = vec![
            make_session("s1", last_wed, 4200, last_entries),
            make_session("s2", today, 4200, this_entries),
        ];

        let changes = compute_score_changes(&sessions, today);
        assert_eq!(changes.len(), 5);
        // Should be sorted by largest absolute delta
        assert!(changes[0].delta.unsigned_abs() >= changes[1].delta.unsigned_abs());
    }

    #[test]
    fn test_score_changes_latest_score_this_week() {
        // Item scored multiple times this week → uses latest
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let mon = NaiveDate::from_ymd_opt(2026, 2, 16).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let sessions = vec![
            make_session(
                "s1",
                last_wed,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(2))],
            ),
            make_session(
                "s2",
                mon,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
            ),
            make_session(
                "s3",
                today,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(5))],
            ),
        ];

        let changes = compute_score_changes(&sessions, today);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].current_score, 5); // latest this week
        assert_eq!(changes[0].previous_score, Some(2));
        assert_eq!(changes[0].delta, 3);
    }

    #[test]
    fn test_score_changes_same_score_excluded() {
        // Item scored same as last week → not included
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let last_wed = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let sessions = vec![
            make_session(
                "s1",
                last_wed,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
            ),
            make_session(
                "s2",
                today,
                600,
                vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
            ),
        ];

        let changes = compute_score_changes(&sessions, today);
        assert!(changes.is_empty());
    }

    // ── Integration: compute_analytics ───────────────────────────────

    #[test]
    fn test_compute_analytics_populates_all_fields() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let sessions = vec![make_session(
            "s1",
            today,
            1800,
            vec![make_entry("p1", "Sonata", ItemKind::Piece, 1800, Some(4))],
        )];

        let analytics = compute_analytics(&sessions, &[], today);
        assert_eq!(analytics.weekly_summary.session_count, 1);
        assert_eq!(analytics.streak.current_days, 1);
        assert_eq!(analytics.daily_totals.len(), 28);
        assert_eq!(analytics.top_items.len(), 1);
        assert_eq!(analytics.score_trends.len(), 1);
    }

    // ── Edge case: ended-early sessions included ──────────────────────

    #[test]
    fn test_ended_early_sessions_included() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let started = Utc.from_utc_datetime(&today.and_hms_opt(10, 0, 0).unwrap());

        let sessions = vec![PracticeSession {
            id: "s1".to_string(),
            started_at: started,
            completed_at: started + chrono::Duration::seconds(600),
            total_duration_secs: 600,
            completion_status: CompletionStatus::EndedEarly,
            session_notes: None,
            session_intention: None,
            entries: vec![make_entry("p1", "Sonata", ItemKind::Piece, 600, Some(3))],
            session_score: None,
        }];

        let analytics = compute_analytics(&sessions, &[], today);
        assert_eq!(analytics.weekly_summary.session_count, 1);
        assert_eq!(analytics.weekly_summary.total_minutes, 10);
        assert_eq!(analytics.streak.current_days, 1);
        assert_eq!(analytics.top_items.len(), 1);
        assert_eq!(analytics.score_trends.len(), 1);
    }
}

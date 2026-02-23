//! Pure analytics computation functions for the Practice Analytics Dashboard.
//!
//! All functions in this module are pure (no I/O, no system clock access) and
//! accept a `today: NaiveDate` parameter for deterministic testing.
//! They operate on existing `PracticeSession` data without creating new
//! persistence or requiring additional API endpoints.

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::domain::session::PracticeSession;

// ── Analytics View Model Types ───────────────────────────────────────

/// Top-level analytics container, added to the existing `ViewModel`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AnalyticsView {
    pub weekly_summary: WeeklySummary,
    pub streak: PracticeStreak,
    pub daily_totals: Vec<DailyPracticeTotal>,
    pub top_items: Vec<ItemRanking>,
    pub score_trends: Vec<ItemScoreTrend>,
}

/// Aggregated stats for the current ISO week (Monday–Sunday).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct WeeklySummary {
    pub total_minutes: u32,
    pub session_count: usize,
}

/// Consecutive-day practice count.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PracticeStreak {
    pub current_days: u32,
}

/// One entry per day for the 28-day history chart.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DailyPracticeTotal {
    pub date: String,
    pub minutes: u32,
}

/// Per-item aggregation for the "most practised" list.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemRanking {
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    pub total_minutes: u32,
    pub session_count: usize,
}

/// Score progression for a single item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemScoreTrend {
    pub item_id: String,
    pub item_title: String,
    pub scores: Vec<ScorePoint>,
    pub latest_score: u8,
}

/// Single data point in a score trend.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScorePoint {
    pub date: String,
    pub score: u8,
}

// ── Computation Functions ────────────────────────────────────────────

/// Compute all analytics from session data.
pub fn compute_analytics(sessions: &[PracticeSession], today: NaiveDate) -> AnalyticsView {
    AnalyticsView {
        weekly_summary: compute_weekly_summary(sessions, today),
        streak: compute_streak(sessions, today),
        daily_totals: compute_daily_totals(sessions, today),
        top_items: compute_top_items(sessions),
        score_trends: compute_score_trends(sessions),
    }
}

/// Compute weekly summary: total minutes and session count for the current ISO week.
///
/// Uses ISO week numbering (Monday = start of week). Sums `total_duration_secs`
/// for all sessions whose `started_at` falls within the same ISO week as `today`.
pub fn compute_weekly_summary(sessions: &[PracticeSession], today: NaiveDate) -> WeeklySummary {
    let today_iso_week = today.iso_week();

    let mut total_secs: u64 = 0;
    let mut session_count: usize = 0;

    for session in sessions {
        let session_date = session.started_at.date_naive();
        if session_date.iso_week() == today_iso_week {
            total_secs += session.total_duration_secs;
            session_count += 1;
        }
    }

    WeeklySummary {
        total_minutes: (total_secs / 60) as u32,
        session_count,
    }
}

/// Compute practice streak: consecutive days with at least one session.
///
/// Counts backwards from `today` (or yesterday if today has no session)
/// as long as each day has at least one session.
pub fn compute_streak(sessions: &[PracticeSession], today: NaiveDate) -> PracticeStreak {
    if sessions.is_empty() {
        return PracticeStreak { current_days: 0 };
    }

    // Collect unique dates that had a session
    let session_dates: HashSet<NaiveDate> =
        sessions.iter().map(|s| s.started_at.date_naive()).collect();

    // Start counting from today; if today has no session, start from yesterday
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

/// Compute daily practice totals for the past 28 days.
///
/// Returns exactly 28 `DailyPracticeTotal` entries, oldest first (today - 27 days through today).
/// For each day, sums `total_duration_secs` across all sessions started on that day,
/// converted to minutes. Days with no sessions have `minutes: 0`.
pub fn compute_daily_totals(
    sessions: &[PracticeSession],
    today: NaiveDate,
) -> Vec<DailyPracticeTotal> {
    // Aggregate seconds per date
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

/// Compute top 10 most-practised items ranked by total time.
///
/// Aggregates all entries across all sessions by `item_id`, sums `duration_secs`
/// (converted to minutes), counts distinct sessions per item, sorts by total_minutes
/// descending, takes top 10.
pub fn compute_top_items(sessions: &[PracticeSession]) -> Vec<ItemRanking> {
    // item_id -> (title, type, total_secs, set of session_ids)
    let mut items: HashMap<String, (String, String, u64, HashSet<String>)> = HashMap::new();

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

/// Compute score trends for the 5 most recently scored items.
///
/// Collects all entries with `score: Some(n)`, groups by `item_id`, builds
/// chronological `ScorePoint` lists, sorts items by most recent score date,
/// takes top 5.
pub fn compute_score_trends(sessions: &[PracticeSession]) -> Vec<ItemScoreTrend> {
    // item_id -> (title, Vec<(date, score)>)
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
            // Sort chronologically (oldest first)
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

    // Sort by most recent score date descending
    trends.sort_by(|a, b| {
        let a_latest = a.scores.last().map(|s| s.date.as_str()).unwrap_or("");
        let b_latest = b.scores.last().map(|s| s.date.as_str()).unwrap_or("");
        b_latest.cmp(a_latest)
    });

    trends.truncate(5);
    trends
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::{CompletionStatus, EntryStatus, PracticeSession, SetlistEntry};
    use chrono::{NaiveDate, TimeZone, Utc};

    /// Helper: create a PracticeSession on a given date with total_duration_secs.
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
        }
    }

    /// Helper: create a basic SetlistEntry.
    fn make_entry(
        item_id: &str,
        title: &str,
        item_type: &str,
        duration_secs: u64,
        score: Option<u8>,
    ) -> SetlistEntry {
        SetlistEntry {
            id: format!("entry-{item_id}-{duration_secs}"),
            item_id: item_id.to_string(),
            item_title: title.to_string(),
            item_type: item_type.to_string(),
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
        }
    }

    // ── US1: Weekly Summary Tests ────────────────────────────────────

    #[test]
    fn test_weekly_summary_basic() {
        // T013: 3 sessions within the current ISO week
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
        // T014: only current week sessions counted
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
        // T015: empty session list
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let summary = compute_weekly_summary(&[], today);
        assert_eq!(summary.total_minutes, 0);
        assert_eq!(summary.session_count, 0);
    }

    // ── US1: Streak Tests ────────────────────────────────────────────

    #[test]
    fn test_streak_consecutive_days() {
        // T016: 3 consecutive days ending today
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
        // T017: gap in days resets streak
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
        // T018: sessions on yesterday and day before, no session today
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
        // T019: empty session list
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let streak = compute_streak(&[], today);
        assert_eq!(streak.current_days, 0);
    }

    // ── US2: Daily Totals Tests ──────────────────────────────────────

    #[test]
    fn test_daily_totals_28_days() {
        // T026: sessions across 5 different days within past 28 days
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

        // First entry is 27 days ago
        assert_eq!(totals[0].date, "2026-01-22");
        assert_eq!(totals[0].minutes, 15); // s5

        // Last entry is today
        assert_eq!(totals[27].date, "2026-02-18");
        assert_eq!(totals[27].minutes, 30); // s1

        // Spot checks
        assert_eq!(totals[26].minutes, 45); // yesterday
        assert_eq!(totals[22].minutes, 10); // 5 days ago
        assert_eq!(totals[17].minutes, 60); // 10 days ago

        // Empty days should be 0
        assert_eq!(totals[25].minutes, 0); // 2 days ago
    }

    #[test]
    fn test_daily_totals_multiple_sessions_same_day() {
        // T027: 3 sessions on the same day
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
        // T028: empty sessions → 28 entries all 0
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let totals = compute_daily_totals(&[], today);
        assert_eq!(totals.len(), 28);
        assert!(totals.iter().all(|t| t.minutes == 0));
    }

    // ── US3: Top Items Tests ─────────────────────────────────────────

    #[test]
    fn test_top_items_ranking() {
        // T034: 5 items with varying durations, verify sorted by total_minutes descending
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![make_session(
            "s1",
            today,
            9000,
            vec![
                make_entry("p1", "Sonata", "piece", 3600, None), // 60 min
                make_entry("p2", "Etude", "piece", 1800, None),  // 30 min
                make_entry("e1", "Scales", "exercise", 900, None), // 15 min
                make_entry("e2", "Arps", "exercise", 1500, None), // 25 min
                make_entry("p3", "Nocturne", "piece", 1200, None), // 20 min
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
        // T035: 15 items → only top 10 returned
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let entries: Vec<SetlistEntry> = (0..15)
            .map(|i| {
                make_entry(
                    &format!("item{i}"),
                    &format!("Item {i}"),
                    "piece",
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
        // T036: same item in 3 sessions → session_count is 3
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let yesterday = today - chrono::Duration::days(1);
        let day_before = today - chrono::Duration::days(2);

        let sessions = vec![
            make_session(
                "s1",
                today,
                1800,
                vec![make_entry("p1", "Sonata", "piece", 1800, None)],
            ),
            make_session(
                "s2",
                yesterday,
                1200,
                vec![make_entry("p1", "Sonata", "piece", 1200, None)],
            ),
            make_session(
                "s3",
                day_before,
                600,
                vec![make_entry("p1", "Sonata", "piece", 600, None)],
            ),
        ];

        let ranking = compute_top_items(&sessions);
        assert_eq!(ranking.len(), 1);
        assert_eq!(ranking[0].session_count, 3);
        assert_eq!(ranking[0].total_minutes, 60); // (1800+1200+600)/60 = 60
    }

    #[test]
    fn test_top_items_empty() {
        // T037: empty sessions
        let ranking = compute_top_items(&[]);
        assert!(ranking.is_empty());
    }

    // ── US4: Score Trends Tests ──────────────────────────────────────

    #[test]
    fn test_score_trends_basic() {
        // T041: 3 sessions scoring the same item with 2, 3, 4
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let d1 = today - chrono::Duration::days(2);
        let d2 = today - chrono::Duration::days(1);
        let d3 = today;

        let sessions = vec![
            make_session(
                "s1",
                d1,
                1800,
                vec![make_entry("p1", "Sonata", "piece", 1800, Some(2))],
            ),
            make_session(
                "s2",
                d2,
                1800,
                vec![make_entry("p1", "Sonata", "piece", 1800, Some(3))],
            ),
            make_session(
                "s3",
                d3,
                1800,
                vec![make_entry("p1", "Sonata", "piece", 1800, Some(4))],
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
        // T042: 8 items scored → only 5 most recently scored returned
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let entries: Vec<SetlistEntry> = (0..8)
            .map(|i| {
                make_entry(
                    &format!("item{i}"),
                    &format!("Item {i}"),
                    "piece",
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
        // T043: mix of scored and unscored entries
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();

        let sessions = vec![make_session(
            "s1",
            today,
            3600,
            vec![
                make_entry("p1", "Sonata", "piece", 1800, Some(4)), // scored
                make_entry("p2", "Etude", "piece", 1800, None),     // unscored
            ],
        )];

        let trends = compute_score_trends(&sessions);
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].item_id, "p1");
    }

    #[test]
    fn test_score_trends_empty() {
        // T044: sessions with no scored entries
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let sessions = vec![make_session(
            "s1",
            today,
            1800,
            vec![make_entry("p1", "Sonata", "piece", 1800, None)],
        )];

        let trends = compute_score_trends(&sessions);
        assert!(trends.is_empty());
    }

    // ── Integration: compute_analytics ───────────────────────────────

    #[test]
    fn test_compute_analytics_populates_all_fields() {
        let today = NaiveDate::from_ymd_opt(2026, 2, 18).unwrap();
        let sessions = vec![make_session(
            "s1",
            today,
            1800,
            vec![make_entry("p1", "Sonata", "piece", 1800, Some(4))],
        )];

        let analytics = compute_analytics(&sessions, today);
        assert_eq!(analytics.weekly_summary.session_count, 1);
        assert_eq!(analytics.streak.current_days, 1);
        assert_eq!(analytics.daily_totals.len(), 28);
        assert_eq!(analytics.top_items.len(), 1);
        assert_eq!(analytics.score_trends.len(), 1);
    }

    // ── Edge case: ended-early sessions included (FR-009) ────────────

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
            entries: vec![make_entry("p1", "Sonata", "piece", 600, Some(3))],
        }];

        let analytics = compute_analytics(&sessions, today);
        assert_eq!(analytics.weekly_summary.session_count, 1);
        assert_eq!(analytics.weekly_summary.total_minutes, 10);
        assert_eq!(analytics.streak.current_days, 1);
        assert_eq!(analytics.top_items.len(), 1);
        assert_eq!(analytics.score_trends.len(), 1);
    }
}

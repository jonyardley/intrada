//! The getting-cold signal (#1416, `specs/getting-cold-signal.md`): how long an
//! item has sat, graded against how long it can reasonably bear to sit.
//!
//! The same gap means different things for different material. A tune marked
//! 3 of 10 and played once is fragile; the same tune marked 9 of 10 across a
//! year of returns is not, and the binary 14-day flag this replaces sent both
//! cold on the same clock.

use crate::analytics::LocalClock;
use crate::model::ItemPracticeSummary;

/// How cold an item has gone, relative to its own expected return interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StalenessBand {
    /// Never practised, or the recorded date can't be read.
    Unknown,
    Fresh,
    GoingCold,
    Cold,
}

/// Fields are private so `grade` is the only constructor: `band == Unknown`
/// exactly when `days == None` is an invariant the compiler then enforces,
/// rather than prose `clause()` has to defend with a dead arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Staleness {
    band: StalenessBand,
    days: Option<u32>,
    expected_interval_days: u32,
}

/// How long an item can reasonably sit before coming back is worth suggesting.
/// Fixed and deliberately lax: no published scheduler is validated for
/// repertoire, one musician will never produce enough history to fit one, and a
/// signal that cries wolf gets ignored (`space-layer.md` §4, §6).
fn expected_interval_days(mark: Option<u8>, session_count: usize) -> u32 {
    let base = match mark {
        None => 10,
        Some(0..=3) => 5,
        Some(4..=6) => 9,
        Some(7..=8) => 16,
        Some(_) => 30,
    };
    let returns_bonus = match session_count {
        0..=1 => 0,
        2..=4 => 2,
        5..=9 => 5,
        _ => 9,
    };
    base + returns_bonus
}

fn grade(days: Option<u32>, mark: Option<u8>, session_count: usize) -> Staleness {
    let expected_interval_days = expected_interval_days(mark, session_count);
    let band = match days {
        None => StalenessBand::Unknown,
        Some(d) if d <= expected_interval_days => StalenessBand::Fresh,
        Some(d) if d <= expected_interval_days * 2 => StalenessBand::GoingCold,
        Some(_) => StalenessBand::Cold,
    };
    Staleness {
        band,
        days,
        expected_interval_days,
    }
}

/// `mark` is the mark the caller will *show*, not always the summary's flat
/// `latest_score`: per-piece is the grain #1081 established.
pub fn assess(
    practice: Option<&ItemPracticeSummary>,
    mark: Option<u8>,
    clock: LocalClock,
) -> Staleness {
    grade(
        days_since_practice(practice, clock),
        mark,
        practice.map_or(0, |p| p.session_count),
    )
}

/// `None` for never practised *and* for a date that can't be read. Defensive:
/// `build_practice_summaries` writes this from a `DateTime<Utc>`, so the parse
/// cannot fail today. An unreadable one degrades to the never-practised
/// treatment, which reads honestly ("not practised yet") but does rank stalest.
fn days_since_practice(practice: Option<&ItemPracticeSummary>, clock: LocalClock) -> Option<u32> {
    let at = practice?.last_practiced_at.as_deref()?;
    let parsed = chrono::DateTime::parse_from_rfc3339(at).ok()?;
    let day = clock.day_of(parsed.with_timezone(&chrono::Utc));
    Some((clock.today - day).num_days().max(0) as u32)
}

impl Staleness {
    /// Days since last practised, for a surface that reports the gap as a fact
    /// alongside the grade.
    pub(crate) fn days(&self) -> Option<u32> {
        self.days
    }

    /// The only band not worth surfacing: `Unknown` means an item the library
    /// has never got to, which is its own kind of left behind.
    pub fn is_fresh(&self) -> bool {
        matches!(self.band, StalenessBand::Fresh)
    }

    /// How far through its interval the item has drifted, in hundredths.
    /// Ranking on this rather than raw days is what keeps
    /// `specs/up-next-card.md` decision 3 true: the headline says the same
    /// thing the ranking used. Never practised is the stalest of all.
    pub fn overdue_key(&self) -> u32 {
        match self.days {
            None => u32::MAX,
            Some(d) => d.saturating_mul(100) / self.expected_interval_days,
        }
    }

    /// Mid-sentence, so it can follow another clause; the caller heads a line
    /// with it. Written against `docs/tone-of-voice.md`.
    pub fn clause(&self) -> String {
        match (self.band, self.days) {
            (StalenessBand::Unknown, _) | (_, None) => "not practised yet".to_string(),
            (StalenessBand::Fresh, Some(0)) => "practised today".to_string(),
            (StalenessBand::Fresh, Some(1)) => "practised yesterday".to_string(),
            (StalenessBand::Fresh, Some(d)) => format!("practised {} ago", gap_phrase(d)),
            (StalenessBand::GoingCold, Some(d)) => format!("going cold after {}", gap_phrase(d)),
            (StalenessBand::Cold, Some(d)) => format!("cold for {}", gap_phrase(d)),
        }
    }
}

/// Grading the units matters as much as grading the band: "not practised for
/// 137 days" is a number nobody reads, where "cold for 5 months" is a sentence.
fn gap_phrase(days: u32) -> String {
    match days {
        0..=13 => format!("{days} days"),
        14..=55 => format!("{} weeks", (days + 3) / 7),
        56..=364 => format!("{} months", (days + 15) / 30),
        _ => "over a year".to_string(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::StalenessBand::*;
    use super::*;
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

    fn practised(days_ago: i64, sessions: usize) -> ItemPracticeSummary {
        let at = (today() - chrono::Duration::days(days_ago))
            .and_hms_opt(10, 0, 0)
            .expect("valid time")
            .and_utc()
            .to_rfc3339();
        ItemPracticeSummary {
            session_count: sessions,
            last_practiced_at: Some(at),
            ..ItemPracticeSummary::fixture()
        }
    }

    // ── The interval ─────────────────────────────────────────────────

    #[test]
    fn a_rough_mark_earns_a_shorter_interval_than_a_solid_one() {
        let rough = expected_interval_days(Some(2), 4);
        let solid = expected_interval_days(Some(9), 4);
        assert!(
            rough < solid,
            "fragile {rough} should come back sooner than consolidated {solid}"
        );
    }

    #[test]
    fn returning_to_an_item_lengthens_its_interval() {
        let once = expected_interval_days(Some(6), 1);
        let often = expected_interval_days(Some(6), 20);
        assert!(
            once < often,
            "played once {once} is less established than returned to twenty times {often}"
        );
    }

    #[test]
    fn an_unmarked_item_sits_between_fragile_and_consolidated() {
        let unmarked = expected_interval_days(None, 3);
        assert!(unmarked > expected_interval_days(Some(1), 3));
        assert!(unmarked < expected_interval_days(Some(9), 3));
    }

    /// `overdue_key` divides by the interval, so a retune that produced a zero
    /// would panic rather than misrank.
    #[test]
    fn no_mark_and_return_count_can_produce_a_zero_interval() {
        for mark in (0..=255).map(Some).chain([None]) {
            for count in [0usize, 1, 3, 7, 40, 10_000] {
                assert!(expected_interval_days(mark, count) >= 1, "{mark:?}/{count}");
            }
        }
    }

    #[test]
    fn the_interval_rises_monotonically_with_the_mark() {
        let intervals: Vec<u32> = (0..=10)
            .map(|m| expected_interval_days(Some(m), 4))
            .collect();
        assert!(
            intervals.windows(2).all(|w| w[1] >= w[0]),
            "a better mark must never shorten the interval: {intervals:?}"
        );
    }

    /// The defect the binary flag had: one clock for every item.
    #[test]
    fn a_fragile_item_and_a_consolidated_one_do_not_go_cold_together() {
        let gap = Some(25);
        assert_eq!(grade(gap, Some(3), 1).band, Cold);
        assert_eq!(grade(gap, Some(9), 12).band, Fresh);
    }

    // ── Bands ────────────────────────────────────────────────────────

    #[test]
    fn never_practised_is_unknown_not_cold() {
        assert_eq!(grade(None, Some(5), 0).band, Unknown);
    }

    #[test]
    fn the_bands_run_fresh_then_going_cold_then_cold() {
        let interval = expected_interval_days(Some(5), 3);
        assert_eq!(grade(Some(interval), Some(5), 3).band, Fresh);
        assert_eq!(grade(Some(interval + 1), Some(5), 3).band, GoingCold);
        assert_eq!(grade(Some(interval * 2), Some(5), 3).band, GoingCold);
        assert_eq!(grade(Some(interval * 2 + 1), Some(5), 3).band, Cold);
    }

    #[test]
    fn cold_only_arrives_at_double_the_interval_so_the_signal_is_lax() {
        let s = grade(Some(20), Some(5), 3);
        assert_eq!(s.expected_interval_days, 11);
        assert_eq!(
            s.band, GoingCold,
            "20 days on an 11-day interval is not cold"
        );
    }

    #[test]
    fn only_the_fresh_band_has_nothing_to_report() {
        assert!(grade(Some(1), None, 0).is_fresh());
        assert!(!grade(Some(15), None, 0).is_fresh());
        assert!(!grade(Some(90), None, 0).is_fresh());
        assert!(
            !grade(None, None, 0).is_fresh(),
            "an item never practised has not been kept warm either"
        );
    }

    // ── Ranking ──────────────────────────────────────────────────────

    #[test]
    fn the_more_overdue_item_ranks_staler_even_on_a_shorter_gap() {
        let fragile = grade(Some(12), Some(2), 1);
        let consolidated = grade(Some(30), Some(10), 15);
        assert!(
            fragile.overdue_key() > consolidated.overdue_key(),
            "12 days on a 5-day interval beats 30 on a 39-day one"
        );
    }

    #[test]
    fn never_practised_outranks_everything_practised() {
        let never = grade(None, None, 0);
        assert!(never.overdue_key() > grade(Some(9_999), Some(0), 0).overdue_key());
    }

    #[test]
    fn a_gap_no_interval_could_survive_does_not_overflow() {
        assert_eq!(grade(Some(u32::MAX), Some(0), 0).band, Cold);
        assert!(grade(Some(u32::MAX), Some(0), 0).overdue_key() > 0);
    }

    // ── Reading the model's own data ─────────────────────────────────

    #[test]
    fn assess_reads_the_gap_and_the_returns_out_of_the_summary() {
        let s = assess(Some(&practised(30, 12)), Some(9), clock());
        assert_eq!(s.days, Some(30));
        assert_eq!(
            s.expected_interval_days,
            expected_interval_days(Some(9), 12)
        );
    }

    #[test]
    fn the_shown_mark_drives_the_interval_not_the_summarys_flat_one() {
        let summary = ItemPracticeSummary {
            latest_score: Some(9),
            ..practised(12, 3)
        };
        // A step mid-climb is rough even though the exercise's flat mark is not.
        assert_eq!(assess(Some(&summary), Some(1), clock()).band, GoingCold);
        assert_eq!(assess(Some(&summary), Some(9), clock()).band, Fresh);
    }

    #[test]
    fn an_unreadable_date_is_no_evidence_rather_than_evidence_of_neglect() {
        let summary = ItemPracticeSummary {
            last_practiced_at: Some("not a date".to_string()),
            ..ItemPracticeSummary::fixture()
        };
        assert_eq!(assess(Some(&summary), None, clock()).band, Unknown);
    }

    #[test]
    fn a_date_ahead_of_the_clock_reads_as_today() {
        // Reachable through the shell's reported UTC offset, not only a broken
        // clock, so it must not wrap to the stalest item alive.
        assert_eq!(assess(Some(&practised(-3, 2)), None, clock()).days, Some(0));
    }

    #[test]
    fn no_practice_summary_at_all_is_unknown() {
        assert_eq!(assess(None, Some(7), clock()).band, Unknown);
    }

    // ── Copy ─────────────────────────────────────────────────────────

    #[test]
    fn the_clause_names_the_grade_and_the_gap() {
        assert_eq!(grade(None, None, 0).clause(), "not practised yet");
        assert_eq!(grade(Some(0), None, 3).clause(), "practised today");
        assert_eq!(grade(Some(1), None, 3).clause(), "practised yesterday");
        assert_eq!(grade(Some(5), None, 3).clause(), "practised 5 days ago");
        assert_eq!(
            grade(Some(21), None, 3).clause(),
            "going cold after 3 weeks"
        );
        assert_eq!(grade(Some(97), None, 3).clause(), "cold for 3 months");
    }

    #[test]
    fn the_gap_is_said_in_the_unit_a_musician_would_use() {
        assert_eq!(gap_phrase(13), "13 days");
        assert_eq!(gap_phrase(14), "2 weeks");
        assert_eq!(gap_phrase(18), "3 weeks");
        assert_eq!(gap_phrase(55), "8 weeks");
        assert_eq!(gap_phrase(56), "2 months");
        assert_eq!(gap_phrase(364), "12 months");
        assert_eq!(gap_phrase(365), "over a year");
    }

    /// Every gap a real library can hand the formatter, not sentences picked to
    /// agree with the code (CLAUDE.md, Testing).
    #[test]
    fn every_clause_is_one_line_of_house_style() {
        let marks = [None, Some(0), Some(4), Some(8), Some(10)];
        let counts = [0usize, 1, 3, 7, 40];
        let gaps = [None]
            .into_iter()
            .chain((0..400).map(Some))
            .collect::<Vec<_>>();

        for mark in marks {
            for count in counts {
                for gap in &gaps {
                    let clause = grade(*gap, mark, count).clause();
                    assert!(!clause.is_empty(), "empty clause for {gap:?}");
                    assert!(!clause.contains('\n'), "multi-line: {clause}");
                    assert!(
                        !clause.contains('—') && !clause.contains("--"),
                        "dash in prose: {clause}"
                    );
                    assert!(!clause.ends_with('.'), "full stop on a label: {clause}");
                    assert!(
                        !clause.to_lowercase().contains("you"),
                        "second person: {clause}"
                    );
                    assert!(!clause.contains("practiced"), "American spelling: {clause}");
                    assert!(
                        clause.chars().next().is_some_and(|c| c.is_lowercase()),
                        "the clause is mid-sentence; the caller capitalises: {clause}"
                    );
                    assert!(
                        clause.split_whitespace().count() <= 5,
                        "over the clause budget: {clause}"
                    );
                    assert!(
                        !clause.contains(" 1 days") && !clause.contains(" 1 weeks"),
                        "plural on a single unit: {clause}"
                    );
                }
            }
        }
    }
}

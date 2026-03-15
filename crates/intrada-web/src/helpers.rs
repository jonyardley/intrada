use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Weekday};
use intrada_core::{LibraryItemView, PracticeSessionView, Tempo};

/// Format an ISO 8601 / RFC 3339 date string to "d Mon YYYY" (e.g. "4 Mar 2026").
pub fn format_date_short(iso: &str) -> String {
    DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%-d %b %Y").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

/// Format an ISO 8601 / RFC 3339 datetime string to "d Mon YYYY, HH:MM" (e.g. "4 Mar 2026, 14:30").
pub fn format_datetime_short(iso: &str) -> String {
    DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%-d %b %Y, %H:%M").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

/// Parse comma-separated tags string into Vec<String>.
/// Trims whitespace, filters empty entries.
pub fn parse_tags(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse tempo marking + BPM string into Option<Tempo>.
/// Returns None if both are empty.
pub fn parse_tempo(marking: &str, bpm_str: &str) -> Option<Tempo> {
    let marking_opt = {
        let m = marking.trim();
        if m.is_empty() {
            None
        } else {
            Some(m.to_string())
        }
    };
    let bpm_opt = bpm_str.trim().parse::<u16>().ok();
    Tempo::from_parts(marking_opt, bpm_opt)
}

/// Parse a formatted tempo display string back into (marking, bpm_str) for edit form pre-population.
/// Handles: "Allegro (132 BPM)", "Allegro", "132 BPM", None
pub fn parse_tempo_display(tempo: &Option<String>) -> (String, String) {
    let Some(t) = tempo else {
        return (String::new(), String::new());
    };

    // Pattern: "Marking (BPM_NUMBER BPM)"
    if let Some(paren_start) = t.rfind('(') {
        let marking = t[..paren_start].trim().to_string();
        let bpm_part = &t[paren_start + 1..];
        let bpm_str = bpm_part
            .trim_end_matches(')')
            .trim()
            .trim_end_matches("BPM")
            .trim()
            .to_string();
        return (marking, bpm_str);
    }

    // Pattern: "NUMBER BPM"
    if t.ends_with("BPM") {
        let bpm_str = t.trim_end_matches("BPM").trim().to_string();
        return (String::new(), bpm_str);
    }

    // Just a marking
    (t.clone(), String::new())
}

/// Collect all unique tags from library items, deduplicated case-insensitively
/// (preserving first-seen casing), sorted alphabetically.
pub fn unique_tags(items: &[LibraryItemView]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        for tag in &item.tags {
            let lower = tag.to_lowercase();
            if seen.insert(lower) {
                result.push(tag.clone());
            }
        }
    }
    result.sort_by_key(|a| a.to_lowercase());
    result
}

/// Extract unique composers from library items.
/// Subtitle is always the composer for all item types.
/// Deduplicates case-insensitively (preserving first-seen casing),
/// filters out empty strings, and sorts alphabetically.
pub fn unique_composers(items: &[LibraryItemView]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let composer = &item.subtitle;
        if composer.is_empty() {
            continue;
        }
        let lower = composer.to_lowercase();
        if seen.insert(lower) {
            result.push(composer.clone());
        }
    }
    result.sort_by_key(|a| a.to_lowercase());
    result
}

/// Filter suggestions by input text. Case-insensitive matching with prefix
/// matches ranked before substring matches. Excludes values in `exclude` list.
/// Returns up to `max` results.
pub fn filter_suggestions(
    suggestions: &[String],
    input: &str,
    exclude: &[String],
    max: usize,
) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let exclude_lower: HashSet<String> = exclude.iter().map(|s| s.to_lowercase()).collect();

    let mut prefix_matches = Vec::new();
    let mut substring_matches = Vec::new();

    for suggestion in suggestions {
        let lower = suggestion.to_lowercase();
        if exclude_lower.contains(&lower) {
            continue;
        }
        if lower.starts_with(&input_lower) {
            prefix_matches.push(suggestion.clone());
        } else if lower.contains(&input_lower) {
            substring_matches.push(suggestion.clone());
        }
    }

    let mut result = prefix_matches;
    result.extend(substring_matches);
    result.truncate(max);
    result
}

// ═══════════════════════════════════════════════════════════════════════
// Week Strip Helpers
// ═══════════════════════════════════════════════════════════════════════

/// Get the Monday (start) of the week at `offset` weeks from the week containing `today`.
/// Offset 0 = current week, -1 = previous week, +1 = next week.
pub fn get_week_start(today: NaiveDate, offset: i32) -> NaiveDate {
    let iso_week = today.iso_week();
    let current_monday = NaiveDate::from_isoywd_opt(iso_week.year(), iso_week.week(), Weekday::Mon)
        .expect("valid ISO week start");
    current_monday + Duration::days(i64::from(offset) * 7)
}

/// Get the 7 dates (Mon–Sun) for a week starting on `week_start`.
pub fn get_week_dates(week_start: NaiveDate) -> [NaiveDate; 7] {
    [
        week_start,
        week_start + Duration::days(1),
        week_start + Duration::days(2),
        week_start + Duration::days(3),
        week_start + Duration::days(4),
        week_start + Duration::days(5),
        week_start + Duration::days(6),
    ]
}

/// Generate a month/year label for the given week.
/// If the week spans a single month: "March 2026".
/// If the week spans two months: "Feb – Mar 2026" (or "Dec 2025 – Jan 2026" across years).
pub fn get_month_label(week_start: NaiveDate, week_end: NaiveDate) -> String {
    let start_month = week_start.month();
    let start_year = week_start.year();
    let end_month = week_end.month();
    let end_year = week_end.year();

    if start_month == end_month && start_year == end_year {
        week_start.format("%B %Y").to_string()
    } else if start_year == end_year {
        format!(
            "{} – {} {}",
            week_start.format("%b"),
            week_end.format("%b"),
            start_year
        )
    } else {
        format!(
            "{} {} – {} {}",
            week_start.format("%b"),
            start_year,
            week_end.format("%b"),
            end_year
        )
    }
}

/// Return the single-letter abbreviation for a weekday.
pub fn day_abbrev(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "M",
        Weekday::Tue => "T",
        Weekday::Wed => "W",
        Weekday::Thu => "T",
        Weekday::Fri => "F",
        Weekday::Sat => "S",
        Weekday::Sun => "S",
    }
}

/// Group sessions by their `started_at` date (parsed from RFC3339).
/// Within each date bucket, sessions are sorted chronologically (earliest first).
pub fn group_sessions_by_date(
    sessions: &[PracticeSessionView],
) -> HashMap<NaiveDate, Vec<PracticeSessionView>> {
    let mut map: HashMap<NaiveDate, Vec<PracticeSessionView>> = HashMap::new();
    for session in sessions {
        if let Ok(dt) = DateTime::parse_from_rfc3339(&session.started_at) {
            let date = dt.date_naive();
            map.entry(date).or_default().push(session.clone());
        }
    }
    // Sort each bucket chronologically (earliest first)
    for sessions in map.values_mut() {
        sessions.sort_by(|a, b| a.started_at.cmp(&b.started_at));
    }
    map
}

/// Return the set of dates within the given week (Mon–Sun) that have sessions.
pub fn sessions_for_week(
    grouped: &HashMap<NaiveDate, Vec<PracticeSessionView>>,
    week_start: NaiveDate,
) -> HashSet<NaiveDate> {
    let dates = get_week_dates(week_start);
    dates
        .iter()
        .filter(|d| grouped.contains_key(d))
        .copied()
        .collect()
}

/// Auto-select logic: pick the best day to highlight in the week strip.
///
/// 1. If `today` is in the displayed week and has sessions → today
/// 2. Else if any day in the week has sessions → latest calendar day with sessions
/// 3. Else if `today` is in the week → today
/// 4. Else → Monday of the week
pub fn auto_select_day(
    week_start: NaiveDate,
    today: NaiveDate,
    session_dates: &HashSet<NaiveDate>,
) -> NaiveDate {
    let week_end = week_start + Duration::days(6);
    let today_in_week = today >= week_start && today <= week_end;

    // 1. Today in week and has sessions
    if today_in_week && session_dates.contains(&today) {
        return today;
    }

    // 2. Most recent day in week with sessions
    if !session_dates.is_empty() {
        let dates = get_week_dates(week_start);
        // Find the most recent day with sessions (iterate from Sunday backward)
        for date in dates.iter().rev() {
            if session_dates.contains(date) {
                return *date;
            }
        }
    }

    // 3. Today if in week
    if today_in_week {
        return today;
    }

    // 4. Monday
    week_start
}

/// Format the start time from an RFC3339 string as "HH:MM" for display.
pub fn format_time_short(iso: &str) -> String {
    DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // T005: parse_tags tests
    #[test]
    fn test_parse_tags_empty_string() {
        assert!(parse_tags("").is_empty());
    }

    #[test]
    fn test_parse_tags_single() {
        assert_eq!(parse_tags("classical"), vec!["classical"]);
    }

    #[test]
    fn test_parse_tags_multiple() {
        assert_eq!(
            parse_tags("classical, piano, romantic"),
            vec!["classical", "piano", "romantic"]
        );
    }

    #[test]
    fn test_parse_tags_whitespace_trimming() {
        assert_eq!(
            parse_tags("  jazz ,  blues , funk  "),
            vec!["jazz", "blues", "funk"]
        );
    }

    #[test]
    fn test_parse_tags_trailing_comma() {
        assert_eq!(parse_tags("rock,"), vec!["rock"]);
    }

    #[test]
    fn test_parse_tags_empty_entries_filtered() {
        assert_eq!(parse_tags("a,,b,,,c"), vec!["a", "b", "c"]);
    }

    // T006: parse_tempo tests
    #[test]
    fn test_parse_tempo_both_empty() {
        assert!(parse_tempo("", "").is_none());
    }

    #[test]
    fn test_parse_tempo_marking_only() {
        let tempo = parse_tempo("Allegro", "").unwrap();
        assert_eq!(tempo.marking, Some("Allegro".to_string()));
        assert_eq!(tempo.bpm, None);
    }

    #[test]
    fn test_parse_tempo_bpm_only() {
        let tempo = parse_tempo("", "120").unwrap();
        assert_eq!(tempo.marking, None);
        assert_eq!(tempo.bpm, Some(120));
    }

    #[test]
    fn test_parse_tempo_both_present() {
        let tempo = parse_tempo("Allegro", "132").unwrap();
        assert_eq!(tempo.marking, Some("Allegro".to_string()));
        assert_eq!(tempo.bpm, Some(132));
    }

    #[test]
    fn test_parse_tempo_invalid_bpm() {
        // Invalid BPM string is treated as no BPM
        let tempo = parse_tempo("Andante", "fast");
        // "Andante" alone should still produce Some with marking
        assert!(tempo.is_some());
        let t = tempo.unwrap();
        assert_eq!(t.marking, Some("Andante".to_string()));
        assert_eq!(t.bpm, None);
    }

    // T007: parse_tempo_display tests
    #[test]
    fn test_parse_tempo_display_none() {
        let (marking, bpm) = parse_tempo_display(&None);
        assert!(marking.is_empty());
        assert!(bpm.is_empty());
    }

    #[test]
    fn test_parse_tempo_display_marking_only() {
        let (marking, bpm) = parse_tempo_display(&Some("Allegro".to_string()));
        assert_eq!(marking, "Allegro");
        assert!(bpm.is_empty());
    }

    #[test]
    fn test_parse_tempo_display_bpm_only() {
        let (marking, bpm) = parse_tempo_display(&Some("132 BPM".to_string()));
        assert!(marking.is_empty());
        assert_eq!(bpm, "132");
    }

    #[test]
    fn test_parse_tempo_display_full_format() {
        let (marking, bpm) = parse_tempo_display(&Some("Allegro (132 BPM)".to_string()));
        assert_eq!(marking, "Allegro");
        assert_eq!(bpm, "132");
    }

    // unique_tags tests
    fn make_item(_item_type: &str, subtitle: &str, tags: &[&str]) -> LibraryItemView {
        LibraryItemView {
            id: String::new(),
            item_type: intrada_core::ItemKind::Piece,
            title: String::new(),
            subtitle: subtitle.to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: String::new(),
            updated_at: String::new(),
            practice: None,
            latest_achieved_tempo: None,
        }
    }

    #[test]
    fn test_unique_tags_empty() {
        assert!(unique_tags(&[]).is_empty());
    }

    #[test]
    fn test_unique_tags_dedup_case_insensitive() {
        let items = vec![
            make_item("piece", "", &["Classical", "jazz"]),
            make_item("piece", "", &["classical", "Jazz"]),
        ];
        let tags = unique_tags(&items);
        assert_eq!(tags, vec!["Classical", "jazz"]);
    }

    #[test]
    fn test_unique_tags_sorted_alphabetically() {
        let items = vec![
            make_item("piece", "", &["piano", "baroque"]),
            make_item("piece", "", &["classical"]),
        ];
        let tags = unique_tags(&items);
        assert_eq!(tags, vec!["baroque", "classical", "piano"]);
    }

    // unique_composers tests
    #[test]
    fn test_unique_composers_empty() {
        assert!(unique_composers(&[]).is_empty());
    }

    #[test]
    fn test_unique_composers_from_pieces() {
        let items = vec![
            make_item("piece", "Bach", &[]),
            make_item("piece", "Mozart", &[]),
            make_item("piece", "Bach", &[]), // duplicate
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Bach", "Mozart"]);
    }

    #[test]
    fn test_unique_composers_empty_subtitle_excluded() {
        let items = vec![
            make_item("piece", "", &[]),
            make_item("piece", "Mozart", &[]),
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Mozart"]);
    }

    #[test]
    fn test_unique_composers_case_insensitive_dedup() {
        let items = vec![
            make_item("piece", "Bach", &[]),
            make_item("piece", "bach", &[]),
        ];
        let composers = unique_composers(&items);
        assert_eq!(composers, vec!["Bach"]);
    }

    // filter_suggestions tests
    #[test]
    fn test_filter_suggestions_empty_input() {
        let suggestions = vec!["Classical".to_string()];
        let result = filter_suggestions(&suggestions, "", &[], 8);
        // Empty input matches everything as prefix
        assert_eq!(result, vec!["Classical"]);
    }

    #[test]
    fn test_filter_suggestions_prefix_before_substring() {
        let suggestions = vec![
            "baroque".to_string(),
            "classical".to_string(),
            "neoclassical".to_string(),
        ];
        let result = filter_suggestions(&suggestions, "class", &[], 8);
        // "classical" is prefix match, "neoclassical" is substring
        assert_eq!(result, vec!["classical", "neoclassical"]);
    }

    #[test]
    fn test_filter_suggestions_case_insensitive() {
        let suggestions = vec!["Classical".to_string(), "Jazz".to_string()];
        let result = filter_suggestions(&suggestions, "cla", &[], 8);
        assert_eq!(result, vec!["Classical"]);
    }

    #[test]
    fn test_filter_suggestions_excludes() {
        let suggestions = vec![
            "classical".to_string(),
            "jazz".to_string(),
            "piano".to_string(),
        ];
        let exclude = vec!["jazz".to_string()];
        let result = filter_suggestions(&suggestions, "", &exclude, 8);
        assert_eq!(result, vec!["classical", "piano"]);
    }

    #[test]
    fn test_filter_suggestions_max_limit() {
        let suggestions: Vec<String> = (0..20).map(|i| format!("tag{i}")).collect();
        let result = filter_suggestions(&suggestions, "tag", &[], 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filter_suggestions_exclude_case_insensitive() {
        let suggestions = vec!["Classical".to_string(), "Jazz".to_string()];
        let exclude = vec!["CLASSICAL".to_string()];
        let result = filter_suggestions(&suggestions, "", &exclude, 8);
        assert_eq!(result, vec!["Jazz"]);
    }

    // format_date_short / format_datetime_short tests
    #[test]
    fn test_format_date_short_valid() {
        assert_eq!(format_date_short("2026-03-04T14:30:00+00:00"), "4 Mar 2026");
    }

    #[test]
    fn test_format_date_short_fallback() {
        assert_eq!(format_date_short("not-a-date"), "not-a-date");
    }

    #[test]
    fn test_format_datetime_short_valid() {
        assert_eq!(
            format_datetime_short("2026-03-04T14:30:00+00:00"),
            "4 Mar 2026, 14:30"
        );
    }

    #[test]
    fn test_format_datetime_short_fallback() {
        assert_eq!(format_datetime_short("nope"), "nope");
    }

    // ═══════════════════════════════════════════════════════════════════
    // T003: Week calculation helper tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_get_week_start_current_week() {
        // 2026-03-04 is a Wednesday
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let monday = get_week_start(today, 0);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2026, 3, 2).unwrap());
    }

    #[test]
    fn test_get_week_start_previous_week() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let monday = get_week_start(today, -1);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2026, 2, 23).unwrap());
    }

    #[test]
    fn test_get_week_start_next_week() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let monday = get_week_start(today, 1);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2026, 3, 9).unwrap());
    }

    #[test]
    fn test_get_week_start_on_monday() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(); // Monday
        let monday = get_week_start(today, 0);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2026, 3, 2).unwrap());
    }

    #[test]
    fn test_get_week_start_on_sunday() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap(); // Sunday
        let monday = get_week_start(today, 0);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2026, 3, 2).unwrap());
    }

    #[test]
    fn test_get_week_dates() {
        let monday = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let dates = get_week_dates(monday);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2026, 3, 2).unwrap()); // Mon
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2026, 3, 3).unwrap()); // Tue
        assert_eq!(dates[6], NaiveDate::from_ymd_opt(2026, 3, 8).unwrap()); // Sun
    }

    #[test]
    fn test_get_month_label_single_month() {
        let start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
        assert_eq!(get_month_label(start, end), "March 2026");
    }

    #[test]
    fn test_get_month_label_two_months() {
        let start = NaiveDate::from_ymd_opt(2026, 2, 23).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        assert_eq!(get_month_label(start, end), "Feb – Mar 2026");
    }

    #[test]
    fn test_get_month_label_year_boundary() {
        let start = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
        assert_eq!(get_month_label(start, end), "Dec 2025 – Jan 2026");
    }

    #[test]
    fn test_day_abbrev_all_days() {
        assert_eq!(day_abbrev(Weekday::Mon), "M");
        assert_eq!(day_abbrev(Weekday::Tue), "T");
        assert_eq!(day_abbrev(Weekday::Wed), "W");
        assert_eq!(day_abbrev(Weekday::Thu), "T");
        assert_eq!(day_abbrev(Weekday::Fri), "F");
        assert_eq!(day_abbrev(Weekday::Sat), "S");
        assert_eq!(day_abbrev(Weekday::Sun), "S");
    }

    // ═══════════════════════════════════════════════════════════════════
    // T004: Session grouping helper tests
    // ═══════════════════════════════════════════════════════════════════

    fn make_session(started_at: &str) -> PracticeSessionView {
        PracticeSessionView {
            id: started_at.to_string(),
            started_at: started_at.to_string(),
            finished_at: started_at.to_string(),
            total_duration_display: "25 min".to_string(),
            completion_status: intrada_core::CompletionStatus::Completed,
            notes: None,
            entries: vec![],
            session_intention: None,
        }
    }

    #[test]
    fn test_group_sessions_by_date_empty() {
        let grouped = group_sessions_by_date(&[]);
        assert!(grouped.is_empty());
    }

    #[test]
    fn test_group_sessions_by_date_groups_correctly() {
        let sessions = vec![
            make_session("2026-03-04T09:00:00+00:00"),
            make_session("2026-03-04T14:00:00+00:00"),
            make_session("2026-03-05T10:00:00+00:00"),
        ];
        let grouped = group_sessions_by_date(&sessions);
        assert_eq!(grouped.len(), 2);
        let mar4 = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let mar5 = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        assert_eq!(grouped[&mar4].len(), 2);
        assert_eq!(grouped[&mar5].len(), 1);
    }

    #[test]
    fn test_group_sessions_by_date_chronological_sort() {
        let sessions = vec![
            make_session("2026-03-04T14:00:00+00:00"), // later
            make_session("2026-03-04T09:00:00+00:00"), // earlier
        ];
        let grouped = group_sessions_by_date(&sessions);
        let mar4 = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let day_sessions = &grouped[&mar4];
        assert!(day_sessions[0].started_at < day_sessions[1].started_at);
    }

    #[test]
    fn test_group_sessions_by_date_skips_invalid_timestamp() {
        let sessions = vec![
            make_session("2026-03-04T09:00:00+00:00"),
            make_session("not-a-date"),
            make_session("2026-03-04T14:00:00+00:00"),
        ];
        let grouped = group_sessions_by_date(&sessions);
        let mar4 = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        // The invalid timestamp is silently skipped
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[&mar4].len(), 2);
    }

    #[test]
    fn test_sessions_for_week() {
        let sessions = vec![
            make_session("2026-03-04T09:00:00+00:00"), // Wed Mar 4
            make_session("2026-03-06T10:00:00+00:00"), // Fri Mar 6
            make_session("2026-03-10T10:00:00+00:00"), // Next week Tue
        ];
        let grouped = group_sessions_by_date(&sessions);
        let week_start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(); // Mon Mar 2
        let week_sessions = sessions_for_week(&grouped, week_start);
        assert_eq!(week_sessions.len(), 2);
        assert!(week_sessions.contains(&NaiveDate::from_ymd_opt(2026, 3, 4).unwrap()));
        assert!(week_sessions.contains(&NaiveDate::from_ymd_opt(2026, 3, 6).unwrap()));
    }

    #[test]
    fn test_sessions_for_week_empty() {
        let grouped = group_sessions_by_date(&[]);
        let week_start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let week_sessions = sessions_for_week(&grouped, week_start);
        assert!(week_sessions.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════
    // Auto-select tests (T018)
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_auto_select_today_with_sessions() {
        let week_start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(); // Wed
        let mut session_dates = HashSet::new();
        session_dates.insert(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap());
        session_dates.insert(NaiveDate::from_ymd_opt(2026, 3, 3).unwrap());
        assert_eq!(auto_select_day(week_start, today, &session_dates), today);
    }

    #[test]
    fn test_auto_select_most_recent_session_day() {
        let week_start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(); // Wed, no sessions
        let mut session_dates = HashSet::new();
        session_dates.insert(NaiveDate::from_ymd_opt(2026, 3, 3).unwrap()); // Tue
        assert_eq!(
            auto_select_day(week_start, today, &session_dates),
            NaiveDate::from_ymd_opt(2026, 3, 3).unwrap()
        );
    }

    #[test]
    fn test_auto_select_no_sessions_today_in_week() {
        let week_start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let session_dates = HashSet::new();
        assert_eq!(auto_select_day(week_start, today, &session_dates), today);
    }

    #[test]
    fn test_auto_select_today_not_in_week() {
        let week_start = NaiveDate::from_ymd_opt(2026, 2, 23).unwrap(); // Previous week
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let session_dates = HashSet::new();
        assert_eq!(
            auto_select_day(week_start, today, &session_dates),
            week_start
        );
    }

    #[test]
    fn test_auto_select_picks_most_recent_not_first() {
        let week_start = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap(); // Sat, no sessions
        let mut session_dates = HashSet::new();
        session_dates.insert(NaiveDate::from_ymd_opt(2026, 3, 3).unwrap()); // Tue
        session_dates.insert(NaiveDate::from_ymd_opt(2026, 3, 5).unwrap()); // Thu
        assert_eq!(
            auto_select_day(week_start, today, &session_dates),
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap() // Most recent: Thu
        );
    }

    #[test]
    fn test_format_time_short() {
        assert_eq!(format_time_short("2026-03-04T14:30:00+00:00"), "14:30");
        assert_eq!(format_time_short("2026-03-04T09:05:00+00:00"), "09:05");
    }
}

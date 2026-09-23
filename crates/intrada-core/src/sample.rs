use crate::domain::item::{Item, ItemKind};
use crate::domain::session::PracticeSession;

/// Canonical demo dataset for `Event::LoadSampleData` — shared by every shell
/// (CI screenshots, local demos, E2E). Stable ids; staggered timestamps so the
/// newest-first sort is deterministic.
pub(crate) fn sample_items() -> Vec<Item> {
    use crate::domain::types::Tempo;
    let now = chrono::Utc::now();

    #[allow(clippy::too_many_arguments)]
    let item = |minutes_ago: i64,
                id: &str,
                title: &str,
                kind: ItemKind,
                composer: Option<&str>,
                key: Option<&str>,
                marking: Option<&str>,
                bpm: Option<u16>,
                notes: Option<&str>,
                tags: &[&str]|
     -> Item {
        let ts = now - chrono::Duration::minutes(minutes_ago);
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind,
            composer: composer.map(str::to_string),
            key: key.map(str::to_string),
            modality: None,
            tempo: Tempo::from_parts(marking.map(str::to_string), bpm),
            notes: notes.map(str::to_string),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: ts,
            updated_at: ts,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        }
    };

    let mut items = vec![
        item(
            0,
            "sample-clair",
            "Clair de Lune",
            ItemKind::Piece,
            Some("Claude Debussy"),
            Some("D♭ major"),
            Some("Andante"),
            Some(72),
            Some("Focus on the rubato in the opening phrase; keep the left hand soft."),
            &["recital", "impressionist"],
        ),
        item(
            1,
            "sample-gymnopedie",
            "Gymnopédie No. 1",
            ItemKind::Piece,
            Some("Erik Satie"),
            Some("D major"),
            Some("Lent"),
            Some(70),
            None,
            &["recital"],
        ),
        item(
            2,
            "sample-nocturne",
            "Nocturne Op. 9 No. 2",
            ItemKind::Piece,
            Some("Frédéric Chopin"),
            Some("E♭ major"),
            Some("Andante"),
            Some(68),
            None,
            &[],
        ),
        item(
            3,
            "sample-hanon",
            "Hanon No. 1",
            ItemKind::Exercise,
            Some("Charles-Louis Hanon"),
            Some("C major"),
            None,
            Some(108),
            Some("Even tone, relaxed wrist."),
            &["warm-up"],
        ),
        item(
            4,
            "sample-scales",
            "Major Scales",
            ItemKind::Exercise,
            None,
            None,
            None,
            Some(120),
            None,
            &["technique"],
        ),
    ];

    // Demo variation ladder (#1083): Major Scales climbs a starter run of keys, so
    // seed mode shows per-variation progress (sample_sessions scores the first two).
    if let Some(scales) = items.iter_mut().find(|i| i.id == "sample-scales") {
        scales.variants = ["C", "G", "D", "A", "E"]
            .iter()
            .enumerate()
            .map(|(position, label)| crate::domain::variant::Variant {
                id: format!("sample-scales-step-{}", label.to_lowercase()),
                label: (*label).to_string(),
                position,
                updated_at: scales.updated_at,
                deleted_at: None,
            })
            .collect();
    }
    // One related exercise, so seed mode has a block worth resuming and the Up
    // next card (#1082) has something to show. Anchored on the Nocturne, not
    // the first two items: `SessionBuilderUITests` needs Clair de Lune to have
    // no related exercises (it drives the empty-state CTA) and adds the top two
    // library cards expecting two standalone rows.
    if let Some(nocturne) = items.iter_mut().find(|i| i.id == "sample-nocturne") {
        nocturne.linked_exercise_ids = vec!["sample-scales".to_string()];
    }
    items
}

/// Canonical demo practice history for `Event::LoadSampleData`. Entries
/// reference the ids minted by `sample_items()` so the home screen's
/// "duration · item count" line and any future detail view stay consistent.
pub(crate) fn sample_sessions() -> Vec<PracticeSession> {
    use crate::domain::session::{CompletionStatus, EntryStatus, SetlistEntry};
    let now = chrono::Utc::now();

    fn sample_play(
        id: &str,
        variation_id: Option<&str>,
        seconds: u64,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> crate::domain::session::VariationPlay {
        crate::domain::session::VariationPlay {
            id: id.to_string(),
            variation_id: variation_id.map(str::to_string),
            started_at,
            seconds,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            achieved_tempo: None,
            click_pattern: None,
            score: None,
        }
    }

    let entry = |position: usize,
                 item_id: &str,
                 item_title: &str,
                 item_type: ItemKind,
                 duration_secs: u64|
     -> SetlistEntry {
        SetlistEntry {
            id: format!("{item_id}-entry-{position}"),
            item_id: item_id.to_string(),
            item_title: item_title.to_string(),
            item_type,
            position,
            duration_secs,
            status: EntryStatus::Completed,
            notes: None,
            intention: None,
            planned_duration_secs: None,
            group_id: None,
            planned_variation_id: None,
            planned_rep_target: None,
            plays: vec![sample_play(
                &format!("{item_id}-play-{position}"),
                None,
                duration_secs,
                now,
            )],
        }
    };

    let session = |id: &str,
                   days_ago: i64,
                   completion_status: CompletionStatus,
                   entries: Vec<SetlistEntry>|
     -> PracticeSession {
        let total_duration_secs = entries.iter().map(|e| e.duration_secs).sum();
        let started_at = now - chrono::Duration::days(days_ago);
        PracticeSession {
            id: id.to_string(),
            entries,
            session_notes: None,
            started_at,
            completed_at: started_at + chrono::Duration::seconds(total_duration_secs as i64),
            total_duration_secs,
            completion_status,
            session_score: None,
        }
    };

    vec![
        session(
            "sample-session-today",
            0,
            CompletionStatus::Completed,
            vec![
                entry(0, "sample-clair", "Clair de Lune", ItemKind::Piece, 720),
                entry(
                    1,
                    "sample-gymnopedie",
                    "Gymnopédie No. 1",
                    ItemKind::Piece,
                    540,
                ),
                entry(
                    2,
                    "sample-nocturne",
                    "Nocturne Op. 9 No. 2",
                    ItemKind::Piece,
                    540,
                ),
            ],
        ),
        session(
            "sample-session-yesterday",
            1,
            CompletionStatus::Completed,
            vec![
                {
                    let mut e = entry(0, "sample-hanon", "Hanon No. 1", ItemKind::Exercise, 480);
                    e.plays[0].achieved_tempo = Some(104);
                    e
                },
                {
                    let mut e = entry(1, "sample-scales", "Major Scales", ItemKind::Exercise, 600);
                    e.plays[0].variation_id = Some("sample-scales-step-g".to_string());
                    e.plays[0].score = Some(7);
                    e
                },
            ],
        ),
        session(
            "sample-session-3d",
            3,
            CompletionStatus::EndedEarly,
            vec![
                {
                    let mut e = entry(0, "sample-clair", "Clair de Lune", ItemKind::Piece, 1500);
                    e.plays[0].achieved_tempo = Some(66);
                    e
                },
                entry(1, "sample-hanon", "Hanon No. 1", ItemKind::Exercise, 600),
                entry(
                    2,
                    "sample-nocturne",
                    "Nocturne Op. 9 No. 2",
                    ItemKind::Piece,
                    600,
                ),
            ],
        ),
        session(
            "sample-session-5d",
            5,
            CompletionStatus::Completed,
            vec![
                {
                    // Two variations in one sitting, which is the case #1739
                    // exists for: the seed data has to show it.
                    let mut e = entry(0, "sample-scales", "Major Scales", ItemKind::Exercise, 720);
                    e.plays[0].variation_id = Some("sample-scales-step-c".to_string());
                    e.plays[0].seconds = 420;
                    e.plays[0].score = Some(8);
                    let mut second = sample_play(
                        "sample-scales-play-0b",
                        Some("sample-scales-step-g"),
                        300,
                        now,
                    );
                    second.score = Some(6);
                    e.plays.push(second);
                    e
                },
                {
                    let mut e = entry(1, "sample-hanon", "Hanon No. 1", ItemKind::Exercise, 420);
                    e.plays[0].achieved_tempo = Some(88);
                    e
                },
            ],
        ),
    ]
}

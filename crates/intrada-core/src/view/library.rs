use crate::domain::item::ItemKind;
use crate::domain::session::PracticeSession;
use crate::domain::types::{LibrarySort, ListQuery, SortDirection, SortField};
use crate::model::{
    ItemPracticeSummary, LibraryItemView, LinkedExerciseView, Model, ScaffoldPreviewView,
    ScaffoldSpecView,
};

/// Every library item projected for the view, unfiltered and unsorted. Shared
/// by `build_view` and the Up next derivation, so the card reads the same marks
/// the Library screens show rather than deriving its own (#1082).
pub(super) fn build_library_item_views(
    model: &Model,
    item_index: &std::collections::HashMap<&str, &crate::domain::item::Item>,
) -> Vec<LibraryItemView> {
    let usage_by_exercise = build_exercise_usage(model, item_index);

    // Per-variation score history, keyed by (item id, variant id); one pass
    // over sessions, attached to laddered exercises below (#1083).
    let variant_scores = build_variant_score_index(&model.sessions);

    let mut items: Vec<LibraryItemView> = Vec::new();

    for item in &model.items {
        let practice = model.practice_summaries.get(&item.id).cloned();
        let subtitle = item.composer.clone().unwrap_or_default();

        let linked_exercises = if item.kind == ItemKind::Piece {
            item.linked_exercise_ids
                .iter()
                .filter_map(|ex_id| {
                    let ex = item_index.get(ex_id.as_str())?;
                    if ex.kind != ItemKind::Exercise {
                        return None;
                    }
                    // The exercise's score in *this piece's* context, pulled
                    // from the same derivation the exercise screen uses so
                    // both sides agree (#1087 B2).
                    let piece_context_score =
                        usage_by_exercise.get(ex.id.as_str()).and_then(|rows| {
                            rows.iter()
                                .find(|r| {
                                    r.piece.as_ref().map(|p| p.id.as_str())
                                        == Some(item.id.as_str())
                                })
                                .and_then(|r| r.latest_score)
                        });
                    Some(LinkedExerciseView {
                        id: ex.id.clone(),
                        title: ex.title.clone(),
                        key: ex.key.clone(),
                        modality: ex.modality,
                        tempo_marking: ex.tempo.as_ref().and_then(|t| t.marking.clone()),
                        tempo_bpm: ex.tempo.as_ref().and_then(|t| t.bpm),
                        practice: model.practice_summaries.get(&ex.id).cloned(),
                        piece_context_score,
                    })
                })
                .collect()
        } else {
            vec![]
        };

        let used_in = if item.kind == ItemKind::Exercise {
            usage_by_exercise
                .get(item.id.as_str())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };

        // `already_linked` uses the same reconciliation key `CommitScaffold`
        // does, so the read-only preview and the commit agree.
        let scaffold_preview = item.chord_chart.as_ref().map(|chart| {
            let (linked_kinds, linked_titles) =
                crate::domain::item::linked_scaffold_state(model, &item.id);
            let specs = crate::domain::chart::derive_scaffold(chart);
            let mut fallback_total: u8 = 0;
            let spec_views = specs
                .iter()
                .map(|s| {
                    let fallback = s.fallback_count > 0;
                    if fallback {
                        fallback_total = fallback_total.saturating_add(1);
                    }
                    ScaffoldSpecView {
                        kind: s.kind,
                        title: s.title.clone(),
                        rationale: s.rationale.clone(),
                        key: s.key.clone(),
                        fallback,
                        already_linked: crate::domain::item::scaffold_already_linked(
                            &linked_kinds,
                            &linked_titles,
                            s.kind,
                            &s.title,
                        ),
                    }
                })
                .collect();
            ScaffoldPreviewView {
                key: chart.key.clone(),
                specs: spec_views,
                fallback_total,
            }
        });

        let variants = if item.kind == ItemKind::Exercise {
            build_variant_views(item, &variant_scores)
        } else {
            vec![]
        };
        let ladder_is_keys =
            crate::domain::variant::ladder_is_all_keys(variants.iter().map(|v| v.label.as_str()));
        let shows_key = crate::domain::variant::shows_key_field(variants.len());
        let solid_variation_count = variants.iter().filter(|v| v.is_solid).count();

        items.push(LibraryItemView {
            id: item.id.clone(),
            item_type: item.kind.clone(),
            title: item.title.clone(),
            subtitle,
            key: item.key.clone(),
            modality: item.modality,
            tempo_marking: item.tempo.as_ref().and_then(|t| t.marking.clone()),
            tempo_bpm: item.tempo.as_ref().and_then(|t| t.bpm),
            notes: item.notes.clone(),
            // Reserved scaffold markers never reach the UI or the tag
            // vocabulary (`available_tags` derives from these view tags).
            tags: item
                .tags
                .iter()
                .filter(|t| !crate::domain::chart::is_scaffold_tag(t))
                .cloned()
                .collect(),
            created_at: item.created_at.to_rfc3339(),
            updated_at: item.updated_at.to_rfc3339(),
            practice,
            priority: item.priority,
            linked_exercises,
            used_in,
            scaffold_preview,
            chord_chart: item.chord_chart.clone(),
            metre: item.metre.clone(),
            variants,
            ladder_is_keys,
            photo_id: item.photo_id.clone(),
            shows_key,
            solid_variation_count,
            key_selection: item
                .key
                .as_deref()
                .and_then(|key| crate::domain::key::wheel_selection(key, item.modality)),
        });
    }

    items
}

/// The Up next suggestion for a given instant. The seeding event re-derives
/// through this rather than trusting anything the shell sends (#1082).
pub(crate) fn derive_up_next(
    model: &Model,
    now: chrono::DateTime<chrono::Utc>,
) -> Option<crate::suggestion::SuggestedSession> {
    let item_index: std::collections::HashMap<&str, &crate::domain::item::Item> =
        model.items.iter().map(|i| (i.id.as_str(), i)).collect();
    let clock = crate::analytics::LocalClock::from_now(now, model.utc_offset_minutes);
    crate::suggestion::compute_up_next(&build_library_item_views(model, &item_index), clock)
}

/// The starred items in the order "Practise your priorities" seeds them
/// (#981). Re-derived in the core for the same reason as `derive_up_next`.
pub(crate) fn derive_priorities(model: &Model, now: chrono::DateTime<chrono::Utc>) -> Vec<String> {
    let item_index: std::collections::HashMap<&str, &crate::domain::item::Item> =
        model.items.iter().map(|i| (i.id.as_str(), i)).collect();
    let clock = crate::analytics::LocalClock::from_now(now, model.utc_offset_minutes);
    crate::priorities::order_priorities(&build_library_item_views(model, &item_index), clock)
        .iter()
        .map(|i| i.id.clone())
        .collect()
}

/// Build practice summaries (keyed by item_id) in a single pass over sessions.
/// Called once when sessions change, not per-render.
pub(crate) fn build_practice_summaries(
    sessions: &[PracticeSession],
) -> std::collections::HashMap<String, ItemPracticeSummary> {
    use crate::model::{ScoreHistoryEntry, TempoTrendPoint, TempoTrendView};
    use std::collections::HashMap;

    // (count, secs, score_history, tempo_points, last_practiced_at)
    type Acc = (
        usize,
        u64,
        Vec<ScoreHistoryEntry>,
        Vec<TempoTrendPoint>,
        Option<String>,
    );

    let mut acc: HashMap<String, Acc> = HashMap::new();

    for session in sessions {
        let session_date = session.started_at.to_rfc3339();
        for entry in &session.entries {
            let record = acc
                .entry(entry.item_id.clone())
                .or_insert_with(|| (0, 0, Vec::new(), Vec::new(), None));
            record.0 += 1;
            record.1 += entry.duration_secs;
            // Keep the latest date (RFC3339 strings compare chronologically).
            if record.4.as_ref().is_none_or(|cur| session_date > *cur) {
                record.4 = Some(session_date.clone());
            }

            if let Some(score) = entry.score_summary() {
                record.2.push(ScoreHistoryEntry {
                    session_date: session_date.clone(),
                    score,
                    session_id: session.id.clone(),
                });
            }

            // Still one point per entry. Across variations the tempos measure
            // different material, so a mean or a max would say nothing: the
            // last one measured is the honest single number (#1739).
            record.3.push(TempoTrendPoint {
                session_date: session_date.clone(),
                session_id: session.id.clone(),
                tempo: entry.plays.iter().rev().find_map(|p| p.achieved_tempo),
            });
        }
    }

    acc.into_iter()
        .map(
            |(
                item_id,
                (session_count, total_secs, mut score_history, mut tempo_points, last_practiced_at),
            )| {
                score_history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
                let latest_score = score_history.first().map(|e| e.score);

                tempo_points.sort_by(|a, b| {
                    (&a.session_date, &a.session_id).cmp(&(&b.session_date, &b.session_id))
                });
                let measured = tempo_points.iter().filter(|p| p.tempo.is_some()).count();
                let tempo_trend = TempoTrendView {
                    points: tempo_points,
                    has_trend: measured >= 2,
                };

                (
                    item_id,
                    ItemPracticeSummary {
                        session_count,
                        total_minutes: (total_secs / 60) as u32,
                        latest_score,
                        score_history,
                        tempo_trend,
                        last_practiced_at,
                    },
                )
            },
        )
        .collect()
}

/// Every piece an exercise is used in, keyed by exercise id: the pieces that
/// *link* it and the pieces it has been *practised with*, merged into one row
/// each (#1363). A pair that has both sources produces one row, not two.
pub(crate) fn build_exercise_usage(
    model: &Model,
    item_index: &std::collections::HashMap<&str, &crate::domain::item::Item>,
) -> std::collections::HashMap<String, Vec<crate::model::ExerciseUsageView>> {
    use crate::model::{ExerciseUsageView, PieceRefView};
    use std::collections::{HashMap, HashSet};

    // (exercise_id, piece_id | None) → accumulated rollup for that pairing.
    #[derive(Default)]
    struct Acc {
        linked: bool,
        piece_title: Option<String>,
        session_ids: HashSet<String>,
        last_practiced_at: Option<String>,
        // (date, score) of the most recent scored entry in this context.
        latest_scored: Option<(String, u8)>,
    }

    let mut acc: HashMap<(String, Option<String>), Acc> = HashMap::new();

    // Seeding from links first is what lets a piece appear before any history
    // exists. The kind check mirrors the piece side's forward filter.
    for piece in model.items.iter().filter(|i| i.kind == ItemKind::Piece) {
        for ex_id in &piece.linked_exercise_ids {
            if !item_index
                .get(ex_id.as_str())
                .is_some_and(|t| t.kind == ItemKind::Exercise)
            {
                continue;
            }
            acc.entry((ex_id.clone(), Some(piece.id.clone())))
                .or_default()
                .linked = true;
        }
    }

    for session in &model.sessions {
        let date = session.started_at.to_rfc3339();
        for entry in &session.entries {
            if entry.item_type != ItemKind::Exercise {
                continue;
            }
            // Context = the piece sharing this entry's block in the same
            // session; ungrouped or piece-less blocks fall to the None bucket.
            let piece = entry.group_id.as_deref().and_then(|g| {
                session
                    .entries
                    .iter()
                    .find(|e| e.item_type == ItemKind::Piece && e.group_id.as_deref() == Some(g))
            });
            let piece_id = piece.map(|p| p.item_id.clone());

            let record = acc.entry((entry.item_id.clone(), piece_id)).or_default();
            if let Some(p) = piece {
                record.piece_title = Some(p.item_title.clone());
            }
            record.session_ids.insert(session.id.clone());
            if record
                .last_practiced_at
                .as_ref()
                .is_none_or(|cur| date > *cur)
            {
                record.last_practiced_at = Some(date.clone());
            }
            if let Some(score) = entry.score_summary() {
                if record.latest_scored.as_ref().is_none_or(|(d, _)| date > *d) {
                    record.latest_scored = Some((date.clone(), score));
                }
            }
        }
    }

    let mut by_exercise: HashMap<String, Vec<ExerciseUsageView>> = HashMap::new();
    for ((exercise_id, piece_id), record) in acc {
        // #1093 (1a): prefer the live piece's current title so a rename shows
        // through; fall back to the practice-time snapshot when the piece is
        // gone. `piece_removed` records that fall-back so the shell can render
        // the row as retired history (2a).
        let piece = piece_id.map(|id| {
            let live = item_index.get(id.as_str());
            (
                PieceRefView {
                    title: live
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| record.piece_title.unwrap_or_default()),
                    subtitle: live.and_then(|p| p.composer.clone()),
                    id,
                },
                live.is_none(),
            )
        });
        let piece_removed = piece.as_ref().is_some_and(|(_, removed)| *removed);
        by_exercise
            .entry(exercise_id)
            .or_default()
            .push(ExerciseUsageView {
                piece: piece.map(|(p, _)| p),
                linked: record.linked,
                latest_score: record.latest_scored.map(|(_, s)| s),
                session_count: record.session_ids.len(),
                last_practiced_at: record.last_practiced_at,
                piece_removed,
            });
    }

    // Live pieces, then deleted ones, then "On its own". Within each, recency
    // descending (which sinks never-practised rows for free, `None` sorting
    // last under the reversed compare), then title, so linked-only rows (all
    // `None`) read alphabetically.
    for rows in by_exercise.values_mut() {
        rows.sort_by(|a, b| match (&a.piece, &b.piece) {
            (Some(x), Some(y)) => a
                .piece_removed
                .cmp(&b.piece_removed)
                .then_with(|| b.last_practiced_at.cmp(&a.last_practiced_at))
                .then_with(|| x.title.cmp(&y.title))
                .then_with(|| x.id.cmp(&y.id)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }
    by_exercise
}

/// Per-variation score history, read straight from the plays and keyed by
/// (item id, variation id), newest first (#1083, #1739 decision 9). Never via
/// `score_summary`, which is the lossy projection this exists to avoid.
pub(crate) fn build_variant_score_index(
    sessions: &[PracticeSession],
) -> std::collections::HashMap<(&str, &str), Vec<crate::model::ScoreHistoryEntry>> {
    use crate::model::ScoreHistoryEntry;
    use std::collections::HashMap;

    let mut index: HashMap<(&str, &str), Vec<ScoreHistoryEntry>> = HashMap::new();
    for session in sessions {
        for entry in &session.entries {
            for play in &entry.plays {
                let (Some(variant_id), Some(score)) = (&play.variation_id, play.score) else {
                    continue;
                };
                index
                    .entry((entry.item_id.as_str(), variant_id.as_str()))
                    .or_default()
                    .push(ScoreHistoryEntry {
                        session_date: session.started_at.to_rfc3339(),
                        score,
                        session_id: session.id.clone(),
                    });
            }
        }
    }
    for history in index.values_mut() {
        history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
    }
    index
}

/// Project an exercise's variations for the view: live ones only, in display
/// order, with their scores and the solid flag. No current rung: a variation
/// is unordered, and what to practise next is a recommendation (#1739
/// decision 1), which is #1501's job.
pub(crate) fn build_variant_views(
    item: &crate::domain::item::Item,
    variant_scores: &std::collections::HashMap<(&str, &str), Vec<crate::model::ScoreHistoryEntry>>,
) -> Vec<crate::model::VariantView> {
    use crate::domain::variant::SOLID_SCORE_MIN;
    use crate::model::{saved_mark_caption, VariantView};

    let mut live: Vec<_> = item
        .variants
        .iter()
        .filter(|v| v.deleted_at.is_none())
        .collect();
    live.sort_by_key(|v| v.position);

    let views: Vec<VariantView> = live
        .into_iter()
        .map(|v| {
            let score_history = variant_scores
                .get(&(item.id.as_str(), v.id.as_str()))
                .cloned()
                .unwrap_or_default();
            let latest_score = score_history.first().map(|e| e.score);
            let is_solid = latest_score.is_some_and(|s| s >= SOLID_SCORE_MIN);
            VariantView {
                id: v.id.clone(),
                label: v.label.clone(),
                position: v.position,
                latest_score,
                score_history,
                is_solid,
                caption: saved_mark_caption(latest_score, is_solid),
            }
        })
        .collect();

    views
}

/// Accents folded onto their base letter, then case removed, so "Étude" files
/// under E instead of after every ASCII title, and beside "Etude" rather than
/// after it (#1447).
fn title_sort_key(title: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    title
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub(super) const RECENTLY_PRACTISED_LIMIT: usize = 5;

/// The fields `compare_candidates` and `candidate_matches` read, borrowed so
/// neither `LibraryItemView` nor `PickerCandidate` needs cloning to share one
/// comparator and one search predicate between the Library's own sort/filter
/// and the picker sheet's (#1653).
struct CandidateRef<'a> {
    id: &'a str,
    title: &'a str,
    subtitle: &'a str,
    notes: Option<&'a str>,
    tags: &'a [String],
    created_at: &'a str,
    last_practiced_at: Option<&'a str>,
}

impl LibraryItemView {
    fn as_candidate_ref(&self) -> CandidateRef<'_> {
        CandidateRef {
            id: &self.id,
            title: &self.title,
            subtitle: &self.subtitle,
            notes: self.notes.as_deref(),
            tags: &self.tags,
            created_at: &self.created_at,
            last_practiced_at: self
                .practice
                .as_ref()
                .and_then(|p| p.last_practiced_at.as_deref()),
        }
    }
}

fn compare_candidates(
    a: &CandidateRef,
    b: &CandidateRef,
    sort: &LibrarySort,
) -> std::cmp::Ordering {
    let primary = match sort.field {
        SortField::DateAdded => a.created_at.cmp(b.created_at),
        SortField::Title => title_sort_key(a.title).cmp(&title_sort_key(b.title)),
        // None = "never practised" = earliest. Option ordering puts
        // None < Some, which is exactly that.
        SortField::LastPracticed => a.last_practiced_at.cmp(&b.last_practiced_at),
    };
    let directed = match sort.direction {
        SortDirection::Ascending => primary,
        SortDirection::Descending => primary.reverse(),
    };
    // Stable tiebreaker so equal keys don't jitter between renders.
    directed
        .then_with(|| b.created_at.cmp(a.created_at))
        .then_with(|| a.id.cmp(b.id))
}

fn candidate_matches(c: &CandidateRef, query_lower: &str) -> bool {
    c.title.to_lowercase().contains(query_lower)
        || c.subtitle.to_lowercase().contains(query_lower)
        || c.notes
            .is_some_and(|n| n.to_lowercase().contains(query_lower))
        || c.tags
            .iter()
            .any(|t| t.to_lowercase().contains(query_lower))
}

pub(super) fn sort_library_items(items: &mut [LibraryItemView], sort: &LibrarySort) {
    items.sort_by(|a, b| compare_candidates(&a.as_candidate_ref(), &b.as_candidate_ref(), sort));
}

pub(super) fn sorted_order(items: &[LibraryItemView], sort: &LibrarySort) -> Vec<usize> {
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|&a, &b| {
        compare_candidates(
            &items[a].as_candidate_ref(),
            &items[b].as_candidate_ref(),
            sort,
        )
    });
    order
}

pub(super) fn matches_query(item: &LibraryItemView, query: &ListQuery) -> bool {
    let scope = Scope {
        kind: query.item_type.as_ref(),
        priority_only: query.priority_only,
        tags: &query.tags,
    };
    if !scope.admits(&item.item_type, item.priority, &item.tags) {
        return false;
    }

    if let Some(ref key) = query.key {
        if item.key.as_deref() != Some(key.as_str()) {
            return false;
        }
    }

    if let Some(ref text) = query.text {
        let text_lower = text.to_lowercase();
        if !candidate_matches(&item.as_candidate_ref(), &text_lower) {
            return false;
        }
    }

    true
}

/// What the Library filter and the picker both narrow by before a search, so
/// the two cannot disagree about which items a star, a type or a tag leaves.
struct Scope<'a> {
    kind: Option<&'a ItemKind>,
    priority_only: bool,
    tags: &'a [String],
}

impl Scope<'_> {
    fn admits(&self, kind: &ItemKind, priority: bool, tags: &[String]) -> bool {
        if self.kind.is_some_and(|k| k != kind) {
            return false;
        }
        if self.priority_only && !priority {
            return false;
        }
        // Multi-tag filter is a union (match ANY, case-insensitive), not an intersection.
        if !self.tags.is_empty() {
            let selected: Vec<String> = self.tags.iter().map(|t| t.to_lowercase()).collect();
            return tags
                .iter()
                .any(|t| selected.contains(&t.trim().to_lowercase()));
        }
        true
    }
}

// ── Picker candidates (#1653) ──

/// The subset of `LibraryItemView` the picker sheet's sort and search read,
/// sent from Swift across the plain FFI call in `intrada-ffi`: not the full
/// 18-field view, only the fields `compare_candidates` and
/// `candidate_matches` use, with no nested view type of its own.
#[derive(Debug, Clone)]
pub struct PickerCandidate {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub last_practiced_at: Option<String>,
    pub kind: ItemKind,
    pub priority: bool,
}

/// The picker's own narrowing, the same rule as the Library's `ListQuery`
/// (#1999): an empty filter leaves every candidate.
#[derive(Debug, Clone, Default)]
pub struct PickerFilter {
    pub kind: Option<ItemKind>,
    pub priority_only: bool,
    pub tags: Vec<String>,
}

impl PickerFilter {
    fn scope(&self) -> Scope<'_> {
        Scope {
            kind: self.kind.as_ref(),
            priority_only: self.priority_only,
            tags: &self.tags,
        }
    }
}

impl PickerCandidate {
    fn as_candidate_ref(&self) -> CandidateRef<'_> {
        CandidateRef {
            id: &self.id,
            title: &self.title,
            subtitle: &self.subtitle,
            notes: self.notes.as_deref(),
            tags: &self.tags,
            created_at: &self.created_at,
            last_practiced_at: self.last_practiced_at.as_deref(),
        }
    }
}

/// The picker sheet's own sort and search, run against a candidate set the
/// shell already holds rather than the core's shared Library `ListQuery`, so
/// a tap in the picker never disturbs the Library screen (#1445, #1440,
/// #1653). Returns ids in filtered, sorted order; the shell reorders its own
/// list by them rather than the full items crossing the bridge again.
#[must_use]
pub fn sort_and_filter_candidates(
    candidates: &[PickerCandidate],
    sort: &LibrarySort,
    search: &str,
    filter: &PickerFilter,
) -> Vec<String> {
    let query = search.trim().to_lowercase();
    let scope = filter.scope();
    let mut filtered: Vec<&PickerCandidate> = candidates
        .iter()
        .filter(|c| scope.admits(&c.kind, c.priority, &c.tags))
        .filter(|c| query.is_empty() || candidate_matches(&c.as_candidate_ref(), &query))
        .collect();
    filtered.sort_by(|a, b| compare_candidates(&a.as_candidate_ref(), &b.as_candidate_ref(), sort));
    filtered.into_iter().map(|c| c.id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Picker candidates (#1653) ──

    fn picker_candidate(id: &str, title: &str, created_at: &str) -> PickerCandidate {
        PickerCandidate {
            id: id.to_string(),
            title: title.to_string(),
            subtitle: String::new(),
            notes: None,
            tags: Vec::new(),
            created_at: created_at.to_string(),
            last_practiced_at: None,
            kind: ItemKind::Piece,
            priority: false,
        }
    }

    fn scoped_candidates() -> Vec<PickerCandidate> {
        let mut p1 = picker_candidate("p1", "Ballade", "2026-01-01");
        p1.priority = true;
        p1.tags = vec!["recital".to_string()];
        let p2 = picker_candidate("p2", "Nocturne", "2026-01-02");
        let mut e1 = picker_candidate("e1", "Arpeggios", "2026-01-03");
        e1.kind = ItemKind::Exercise;
        e1.priority = true;
        e1.tags = vec!["Warm-up".to_string()];
        let mut e2 = picker_candidate("e2", "Scales", "2026-01-04");
        e2.kind = ItemKind::Exercise;
        e2.tags = vec!["warm-up ".to_string()];
        vec![p2, e2, p1, e1]
    }

    #[test]
    fn sort_and_filter_candidates_scopes_by_kind_star_and_tags() {
        let sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let warm_up = || vec!["WARM-UP".to_string()];
        let cases: Vec<(PickerFilter, &[&str], &str)> = vec![
            (
                PickerFilter::default(),
                &["e1", "p1", "p2", "e2"],
                "no scope",
            ),
            (
                PickerFilter {
                    kind: Some(ItemKind::Exercise),
                    ..Default::default()
                },
                &["e1", "e2"],
                "exercises only",
            ),
            (
                PickerFilter {
                    kind: Some(ItemKind::Piece),
                    ..Default::default()
                },
                &["p1", "p2"],
                "pieces only",
            ),
            (
                PickerFilter {
                    priority_only: true,
                    ..Default::default()
                },
                &["e1", "p1"],
                "the star",
            ),
            (
                PickerFilter {
                    tags: warm_up(),
                    ..Default::default()
                },
                &["e1", "e2"],
                "a tag matches whatever its case or stray spaces",
            ),
            (
                PickerFilter {
                    kind: Some(ItemKind::Piece),
                    priority_only: true,
                    ..Default::default()
                },
                &["p1"],
                "kind and star",
            ),
            (
                PickerFilter {
                    kind: Some(ItemKind::Exercise),
                    priority_only: true,
                    tags: warm_up(),
                },
                &["e1"],
                "all three",
            ),
            (
                PickerFilter {
                    kind: Some(ItemKind::Piece),
                    tags: warm_up(),
                    ..Default::default()
                },
                &[],
                "nothing in scope",
            ),
        ];
        for (filter, expected, why) in cases {
            let ids = sort_and_filter_candidates(&scoped_candidates(), &sort, "", &filter);
            assert_eq!(ids, expected, "{why}");
        }
    }

    #[test]
    fn sort_and_filter_candidates_scope_and_search_combine() {
        let filter = PickerFilter {
            kind: Some(ItemKind::Exercise),
            ..Default::default()
        };
        let ids = sort_and_filter_candidates(
            &scoped_candidates(),
            &LibrarySort::default(),
            "scal",
            &filter,
        );
        assert_eq!(ids, vec!["e2"]);
    }

    #[test]
    fn the_library_scope_and_the_picker_scope_agree() {
        let candidates = scoped_candidates();
        let views: Vec<LibraryItemView> = candidates
            .iter()
            .map(|c| {
                let mut view = LibraryItemView::fixture(&c.id, &c.title, c.kind.clone());
                view.priority = c.priority;
                view.tags = c.tags.clone();
                view
            })
            .collect();
        for kind in [None, Some(ItemKind::Piece), Some(ItemKind::Exercise)] {
            for priority_only in [false, true] {
                for tags in [
                    vec![],
                    vec!["warm-up".to_string()],
                    vec!["RECITAL".to_string()],
                ] {
                    let query = ListQuery {
                        item_type: kind.clone(),
                        priority_only,
                        tags: tags.clone(),
                        ..Default::default()
                    };
                    let mut library: Vec<&str> = views
                        .iter()
                        .filter(|v| matches_query(v, &query))
                        .map(|v| v.id.as_str())
                        .collect();
                    library.sort_unstable();
                    let filter = PickerFilter {
                        kind: kind.clone(),
                        priority_only,
                        tags,
                    };
                    let mut picker = sort_and_filter_candidates(
                        &candidates,
                        &LibrarySort::default(),
                        "",
                        &filter,
                    );
                    picker.sort_unstable();
                    assert_eq!(library, picker, "{query:?}");
                }
            }
        }
    }

    #[test]
    fn sort_and_filter_candidates_sorts_by_title_ascending() {
        let candidates = vec![
            picker_candidate("p1", "Clair de Lune", "2026-01-01"),
            picker_candidate("p2", "Etude", "2026-01-02"),
            picker_candidate("p3", "Ballade", "2026-01-03"),
        ];
        let sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(ids, vec!["p3", "p1", "p2"], "ascending title order");
    }

    #[test]
    fn sort_and_filter_candidates_sorts_accented_titles_beside_unaccented() {
        let candidates = vec![
            picker_candidate("p1", "Zephyr", "2026-01-01"),
            picker_candidate("p2", "Étude", "2026-01-02"),
            picker_candidate("p3", "Etude no. 2", "2026-01-03"),
        ];
        let sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["p2", "p3", "p1"],
            "Étude files under E beside Etude, matching the Library's own rule (#1447)"
        );
    }

    // Deliberately in the wrong order for every assertion below, so a
    // comparator that returns "equal" and leaves the input alone fails.
    // Translated from the deleted `LibraryItemSortTests.swift` (#1653).
    fn never_practiced_candidates() -> Vec<PickerCandidate> {
        vec![
            picker_candidate("b", "Scales", "2026-01-01"),
            picker_candidate("c", "Thirds", "2026-06-01"),
            picker_candidate("a", "Arpeggios", "2026-06-01"),
        ]
    }

    #[test]
    fn sort_and_filter_candidates_ties_resolve_newest_first_then_id() {
        let candidates = never_practiced_candidates();
        let sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Ascending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["a", "c", "b"],
            "never-practised items tie on the primary key; the newest created_at \
             wins the tiebreak, then id"
        );
    }

    #[test]
    fn sort_and_filter_candidates_tiebreak_ignores_direction() {
        let candidates = never_practiced_candidates();
        let sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Descending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["a", "c", "b"],
            "reversing the sort direction does not reverse the tiebreak"
        );
    }

    #[test]
    fn sort_and_filter_candidates_titles_sort_case_insensitively_both_directions() {
        let candidates = vec![
            picker_candidate("a", "Scales", "2026-01-01"),
            picker_candidate("b", "arpeggios", "2026-01-02"),
        ];
        let ascending = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let descending = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Descending,
        };

        assert_eq!(
            sort_and_filter_candidates(&candidates, &ascending, "", &PickerFilter::default()),
            vec!["b", "a"],
            "case-insensitive ascending: arpeggios before Scales"
        );
        assert_eq!(
            sort_and_filter_candidates(&candidates, &descending, "", &PickerFilter::default()),
            vec!["a", "b"],
            "case-insensitive descending: Scales before arpeggios"
        );
    }

    #[test]
    fn sort_and_filter_candidates_practised_item_sorts_after_never_practised() {
        // The never-practised one is older, so the tiebreak alone would put it
        // second; the primary key must decide first.
        let mut practised = picker_candidate("a", "Scales", "2026-01-02");
        practised.last_practiced_at = Some("2026-08-01".to_string());
        let never = picker_candidate("b", "Arpeggios", "2026-01-01");
        let candidates = vec![practised, never];
        let sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Ascending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["b", "a"],
            "never-practised sorts before a practised item"
        );
    }

    #[test]
    fn sort_and_filter_candidates_never_practiced_sorts_as_oldest() {
        let mut never = picker_candidate("p1", "Never practised", "2026-01-01");
        never.last_practiced_at = None;
        let mut practiced = picker_candidate("p2", "Practised", "2026-01-02");
        practiced.last_practiced_at = Some("2026-01-10".to_string());
        let candidates = vec![practiced, never];
        let sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Ascending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["p1", "p2"],
            "never-practised sorts earliest, same rule as the Library screen"
        );
    }

    #[test]
    fn sort_and_filter_candidates_stable_tiebreak_on_equal_keys() {
        let candidates = vec![
            picker_candidate("p2", "Same title", "2026-01-01"),
            picker_candidate("p1", "Same title", "2026-01-01"),
        ];
        let sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };

        let ids = sort_and_filter_candidates(&candidates, &sort, "", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["p1", "p2"],
            "equal keys break ties by id, never jitter"
        );
    }

    #[test]
    fn sort_and_filter_candidates_empty_search_matches_everything() {
        let candidates = vec![
            picker_candidate("p1", "Clair de Lune", "2026-01-01"),
            picker_candidate("p2", "Moonlight Sonata", "2026-01-02"),
        ];
        let sort = LibrarySort::default();

        let ids = sort_and_filter_candidates(&candidates, &sort, "   ", &PickerFilter::default());

        assert_eq!(ids.len(), 2, "whitespace-only search is no search");
    }

    #[test]
    fn sort_and_filter_candidates_search_matches_title_subtitle_notes_and_tags() {
        let mut by_title = picker_candidate("p1", "Moonlight Sonata", "2026-01-01");
        by_title.subtitle = "Beethoven".to_string();
        let mut by_subtitle = picker_candidate("p2", "Etude", "2026-01-02");
        by_subtitle.subtitle = "Debussy arrangement".to_string();
        let mut by_notes = picker_candidate("p3", "Ballade", "2026-01-03");
        by_notes.notes = Some("practice slowly for debussy voicing".to_string());
        let mut by_tag = picker_candidate("p4", "Prelude", "2026-01-04");
        by_tag.tags = vec!["Debussy".to_string()];
        let unrelated = picker_candidate("p5", "Nocturne", "2026-01-05");
        let candidates = vec![by_title, by_subtitle, by_notes, by_tag, unrelated];
        let sort = LibrarySort {
            field: SortField::DateAdded,
            direction: SortDirection::Ascending,
        };

        let ids =
            sort_and_filter_candidates(&candidates, &sort, "debussy", &PickerFilter::default());

        assert_eq!(
            ids,
            vec!["p2", "p3", "p4"],
            "case-insensitive match across subtitle, notes and tags; title itself \
             is not a Debussy match here"
        );
    }

    #[test]
    fn sort_and_filter_candidates_search_matching_nothing_returns_empty() {
        let candidates = vec![picker_candidate("p1", "Clair de Lune", "2026-01-01")];
        let sort = LibrarySort::default();

        let ids =
            sort_and_filter_candidates(&candidates, &sort, "nonexistent", &PickerFilter::default());

        assert!(ids.is_empty());
    }

    #[test]
    fn the_library_order_and_filter_agree_with_sort_and_filter_candidates() {
        // Two items share both title and created_at, so the id tiebreak has to
        // fire on both paths, and a third item is excluded by the text query
        // built directly through matches_query: this fails if either path
        // stops sharing the comparator or the predicate (#1653).
        let library_fixture = |id: &str, title: &str, created_at: &str, subtitle: &str| {
            let mut view = LibraryItemView::fixture(id, title, ItemKind::Piece);
            view.created_at = created_at.to_string();
            view.subtitle = subtitle.to_string();
            view
        };
        let library_items = vec![
            library_fixture("p2", "Debussy Prelude", "2026-01-01", "practice notes"),
            library_fixture("p1", "Debussy Prelude", "2026-01-01", "practice notes"),
            library_fixture("p3", "Chopin Ballade", "2026-01-02", "unrelated"),
        ];
        let sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let query = ListQuery {
            text: Some("practice".to_string()),
            ..Default::default()
        };
        let library_ids: Vec<String> = sorted_order(&library_items, &sort)
            .into_iter()
            .map(|i| &library_items[i])
            .filter(|i| matches_query(i, &query))
            .map(|i| i.id.clone())
            .collect();

        let picker_fixture = |id: &str, title: &str, created_at: &str, subtitle: &str| {
            let mut candidate = picker_candidate(id, title, created_at);
            candidate.subtitle = subtitle.to_string();
            candidate
        };
        let candidates = vec![
            picker_fixture("p2", "Debussy Prelude", "2026-01-01", "practice notes"),
            picker_fixture("p1", "Debussy Prelude", "2026-01-01", "practice notes"),
            picker_fixture("p3", "Chopin Ballade", "2026-01-02", "unrelated"),
        ];
        let picker_ids =
            sort_and_filter_candidates(&candidates, &sort, "practice", &PickerFilter::default());

        assert_eq!(
            library_ids,
            vec!["p1", "p2"],
            "tie on title and created_at breaks by id; p3 excluded by the text query"
        );
        assert_eq!(
            library_ids, picker_ids,
            "the Library's own path and the picker's path agree on the same input"
        );
    }
}

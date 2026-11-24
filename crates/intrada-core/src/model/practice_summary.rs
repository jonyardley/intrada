use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ItemPracticeSummary {
    pub session_count: usize,
    pub total_minutes: u32,
    pub latest_score: Option<u8>,
    pub score_history: Vec<ScoreHistoryEntry>,
    pub score_trend: Option<ScoreTrend>,
    pub tempo_trend: TempoTrendView,
    /// Most recent session date for this item (max `started_at`), independent
    /// of whether a score/tempo was recorded. `None` if never practised.
    /// RFC3339 — sorts chronologically as a string.
    pub last_practiced_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScoreHistoryEntry {
    pub session_date: String,
    pub score: u8,
    pub session_id: String,
}

/// The recent-sessions chip: the oldest and newest marks in `score_history`,
/// only when they differ.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScoreTrend {
    pub from: u8,
    pub to: u8,
}

impl ScoreTrend {
    pub(crate) fn over(newest_first: &[ScoreHistoryEntry]) -> Option<Self> {
        let (to, from) = (newest_first.first()?.score, newest_first.last()?.score);
        (from != to).then_some(Self { from, to })
    }
}

/// One practised session's slot in a `TempoTrendView`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoTrendPoint {
    pub session_date: String,
    pub session_id: String,
    pub tempo: Option<u16>,
}

/// An item's measured tempo over time, as a plottable series. A session that
/// measured no tempo keeps its slot with `tempo: None`, so the chart breaks the
/// line there rather than drawing a zero or interpolating across it (#1420).
/// Every number present was measured, because the evidence contract keeps
/// unevidenced defaults out of `achieved_tempo` upstream (T16, #1422).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoTrendView {
    /// Oldest first, the reverse of `score_history`. One point per setlist
    /// entry, matching `session_count`: an item practised twice in one session
    /// contributes two points at the same date.
    pub points: Vec<TempoTrendPoint>,
    /// Two or more measured tempos; below that there is no direction to read.
    pub has_trend: bool,
    /// `♩ = 88 → 104`, beside the title once there is a trend.
    pub chip: Option<String>,
    /// `Measured once, at ♩ = 72 · no trend yet`, in place of the plot.
    pub caption: Option<String>,
    pub spoken: String,
}

impl TempoTrendView {
    #[must_use]
    pub fn new(points: Vec<TempoTrendPoint>) -> Self {
        let measured: Vec<u16> = points.iter().filter_map(|p| p.tempo).collect();
        let words = crate::domain::tempo_words::trend_words(&measured, points.len());
        Self {
            has_trend: measured.len() >= 2,
            points,
            chip: words.chip,
            caption: words.caption,
            spoken: words.spoken,
        }
    }
}

#[cfg(test)]
impl ItemPracticeSummary {
    pub(crate) fn fixture() -> Self {
        Self {
            session_count: 0,
            total_minutes: 0,
            latest_score: None,
            score_history: Vec::new(),
            score_trend: None,
            tempo_trend: TempoTrendView::new(Vec::new()),
            last_practiced_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Score trend (#2233) ──

    fn history(newest_first: &[u8]) -> Vec<ScoreHistoryEntry> {
        newest_first
            .iter()
            .enumerate()
            .map(|(i, score)| ScoreHistoryEntry {
                session_date: format!("2026-09-{:02}T09:00:00Z", 28 - i),
                score: *score,
                session_id: format!("s{i}"),
            })
            .collect()
    }

    type TrendCase<'a> = (&'a [u8], Option<(u8, u8)>);

    #[test]
    fn score_trend_runs_from_the_oldest_mark_to_the_newest() {
        let cases: &[TrendCase] = &[
            (&[], None),
            (&[7], None),
            (&[5, 3], Some((3, 5))),
            (&[3, 5], Some((5, 3))),
            (&[6, 2, 9, 4], Some((4, 6))),
            (&[4, 9, 4], None),
        ];
        for (newest_first, expected) in cases {
            let trend = ScoreTrend::over(&history(newest_first));
            assert_eq!(
                trend.map(|t| (t.from, t.to)),
                *expected,
                "scores {newest_first:?}"
            );
        }
    }

    #[test]
    fn tempo_trend_view_round_trips_on_ffi_bincode_wire() {
        let point = |tempo| TempoTrendPoint {
            session_date: "2026-10-01T00:00:00+00:00".to_string(),
            session_id: "s1".to_string(),
            tempo,
        };
        crate::domain::types::assert_round_trips(TempoTrendView::new(vec![
            point(Some(88)),
            point(None),
            point(Some(104)),
        ]));
    }
}

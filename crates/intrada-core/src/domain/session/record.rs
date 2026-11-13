//! Time away on the open play (#2306): facts only; anything the app works out
//! is read here, when asked, and never stored.

use super::Play;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Live ──

/// Leaving and coming back on the open play (#2306). The raw times stay;
/// `left_out` takes the gap off the play's seconds as it closes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Away {
    pub left_at: DateTime<Utc>,
    /// `None` while still away.
    pub back_at: Option<DateTime<Utc>>,
    pub left_out: bool,
}

/// A screen lock for a few seconds is not time away.
pub const MIN_AWAY_SECS: i64 = 60;

// ── Away ──

/// Only gaps that began at or after `from` count: a resume re-anchors the
/// clock past the gaps its stamped seconds already left out.
pub(crate) fn left_out_secs(gaps: &[Away], from: DateTime<Utc>, to: DateTime<Utc>) -> u64 {
    gaps.iter()
        .filter(|a| a.left_out && a.left_at >= from)
        .filter_map(|a| {
            let end = a.back_at?.min(to);
            Some((end - a.left_at).num_seconds().max(0) as u64)
        })
        .sum()
}

/// The last away on the play, when it is back and long enough to offer.
pub(crate) fn away_to_offer(play: &Play) -> Option<(&Away, i64)> {
    let away = play.away.last()?;
    let secs = (away.back_at? - away.left_at).num_seconds();
    (!away.left_out && secs >= MIN_AWAY_SECS).then_some((away, secs))
}

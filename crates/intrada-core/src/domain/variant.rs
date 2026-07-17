use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One rung of an exercise's step ladder; "C", "Root position", "Land on
/// the 3rd". Users see "Steps"; `variant` is the core's name and never
/// appears on screen (#1083).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Variant {
    pub id: String,
    pub label: String,
    pub position: usize,
    /// Per-row LWW timestamp so a step can sync independently of its
    /// exercise (invariant 2). Same format as `Item::updated_at`.
    pub updated_at: DateTime<Utc>,
    /// Soft-delete tombstone. Tombstoned variants stay in `Item::variants`
    /// (views filter them) so session history keeps resolving labels and a
    /// re-added label resurrects its score history.
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A step is "Solid" (UI copy) once its latest score reaches this, of 10.
/// The current step is the first that isn't; progress means advancing the
/// rung, not polishing one rating (#1083; threshold decision in
/// specs/exercise-variants.md).
pub const SOLID_SCORE_MIN: u8 = 8;

/// Reconcile a ladder against the requested `labels` (ordered), matching by
/// case-insensitive label. A match keeps its id; and so its score history;
/// adopting the incoming casing and position. `updated_at` bumps only on rows
/// that actually changed (per-row LWW hygiene).
pub fn reconcile_variants(
    existing: Vec<Variant>,
    labels: &[String],
    now: DateTime<Utc>,
) -> Vec<Variant> {
    let mut pool = existing;
    let mut next: Vec<Variant> = Vec::with_capacity(labels.len());

    for (position, label) in labels.iter().enumerate() {
        // Prefer a live match; fall back to a tombstone, which a re-added
        // label resurrects; its id, and so its score history, come back.
        let matched = pool
            .iter()
            .position(|v| v.deleted_at.is_none() && v.label.to_lowercase() == label.to_lowercase())
            .or_else(|| {
                pool.iter()
                    .position(|v| v.label.to_lowercase() == label.to_lowercase())
            });
        match matched {
            Some(i) => {
                let mut v = pool.remove(i);
                let changed = v.position != position || v.label != *label || v.deleted_at.is_some();
                v.position = position;
                v.label = label.clone();
                v.deleted_at = None;
                if changed {
                    v.updated_at = now;
                }
                next.push(v);
            }
            None => next.push(Variant {
                id: ulid::Ulid::gen().to_string(),
                label: label.clone(),
                position,
                updated_at: now,
                deleted_at: None,
            }),
        }
    }

    // Whatever wasn't matched has left the ladder: tombstone live rows;
    // rows already tombstoned carry through untouched.
    for mut v in pool {
        if v.deleted_at.is_none() {
            v.deleted_at = Some(now);
            v.updated_at = now;
        }
        next.push(v);
    }

    next
}

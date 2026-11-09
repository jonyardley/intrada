use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::item::Item;

/// An exercise linked to a piece, at most one live link per pair (#2248).
/// Held on the piece, tombstones included.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ExerciseLink {
    pub id: String,
    pub exercise_id: String,
    pub position: usize,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// One change to a piece's exercise card (#2447); the core builds the whole set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum LinkChange {
    Move { exercise_id: String, to: usize },
    Unlink { exercise_id: String },
}

/// A link a write asks for. `position: None` keeps a claimed row where it is
/// and appends a new one after the piece's last.
pub(crate) struct WantedLink {
    pub exercise_id: String,
    pub position: Option<usize>,
}

impl ExerciseLink {
    pub(crate) fn new(exercise_id: String, position: usize, now: DateTime<Utc>) -> Self {
        Self {
            id: ulid::Ulid::generate().to_string(),
            exercise_id,
            position,
            updated_at: now,
            deleted_at: None,
        }
    }
}

pub(crate) fn links_to(ids: &[&str], now: DateTime<Utc>) -> Vec<ExerciseLink> {
    ids.iter()
        .enumerate()
        .map(|(position, id)| ExerciseLink::new(id.to_string(), position, now))
        .collect()
}

impl Item {
    /// Live links, by position.
    pub(crate) fn live_links(&self) -> Vec<&ExerciseLink> {
        let mut live: Vec<&ExerciseLink> = self
            .exercise_links
            .iter()
            .filter(|l| l.deleted_at.is_none())
            .collect();
        live.sort_by_key(|l| l.position);
        live
    }

    /// Each linked exercise once, in link order.
    pub(crate) fn linked_exercise_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for link in self.live_links() {
            if !ids.contains(&link.exercise_id) {
                ids.push(link.exercise_id.clone());
            }
        }
        ids
    }

    pub(crate) fn has_live_link(&self, exercise_id: &str) -> bool {
        self.live_links()
            .iter()
            .any(|l| l.exercise_id == exercise_id)
    }

    pub(crate) fn next_link_position(&self) -> usize {
        self.exercise_links
            .iter()
            .map(|l| l.position + 1)
            .max()
            .unwrap_or(0)
    }
}

/// Claims by exercise, reviving a tombstone before minting (#2248).
/// `wanted` holds each key once.
pub(crate) fn reconcile_links(
    existing: &[ExerciseLink],
    in_scope: impl Fn(&ExerciseLink) -> bool,
    wanted: Vec<WantedLink>,
    now: DateTime<Utc>,
) -> Vec<ExerciseLink> {
    let mut next: Vec<ExerciseLink> = existing.iter().filter(|l| !in_scope(l)).cloned().collect();
    let mut pool: Vec<ExerciseLink> = existing.iter().filter(|l| in_scope(l)).cloned().collect();
    let mut append_at = existing.iter().map(|l| l.position + 1).max().unwrap_or(0);
    let mut append = || {
        let p = append_at;
        append_at += 1;
        p
    };

    for w in wanted {
        let same = |l: &ExerciseLink| l.exercise_id == w.exercise_id;
        let claimed = pool
            .iter()
            .position(|l| same(l) && l.deleted_at.is_none())
            .or_else(|| pool.iter().position(same))
            .map(|i| pool.remove(i));
        match claimed {
            Some(mut l) => {
                let position = match (w.position, l.deleted_at.is_some()) {
                    (Some(p), _) => p,
                    (None, false) => l.position,
                    (None, true) => append(),
                };
                if l.position != position || l.deleted_at.is_some() {
                    l.position = position;
                    l.deleted_at = None;
                    l.updated_at = now;
                }
                next.push(l);
            }
            None => {
                let position = w.position.unwrap_or_else(&mut append);
                next.push(ExerciseLink::new(w.exercise_id, position, now));
            }
        }
    }

    for mut l in pool {
        if l.deleted_at.is_none() {
            l.deleted_at = Some(now);
            l.updated_at = now;
        }
        next.push(l);
    }
    next
}

/// Order-insensitive equality: the store loads rows in its own order.
pub(crate) fn same_links(a: &[ExerciseLink], b: &[ExerciseLink]) -> bool {
    let sorted = |rows: &[ExerciseLink]| {
        let mut rows = rows.to_vec();
        rows.sort_by(|x, y| x.id.cmp(&y.id));
        rows
    };
    sorted(a) == sorted(b)
}

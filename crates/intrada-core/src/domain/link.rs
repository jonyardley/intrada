use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::item::{Item, ScaffoldEntry};

/// An exercise linked to a piece, as a whole or to one of its sections
/// (#2248). Held on the piece beside its sections, tombstones included.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ExerciseLink {
    pub id: String,
    pub exercise_id: String,
    /// `None` is the whole piece.
    pub section_id: Option<String>,
    pub position: usize,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// One row of the piece's chosen set: an exercise, existing or written now,
/// and what it is linked to.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LinkEdit {
    pub exercise: ScaffoldEntry,
    pub section_id: Option<String>,
}

/// One row of the exercise's chosen set: a piece, as a whole or one section.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LinkTarget {
    pub piece_id: String,
    pub section_id: Option<String>,
}

/// A link a write asks for. `position: None` keeps a claimed row where it is
/// and appends a new one after the piece's last.
pub(crate) struct WantedLink {
    pub exercise_id: String,
    pub section_id: Option<String>,
    pub position: Option<usize>,
}

impl ExerciseLink {
    pub(crate) fn new(
        exercise_id: String,
        section_id: Option<String>,
        position: usize,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: ulid::Ulid::generate().to_string(),
            exercise_id,
            section_id,
            position,
            updated_at: now,
            deleted_at: None,
        }
    }
}

/// Whole-piece links to `ids`, in order.
pub(crate) fn whole_piece_links(ids: &[&str], now: DateTime<Utc>) -> Vec<ExerciseLink> {
    ids.iter()
        .enumerate()
        .map(|(position, id)| ExerciseLink::new(id.to_string(), None, position, now))
        .collect()
}

impl Item {
    /// Live links whose section, when they name one, is a live section of
    /// this item, by position.
    pub fn live_links(&self) -> Vec<&ExerciseLink> {
        let mut live: Vec<&ExerciseLink> = self
            .exercise_links
            .iter()
            .filter(|l| l.deleted_at.is_none())
            .filter(|l| {
                l.section_id.as_ref().is_none_or(|sid| {
                    self.sections
                        .iter()
                        .any(|s| &s.id == sid && s.deleted_at.is_none())
                })
            })
            .collect();
        live.sort_by_key(|l| l.position);
        live
    }

    /// Each linked exercise once, in the order of its first live link.
    pub fn linked_exercise_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for link in self.live_links() {
            if !ids.contains(&link.exercise_id) {
                ids.push(link.exercise_id.clone());
            }
        }
        ids
    }

    pub(crate) fn has_live_link(&self, exercise_id: &str, section_id: Option<&str>) -> bool {
        self.live_links()
            .iter()
            .any(|l| l.exercise_id == exercise_id && l.section_id.as_deref() == section_id)
    }

    pub(crate) fn is_live_section(&self, section_id: &str) -> bool {
        self.sections
            .iter()
            .any(|s| s.id == section_id && s.deleted_at.is_none())
    }

    pub(crate) fn next_link_position(&self) -> usize {
        self.exercise_links
            .iter()
            .map(|l| l.position + 1)
            .max()
            .unwrap_or(0)
    }
}

/// Reconcile the rows `in_scope` picks out against `wanted`, keyed by
/// (exercise, section): a claimed row keeps its id, a tombstone is revived
/// rather than a twin minted, and a live row left out is tombstoned. Rows out
/// of scope pass through untouched. `wanted` must hold each key once.
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
        let same =
            |l: &ExerciseLink| l.exercise_id == w.exercise_id && l.section_id == w.section_id;
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
                next.push(ExerciseLink::new(
                    w.exercise_id,
                    w.section_id,
                    position,
                    now,
                ));
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

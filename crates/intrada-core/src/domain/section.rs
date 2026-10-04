use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A part of a piece or exercise in score order: a form section ("A1") or a
/// trouble spot ("bars 12 to 14"). Library data only: how it is going is
/// derived from plays when read, never stored here (#2321). Not `Section`,
/// which would clash with SwiftUI's on every screen that shows one (#2245).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ItemSection {
    pub id: String,
    /// May be empty when `bars` is set; the view then labels it by its bars.
    pub name: String,
    pub bars: Option<BarRange>,
    pub kind: SectionKind,
    /// `None` is the piece's own tempo.
    pub target_bpm: Option<u16>,
    pub position: usize,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// "Bar 12" is `12..12`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BarRange {
    pub first: u16,
    pub last: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SectionKind {
    Form,
    TroubleSpot,
}

/// One row of the item screen's section list as sent: `id` names the row it
/// started from; `None` is a new row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SectionEdit {
    pub id: Option<String>,
    pub name: String,
    pub bars: BarsInput,
    pub kind: SectionKind,
    /// Typed text, as `TempoInput`: blank is the piece's own tempo.
    pub target_bpm: String,
}

/// Bars from the picker arrive as numbers; bars typed into a field arrive as
/// the raw text, so the core reads them by one rule (`parse_bar_range`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum BarsInput {
    Blank,
    Picked { first: u16, last: u16 },
    Typed(String),
}

/// A `SectionEdit` that passed validation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SectionDraft {
    pub id: Option<String>,
    pub name: String,
    pub bars: Option<BarRange>,
    pub kind: SectionKind,
    pub target_bpm: Option<u16>,
}

impl BarRange {
    pub fn caption(&self) -> String {
        if self.first == self.last {
            format!("Bar {}", self.first)
        } else {
            format!("Bars {} to {}", self.first, self.last)
        }
    }

    pub fn in_text(&self) -> String {
        if self.first == self.last {
            format!("bar {}", self.first)
        } else {
            format!("bars {} to {}", self.first, self.last)
        }
    }
}

impl ItemSection {
    /// Derived, not stored, so editing the bars keeps a nameless spot's label true.
    pub fn label(&self) -> String {
        match &self.bars {
            Some(bars) if self.name.is_empty() => bars.caption(),
            _ => self.name.clone(),
        }
    }

    /// `label` as it reads inside a sentence (#2248): a name as typed, bars
    /// in lower case.
    pub fn label_in_text(&self) -> String {
        match &self.bars {
            Some(bars) if self.name.is_empty() => bars.in_text(),
            _ => self.name.clone(),
        }
    }
}

/// By id only, so repeated names stay separate sections (#2245).
pub(crate) fn reconcile_sections(
    existing: Vec<ItemSection>,
    drafts: Vec<SectionDraft>,
    now: DateTime<Utc>,
) -> Vec<ItemSection> {
    let mut pool = existing;
    let mut next = Vec::with_capacity(drafts.len() + pool.len());

    for (position, draft) in drafts.into_iter().enumerate() {
        let claimed = draft
            .id
            .as_ref()
            .and_then(|id| pool.iter().position(|s| &s.id == id))
            .map(|i| pool.remove(i));
        match claimed {
            Some(mut s) => {
                let changed = s.name != draft.name
                    || s.bars != draft.bars
                    || s.kind != draft.kind
                    || s.target_bpm != draft.target_bpm
                    || s.position != position
                    || s.deleted_at.is_some();
                if changed {
                    s.name = draft.name;
                    s.bars = draft.bars;
                    s.kind = draft.kind;
                    s.target_bpm = draft.target_bpm;
                    s.position = position;
                    s.deleted_at = None;
                    s.updated_at = now;
                }
                next.push(s);
            }
            None => next.push(ItemSection {
                id: ulid::Ulid::generate().to_string(),
                name: draft.name,
                bars: draft.bars,
                kind: draft.kind,
                target_bpm: draft.target_bpm,
                position,
                updated_at: now,
                deleted_at: None,
            }),
        }
    }

    for mut s in pool {
        if s.deleted_at.is_none() {
            s.deleted_at = Some(now);
            s.updated_at = now;
        }
        next.push(s);
    }

    next
}

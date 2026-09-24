use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One rung of an exercise's variation ladder: "C", "Root position", "Land on
/// the 3rd". The core's name; on screen a ladder is "Variations", or "Keys"
/// when every live rung names one. Which of the two is the core's call, via
/// `ladder_is_all_keys` (#1083, moved off the shell in #1467): the shell
/// only prints the word. Score history is derived from session entries
/// tagged with this `id`, never stored here.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Variant {
    pub id: String,
    pub label: String,
    pub position: usize,
    /// Per-row LWW timestamp so a variation can sync independently of its
    /// exercise (invariant 2). Same format as `Item::updated_at`.
    pub updated_at: DateTime<Utc>,
    /// Soft-delete tombstone. Tombstoned variants stay in `Item::variants`
    /// (views filter them) so session history keeps resolving labels and a
    /// re-added label resurrects its score history.
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A variation is "Solid" (UI copy) once its latest score reaches this, of 10.
/// The current variation is the first that isn't; progress means advancing the
/// rung, not polishing one rating (#1083; threshold decision in
/// specs/exercise-variants.md).
pub const SOLID_SCORE_MIN: u8 = 8;

/// One row of a ladder as the Edit form sends it (#1783): `id` names the row
/// it started from, so a renamed row keeps its marks; `None` is a row typed
/// fresh, matched by label (a removed label re-added resurrects) or minted.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct VariantEdit {
    pub id: Option<String>,
    pub label: String,
}

/// Reconcile a ladder against the requested `labels` (ordered), matching by
/// case-insensitive label. A match keeps its id (and so its score history)
/// and adopts the incoming casing and position. `updated_at` bumps only on
/// rows that actually changed (per-row LWW hygiene).
pub fn reconcile_variants(
    existing: Vec<Variant>,
    labels: &[String],
    now: DateTime<Utc>,
) -> Vec<Variant> {
    let edits: Vec<VariantEdit> = labels
        .iter()
        .map(|label| VariantEdit {
            id: None,
            label: label.clone(),
        })
        .collect();
    reconcile_variant_edits(existing, &edits, now)
}

/// `reconcile_variants` with the rows' own ids in play: every id that still
/// names a row claims it first, so a swap of two labels or a rename beside a
/// fresh row with the old name never hands one row's marks to another. Only
/// then do the id-less rows match by label, live before tombstoned.
pub fn reconcile_variant_edits(
    existing: Vec<Variant>,
    edits: &[VariantEdit],
    now: DateTime<Utc>,
) -> Vec<Variant> {
    let mut pool = existing;
    let mut slots: Vec<Option<Variant>> = (0..edits.len()).map(|_| None).collect();

    for (slot, edit) in slots.iter_mut().zip(edits) {
        let Some(id) = &edit.id else { continue };
        if let Some(i) = pool.iter().position(|v| &v.id == id) {
            *slot = Some(pool.remove(i));
        }
    }

    for (slot, edit) in slots.iter_mut().zip(edits) {
        if slot.is_some() {
            continue;
        }
        let label = &edit.label;
        let matched = pool
            .iter()
            .position(|v| v.deleted_at.is_none() && v.label.to_lowercase() == label.to_lowercase())
            .or_else(|| {
                pool.iter()
                    .position(|v| v.label.to_lowercase() == label.to_lowercase())
            });
        if let Some(i) = matched {
            *slot = Some(pool.remove(i));
        }
    }

    let mut next: Vec<Variant> = Vec::with_capacity(edits.len());
    for (position, (slot, edit)) in slots.into_iter().zip(edits).enumerate() {
        let label = &edit.label;
        match slot {
            Some(mut v) => {
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
                id: ulid::Ulid::generate().to_string(),
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

/// An exercise in several keys has no single key (#1783 decision 1), so the
/// Key field hides on Add and Edit once the ladder has a live rung. A plain
/// call rather than an `Event`, like `sort_and_filter_picker_candidates`
/// (`ffi.rs`): the form re-derives this on every add/remove of an unsaved
/// variation row, before anything is sent to the core.
#[must_use]
pub fn shows_key_field(live_variant_count: usize) -> bool {
    live_variant_count == 0
}

// ── Keys or variations ────────────────────────────────────────────────

/// True when the ladder has rungs and every one of them names a key, so the
/// Library row can say "12 keys" rather than "12 variations" (#1467). All or
/// nothing: one rung that isn't a key and "keys" would be a lie about it.
/// A ladder with no rungs has nothing to be all of, so it is not keys.
pub(crate) fn ladder_is_all_keys<'a>(labels: impl IntoIterator<Item = &'a str>) -> bool {
    let mut any = false;
    for label in labels {
        any = true;
        if !is_key_label(label) {
            return false;
        }
    }
    any
}

/// Deliberately not a circle-of-fifths lookup: spellings off the wheel
/// ("D♯ major") are keys a musician types, and the wheel would reject them.
/// Swift's `KeyHelper` keeps its own wheel for the picker, which answers a
/// different question — which spoke is this — and is view-only (#819).
fn is_key_label(raw: &str) -> bool {
    let normalised = ascii_accidentals(raw);
    let tonic = strip_mode_word(&normalised);
    is_tonic(tonic)
}

fn ascii_accidentals(raw: &str) -> String {
    raw.replace('\u{266F}', "#")
        .replace('\u{266D}', "b")
        .trim()
        .to_string()
}

/// "major"/"minor" are ASCII, so a five-byte tail is a five-char tail; the
/// boundary check keeps a multi-byte rung from slicing mid-character.
fn strip_mode_word(value: &str) -> &str {
    match value.len().checked_sub(5) {
        Some(cut)
            if value.is_char_boundary(cut)
                && matches!(value[cut..].to_lowercase().as_str(), "major" | "minor") =>
        {
            &value[..cut]
        }
        _ => value,
    }
}

/// A spelled-out accidental is a key too (#1478).
fn is_tonic(raw: &str) -> bool {
    let trimmed = raw.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !('A'..='G').contains(&first.to_ascii_uppercase()) {
        return false;
    }
    let sign = chars
        .as_str()
        .trim_start()
        .trim_start_matches('-')
        .trim_start();
    matches!(
        sign.to_lowercase().as_str(),
        "" | "#" | "b" | "flat" | "sharp"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rungs a musician would actually type, not cases picked to match the
    /// scanner; the consumer is `ladder_is_all_keys`.
    #[test]
    fn labels_that_read_as_keys() {
        for label in [
            "C",
            "F#",
            "Bb",
            "F\u{266F}",
            "B\u{266D}",
            "c",
            " D ",
            "C major",
            "f# minor",
            // Spelled out and off the wheel: "C major, C♯ major, D♯ major…"
            // is all keys, so this is not a circle-of-fifths lookup.
            "D\u{266F} major",
            "G# major",
            "Db minor",
            "E flat major",
            "F sharp minor",
            "A flat",
            "c sharp",
            "E-flat major",
            "F-sharp minor",
        ] {
            assert!(is_key_label(label), "{label:?} names a key");
        }
    }

    #[test]
    fn labels_that_do_not() {
        for label in [
            "Root position",
            "1st inversion",
            "Land on the 3rd",
            "Hands together",
            "Variation 1",
            "Am",
            "Dm",
            "C dorian",
            "G mixolydian",
            "C/E",
            "H",
            "",
            "major",
            "E flatten major",
            "F sharpish minor",
        ] {
            assert!(!is_key_label(label), "{label:?} does not name a key");
        }
    }

    #[test]
    fn a_ladder_of_keys_is_keys() {
        assert!(ladder_is_all_keys(["C", "G", "D"]));
        assert!(ladder_is_all_keys([
            "C major",
            "C\u{266F} major",
            "D major"
        ]));
    }

    #[test]
    fn one_rung_that_is_not_a_key_makes_the_whole_ladder_variations() {
        assert!(!ladder_is_all_keys(["C", "G", "Hands together"]));
        assert!(!ladder_is_all_keys(["Root position", "1st inversion"]));
    }

    #[test]
    fn an_empty_ladder_is_not_keys() {
        assert!(!ladder_is_all_keys(std::iter::empty::<&str>()));
    }

    #[test]
    fn key_shows_with_no_variations_and_hides_from_the_first() {
        assert!(shows_key_field(0));
        assert!(!shows_key_field(1));
        assert!(!shows_key_field(2));
    }

    fn row(id: &str, label: &str, position: usize) -> Variant {
        Variant {
            id: id.to_string(),
            label: label.to_string(),
            position,
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    fn edit(id: Option<&str>, label: &str) -> VariantEdit {
        VariantEdit {
            id: id.map(str::to_string),
            label: label.to_string(),
        }
    }

    #[test]
    fn an_id_match_wins_over_a_label_match() {
        let existing = vec![row("c", "C", 0), row("f", "F", 1)];
        let edits = [edit(Some("c"), "F"), edit(Some("f"), "C")];

        let next = reconcile_variant_edits(existing, &edits, Utc::now());

        let live: Vec<(&str, &str, usize)> = next
            .iter()
            .filter(|v| v.deleted_at.is_none())
            .map(|v| (v.id.as_str(), v.label.as_str(), v.position))
            .collect();
        assert_eq!(live, vec![("c", "F", 0), ("f", "C", 1)], "swapped in place");
    }

    #[test]
    fn an_unknown_id_falls_back_to_the_label() {
        let existing = vec![row("c", "C", 0)];
        let edits = [edit(Some("gone"), "c")];

        let next = reconcile_variant_edits(existing, &edits, Utc::now());

        assert_eq!(next.len(), 1);
        assert_eq!(
            next[0].id, "c",
            "the row it named is gone, so the label decides"
        );
        assert_eq!(next[0].label, "c", "incoming casing adopted");
        assert!(next[0].deleted_at.is_none());
    }

    #[test]
    fn a_resurrected_row_bumps_updated_at_even_in_its_old_place() {
        let removed_at = Utc::now() - chrono::Duration::days(3);
        let existing = vec![Variant {
            updated_at: removed_at,
            deleted_at: Some(removed_at),
            ..row("c", "C", 0)
        }];
        let now = Utc::now();

        let next = reconcile_variant_edits(existing, &[edit(None, "C")], now);

        assert_eq!(next.len(), 1);
        assert_eq!(next[0].id, "c");
        assert!(next[0].deleted_at.is_none());
        assert_eq!(next[0].updated_at, now);
    }
}

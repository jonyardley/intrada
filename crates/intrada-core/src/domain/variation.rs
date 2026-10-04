use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One way of playing something, shared across the library like a tag:
/// "Hands separately", "Dotted rhythms" (#2246). Each item keeps the set it
/// uses in `Item::variation_ids`; a play names the ones it was on. Deleting
/// one tombstones it, so the plays that used it still say what it was.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Variation {
    pub id: String,
    pub label: String,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Seeded into an empty library with fixed ids, so two devices agree on them.
/// Ordinary rows from then on: renamed or deleted like any other.
pub const BUILT_INS: [(&str, &str); 4] = [
    ("00000000000000000000000001", "Hands separately"),
    ("00000000000000000000000002", "Dotted rhythms"),
    ("00000000000000000000000003", "Back to front"),
    ("00000000000000000000000004", "Three chord tones only"),
];

/// The built-ins, for a library that has never held a variation. A library
/// with only tombstones is not empty, so a deleted built-in stays deleted.
pub fn seed_if_empty(library: &[Variation], now: DateTime<Utc>) -> Option<Vec<Variation>> {
    library.is_empty().then(|| {
        BUILT_INS
            .iter()
            .map(|(id, label)| Variation {
                id: (*id).to_string(),
                label: (*label).to_string(),
                updated_at: now,
                deleted_at: None,
            })
            .collect()
    })
}

pub fn live_with_label<'a>(library: &'a [Variation], label: &str) -> Option<&'a Variation> {
    let wanted = label.to_lowercase();
    library
        .iter()
        .find(|v| v.deleted_at.is_none() && v.label.to_lowercase() == wanted)
}

pub fn is_live(library: &[Variation], id: &str) -> bool {
    library.iter().any(|v| v.id == id && v.deleted_at.is_none())
}

/// The ids for `labels`, in order: a live row with the label is reused, case
/// ignored, and any other label mints a row, returned in `minted`.
pub fn ids_for_labels(
    library: &[Variation],
    labels: &[String],
    now: DateTime<Utc>,
) -> (Vec<String>, Vec<Variation>) {
    let mut ids = Vec::with_capacity(labels.len());
    let mut minted: Vec<Variation> = Vec::new();
    for label in labels {
        let found = live_with_label(library, label).or_else(|| live_with_label(&minted, label));
        match found {
            Some(v) => ids.push(v.id.clone()),
            None => {
                let row = Variation {
                    id: ulid::Ulid::generate().to_string(),
                    label: label.clone(),
                    updated_at: now,
                    deleted_at: None,
                };
                ids.push(row.id.clone());
                minted.push(row);
            }
        }
    }
    (ids, minted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, label: &str) -> Variation {
        Variation {
            id: id.to_string(),
            label: label.to_string(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn a_new_label_mints_one_row_and_a_live_one_is_reused() {
        let library = vec![row("hs", "Hands separately")];
        let labels = ["hands SEPARATELY", "Slow", "slow"].map(String::from);

        let (ids, minted) = ids_for_labels(&library, &labels, Utc::now());

        assert_eq!(minted.len(), 1, "one row for Slow, typed twice");
        assert_eq!(minted[0].label, "Slow");
        assert_eq!(
            ids,
            vec!["hs".to_string(), minted[0].id.clone(), minted[0].id.clone()]
        );
    }

    #[test]
    fn a_tombstoned_label_is_not_reused() {
        let library = vec![Variation {
            deleted_at: Some(Utc::now()),
            ..row("old", "Slow")
        }];

        let (ids, minted) = ids_for_labels(&library, &["Slow".to_string()], Utc::now());

        assert_eq!(minted.len(), 1);
        assert_ne!(ids[0], "old");
    }

    #[test]
    fn the_built_ins_seed_once_and_never_return_after_deletion() {
        let now = Utc::now();
        let seeded = seed_if_empty(&[], now).expect("an empty library is seeded");
        let labels: Vec<&str> = seeded.iter().map(|v| v.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Hands separately",
                "Dotted rhythms",
                "Back to front",
                "Three chord tones only"
            ]
        );
        assert_eq!(seed_if_empty(&seeded, now), None);

        let all_deleted: Vec<Variation> = seeded
            .into_iter()
            .map(|v| Variation {
                deleted_at: Some(now),
                ..v
            })
            .collect();
        assert_eq!(seed_if_empty(&all_deleted, now), None);
    }

    #[test]
    fn built_in_ids_are_fixed_ulids() {
        for (id, _) in BUILT_INS {
            assert!(id.parse::<ulid::Ulid>().is_ok(), "{id}");
        }
    }
}

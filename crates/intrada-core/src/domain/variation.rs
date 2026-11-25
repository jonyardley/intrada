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

/// Fixed, not the seeding time, so a fresh device never outranks a rename
/// synced from another (`specs/icloud-sync.md`).
pub fn built_ins_seeded_at() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH
}

/// The built-ins, for a library that has never held a variation. A library
/// with only tombstones is not empty, so a deleted built-in stays deleted.
pub fn seed_if_empty(library: &[Variation]) -> Option<Vec<Variation>> {
    library.is_empty().then(|| {
        BUILT_INS
            .iter()
            .map(|(id, label)| Variation {
                id: (*id).to_string(),
                label: (*label).to_string(),
                updated_at: built_ins_seeded_at(),
                deleted_at: None,
            })
            .collect()
    })
}

pub fn live_with_label<'a>(library: &'a [Variation], label: &str) -> Option<&'a Variation> {
    let wanted = fold(label);
    library
        .iter()
        .find(|v| v.deleted_at.is_none() && fold(&v.label) == wanted)
}

/// One fold for "same label", so the sheet offers as new exactly what a save
/// would mint (#2754).
fn fold(label: &str) -> String {
    label.to_lowercase()
}

/// The positions in `labels` holding the typed text anywhere, case ignored;
/// none for blank text.
pub fn matching_labels<S: AsRef<str>>(query: &str, labels: &[S]) -> Vec<usize> {
    let wanted = fold(query.trim());
    if wanted.is_empty() {
        return Vec::new();
    }
    labels
        .iter()
        .enumerate()
        .filter(|(_, label)| fold(label.as_ref()).contains(&wanted))
        .map(|(index, _)| index)
        .collect()
}

/// The typed text, trimmed, as a new label, unless blank or already in
/// `taken` whatever its case.
pub fn addable_label<S: AsRef<str>>(query: &str, taken: &[S]) -> Option<String> {
    let text = query.trim();
    let wanted = fold(text);
    if wanted.is_empty() || taken.iter().any(|t| fold(t.as_ref()) == wanted) {
        return None;
    }
    Some(text.to_string())
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
        let seeded = seed_if_empty(&[]).expect("an empty library is seeded");
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
        assert_eq!(seed_if_empty(&seeded), None);

        let all_deleted: Vec<Variation> = seeded
            .into_iter()
            .map(|v| Variation {
                deleted_at: Some(now),
                ..v
            })
            .collect();
        assert_eq!(seed_if_empty(&all_deleted), None);
    }

    #[test]
    fn a_seeded_built_in_is_older_than_any_rename() {
        let seeded = seed_if_empty(&[]).expect("an empty library is seeded");
        assert!(seeded.iter().all(|v| v.updated_at == built_ins_seeded_at()));
        assert!(
            built_ins_seeded_at()
                < DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").expect("date")
        );
    }

    #[test]
    fn built_in_ids_are_fixed_ulids() {
        for (id, _) in BUILT_INS {
            assert!(id.parse::<ulid::Ulid>().is_ok(), "{id}");
        }
    }

    const SHEET: [&str; 3] = [
        "Hands separately",
        "Dotted rhythms",
        "Left hand alone, eyes closed",
    ];

    #[test]
    fn matching_finds_any_part_of_a_label_ignoring_case_and_spaces_round_the_text() {
        for (query, expected) in [
            ("left hand", vec![2]),
            ("  HANDS ", vec![0]),
            ("hand", vec![0, 2]),
            ("", vec![]),
            ("   ", vec![]),
            ("trill", vec![]),
        ] {
            assert_eq!(matching_labels(query, &SHEET), expected, "{query:?}");
        }
    }

    #[test]
    fn typed_text_is_addable_only_when_no_label_holds_it_whatever_its_case() {
        for (query, expected) in [
            ("Left hand", Some("Left hand")),
            ("  Slow practice ", Some("Slow practice")),
            ("dotted rhythms", None),
            ("DOTTED RHYTHMS ", None),
            ("", None),
            ("  ", None),
        ] {
            assert_eq!(
                addable_label(query, &SHEET).as_deref(),
                expected,
                "{query:?}"
            );
        }
    }

    #[test]
    fn a_label_typed_earlier_in_the_sheet_is_not_addable_twice() {
        let taken = [SHEET.as_slice(), &["Slow"]].concat();
        assert_eq!(addable_label("slow", &taken), None);
        assert_eq!(addable_label("slow", &SHEET).as_deref(), Some("slow"));
    }

    #[test]
    fn an_addable_label_is_one_a_save_would_mint_and_a_taken_one_it_would_reuse() {
        let library = vec![row("v1", "Dotted rhythms")];
        let labels: Vec<&str> = library.iter().map(|v| v.label.as_str()).collect();
        for query in ["dotted RHYTHMS", "Slow"] {
            let offered = addable_label(query, &labels);
            let (_, minted) = ids_for_labels(&library, &[query.to_string()], Utc::now());
            assert_eq!(offered.is_some(), !minted.is_empty(), "{query:?}");
        }
    }
}

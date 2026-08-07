//! Journey B's three altitudes: what a piece can be played at, and where a
//! gated run-through's verdicts land.
//!
//! **Open question 1, resolved (Phase C): v1 has no pipeline.** What a
//! run-through gates on is the piece's own named chart sections — the `[A]`,
//! `[Bridge]` labels the user already writes into their chart, in reading
//! order. There is no stage graph, no authored pipeline entity and nothing
//! between the piece and its sections. A piece with no chart, or a chart with
//! no labelled section, cannot be run through: B0 offers the two lower
//! altitudes instead, which is the graceful degradation B1's note asked for.
//!
//! Nothing lands on the piece as a whole. A section verdict is a bounded claim
//! about a named stretch of music; "the piece held" is not a claim anything can
//! currently make, so nothing here makes it (Phase B's interim, unchanged).

use serde::{Deserialize, Serialize};

use crate::domain::item::{Item, ItemKind};
use crate::engine::Altitude;

/// The sections a run-through would gate on, in reading order. Unlabelled
/// sections are skipped rather than numbered: a verdict the user cannot name is
/// a verdict they cannot give.
pub fn named_sections(item: &Item) -> Vec<String> {
    let Some(chart) = &item.chord_chart else {
        return Vec::new();
    };
    chart
        .sections
        .iter()
        .filter_map(|section| section.label.clone())
        .filter(|label| !label.trim().is_empty())
        .collect()
}

/// Where one section's verdicts land. Derived rather than stored, so the live
/// path and the launch replay cannot name the same section two ways (the #1214
/// class, and key decision 7's reason for putting the rule in one function).
///
/// Prefixed like [`super::blocks::node_id`], so a section can never collide
/// with an authored node by name.
pub fn section_node(item_id: &str, label: &str) -> String {
    format!("piece:{item_id}#{label}")
}

/// What B0 may offer for this piece. The core decides, so the sheet never has
/// to work out whether a run-through has anything to gate on.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct AltitudeOffer {
    pub item_id: String,
    pub title: String,
    /// False where the piece has no named sections. The sheet still shows the
    /// altitude and says why it is not available: a missing option with no
    /// explanation reads as a bug.
    pub run_through_available: bool,
    pub sections: Vec<String>,
}

/// `None` for anything that is not a piece: an exercise is drilled, not played
/// through.
pub fn altitude_offer(item: &Item) -> Option<AltitudeOffer> {
    if item.kind != ItemKind::Piece {
        return None;
    }
    let sections = named_sections(item);
    Some(AltitudeOffer {
        item_id: item.id.clone(),
        title: item.title.clone(),
        run_through_available: !sections.is_empty(),
        sections,
    })
}

/// Whether this altitude can be entered for this piece at all. Only the
/// run-through has a precondition; the two lower ones need nothing, which is
/// the point of them.
pub fn can_enter(item: &Item, altitude: Altitude) -> bool {
    match altitude {
        Altitude::RunThrough => !named_sections(item).is_empty(),
        Altitude::OffPiste | Altitude::Unmonitored => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::built_session::tests_support::sample_item;
    use crate::domain::chart::parse_chart;
    use crate::domain::item::Modality;

    fn charted(raw: &str) -> Item {
        let mut item = sample_item("p1", "Alice in Wonderland", ItemKind::Piece);
        item.chord_chart = Some(parse_chart(raw, "C", Modality::Major).expect("a valid chart"));
        item
    }

    #[test]
    fn a_pieces_named_sections_are_what_a_run_through_gates_on() {
        let item = charted("[A]\n| Cmaj7 | Am7 |\n[Bridge]\n| Dm7 | G7 |");
        assert_eq!(named_sections(&item), vec!["A", "Bridge"]);
    }

    #[test]
    fn an_uncharted_piece_has_nothing_to_gate_on() {
        let item = sample_item("p1", "Alice in Wonderland", ItemKind::Piece);
        assert!(named_sections(&item).is_empty());
        assert!(
            !can_enter(&item, Altitude::RunThrough),
            "a run-through with no sections would be a whole-piece verdict, which nothing can make"
        );
    }

    #[test]
    fn a_chart_with_no_labels_degrades_to_the_lower_altitudes() {
        let item = charted("| Cmaj7 | Am7 |");
        assert!(named_sections(&item).is_empty());
        assert!(!can_enter(&item, Altitude::RunThrough));
        assert!(can_enter(&item, Altitude::OffPiste));
        assert!(
            can_enter(&item, Altitude::Unmonitored),
            "the two lower altitudes need nothing authored — that is the point of them"
        );
    }

    #[test]
    fn an_unlabelled_section_is_skipped_rather_than_numbered() {
        let item = charted("| Cmaj7 |\n[B]\n| Dm7 |");
        assert_eq!(
            named_sections(&item),
            vec!["B"],
            "a verdict the user cannot name is a verdict they cannot give"
        );
    }

    #[test]
    fn a_section_node_can_never_collide_with_an_authored_one() {
        let node = section_node("p1", "A");
        assert_eq!(node, "piece:p1#A");
        assert!(crate::engine::ContentIndex::shipped().node(&node).is_none());
    }

    #[test]
    fn two_pieces_with_the_same_section_name_keep_separate_evidence() {
        assert_ne!(section_node("p1", "A"), section_node("p2", "A"));
    }

    #[test]
    fn the_offer_tells_the_sheet_whether_a_run_through_is_available() {
        let offer = altitude_offer(&charted("[A]\n| Cmaj7 |")).expect("a piece is offered");
        assert!(offer.run_through_available);
        assert_eq!(offer.sections, vec!["A"]);
        assert_eq!(offer.title, "Alice in Wonderland");

        let bare = altitude_offer(&sample_item("p2", "Untitled", ItemKind::Piece))
            .expect("an uncharted piece is still offered the lower altitudes");
        assert!(
            !bare.run_through_available,
            "the sheet says why rather than hiding the option"
        );
    }

    #[test]
    fn an_exercise_is_drilled_rather_than_played_through() {
        let exercise = sample_item("e1", "Two-fives", ItemKind::Exercise);
        assert_eq!(altitude_offer(&exercise), None);
    }
}

//! The Up next suggestion: one block worth resuming, in plain words (#1082,
//! `specs/up-next-card.md`). Pure and clock-injected like `analytics`, and
//! derived from the same `LibraryItemView` projection the Library screens
//! read, so the card and piece detail cannot disagree about a mark.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::analytics::LocalClock;
use crate::domain::item::ItemKind;
use crate::model::{ItemPracticeSummary, LibraryItemView, VariantView};

/// Related exercises the card suggests alongside the piece; with the piece
/// itself that makes the two-to-three rows the surface was designed for.
const MAX_SUGGESTED_EXERCISES: usize = 2;

/// What to assume an item takes when it has never been practised, so the
/// estimate has something honest to add for a brand-new piece.
const UNPRACTISED_ESTIMATE_MINS: u32 = 5;

/// One block worth resuming: the anchor piece, why, and what to play.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SuggestedSession {
    pub piece_id: String,
    pub piece_title: String,
    /// Composer, when the piece has one.
    pub piece_subtitle: Option<String>,
    /// Why this block: priority and staleness, in the user's language.
    pub reason: String,
    pub items: Vec<SuggestedItem>,
    /// What this block usually takes, to the nearest 5 minutes. An estimate
    /// drawn from past sessions, never a target and never enforced.
    pub estimated_minutes: u32,
}

/// One row of the suggestion, and one entry of the setlist it seeds.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SuggestedItem {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    /// The ladder step to practise, where the exercise has one that isn't solid.
    pub variant_id: Option<String>,
    pub variant_label: Option<String>,
    /// The mark this row's reason is drawn from: the step's where a step was
    /// chosen, otherwise the exercise's mark in this piece's context, or the
    /// piece's own latest mark.
    pub latest_score: Option<u8>,
    pub reason: String,
}

/// The block worth resuming, or `None` when nothing qualifies: an empty
/// library, or no piece with a related exercise linked. The card is a
/// suggestion and never a gate, so `None` simply leaves the Practice tab as
/// it was.
pub fn compute_up_next(items: &[LibraryItemView], clock: LocalClock) -> Option<SuggestedSession> {
    let by_id: HashMap<&str, &LibraryItemView> = items.iter().map(|i| (i.id.as_str(), i)).collect();

    let anchor = items
        .iter()
        .filter(|i| i.item_type == ItemKind::Piece && !i.linked_exercises.is_empty())
        .min_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| {
                    staleness_key(b.practice.as_ref(), clock)
                        .cmp(&staleness_key(a.practice.as_ref(), clock))
                })
                .then_with(|| latest_mark(a).cmp(&latest_mark(b)))
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
                .then_with(|| a.id.cmp(&b.id))
        })?;

    // Ranked by the mark that will be shown, so the row the card puts first is
    // the row whose reason says why. `sort_by` is stable, which keeps the
    // user's own link order as the tie-break.
    let mut ranked: Vec<Suggestable> = anchor
        .linked_exercises
        .iter()
        .map(|ex| Suggestable {
            id: ex.id.as_str(),
            title: ex.title.as_str(),
            step: by_id
                .get(ex.id.as_str())
                .and_then(|full| full.variants.iter().find(|v| v.is_current)),
            context_mark: ex.piece_context_score,
            practice: ex.practice.as_ref(),
        })
        .collect();
    ranked.sort_by(|a, b| {
        a.mark().cmp(&b.mark()).then_with(|| {
            staleness_key(a.practice, clock)
                .cmp(&staleness_key(b.practice, clock))
                .reverse()
        })
    });
    ranked.truncate(MAX_SUGGESTED_EXERCISES);

    let mut items_out: Vec<SuggestedItem> = ranked
        .iter()
        .map(|ex| SuggestedItem {
            item_id: ex.id.to_string(),
            item_title: ex.title.to_string(),
            item_type: ItemKind::Exercise,
            variant_id: ex.step.map(|s| s.id.clone()),
            variant_label: ex.step.map(|s| s.label.clone()),
            latest_score: ex.mark(),
            reason: mark_clause(ex.mark(), ex.step.is_some()),
        })
        .collect();

    let piece_staleness = staleness_days(anchor.practice.as_ref(), clock);
    items_out.push(SuggestedItem {
        item_id: anchor.id.clone(),
        item_title: anchor.title.clone(),
        item_type: ItemKind::Piece,
        variant_id: None,
        variant_label: None,
        latest_score: latest_mark(anchor),
        reason: piece_mark_clause(latest_mark(anchor)),
    });

    let estimated_minutes = round_to_five(
        ranked
            .iter()
            .map(|ex| average_minutes(ex.practice))
            .sum::<u32>()
            + average_minutes(anchor.practice.as_ref()),
    );

    let reason = if anchor.priority {
        format!("A priority · {}", staleness_clause(piece_staleness))
    } else {
        capitalise(&staleness_clause(piece_staleness))
    };

    Some(SuggestedSession {
        piece_id: anchor.id.clone(),
        piece_title: anchor.title.clone(),
        piece_subtitle: {
            let composer = anchor.subtitle.trim();
            (!composer.is_empty()).then(|| composer.to_string())
        },
        reason,
        items: items_out,
        estimated_minutes,
    })
}

/// A linked exercise plus the ladder rung it would practise, gathered once so
/// ranking, reasons and the estimate all read the same numbers.
struct Suggestable<'a> {
    id: &'a str,
    title: &'a str,
    step: Option<&'a VariantView>,
    context_mark: Option<u8>,
    practice: Option<&'a ItemPracticeSummary>,
}

impl Suggestable<'_> {
    /// The step's mark where a step was chosen, otherwise the exercise's mark
    /// in this piece's context. Per-piece, not flat: a drill solid under one
    /// tune can be rough under another (#1081).
    fn mark(&self) -> Option<u8> {
        match self.step {
            Some(step) => step.latest_score,
            None => self.context_mark,
        }
    }
}

fn staleness_days(practice: Option<&ItemPracticeSummary>, clock: LocalClock) -> Option<u32> {
    let at = practice?.last_practiced_at.as_deref()?;
    let parsed = chrono::DateTime::parse_from_rfc3339(at).ok()?;
    let day = clock.day_of(parsed.with_timezone(&chrono::Utc));
    Some((clock.today - day).num_days().max(0) as u32)
}

/// Ranking key for staleness: never practised is the stalest of all. One
/// convention for pieces and exercises alike, so the two rankings can't drift.
fn staleness_key(practice: Option<&ItemPracticeSummary>, clock: LocalClock) -> u32 {
    staleness_days(practice, clock).unwrap_or(u32::MAX)
}

fn latest_mark(item: &LibraryItemView) -> Option<u8> {
    item.practice.as_ref().and_then(|p| p.latest_score)
}

/// Mid-sentence, so it can follow the priority clause; `capitalise` heads a
/// line with it.
fn staleness_clause(days: Option<u32>) -> String {
    match days {
        None => "not practised yet".to_string(),
        Some(0) => "practised today".to_string(),
        Some(1) => "practised yesterday".to_string(),
        Some(d) => format!("not practised for {d} days"),
    }
}

fn mark_clause(mark: Option<u8>, has_step: bool) -> String {
    match (mark, has_step) {
        (Some(m), _) => marked_clause(m),
        (None, true) => "Step not marked yet".to_string(),
        (None, false) => "Not marked with this piece".to_string(),
    }
}

/// The anchor's own row. Distinct from the exercise wording because "with this
/// piece" makes no sense on the piece itself, and distinct from the card
/// headline so the two don't print the same sentence twice.
fn piece_mark_clause(mark: Option<u8>) -> String {
    match mark {
        Some(m) => marked_clause(m),
        None => "Not marked yet".to_string(),
    }
}

fn marked_clause(mark: u8) -> String {
    format!("Marked {mark} of 10 last time")
}

fn capitalise(clause: &str) -> String {
    let mut chars = clause.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// What one item usually takes: the average of the time actually spent on it,
/// falling back to the builder's default for anything never practised.
fn average_minutes(practice: Option<&ItemPracticeSummary>) -> u32 {
    match practice {
        Some(p) if p.session_count > 0 => (p.total_minutes / p.session_count as u32).max(1),
        _ => UNPRACTISED_ESTIMATE_MINS,
    }
}

fn round_to_five(minutes: u32) -> u32 {
    (((minutes + 2) / 5) * 5).max(5)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LinkedExerciseView;
    use chrono::NaiveDate;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 27).expect("valid date")
    }

    fn clock() -> LocalClock {
        LocalClock {
            today: today(),
            utc_offset_minutes: 0,
        }
    }

    /// A practice summary whose last session was `days_ago`, with `minutes`
    /// spent across `sessions` entries.
    fn practised(days_ago: i64, minutes: u32, sessions: usize) -> ItemPracticeSummary {
        let day = today() - chrono::Duration::days(days_ago);
        let at = day
            .and_hms_opt(10, 0, 0)
            .expect("valid time")
            .and_utc()
            .to_rfc3339();
        ItemPracticeSummary {
            session_count: sessions,
            total_minutes: minutes,
            last_practiced_at: Some(at),
            ..ItemPracticeSummary::fixture()
        }
    }

    fn exercise(id: &str, title: &str) -> LibraryItemView {
        LibraryItemView::fixture(id, title, ItemKind::Exercise)
    }

    /// A piece with `n` linked exercises, all present in the returned library.
    fn piece_with_exercises(id: &str, title: &str, n: usize) -> Vec<LibraryItemView> {
        let mut library = Vec::new();
        let mut piece = LibraryItemView::fixture(id, title, ItemKind::Piece);
        for i in 0..n {
            let ex_id = format!("{id}-ex{i}");
            let ex_title = format!("{title} drill {i}");
            piece
                .linked_exercises
                .push(LinkedExerciseView::fixture(&ex_id, &ex_title));
            library.push(exercise(&ex_id, &ex_title));
        }
        library.insert(0, piece);
        library
    }

    fn suggest(library: &[LibraryItemView]) -> SuggestedSession {
        compute_up_next(library, clock()).expect("a suggestion")
    }

    // ── Eligibility ──────────────────────────────────────────────────

    #[test]
    fn empty_library_suggests_nothing() {
        assert_eq!(compute_up_next(&[], clock()), None);
    }

    #[test]
    fn piece_with_no_linked_exercises_is_never_suggested() {
        let library = vec![LibraryItemView::fixture("p1", "Prelude", ItemKind::Piece)];
        assert_eq!(compute_up_next(&library, clock()), None);
    }

    #[test]
    fn an_exercise_is_never_the_anchor() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        library.push(exercise("solo", "Scales"));
        assert_eq!(suggest(&library).piece_id, "p1");
    }

    // ── Anchor ranking, one test per level of the tuple ──────────────

    #[test]
    fn a_priority_wins_over_a_staler_unstarred_piece() {
        let mut library = piece_with_exercises("starred", "Nocturne", 1);
        library[0].priority = true;
        library[0].practice = Some(practised(1, 10, 1));

        let mut other = piece_with_exercises("stale", "Fugue", 1);
        other[0].practice = Some(practised(40, 10, 1));
        library.extend(other);

        assert_eq!(suggest(&library).piece_id, "starred");
    }

    #[test]
    fn the_stalest_piece_wins_at_equal_priority() {
        let mut library = piece_with_exercises("fresh", "Nocturne", 1);
        library[0].practice = Some(practised(2, 10, 1));

        let mut stale = piece_with_exercises("stale", "Fugue", 1);
        stale[0].practice = Some(practised(20, 10, 1));
        library.extend(stale);

        assert_eq!(suggest(&library).piece_id, "stale");
    }

    #[test]
    fn a_piece_never_practised_is_the_stalest_of_all() {
        let mut library = piece_with_exercises("played", "Nocturne", 1);
        library[0].practice = Some(practised(90, 10, 1));
        library.extend(piece_with_exercises("never", "Fugue", 1));

        assert_eq!(suggest(&library).piece_id, "never");
    }

    #[test]
    fn the_lowest_mark_breaks_a_staleness_tie() {
        let mut library = piece_with_exercises("solid", "Nocturne", 1);
        library[0].practice = Some(ItemPracticeSummary {
            latest_score: Some(9),
            ..practised(5, 10, 1)
        });

        let mut rough = piece_with_exercises("rough", "Fugue", 1);
        rough[0].practice = Some(ItemPracticeSummary {
            latest_score: Some(3),
            ..practised(5, 10, 1)
        });
        library.extend(rough);

        assert_eq!(suggest(&library).piece_id, "rough");
    }

    #[test]
    fn the_title_breaks_a_total_tie() {
        let mut library = piece_with_exercises("p2", "Zortziko", 1);
        library.extend(piece_with_exercises("p1", "Aria", 1));

        assert_eq!(suggest(&library).piece_id, "p1");
    }

    // ── Setlist shape ────────────────────────────────────────────────

    #[test]
    fn suggests_at_most_two_exercises_then_the_piece() {
        let library = piece_with_exercises("p1", "Prelude", 4);
        let session = suggest(&library);

        assert_eq!(session.items.len(), 3);
        assert!(session.items[..2]
            .iter()
            .all(|i| i.item_type == ItemKind::Exercise));
        let last = session.items.last().expect("a last item");
        assert_eq!(last.item_type, ItemKind::Piece);
        assert_eq!(last.item_id, "p1");
    }

    #[test]
    fn a_piece_with_one_exercise_suggests_two_items() {
        let library = piece_with_exercises("p1", "Prelude", 1);
        assert_eq!(suggest(&library).items.len(), 2);
    }

    #[test]
    fn the_weakest_exercise_in_this_piece_comes_first() {
        let mut library = piece_with_exercises("p1", "Prelude", 3);
        library[0].linked_exercises[0].piece_context_score = Some(8);
        library[0].linked_exercises[1].piece_context_score = Some(3);
        library[0].linked_exercises[2].piece_context_score = Some(6);

        let session = suggest(&library);
        assert_eq!(session.items[0].item_id, "p1-ex1");
        assert_eq!(session.items[1].item_id, "p1-ex2");
    }

    #[test]
    fn an_exercise_never_marked_here_outranks_a_marked_one() {
        let mut library = piece_with_exercises("p1", "Prelude", 2);
        library[0].linked_exercises[0].piece_context_score = Some(2);
        library[0].linked_exercises[1].piece_context_score = None;

        assert_eq!(suggest(&library).items[0].item_id, "p1-ex1");
    }

    #[test]
    fn staleness_breaks_a_mark_tie_between_exercises() {
        let mut library = piece_with_exercises("p1", "Prelude", 3);
        for ex in &mut library[0].linked_exercises {
            ex.piece_context_score = Some(5);
        }
        library[0].linked_exercises[0].practice = Some(practised(1, 10, 1));
        library[0].linked_exercises[1].practice = Some(practised(30, 10, 1));
        library[0].linked_exercises[2].practice = Some(practised(3, 10, 1));

        assert_eq!(suggest(&library).items[0].item_id, "p1-ex1");
    }

    #[test]
    fn an_exercise_never_practised_is_the_stalest_of_all() {
        let mut library = piece_with_exercises("p1", "Prelude", 2);
        for ex in &mut library[0].linked_exercises {
            ex.piece_context_score = Some(5);
        }
        library[0].linked_exercises[0].practice = Some(practised(30, 10, 1));
        library[0].linked_exercises[1].practice = None;

        assert_eq!(suggest(&library).items[0].item_id, "p1-ex1");
    }

    // ── Steps ────────────────────────────────────────────────────────

    #[test]
    fn a_laddered_exercise_carries_its_current_step() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        let ex = library
            .iter_mut()
            .find(|i| i.id == "p1-ex0")
            .expect("the exercise");
        ex.variants = vec![
            VariantView {
                latest_score: Some(9),
                is_solid: true,
                ..VariantView::fixture("v0", "C", 0)
            },
            VariantView {
                latest_score: Some(4),
                is_current: true,
                ..VariantView::fixture("v1", "F", 1)
            },
        ];

        let first = &suggest(&library).items[0];
        assert_eq!(first.variant_id.as_deref(), Some("v1"));
        assert_eq!(first.variant_label.as_deref(), Some("F"));
        assert_eq!(first.latest_score, Some(4));
    }

    #[test]
    fn a_fully_solid_ladder_carries_no_step() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        library[0].linked_exercises[0].piece_context_score = Some(8);
        let ex = library
            .iter_mut()
            .find(|i| i.id == "p1-ex0")
            .expect("the exercise");
        ex.variants = vec![VariantView {
            latest_score: Some(9),
            is_solid: true,
            ..VariantView::fixture("v0", "C", 0)
        }];

        let first = &suggest(&library).items[0];
        assert_eq!(first.variant_id, None);
        assert_eq!(first.latest_score, Some(8));
    }

    #[test]
    fn a_steps_mark_orders_the_exercise_not_its_flat_one() {
        let mut library = piece_with_exercises("p1", "Prelude", 2);
        library[0].linked_exercises[0].piece_context_score = Some(4);
        library[0].linked_exercises[1].piece_context_score = Some(7);
        // ex1's ladder is on a rung far rougher than its flat mark suggests.
        let ex = library
            .iter_mut()
            .find(|i| i.id == "p1-ex1")
            .expect("the exercise");
        ex.variants = vec![VariantView {
            latest_score: Some(1),
            is_current: true,
            ..VariantView::fixture("v1", "F♯", 0)
        }];

        assert_eq!(suggest(&library).items[0].item_id, "p1-ex1");
    }

    // ── Reasons ──────────────────────────────────────────────────────

    #[test]
    fn the_headline_names_a_priority_and_the_staleness() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        library[0].priority = true;
        library[0].practice = Some(practised(6, 10, 1));

        assert_eq!(
            suggest(&library).reason,
            "A priority · not practised for 6 days"
        );
    }

    #[test]
    fn the_headline_of_an_unstarred_piece_is_the_staleness_alone() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        library[0].practice = Some(practised(6, 10, 1));

        assert_eq!(suggest(&library).reason, "Not practised for 6 days");
    }

    #[test]
    fn the_piece_row_does_not_echo_the_card_headline() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        library[0].practice = Some(practised(6, 10, 1));

        let session = suggest(&library);
        let piece_row = session.items.last().expect("the piece row");
        assert_ne!(
            piece_row.reason, session.reason,
            "the headline says when, the row says how it went"
        );
        assert_eq!(piece_row.reason, "Not marked yet");
    }

    /// The reasons a real library actually produces, not sentences written to
    /// match the formatter (CLAUDE.md, Testing). Asserts the property the
    /// screen needs of every one of them.
    #[test]
    fn every_reason_is_one_line_of_house_style() {
        let mut libraries: Vec<Vec<LibraryItemView>> = Vec::new();

        // Never touched.
        libraries.push(piece_with_exercises("a", "Aria", 2));

        // Played today, all marked.
        let mut fresh = piece_with_exercises("b", "Barcarolle", 2);
        fresh[0].practice = Some(practised(0, 12, 2));
        for (i, ex) in fresh[0].linked_exercises.iter_mut().enumerate() {
            ex.piece_context_score = Some(4 + i as u8);
            ex.practice = Some(practised(0, 5, 2));
        }
        libraries.push(fresh);

        // Yesterday, starred, one exercise unmarked here.
        let mut yesterday = piece_with_exercises("c", "Cavatina", 2);
        yesterday[0].priority = true;
        yesterday[0].practice = Some(practised(1, 20, 3));
        yesterday[0].linked_exercises[0].piece_context_score = Some(7);
        libraries.push(yesterday);

        // Long gone, with a laddered exercise mid-climb.
        let mut cold = piece_with_exercises("d", "Danza", 3);
        cold[0].practice = Some(practised(97, 30, 4));
        // Starred and long gone is the longest headline the formatter can
        // produce, so the copy budget below is measured against it.
        let mut starred_and_cold = piece_with_exercises("e", "Elegy", 2);
        starred_and_cold[0].priority = true;
        starred_and_cold[0].practice = Some(practised(97, 30, 4));
        libraries.push(starred_and_cold);
        let ex = cold
            .iter_mut()
            .find(|i| i.id == "d-ex1")
            .expect("the exercise");
        ex.variants = vec![VariantView {
            is_current: true,
            ..VariantView::fixture("v", "B♭", 0)
        }];
        libraries.push(cold);

        for library in &libraries {
            let session = suggest(library);
            let reasons: Vec<&str> = std::iter::once(session.reason.as_str())
                .chain(session.items.iter().map(|i| i.reason.as_str()))
                .collect();
            for reason in reasons {
                assert!(
                    !reason.is_empty(),
                    "empty reason in {}",
                    session.piece_title
                );
                assert!(!reason.contains('\n'), "multi-line reason: {reason}");
                assert!(
                    !reason.contains('—') && !reason.contains("--"),
                    "dash instead of the house separator: {reason}"
                );
                assert!(!reason.ends_with('.'), "full stop on a label: {reason}");
                assert!(!reason.contains('!'), "exclamation mark: {reason}");
                assert!(
                    !reason.to_lowercase().contains("score"),
                    "the screen says mark, not score: {reason}"
                );
                assert!(
                    !reason.to_lowercase().contains("you"),
                    "second person is for the user's own words: {reason}"
                );

                let lower = reason.to_lowercase();
                assert!(
                    !lower.contains("practiced") && !lower.contains("practicing"),
                    "American spelling (rule 1): {reason}"
                );
                assert!(
                    reason
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_uppercase() || c.is_numeric()),
                    "sentence case starts the line (rule 5): {reason}"
                );
                // Rule 9's one-line budget for a caption, not today's longest
                // output: it has room to grow before it needs re-deciding.
                assert!(
                    reason.split_whitespace().count() <= 10,
                    "over the copy budget: {reason}"
                );
            }
        }
    }

    // ── Estimate ─────────────────────────────────────────────────────

    #[test]
    fn the_estimate_averages_real_minutes_and_rounds_to_five() {
        let mut library = piece_with_exercises("p1", "Prelude", 2);
        // 22 min over 2 entries = 11; each exercise 8 over 2 = 4. 11+4+4 = 19.
        library[0].practice = Some(practised(3, 22, 2));
        for ex in &mut library[0].linked_exercises {
            ex.practice = Some(practised(3, 8, 2));
        }

        assert_eq!(suggest(&library).estimated_minutes, 20);
    }

    #[test]
    fn the_estimate_falls_back_to_the_default_for_the_unpractised() {
        let library = piece_with_exercises("p1", "Prelude", 2);
        let expected = 3 * UNPRACTISED_ESTIMATE_MINS;

        assert_eq!(suggest(&library).estimated_minutes, expected);
    }

    #[test]
    fn the_estimate_is_never_zero() {
        let mut library = piece_with_exercises("p1", "Prelude", 1);
        library[0].practice = Some(practised(1, 0, 1));
        library[0].linked_exercises[0].practice = Some(practised(1, 0, 1));

        assert!(suggest(&library).estimated_minutes >= 5);
    }

    // ── FFI wire ─────────────────────────────────────────────────────

    #[test]
    fn suggested_session_round_trips_on_ffi_bincode_wire() {
        let library = piece_with_exercises("p1", "Prelude", 2);
        crate::domain::types::assert_round_trips(suggest(&library));
    }
}

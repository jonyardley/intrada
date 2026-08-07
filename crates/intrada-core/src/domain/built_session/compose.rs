//! The steer sheet and decision 19's three-way resolution (Journey A, frames
//! A2–A5).
//!
//! The rule the whole module serves: **resolution is paid once per item, ever.**
//! A name the user has already answered for resolves silently on every later
//! visit, which is what makes a repeat composition add-add-add-build (A2r). A
//! name nothing knows costs exactly one question, and that question always has
//! an honest exit to the judgement track — the form never forces a fake gate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{BuiltTarget, JournalItem, UserDrill};
use crate::domain::item::{Item, ItemKind};
use crate::engine::{runnable_rung, ContentIndex, GateProgress, MasteryStore, Maturity};

/// The sheet's working state (A2). Not persisted: an abandoned composition is
/// abandoned. What it *created* on the way — a user drill, a journal item — is
/// a row of its own and survives independently, which is exactly why the second
/// visit asks nothing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ComposeDraft {
    pub entries: Vec<ComposeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComposeEntry {
    pub id: String,
    /// What the user typed or said, verbatim — the question screens quote it
    /// back, and a rejected match pre-fills the next form with it (A3).
    pub name: String,
    pub resolution: Resolution,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Resolution {
    /// Nothing more to ask: this entry is a block already.
    Settled(BuiltTarget),
    /// A question the sheet still owes. The count of these is the price the
    /// primary action states out loud (A2).
    Asking(Question),
}

/// The three resolution screens, one variant each. A question can change kind
/// without leaving the queue: "No, it's different" turns [`Question::NodeMatch`]
/// into [`Question::UserDrill`], and A4's quiet escape turns that into
/// [`Question::Journal`], so a misjudged kind costs one tap either way.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Question {
    /// A3 — an authored node whose name matches. Proposed, never assumed: the
    /// card carries its own evidence so the confirmation is informed.
    NodeMatch {
        node: String,
        title: String,
        /// The player's words for what the track says, snapshotted when the
        /// match was proposed so the view needs no clock.
        evidence_line: String,
        gate_filled: u8,
        gate_target: u8,
    },
    /// A4 — no node, so the criterion sentence is the gate (decision 19b).
    UserDrill,
    /// A5 — the judgement track (decision 19c).
    Journal,
}

/// Which of A4's two exits the user chose. Both forms are one surface.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ResolutionKind {
    UserDrill,
    Journal,
}

impl ComposeEntry {
    pub fn question(&self) -> Option<&Question> {
        match &self.resolution {
            Resolution::Asking(question) => Some(question),
            Resolution::Settled(_) => None,
        }
    }
}

impl ComposeDraft {
    pub fn entry_mut(&mut self, entry_id: &str) -> Option<&mut ComposeEntry> {
        self.entries.iter_mut().find(|entry| entry.id == entry_id)
    }

    pub fn open_questions(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.question().is_some())
            .count()
    }
}

/// What the sheet knows when it resolves a name. Passed rather than reached for
/// so the resolver stays a pure function over the library and the track.
pub struct ResolutionContext<'a> {
    pub items: &'a [Item],
    pub user_drills: &'a [UserDrill],
    pub journal_items: &'a [JournalItem],
    pub content: &'a ContentIndex,
    pub mastery: &'a MasteryStore,
    pub now: DateTime<Utc>,
}

/// Decision 19, in order of what the user has already paid for.
///
/// An item the user has answered for once — a drill, a journal target, a piece
/// they picked from the library — resolves with no question at all. Only a name
/// nothing recognises reaches a question, and a proposed authored node is the
/// one question worth asking, because it is the only one that can hand back
/// existing evidence.
pub fn resolve(name: &str, picked_item_id: Option<&str>, ctx: &ResolutionContext) -> Resolution {
    if let Some(item_id) = picked_item_id {
        if let Some(item) = ctx.items.iter().find(|item| item.id == item_id) {
            return Resolution::Settled(settled_item(item));
        }
    }
    let wanted = normalised(name);
    if wanted.is_empty() {
        return Resolution::Asking(Question::UserDrill);
    }

    // Paid once, ever (A2r): the user's own drills and journal targets first.
    if let Some(drill) = ctx
        .user_drills
        .iter()
        .find(|drill| normalised(&drill.name) == wanted)
    {
        return Resolution::Settled(BuiltTarget::UserDrill {
            drill_id: drill.id.clone(),
        });
    }
    if let Some(journal) = ctx
        .journal_items
        .iter()
        .find(|journal| normalised(&journal.name) == wanted)
    {
        return Resolution::Settled(BuiltTarget::Journal {
            journal_id: journal.id.clone(),
        });
    }
    if let Some(item) = ctx
        .items
        .iter()
        .find(|item| normalised(&item.title) == wanted)
    {
        // A piece costs nothing — pieces are user-added by nature. A library
        // exercise carries no gate, so it is case (b) with the name known.
        if item.kind == ItemKind::Piece {
            return Resolution::Settled(settled_item(item));
        }
    }

    match propose_node(&wanted, ctx) {
        Some(question) => Resolution::Asking(question),
        None => Resolution::Asking(Question::UserDrill),
    }
}

fn settled_item(item: &Item) -> BuiltTarget {
    BuiltTarget::Piece {
        item_id: item.id.clone(),
    }
}

/// (a) An authored node by name. A looser match than the settled kinds get,
/// deliberately: this one is proposed and confirmed, so "Hanon №4, hands
/// together" may reach "Hanon №4 — hands together, crotchet = 96", and a wrong
/// guess costs one tap.
fn propose_node(wanted: &str, ctx: &ResolutionContext) -> Option<Question> {
    let node = ctx
        .content
        .nodes
        .values()
        .filter(|node| runnable_rung(ctx.content, &node.id).is_some())
        .find(|node| {
            let title = normalised(&node.title);
            title == *wanted || title.contains(wanted) || wanted.contains(&title)
        })?;
    let (_, level) = runnable_rung(ctx.content, &node.id)?;
    let reading = ctx.mastery.reading(&node.id, level, ctx.now);
    let target = ctx
        .content
        .drill(node.drills.first()?)
        .and_then(|drill| ctx.content.gate(&drill.gate))
        .map(|gate| GateProgress::new(&gate.requirement).target())
        .unwrap_or(1);
    let maturity = if reading.estimate >= ctx.content.planner.maintenance_estimate {
        Maturity::Maintaining
    } else if reading.evidence >= 1.0 {
        Maturity::Acquiring
    } else {
        Maturity::New
    };
    Some(Question::NodeMatch {
        node: node.id.clone(),
        title: node.title.clone(),
        evidence_line: evidence_line(maturity, level.tempo_bpm, level.is_untimed()),
        gate_filled: reading.evidence.round().clamp(0.0, f32::from(target)) as u8,
        gate_target: target,
    })
}

/// The track in the player's words (T13): no estimate, no evidence count, no
/// engine vocabulary on screen.
fn evidence_line(maturity: Maturity, tempo_bpm: u16, untimed: bool) -> String {
    let at = if untimed {
        String::new()
    } else {
        format!(" at {tempo_bpm}")
    };
    match maturity {
        Maturity::New => "New ground".to_string(),
        Maturity::Acquiring => format!("Getting there{at}"),
        Maturity::Maintaining => format!("Solid{at}"),
    }
}

// ── The view half ────────────────────────────────────────────────────

/// What the steer sheet and the composed session draw (A2–A6). `None` on both
/// halves is the ordinary prescribed day, hero untouched.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BuiltView {
    /// `Some` while the sheet is open.
    pub compose: Option<ComposeView>,
    /// `Some` once a session is composed (A6), until it is replaced.
    pub session: Option<ComposedSessionView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ComposeView {
    pub entries: Vec<ComposeEntryView>,
    /// The questions still owed, in list order — the wizard's "1 of 3". Empty
    /// on a repeat visit, which is the whole point of A2r.
    pub questions: Vec<ComposeQuestionView>,
    /// The primary action's words, price included, written by the core so the
    /// shell never composes a sentence about what it will cost.
    pub build_label: String,
    pub can_build: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ComposeEntryView {
    pub id: String,
    pub name: String,
    pub kind: ComposeKind,
    /// One line under the name where there is something worth saying.
    pub note: Option<String>,
}

/// What chip a row wears (A2). `Unresolved` is the row that still owes a
/// question, and it wears no false kind while it does.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ComposeKind {
    Piece,
    Exercise,
    Journal,
    Unresolved,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ComposeQuestionView {
    pub entry_id: String,
    /// The name the user gave, quoted back at the top of every question.
    pub name: String,
    pub ask: AskView,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum AskView {
    /// A3.
    NodeMatch {
        title: String,
        evidence_line: String,
        gate_filled: u8,
        gate_target: u8,
    },
    /// A4. The criterion the shell starts the field with, already read back —
    /// the form shows chips, never empty fields.
    UserDrill {
        criterion: String,
        tempo_bpm: Option<u16>,
        keys: Vec<String>,
        passes_to_open: u8,
        serves_options: Vec<ServesOptionView>,
    },
    /// A5.
    Journal,
}

/// One tappable *helps with* tag (A4). The piece the lesson handed over comes
/// pre-proposed, so the common case is one tap.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ServesOptionView {
    pub label: String,
    pub serves: super::Serves,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ComposedSessionView {
    pub id: String,
    /// "From Friday's lesson" — the hero names its source rather than
    /// pretending the plan wrote it (A6).
    pub source: Option<String>,
    pub total_minutes: u16,
    pub blocks: Vec<ComposedBlockView>,
    /// One line, declinable, `None` when the order already is the shape.
    pub shape_advice: Option<String>,
    /// What could not run today, in the plan's own words. Rendered as it
    /// stands; silent dropping is a defect.
    pub deferred: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ComposedBlockView {
    pub title: String,
    pub minutes: u16,
    pub kind: ComposeKind,
}

/// What A2's primary action says. The price is named, never hidden: on a
/// repeat visit it confirms the price is zero, which is what makes the second
/// composition feel like the drag-and-go the old builder was.
fn build_label(entries: usize, questions: usize) -> String {
    match (entries, questions) {
        (0, _) => "Build session".to_string(),
        (_, 0) => "Build session — no questions today".to_string(),
        (_, 1) => "Continue — one quick question".to_string(),
        (_, open) => format!("Continue — {} quick questions", spoken(open)),
    }
}

fn spoken(count: usize) -> String {
    match count {
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        5 => "five".to_string(),
        other => other.to_string(),
    }
}

fn compose_kind(target: &BuiltTarget) -> ComposeKind {
    match target {
        BuiltTarget::Piece { .. } => ComposeKind::Piece,
        BuiltTarget::Node { .. } | BuiltTarget::UserDrill { .. } => ComposeKind::Exercise,
        BuiltTarget::Journal { .. } => ComposeKind::Journal,
    }
}

fn entry_note(resolution: &Resolution) -> Option<String> {
    match resolution {
        Resolution::Settled(BuiltTarget::Piece { .. }) => {
            Some("Joins your tunes — learn, memorise, run cold".to_string())
        }
        _ => None,
    }
}

/// The sheet as the shell draws it. Built rather than stored, so a drill
/// created mid-composition changes the row it came from with no extra event.
pub fn compose_view(draft: &ComposeDraft, items: &[Item]) -> ComposeView {
    let questions: Vec<ComposeQuestionView> = draft
        .entries
        .iter()
        .filter_map(|entry| {
            Some(ComposeQuestionView {
                entry_id: entry.id.clone(),
                name: entry.name.clone(),
                ask: ask_view(entry, draft, items)?,
            })
        })
        .collect();
    ComposeView {
        entries: draft
            .entries
            .iter()
            .map(|entry| ComposeEntryView {
                id: entry.id.clone(),
                name: entry.name.clone(),
                kind: match &entry.resolution {
                    Resolution::Settled(target) => compose_kind(target),
                    Resolution::Asking(_) => ComposeKind::Unresolved,
                },
                note: entry_note(&entry.resolution),
            })
            .collect(),
        build_label: build_label(draft.entries.len(), questions.len()),
        can_build: !draft.entries.is_empty() && questions.is_empty(),
        questions,
    }
}

fn ask_view(entry: &ComposeEntry, draft: &ComposeDraft, items: &[Item]) -> Option<AskView> {
    Some(match entry.question()? {
        Question::NodeMatch {
            title,
            evidence_line,
            gate_filled,
            gate_target,
            ..
        } => AskView::NodeMatch {
            title: title.clone(),
            evidence_line: evidence_line.clone(),
            gate_filled: *gate_filled,
            gate_target: *gate_target,
        },
        Question::UserDrill => {
            // The field opens on the name the user already said, so the
            // sentence is edited rather than written from nothing.
            let parsed = super::criterion::parse_criterion(&entry.name);
            AskView::UserDrill {
                criterion: entry.name.clone(),
                tempo_bpm: parsed.tempo_bpm,
                keys: parsed.keys,
                passes_to_open: parsed.passes_to_open,
                serves_options: serves_options(draft, items),
            }
        }
        Question::Journal => AskView::Journal,
    })
}

/// The pieces already in today's list, offered first: a lesson that handed over
/// a piece and a drill for it means the tag is one tap, not a search.
fn serves_options(draft: &ComposeDraft, items: &[Item]) -> Vec<ServesOptionView> {
    let mut options: Vec<ServesOptionView> = draft
        .entries
        .iter()
        .filter_map(|entry| match &entry.resolution {
            Resolution::Settled(BuiltTarget::Piece { item_id }) => {
                let item = items.iter().find(|item| item.id == *item_id)?;
                Some(ServesOptionView {
                    label: item.title.clone(),
                    serves: super::Serves::Node(item.id.clone()),
                })
            }
            _ => None,
        })
        .collect();
    for (label, circle) in [
        ("What you can hear", crate::engine::Circle::Head),
        ("What your hands know", crate::engine::Circle::Hands),
    ] {
        options.push(ServesOptionView {
            label: label.to_string(),
            serves: super::Serves::Circle(circle),
        });
    }
    options
}

/// A6, once the session exists. Reads the composed row rather than the plan, so
/// what the user sees before Start is exactly what they built.
pub fn composed_session_view(
    session: &super::BuiltSession,
    ctx: &super::blocks::BuildContext,
) -> ComposedSessionView {
    let plan = super::blocks::plan_from_built(session, ctx);
    ComposedSessionView {
        id: session.id.clone(),
        source: session.source.clone(),
        total_minutes: plan.blocks.iter().map(|block| block.spec.minutes).sum(),
        blocks: session
            .blocks
            .iter()
            .filter_map(|block| {
                // A block the plan could not take is already spoken for in
                // `deferred`; it must not also show as a row of the session.
                let node = super::blocks::node_id(&block.target);
                let planned = plan.blocks.iter().find(|p| p.spec.node == node)?;
                Some(ComposedBlockView {
                    title: planned.spec.drill_title.clone(),
                    minutes: planned.spec.minutes,
                    kind: compose_kind(&block.target),
                })
            })
            .collect(),
        shape_advice: super::blocks::shape_advice(&session.blocks).map(str::to_string),
        deferred: plan.deferred.clone(),
    }
}

/// Case-folded, punctuation-free, single-spaced. Two names that differ only in
/// how they were typed are the same name.
pub fn normalised(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space_pending = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            if space_pending && !out.is_empty() {
                out.push(' ');
            }
            space_pending = false;
            out.extend(ch.to_lowercase());
        } else {
            space_pending = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::built_session::tests_support::{journal_item, sample_item, user_drill};
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 7, 10, 0, 0).unwrap()
    }

    struct Fixture {
        items: Vec<Item>,
        drills: Vec<UserDrill>,
        journals: Vec<JournalItem>,
        mastery: MasteryStore,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                items: vec![],
                drills: vec![],
                journals: vec![],
                mastery: MasteryStore::seeded_from(ContentIndex::shipped()),
            }
        }

        fn resolve(&self, name: &str, picked: Option<&str>) -> Resolution {
            super::resolve(
                name,
                picked,
                &ResolutionContext {
                    items: &self.items,
                    user_drills: &self.drills,
                    journal_items: &self.journals,
                    content: ContentIndex::shipped(),
                    mastery: &self.mastery,
                    now: now(),
                },
            )
        }
    }

    /// The named node the shipped content actually carries, so the tests assert
    /// the resolver rather than whatever `content/nodes.md` currently says.
    fn a_runnable_node() -> (String, String) {
        let content = ContentIndex::shipped();
        let node = content
            .nodes
            .values()
            .find(|node| runnable_rung(content, &node.id).is_some())
            .expect("the shipped content has at least one runnable node");
        (node.id.clone(), node.title.clone())
    }

    #[test]
    fn a_name_nothing_knows_costs_one_question() {
        assert_eq!(
            Fixture::new().resolve("Stride pattern — bars 1–8", None),
            Resolution::Asking(Question::UserDrill),
            "the criterion sentence is where an unknown name goes (A4)"
        );
    }

    #[test]
    fn a_drill_the_user_already_answered_for_asks_nothing_ever_again() {
        let mut fixture = Fixture::new();
        fixture
            .drills
            .push(user_drill("d1", "Stride pattern — bars 1–8"));
        assert_eq!(
            fixture.resolve("stride pattern  bars 1-8", None),
            Resolution::Settled(BuiltTarget::UserDrill {
                drill_id: "d1".into()
            }),
            "resolution is paid once per item, ever (A2r)"
        );
    }

    #[test]
    fn a_journal_target_the_user_already_answered_for_asks_nothing_either() {
        let mut fixture = Fixture::new();
        fixture
            .journals
            .push(journal_item("j1", "Freer rubato in the intro"));
        assert_eq!(
            fixture.resolve("Freer rubato in the intro", None),
            Resolution::Settled(BuiltTarget::Journal {
                journal_id: "j1".into()
            })
        );
    }

    #[test]
    fn a_piece_costs_nothing_because_pieces_are_user_added_by_nature() {
        let mut fixture = Fixture::new();
        fixture
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        assert_eq!(
            fixture.resolve("alice in wonderland", None),
            Resolution::Settled(BuiltTarget::Piece {
                item_id: "p1".into()
            })
        );
    }

    #[test]
    fn picking_a_library_suggestion_settles_without_matching_text_at_all() {
        let mut fixture = Fixture::new();
        fixture
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        assert_eq!(
            fixture.resolve("something else entirely", Some("p1")),
            Resolution::Settled(BuiltTarget::Piece {
                item_id: "p1".into()
            }),
            "the shell reports which row was tapped; it decides nothing"
        );
    }

    #[test]
    fn a_library_exercise_still_costs_a_question_because_it_carries_no_gate() {
        let mut fixture = Fixture::new();
        fixture.items.push(sample_item(
            "e1",
            "Zzz private exercise",
            ItemKind::Exercise,
        ));
        assert_eq!(
            fixture.resolve("Zzz private exercise", None),
            Resolution::Asking(Question::UserDrill),
            "a library row is a name, not a countable criterion"
        );
    }

    #[test]
    fn an_authored_node_is_proposed_rather_than_assumed() {
        let (node_id, title) = a_runnable_node();
        let resolved = Fixture::new().resolve(&title, None);
        match resolved {
            Resolution::Asking(Question::NodeMatch { node, .. }) => assert_eq!(node, node_id),
            other => panic!("expected a proposed match, got {other:?}"),
        }
    }

    #[test]
    fn a_partial_name_still_reaches_the_node_it_meant() {
        let (node_id, title) = a_runnable_node();
        let shortened: String = title
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        if shortened.split_whitespace().count() < 2 {
            return; // nothing to shorten; the exact-match test already covers it
        }
        match Fixture::new().resolve(&shortened, None) {
            Resolution::Asking(Question::NodeMatch { node, .. }) => assert_eq!(node, node_id),
            other => panic!("a wrong guess must cost one tap, not a retype: {other:?}"),
        }
    }

    #[test]
    fn the_proposal_carries_the_players_words_not_the_engines() {
        let (_, title) = a_runnable_node();
        match Fixture::new().resolve(&title, None) {
            Resolution::Asking(Question::NodeMatch {
                evidence_line,
                gate_target,
                ..
            }) => {
                assert!(!evidence_line.is_empty());
                for engine_word in ["estimate", "evidence", "mastery", "prior", "band"] {
                    assert!(
                        !evidence_line.to_lowercase().contains(engine_word),
                        "T13: {engine_word:?} is engine vocabulary, not screen words"
                    );
                }
                assert!(gate_target >= 1, "dots need something to fill");
            }
            other => panic!("expected a proposed match, got {other:?}"),
        }
    }

    #[test]
    fn a_picked_id_the_library_no_longer_holds_falls_back_to_asking() {
        assert_eq!(
            Fixture::new().resolve("Alice in Wonderland", Some("gone")),
            Resolution::Asking(Question::UserDrill),
            "a stale selection must not settle into a block that resolves to nothing"
        );
    }

    #[test]
    fn names_that_differ_only_in_typing_are_the_same_name() {
        assert_eq!(
            normalised("Hanon №4, hands together"),
            "hanon 4 hands together"
        );
        assert_eq!(
            normalised("  HANON  4 — hands  together "),
            "hanon 4 hands together"
        );
        assert_eq!(normalised("!!!"), "");
    }
}

//! C3 — last night's reflection, back as a proposal (decision 12: propose,
//! confirm, never plan).
//!
//! v1 is rule-based, which settles open question 3 in `specs/built-session.md`.
//! The narrated version — an LLM reading the reflection and writing the offer —
//! is Phase 3 of the coach roadmap; what ships now quotes the user's own
//! sentence back and names a target it can actually resolve. Everything about
//! it is deliberately conservative: **a wrong quote-back is worse than none**,
//! so an ambiguous match proposes nothing rather than guessing.

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::compose::normalised;
use super::playthrough::named_sections;
use super::{BuiltTarget, JournalItem, Reflection, ReflectionKind, SteerState, UserDrill};
use crate::domain::item::{Item, ItemKind};

/// Older than this and "you said, last night" is not true any more.
const MAX_AGE_HOURS: i64 = 48;
/// Younger than this and the reflection is still the evening it was said in.
/// The morning card is a return, not an echo: quoting someone back to
/// themselves twenty minutes later is the app being strange at them.
const MIN_AGE_HOURS: i64 = 6;
/// How long an accepted steer rides today's plan. Today, approximated in hours
/// because the core is handed UTC and never the user's calendar.
const ACCEPTED_ACTIVE_HOURS: i64 = 24;
/// What one steer block is worth. A single concrete offer, per decision 12 —
/// the proposal never reshapes the rest of the session.
pub const STEER_MINUTES: u16 = 8;

/// What the resolver may name a target from. The same shape as composition's
/// context, minus everything a quote-back has no use for.
pub struct SteerContext<'a> {
    pub items: &'a [Item],
    pub user_drills: &'a [UserDrill],
    pub journal_items: &'a [JournalItem],
}

/// The morning card (C3). `quote` is the user's own sentence, verbatim: the
/// serif voice is reserved for the user's words, so the core hands them over
/// untouched rather than tidied.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ProposedSteer {
    pub reflection_id: String,
    pub quote: String,
    /// The one concrete offer, written by the core so the shell composes no
    /// sentence of its own.
    pub offer: String,
    pub minutes: u16,
}

/// A target the quote-back resolved to, and how it should read in the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SteerTarget {
    pub target: BuiltTarget,
    pub title: String,
    /// Where in the piece, when the sentence named a chart section.
    pub section: Option<String>,
}

/// The reflection worth proposing this morning, if there is one.
///
/// One at a time, and only the most recent: a queue of yesterdays is a nag, and
/// decision 12 allows one offer.
pub fn propose(
    reflections: &[Reflection],
    ctx: &SteerContext,
    now: DateTime<Utc>,
) -> Option<ProposedSteer> {
    let reflection = candidates(reflections, now)
        .filter(|reflection| reflection.steer == SteerState::Unoffered)
        .find(|reflection| target_of(reflection, ctx).is_some())?;
    let (quote, target) = quoted_target(reflection, ctx)?;
    Some(ProposedSteer {
        reflection_id: reflection.id.clone(),
        quote,
        offer: offer_line(&target),
        minutes: STEER_MINUTES,
    })
}

/// The steer already accepted, for as long as it is still today's.
///
/// Derived every time rather than banked as a block: the accepted state is one
/// column on the reflection, so a relaunch before the session rebuilds the same
/// block instead of losing it with the plan (a plan is remade, never recovered).
pub fn accepted(
    reflections: &[Reflection],
    ctx: &SteerContext,
    now: DateTime<Utc>,
) -> Option<SteerTarget> {
    let reflection = reflections
        .iter()
        .filter(|reflection| reflection.deleted_at.is_none())
        .filter(|reflection| reflection.steer == SteerState::Accepted)
        .filter(|reflection| {
            reflection
                .steer_at
                .is_some_and(|at| within(at, now, 0, ACCEPTED_ACTIVE_HOURS))
        })
        .max_by_key(|reflection| reflection.steer_at)?;
    target_of(reflection, ctx)
}

/// The target a named reflection resolves to, whatever its steer state — what
/// accepting has to be able to build a block from.
pub fn target_of(reflection: &Reflection, ctx: &SteerContext) -> Option<SteerTarget> {
    quoted_target(reflection, ctx).map(|(_, target)| target)
}

fn candidates(
    reflections: &[Reflection],
    now: DateTime<Utc>,
) -> impl Iterator<Item = &Reflection> + '_ {
    let mut ordered: Vec<&Reflection> = reflections
        .iter()
        .filter(|reflection| reflection.deleted_at.is_none())
        .filter(|reflection| reflection.kind == ReflectionKind::SessionClose)
        .filter(|reflection| within(reflection.at, now, MIN_AGE_HOURS, MAX_AGE_HOURS))
        .collect();
    ordered.sort_by_key(|reflection| std::cmp::Reverse(reflection.at));
    ordered.into_iter()
}

fn within(at: DateTime<Utc>, now: DateTime<Utc>, min_hours: i64, max_hours: i64) -> bool {
    let age = now - at;
    age >= TimeDelta::hours(min_hours) && age <= TimeDelta::hours(max_hours)
}

/// The first sentence of the transcript that names something the library can
/// resolve, and what it named.
fn quoted_target(reflection: &Reflection, ctx: &SteerContext) -> Option<(String, SteerTarget)> {
    let transcript = reflection.transcript.as_deref()?;
    sentences(transcript).find_map(|sentence| {
        resolve_sentence(&sentence, ctx).map(|target| (sentence.clone(), target))
    })
}

/// Sentence-ended, and nothing cleverer. A dash or a comma inside a thought is
/// still that thought, so only a full stop, a question mark or an exclamation
/// mark ends one.
fn sentences(transcript: &str) -> impl Iterator<Item = String> + '_ {
    transcript
        .split_inclusive(['.', '?', '!'])
        .map(|sentence| sentence.trim().to_string())
        .filter(|sentence| !sentence.is_empty())
}

/// What one sentence named, in the order of what the user has already paid a
/// resolution for. A name matching more than one thing resolves to nothing:
/// "the bridge" in a library with two pieces that have one is not an answer.
fn resolve_sentence(sentence: &str, ctx: &SteerContext) -> Option<SteerTarget> {
    let said = normalised(sentence);
    if said.is_empty() {
        return None;
    }

    let drills = ctx
        .user_drills
        .iter()
        .filter(|drill| drill.deleted_at.is_none() && mentions(&said, &drill.name))
        .map(|drill| SteerTarget {
            target: BuiltTarget::UserDrill {
                drill_id: drill.id.clone(),
            },
            title: drill.name.clone(),
            section: None,
        });
    if let Some(target) = only(drills) {
        return Some(target);
    }

    let journals = ctx
        .journal_items
        .iter()
        .filter(|journal| journal.deleted_at.is_none() && mentions(&said, &journal.name))
        .map(|journal| SteerTarget {
            target: BuiltTarget::Journal {
                journal_id: journal.id.clone(),
            },
            title: journal.name.clone(),
            section: None,
        });
    if let Some(target) = only(journals) {
        return Some(target);
    }

    let titles = ctx
        .items
        .iter()
        .filter(|item| mentions(&said, &item.title))
        .map(|item| piece_target(item, None));
    if let Some(target) = only(titles) {
        return Some(target);
    }

    // "The bridge still rushes" — a section names its piece, but only while
    // exactly one piece has a section by that name.
    let sections = ctx
        .items
        .iter()
        .filter(|item| item.kind == ItemKind::Piece)
        .flat_map(|item| {
            named_sections(item)
                .into_iter()
                .filter(|label| mentions(&said, label))
                .map(move |label| piece_target(item, Some(label)))
        });
    only(sections)
}

fn piece_target(item: &Item, section: Option<String>) -> SteerTarget {
    SteerTarget {
        target: BuiltTarget::Piece {
            item_id: item.id.clone(),
        },
        title: item.title.clone(),
        section,
    }
}

/// Word-boundary containment over normalised text: "bridge" must be a word the
/// sentence said, not a run of letters inside "bridges".
fn mentions(said: &str, name: &str) -> bool {
    let wanted = normalised(name);
    if wanted.is_empty() {
        return false;
    }
    let said_words: Vec<&str> = said.split(' ').collect();
    let wanted_words: Vec<&str> = wanted.split(' ').collect();
    said_words
        .windows(wanted_words.len())
        .any(|window| window == wanted_words.as_slice())
}

/// The one match, or none. Two candidates is an ambiguity the core refuses to
/// resolve on the user's behalf.
fn only<T>(mut candidates: impl Iterator<Item = T>) -> Option<T> {
    let first = candidates.next()?;
    candidates.next().is_none().then_some(first)
}

/// The offer, in plain peer voice: the morning card is a set-up surface, so it
/// may speak, but it still names no engine vocabulary and makes no claim about
/// what the session will do (T13).
fn offer_line(target: &SteerTarget) -> String {
    let what = match &target.section {
        Some(section) => format!("{}, {}", target.title, lower_first(section)),
        None => target.title.clone(),
    };
    format!("Give {what} {STEER_MINUTES} minutes today?")
}

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::built_session::tests_support::{journal_item, sample_item, user_drill};
    use crate::domain::chart::parse_chart;
    use crate::domain::item::Modality;
    use chrono::TimeZone;

    fn last_night() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 7, 22, 0, 0).unwrap()
    }

    fn morning() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 8, 9, 0, 0).unwrap()
    }

    fn reflection(transcript: &str) -> Reflection {
        Reflection {
            id: "01J00000000000000000REFLE".into(),
            kind: ReflectionKind::SessionClose,
            session_ref: None,
            transcript: Some(transcript.into()),
            audio_path: None,
            duration_s: Some(24),
            at: last_night(),
            steer: SteerState::Unoffered,
            steer_at: None,
            updated_at: last_night(),
            deleted_at: None,
        }
    }

    fn alice() -> Item {
        let mut item = sample_item(
            "01J0000000000000000000PIEC",
            "Alice in Wonderland",
            ItemKind::Piece,
        );
        item.chord_chart = parse_chart(
            "[A]\n| Cmaj7 | Dm7 |\n[Bridge]\n| Fmaj7 | G7 |",
            "C",
            Modality::Major,
        )
        .ok();
        item
    }

    fn ctx<'a>(
        items: &'a [Item],
        drills: &'a [UserDrill],
        journals: &'a [JournalItem],
    ) -> SteerContext<'a> {
        SteerContext {
            items,
            user_drills: drills,
            journal_items: journals,
        }
    }

    /// The brief's own transcript, read by the rules that ship.
    #[test]
    fn a_section_named_in_the_reflection_becomes_the_morning_offer() {
        let items = vec![alice()];
        let said = reflection(
            "Stride's nearly there at 72. The bridge still rushes when I go from memory \
             — that's the thing to hit next. And the rubato idea from the lesson works.",
        );
        let proposed = propose(&[said], &ctx(&items, &[], &[]), morning()).expect("an offer");
        assert_eq!(
            proposed.quote,
            "The bridge still rushes when I go from memory — that's the thing to hit next."
        );
        assert_eq!(
            proposed.offer,
            "Give Alice in Wonderland, bridge 8 minutes today?"
        );
        assert_eq!(proposed.minutes, STEER_MINUTES);
    }

    #[test]
    fn a_drill_the_user_already_named_wins_over_a_section() {
        let items = vec![alice()];
        let drills = vec![user_drill("01J0000000000000000000DRIL", "Stride")];
        let said = reflection("Stride's nearly there at 72. The bridge still rushes.");
        let proposed = propose(&[said], &ctx(&items, &drills, &[]), morning()).expect("an offer");
        assert_eq!(proposed.quote, "Stride's nearly there at 72.");
        assert_eq!(proposed.offer, "Give Stride 8 minutes today?");
    }

    #[test]
    fn a_journal_target_resolves_too() {
        let journals = vec![journal_item(
            "01J0000000000000000000JRNL",
            "Rubato in the intro",
        )];
        let said = reflection("Rubato in the intro is still not settled.");
        let proposed = propose(&[said], &ctx(&[], &[], &journals), morning()).expect("an offer");
        assert_eq!(proposed.offer, "Give Rubato in the intro 8 minutes today?");
    }

    #[test]
    fn a_name_two_things_answer_to_proposes_nothing() {
        let mut other = alice();
        other.id = "01J000000000000000000PIEC2".into();
        other.title = "Blue in Green".into();
        let items = vec![alice(), other];
        let said = reflection("The bridge still rushes when I go from memory.");
        assert!(propose(&[said], &ctx(&items, &[], &[]), morning()).is_none());
    }

    #[test]
    fn a_reflection_naming_nothing_the_library_knows_proposes_nothing() {
        let items = vec![alice()];
        let said = reflection("Good session. Hands felt loose for once.");
        assert!(propose(&[said], &ctx(&items, &[], &[]), morning()).is_none());
    }

    #[test]
    fn a_reflection_said_this_evening_is_not_quoted_back_the_same_evening() {
        let items = vec![alice()];
        let said = reflection("The bridge still rushes.");
        let an_hour_later = last_night() + TimeDelta::hours(1);
        assert!(propose(&[said], &ctx(&items, &[], &[]), an_hour_later).is_none());
    }

    #[test]
    fn a_reflection_from_last_week_is_not_last_night() {
        let items = vec![alice()];
        let said = reflection("The bridge still rushes.");
        let next_week = last_night() + TimeDelta::days(7);
        assert!(propose(&[said], &ctx(&items, &[], &[]), next_week).is_none());
    }

    #[test]
    fn a_declined_steer_leaves_no_trace_and_never_asks_again() {
        let items = vec![alice()];
        let said = Reflection {
            steer: SteerState::Declined,
            steer_at: Some(morning()),
            ..reflection("The bridge still rushes.")
        };
        assert!(propose(
            std::slice::from_ref(&said),
            &ctx(&items, &[], &[]),
            morning()
        )
        .is_none());
        assert!(accepted(&[said], &ctx(&items, &[], &[]), morning()).is_none());
    }

    #[test]
    fn an_accepted_steer_is_not_proposed_a_second_time() {
        let items = vec![alice()];
        let said = Reflection {
            steer: SteerState::Accepted,
            steer_at: Some(morning()),
            ..reflection("The bridge still rushes.")
        };
        assert!(propose(&[said], &ctx(&items, &[], &[]), morning()).is_none());
    }

    #[test]
    fn an_accepted_steer_rides_todays_plan_and_stops_riding_tomorrows() {
        let items = vec![alice()];
        let said = Reflection {
            steer: SteerState::Accepted,
            steer_at: Some(morning()),
            ..reflection("The bridge still rushes.")
        };
        let target = accepted(
            std::slice::from_ref(&said),
            &ctx(&items, &[], &[]),
            morning(),
        )
        .expect("a block");
        assert_eq!(
            target.target,
            BuiltTarget::Piece {
                item_id: "01J0000000000000000000PIEC".into()
            }
        );
        assert_eq!(target.section.as_deref(), Some("Bridge"));

        let tomorrow = morning() + TimeDelta::hours(25);
        assert!(accepted(&[said], &ctx(&items, &[], &[]), tomorrow).is_none());
    }

    #[test]
    fn the_most_recent_reflection_is_the_one_proposed() {
        let items = vec![alice()];
        let older = Reflection {
            id: "01J00000000000000000OLDER".into(),
            at: last_night() - TimeDelta::hours(24),
            ..reflection("The bridge still rushes.")
        };
        let newer = reflection("The bridge still rushes.");
        let proposed =
            propose(&[older, newer], &ctx(&items, &[], &[]), morning()).expect("an offer");
        assert_eq!(proposed.reflection_id, "01J00000000000000000REFLE");
    }

    #[test]
    fn a_voice_note_is_not_a_session_close_reflection() {
        let items = vec![alice()];
        let said = Reflection {
            kind: ReflectionKind::VoiceNote,
            ..reflection("The bridge still rushes.")
        };
        assert!(propose(&[said], &ctx(&items, &[], &[]), morning()).is_none());
    }

    #[test]
    fn a_word_inside_a_longer_word_is_not_a_mention() {
        assert!(mentions("the bridge still rushes", "bridge"));
        assert!(!mentions("the bridges still rush", "bridge"));
    }
}

//! The welcome and the four first-session steps (`specs/first-run.md`).

use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{AppEffect, Effect, Event};
use crate::domain::profile::Profile;
use crate::domain::session::{EntryStatus, PracticeSession, SessionStatus};
use crate::model::Model;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FirstRun {
    pub welcome_seen: bool,
}

impl FirstRun {
    /// The blob is positional bincode, so a build reads only a blob of its
    /// own shape. The shell names its storage key by this number (#2026).
    pub const BLOB_VERSION: u32 = 1;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FirstRunEvent {
    SkipWelcome,
    /// The shell replaying the stored blob at launch: no re-save.
    Loaded(FirstRun),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FirstRunView {
    pub shows_welcome: bool,
    pub shows_start_here: bool,
    pub added: bool,
    pub built: bool,
    pub played: bool,
    pub marked: bool,
    pub first_item_title: Option<String>,
}

pub fn handle_first_run_event(event: FirstRunEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        FirstRunEvent::SkipWelcome => Command::all([dismiss_welcome(model), render()]),
        FirstRunEvent::Loaded(first_run) => {
            model.first_run = first_run;
            render()
        }
    }
}

/// Saving a profile is the welcome's other way out, so the profile handler
/// calls this too.
pub fn dismiss_welcome(model: &mut Model) -> Command<Effect, Event> {
    if model.first_run.welcome_seen {
        return Command::done();
    }
    model.first_run.welcome_seen = true;
    Command::notify_shell(AppEffect::SaveFirstRun(model.first_run)).into()
}

fn render() -> Command<Effect, Event> {
    crux_core::render::render()
}

pub fn build_first_run_view(model: &Model) -> FirstRunView {
    let loaded = model.items_sync.has_loaded() && model.sessions_sync.has_loaded();
    let saved = || model.sessions.iter().chain(model.saving_session.iter());
    let added = !model.items.is_empty();
    let marked = saved().any(is_marked);
    FirstRunView {
        shows_welcome: loaded
            && !model.first_run.welcome_seen
            && !added
            && model.sessions.is_empty()
            && model.profile == Profile::default(),
        shows_start_here: loaded && !marked,
        added,
        built: matches!(
            model.session_status,
            SessionStatus::Active(_) | SessionStatus::Summary(_)
        ) || saved().next().is_some(),
        played: saved().any(|s| s.entries.iter().any(|e| e.status == EntryStatus::Completed)),
        marked,
        first_item_title: model
            .items
            .iter()
            .min_by_key(|item| item.created_at)
            .map(|item| item.title.clone()),
    }
}

fn is_marked(session: &PracticeSession) -> bool {
    session.session_score.is_some()
        || session
            .entries
            .iter()
            .any(|e| e.plays.iter().any(|p| p.score.is_some()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Intrada;
    use crate::domain::item::{Item, ItemKind};
    use crate::domain::profile::ProfileEvent;
    use crate::domain::session::{CompletionStatus, SetlistEntry, VariationPlay};
    use crate::domain::types::assert_round_trips;
    use crate::persistence::PersistenceOutput;
    use chrono::{DateTime, TimeZone, Utc};
    use crux_core::App;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).single().expect("valid instant")
    }

    fn item(id: &str, title: &str, created: i64) -> Item {
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: at(created),
            updated_at: at(created),
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        }
    }

    fn session(entries: Vec<SetlistEntry>, session_score: Option<u8>) -> PracticeSession {
        PracticeSession {
            id: "s1".to_string(),
            entries,
            session_notes: None,
            started_at: at(0),
            completed_at: at(600),
            total_duration_secs: 600,
            completion_status: CompletionStatus::Completed,
            session_score,
        }
    }

    fn entry(status: EntryStatus, score: Option<u8>) -> SetlistEntry {
        SetlistEntry {
            status,
            plays: vec![VariationPlay {
                score,
                ..VariationPlay::fixture()
            }],
            ..SetlistEntry::fixture()
        }
    }

    /// Both lists answered by the store, as at a real launch.
    fn launched(items: Vec<Item>, sessions: Vec<PracticeSession>) -> Model {
        let mut model = Model::default();
        let _ = Intrada.update(Event::StartApp, &mut model);
        let _ = Intrada.update(
            Event::StoreLoaded(PersistenceOutput::Items(items)),
            &mut model,
        );
        let _ = Intrada.update(
            Event::SessionsStoreLoaded(PersistenceOutput::Sessions(sessions)),
            &mut model,
        );
        model
    }

    fn view(model: &Model) -> FirstRunView {
        Intrada.view(model).first_run
    }

    fn emits_save(cmd: &mut Command<Effect, Event>) -> Option<FirstRun> {
        cmd.effects().find_map(|e| match e {
            Effect::App(req) => match &req.operation {
                AppEffect::SaveFirstRun(f) => Some(*f),
                _ => None,
            },
            _ => None,
        })
    }

    // ── The four steps ──

    type Case = (&'static str, Vec<Item>, Vec<PracticeSession>, [bool; 4]);

    #[test]
    fn the_steps_walk_from_an_empty_app_to_a_marked_session() {
        let piece = || vec![item("p1", "Clair de Lune", 0)];
        let unplayed = session(vec![entry(EntryStatus::NotAttempted, None)], None);
        let played = session(vec![entry(EntryStatus::Completed, None)], None);
        let play_marked = session(vec![entry(EntryStatus::Completed, Some(7))], None);
        let session_marked = session(vec![entry(EntryStatus::Completed, None)], Some(6));
        let cases: [Case; 6] = [
            ("empty", vec![], vec![], [false, false, false, false]),
            ("a piece", piece(), vec![], [true, false, false, false]),
            (
                "ended before any play",
                piece(),
                vec![unplayed],
                [true, true, false, false],
            ),
            ("played", piece(), vec![played], [true, true, true, false]),
            (
                "a play marked",
                piece(),
                vec![play_marked],
                [true, true, true, true],
            ),
            (
                "the session marked",
                piece(),
                vec![session_marked],
                [true, true, true, true],
            ),
        ];
        for (name, items, sessions, expected) in cases {
            let v = view(&launched(items, sessions));
            assert_eq!([v.added, v.built, v.played, v.marked], expected, "{name}");
            assert_eq!(
                v.shows_start_here, !expected[3],
                "{name}: the card stays until a mark"
            );
        }
    }

    #[test]
    fn starting_todays_plan_ticks_built_without_the_builder() {
        let mut piece = item("p1", "Clair de Lune", 0);
        piece.linked_exercise_ids = vec!["ex1".to_string()];
        let exercise = Item {
            kind: ItemKind::Exercise,
            ..item("ex1", "Scales", 0)
        };
        let mut model = launched(vec![piece, exercise], vec![]);
        crate::view::cache::refresh(&mut model, Utc::now());
        let _ = Intrada.update(
            Event::Session(crate::domain::session::SessionEvent::StartFromSuggestion {
                now: Utc::now(),
            }),
            &mut model,
        );
        assert!(matches!(model.session_status, SessionStatus::Active(_)));
        assert!(view(&model).built);
    }

    #[test]
    fn a_half_built_setlist_is_not_built() {
        let mut model = launched(vec![item("p1", "Clair de Lune", 0)], vec![]);
        let _ = Intrada.update(
            Event::Session(crate::domain::session::SessionEvent::StartBuildingWith {
                item_id: "p1".to_string(),
            }),
            &mut model,
        );
        assert!(matches!(model.session_status, SessionStatus::Building(_)));
        assert!(
            !view(&model).built,
            "the builder is not saved, so the tick would not survive"
        );
    }

    #[test]
    fn the_first_item_is_the_earliest_added() {
        let model = launched(
            vec![item("p2", "Berceuse", 200), item("p1", "Arabesque", 100)],
            vec![],
        );
        assert_eq!(view(&model).first_item_title.as_deref(), Some("Arabesque"));
        assert_eq!(view(&launched(vec![], vec![])).first_item_title, None);
    }

    // ── The welcome ──

    #[test]
    fn a_fresh_install_shows_the_welcome_once_both_lists_have_loaded() {
        let mut model = Model::default();
        let _ = Intrada.update(Event::StartApp, &mut model);
        assert!(
            !view(&model).shows_welcome,
            "never before the store answers"
        );
        assert!(!view(&model).shows_start_here);
        let _ = Intrada.update(
            Event::StoreLoaded(PersistenceOutput::Items(vec![])),
            &mut model,
        );
        assert!(
            !view(&model).shows_welcome,
            "the history has not answered yet"
        );
        let _ = Intrada.update(
            Event::SessionsStoreLoaded(PersistenceOutput::Sessions(vec![])),
            &mut model,
        );
        assert!(view(&model).shows_welcome);
        assert!(view(&model).shows_start_here);
    }

    #[test]
    fn a_failed_load_keeps_the_welcome_hidden() {
        let mut model = Model::default();
        let _ = Intrada.update(Event::StartApp, &mut model);
        let _ = Intrada.update(Event::StoreLoaded(PersistenceOutput::Failed), &mut model);
        let _ = Intrada.update(
            Event::SessionsStoreLoaded(PersistenceOutput::Sessions(vec![])),
            &mut model,
        );
        assert!(!view(&model).shows_welcome);
    }

    #[test]
    fn a_musician_who_already_uses_the_app_never_sees_the_welcome() {
        let with_item = launched(vec![item("p1", "Clair de Lune", 0)], vec![]);
        assert!(!view(&with_item).shows_welcome, "a library with an item");

        let with_session = launched(vec![], vec![session(vec![], None)]);
        assert!(
            !view(&with_session).shows_welcome,
            "a history with a session"
        );

        let mut with_profile = launched(vec![], vec![]);
        let _ = Intrada.update(
            Event::Profile(ProfileEvent::Loaded(Profile {
                name: "Jon".to_string(),
                ..Profile::default()
            })),
            &mut with_profile,
        );
        assert!(!view(&with_profile).shows_welcome, "a saved profile");
    }

    #[test]
    fn skip_dismisses_the_welcome_and_saves_once() {
        let mut model = launched(vec![], vec![]);
        let mut cmd = Intrada.update(Event::FirstRun(FirstRunEvent::SkipWelcome), &mut model);
        assert_eq!(emits_save(&mut cmd), Some(FirstRun { welcome_seen: true }));
        assert!(!view(&model).shows_welcome);
        let mut again = Intrada.update(Event::FirstRun(FirstRunEvent::SkipWelcome), &mut model);
        assert_eq!(emits_save(&mut again), None);
    }

    #[test]
    fn saving_a_profile_dismisses_the_welcome() {
        let mut model = launched(vec![], vec![]);
        let mut cmd = Intrada.update(
            Event::Profile(ProfileEvent::Save(Profile::default())),
            &mut model,
        );
        assert_eq!(emits_save(&mut cmd), Some(FirstRun { welcome_seen: true }));
        assert!(!view(&model).shows_welcome);
    }

    #[test]
    fn a_refused_profile_save_leaves_the_welcome() {
        let mut model = launched(vec![], vec![]);
        let too_long = Profile {
            name: "n".repeat(crate::validation::MAX_PROFILE_NAME + 1),
            ..Profile::default()
        };
        let mut cmd = Intrada.update(Event::Profile(ProfileEvent::Save(too_long)), &mut model);
        assert_eq!(emits_save(&mut cmd), None);
        assert!(view(&model).shows_welcome);
    }

    #[test]
    fn loaded_restores_the_dismissal_without_re_saving() {
        let mut model = launched(vec![], vec![]);
        let mut cmd = Intrada.update(
            Event::FirstRun(FirstRunEvent::Loaded(FirstRun { welcome_seen: true })),
            &mut model,
        );
        assert_eq!(
            emits_save(&mut cmd),
            None,
            "restore never re-writes the blob"
        );
        assert!(!view(&model).shows_welcome);
    }

    // ── Wire ──

    const PINNED_FIRST_RUN_HEX: &str = "01";
    const PINNED_BLOB_VERSION: u32 = 1;

    /// Positional bincode: a new field breaks every stored blob (#1345). The
    /// shell's key follows `BLOB_VERSION` (#2026); the failure message is the protocol.
    #[test]
    fn first_run_blob_wire_is_pinned() {
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
        let first_run = FirstRun { welcome_seen: true };
        let mut bytes = Vec::new();
        BincodeFfiFormat::serialize(&mut bytes, &first_run).expect("serialize");
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            (FirstRun::BLOB_VERSION, hex.as_str()),
            (PINNED_BLOB_VERSION, PINNED_FIRST_RUN_HEX),
            "the first-run blob changed shape: bump FirstRun::BLOB_VERSION and PINNED_BLOB_VERSION, then re-pin the hex. Never only the hex."
        );
        let back: FirstRun =
            BincodeFfiFormat::deserialize(&bytes).expect("must decode on the FFI wire (#846)");
        assert_eq!(back, first_run);
    }

    #[test]
    fn first_run_events_round_trip_on_the_ffi_wire() {
        assert_round_trips(Event::FirstRun(FirstRunEvent::SkipWelcome));
        assert_round_trips(Event::FirstRun(FirstRunEvent::Loaded(FirstRun {
            welcome_seen: true,
        })));
        assert_round_trips(AppEffect::SaveFirstRun(FirstRun { welcome_seen: true }));
        assert_round_trips(FirstRunView {
            shows_welcome: true,
            shows_start_here: true,
            added: true,
            built: false,
            played: true,
            marked: false,
            first_item_title: Some("Clair de Lune".to_string()),
        });
    }
}

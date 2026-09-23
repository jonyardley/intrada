// The `#[effect]` macro generates large variant size differences and we can't Box through it.
#![allow(clippy::large_enum_variant)]

use crux_core::capability::Operation;
use crux_core::macros::effect;
use crux_core::render::RenderOperation;
use crux_core::{App, Command};
use serde::{Deserialize, Serialize};

use crate::domain::item::{handle_item_event, ItemEvent};
use crate::domain::profile::{handle_profile_event, Profile, ProfileEvent};
use crate::domain::session::{handle_session_event, ActiveSession, SessionEvent};
use crate::domain::types::{LibrarySort, ListQuery};
use crate::model::{Model, ViewModel};
use crate::persistence::{self, PersistenceOperation, PersistenceOutput};
use crate::recognition::{self, RecognitionOperation, RecognitionOutput};
use crate::view::library::build_practice_summaries;

/// Root Crux application for the music practice library.
#[derive(Default)]
pub struct Intrada;

/// All events the application can process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Event {
    // ── Lifecycle ────────────────────────────────────────────────────
    /// Hydrate the library and sessions from the on-device store at launch.
    /// Named `StartApp` (not `Init`) to avoid Swift keyword collision.
    StartApp,
    /// Shell reports the device's UTC offset (minutes east of UTC) at launch
    /// and on foreground, so analytics turn days over at the user's midnight,
    /// not UTC's (#1330).
    SetUtcOffset {
        minutes: i32,
    },
    /// Demo dataset, opt-in only (e.g. iOS `--seed-sample-data`) — never in production.
    LoadSampleData,

    // ── Domain ──────────────────────────────────────────────────────
    Item(ItemEvent),
    Session(SessionEvent),
    Profile(ProfileEvent),

    // ── Error handling ──────────────────────────────────────────────
    ClearError,
    /// Dismiss the neutral banner (#1325). Independent of `ClearError`.
    ClearNotice,
    SetQuery(Option<ListQuery>),
    /// User chose a library sort order; persist it and re-render.
    SetSort(LibrarySort),

    // ── Local-first persistence ──────────────────────────────────────
    HydrateFromStore,
    StoreLoaded(PersistenceOutput),
    /// Write result (split from `StoreLoaded` so a failed write reloads without looping — #825).
    StoreWritten(PersistenceOutput),
    SessionsStoreLoaded(PersistenceOutput),
    /// Session write result, kept separate so a failed save reloads sessions (not items).
    SessionStoreWritten(PersistenceOutput),

    // ── On-device recognition ────────────────────────────────────────
    /// The shell read the page. Carries the `photo_id` it was asked to read, so
    /// a result can only ever land on the read that asked for it — two scans in
    /// flight would otherwise show one page beside the other's fields.
    /// `Unsupported` and `Failed` are outcomes the form shows, not errors that
    /// mute the banner.
    PhotoRead {
        photo_id: String,
        output: RecognitionOutput,
    },
    /// The user finished with (or backed out of) the recognised draft.
    DiscardPhotoDraft,
}

/// Side effects the core requests from shells.
///
/// Variants hold operation types; the `#[effect]` macro wraps each in
/// `Request<Op>` in the compiled enum.
#[effect(facet_typegen)]
pub enum Effect {
    Render(RenderOperation),
    /// Shell-only side effects that are not relational persistence.
    App(AppEffect),
    /// Local-first persistence (the core's first effect with typed-data output).
    Persistence(PersistenceOperation),
    /// On-device page recognition. The shell runs the frameworks and returns
    /// text with geometry; the core decides what it means (spec decision 4).
    Recognition(RecognitionOperation),
}

/// Singleton side-effect operations handled by the shell (UserDefaults).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum AppEffect {
    /// Persist the active session to localStorage for crash recovery (FR-008).
    SaveSessionInProgress(ActiveSession),
    /// Clear the active session from localStorage.
    ClearSessionInProgress,
    /// Persist the chosen library sort order (small singleton — UserDefaults
    /// on iOS / localStorage on web). Fire-and-forget; output is `()`.
    SaveLibrarySort(LibrarySort),
    /// Persist the musician's profile (UserDefaults, key versioned per
    /// `specs/profile.md`). Fire-and-forget; output is `()`.
    SaveProfile(Profile),
}

impl Operation for AppEffect {
    type Output = ();
}

impl App for Intrada {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
    ) -> Command<Self::Effect, Self::Event> {
        // Before the handler, not after: a target only ever belongs to the
        // error the event in hand reported (#1595).
        model.last_error_target = None;
        self.handle_event(event, model)
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        self.build_view(model)
    }
}

impl Intrada {
    fn handle_event(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            // ── Lifecycle ────────────────────────────────────────────
            Event::StartApp => {
                Command::all([persistence::load_items(), persistence::load_sessions()])
            }
            Event::SetUtcOffset { minutes } => {
                model.utc_offset_minutes = minutes;
                crux_core::render::render()
            }
            Event::LoadSampleData => {
                model.items = crate::sample::sample_items();
                model.sessions = crate::sample::sample_sessions();
                model.practice_summaries = build_practice_summaries(&model.sessions);
                model.last_error = None;
                crux_core::render::render()
            }

            // ── Domain handlers ──────────────────────────────────────
            Event::Item(item_event) => handle_item_event(item_event, model),
            Event::Session(session_event) => handle_session_event(session_event, model),
            Event::Profile(profile_event) => handle_profile_event(profile_event, model),

            // ── Error handling ───────────────────────────────────────
            Event::ClearError => {
                model.dismiss_error();
                crux_core::render::render()
            }
            Event::ClearNotice => {
                model.clear_notice();
                crux_core::render::render()
            }
            Event::SetQuery(query) => {
                model.active_query = query;
                crux_core::render::render()
            }
            Event::SetSort(sort) => {
                model.active_sort = sort;
                Command::all([
                    Command::notify_shell(AppEffect::SaveLibrarySort(sort)).into(),
                    crux_core::render::render(),
                ])
            }

            // ── Local-first persistence ──────────────────────────────
            Event::HydrateFromStore => persistence::load_items(),
            Event::StoreLoaded(output) => match output {
                PersistenceOutput::Items(items) => {
                    model.items = items;
                    crux_core::render::render()
                }
                PersistenceOutput::Ack | PersistenceOutput::Sessions(_) => Command::done(),
                // Failed read: surface only — no reload (would loop a broken store).
                PersistenceOutput::Failed => {
                    model.surface_error("Couldn't access local storage.");
                    crux_core::render::render()
                }
            },
            Event::StoreWritten(output) => match output {
                PersistenceOutput::Ack => {
                    model.record_ack();
                    Command::done()
                }
                PersistenceOutput::Items(_) | PersistenceOutput::Sessions(_) => Command::done(),
                // Failed write → reload to roll back the un-persisted change (#825).
                PersistenceOutput::Failed => {
                    model.surface_error("Couldn't access local storage.");
                    persistence::load_items()
                }
            },
            Event::SessionsStoreLoaded(output) => match output {
                PersistenceOutput::Sessions(sessions) => {
                    model.sessions = sessions;
                    model.practice_summaries = build_practice_summaries(&model.sessions);
                    crux_core::render::render()
                }
                PersistenceOutput::Items(_) | PersistenceOutput::Ack => Command::done(),
                PersistenceOutput::Failed => {
                    model.surface_error("Couldn't access local storage.");
                    crux_core::render::render()
                }
            },
            Event::SessionStoreWritten(output) => match output {
                PersistenceOutput::Ack => {
                    model.record_ack();
                    crate::domain::session::save_acknowledged(model)
                }
                PersistenceOutput::Items(_) | PersistenceOutput::Sessions(_) => Command::done(),
                PersistenceOutput::Failed => crate::domain::session::save_refused(model)
                    .unwrap_or_else(|| {
                        model.surface_error("Couldn't access local storage.");
                        persistence::load_sessions()
                    }),
            },

            // ── On-device recognition ────────────────────────────────
            Event::PhotoRead { photo_id, output } => {
                // Anything but "still reading this exact photo" is a result
                // nothing is waiting on — a scan the user backed out of, or one
                // superseded by a later scan. Leave the state untouched: the
                // read that *is* current must survive its predecessor landing.
                if model.photo_recognition
                    != (crate::model::PhotoRecognition::Reading {
                        photo_id: photo_id.clone(),
                    })
                {
                    return crux_core::render::render();
                }

                model.photo_recognition = match output {
                    RecognitionOutput::Page(page) => crate::model::PhotoRecognition::Ready {
                        photo_id,
                        draft: recognition::read_fields(&page),
                    },
                    RecognitionOutput::Unsupported => {
                        crate::model::PhotoRecognition::Unsupported { photo_id }
                    }
                    RecognitionOutput::Failed => {
                        crate::model::PhotoRecognition::Failed { photo_id }
                    }
                };
                crux_core::render::render()
            }
            Event::DiscardPhotoDraft => {
                model.photo_recognition = crate::model::PhotoRecognition::Idle;
                crux_core::render::render()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::{Item, ItemKind};
    use crate::domain::session::VariationPlay;
    use crate::domain::session::{
        CompletionStatus, EntryStatus, PracticeSession, SessionStatus, SetlistEntry,
    };
    use crate::domain::types::{SortDirection, SortField};
    use crate::model::{ItemPracticeSummary, LibraryItemView};
    use crate::view::library::{
        build_exercise_usage, build_variant_score_index, build_variant_views, derive_priorities,
    };

    #[test]
    fn charted_piece_surfaces_a_scaffold_preview_in_the_view() {
        use crate::domain::item::ItemEvent;
        let app = Intrada;
        let mut model = Model::default();

        let now = chrono::Utc::now();
        model.items.push(Item {
            id: "p1".to_string(),
            title: "Autumn Leaves".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: Some("G".to_string()),
            modality: Some(crate::domain::item::Modality::Minor),
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        let _ = app.update(
            Event::Item(ItemEvent::SetChordChart {
                piece_id: "p1".to_string(),
                raw_chart: "| Cm7 | F7 | Bbmaj7 | Ebmaj7 |".to_string(),
            }),
            &mut model,
        );

        let vm = app.view(&model);
        let piece = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let preview = piece
            .scaffold_preview
            .as_ref()
            .expect("charted piece has a preview");
        assert_eq!(preview.key, "G");
        assert_eq!(preview.specs.len(), 5);
        assert_eq!(preview.specs[0].title, "Learn the melody");
        assert_eq!(preview.fallback_total, 0);

        // An un-charted exercise has no preview.
        let uncharted = vm.items.iter().find(|i| i.id != "p1");
        assert!(uncharted
            .map(|i| i.scaffold_preview.is_none())
            .unwrap_or(true));
    }

    #[test]
    fn test_clear_error() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("some error".to_string()),
            ..Default::default()
        };

        let _cmd = app.update(Event::ClearError, &mut model);

        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_view_empty_model() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);

        assert!(vm.items.is_empty());
        assert_eq!(vm.items.len(), 0);
        assert!(vm.error.is_none());
    }

    #[test]
    fn test_delete_after_seed_does_not_panic() {
        let app = Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::LoadSampleData, &mut model);
        let id = model.items[0].id.clone();

        let _ = app.update(
            Event::Item(ItemEvent::Delete { id: id.clone() }),
            &mut model,
        );

        assert!(!model.items.iter().any(|i| i.id == id));
    }

    #[test]
    fn test_load_sample_data_populates_pieces_and_exercises() {
        let app = Intrada;
        let mut model = Model::default();

        let _ = app.update(Event::LoadSampleData, &mut model);

        assert!(model.items.len() >= 4, "expected a few sample items");
        assert!(model.items.iter().any(|i| i.kind == ItemKind::Piece));
        assert!(model.items.iter().any(|i| i.kind == ItemKind::Exercise));
        // At least one carries structured tempo so the card's ♩ = bpm shows.
        assert!(model
            .items
            .iter()
            .any(|i| i.tempo.as_ref().and_then(|t| t.bpm).is_some()));
    }

    #[test]
    fn load_sample_data_gives_scales_a_step_ladder_with_progress() {
        let app = Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::LoadSampleData, &mut model);

        let vm = app.view(&model);
        let scales = vm.items.iter().find(|i| i.id == "sample-scales").unwrap();
        assert!(
            scales.variants.len() >= 3,
            "the demo exercise carries a keys ladder"
        );
        let first = &scales.variants[0];
        assert!(first.is_solid, "the first demo variation reads as Solid");
        assert!(first.latest_score.is_some());
    }

    #[test]
    fn test_load_sample_data_populates_practice_sessions() {
        use crate::domain::session::CompletionStatus;

        let app = Intrada;
        let mut model = Model::default();

        let _ = app.update(Event::LoadSampleData, &mut model);

        assert!(
            model.sessions.len() >= 3,
            "expected a few sample practice sessions"
        );
        // Every session has at least one entry referencing a seeded item, so the
        // home-screen "duration · item count" line is never zero.
        assert!(model.sessions.iter().all(|s| !s.entries.is_empty()));
        let item_ids: std::collections::HashSet<_> =
            model.items.iter().map(|i| i.id.as_str()).collect();
        assert!(model.sessions.iter().all(|s| s
            .entries
            .iter()
            .all(|e| item_ids.contains(e.item_id.as_str()))));
        // Both completion states are represented so the card can show each.
        assert!(model
            .sessions
            .iter()
            .any(|s| s.completion_status == CompletionStatus::Completed));
        assert!(model
            .sessions
            .iter()
            .any(|s| s.completion_status == CompletionStatus::EndedEarly));

        // The view projects them with a human-readable duration + entries.
        let vm = app.view(&model);
        assert_eq!(vm.sessions.len(), model.sessions.len());
        assert!(vm
            .sessions
            .iter()
            .all(|s| !s.total_duration_display.is_empty() && !s.entries.is_empty()));
    }

    #[test]
    fn view_carries_the_photo_id_through_to_the_screens() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let item = Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            metre: None,
        };
        let model = Model {
            items: vec![item],
            ..Model::default()
        };

        let view = app.view(&model);

        assert_eq!(
            view.items[0].photo_id.as_deref(),
            Some("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            "the screens read the ViewModel, never the core Item"
        );
    }

    #[test]
    fn test_view_with_items() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            items: vec![
                Item {
                    id: "p1".to_string(),
                    title: "Sonata".to_string(),
                    kind: ItemKind::Piece,
                    composer: Some("Beethoven".to_string()),
                    key: None,
                    modality: None,
                    tempo: Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(132),
                    }),
                    notes: None,
                    tags: vec!["classical".to_string()],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "p2".to_string(),
                    title: "Etude".to_string(),
                    kind: ItemKind::Piece,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: Some(crate::domain::types::Tempo {
                        marking: None,
                        bpm: Some(96),
                    }),
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "p3".to_string(),
                    title: "Nocturne".to_string(),
                    kind: ItemKind::Piece,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: Some(crate::domain::types::Tempo {
                        marking: Some("Largo".to_string()),
                        bpm: None,
                    }),
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "e1".to_string(),
                    title: "Scales".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
            ],
            ..Default::default()
        };

        let vm = app.view(&model);

        assert_eq!(vm.items.len(), 4);

        // Check piece — keeps the flattened string (web) AND exposes structured
        // marking + bpm so the iOS card can render "Allegro · ♩ = 132".
        let piece_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        assert_eq!(piece_view.item_type, ItemKind::Piece);
        assert_eq!(piece_view.title, "Sonata");
        assert_eq!(piece_view.subtitle, "Beethoven");
        assert_eq!(piece_view.tempo, Some("Allegro (132 BPM)".to_string()));
        assert_eq!(piece_view.tempo_marking, Some("Allegro".to_string()));
        assert_eq!(piece_view.tempo_bpm, Some(132));
        assert_eq!(piece_view.tags, vec!["classical".to_string()]);

        // bpm-only item: marking and bpm pass through independently.
        let etude_view = vm.items.iter().find(|i| i.id == "p2").unwrap();
        assert_eq!(etude_view.tempo_marking, None);
        assert_eq!(etude_view.tempo_bpm, Some(96));

        // marking-only item.
        let nocturne_view = vm.items.iter().find(|i| i.id == "p3").unwrap();
        assert_eq!(nocturne_view.tempo_marking, Some("Largo".to_string()));
        assert_eq!(nocturne_view.tempo_bpm, None);
        assert_eq!(nocturne_view.tempo, Some("Largo".to_string()));

        // Check exercise — no tempo at all.
        let ex_view = vm.items.iter().find(|i| i.id == "e1").unwrap();
        assert_eq!(ex_view.item_type, ItemKind::Exercise);
        assert_eq!(ex_view.title, "Scales");
        assert_eq!(ex_view.subtitle, "");
        assert_eq!(ex_view.tempo_marking, None);
        assert_eq!(ex_view.tempo_bpm, None);
    }

    #[test]
    fn test_view_shows_error() {
        let app = Intrada;
        let model = Model {
            last_error: Some("Something went wrong".to_string()),
            ..Default::default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.error, Some("Something went wrong".to_string()));
    }

    // --- Query filtering in core ---

    #[test]
    fn test_set_query_filters_by_type() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });
        model.items.push(Item {
            id: "e1".to_string(),
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 2);

        let _cmd = app.update(
            Event::SetQuery(Some(ListQuery {
                item_type: Some(ItemKind::Piece),
                ..Default::default()
            })),
            &mut model,
        );
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].item_type, ItemKind::Piece);

        let _cmd = app.update(Event::SetQuery(None), &mut model);
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 2);
    }

    #[test]
    fn set_sort_updates_model_and_emits_save_effect() {
        let app = Intrada;
        let mut model = Model::default();

        let sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let mut cmd = app.update(Event::SetSort(sort), &mut model);

        assert_eq!(model.active_sort, sort, "model sort is updated");
        assert!(
            cmd.effects().any(|e| matches!(e, Effect::App(req)
                if req.operation == AppEffect::SaveLibrarySort(sort))),
            "SetSort emits SaveLibrarySort with the chosen order"
        );
    }

    #[test]
    fn test_set_query_filters_by_text() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Moonlight Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });
        model.items.push(Item {
            id: "p2".to_string(),
            title: "Clair de Lune".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        model.active_query = Some(ListQuery {
            text: Some("beethoven".to_string()),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Moonlight Sonata");
    }

    #[test]
    fn test_set_query_filters_by_tags() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec!["classical".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });
        model.items.push(Item {
            id: "p2".to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec!["romantic".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        model.active_query = Some(ListQuery {
            tags: vec!["classical".to_string()],
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Sonata");
    }

    #[test]
    fn test_query_tags_match_any_not_all() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mk = |id: &str, title: &str, tags: &[&str]| Item {
            id: id.to_string(),
            title: title.to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        model.items = vec![
            mk("a", "Bebop", &["jazz"]),
            mk("b", "Nocturne", &["classical"]),
            mk("c", "Riff", &["rock"]),
        ];
        // "studying classical and jazz" → the union, not the (empty) intersection.
        model.active_query = Some(ListQuery {
            tags: vec!["classical".to_string(), "jazz".to_string()],
            ..Default::default()
        });

        let titles: Vec<String> = app
            .view(&model)
            .items
            .iter()
            .map(|i| i.title.clone())
            .collect();
        assert_eq!(titles.len(), 2, "OR semantics: matches classical OR jazz");
        assert!(titles.contains(&"Bebop".to_string()));
        assert!(titles.contains(&"Nocturne".to_string()));
    }

    #[test]
    fn view_exposes_sorted_unique_available_tags() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mk = |id: &str, tags: &[&str]| Item {
            id: id.to_string(),
            title: id.to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        model.items = vec![mk("a", &["Jazz", "piano"]), mk("b", &["classical", "jazz"])];
        // Case-insensitive dedupe (first-seen casing), sorted by lowercase — the
        // whole vocabulary, independent of the active filter.
        model.active_query = Some(ListQuery {
            tags: vec!["classical".to_string()],
            ..Default::default()
        });

        let vm = app.view(&model);
        assert_eq!(vm.available_tags, vec!["classical", "Jazz", "piano"]);
    }

    #[test]
    fn available_composers_span_whole_library_under_active_filter() {
        // Regression (mirrors available_tags, #851): the composer pool must stay
        // the full-library vocabulary when filtered to a composer-less type.
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mk = |id: &str, kind: ItemKind, composer: Option<&str>| Item {
            id: id.to_string(),
            title: id.to_string(),
            kind,
            composer: composer.map(str::to_string),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        model.items = vec![
            mk("p1", ItemKind::Piece, Some("Chopin")),
            mk("p2", ItemKind::Piece, Some("Beethoven")),
            mk("p3", ItemKind::Piece, Some("chopin")),
            mk("p4", ItemKind::Piece, Some("  Ravel  ")),
            mk("e1", ItemKind::Exercise, None),
        ];
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            ..Default::default()
        });

        let vm = app.view(&model);
        // Whole-library vocabulary: case-folded dedupe (first-seen "Chopin"), trimmed.
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.available_composers, vec!["Beethoven", "Chopin", "Ravel"]);
    }

    // --- Free-text normalisation on add (#883) ---

    #[test]
    fn add_normalises_whitespace_composer() {
        let app = Intrada;
        let mut model = Model::default();

        // Whitespace-only composer on an exercise stores as None, not "   ".
        let _ = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "  Scales  ".to_string(),
                kind: ItemKind::Exercise,
                composer: Some("   ".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec!["  warm-up ".to_string()],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Scales");
        assert_eq!(model.items[0].composer, None);
        assert_eq!(model.items[0].tags, vec!["warm-up".to_string()]);

        // A padded composer is trimmed, not stored verbatim.
        let _ = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "Hanon".to_string(),
                kind: ItemKind::Exercise,
                composer: Some("  Hanon ".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );
        assert_eq!(model.items[1].composer, Some("Hanon".to_string()));
    }

    #[test]
    fn add_piece_with_blank_composer_normalizes_to_none() {
        let app = Intrada;
        let mut model = Model::default();

        let _ = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "Sonata".to_string(),
                kind: ItemKind::Piece,
                composer: Some("   ".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );

        assert_eq!(model.items.len(), 1, "a blank composer is not a rejection");
        assert_eq!(model.items[0].composer, None);
    }

    // --- T042: Unicode handling in core ---

    #[test]
    fn test_unicode_in_item_add() {
        let app = Intrada;
        let mut model = Model::default();

        let _cmd = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "Ménuet en Sol".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Dvořák".to_string()),
                key: Some("ré mineur".to_string()),
                modality: None,
                tempo: None,
                notes: Some("Pièce très jolie — «superbe»".to_string()),
                tags: vec!["日本語タグ".to_string()],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Ménuet en Sol");
        assert_eq!(model.items[0].composer, Some("Dvořák".to_string()));
        assert_eq!(model.items[0].key, Some("ré mineur".to_string()));
        assert_eq!(
            model.items[0].notes,
            Some("Pièce très jolie — «superbe»".to_string())
        );
        assert_eq!(model.items[0].tags, vec!["日本語タグ".to_string()]);

        // Verify ViewModel preserves Unicode
        let vm = app.view(&model);
        assert_eq!(vm.items[0].title, "Ménuet en Sol");
        assert_eq!(vm.items[0].subtitle, "Dvořák");
    }

    // --- T045: Performance benchmark ---

    #[test]
    fn test_performance_10k_items() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();

        // Populate 10,000 items (5k pieces + 5k exercises).
        // Each piece links 5 exercise ids that exist in the fixture (e00000–e04999),
        // so the reverse-index path is load-tested — an O(n²) scan would be caught.
        let start = std::time::Instant::now();
        for i in 0..5000 {
            let linked_exercise_ids: Vec<String> = (0..5)
                .map(|k| format!("e{:05}", (i * 7 + k * 997) % 5000))
                .collect();
            model.items.push(Item {
                id: format!("p{i:05}"),
                title: format!("Piece {i}"),
                kind: ItemKind::Piece,
                composer: Some(format!("Composer {}", i % 100)),
                key: if i % 3 == 0 {
                    Some("C Major".to_string())
                } else {
                    None
                },
                modality: None,
                tempo: if i % 5 == 0 {
                    Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(120),
                    })
                } else {
                    None
                },
                notes: if i % 7 == 0 {
                    Some(format!("Notes for piece {i}"))
                } else {
                    None
                },
                tags: vec![format!("tag{}", i % 10)],
                created_at: now,
                updated_at: now,
                linked_exercise_ids,
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            });
        }
        for i in 0..5000 {
            model.items.push(Item {
                id: format!("e{i:05}"),
                title: format!("Exercise {i}"),
                kind: ItemKind::Exercise,
                composer: None,
                key: if i % 4 == 0 {
                    Some("G Major".to_string())
                } else {
                    None
                },
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![format!("etag{}", i % 10)],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            });
        }
        let populate_time = start.elapsed();
        // Heavier than a bare-item fixture: each of the 5k pieces builds 5 linked
        // exercise-id strings to load-test the reverse index. This is fixture setup,
        // not the gate — the bound is generous to absorb slow-CI debug-build variance.
        assert!(
            populate_time.as_millis() < 500,
            "Populating 10k items took {}ms (target: <500ms)",
            populate_time.as_millis()
        );

        // Populate 500 sessions with 5 entries each (2,500 entries total)
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };
        let start = std::time::Instant::now();
        for s in 0..500u32 {
            let entries: Vec<SetlistEntry> = (0..5u32)
                .map(|e| {
                    let item_idx = ((s * 5 + e) % 10_000) as usize;
                    let (item_id, item_title, item_type) = if item_idx < 5000 {
                        (
                            format!("p{item_idx:05}"),
                            format!("Piece {item_idx}"),
                            ItemKind::Piece,
                        )
                    } else {
                        let idx = item_idx - 5000;
                        (
                            format!("e{idx:05}"),
                            format!("Exercise {idx}"),
                            ItemKind::Exercise,
                        )
                    };
                    SetlistEntry {
                        id: format!("se{s:04}_{e}"),
                        item_id,
                        item_title,
                        item_type,
                        position: e as usize,
                        duration_secs: 300,
                        status: EntryStatus::Completed,
                        plays: vec![VariationPlay {
                            id: format!("se{s:04}_{e}-play"),
                            seconds: 300,
                            achieved_tempo: if e % 3 == 0 { Some(120) } else { None },
                            score: if e % 2 == 0 { Some(3) } else { None },
                            ..VariationPlay::fixture()
                        }],
                        ..SetlistEntry::fixture()
                    }
                })
                .collect();
            model.sessions.push(PracticeSession {
                id: format!("sess{s:04}"),
                started_at: now - chrono::Duration::hours(s as i64 + 1),
                completed_at: now - chrono::Duration::hours(s as i64),
                total_duration_secs: 1500,
                completion_status: CompletionStatus::Completed,
                session_notes: None,
                entries,
                session_score: None,
            });
        }
        model.practice_summaries = build_practice_summaries(&model.sessions);
        let session_populate_time = start.elapsed();
        assert!(
            session_populate_time.as_millis() < 200,
            "Populating 500 sessions + cache took {}ms (target: <200ms)",
            session_populate_time.as_millis()
        );

        // Benchmark: view() with 10k items + 500 sessions
        let start = std::time::Instant::now();
        let vm = app.view(&model);
        let view_time = start.elapsed();
        assert_eq!(vm.items.len(), 10_000);
        // O(n): forward resolution + the O(n) reverse index over 10k items + 25k
        // links. A naive O(n²) reverse scan (5k exercises × 5k pieces = 25M) would
        // run in seconds, so this still catches that regression with wide margin;
        // the bound is loose only to absorb slow-CI debug-build wall-clock variance.
        assert!(
            view_time.as_millis() < 1000,
            "view() with 10k items took {}ms (target: <1000ms)",
            view_time.as_millis()
        );

        // Benchmark: add one more item with 10k existing
        let start = std::time::Instant::now();
        let _cmd = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "New Piece".to_string(),
                kind: ItemKind::Piece,
                composer: Some("New Composer".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );
        let add_time = start.elapsed();
        assert_eq!(model.items.len(), 10_001);
        assert!(
            add_time.as_millis() < 100,
            "Adding item with 10k existing took {}ms (target: <100ms)",
            add_time.as_millis()
        );

        // Benchmark: delete item with 10k existing
        let start = std::time::Instant::now();
        let _cmd = app.update(
            Event::Item(ItemEvent::Delete {
                id: "p00042".to_string(),
            }),
            &mut model,
        );
        let delete_time = start.elapsed();
        assert_eq!(model.items.len(), 10_000);
        assert!(
            delete_time.as_millis() < 100,
            "Deleting item with 10k existing took {}ms (target: <100ms)",
            delete_time.as_millis()
        );
    }

    // --- Practice summary with new setlist sessions ---

    #[test]
    fn test_view_practice_summary_with_setlist_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::default();

        let p1 = Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        let p2 = Item {
            id: "p2".to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        model.items = vec![p1, p2];

        // Create a completed session with two entries
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };
        let sess1_started = now - chrono::Duration::minutes(60);
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: sess1_started,
            completed_at: now,
            total_duration_secs: 2700,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![
                SetlistEntry {
                    id: "e1".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 0,
                    duration_secs: 1800, // 30 min
                    status: EntryStatus::Completed,
                    notes: None,
                    intention: None,
                    planned_duration_secs: None,
                    group_id: None,
                    planned_variation_id: None,
                    planned_rep_target: None,
                    plays: vec![VariationPlay {
                        seconds: 1800,
                        achieved_tempo: None,
                        score: None,
                        ..VariationPlay::fixture()
                    }],
                },
                SetlistEntry {
                    id: "e2".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 1,
                    duration_secs: 900, // 15 min
                    status: EntryStatus::Completed,
                    notes: None,
                    intention: None,
                    planned_duration_secs: None,
                    group_id: None,
                    planned_variation_id: None,
                    planned_rep_target: None,
                    plays: vec![VariationPlay {
                        seconds: 900,
                        achieved_tempo: None,
                        score: None,
                        ..VariationPlay::fixture()
                    }],
                },
            ],
        });
        model.practice_summaries = build_practice_summaries(&model.sessions);

        let vm = app.view(&model);
        let p1_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let p2_view = vm.items.iter().find(|i| i.id == "p2").unwrap();

        // p1 has 2 entries totalling 45 minutes, no scores, no tempo
        let p1 = p1_view.practice.as_ref().expect("p1 practice summary");
        assert_eq!(p1.session_count, 2);
        assert_eq!(p1.total_minutes, 45);
        assert_eq!(p1.latest_score, None);
        assert_eq!(p1.latest_tempo, None);
        assert!(p1.score_history.is_empty());
        assert_eq!(p1.last_practiced_at, Some(sess1_started.to_rfc3339()));
        assert_eq!(p1.tempo_trend.points.len(), 2);
        assert!(!p1.tempo_trend.has_trend);
        // p2 has no entries
        assert_eq!(p2_view.practice, None);
    }

    // ── Score history tests (T019) ────────────────────────────────────

    #[test]
    fn test_score_history_multiple_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // Session 1: older, score 3
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(2),
            completed_at: now - chrono::Duration::hours(1),
            total_duration_secs: 3600,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: "e1".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 1800,
                status: EntryStatus::Completed,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: None,
                planned_rep_target: None,
                plays: vec![VariationPlay {
                    seconds: 1800,
                    achieved_tempo: None,
                    score: Some(3),
                    ..VariationPlay::fixture()
                }],
            }],
        });

        // Session 2: newer, score 5
        model.sessions.push(PracticeSession {
            id: "sess2".to_string(),
            started_at: now - chrono::Duration::minutes(30),
            completed_at: now,
            total_duration_secs: 1800,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: "e2".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 900,
                status: EntryStatus::Completed,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: None,
                planned_rep_target: None,
                plays: vec![VariationPlay {
                    seconds: 900,
                    achieved_tempo: None,
                    score: Some(5),
                    ..VariationPlay::fixture()
                }],
            }],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        // latest_score should be from the newer session
        assert_eq!(practice.latest_score, Some(5));
        assert_eq!(practice.score_history.len(), 2);
        // First entry = most recent (score 5)
        assert_eq!(practice.score_history[0].score, 5);
        assert_eq!(practice.score_history[0].session_id, "sess2");
        // Second entry = older (score 3)
        assert_eq!(practice.score_history[1].score, 3);
        assert_eq!(practice.score_history[1].session_id, "sess1");
    }

    #[test]
    fn test_score_history_no_scored_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // Session with no score
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(1),
            completed_at: now,
            total_duration_secs: 1800,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: "e1".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 1800,
                status: EntryStatus::Completed,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: None,
                planned_rep_target: None,
                plays: vec![VariationPlay {
                    seconds: 1800,
                    achieved_tempo: None,
                    score: None,
                    ..VariationPlay::fixture()
                }],
            }],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        assert_eq!(practice.latest_score, None);
        assert!(practice.score_history.is_empty());
    }

    #[test]
    fn test_score_history_item_multiple_times_in_one_session() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // Single session with the same item twice (different scores)
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(1),
            completed_at: now,
            total_duration_secs: 3600,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![
                SetlistEntry {
                    id: "e1".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 0,
                    duration_secs: 1800,
                    status: EntryStatus::Completed,
                    notes: None,
                    intention: None,
                    planned_duration_secs: None,
                    group_id: None,
                    planned_variation_id: None,
                    planned_rep_target: None,
                    plays: vec![VariationPlay {
                        seconds: 1800,
                        achieved_tempo: None,
                        score: Some(2),
                        ..VariationPlay::fixture()
                    }],
                },
                SetlistEntry {
                    id: "e2".to_string(),
                    item_id: "p1".to_string(),
                    item_title: "Sonata".to_string(),
                    item_type: ItemKind::Piece,
                    position: 1,
                    duration_secs: 1800,
                    status: EntryStatus::Completed,
                    notes: None,
                    intention: None,
                    planned_duration_secs: None,
                    group_id: None,
                    planned_variation_id: None,
                    planned_rep_target: None,
                    plays: vec![VariationPlay {
                        seconds: 1800,
                        achieved_tempo: None,
                        score: Some(4),
                        ..VariationPlay::fixture()
                    }],
                },
            ],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        // Both entries from the same session should appear in score_history
        assert_eq!(practice.score_history.len(), 2);
        // Both have the same session_id
        assert!(practice
            .score_history
            .iter()
            .all(|e| e.session_id == "sess1"));
    }

    #[test]
    fn test_score_history_skipped_entries_excluded() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::default();

        model.items.push(Item {
            id: "p1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Beethoven".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        });

        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, SetlistEntry,
        };

        // A skipped entry won't have a score (scores only set on completed entries)
        model.sessions.push(PracticeSession {
            id: "sess1".to_string(),
            started_at: now - chrono::Duration::hours(1),
            completed_at: now,
            total_duration_secs: 600,
            completion_status: CompletionStatus::EndedEarly,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: "e1".to_string(),
                item_id: "p1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 600,
                status: EntryStatus::Skipped,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: None,
                planned_rep_target: None,
                // A skipped entry records no play, so it never carries a mark.
                plays: Vec::new(),
            }],
        });

        model.practice_summaries = build_practice_summaries(&model.sessions);
        let vm = app.view(&model);
        let p1 = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let practice = p1.practice.as_ref().unwrap();

        assert_eq!(practice.latest_score, None);
        assert!(practice.score_history.is_empty());
    }

    // --- Lifecycle events ---

    #[test]
    fn set_utc_offset_updates_model() {
        let app = Intrada;
        let mut model = Model::default();
        assert_eq!(model.utc_offset_minutes, 0);

        let _cmd = app.update(Event::SetUtcOffset { minutes: 60 }, &mut model);

        assert_eq!(model.utc_offset_minutes, 60);
    }

    #[test]
    fn set_utc_offset_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(Event::SetUtcOffset { minutes: -300 });
    }

    fn make_session(
        id: &str,
        item_id: &str,
        score: Option<u8>,
        tempo: Option<u16>,
    ) -> PracticeSession {
        make_session_at(id, item_id, chrono::Utc::now(), score, tempo)
    }

    fn make_session_at(
        id: &str,
        item_id: &str,
        started_at: chrono::DateTime<chrono::Utc>,
        score: Option<u8>,
        tempo: Option<u16>,
    ) -> PracticeSession {
        PracticeSession {
            id: id.to_string(),
            started_at,
            completed_at: started_at,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: format!("{id}-e1"),
                item_id: item_id.to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 300,
                status: EntryStatus::Completed,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: None,
                planned_rep_target: None,
                plays: vec![VariationPlay {
                    seconds: 300,
                    achieved_tempo: tempo,
                    score,
                    ..VariationPlay::fixture()
                }],
            }],
        }
    }

    // ── The tempo trend (#1420) ──

    /// `measured[i]` is the tempo of the i-th session chronologically. The
    /// sessions are handed over newest first, so a projection that skipped its
    /// own sort would emit the series backwards.
    fn summary_for_tempos(measured: &[Option<u16>]) -> ItemPracticeSummary {
        let base = chrono::Utc::now() - chrono::Duration::days(30);
        let sessions: Vec<PracticeSession> = measured
            .iter()
            .enumerate()
            .rev()
            .map(|(i, tempo)| {
                make_session_at(
                    &format!("s{i}"),
                    "item-1",
                    base + chrono::Duration::days(i as i64),
                    None,
                    *tempo,
                )
            })
            .collect();
        build_practice_summaries(&sessions)
            .remove("item-1")
            .expect("summary for item-1")
    }

    fn tempo_trend_for(measured: &[Option<u16>]) -> crate::model::TempoTrendView {
        summary_for_tempos(measured).tempo_trend
    }

    #[test]
    fn tempo_trend_runs_oldest_first_one_point_per_session() {
        let trend = tempo_trend_for(&[Some(88), Some(96), Some(104)]);

        assert_eq!(
            trend.points.iter().map(|p| p.tempo).collect::<Vec<_>>(),
            vec![Some(88), Some(96), Some(104)]
        );
        let dates: Vec<&String> = trend.points.iter().map(|p| &p.session_date).collect();
        assert!(
            dates.windows(2).all(|w| w[0] < w[1]),
            "oldest first: {dates:?}"
        );
    }

    #[test]
    fn tempo_trend_leaves_a_gap_where_no_tempo_was_measured() {
        let trend = tempo_trend_for(&[Some(88), None, Some(104)]);

        assert_eq!(
            trend.points.iter().map(|p| p.tempo).collect::<Vec<_>>(),
            vec![Some(88), None, Some(104)]
        );
    }

    #[test]
    fn latest_tempo_is_the_newest_measured_not_the_newest_session() {
        // Reading the newest slot rather than the newest number would drop it.
        assert_eq!(summary_for_tempos(&[Some(88), None]).latest_tempo, Some(88));
    }

    #[test]
    fn tempo_trend_has_no_trend_below_two_measured_tempos() {
        assert!(!tempo_trend_for(&[None, None]).has_trend);
        assert!(!tempo_trend_for(&[Some(96), None, None]).has_trend);
    }

    #[test]
    fn tempo_trend_has_a_trend_at_two_measured_tempos() {
        assert!(tempo_trend_for(&[Some(96), None, Some(104)]).has_trend);
    }

    #[test]
    fn tempo_trend_round_trips_a_gap_on_the_ffi_bincode_wire() {
        // A `None` inside the points vec is the shape most at risk on the
        // positional wire, and the gap is the whole point of the chart (#846).
        crate::domain::types::assert_round_trips(tempo_trend_for(&[Some(88), None, Some(104)]));
    }

    #[test]
    fn summary_last_practiced_is_max_session_date() {
        let earlier = chrono::Utc::now() - chrono::Duration::days(3);
        let later = chrono::Utc::now() - chrono::Duration::days(1);

        let mk = |id: &str, started: chrono::DateTime<chrono::Utc>| PracticeSession {
            id: id.to_string(),
            started_at: started,
            completed_at: started,
            total_duration_secs: 60,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: format!("{id}-e"),
                item_id: "item-1".to_string(),
                item_title: "Sonata".to_string(),
                item_type: ItemKind::Piece,
                position: 0,
                duration_secs: 60,
                status: EntryStatus::Completed,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: None,
                planned_rep_target: None,
                plays: vec![VariationPlay {
                    seconds: 60,
                    achieved_tempo: None,
                    score: None,
                    ..VariationPlay::fixture()
                }],
            }],
        };

        let summaries = build_practice_summaries(&[mk("s1", earlier), mk("s2", later)]);
        let summary = summaries.get("item-1").expect("summary for item-1");
        assert_eq!(summary.last_practiced_at, Some(later.to_rfc3339()));
    }

    // --- Error handling ---

    #[test]
    fn test_clear_error_sets_muted_flag() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("some error".to_string()),
            ..Model::default()
        };

        let _ = app.update(Event::ClearError, &mut model);

        assert_eq!(model.last_error, None);
        assert!(model.error_muted);
    }

    // --- View: which session slot the builder fills ---

    #[test]
    fn test_view_populates_building_setlist_only() {
        use crate::domain::session::BuildingSession;

        let app = Intrada;
        let model = Model {
            session_status: SessionStatus::Building(BuildingSession::default()),
            ..Model::default()
        };

        let vm = app.view(&model);
        assert!(vm.building_setlist.is_some());
        assert!(vm.active_session.is_none());
        assert!(vm.summary.is_none());
    }

    // --- Practice summaries edge cases ---

    #[test]
    fn test_practice_summaries_empty_sessions() {
        let summaries = build_practice_summaries(&[]);
        assert!(summaries.is_empty());
    }

    #[test]
    fn test_practice_summaries_entry_without_score_or_tempo() {
        let sessions = vec![{
            let mut s = make_session("s1", "item-1", None, None);
            s.entries[0].duration_secs = 180;
            s
        }];

        let summaries = build_practice_summaries(&sessions);
        let summary = &summaries["item-1"];
        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.total_minutes, 3);
        assert!(summary.latest_score.is_none());
        assert!(summary.latest_tempo.is_none());
        assert!(summary.score_history.is_empty());
        assert_eq!(summary.tempo_trend.points.len(), 1);
        assert!(summary.tempo_trend.points[0].tempo.is_none());
        assert!(!summary.tempo_trend.has_trend);
    }

    #[test]
    fn test_view_empty_sessions() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);
        assert!(vm.sessions.is_empty());
    }

    #[test]
    fn test_tempo_format_display() {
        use crate::domain::types::Tempo;

        // None tempo — map returns None
        let none_tempo: Option<Tempo> = None;
        assert_eq!(none_tempo.as_ref().map(|t| t.format_display()), None);

        // Both None — empty string
        let tempo = Tempo {
            marking: None,
            bpm: None,
        };
        assert_eq!(tempo.format_display(), "");

        // Marking only
        let tempo = Tempo {
            marking: Some("Adagio".to_string()),
            bpm: None,
        };
        assert_eq!(tempo.format_display(), "Adagio");

        // BPM only
        let tempo = Tempo {
            marking: None,
            bpm: Some(120),
        };
        assert_eq!(tempo.format_display(), "120 BPM");

        // Both
        let tempo = Tempo {
            marking: Some("Allegro".to_string()),
            bpm: Some(132),
        };
        assert_eq!(tempo.format_display(), "Allegro (132 BPM)");
    }

    // ── ViewModel projection tests (#554) ──────────────────────────────

    fn make_item(
        id: &str,
        title: &str,
        kind: ItemKind,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Item {
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at,
            updated_at: created_at,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        }
    }

    #[test]
    fn view_exposes_the_up_next_suggestion() {
        // Wiring only: the ranking and the wording are pinned in
        // suggestion::tests, which can hold the clock still.
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mut piece = make_item("p1", "Sonata", ItemKind::Piece, now);
        piece.linked_exercise_ids = vec!["ex1".to_string()];
        model.items = vec![piece, make_item("ex1", "Scales", ItemKind::Exercise, now)];

        let up_next = app.view(&model).up_next.expect("a suggestion");
        assert_eq!(up_next.piece_id, "p1");
        assert_eq!(
            up_next
                .items
                .iter()
                .map(|i| i.item_id.as_str())
                .collect::<Vec<_>>(),
            ["ex1", "p1"]
        );
    }

    #[test]
    fn a_library_filter_cannot_hide_the_up_next_suggestion() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mut piece = make_item("p1", "Sonata", ItemKind::Piece, now);
        piece.linked_exercise_ids = vec!["ex1".to_string()];
        model.items = vec![piece, make_item("ex1", "Scales", ItemKind::Exercise, now)];
        // Filtering the library to exercises hides the anchor piece from the
        // list; the suggestion is derived pre-filter and must survive it.
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert!(
            !vm.items.iter().any(|i| i.id == "p1"),
            "the filter really does hide the piece from the list"
        );
        assert_eq!(
            vm.up_next.expect("a suggestion").piece_id,
            "p1",
            "the suggestion is derived before the filter"
        );
    }

    #[test]
    fn a_library_filter_cannot_hide_a_link_picker_candidate() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Sonata", ItemKind::Piece, now),
            make_item("ex1", "Scales", ItemKind::Exercise, now),
        ];
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert!(
            !vm.items.iter().any(|i| i.id == "p1"),
            "the filter really does hide the piece from the list"
        );
        assert!(
            vm.all_items.iter().any(|i| i.id == "p1"),
            "all_items is the unfiltered picker source and must still offer it"
        );
    }

    #[test]
    fn a_library_filter_cannot_hide_a_variation_coverage_row() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mut scales = make_item("ex1", "Scales", ItemKind::Exercise, now);
        scales.variants = ["C", "G"]
            .iter()
            .enumerate()
            .map(|(position, label)| crate::domain::variant::Variant {
                id: format!("ex1-{position}"),
                label: label.to_string(),
                position,
                updated_at: now,
                deleted_at: None,
            })
            .collect();
        model.items = vec![make_item("p1", "Sonata", ItemKind::Piece, now), scales];
        model.sessions = vec![make_session("s1", "ex1", Some(8), None)];
        model.practice_summaries = build_practice_summaries(&model.sessions);
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Piece),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert!(
            !vm.items.iter().any(|i| i.id == "ex1"),
            "the filter really does hide the exercise from the list"
        );
        let analytics = vm.analytics.expect("a session makes the analytics view");
        assert_eq!(
            analytics
                .variation_coverage
                .iter()
                .map(|r| r.item_id.as_str())
                .collect::<Vec<_>>(),
            ["ex1"],
            "coverage is derived before the filter"
        );
    }

    #[test]
    fn view_has_no_up_next_when_no_piece_has_a_related_exercise() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Sonata", ItemKind::Piece, now),
            make_item("ex1", "Scales", ItemKind::Exercise, now),
        ];

        assert!(
            app.view(&model).up_next.is_none(),
            "it suggests, it never invents"
        );
    }

    #[test]
    fn view_flags_priorities_only_when_something_is_starred() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![make_item("p1", "Sonata", ItemKind::Piece, now)];

        assert!(
            !app.view(&model).has_priorities,
            "nothing starred, so the button cannot be on screen"
        );

        model.items[0].priority = true;
        assert!(app.view(&model).has_priorities);
    }

    #[test]
    fn the_priorities_flag_agrees_with_what_the_event_would_seed() {
        // Two independent spellings of "starred": drift means a button on
        // screen whose tap silently seeds nothing (#981).
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Sonata", ItemKind::Piece, now),
            make_item("ex1", "Scales", ItemKind::Exercise, now),
        ];

        for starred in [false, true] {
            model.items[0].priority = starred;
            assert_eq!(
                app.view(&model).has_priorities,
                !derive_priorities(&model, now).is_empty(),
                "the flag and the event must agree, starred: {starred}"
            );
        }
    }

    #[test]
    fn a_library_filter_cannot_hide_the_priorities_button() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mut piece = make_item("p1", "Sonata", ItemKind::Piece, now);
        piece.priority = true;
        model.items = vec![piece, make_item("ex1", "Scales", ItemKind::Exercise, now)];
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            ..Default::default()
        });

        let vm = app.view(&model);
        assert!(
            !vm.items.iter().any(|i| i.id == "p1"),
            "the filter really does hide the starred piece from the list"
        );
        assert!(
            vm.has_priorities,
            "the flag is derived before the filter, like up_next"
        );
    }

    #[test]
    fn view_exposes_last_practised_when_a_session_has_been_played() {
        // Wiring only: the relative-day wording is pinned in analytics::tests,
        // which can hold the clock still.
        let app = Intrada;
        let model = Model {
            sessions: vec![make_session("s1", "item-1", None, None)],
            ..Default::default()
        };
        let last = app
            .view(&model)
            .last_practised
            .expect("a session was played");
        assert_eq!(last.item_title, "Sonata");
    }

    #[test]
    fn view_items_sorted_newest_first() {
        let app = Intrada;
        let mut model = Model::default();
        let t1 = chrono::Utc::now() - chrono::Duration::hours(2);
        let t2 = chrono::Utc::now() - chrono::Duration::hours(1);
        let t3 = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Old", ItemKind::Piece, t1),
            make_item("c", "Newest", ItemKind::Exercise, t3),
            make_item("b", "Middle", ItemKind::Piece, t2),
        ];
        let vm = app.view(&model);
        assert_eq!(vm.items[0].title, "Newest");
        assert_eq!(vm.items[1].title, "Middle");
        assert_eq!(vm.items[2].title, "Old");
    }

    fn set_last_practiced(model: &mut Model, item_id: &str, at: chrono::DateTime<chrono::Utc>) {
        model.practice_summaries.insert(
            item_id.to_string(),
            crate::model::ItemPracticeSummary {
                session_count: 1,
                total_minutes: 1,
                last_practiced_at: Some(at.to_rfc3339()),
                ..crate::model::ItemPracticeSummary::fixture()
            },
        );
    }

    #[test]
    fn view_sorts_by_title_ascending() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Sonata", ItemKind::Piece, now),
            make_item("b", "etude", ItemKind::Piece, now), // lowercase: case-insensitive
            make_item("c", "Ballade", ItemKind::Piece, now),
        ];
        model.active_sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let vm = app.view(&model);
        let titles: Vec<_> = vm.items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["Ballade", "etude", "Sonata"]);
    }

    #[test]
    fn view_sorts_accented_titles_by_their_base_letter() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Waltz", ItemKind::Piece, now),
            make_item("b", "\u{c9}tude", ItemKind::Piece, now),
            make_item("c", "Ballade", ItemKind::Piece, now),
        ];
        model.active_sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let vm = app.view(&model);
        let titles: Vec<_> = vm.items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["Ballade", "\u{c9}tude", "Waltz"]);
    }

    /// The accent is dropped, not just decomposed: kept as a combining mark it
    /// would sort after every letter, putting "Étude" behind "Etudes".
    #[test]
    fn view_sorts_an_accented_title_beside_its_unaccented_neighbour() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Etudes", ItemKind::Piece, now),
            make_item("b", "\u{c9}tude", ItemKind::Piece, now),
        ];
        model.active_sort = LibrarySort {
            field: SortField::Title,
            direction: SortDirection::Ascending,
        };
        let vm = app.view(&model);
        let titles: Vec<_> = vm.items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["\u{c9}tude", "Etudes"]);
    }

    #[test]
    fn view_sorts_by_last_practiced_descending_most_recent_first() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Stale", ItemKind::Piece, now),
            make_item("b", "Fresh", ItemKind::Piece, now),
        ];
        set_last_practiced(&mut model, "a", now - chrono::Duration::days(5));
        set_last_practiced(&mut model, "b", now - chrono::Duration::days(1));
        model.active_sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Descending,
        };
        let vm = app.view(&model);
        assert_eq!(vm.items[0].title, "Fresh");
        assert_eq!(vm.items[1].title, "Stale");
    }

    #[test]
    fn view_never_practiced_sorts_as_oldest() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Practiced", ItemKind::Piece, now),
            make_item("b", "NeverPractised", ItemKind::Piece, now),
        ];
        set_last_practiced(&mut model, "a", now - chrono::Duration::days(2));
        // "b" has no practice summary -> never practised.

        // Ascending (longest since practised first): never-practised rises to the top.
        model.active_sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Ascending,
        };
        assert_eq!(app.view(&model).items[0].title, "NeverPractised");

        // Descending (most recent first): never-practised sinks to the bottom.
        model.active_sort = LibrarySort {
            field: SortField::LastPracticed,
            direction: SortDirection::Descending,
        };
        assert_eq!(
            app.view(&model).items.last().unwrap().title,
            "NeverPractised"
        );
    }

    #[test]
    fn recently_practised_lists_practised_items_most_recent_first() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Stale", ItemKind::Piece, now),
            make_item("b", "Fresh", ItemKind::Exercise, now),
            make_item("c", "NeverPractised", ItemKind::Piece, now),
        ];
        set_last_practiced(&mut model, "a", now - chrono::Duration::days(5));
        set_last_practiced(&mut model, "b", now - chrono::Duration::days(1));

        let vm = app.view(&model);

        assert_eq!(
            vm.recently_practised
                .iter()
                .map(|i| i.title.as_str())
                .collect::<Vec<_>>(),
            ["Fresh", "Stale"],
            "most recently practised first, never-practised excluded"
        );
    }

    #[test]
    fn recently_practised_caps_at_five() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        for i in 0..8 {
            let id = format!("p{i}");
            model.items.push(make_item(&id, &id, ItemKind::Piece, now));
            set_last_practiced(&mut model, &id, now - chrono::Duration::days(i as i64));
        }

        let vm = app.view(&model);

        assert_eq!(vm.recently_practised.len(), 5);
        assert_eq!(vm.recently_practised[0].title, "p0");
    }

    #[test]
    fn a_library_filter_cannot_hide_a_recently_practised_item() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Sonata", ItemKind::Piece, now),
            make_item("ex1", "Scales", ItemKind::Exercise, now),
        ];
        set_last_practiced(&mut model, "p1", now - chrono::Duration::days(1));
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            ..Default::default()
        });

        let vm = app.view(&model);

        assert!(!vm.items.iter().any(|i| i.id == "p1"));
        assert!(vm.recently_practised.iter().any(|i| i.id == "p1"));
    }

    #[test]
    fn view_default_sort_is_date_added_newest_first() {
        let app = Intrada;
        let mut model = Model::default();
        let t1 = chrono::Utc::now() - chrono::Duration::hours(2);
        let t2 = chrono::Utc::now();
        model.items = vec![
            make_item("a", "Old", ItemKind::Piece, t1),
            make_item("b", "New", ItemKind::Piece, t2),
        ];
        let vm = app.view(&model); // default active_sort
        assert_eq!(vm.items[0].title, "New");
        assert_eq!(vm.items[1].title, "Old");
    }

    #[test]
    fn view_query_filters_by_item_type() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Piece One", ItemKind::Piece, now),
            make_item("e1", "Exercise One", ItemKind::Exercise, now),
        ];
        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            key: None,
            tags: vec![],
            text: None,
        });
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Exercise One");
    }

    #[test]
    fn view_query_filters_by_text_search() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Clair de Lune", ItemKind::Piece, now),
            make_item("p2", "Moonlight Sonata", ItemKind::Piece, now),
        ];
        model.active_query = Some(ListQuery {
            item_type: None,
            key: None,
            tags: vec![],
            text: Some("clair".to_string()),
        });
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Clair de Lune");
    }

    #[test]
    fn view_query_filters_by_tags() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        let mut tagged = make_item("p1", "Tagged", ItemKind::Piece, now);
        tagged.tags = vec!["Warm-up".to_string(), "Scales".to_string()];
        let untagged = make_item("p2", "Untagged", ItemKind::Piece, now);
        model.items = vec![tagged, untagged];
        model.active_query = Some(ListQuery {
            item_type: None,
            key: None,
            tags: vec!["warm-up".to_string()],
            text: None,
        });
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Tagged");
    }

    #[test]
    fn view_exposes_active_query() {
        let app = Intrada;
        let mut model = Model::default();
        let query = ListQuery {
            item_type: Some(ItemKind::Piece),
            key: None,
            tags: vec![],
            text: None,
        };
        model.active_query = Some(query.clone());
        let vm = app.view(&model);
        assert_eq!(vm.active_query, Some(query));
    }

    #[test]
    fn view_active_query_none_when_unset() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);
        assert_eq!(vm.active_query, None);
    }

    #[test]
    fn view_counts_describe_the_visible_set() {
        let app = Intrada;
        let mut model = Model::default();
        let now = chrono::Utc::now();
        model.items = vec![
            make_item("p1", "Piece One", ItemKind::Piece, now),
            make_item("p2", "Piece Two", ItemKind::Piece, now),
            make_item("e1", "Exercise One", ItemKind::Exercise, now),
        ];

        let vm = app.view(&model);
        assert_eq!(vm.visible_pieces, 2);
        assert_eq!(vm.visible_exercises, 1);

        model.active_query = Some(ListQuery {
            item_type: Some(ItemKind::Exercise),
            key: None,
            tags: vec![],
            text: None,
        });
        let vm = app.view(&model);
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.visible_pieces, 0);
        assert_eq!(vm.visible_exercises, 1);

        model.active_query = Some(ListQuery {
            item_type: None,
            key: None,
            tags: vec![],
            text: Some("Piece One".to_string()),
        });
        let vm = app.view(&model);
        assert_eq!(vm.visible_pieces, 1);
        assert_eq!(vm.visible_exercises, 0);
    }

    #[test]
    fn view_sessions_sorted_newest_first() {
        let app = Intrada;
        let mut model = Model::default();
        let t1 = chrono::Utc::now() - chrono::Duration::hours(3);
        let t2 = chrono::Utc::now() - chrono::Duration::hours(1);
        model.sessions = vec![
            PracticeSession {
                id: "s1".to_string(),
                started_at: t1,
                completed_at: t1 + chrono::Duration::minutes(30),
                total_duration_secs: 1800,
                completion_status: CompletionStatus::Completed,
                entries: vec![],
                session_notes: None,
                session_score: None,
            },
            PracticeSession {
                id: "s2".to_string(),
                started_at: t2,
                completed_at: t2 + chrono::Duration::minutes(15),
                total_duration_secs: 900,
                completion_status: CompletionStatus::Completed,
                entries: vec![],
                session_notes: None,
                session_score: None,
            },
        ];
        let vm = app.view(&model);
        assert_eq!(vm.sessions[0].id, "s2");
        assert_eq!(vm.sessions[1].id, "s1");
    }

    #[test]
    fn view_error_maps_from_last_error() {
        let app = Intrada;
        let model = Model {
            last_error: Some("bad request".to_string()),
            ..Default::default()
        };
        let vm = app.view(&model);
        assert_eq!(vm.error.as_deref(), Some("bad request"));
    }

    #[test]
    fn view_error_target_maps_from_the_model() {
        let app = Intrada;
        let mut model = Model::default();
        assert!(app.view(&model).error_target.is_none());

        model.last_error_target = Some(crate::model::FormErrorTarget::ChartBar {
            bar_number: 3,
            token: "(F7)".to_string(),
        });

        assert_eq!(
            app.view(&model).error_target,
            Some(crate::model::FormErrorTarget::ChartBar {
                bar_number: 3,
                token: "(F7)".to_string()
            })
        );
    }

    #[test]
    fn view_empty_sessions_produces_no_analytics() {
        let app = Intrada;
        let model = Model::default();
        let vm = app.view(&model);
        assert!(vm.analytics.is_none());
    }

    fn building_entry(id: &str, planned_duration_secs: Option<u32>) -> SetlistEntry {
        SetlistEntry {
            id: id.to_string(),
            item_id: format!("item-{id}"),
            item_title: "Etude".to_string(),
            planned_duration_secs,
            ..SetlistEntry::fixture()
        }
    }

    #[test]
    fn error_seq_bumps_on_each_failed_update_even_with_identical_message() {
        let app = Intrada;
        let mut model = Model::default();
        let fail = || {
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "x".to_string(),
            })
        };
        let _ = app.update(fail(), &mut model);
        let seq1 = app.view(&model).error_seq;
        let _ = app.update(fail(), &mut model);
        let seq2 = app.view(&model).error_seq;
        assert!(seq1 > 0);
        assert!(
            seq2 > seq1,
            "a repeated identical failure must still advance the sequence"
        );
    }

    #[test]
    fn error_seq_stable_across_successful_updates() {
        let app = Intrada;
        let mut model = Model::default();
        let before = app.view(&model).error_seq;
        let _ = app.update(Event::Session(SessionEvent::StartBuilding), &mut model);
        assert_eq!(app.view(&model).error_seq, before);
    }

    #[test]
    fn a_success_under_a_standing_storage_banner_leaves_error_seq_alone() {
        let app = Intrada;
        let mut model = Model::default();
        let _ = app.update(
            Event::StoreLoaded(crate::persistence::PersistenceOutput::Failed),
            &mut model,
        );
        let before = app.view(&model).error_seq;
        let _ = app.update(Event::SetQuery(None), &mut model);
        assert!(model.last_error.is_some(), "the banner is still up");
        assert_eq!(
            app.view(&model).error_seq,
            before,
            "an accepted send must not read as refused in the shell"
        );
    }

    #[test]
    fn a_refusal_after_a_dismiss_still_shows() {
        let app = Intrada;
        let mut model = Model::default();
        let _ = app.update(
            Event::StoreLoaded(crate::persistence::PersistenceOutput::Failed),
            &mut model,
        );
        let _ = app.update(Event::ClearError, &mut model);
        let before = app.view(&model).error_seq;
        let _ = app.update(
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "x".to_string(),
            }),
            &mut model,
        );
        assert!(
            model.last_error.is_some(),
            "a dismiss never mutes a refusal"
        );
        assert!(app.view(&model).error_seq > before);
    }

    // ── The notice channel (#1325) ──

    #[test]
    fn view_carries_the_notice_and_its_sequence() {
        let app = Intrada;
        let mut model = Model::default();
        assert_eq!(app.view(&model).notice, None);
        assert_eq!(app.view(&model).notice_seq, 0);

        model.raise_notice("kept, but");

        let view = app.view(&model);
        assert_eq!(view.notice.as_deref(), Some("kept, but"));
        assert_eq!(view.notice_seq, 1);
    }

    #[test]
    fn clear_notice_leaves_a_standing_error_and_clear_error_leaves_a_notice() {
        let app = Intrada;
        let mut model = Model::default();
        model.raise_error("refused");
        model.raise_notice("kept, but");

        let _ = app.update(Event::ClearNotice, &mut model);
        let view = app.view(&model);
        assert_eq!(view.notice, None, "the notice is dismissed");
        assert_eq!(view.error.as_deref(), Some("refused"), "the error stands");

        model.raise_notice("kept, but");
        let _ = app.update(Event::ClearError, &mut model);
        let view = app.view(&model);
        assert_eq!(view.error, None, "the error is dismissed");
        assert_eq!(
            view.notice.as_deref(),
            Some("kept, but"),
            "the notice stands"
        );
    }

    #[test]
    fn a_notice_never_moves_error_seq() {
        let app = Intrada;
        let mut model = Model::default();
        let before = app.view(&model).error_seq;

        model.raise_notice("kept, but");
        let _ = app.update(Event::SetQuery(None), &mut model);

        assert_eq!(
            app.view(&model).error_seq,
            before,
            "a notice is not a refusal, so the shell's success haptic still fires"
        );
    }

    #[test]
    fn view_building_setlist_total_duration_sums_planned() {
        let app = Intrada;
        let model = Model {
            session_status: SessionStatus::Building(crate::domain::session::BuildingSession {
                entries: vec![
                    building_entry("e1", Some(900)),
                    building_entry("e2", Some(630)),
                    building_entry("e3", None),
                ],
            }),
            ..Default::default()
        };
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.total_duration_display.as_deref(), Some("25m 30s"));
        assert_eq!(building.total_duration_summary.as_deref(), Some("25m 30s"));
    }

    #[test]
    fn view_building_setlist_total_duration_whole_minutes_matches_block_dialect() {
        let app = Intrada;
        let model = Model {
            session_status: SessionStatus::Building(crate::domain::session::BuildingSession {
                entries: vec![
                    building_entry("e1", Some(900)),
                    building_entry("e2", Some(300)),
                ],
            }),
            ..Default::default()
        };
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.total_duration_summary.as_deref(), Some("20 min"));
    }

    #[test]
    fn view_building_setlist_total_duration_none_when_unplanned() {
        let app = Intrada;
        let model = Model {
            session_status: SessionStatus::Building(crate::domain::session::BuildingSession {
                entries: vec![building_entry("e1", None), building_entry("e2", None)],
            }),
            ..Default::default()
        };
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.total_duration_display, None);
        assert_eq!(building.total_duration_summary, None);
    }

    #[test]
    fn test_new_item_defaults_to_not_priority() {
        let app = Intrada;
        let mut model = Model::default();

        let _cmd = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "Prelude".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Bach".to_string()),
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert!(!model.items[0].priority);

        let vm = app.view(&model);
        assert!(!vm.items[0].priority);
    }

    #[test]
    fn test_update_sets_item_priority() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Etude".to_string(),
                kind: ItemKind::Piece,
                composer: None,
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            }],
            ..Model::default()
        };

        let _cmd = app.update(
            Event::Item(ItemEvent::Update {
                id: "p1".to_string(),
                input: crate::domain::types::UpdateItem {
                    priority: Some(true),
                    ..Default::default()
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert!(model.items[0].priority);
    }

    #[test]
    fn test_add_item_carries_modality() {
        use crate::domain::item::Modality;
        let app = Intrada;
        let mut model = Model::default();

        let _cmd = app.update(
            Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
                title: "Clair de Lune".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Debussy".to_string()),
                key: Some("Db".to_string()),
                modality: Some(Modality::Major),
                tempo: None,
                notes: None,
                tags: vec![],
                photo_id: None,
                variant_labels: Vec::new(),
            })),
            &mut model,
        );

        assert_eq!(model.items[0].key.as_deref(), Some("Db"));
        assert_eq!(model.items[0].modality, Some(Modality::Major));
        let vm = app.view(&model);
        assert_eq!(vm.items[0].modality, Some(Modality::Major));
    }

    #[test]
    fn test_update_modality_is_three_state() {
        use crate::domain::item::Modality;
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Etude".to_string(),
                kind: ItemKind::Piece,
                composer: None,
                key: Some("F#".to_string()),
                modality: Some(Modality::Major),
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            }],
            ..Model::default()
        };

        let update = |m: &mut Model, input: crate::domain::types::UpdateItem| {
            let _ = app.update(
                Event::Item(ItemEvent::Update {
                    id: "p1".to_string(),
                    input,
                }),
                m,
            );
        };

        // set → Minor
        update(
            &mut model,
            crate::domain::types::UpdateItem {
                modality: Some(Some(Modality::Minor)),
                ..Default::default()
            },
        );
        assert_eq!(model.items[0].modality, Some(Modality::Minor));

        // skip (modality absent) → unchanged
        update(
            &mut model,
            crate::domain::types::UpdateItem {
                priority: Some(true),
                ..Default::default()
            },
        );
        assert_eq!(model.items[0].modality, Some(Modality::Minor));

        // clear → None
        update(
            &mut model,
            crate::domain::types::UpdateItem {
                modality: Some(None),
                ..Default::default()
            },
        );
        assert_eq!(model.items[0].modality, None);
    }

    #[test]
    fn test_update_changes_kind() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Scales".to_string(),
                kind: ItemKind::Piece,
                composer: None,
                key: None,
                modality: None,
                tempo: None,
                notes: None,
                tags: vec![],
                created_at: now,
                updated_at: now,
                linked_exercise_ids: vec![],
                priority: false,
                chord_chart: None,
                variants: vec![],
                photo_id: None,
                metre: None,
            }],
            ..Model::default()
        };

        let _ = app.update(
            Event::Item(ItemEvent::Update {
                id: "p1".to_string(),
                input: crate::domain::types::UpdateItem {
                    kind: Some(ItemKind::Exercise),
                    ..Default::default()
                },
            }),
            &mut model,
        );

        assert!(model.last_error.is_none());
        assert_eq!(model.items[0].kind, ItemKind::Exercise);
        // kind absent → unchanged
        let _ = app.update(
            Event::Item(ItemEvent::Update {
                id: "p1".to_string(),
                input: crate::domain::types::UpdateItem {
                    priority: Some(true),
                    ..Default::default()
                },
            }),
            &mut model,
        );
        assert_eq!(model.items[0].kind, ItemKind::Exercise);
    }

    #[test]
    fn linked_from_piece_ref_carries_composer_subtitle() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let piece = Item {
            id: "piece-1".to_string(),
            title: "Sonata".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec!["ex-1".to_string()],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        let ex = Item {
            id: "ex-1".to_string(),
            title: "Scales".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        };
        let model = Model {
            items: vec![piece, ex],
            ..Model::default()
        };
        let vm = app.view(&model);
        let exercise = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(
            exercise.used_in[0]
                .piece
                .as_ref()
                .unwrap()
                .subtitle
                .as_deref(),
            Some("Debussy")
        );
    }

    #[test]
    fn test_view_resolves_linked_exercises_and_used_in() {
        let app = Intrada;
        let now = chrono::Utc::now();

        // Piece P links ["ex-1", "ex-missing", "ex-2"]: ex-1 and ex-2 are present,
        // ex-missing is absent. Proves order is preserved AND the missing id is
        // dropped from the middle (not just from the end).
        // Unrelated exercise ex-3 is present but not linked by P.
        let model = Model {
            items: vec![
                Item {
                    id: "piece-1".to_string(),
                    title: "Sonata".to_string(),
                    kind: ItemKind::Piece,
                    composer: None,
                    key: Some("C".to_string()),
                    modality: None,
                    tempo: Some(crate::domain::types::Tempo {
                        marking: Some("Allegro".to_string()),
                        bpm: Some(120),
                    }),
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![
                        "ex-1".to_string(),
                        "ex-missing".to_string(),
                        "ex-2".to_string(),
                    ],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "ex-1".to_string(),
                    title: "Scales".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: Some("G".to_string()),
                    modality: None,
                    tempo: Some(crate::domain::types::Tempo {
                        marking: None,
                        bpm: Some(80),
                    }),
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "ex-2".to_string(),
                    title: "Arpeggios".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "ex-3".to_string(),
                    title: "Trills".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
            ],
            ..Default::default()
        };

        let vm = app.view(&model);

        // Piece: resolves [ex-1, ex-2] — ex-missing dropped from middle, order preserved.
        let piece_view = vm.items.iter().find(|i| i.id == "piece-1").unwrap();
        assert_eq!(
            piece_view.linked_exercises.len(),
            2,
            "both present exercises resolved; missing id dropped"
        );
        assert_eq!(piece_view.linked_exercises[0].id, "ex-1", "ex-1 is first");
        assert_eq!(piece_view.linked_exercises[1].id, "ex-2", "ex-2 is second");
        assert_eq!(piece_view.linked_exercises[0].title, "Scales");
        assert_eq!(piece_view.linked_exercises[0].key, Some("G".to_string()));
        assert_eq!(
            piece_view.linked_exercises[0].tempo,
            Some("80 BPM".to_string())
        );
        assert!(piece_view.used_in.is_empty(), "pieces carry no usage rows");

        let ex1_view = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex1_view.used_in.len(), 1);
        assert!(ex1_view.used_in[0].linked);
        assert_eq!(ex1_view.used_in[0].piece.as_ref().unwrap().id, "piece-1");
        assert_eq!(ex1_view.used_in[0].piece.as_ref().unwrap().title, "Sonata");
        assert!(ex1_view.linked_exercises.is_empty());

        let ex2_view = vm.items.iter().find(|i| i.id == "ex-2").unwrap();
        assert_eq!(ex2_view.used_in.len(), 1);
        assert_eq!(ex2_view.used_in[0].piece.as_ref().unwrap().id, "piece-1");
        assert!(ex2_view.linked_exercises.is_empty());

        // Unrelated exercise: both lists empty.
        let ex3_view = vm.items.iter().find(|i| i.id == "ex-3").unwrap();
        assert!(ex3_view.linked_exercises.is_empty());
        assert!(ex3_view.used_in.is_empty());
    }

    // ── Used in: links merged with practice history (#1363) ──────────────

    /// A link is an intention, so the row has to exist before any history does.
    #[test]
    fn used_in_includes_a_linked_piece_with_no_practice() {
        let app = Intrada;
        let mut piece = ctx_item("P", "Sonata", ItemKind::Piece, Some("Beethoven"));
        piece.linked_exercise_ids = vec!["ex-1".to_string()];

        let model = Model {
            items: vec![piece, ctx_item("ex-1", "Scales", ItemKind::Exercise, None)],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex1 = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex1.used_in.len(), 1, "the link alone makes a row");

        let row = &ex1.used_in[0];
        assert!(row.linked, "seeded from the piece's link");
        assert_eq!(row.piece.as_ref().unwrap().id, "P");
        assert_eq!(
            row.piece.as_ref().unwrap().subtitle.as_deref(),
            Some("Beethoven")
        );
        assert_eq!(row.latest_score, None, "never practised together");
        assert_eq!(row.session_count, 0);
        assert_eq!(row.last_practiced_at, None);
        assert!(!row.piece_removed);
    }

    /// `linked: false` is what the shell hangs its "Link" action off.
    #[test]
    fn used_in_marks_a_practised_piece_that_is_not_linked() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            items: vec![
                ctx_item("P", "Sonata", ItemKind::Piece, Some("Beethoven")),
                ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
            ],
            sessions: vec![ctx_session(
                "s1",
                now,
                vec![
                    ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(7), Some("g1")),
                    ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g1")),
                ],
            )],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex1 = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex1.used_in.len(), 1);

        let row = &ex1.used_in[0];
        assert!(!row.linked, "history without a link");
        assert_eq!(row.latest_score, Some(7));
        assert_eq!(row.session_count, 1);
    }

    /// Two lists disagreeing is the bug #1363 fixes, so the merge is the test.
    #[test]
    fn used_in_merges_a_linked_and_practised_piece_into_one_row() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut piece = ctx_item("P", "Sonata", ItemKind::Piece, Some("Beethoven"));
        piece.linked_exercise_ids = vec!["ex-1".to_string()];

        let model = Model {
            items: vec![piece, ctx_item("ex-1", "Scales", ItemKind::Exercise, None)],
            sessions: vec![ctx_session(
                "s1",
                now,
                vec![
                    ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(7), Some("g1")),
                    ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g1")),
                ],
            )],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex1 = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex1.used_in.len(), 1, "one row, both sources");
        assert!(ex1.used_in[0].linked);
        assert_eq!(ex1.used_in[0].latest_score, Some(7));
        assert_eq!(ex1.used_in[0].session_count, 1);
    }

    #[test]
    fn used_in_orders_practised_first_then_linked_only_then_on_its_own() {
        let app = Intrada;
        let old = chrono::Utc::now() - chrono::Duration::days(9);
        let recent = chrono::Utc::now() - chrono::Duration::days(1);
        let solo_day = chrono::Utc::now() - chrono::Duration::days(4);

        // Ids deliberately run counter to the titles, so an id-only tie-break
        // would order these the other way round.
        let mut zeta = ctx_item("p-1", "Zeta", ItemKind::Piece, None);
        zeta.linked_exercise_ids = vec!["ex-1".to_string()];
        let mut alpha = ctx_item("p-2", "Alpha", ItemKind::Piece, None);
        alpha.linked_exercise_ids = vec!["ex-1".to_string()];

        let model = Model {
            items: vec![
                ctx_item("p-old", "Older practice", ItemKind::Piece, None),
                ctx_item("p-recent", "Recent practice", ItemKind::Piece, None),
                zeta,
                alpha,
                ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
            ],
            sessions: vec![
                ctx_session(
                    "s-old",
                    old,
                    vec![
                        ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(3), Some("g1")),
                        ctx_entry("p-old", "Older practice", ItemKind::Piece, None, Some("g1")),
                    ],
                ),
                ctx_session(
                    "s-solo",
                    solo_day,
                    vec![ctx_entry(
                        "ex-1",
                        "Scales",
                        ItemKind::Exercise,
                        Some(5),
                        None,
                    )],
                ),
                ctx_session(
                    "s-recent",
                    recent,
                    vec![
                        ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(8), Some("g2")),
                        ctx_entry(
                            "p-recent",
                            "Recent practice",
                            ItemKind::Piece,
                            None,
                            Some("g2"),
                        ),
                    ],
                ),
            ],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex1 = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        let order: Vec<&str> = ex1
            .used_in
            .iter()
            .map(|c| c.piece.as_ref().map(|p| p.id.as_str()).unwrap_or("solo"))
            .collect();
        assert_eq!(
            order,
            vec!["p-recent", "p-old", "p-2", "p-1", "solo"],
            "practised newest-first, then linked-only by title, then on its own"
        );
    }

    /// Same kind filter the piece side already applies, in reverse. Asserted on
    /// the derivation rather than the view: the view drops these anyway (only
    /// exercises are given a `used_in`), so a view-level assert would pass with
    /// the filter deleted.
    #[test]
    fn used_in_derivation_seeds_nothing_for_a_link_to_a_missing_or_wrong_kind_item() {
        let mut piece = ctx_item("P", "Sonata", ItemKind::Piece, Some("Beethoven"));
        piece.linked_exercise_ids = vec!["gone".to_string(), "other-piece".to_string()];

        let model = Model {
            items: vec![
                piece,
                ctx_item("other-piece", "Nocturne", ItemKind::Piece, None),
                ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
            ],
            ..Default::default()
        };
        let index: std::collections::HashMap<&str, &crate::domain::item::Item> =
            model.items.iter().map(|i| (i.id.as_str(), i)).collect();

        let usage = build_exercise_usage(&model, &index);
        assert!(
            usage.is_empty(),
            "a dangling id and a piece-kind target both seed nothing, got {usage:?}"
        );
    }

    /// A piece deleted since it was practised sinks below every live piece,
    /// including ones with no practice at all.
    #[test]
    fn used_in_sorts_removed_pieces_below_live_ones() {
        let recent = chrono::Utc::now() - chrono::Duration::days(1);

        let mut linked_only = ctx_item("p-live", "Live linked", ItemKind::Piece, None);
        linked_only.linked_exercise_ids = vec!["ex-1".to_string()];

        let model = Model {
            items: vec![
                linked_only,
                ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
            ],
            // "p-gone" is practised and recent, but no longer in the library.
            sessions: vec![ctx_session(
                "s-gone",
                recent,
                vec![
                    ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(9), Some("g1")),
                    ctx_entry("p-gone", "Deleted piece", ItemKind::Piece, None, Some("g1")),
                ],
            )],
            ..Default::default()
        };

        let vm = Intrada.view(&model);
        let ex1 = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        let order: Vec<&str> = ex1
            .used_in
            .iter()
            .map(|r| r.piece.as_ref().map(|p| p.id.as_str()).unwrap_or("solo"))
            .collect();
        assert_eq!(
            order,
            vec!["p-live", "p-gone"],
            "the removed piece sinks despite being the only one practised"
        );
        assert!(ex1.used_in[1].piece_removed);
    }

    // ── Exercise context derivation (#1087 B1) ───────────────────────────

    fn ctx_entry(
        item_id: &str,
        title: &str,
        kind: ItemKind,
        score: Option<u8>,
        group: Option<&str>,
    ) -> SetlistEntry {
        SetlistEntry {
            id: format!("{item_id}-{}", group.unwrap_or("solo")),
            item_id: item_id.to_string(),
            item_title: title.to_string(),
            item_type: kind,
            position: 0,
            duration_secs: 300,
            status: EntryStatus::Completed,
            notes: None,
            intention: None,
            planned_duration_secs: None,
            group_id: group.map(String::from),
            planned_variation_id: None,
            planned_rep_target: None,
            plays: vec![VariationPlay {
                seconds: 300,
                achieved_tempo: None,
                score,
                ..VariationPlay::fixture()
            }],
        }
    }

    fn ctx_session(
        id: &str,
        started: chrono::DateTime<chrono::Utc>,
        entries: Vec<SetlistEntry>,
    ) -> PracticeSession {
        PracticeSession {
            id: id.to_string(),
            started_at: started,
            completed_at: started,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries,
        }
    }

    fn ctx_item(id: &str, title: &str, kind: ItemKind, composer: Option<&str>) -> Item {
        let now = chrono::Utc::now();
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind,
            composer: composer.map(String::from),
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
            photo_id: None,
            metre: None,
        }
    }

    /// The picker's captions come from the unfiltered library: a Library
    /// search that hides the current exercise must not empty them (#1784).
    #[test]
    fn a_library_filter_cannot_empty_the_players_variation_picker() {
        let app = Intrada;
        let mut entry = variant_entry("ex-1", "ex-1-v0", None);
        entry.plays[0].seconds = 250;
        entry.plays.push(VariationPlay {
            id: "play-2".to_string(),
            variation_id: Some("ex-1-v1".to_string()),
            seconds: 0,
            ..VariationPlay::fixture()
        });
        let model = Model {
            items: vec![exercise_with_variants("ex-1", &["C", "G"])],
            session_status: SessionStatus::Active(crate::domain::session::ActiveSession {
                id: "as1".to_string(),
                entries: vec![entry],
                current_index: 0,
                current_item_started_at: chrono::Utc::now(),
                session_started_at: chrono::Utc::now(),
            }),
            active_query: Some(ListQuery {
                item_type: Some(ItemKind::Piece),
                ..Default::default()
            }),
            ..Model::default()
        };

        let vm = app.view(&model);
        assert!(
            vm.items.is_empty(),
            "the filter really does hide the exercise"
        );
        let captions: Vec<&str> = vm
            .active_session
            .as_ref()
            .expect("an active session")
            .current_variations
            .iter()
            .map(|v| v.caption.as_str())
            .collect();
        assert_eq!(captions, ["Played this session · 4m 10s", "Playing now"]);
    }

    // ── Variation derivation (#1083 C1) ──

    fn variant_entry(item_id: &str, variant_id: &str, score: Option<u8>) -> SetlistEntry {
        let mut e = ctx_entry(item_id, "Scales", ItemKind::Exercise, score, None);
        e.planned_variation_id = Some(variant_id.to_string());
        e.plays[0].variation_id = Some(variant_id.to_string());
        e
    }

    fn exercise_with_variants(id: &str, labels: &[&str]) -> Item {
        let mut ex = ctx_item(id, "Scales", ItemKind::Exercise, None);
        let now = chrono::Utc::now();
        ex.variants = labels
            .iter()
            .enumerate()
            .map(|(i, l)| crate::domain::item::Variant {
                id: format!("{id}-v{i}"),
                label: l.to_string(),
                position: i,
                updated_at: now,
                deleted_at: None,
            })
            .collect();
        ex
    }

    fn derived_variants(
        item: &Item,
        sessions: &[PracticeSession],
    ) -> Vec<crate::model::VariantView> {
        build_variant_views(item, &build_variant_score_index(sessions))
    }

    #[test]
    fn variant_views_derive_latest_score_and_solidity() {
        let ex = exercise_with_variants("ex-1", &["F", "Bb", "Eb"]);
        let t0 = chrono::Utc::now() - chrono::Duration::days(2);
        let t1 = chrono::Utc::now();
        let sessions = vec![
            ctx_session("s0", t0, vec![variant_entry("ex-1", "ex-1-v0", Some(4))]),
            ctx_session(
                "s1",
                t1,
                vec![
                    variant_entry("ex-1", "ex-1-v0", Some(9)),
                    variant_entry("ex-1", "ex-1-v1", Some(5)),
                ],
            ),
        ];

        let variations = derived_variants(&ex, &sessions);

        assert_eq!(variations.len(), 3);
        assert_eq!(
            variations[0].latest_score,
            Some(9),
            "latest session wins over older"
        );
        assert!(variations[0].is_solid, "score >= threshold is solid");
        assert_eq!(variations[1].latest_score, Some(5));
        assert!(!variations[1].is_solid);
        assert_eq!(
            variations[2].latest_score, None,
            "unpractised variation has no score"
        );
        assert!(!variations[2].is_solid);
    }

    /// The Library row and the practice session's Switch variation sheet read
    /// the same saved-mark caption, written once in the core (#1809).
    #[test]
    fn variant_views_caption_the_saved_mark() {
        let ex = exercise_with_variants("ex-1", &["F", "Bb", "Eb"]);
        let now = chrono::Utc::now();
        let sessions = vec![ctx_session(
            "s1",
            now,
            vec![
                variant_entry("ex-1", "ex-1-v0", Some(9)),
                variant_entry("ex-1", "ex-1-v1", Some(5)),
            ],
        )];

        let captions: Vec<String> = derived_variants(&ex, &sessions)
            .into_iter()
            .map(|v| v.caption)
            .collect();

        assert_eq!(captions, ["Solid · 9 of 10", "5 of 10", "Not yet played"]);
    }

    #[test]
    fn variant_views_scope_scores_to_this_item() {
        let ex = exercise_with_variants("ex-1", &["F"]);
        let now = chrono::Utc::now();
        // A different item reusing the same variant-id string must not leak in.
        let sessions = vec![ctx_session(
            "s1",
            now,
            vec![variant_entry("other", "ex-1-v0", Some(10))],
        )];
        let variations = derived_variants(&ex, &sessions);
        assert_eq!(
            variations[0].latest_score, None,
            "scores are scoped to this item"
        );
    }

    /// The core B1 derivation: an exercise practised in a piece's block twice
    /// and standalone once yields two contexts — the piece (latest score, count,
    /// date rolled up) then the "On its own" bucket last.
    #[test]
    fn test_used_in_derives_piece_and_on_its_own() {
        let app = Intrada;
        let d1 = chrono::Utc::now() - chrono::Duration::days(5);
        let d2 = chrono::Utc::now() - chrono::Duration::days(3);
        let d3 = chrono::Utc::now() - chrono::Duration::days(1);

        let model = Model {
            items: vec![
                ctx_item("P", "Sonata", ItemKind::Piece, Some("Beethoven")),
                ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
            ],
            sessions: vec![
                // s1 (earliest): ex-1 scored 4 in block g1 with piece P.
                ctx_session(
                    "s1",
                    d1,
                    vec![
                        ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(4), Some("g1")),
                        ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g1")),
                    ],
                ),
                // s2 (middle): ex-1 scored 6 again in a P block.
                ctx_session(
                    "s2",
                    d2,
                    vec![
                        ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(6), Some("g2")),
                        ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g2")),
                    ],
                ),
                // s3 (latest): ex-1 scored 8 standalone.
                ctx_session(
                    "s3",
                    d3,
                    vec![ctx_entry(
                        "ex-1",
                        "Scales",
                        ItemKind::Exercise,
                        Some(8),
                        None,
                    )],
                ),
            ],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex1 = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex1.used_in.len(), 2, "piece + on-its-own");

        let piece_ctx = &ex1.used_in[0];
        let piece = piece_ctx.piece.as_ref().expect("piece context first");
        assert_eq!(piece.id, "P");
        assert_eq!(piece.title, "Sonata");
        assert_eq!(
            piece.subtitle.as_deref(),
            Some("Beethoven"),
            "composer live"
        );
        assert_eq!(
            piece_ctx.latest_score,
            Some(6),
            "latest scored in P is d2's 6"
        );
        assert_eq!(piece_ctx.session_count, 2);
        assert_eq!(piece_ctx.last_practiced_at, Some(d2.to_rfc3339()));

        let solo = &ex1.used_in[1];
        assert!(solo.piece.is_none(), "'On its own' bucket has no piece");
        assert_eq!(solo.latest_score, Some(8));
        assert_eq!(solo.session_count, 1);
        assert_eq!(solo.last_practiced_at, Some(d3.to_rfc3339()));

        // Pieces themselves carry no contexts.
        let piece_view = vm.items.iter().find(|i| i.id == "P").unwrap();
        assert!(piece_view.used_in.is_empty());
    }

    /// A grouped run with no piece entry (a dissolved block) falls into the
    /// "On its own" bucket, not a phantom piece context.
    #[test]
    fn test_used_in_grouped_without_piece_is_on_its_own() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            items: vec![ctx_item("ex-1", "Scales", ItemKind::Exercise, None)],
            sessions: vec![ctx_session(
                "s1",
                now,
                vec![ctx_entry(
                    "ex-1",
                    "Scales",
                    ItemKind::Exercise,
                    Some(5),
                    Some("orphan-group"),
                )],
            )],
            ..Default::default()
        };

        let ex1 = app
            .view(&model)
            .items
            .into_iter()
            .find(|i| i.id == "ex-1")
            .unwrap();
        assert_eq!(ex1.used_in.len(), 1);
        assert!(ex1.used_in[0].piece.is_none());
        assert_eq!(ex1.used_in[0].latest_score, Some(5));
    }

    // #1093 (1a): a live rename shows through; the snapshot title is only a
    // fallback for a since-deleted piece.
    #[test]
    fn test_exercise_context_prefers_live_title_over_snapshot() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            items: vec![
                ctx_item("P", "Sonata No. 14", ItemKind::Piece, Some("Beethoven")),
                ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
            ],
            sessions: vec![ctx_session(
                "s1",
                now,
                vec![
                    ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(6), Some("g1")),
                    ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g1")),
                ],
            )],
            ..Default::default()
        };

        let ex1 = app
            .view(&model)
            .items
            .into_iter()
            .find(|i| i.id == "ex-1")
            .unwrap();
        let ctx = &ex1.used_in[0];
        let piece = ctx.piece.as_ref().expect("piece context");
        assert_eq!(piece.title, "Sonata No. 14", "live title, not snapshot");
        assert_eq!(piece.subtitle.as_deref(), Some("Beethoven"));
        assert!(!ctx.piece_removed, "piece still exists");
    }

    // #1093 (2a): a deleted piece's context is kept (real history) with the
    // snapshot title, no composer, and `piece_removed` set — not filtered out.
    #[test]
    fn test_exercise_context_keeps_removed_piece_as_snapshot_history() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            items: vec![ctx_item("ex-1", "Scales", ItemKind::Exercise, None)],
            sessions: vec![ctx_session(
                "s1",
                now,
                vec![
                    ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(5), Some("g1")),
                    ctx_entry("P", "Autumn Leaves", ItemKind::Piece, None, Some("g1")),
                ],
            )],
            ..Default::default()
        };

        let ex1 = app
            .view(&model)
            .items
            .into_iter()
            .find(|i| i.id == "ex-1")
            .unwrap();
        assert_eq!(ex1.used_in.len(), 1, "removed piece kept, not dropped");
        let ctx = &ex1.used_in[0];
        let piece = ctx.piece.as_ref().expect("context retained");
        assert_eq!(piece.id, "P");
        assert_eq!(piece.title, "Autumn Leaves", "snapshot title survives");
        assert_eq!(piece.subtitle, None, "no live composer for a gone piece");
        assert!(ctx.piece_removed, "flagged removed");
        assert_eq!(ctx.latest_score, Some(5), "its history still counts");
    }

    // #1087 B2: the piece's linked-exercise row scores the exercise *on this
    // piece* — 7 with the piece must win over a later, higher standalone 9.
    #[test]
    fn test_linked_exercise_carries_per_piece_context_score() {
        let app = Intrada;
        let earlier = chrono::Utc::now() - chrono::Duration::days(1);
        let later = chrono::Utc::now();
        let mut piece = ctx_item("P", "Sonata", ItemKind::Piece, None);
        piece.linked_exercise_ids = vec!["ex-1".to_string()];
        let model = Model {
            items: vec![piece, ctx_item("ex-1", "Scales", ItemKind::Exercise, None)],
            sessions: vec![
                ctx_session(
                    "s1",
                    earlier,
                    vec![
                        ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(7), Some("g1")),
                        ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g1")),
                    ],
                ),
                ctx_session(
                    "s2",
                    later,
                    vec![ctx_entry(
                        "ex-1",
                        "Scales",
                        ItemKind::Exercise,
                        Some(9),
                        None,
                    )],
                ),
            ],
            ..Default::default()
        };

        let piece_view = app
            .view(&model)
            .items
            .into_iter()
            .find(|i| i.id == "P")
            .unwrap();
        let linked = &piece_view.linked_exercises[0];
        assert_eq!(
            linked.piece_context_score,
            Some(7),
            "score on this piece, not the standalone 9"
        );
    }

    /// A pure projection over `model.sessions`, so it holds for sessions that
    /// arrive from the on-device store rather than being derived at write time.
    #[test]
    fn test_used_in_derives_from_stored_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let items = vec![
            ctx_item("P", "Sonata", ItemKind::Piece, None),
            ctx_item("ex-1", "Scales", ItemKind::Exercise, None),
        ];
        let sessions = vec![ctx_session(
            "s1",
            now,
            vec![
                ctx_entry("ex-1", "Scales", ItemKind::Exercise, Some(7), Some("g1")),
                ctx_entry("P", "Sonata", ItemKind::Piece, None, Some("g1")),
            ],
        )];

        let mut local = Model {
            items,
            ..Default::default()
        };
        let _ = app.update(
            Event::SessionsStoreLoaded(PersistenceOutput::Sessions(sessions)),
            &mut local,
        );
        let local_ctx = app
            .view(&local)
            .items
            .into_iter()
            .find(|i| i.id == "ex-1")
            .unwrap()
            .used_in;

        assert_eq!(local_ctx.len(), 1);
        assert_eq!(local_ctx[0].piece.as_ref().unwrap().id, "P");
        assert_eq!(local_ctx[0].latest_score, Some(7));
    }

    #[test]
    fn test_reverse_index_drops_non_exercise_kind() {
        // Symmetric to the forward-path kind filter: if a linked id resolves to a
        // Piece (not an Exercise), both views must drop it — the forward
        // linked_exercises already does this; the reverse used_in must too.
        let app = Intrada;
        let now = chrono::Utc::now();

        // Piece P links "item-b", which is itself a Piece (not an Exercise).
        let model = Model {
            items: vec![
                Item {
                    id: "piece-a".to_string(),
                    title: "Sonata".to_string(),
                    kind: ItemKind::Piece,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec!["item-b".to_string()],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
                Item {
                    id: "item-b".to_string(),
                    title: "Not An Exercise".to_string(),
                    kind: ItemKind::Piece,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                    photo_id: None,
                    metre: None,
                },
            ],
            ..Default::default()
        };

        let vm = app.view(&model);

        // Forward: piece-a drops item-b (wrong kind).
        let piece_a = vm.items.iter().find(|i| i.id == "piece-a").unwrap();
        assert!(
            piece_a.linked_exercises.is_empty(),
            "forward path must drop non-Exercise from linked_exercises"
        );

        // Reverse: covered on the derivation itself by
        // `used_in_derivation_seeds_nothing_for_a_link_to_a_missing_or_wrong_kind_item`
        // — a view-level assert here passes whether or not the filter exists.
    }
    // ── Variation ladder view derivation (#1083 C1) ─────────────────────────

    fn laddered_exercise(id: &str) -> Item {
        use crate::domain::variant::Variant;
        let now = chrono::Utc::now();
        let mut item = make_item(id, "Shells", ItemKind::Exercise, now);
        item.variants = vec![
            Variant {
                id: "v-f".to_string(),
                label: "F".to_string(),
                position: 1,
                updated_at: now,
                deleted_at: None,
            },
            Variant {
                id: "v-c".to_string(),
                label: "C".to_string(),
                position: 0,
                updated_at: now,
                deleted_at: None,
            },
            Variant {
                id: "v-g".to_string(),
                label: "G".to_string(),
                position: 2,
                updated_at: now,
                deleted_at: Some(now),
            },
        ];
        item
    }

    fn step_session(
        id: &str,
        item_id: &str,
        variant_id: Option<&str>,
        score: Option<u8>,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> PracticeSession {
        PracticeSession {
            id: id.to_string(),
            started_at,
            completed_at: started_at,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_score: None,
            entries: vec![SetlistEntry {
                id: format!("{id}-e1"),
                item_id: item_id.to_string(),
                item_title: "Shells".to_string(),
                item_type: ItemKind::Exercise,
                position: 0,
                duration_secs: 300,
                status: EntryStatus::Completed,
                notes: None,
                intention: None,
                planned_duration_secs: None,
                group_id: None,
                planned_variation_id: variant_id.map(str::to_string),
                planned_rep_target: None,
                plays: vec![VariationPlay {
                    id: format!("{id}-e1-play"),
                    variation_id: variant_id.map(str::to_string),
                    seconds: 300,
                    score,
                    ..VariationPlay::fixture()
                }],
            }],
        }
    }

    fn step_view_model(sessions: Vec<PracticeSession>) -> ViewModel {
        let app = Intrada;
        let mut model = Model {
            items: vec![laddered_exercise("ex-1")],
            sessions,
            ..Default::default()
        };
        model.practice_summaries = build_practice_summaries(&model.sessions);
        app.view(&model)
    }

    #[test]
    fn view_carries_this_week_with_no_sessions_and_today_with_one() {
        let empty = Intrada.view(&Model::default());
        assert_eq!(empty.practice_weeks.len(), 1);
        assert!(empty.practice_weeks[0].days.iter().any(|d| d.is_today));

        let vm = step_view_model(vec![make_session("s-now", "ex-1", None, None)]);
        let this_week = vm.practice_weeks.last().expect("this week");
        let today = this_week.days.iter().find(|d| d.is_today).expect("today");
        assert_eq!(today.session_ids, vec!["s-now"]);
        assert_eq!(this_week.days[this_week.opening_day].date, today.date);
    }

    #[test]
    fn view_exposes_live_steps_sorted_by_position_tombstones_excluded() {
        let vm = step_view_model(vec![]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(
            ex.variants
                .iter()
                .map(|v| (v.id.as_str(), v.label.as_str(), v.position))
                .collect::<Vec<_>>(),
            vec![("v-c", "C", 0), ("v-f", "F", 1)],
            "live variations only, in ladder order"
        );
    }

    #[test]
    fn view_derives_per_step_latest_score_and_history() {
        let t0 = chrono::Utc::now();
        let vm = step_view_model(vec![
            step_session("s1", "ex-1", Some("v-c"), Some(5), t0),
            step_session(
                "s2",
                "ex-1",
                Some("v-c"),
                Some(7),
                t0 + chrono::Duration::days(1),
            ),
            // A flat (unattributed) score never counts towards a variation.
            step_session("s3", "ex-1", None, Some(9), t0 + chrono::Duration::days(2)),
            step_session(
                "s4",
                "ex-1",
                Some("v-f"),
                Some(3),
                t0 + chrono::Duration::days(3),
            ),
        ]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        let c = ex.variants.iter().find(|v| v.id == "v-c").unwrap();
        assert_eq!(c.latest_score, Some(7));
        assert_eq!(c.score_history.len(), 2);
        assert_eq!(c.score_history[0].score, 7, "history is newest-first");
        let f = ex.variants.iter().find(|v| v.id == "v-f").unwrap();
        assert_eq!(f.latest_score, Some(3));
        assert_eq!(f.score_history.len(), 1);
    }

    #[test]
    fn view_marks_solid_steps() {
        let t0 = chrono::Utc::now();
        let vm = step_view_model(vec![
            step_session("s1", "ex-1", Some("v-c"), Some(8), t0),
            step_session(
                "s2",
                "ex-1",
                Some("v-f"),
                Some(7),
                t0 + chrono::Duration::days(1),
            ),
        ]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        let c = ex.variants.iter().find(|v| v.id == "v-c").unwrap();
        assert!(c.is_solid, "8 of 10 is solid");
        let f = ex.variants.iter().find(|v| v.id == "v-f").unwrap();
        assert!(!f.is_solid, "7 of 10 is not yet solid");
    }

    #[test]
    fn view_marks_a_ladder_of_key_names_as_keys() {
        let vm = step_view_model(vec![]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.ladder_is_keys, "C and F are both keys");
    }

    #[test]
    fn view_one_non_key_rung_makes_the_whole_ladder_steps() {
        let app = Intrada;
        let mut exercise = laddered_exercise("ex-1");
        exercise.variants[0].label = "Hands together".to_string();
        let model = Model {
            items: vec![exercise],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(!ex.ladder_is_keys, "one non-key rung and \"keys\" is a lie");
    }

    /// A rung the user removed is not on screen, so it cannot change the word.
    #[test]
    fn view_a_tombstoned_non_key_rung_leaves_the_ladder_reading_as_keys() {
        let app = Intrada;
        let mut exercise = laddered_exercise("ex-1");
        let tombstoned = exercise
            .variants
            .iter_mut()
            .find(|v| v.deleted_at.is_some())
            .unwrap();
        tombstoned.label = "Hands together".to_string();
        let model = Model {
            items: vec![exercise],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.ladder_is_keys, "the removed rung is not on the ladder");
    }

    #[test]
    fn view_an_item_with_no_ladder_is_not_keys() {
        let app = Intrada;
        let model = Model {
            items: vec![make_item(
                "p-1",
                "Clair de Lune",
                ItemKind::Piece,
                chrono::Utc::now(),
            )],
            ..Default::default()
        };

        let vm = app.view(&model);
        let piece = vm.items.iter().find(|i| i.id == "p-1").unwrap();
        assert!(!piece.ladder_is_keys, "no rungs is not a ladder of keys");
    }

    #[test]
    fn view_an_exercise_with_live_variations_hides_the_key_field() {
        let app = Intrada;
        let model = Model {
            items: vec![laddered_exercise("ex-1")],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(
            !ex.shows_key,
            "two live rungs, one tombstoned, still hides Key"
        );
    }

    #[test]
    fn view_an_un_laddered_exercise_shows_the_key_field() {
        let app = Intrada;
        let model = Model {
            items: vec![make_item(
                "ex-1",
                "Shells",
                ItemKind::Exercise,
                chrono::Utc::now(),
            )],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.shows_key, "no rungs, nothing to hide the field for");
    }

    #[test]
    fn view_a_piece_shows_the_key_field() {
        let app = Intrada;
        let model = Model {
            items: vec![make_item(
                "p-1",
                "Clair de Lune",
                ItemKind::Piece,
                chrono::Utc::now(),
            )],
            ..Default::default()
        };

        let vm = app.view(&model);
        let piece = vm.items.iter().find(|i| i.id == "p-1").unwrap();
        assert!(piece.shows_key, "a piece never has a ladder to hide it for");
    }

    /// A tombstoned rung left alone once the last live one goes: the field
    /// comes back, mirroring `ladder_is_all_keys`'s own tombstone handling.
    #[test]
    fn view_an_exercise_with_only_tombstoned_variations_shows_the_key_field() {
        let app = Intrada;
        let mut exercise = laddered_exercise("ex-1");
        for v in &mut exercise.variants {
            v.deleted_at = Some(chrono::Utc::now());
        }
        let model = Model {
            items: vec![exercise],
            ..Default::default()
        };

        let vm = app.view(&model);
        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.shows_key, "nothing live on the ladder");
    }

    /// Positional bincode has no "absent": a new `LibraryItemView` field that
    /// does not survive the wire is a silent no-op, not a crash (#846).
    #[test]
    fn library_item_view_round_trips_on_the_ffi_bincode_wire() {
        let mut view = LibraryItemView::fixture("ex-1", "Shells", ItemKind::Exercise);
        view.variants = vec![crate::model::VariantView::fixture("v-c", "C", 0)];
        view.ladder_is_keys = true;
        view.shows_key = false;
        crate::domain::types::assert_round_trips(view);
    }

    #[test]
    fn view_session_entries_expose_the_variation_played() {
        let vm = step_view_model(vec![step_session(
            "s1",
            "ex-1",
            Some("v-c"),
            Some(6),
            chrono::Utc::now(),
        )]);

        assert_eq!(
            vm.sessions[0].entries[0].plays[0].variation_id.as_deref(),
            Some("v-c"),
            "history entries carry their variation through the view"
        );
    }
}

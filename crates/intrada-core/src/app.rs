// The `#[effect]` macro generates large variant size differences and we can't Box through it.
#![allow(clippy::large_enum_variant)]

use crux_core::capability::Operation;
use crux_core::macros::effect;
use crux_core::render::RenderOperation;
use crux_core::{App, Command};
use crux_http::HttpRequest;
use serde::{Deserialize, Serialize};

use crate::analytics::compute_analytics;
use crate::domain::account::{handle_account_event, AccountEvent};
use crate::domain::item::{handle_item_event, Item, ItemEvent, ItemKind};
use crate::domain::mcp_audit::{handle_mcp_audit_event, McpAuditEvent};
use crate::domain::mcp_tokens::{handle_mcp_token_event, McpTokenEvent};
use crate::domain::oauth::{handle_oauth_event, OAuthEvent};
use crate::domain::session::{
    handle_session_event, ActiveSession, PracticeSession, SessionEvent, SessionStatus,
};
#[cfg(test)]
use crate::domain::session::{CompletionStatus, EntryStatus, SetlistEntry};
use crate::domain::set::{handle_set_event, Set, SetEvent};
use crate::domain::types::{LibrarySort, ListQuery, SortDirection, SortField};
use crate::http;
use crate::model::{
    build_active_session_view, build_blocks, build_summary_view, entry_to_view, session_to_view,
    BuildingSetlistView, ItemPracticeSummary, LibraryItemView, LinkedExerciseView, Model,
    PieceRefView, ScaffoldPreviewView, ScaffoldSpecView, SessionStatusView, SetSourceStatus,
    ViewModel,
};
use crate::persistence::{self, PersistenceOperation, PersistenceOutput};

/// Root Crux application for the music practice library.
#[derive(Default)]
pub struct Intrada;

/// All events the application can process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Event {
    // ── Lifecycle ────────────────────────────────────────────────────
    /// Shell provides the API base URL on startup.
    /// Named `StartApp` (not `Init`) to avoid Swift keyword collision.
    StartApp {
        api_base_url: String,
        /// iOS passes true (Library local-first); web passes false (online).
        local_first: bool,
    },
    /// Demo dataset, opt-in only (e.g. iOS `--seed-sample-data`) — never in production.
    LoadSampleData,
    /// Fetch all data from the API (items, sessions, sets).
    FetchAll,
    /// Re-fetch a single resource kind after a mutation (refresh-after-mutate).
    RefetchItems,
    RefetchSessions,
    RefetchSets,
    /// Reset all user-scoped state so the next sign-in doesn't inherit the
    /// previous user's data (#645).
    SignedOut,

    // ── Domain ──────────────────────────────────────────────────────
    Item(ItemEvent),
    Session(SessionEvent),
    Set(SetEvent),
    Account(AccountEvent),
    McpToken(McpTokenEvent),
    McpAudit(McpAuditEvent),
    OAuth(OAuthEvent),

    // ── Data loaded callbacks ───────────────────────────────────────
    DataLoaded {
        items: Vec<Item>,
    },
    SessionsLoaded {
        sessions: Vec<PracticeSession>,
    },
    SetsLoaded {
        sets: Vec<Set>,
    },

    // ── Write-confirmation callbacks ────────────────────────────────
    // Temp-id mutate-response: see CLAUDE.md "Mutate response".
    ItemCreated {
        temp_id: String,
        item: Item,
    },
    ItemUpdated {
        item: Item,
    },
    SetUpdated {
        set: Set,
    },
    /// Server confirmed `Save{Building,Summary}AsSet`. `request_id` echoes
    /// the shell's dispatch tag so per-form promotion stays isolated (#663).
    SetSaveSucceeded {
        request_id: String,
    },
    /// Server confirmed a delete — model already updated optimistically.
    DeleteConfirmed,
    /// Server confirmed session creation — model already updated optimistically.
    SessionSaved,

    // ── Error handling ──────────────────────────────────────────────
    LoadFailed(String),
    ClearError,
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
}

/// Side effects the core requests from shells.
///
/// Variants hold operation types; the `#[effect]` macro wraps each in
/// `Request<Op>` in the compiled enum.
#[effect(facet_typegen)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    /// Shell-only side effects that are NOT HTTP (localStorage only).
    App(AppEffect),
    /// Local-first persistence (the core's first effect with typed-data output).
    Persistence(PersistenceOperation),
}

/// Non-HTTP side-effect operations handled by the shell (localStorage only).
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
        let command = self.handle_event(event, model);
        if model.last_error.is_some() {
            model.error_seq = model.error_seq.wrapping_add(1);
        }
        command
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        self.build_view(model)
    }
}

impl Intrada {
    fn handle_event(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            // ── Lifecycle ────────────────────────────────────────────
            Event::StartApp {
                api_base_url,
                local_first,
            } => {
                model.api_base_url = api_base_url;
                model.local_first = local_first;
                if local_first {
                    Command::all([persistence::load_items(), persistence::load_sessions()])
                } else {
                    Command::all([
                        http::fetch_items(&model.api_base_url),
                        http::fetch_sessions(&model.api_base_url),
                        http::fetch_sets(&model.api_base_url),
                    ])
                }
            }
            Event::LoadSampleData => {
                model.items = sample_items();
                model.sessions = sample_sessions();
                model.practice_summaries = build_practice_summaries(&model.sessions);
                // Seed mode is offline (DEBUG/CI) — keep writes local so a demo
                // edit doesn't surprise-POST to the API.
                model.local_first = true;
                // crux_http panics on a relative URL; demo mode skips StartApp, so set one.
                if model.api_base_url.is_empty() {
                    "http://localhost:3001".clone_into(&mut model.api_base_url);
                }
                model.last_error = None;
                crux_core::render::render()
            }
            Event::FetchAll => Command::all([
                http::fetch_items(&model.api_base_url),
                http::fetch_sessions(&model.api_base_url),
                http::fetch_sets(&model.api_base_url),
            ]),
            Event::RefetchItems => http::fetch_items(&model.api_base_url),
            Event::RefetchSessions => http::fetch_sessions(&model.api_base_url),
            Event::RefetchSets => http::fetch_sets(&model.api_base_url),
            Event::SignedOut => {
                model.reset_for_sign_out();
                // The crash-recovery blob isn't user-scoped, so clear it too —
                // else user A's session hydrates into user B on next sign-in (#645).
                Command::all([
                    Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
                    crux_core::render::render(),
                ])
            }

            // ── Domain handlers ──────────────────────────────────────
            Event::Item(item_event) => handle_item_event(item_event, model),
            Event::Session(session_event) => handle_session_event(session_event, model),
            Event::Set(set_event) => handle_set_event(set_event, model),
            Event::Account(account_event) => handle_account_event(account_event, model),
            Event::McpToken(token_event) => handle_mcp_token_event(token_event, model),
            Event::McpAudit(audit_event) => handle_mcp_audit_event(audit_event, model),
            Event::OAuth(oauth_event) => handle_oauth_event(oauth_event, model),

            // ── Data loaded callbacks ────────────────────────────────
            Event::DataLoaded { items } => {
                model.items = items;
                model.record_success();
                crux_core::render::render()
            }
            Event::SessionsLoaded { sessions } => {
                model.sessions = sessions;
                model.practice_summaries = build_practice_summaries(&model.sessions);
                model.record_success();
                crux_core::render::render()
            }
            Event::SetsLoaded { sets } => {
                model.sets = sets;
                model.record_success();
                crux_core::render::render()
            }

            // ── Write-confirmation callbacks ─────────────────────────
            Event::ItemCreated { temp_id, item } => {
                if let Some(existing) = model.items.iter_mut().find(|i| i.id == temp_id) {
                    *existing = item.clone();
                } else {
                    model.items.push(item.clone());
                }
                model.record_success();
                crux_core::render::render()
            }
            Event::ItemUpdated { item } => {
                if let Some(existing) = model.items.iter_mut().find(|i| i.id == item.id) {
                    *existing = item;
                }
                model.record_success();
                crux_core::render::render()
            }
            Event::SetUpdated { set } => {
                if let Some(existing) = model.sets.iter_mut().find(|r| r.id == set.id) {
                    *existing = set;
                }
                model.record_success();
                crux_core::render::render()
            }
            Event::DeleteConfirmed | Event::SessionSaved => {
                // Model already updated optimistically — just record the success.
                model.record_success();
                crux_core::render::render()
            }
            Event::SetSaveSucceeded { request_id } => {
                model.last_set_save_request_id = Some(request_id);
                model.record_success();
                crate::http::fetch_sets(&model.api_base_url)
            }

            // ── Error handling ───────────────────────────────────────
            Event::LoadFailed(msg) => {
                // surface_error does the dismiss-mute check + dedupe (#346); always
                // render anyway so domain *Failed state changes (rollback) flush.
                model.surface_error(msg);
                crux_core::render::render()
            }
            Event::ClearError => {
                model.dismiss_error();
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
                PersistenceOutput::Ack => Command::done(),
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
                PersistenceOutput::Ack => Command::done(),
                PersistenceOutput::Items(_) | PersistenceOutput::Sessions(_) => Command::done(),
                // Failed save → reload sessions to roll back the optimistic push (#825).
                PersistenceOutput::Failed => {
                    model.surface_error("Couldn't access local storage.");
                    persistence::load_sessions()
                }
            },
        }
    }

    fn build_view(&self, model: &Model) -> ViewModel {
        use std::collections::HashMap;

        // Build an id→item index for O(1) linked-exercise lookup.
        let item_index: HashMap<&str, &crate::domain::item::Item> =
            model.items.iter().map(|i| (i.id.as_str(), i)).collect();

        // Build a reverse index: exercise_id → [PieceRefView] in one pass.
        // Mirror the forward filter: only push a PieceRefView when the target
        // id resolves to a present item whose kind == Exercise.
        let mut piece_refs_by_exercise: HashMap<&str, Vec<PieceRefView>> = HashMap::new();
        for item in &model.items {
            if item.kind == ItemKind::Piece {
                for ex_id in &item.linked_exercise_ids {
                    if item_index
                        .get(ex_id.as_str())
                        .is_some_and(|t| t.kind == ItemKind::Exercise)
                    {
                        piece_refs_by_exercise
                            .entry(ex_id.as_str())
                            .or_default()
                            .push(PieceRefView {
                                id: item.id.clone(),
                                title: item.title.clone(),
                                subtitle: item.composer.clone(),
                            });
                    }
                }
            }
        }

        // Derived per-exercise practice contexts (piece + "on its own"), keyed
        // by exercise id. Built once over sessions, attached to exercises below.
        let contexts_by_exercise = build_exercise_contexts(&model.sessions, &item_index);

        // Per-step score history, keyed by (item id, variant id); one pass
        // over sessions, attached to laddered exercises below (#1083).
        let variant_scores = build_variant_score_index(&model.sessions);

        let mut items: Vec<LibraryItemView> = Vec::new();

        for item in &model.items {
            let practice = model.practice_summaries.get(&item.id).cloned();
            let subtitle = item.composer.clone().unwrap_or_default();
            let latest_achieved_tempo = practice.as_ref().and_then(|p| p.latest_tempo);

            let linked_exercises = if item.kind == ItemKind::Piece {
                item.linked_exercise_ids
                    .iter()
                    .filter_map(|ex_id| {
                        let ex = item_index.get(ex_id.as_str())?;
                        if ex.kind != ItemKind::Exercise {
                            return None;
                        }
                        // The exercise's score in *this piece's* context, pulled
                        // from the same derivation the exercise screen uses so
                        // both sides agree (#1087 B2).
                        let piece_context_score = contexts_by_exercise
                            .get(ex.id.as_str())
                            .and_then(|contexts| {
                                contexts
                                    .iter()
                                    .find(|c| {
                                        c.piece.as_ref().map(|p| p.id.as_str())
                                            == Some(item.id.as_str())
                                    })
                                    .and_then(|c| c.latest_score)
                            });
                        Some(LinkedExerciseView {
                            id: ex.id.clone(),
                            title: ex.title.clone(),
                            key: ex.key.clone(),
                            tempo: ex
                                .tempo
                                .as_ref()
                                .map(|t| t.format_display())
                                .filter(|s| !s.is_empty()),
                            practice: model.practice_summaries.get(&ex.id).cloned(),
                            piece_context_score,
                        })
                    })
                    .collect()
            } else {
                vec![]
            };

            let linked_from_pieces = if item.kind == ItemKind::Exercise {
                piece_refs_by_exercise
                    .get(item.id.as_str())
                    .cloned()
                    .unwrap_or_default()
            } else {
                vec![]
            };

            let exercise_contexts = if item.kind == ItemKind::Exercise {
                contexts_by_exercise
                    .get(item.id.as_str())
                    .cloned()
                    .unwrap_or_default()
            } else {
                vec![]
            };

            // `already_linked` uses the same reconciliation key `CommitScaffold`
            // does, so the read-only preview and the commit agree.
            let scaffold_preview = item.chord_chart.as_ref().map(|chart| {
                let (linked_kinds, linked_titles) =
                    crate::domain::item::linked_scaffold_state(model, &item.id);
                let specs = crate::domain::chart::derive_scaffold(chart);
                let mut fallback_total: u8 = 0;
                let spec_views = specs
                    .iter()
                    .map(|s| {
                        let fallback = s.fallback_count > 0;
                        if fallback {
                            fallback_total = fallback_total.saturating_add(1);
                        }
                        ScaffoldSpecView {
                            kind: s.kind,
                            title: s.title.clone(),
                            rationale: s.rationale.clone(),
                            key: s.key.clone(),
                            fallback,
                            already_linked: crate::domain::item::scaffold_already_linked(
                                &linked_kinds,
                                &linked_titles,
                                s.kind,
                                &s.title,
                            ),
                        }
                    })
                    .collect();
                ScaffoldPreviewView {
                    key: chart.key.clone(),
                    specs: spec_views,
                    fallback_total,
                }
            });

            let variants = if item.kind == ItemKind::Exercise {
                build_variant_views(item, &variant_scores)
            } else {
                vec![]
            };

            items.push(LibraryItemView {
                id: item.id.clone(),
                item_type: item.kind.clone(),
                title: item.title.clone(),
                subtitle,
                key: item.key.clone(),
                modality: item.modality,
                tempo: item
                    .tempo
                    .as_ref()
                    .map(|t| t.format_display())
                    .filter(|s| !s.is_empty()),
                tempo_marking: item.tempo.as_ref().and_then(|t| t.marking.clone()),
                tempo_bpm: item.tempo.as_ref().and_then(|t| t.bpm),
                notes: item.notes.clone(),
                // Reserved scaffold markers never reach the UI or the tag
                // vocabulary (`available_tags` derives from these view tags).
                tags: item
                    .tags
                    .iter()
                    .filter(|t| !crate::domain::chart::is_scaffold_tag(t))
                    .cloned()
                    .collect(),
                created_at: item.created_at.to_rfc3339(),
                updated_at: item.updated_at.to_rfc3339(),
                practice,
                latest_achieved_tempo,
                priority: item.priority,
                linked_exercises,
                linked_from_pieces,
                exercise_contexts,
                scaffold_preview,
                chord_chart: item.chord_chart.clone(),
                variants,
            });
        }

        // Computed before the filter so the vocabulary stays stable as the
        // filter narrows. Case-insensitive dedupe, first-seen casing.
        let available_tags = {
            let mut seen = std::collections::HashSet::new();
            let mut tags: Vec<String> = Vec::new();
            for item in &items {
                for tag in &item.tags {
                    if seen.insert(tag.to_lowercase()) {
                        tags.push(tag.clone());
                    }
                }
            }
            tags.sort_by_key(|t| t.to_lowercase());
            tags
        };

        // Like available_tags: whole-library vocabulary computed before the
        // filter, so a narrowed list can't collapse the pool. Case-folded dedupe.
        let available_composers = {
            let mut seen = std::collections::HashSet::new();
            let mut composers: Vec<String> = Vec::new();
            for item in &items {
                let composer = item.subtitle.trim();
                if !composer.is_empty() && seen.insert(composer.to_lowercase()) {
                    composers.push(composer.to_string());
                }
            }
            composers.sort_by_key(|c| c.to_lowercase());
            composers
        };

        if let Some(ref query) = model.active_query {
            items = apply_query_filter(items, query);
        }

        sort_library_items(&mut items, &model.active_sort);

        // Counted after the filter so the subtitle describes the visible set.
        let visible_pieces = items
            .iter()
            .filter(|i| i.item_type == ItemKind::Piece)
            .count();
        let visible_exercises = items
            .iter()
            .filter(|i| i.item_type == ItemKind::Exercise)
            .count();

        let mut sessions: Vec<_> = model.sessions.iter().map(session_to_view).collect();
        sessions.sort_by(|a, b| b.finished_at.cmp(&a.finished_at));

        let (active_session, building_setlist, summary) = match &model.session_status {
            SessionStatus::Idle => (None, None, None),
            SessionStatus::Building(building) => {
                let entries: Vec<_> = building.entries.iter().map(entry_to_view).collect();
                let item_count = entries.len();
                let blocks = build_blocks(&entries);
                let block_count = blocks.len();
                let source_status = match &building.source_set_id {
                    None => SetSourceStatus::NoSource,
                    Some(sid) => {
                        let set_name = model
                            .sets
                            .iter()
                            .find(|s| &s.id == sid)
                            .map(|s| s.name.clone());
                        match set_name {
                            None => SetSourceStatus::NoSource,
                            Some(name) => {
                                let current_ids: Vec<&str> = building
                                    .entries
                                    .iter()
                                    .map(|e| e.item_id.as_str())
                                    .collect();
                                let snapshot_ids: Vec<&str> = building
                                    .source_set_entry_snapshot
                                    .iter()
                                    .map(|s| s.as_str())
                                    .collect();
                                if current_ids == snapshot_ids {
                                    SetSourceStatus::UnmodifiedFromSource {
                                        set_id: sid.clone(),
                                        set_name: name,
                                    }
                                } else {
                                    SetSourceStatus::ModifiedFromSource {
                                        set_id: sid.clone(),
                                        set_name: name,
                                    }
                                }
                            }
                        }
                    }
                };
                let planned_total_secs: u64 = building
                    .entries
                    .iter()
                    .filter_map(|e| e.planned_duration_secs)
                    .map(u64::from)
                    .sum();
                let (total_duration_display, total_duration_summary) = if planned_total_secs > 0 {
                    (
                        Some(crate::domain::session::format_duration_display(
                            planned_total_secs,
                        )),
                        Some(crate::domain::session::format_planned_duration(
                            planned_total_secs,
                        )),
                    )
                } else {
                    (None, None)
                };
                (
                    None,
                    Some(BuildingSetlistView {
                        entries,
                        item_count,
                        blocks,
                        block_count,
                        total_duration_display,
                        total_duration_summary,
                        session_intention: building.session_intention.clone(),
                        target_duration_mins: building.target_duration_mins,
                        source_status,
                    }),
                    None,
                )
            }
            SessionStatus::Active(active) => (
                Some(build_active_session_view(active, &item_index)),
                None,
                None,
            ),
            SessionStatus::Summary(summary_session) => {
                (None, None, Some(build_summary_view(summary_session)))
            }
        };

        let session_status = match &model.session_status {
            SessionStatus::Idle => SessionStatusView::Idle,
            SessionStatus::Building(_) => SessionStatusView::Building,
            SessionStatus::Active(_) => SessionStatusView::Active,
            SessionStatus::Summary(_) => SessionStatusView::Summary,
        };

        // Uses Utc::now(), making view() impure — pragmatic tradeoff since the
        // date changes once/day; computation fns take `today` for testability.
        let analytics = if model.sessions.is_empty() {
            None
        } else {
            let today = chrono::Utc::now().date_naive();
            Some(compute_analytics(&model.sessions, &model.items, today))
        };

        let sets = model
            .sets
            .iter()
            .map(|r| {
                use crate::model::{SetEntryView, SetView};
                SetView {
                    id: r.id.clone(),
                    name: r.name.clone(),
                    entry_count: r.entries.len(),
                    entries: r
                        .entries
                        .iter()
                        .map(|e| SetEntryView {
                            id: e.id.clone(),
                            item_id: e.item_id.clone(),
                            item_title: e.item_title.clone(),
                            item_type: e.item_type.clone(),
                            position: e.position,
                        })
                        .collect(),
                }
            })
            .collect();

        ViewModel {
            items,
            active_query: model.active_query.clone(),
            active_sort: model.active_sort,
            visible_pieces,
            visible_exercises,
            available_tags,
            available_composers,
            sessions,
            active_session,
            building_setlist,
            summary,
            session_status,
            error: model.last_error.clone(),
            error_seq: model.error_seq,
            analytics,
            sets,
            account_preferences: model.account_preferences.clone(),
            delete_in_flight: model.delete_in_flight,
            account_deleted: model.account_deleted,
            mcp_tokens: model.mcp_tokens.clone(),
            mcp_audit: model.mcp_audit.clone(),
            mcp_audit_loaded: model.mcp_audit_loaded,
            mcp_audit_loading: model.mcp_audit_loading,
            mcp_tokens_loaded: model.mcp_tokens_loaded,
            mcp_tokens_loading: model.mcp_tokens_loading,
            just_created_token: model.just_created_token.clone(),
            oauth_in_flight: model.oauth_in_flight,
            oauth_redirect_url: model.oauth_redirect_url.clone(),
            last_set_save_request_id: model.last_set_save_request_id.clone(),
        }
    }
}

/// Build practice summaries (keyed by item_id) in a single pass over sessions.
/// Called once when sessions change, not per-render.
pub(crate) fn build_practice_summaries(
    sessions: &[PracticeSession],
) -> std::collections::HashMap<String, ItemPracticeSummary> {
    use crate::model::{ScoreHistoryEntry, TempoHistoryEntry};
    use std::collections::HashMap;

    // (count, secs, score_history, tempo_history, last_practiced_at)
    type Acc = (
        usize,
        u64,
        Vec<ScoreHistoryEntry>,
        Vec<TempoHistoryEntry>,
        Option<String>,
    );

    let mut acc: HashMap<String, Acc> = HashMap::new();

    for session in sessions {
        let session_date = session.started_at.to_rfc3339();
        for entry in &session.entries {
            let record = acc
                .entry(entry.item_id.clone())
                .or_insert_with(|| (0, 0, Vec::new(), Vec::new(), None));
            record.0 += 1;
            record.1 += entry.duration_secs;
            // Keep the latest date (RFC3339 strings compare chronologically).
            if record.4.as_ref().map_or(true, |cur| session_date > *cur) {
                record.4 = Some(session_date.clone());
            }

            if let Some(score) = entry.score {
                record.2.push(ScoreHistoryEntry {
                    session_date: session.started_at.to_rfc3339(),
                    score,
                    session_id: session.id.clone(),
                });
            }

            if let Some(tempo) = entry.achieved_tempo {
                record.3.push(TempoHistoryEntry {
                    session_date: session.started_at.to_rfc3339(),
                    tempo,
                    session_id: session.id.clone(),
                });
            }
        }
    }

    acc.into_iter()
        .map(
            |(
                item_id,
                (
                    session_count,
                    total_secs,
                    mut score_history,
                    mut tempo_history,
                    last_practiced_at,
                ),
            )| {
                score_history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
                let latest_score = score_history.first().map(|e| e.score);

                tempo_history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
                let latest_tempo = tempo_history.first().map(|e| e.tempo);

                (
                    item_id,
                    ItemPracticeSummary {
                        session_count,
                        total_minutes: (total_secs / 60) as u32,
                        latest_score,
                        score_history,
                        latest_tempo,
                        tempo_history,
                        last_practiced_at,
                    },
                )
            },
        )
        .collect()
}

/// Derive, per exercise, the piece/standalone contexts it has been practised
/// in across `sessions` (#1087 B1). For each session entry that is an exercise,
/// the context is the piece sharing its block `group_id` in that same session,
/// or the "On its own" bucket (`piece: None`) when the entry is ungrouped or its
/// block carries no piece. Rollup v1 keeps the latest recorded score, distinct
/// session count, and most-recent date per context.
fn build_exercise_contexts(
    sessions: &[PracticeSession],
    item_index: &std::collections::HashMap<&str, &crate::domain::item::Item>,
) -> std::collections::HashMap<String, Vec<crate::model::ExerciseContextView>> {
    use crate::model::{ExerciseContextView, PieceRefView};
    use std::collections::{HashMap, HashSet};

    // (exercise_id, piece_id | None) → accumulated rollup for that context.
    struct Acc {
        piece_title: Option<String>,
        session_ids: HashSet<String>,
        last_practiced_at: Option<String>,
        // (date, score) of the most recent scored entry in this context.
        latest_scored: Option<(String, u8)>,
    }

    let mut acc: HashMap<(String, Option<String>), Acc> = HashMap::new();

    for session in sessions {
        let date = session.started_at.to_rfc3339();
        for entry in &session.entries {
            if entry.item_type != ItemKind::Exercise {
                continue;
            }
            // Context = the piece sharing this entry's block in the same
            // session; ungrouped or piece-less blocks fall to the None bucket.
            let piece = entry.group_id.as_deref().and_then(|g| {
                session
                    .entries
                    .iter()
                    .find(|e| e.item_type == ItemKind::Piece && e.group_id.as_deref() == Some(g))
            });
            let piece_id = piece.map(|p| p.item_id.clone());

            let record = acc
                .entry((entry.item_id.clone(), piece_id))
                .or_insert_with(|| Acc {
                    piece_title: None,
                    session_ids: HashSet::new(),
                    last_practiced_at: None,
                    latest_scored: None,
                });
            if let Some(p) = piece {
                record.piece_title = Some(p.item_title.clone());
            }
            record.session_ids.insert(session.id.clone());
            if record
                .last_practiced_at
                .as_ref()
                .map_or(true, |cur| date > *cur)
            {
                record.last_practiced_at = Some(date.clone());
            }
            if let Some(score) = entry.score {
                if record
                    .latest_scored
                    .as_ref()
                    .map_or(true, |(d, _)| date > *d)
                {
                    record.latest_scored = Some((date.clone(), score));
                }
            }
        }
    }

    let mut by_exercise: HashMap<String, Vec<ExerciseContextView>> = HashMap::new();
    for ((exercise_id, piece_id), record) in acc {
        // #1093 (1a): prefer the live piece's current title so a rename shows
        // through; fall back to the practice-time snapshot when the piece is
        // gone. `piece_removed` records that fall-back so the shell can render
        // the row as retired history (2a).
        let piece = piece_id.map(|id| {
            let live = item_index.get(id.as_str());
            (
                PieceRefView {
                    title: live
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| record.piece_title.unwrap_or_default()),
                    subtitle: live.and_then(|p| p.composer.clone()),
                    id,
                },
                live.is_none(),
            )
        });
        let piece_removed = piece.as_ref().is_some_and(|(_, removed)| *removed);
        by_exercise
            .entry(exercise_id)
            .or_default()
            .push(ExerciseContextView {
                piece: piece.map(|(p, _)| p),
                latest_score: record.latest_scored.map(|(_, s)| s),
                session_count: record.session_ids.len(),
                last_practiced_at: record.last_practiced_at,
                piece_removed,
            });
    }

    // Deterministic order: piece contexts first (most-recent practice first,
    // tie-broken by piece id), then the "On its own" bucket last.
    for contexts in by_exercise.values_mut() {
        contexts.sort_by(|a, b| match (&a.piece, &b.piece) {
            (Some(x), Some(y)) => b
                .last_practiced_at
                .cmp(&a.last_practiced_at)
                .then_with(|| x.id.cmp(&y.id)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }
    by_exercise
}

/// Per-step score history from session entries that were attributed to a
/// ladder step, keyed by (item id, variant id), newest first (#1083).
fn build_variant_score_index(
    sessions: &[PracticeSession],
) -> std::collections::HashMap<(&str, &str), Vec<crate::model::ScoreHistoryEntry>> {
    use crate::model::ScoreHistoryEntry;
    use std::collections::HashMap;

    let mut index: HashMap<(&str, &str), Vec<ScoreHistoryEntry>> = HashMap::new();
    for session in sessions {
        for entry in &session.entries {
            let (Some(variant_id), Some(score)) = (&entry.variant_id, entry.score) else {
                continue;
            };
            index
                .entry((entry.item_id.as_str(), variant_id.as_str()))
                .or_default()
                .push(ScoreHistoryEntry {
                    session_date: session.started_at.to_rfc3339(),
                    score,
                    session_id: session.id.clone(),
                });
        }
    }
    for history in index.values_mut() {
        history.sort_by(|a, b| b.session_date.cmp(&a.session_date));
    }
    index
}

/// Project an exercise's ladder for the view: live steps only, in ladder
/// order, with per-step scores, the solid flag, and the single current rung.
fn build_variant_views(
    item: &crate::domain::item::Item,
    variant_scores: &std::collections::HashMap<(&str, &str), Vec<crate::model::ScoreHistoryEntry>>,
) -> Vec<crate::model::VariantView> {
    use crate::domain::variant::SOLID_SCORE_MIN;
    use crate::model::VariantView;

    let mut live: Vec<_> = item
        .variants
        .iter()
        .filter(|v| v.deleted_at.is_none())
        .collect();
    live.sort_by_key(|v| v.position);

    let mut views: Vec<VariantView> = live
        .into_iter()
        .map(|v| {
            let score_history = variant_scores
                .get(&(item.id.as_str(), v.id.as_str()))
                .cloned()
                .unwrap_or_default();
            let latest_score = score_history.first().map(|e| e.score);
            VariantView {
                id: v.id.clone(),
                label: v.label.clone(),
                position: v.position,
                latest_score,
                score_history,
                is_solid: latest_score.is_some_and(|s| s >= SOLID_SCORE_MIN),
                is_current: false,
            }
        })
        .collect();

    if let Some(current) = views.iter_mut().find(|v| !v.is_solid) {
        current.is_current = true;
    }
    views
}

fn sort_library_items(items: &mut [LibraryItemView], sort: &LibrarySort) {
    items.sort_by(|a, b| {
        let primary = match sort.field {
            SortField::DateAdded => a.created_at.cmp(&b.created_at),
            SortField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            SortField::LastPracticed => {
                // None = "never practised" = earliest. Option ordering puts
                // None < Some, which is exactly that.
                let la = a
                    .practice
                    .as_ref()
                    .and_then(|p| p.last_practiced_at.as_deref());
                let lb = b
                    .practice
                    .as_ref()
                    .and_then(|p| p.last_practiced_at.as_deref());
                la.cmp(&lb)
            }
        };
        let directed = match sort.direction {
            SortDirection::Ascending => primary,
            SortDirection::Descending => primary.reverse(),
        };
        // Stable tiebreaker so equal keys don't jitter between renders.
        directed
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn apply_query_filter(items: Vec<LibraryItemView>, query: &ListQuery) -> Vec<LibraryItemView> {
    items
        .into_iter()
        .filter(|item| {
            if let Some(ref item_type) = query.item_type {
                if item.item_type != *item_type {
                    return false;
                }
            }

            if let Some(ref key) = query.key {
                if item.key.as_deref() != Some(key.as_str()) {
                    return false;
                }
            }

            // Multi-tag filter is a union (match ANY, case-insensitive), not an intersection.
            if !query.tags.is_empty() {
                let selected: Vec<String> = query.tags.iter().map(|t| t.to_lowercase()).collect();
                let matches_any = item
                    .tags
                    .iter()
                    .any(|t| selected.contains(&t.to_lowercase()));
                if !matches_any {
                    return false;
                }
            }

            if let Some(ref text) = query.text {
                let text_lower = text.to_lowercase();
                let matches = item.title.to_lowercase().contains(&text_lower)
                    || item.subtitle.to_lowercase().contains(&text_lower)
                    || item
                        .notes
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&text_lower))
                    || item
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&text_lower));
                if !matches {
                    return false;
                }
            }

            true
        })
        .collect()
}

/// Canonical demo dataset for `Event::LoadSampleData` — shared by every shell
/// (CI screenshots, local demos, E2E). Stable ids; staggered timestamps so the
/// newest-first sort is deterministic.
fn sample_items() -> Vec<Item> {
    use crate::domain::types::Tempo;
    let now = chrono::Utc::now();

    #[allow(clippy::too_many_arguments)]
    let item = |minutes_ago: i64,
                id: &str,
                title: &str,
                kind: ItemKind,
                composer: Option<&str>,
                key: Option<&str>,
                marking: Option<&str>,
                bpm: Option<u16>,
                notes: Option<&str>,
                tags: &[&str]|
     -> Item {
        let ts = now - chrono::Duration::minutes(minutes_ago);
        Item {
            id: id.to_string(),
            title: title.to_string(),
            kind,
            composer: composer.map(str::to_string),
            key: key.map(str::to_string),
            modality: None,
            tempo: Tempo::from_parts(marking.map(str::to_string), bpm),
            notes: notes.map(str::to_string),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: ts,
            updated_at: ts,
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
        }
    };

    let mut items = vec![
        item(
            0,
            "sample-clair",
            "Clair de Lune",
            ItemKind::Piece,
            Some("Claude Debussy"),
            Some("D♭ major"),
            Some("Andante"),
            Some(72),
            Some("Focus on the rubato in the opening phrase; keep the left hand soft."),
            &["recital", "impressionist"],
        ),
        item(
            1,
            "sample-gymnopedie",
            "Gymnopédie No. 1",
            ItemKind::Piece,
            Some("Erik Satie"),
            Some("D major"),
            Some("Lent"),
            Some(70),
            None,
            &["recital"],
        ),
        item(
            2,
            "sample-nocturne",
            "Nocturne Op. 9 No. 2",
            ItemKind::Piece,
            Some("Frédéric Chopin"),
            Some("E♭ major"),
            Some("Andante"),
            Some(68),
            None,
            &[],
        ),
        item(
            3,
            "sample-hanon",
            "Hanon No. 1",
            ItemKind::Exercise,
            Some("Charles-Louis Hanon"),
            Some("C major"),
            None,
            Some(108),
            Some("Even tone, relaxed wrist."),
            &["warm-up"],
        ),
        item(
            4,
            "sample-scales",
            "Major Scales",
            ItemKind::Exercise,
            None,
            None,
            None,
            Some(120),
            None,
            &["technique"],
        ),
    ];

    // Demo step ladder (#1083): Major Scales climbs a starter run of keys, so
    // seed mode shows per-step progress (sample_sessions scores the first two).
    if let Some(scales) = items.iter_mut().find(|i| i.id == "sample-scales") {
        scales.variants = ["C", "G", "D", "A", "E"]
            .iter()
            .enumerate()
            .map(|(position, label)| crate::domain::variant::Variant {
                id: format!("sample-scales-step-{}", label.to_lowercase()),
                label: (*label).to_string(),
                position,
                updated_at: scales.updated_at,
                deleted_at: None,
            })
            .collect();
    }
    items
}

/// Canonical demo practice history for `Event::LoadSampleData`. Entries
/// reference the ids minted by `sample_items()` so the home screen's
/// "duration · item count" line and any future detail view stay consistent.
fn sample_sessions() -> Vec<PracticeSession> {
    use crate::domain::session::{CompletionStatus, EntryStatus, SetlistEntry};
    let now = chrono::Utc::now();

    let entry = |position: usize,
                 item_id: &str,
                 item_title: &str,
                 item_type: ItemKind,
                 duration_secs: u64|
     -> SetlistEntry {
        SetlistEntry {
            id: format!("{item_id}-entry-{position}"),
            item_id: item_id.to_string(),
            item_title: item_title.to_string(),
            item_type,
            position,
            duration_secs,
            status: EntryStatus::Completed,
            notes: None,
            score: None,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs: None,
            achieved_tempo: None,
            group_id: None,
            variant_id: None,
        }
    };

    let session = |id: &str,
                   days_ago: i64,
                   completion_status: CompletionStatus,
                   entries: Vec<SetlistEntry>|
     -> PracticeSession {
        let total_duration_secs = entries.iter().map(|e| e.duration_secs).sum();
        let started_at = now - chrono::Duration::days(days_ago);
        PracticeSession {
            id: id.to_string(),
            entries,
            session_notes: None,
            session_intention: None,
            started_at,
            completed_at: started_at + chrono::Duration::seconds(total_duration_secs as i64),
            total_duration_secs,
            completion_status,
            session_score: None,
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
        }
    };

    vec![
        session(
            "sample-session-today",
            0,
            CompletionStatus::Completed,
            vec![
                entry(0, "sample-clair", "Clair de Lune", ItemKind::Piece, 720),
                entry(
                    1,
                    "sample-gymnopedie",
                    "Gymnopédie No. 1",
                    ItemKind::Piece,
                    540,
                ),
                entry(
                    2,
                    "sample-nocturne",
                    "Nocturne Op. 9 No. 2",
                    ItemKind::Piece,
                    540,
                ),
            ],
        ),
        session(
            "sample-session-yesterday",
            1,
            CompletionStatus::Completed,
            vec![
                entry(0, "sample-hanon", "Hanon No. 1", ItemKind::Exercise, 480),
                {
                    let mut e = entry(1, "sample-scales", "Major Scales", ItemKind::Exercise, 600);
                    e.variant_id = Some("sample-scales-step-g".to_string());
                    e.score = Some(7);
                    e
                },
            ],
        ),
        session(
            "sample-session-3d",
            3,
            CompletionStatus::EndedEarly,
            vec![
                entry(0, "sample-clair", "Clair de Lune", ItemKind::Piece, 1500),
                entry(1, "sample-hanon", "Hanon No. 1", ItemKind::Exercise, 600),
                entry(
                    2,
                    "sample-nocturne",
                    "Nocturne Op. 9 No. 2",
                    ItemKind::Piece,
                    600,
                ),
            ],
        ),
        session(
            "sample-session-5d",
            5,
            CompletionStatus::Completed,
            vec![{
                let mut e = entry(0, "sample-scales", "Major Scales", ItemKind::Exercise, 720);
                e.variant_id = Some("sample-scales-step-c".to_string());
                e.score = Some(8);
                e
            }],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charted_piece_surfaces_a_scaffold_preview_in_the_view() {
        use crate::domain::item::ItemEvent;
        let app = Intrada;
        let mut model = Model::test_default();
        model.local_first = true;

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
    fn test_data_loaded_populates_model() {
        let app = Intrada;
        let mut model = Model::test_default();

        let now = chrono::Utc::now();
        let items = vec![
            Item {
                id: "piece1".to_string(),
                title: "Clair de Lune".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Debussy".to_string()),
                key: Some("Db Major".to_string()),
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
            },
            Item {
                id: "ex1".to_string(),
                title: "C Major Scale".to_string(),
                kind: ItemKind::Exercise,
                composer: None,
                key: Some("C Major".to_string()),
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
            },
        ];

        let _cmd = app.update(Event::DataLoaded { items }, &mut model);

        assert_eq!(model.items.len(), 2);
        assert_eq!(model.items[0].title, "Clair de Lune");
        assert_eq!(model.items[1].title, "C Major Scale");
        assert!(model.last_error.is_none());
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
    fn test_load_failed_does_not_set_last_set_save_request_id() {
        // Failure must not surface a request_id — would flip "Saved" on a
        // failed save (#449).
        let app = Intrada;
        let mut model = Model {
            api_base_url: "http://localhost:3001".to_string(),
            last_set_save_request_id: Some("req-old".to_string()),
            ..Default::default()
        };

        let _cmd = app.update(
            Event::LoadFailed("Failed to save set: timeout".to_string()),
            &mut model,
        );

        assert_eq!(
            model.last_set_save_request_id.as_deref(),
            Some("req-old"),
            "request_id must not change on failure"
        );
        assert_eq!(
            model.last_error.as_deref(),
            Some("Failed to save set: timeout")
        );
    }

    #[test]
    fn test_set_save_succeeded_records_request_id_and_clears_error() {
        let app = Intrada;
        let mut model = Model {
            api_base_url: "http://localhost:3001".to_string(),
            last_set_save_request_id: Some("req-old".to_string()),
            last_error: Some("Failed to save set: timeout".to_string()),
            error_muted: true,
            ..Default::default()
        };

        let _cmd = app.update(
            Event::SetSaveSucceeded {
                request_id: "req-new".to_string(),
            },
            &mut model,
        );

        assert_eq!(model.last_set_save_request_id.as_deref(), Some("req-new"));
        assert!(model.last_error.is_none());
        assert!(!model.error_muted);
        let vm = app.view(&model);
        assert_eq!(vm.last_set_save_request_id.as_deref(), Some("req-new"));
    }

    #[test]
    fn test_concurrent_set_saves_only_promote_matching_form() {
        // The invariant behind #663: each success overwrites with its own id.
        let app = Intrada;
        let mut model = Model {
            api_base_url: "http://localhost:3001".to_string(),
            ..Default::default()
        };

        let _cmd = app.update(
            Event::SetSaveSucceeded {
                request_id: "req-A".to_string(),
            },
            &mut model,
        );
        assert_eq!(model.last_set_save_request_id.as_deref(), Some("req-A"));

        let _cmd = app.update(
            Event::SetSaveSucceeded {
                request_id: "req-B".to_string(),
            },
            &mut model,
        );
        assert_eq!(model.last_set_save_request_id.as_deref(), Some("req-B"));
    }

    #[test]
    fn test_signed_out_resets_user_scoped_state() {
        let app = Intrada;
        let now = chrono::Utc::now();

        // Populate a model with state from a fully signed-in user across
        // every sensitive field that could leak to the next user (#645).
        let mut model = Model {
            api_base_url: "http://localhost:3001".to_string(),
            items: vec![Item {
                id: "i1".to_string(),
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
            }],
            sessions: vec![PracticeSession {
                id: "sess1".to_string(),
                entries: vec![],
                session_notes: Some("private notes".to_string()),
                session_intention: Some("focus".to_string()),
                started_at: now,
                completed_at: now,
                total_duration_secs: 60,
                completion_status: CompletionStatus::Completed,
                session_score: None,
                reflection_improved: None,
                reflection_still_rough: None,
                reflection_next_target: None,
            }],
            session_status: SessionStatus::Active(ActiveSession {
                id: "active1".to_string(),
                entries: vec![],
                current_index: 0,
                current_item_started_at: now,
                session_started_at: now,
                session_intention: Some("in-progress intention".to_string()),
            }),
            last_error: Some("connection lost".to_string()),
            error_muted: true,
            mcp_tokens: vec![crate::domain::mcp_tokens::McpToken {
                id: "tok1".to_string(),
                name: "ci-bot".to_string(),
                prefix: "intr_pat_".to_string(),
                last_used_at: None,
                created_at: now,
                revoked_at: None,
            }],
            mcp_tokens_loaded: true,
            mcp_audit: vec![crate::domain::mcp_audit::McpAuditEntry {
                id: "audit1".to_string(),
                token_id: None,
                token_name: None,
                token_prefix: None,
                tool: "list_items".to_string(),
                args_hash: "abc".to_string(),
                created_at: now,
            }],
            mcp_audit_loaded: true,
            ..Default::default()
        };

        let _cmd = app.update(Event::SignedOut, &mut model);

        // api_base_url is set at startup, not per-user — must survive.
        assert_eq!(model.api_base_url, "http://localhost:3001");
        // Everything else returns to Default — exhaustive checks across the
        // most sensitive fields (anything visible in the ViewModel between
        // sign-out and first refetch).
        assert!(model.items.is_empty());
        assert!(model.sessions.is_empty());
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert!(model.last_error.is_none());
        assert!(!model.error_muted);
        assert!(model.mcp_tokens.is_empty());
        assert!(!model.mcp_tokens_loaded);
        assert!(model.mcp_audit.is_empty());
        assert!(!model.mcp_audit_loaded);
    }

    #[test]
    fn test_view_empty_model() {
        let app = Intrada;
        let model = Model::test_default();
        let vm = app.view(&model);

        assert!(vm.items.is_empty());
        assert_eq!(vm.items.len(), 0);
        assert!(vm.error.is_none());
        assert_eq!(vm.session_status, SessionStatusView::Idle);
    }

    #[test]
    fn test_delete_after_seed_does_not_panic() {
        let app = Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::LoadSampleData, &mut model);
        assert_eq!(model.api_base_url, "http://localhost:3001");
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
        assert!(first.is_solid, "the first demo step reads as Solid");
        assert!(
            scales.variants[1].is_current,
            "the second demo step is the current rung"
        );
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();

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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();

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
            })),
            &mut model,
        );
        assert_eq!(model.items[1].composer, Some("Hanon".to_string()));
    }

    #[test]
    fn add_piece_with_blank_composer_is_required_error() {
        let app = Intrada;
        let mut model = Model::test_default();

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
            })),
            &mut model,
        );

        assert!(
            model.items.is_empty(),
            "blank composer must not create a piece"
        );
        assert!(model
            .last_error
            .as_deref()
            .is_some_and(|e| e.contains("Composer is required")));
    }

    // --- T042: Unicode handling in core ---

    #[test]
    fn test_unicode_in_item_add() {
        let app = Intrada;
        let mut model = Model::test_default();

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
        let mut model = Model::test_default();
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
                        notes: None,
                        score: if e % 2 == 0 { Some(3) } else { None },
                        intention: None,
                        rep_target: None,
                        rep_count: None,
                        rep_target_reached: None,
                        rep_history: None,
                        planned_duration_secs: None,
                        achieved_tempo: if e % 3 == 0 { Some(120) } else { None },
                        group_id: None,
                        variant_id: None,
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
                session_intention: None,
                entries,
                session_score: None,
                reflection_improved: None,
                reflection_still_rough: None,
                reflection_next_target: None,
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
        let mut model = Model::test_default();

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
            session_intention: None,
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
                    score: None,
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                    group_id: None,
                    variant_id: None,
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
                    score: None,
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                    group_id: None,
                    variant_id: None,
                },
            ],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
        });
        model.practice_summaries = build_practice_summaries(&model.sessions);

        let vm = app.view(&model);
        let p1_view = vm.items.iter().find(|i| i.id == "p1").unwrap();
        let p2_view = vm.items.iter().find(|i| i.id == "p2").unwrap();

        // p1 has 2 entries totalling 45 minutes, no scores, no tempo
        assert_eq!(
            p1_view.practice,
            Some(ItemPracticeSummary {
                session_count: 2,
                total_minutes: 45,
                latest_score: None,
                score_history: vec![],
                latest_tempo: None,
                tempo_history: vec![],
                last_practiced_at: Some(sess1_started.to_rfc3339()),
            })
        );
        // p2 has no entries
        assert_eq!(p2_view.practice, None);
    }

    // ── Score history tests (T019) ────────────────────────────────────

    #[test]
    fn test_score_history_multiple_sessions() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model::test_default();

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
            session_intention: None,
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
                score: Some(3),
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
        });

        // Session 2: newer, score 5
        model.sessions.push(PracticeSession {
            id: "sess2".to_string(),
            started_at: now - chrono::Duration::minutes(30),
            completed_at: now,
            total_duration_secs: 1800,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
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
                score: Some(5),
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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
        let mut model = Model::test_default();

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
            session_intention: None,
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
                score: None,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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
        let mut model = Model::test_default();

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
            session_intention: None,
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
                    score: Some(2),
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                    group_id: None,
                    variant_id: None,
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
                    score: Some(4),
                    intention: None,
                    rep_target: None,
                    rep_count: None,
                    rep_target_reached: None,
                    rep_history: None,
                    planned_duration_secs: None,
                    achieved_tempo: None,
                    group_id: None,
                    variant_id: None,
                },
            ],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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
        let mut model = Model::test_default();

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
            session_intention: None,
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
                score: None, // Skipped entries never have scores
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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
    fn test_start_app_sets_api_base_url() {
        let app = Intrada;
        let mut model = Model::default();
        assert!(model.api_base_url.is_empty());

        let _cmd = app.update(
            Event::StartApp {
                api_base_url: "https://api.example.com".to_string(),
                local_first: false,
            },
            &mut model,
        );

        assert_eq!(model.api_base_url, "https://api.example.com");
    }

    // --- Data loaded callbacks ---

    fn make_session(
        id: &str,
        item_id: &str,
        score: Option<u8>,
        tempo: Option<u16>,
    ) -> PracticeSession {
        let now = chrono::Utc::now();
        PracticeSession {
            id: id.to_string(),
            started_at: now,
            completed_at: now,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: None,
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
                score,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: tempo,
                group_id: None,
                variant_id: None,
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
        }
    }

    #[test]
    fn test_sessions_loaded_populates_model_and_summaries() {
        let app = Intrada;
        let mut model = Model::test_default();

        let sessions = vec![make_session("s1", "item-1", Some(4), Some(120))];
        let _cmd = app.update(Event::SessionsLoaded { sessions }, &mut model);

        assert_eq!(model.sessions.len(), 1);
        let summary = model.practice_summaries.get("item-1");
        assert!(summary.is_some());
        let summary = summary.unwrap();
        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.total_minutes, 5);
        assert_eq!(summary.latest_score, Some(4));
        assert_eq!(summary.latest_tempo, Some(120));
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
            session_intention: None,
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
                score: None,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: None,
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
        };

        let summaries = build_practice_summaries(&[mk("s1", earlier), mk("s2", later)]);
        let summary = summaries.get("item-1").expect("summary for item-1");
        assert_eq!(summary.last_practiced_at, Some(later.to_rfc3339()));
    }

    #[test]
    fn test_sets_loaded_populates_model() {
        use crate::domain::set::{Set, SetEntry};

        let app = Intrada;
        let mut model = Model::test_default();
        let now = chrono::Utc::now();

        let sets = vec![Set {
            id: "r1".to_string(),
            name: "Warm-up".to_string(),
            entries: vec![SetEntry {
                id: "re1".to_string(),
                item_id: "item-1".to_string(),
                item_title: "Scales".to_string(),
                item_type: ItemKind::Exercise,
                position: 0,
            }],
            created_at: now,
            updated_at: now,
        }];

        let _cmd = app.update(Event::SetsLoaded { sets }, &mut model);

        assert_eq!(model.sets.len(), 1);
        assert_eq!(model.sets[0].name, "Warm-up");
    }

    // --- Write-confirmation callbacks ---

    #[test]
    fn test_item_updated_replaces_existing() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Old Title".to_string(),
                kind: ItemKind::Piece,
                composer: Some("Composer".to_string()),
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
            }],
            ..Model::test_default()
        };

        let updated = Item {
            id: "p1".to_string(),
            title: "New Title".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Composer".to_string()),
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
        };

        let _cmd = app.update(Event::ItemUpdated { item: updated }, &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "New Title");
    }

    #[test]
    fn test_item_updated_ignores_unknown_id() {
        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            items: vec![Item {
                id: "p1".to_string(),
                title: "Original".to_string(),
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
            }],
            ..Model::test_default()
        };

        let unknown = Item {
            id: "unknown".to_string(),
            title: "Ghost".to_string(),
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
        };

        let _cmd = app.update(Event::ItemUpdated { item: unknown }, &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Original");
    }

    #[test]
    fn test_set_updated_replaces_existing() {
        use crate::domain::set::Set;

        let app = Intrada;
        let now = chrono::Utc::now();
        let mut model = Model {
            sets: vec![Set {
                id: "r1".to_string(),
                name: "Old Set".to_string(),
                entries: vec![],
                created_at: now,
                updated_at: now,
            }],
            ..Model::test_default()
        };

        let updated = Set {
            id: "r1".to_string(),
            name: "Renamed Set".to_string(),
            entries: vec![],
            created_at: now,
            updated_at: now,
        };

        let _cmd = app.update(Event::SetUpdated { set: updated }, &mut model);

        assert_eq!(model.sets[0].name, "Renamed Set");
    }

    #[test]
    fn test_delete_confirmed_is_noop() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.items.push(Item {
            id: "p1".to_string(),
            title: "Still Here".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
        });

        let _cmd = app.update(Event::DeleteConfirmed, &mut model);

        // Model unchanged — optimistic delete already happened
        assert_eq!(model.items.len(), 1);
    }

    // --- Error handling ---

    #[test]
    fn test_load_failed_sets_error() {
        let app = Intrada;
        let mut model = Model::test_default();

        let _cmd = app.update(
            Event::LoadFailed("Connection refused".to_string()),
            &mut model,
        );

        assert_eq!(model.last_error, Some("Connection refused".to_string()));
    }

    #[test]
    fn test_load_failed_dedupes_identical_messages() {
        // Identical messages no-op so the shell doesn't re-render with the
        // same text. (#346) Separate from mount-stability — this is just
        // belt-and-braces for repeated retries with the same error.
        let app = Intrada;
        let mut model = Model::test_default();

        let _ = app.update(Event::LoadFailed("timeout".to_string()), &mut model);
        let _ = app.update(Event::LoadFailed("timeout".to_string()), &mut model);
        let _ = app.update(Event::LoadFailed("timeout".to_string()), &mut model);

        assert_eq!(model.last_error, Some("timeout".to_string()));
    }

    #[test]
    fn test_load_failed_distinct_message_replaces_existing() {
        // A user-action error (save/delete) must surface even if a stale
        // load-error banner is still up — otherwise the user has no
        // feedback that their action failed. Burst re-animation is
        // suppressed at the shell mount level, not by swallowing distinct
        // messages here.
        let app = Intrada;
        let mut model = Model {
            last_error: Some("Failed to load items".to_string()),
            ..Model::test_default()
        };

        let _ = app.update(
            Event::LoadFailed("Failed to save item: 409 conflict".to_string()),
            &mut model,
        );

        assert_eq!(
            model.last_error,
            Some("Failed to save item: 409 conflict".to_string())
        );
    }

    #[test]
    fn test_load_failed_after_dismiss_is_muted_until_success() {
        // After the user dismisses the banner, subsequent failures stay
        // suppressed — otherwise every retry/refetch against a still-broken
        // backend pops the banner back up (#346). Once a success arrives,
        // the mute clears and new failures surface again.
        let app = Intrada;
        let mut model = Model::test_default();

        let _ = app.update(Event::LoadFailed("first".to_string()), &mut model);
        let _ = app.update(Event::ClearError, &mut model);
        assert!(model.error_muted);

        // Muted: a different LoadFailed while still broken stays hidden.
        let _ = app.update(Event::LoadFailed("second".to_string()), &mut model);
        assert_eq!(model.last_error, None);

        // Success unmutes — system has recovered.
        let _ = app.update(Event::DataLoaded { items: vec![] }, &mut model);
        assert!(!model.error_muted);

        // Now a new failure surfaces.
        let _ = app.update(Event::LoadFailed("third".to_string()), &mut model);
        assert_eq!(model.last_error, Some("third".to_string()));
    }

    #[test]
    fn test_burst_after_dismiss_stays_muted() {
        // Mirrors the user-reported reproduction in #346: dismiss, then a
        // burst of distinct failures (e.g. parallel refetches against a
        // still-broken backend) all stay suppressed.
        let app = Intrada;
        let mut model = Model::test_default();

        let _ = app.update(Event::LoadFailed("Failed to load items".into()), &mut model);
        let _ = app.update(Event::ClearError, &mut model);

        for msg in [
            "Failed to load items: timeout",
            "Failed to load sessions: 503",
            "Failed to load sets: connection refused",
            "Failed to load analytics: timeout",
        ] {
            let _ = app.update(Event::LoadFailed(msg.into()), &mut model);
            assert_eq!(model.last_error, None, "burst msg should stay muted: {msg}");
            assert!(model.error_muted, "mute should persist across burst");
        }
    }

    #[test]
    fn test_clear_error_sets_muted_flag() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("some error".to_string()),
            ..Model::test_default()
        };

        let _ = app.update(Event::ClearError, &mut model);

        assert_eq!(model.last_error, None);
        assert!(model.error_muted);
    }

    #[test]
    fn test_sessions_loaded_unmutes() {
        // Any confirmed API success should unmute, not just DataLoaded —
        // otherwise the muted state could persist forever if items never
        // load again (e.g. user goes straight into the sessions tab).
        let app = Intrada;
        let mut model = Model {
            error_muted: true,
            ..Model::test_default()
        };

        let _ = app.update(Event::SessionsLoaded { sessions: vec![] }, &mut model);
        assert!(!model.error_muted);
    }

    #[test]
    fn test_data_loaded_clears_previous_error() {
        let app = Intrada;
        let mut model = Model {
            last_error: Some("Old error".to_string()),
            ..Model::test_default()
        };

        let _cmd = app.update(Event::DataLoaded { items: vec![] }, &mut model);

        assert!(model.last_error.is_none());
    }

    // --- View: session status mapping ---

    #[test]
    fn test_view_session_status_building() {
        use crate::domain::session::BuildingSession;

        let app = Intrada;
        let model = Model {
            session_status: SessionStatus::Building(BuildingSession {
                session_intention: Some("Focus on dynamics".to_string()),
                ..Default::default()
            }),
            ..Model::test_default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.session_status, SessionStatusView::Building);
        assert!(vm.building_setlist.is_some());
        assert!(vm.active_session.is_none());
        assert!(vm.summary.is_none());
        let setlist = vm.building_setlist.unwrap();
        assert_eq!(
            setlist.session_intention,
            Some("Focus on dynamics".to_string())
        );
    }

    // --- View: sets ---

    #[test]
    fn test_view_renders_sets() {
        use crate::domain::set::{Set, SetEntry};

        let app = Intrada;
        let now = chrono::Utc::now();
        let model = Model {
            sets: vec![Set {
                id: "r1".to_string(),
                name: "Morning Warm-up".to_string(),
                entries: vec![
                    SetEntry {
                        id: "re1".to_string(),
                        item_id: "item-1".to_string(),
                        item_title: "Scales".to_string(),
                        item_type: ItemKind::Exercise,
                        position: 0,
                    },
                    SetEntry {
                        id: "re2".to_string(),
                        item_id: "item-2".to_string(),
                        item_title: "Arpeggios".to_string(),
                        item_type: ItemKind::Exercise,
                        position: 1,
                    },
                ],
                created_at: now,
                updated_at: now,
            }],
            ..Model::test_default()
        };

        let vm = app.view(&model);
        assert_eq!(vm.sets.len(), 1);
        assert_eq!(vm.sets[0].name, "Morning Warm-up");
        assert_eq!(vm.sets[0].entry_count, 2);
        assert_eq!(vm.sets[0].entries[0].item_title, "Scales");
        assert_eq!(vm.sets[0].entries[1].item_title, "Arpeggios");
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
        assert!(summary.tempo_history.is_empty());
    }

    #[test]
    fn test_view_empty_sessions() {
        let app = Intrada;
        let model = Model::test_default();
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
        }
    }

    #[test]
    fn view_items_sorted_newest_first() {
        let app = Intrada;
        let mut model = Model::test_default();
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
                latest_score: None,
                score_history: vec![],
                latest_tempo: None,
                tempo_history: vec![],
                last_practiced_at: Some(at.to_rfc3339()),
            },
        );
    }

    #[test]
    fn view_sorts_by_title_ascending() {
        let app = Intrada;
        let mut model = Model::test_default();
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
    fn view_sorts_by_last_practiced_descending_most_recent_first() {
        let app = Intrada;
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
    fn view_default_sort_is_date_added_newest_first() {
        let app = Intrada;
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
        let model = Model::test_default();
        let vm = app.view(&model);
        assert_eq!(vm.active_query, None);
    }

    #[test]
    fn view_counts_describe_the_visible_set() {
        let app = Intrada;
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
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
                session_intention: None,
                session_score: None,
                reflection_improved: None,
                reflection_still_rough: None,
                reflection_next_target: None,
            },
            PracticeSession {
                id: "s2".to_string(),
                started_at: t2,
                completed_at: t2 + chrono::Duration::minutes(15),
                total_duration_secs: 900,
                completion_status: CompletionStatus::Completed,
                entries: vec![],
                session_notes: None,
                session_intention: None,
                session_score: None,
                reflection_improved: None,
                reflection_still_rough: None,
                reflection_next_target: None,
            },
        ];
        let vm = app.view(&model);
        assert_eq!(vm.sessions[0].id, "s2");
        assert_eq!(vm.sessions[1].id, "s1");
    }

    #[test]
    fn view_error_maps_from_last_error() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.last_error = Some("bad request".to_string());
        let vm = app.view(&model);
        assert_eq!(vm.error.as_deref(), Some("bad request"));
    }

    #[test]
    fn view_empty_sessions_produces_no_analytics() {
        let app = Intrada;
        let model = Model::test_default();
        let vm = app.view(&model);
        assert!(vm.analytics.is_none());
    }

    #[test]
    fn view_set_source_status_no_source() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.session_status =
            SessionStatus::Building(crate::domain::session::BuildingSession::default());
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.source_status, SetSourceStatus::NoSource);
    }

    #[test]
    fn view_set_source_status_unmodified() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.sets = vec![crate::domain::set::Set {
            id: "set-1".to_string(),
            name: "Morning".to_string(),
            entries: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];
        let entry = SetlistEntry {
            id: "e1".to_string(),
            item_id: "item-a".to_string(),
            item_title: "Scale".to_string(),
            item_type: ItemKind::Exercise,
            position: 0,
            duration_secs: 0,
            status: EntryStatus::NotAttempted,
            notes: None,
            score: None,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs: None,
            achieved_tempo: None,
            group_id: None,
            variant_id: None,
        };
        model.session_status = SessionStatus::Building(crate::domain::session::BuildingSession {
            entries: vec![entry],
            source_set_id: Some("set-1".to_string()),
            source_set_entry_snapshot: vec!["item-a".to_string()],
            ..Default::default()
        });
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert!(matches!(
            building.source_status,
            SetSourceStatus::UnmodifiedFromSource { .. }
        ));
    }

    #[test]
    fn view_set_source_status_modified() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.sets = vec![crate::domain::set::Set {
            id: "set-1".to_string(),
            name: "Morning".to_string(),
            entries: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];
        let entry = SetlistEntry {
            id: "e1".to_string(),
            item_id: "item-b".to_string(),
            item_title: "Etude".to_string(),
            item_type: ItemKind::Piece,
            position: 0,
            duration_secs: 0,
            status: EntryStatus::NotAttempted,
            notes: None,
            score: None,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs: None,
            achieved_tempo: None,
            group_id: None,
            variant_id: None,
        };
        model.session_status = SessionStatus::Building(crate::domain::session::BuildingSession {
            entries: vec![entry],
            source_set_id: Some("set-1".to_string()),
            source_set_entry_snapshot: vec!["item-a".to_string()],
            ..Default::default()
        });
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert!(matches!(
            building.source_status,
            SetSourceStatus::ModifiedFromSource { .. }
        ));
    }

    fn building_entry(id: &str, planned_duration_secs: Option<u32>) -> SetlistEntry {
        SetlistEntry {
            id: id.to_string(),
            item_id: format!("item-{id}"),
            item_title: "Etude".to_string(),
            item_type: ItemKind::Piece,
            position: 0,
            duration_secs: 0,
            status: EntryStatus::NotAttempted,
            notes: None,
            score: None,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs,
            achieved_tempo: None,
            group_id: None,
            variant_id: None,
        }
    }

    #[test]
    fn error_seq_bumps_on_each_failed_update_even_with_identical_message() {
        let app = Intrada;
        let mut model = Model::test_default();
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
        let mut model = Model::test_default();
        let before = app.view(&model).error_seq;
        let _ = app.update(Event::Session(SessionEvent::StartBuilding), &mut model);
        assert_eq!(app.view(&model).error_seq, before);
    }

    #[test]
    fn view_building_setlist_total_duration_sums_planned() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.session_status = SessionStatus::Building(crate::domain::session::BuildingSession {
            entries: vec![
                building_entry("e1", Some(900)),
                building_entry("e2", Some(630)),
                building_entry("e3", None),
            ],
            ..Default::default()
        });
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.total_duration_display.as_deref(), Some("25m 30s"));
        assert_eq!(building.total_duration_summary.as_deref(), Some("25m 30s"));
    }

    #[test]
    fn view_building_setlist_total_duration_whole_minutes_matches_block_dialect() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.session_status = SessionStatus::Building(crate::domain::session::BuildingSession {
            entries: vec![
                building_entry("e1", Some(900)),
                building_entry("e2", Some(300)),
            ],
            ..Default::default()
        });
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.total_duration_summary.as_deref(), Some("20 min"));
    }

    #[test]
    fn view_building_setlist_total_duration_none_when_unplanned() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.session_status = SessionStatus::Building(crate::domain::session::BuildingSession {
            entries: vec![building_entry("e1", None), building_entry("e2", None)],
            ..Default::default()
        });
        let vm = app.view(&model);
        let building = vm.building_setlist.unwrap();
        assert_eq!(building.total_duration_display, None);
        assert_eq!(building.total_duration_summary, None);
    }

    #[test]
    fn view_last_set_save_request_id_mirrors_model() {
        let app = Intrada;
        let mut model = Model::test_default();
        model.last_set_save_request_id = Some("req-42".to_string());
        let vm = app.view(&model);
        assert_eq!(vm.last_set_save_request_id.as_deref(), Some("req-42"));
    }

    #[test]
    fn item_created_replaces_optimistic_by_temp_id() {
        let app = Intrada;
        let mut model = Model::test_default();

        let now = chrono::Utc::now();
        let temp_id = "temp_ulid".to_string();
        model.items.push(Item {
            id: temp_id.clone(),
            title: "Optimistic".to_string(),
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
        });

        let server_item = Item {
            id: "server_ulid".to_string(),
            title: "Optimistic".to_string(),
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
        };
        let _cmd = app.update(
            Event::ItemCreated {
                temp_id: temp_id.clone(),
                item: server_item.clone(),
            },
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "server_ulid");
    }

    #[test]
    fn item_created_pushes_when_temp_id_absent() {
        let app = Intrada;
        let mut model = Model::test_default();

        let now = chrono::Utc::now();
        let server_item = Item {
            id: "server_ulid".to_string(),
            title: "Late confirmation".to_string(),
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
        };

        // No optimistic entry — caller may have navigated away and back.
        let _cmd = app.update(
            Event::ItemCreated {
                temp_id: "missing_temp".into(),
                item: server_item,
            },
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "server_ulid");
    }

    #[test]
    fn test_new_item_defaults_to_not_priority() {
        let app = Intrada;
        let mut model = Model::test_default();

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
            }],
            ..Model::test_default()
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
        let mut model = Model::test_default();

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
            }],
            ..Model::test_default()
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
            }],
            ..Model::test_default()
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
        };
        let model = Model {
            items: vec![piece, ex],
            ..Model::test_default()
        };
        let vm = app.view(&model);
        let exercise = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(
            exercise.linked_from_pieces[0].subtitle.as_deref(),
            Some("Debussy")
        );
    }

    #[test]
    fn test_view_resolves_linked_exercises_and_linked_from_pieces() {
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
        assert!(piece_view.linked_from_pieces.is_empty());

        // Linked exercise ex-1: linked_from_pieces lists the piece.
        let ex1_view = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(ex1_view.linked_from_pieces.len(), 1);
        assert_eq!(ex1_view.linked_from_pieces[0].id, "piece-1");
        assert_eq!(ex1_view.linked_from_pieces[0].title, "Sonata");
        assert!(ex1_view.linked_exercises.is_empty());

        // Linked exercise ex-2: also appears in linked_from_pieces.
        let ex2_view = vm.items.iter().find(|i| i.id == "ex-2").unwrap();
        assert_eq!(ex2_view.linked_from_pieces.len(), 1);
        assert_eq!(ex2_view.linked_from_pieces[0].id, "piece-1");
        assert!(ex2_view.linked_exercises.is_empty());

        // Unrelated exercise: both lists empty.
        let ex3_view = vm.items.iter().find(|i| i.id == "ex-3").unwrap();
        assert!(ex3_view.linked_exercises.is_empty());
        assert!(ex3_view.linked_from_pieces.is_empty());
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
            score,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs: None,
            achieved_tempo: None,
            group_id: group.map(String::from),
            variant_id: None,
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
            session_intention: None,
            session_score: None,
            entries,
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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
        }
    }

    /// The core B1 derivation: an exercise practised in a piece's block twice
    /// and standalone once yields two contexts — the piece (latest score, count,
    /// date rolled up) then the "On its own" bucket last.
    #[test]
    fn test_exercise_contexts_derive_piece_and_on_its_own() {
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
        assert_eq!(ex1.exercise_contexts.len(), 2, "piece + on-its-own");

        let piece_ctx = &ex1.exercise_contexts[0];
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

        let solo = &ex1.exercise_contexts[1];
        assert!(solo.piece.is_none(), "'On its own' bucket has no piece");
        assert_eq!(solo.latest_score, Some(8));
        assert_eq!(solo.session_count, 1);
        assert_eq!(solo.last_practiced_at, Some(d3.to_rfc3339()));

        // Pieces themselves carry no contexts.
        let piece_view = vm.items.iter().find(|i| i.id == "P").unwrap();
        assert!(piece_view.exercise_contexts.is_empty());
    }

    /// A grouped run with no piece entry (a dissolved block) falls into the
    /// "On its own" bucket, not a phantom piece context.
    #[test]
    fn test_exercise_contexts_grouped_without_piece_is_on_its_own() {
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
        assert_eq!(ex1.exercise_contexts.len(), 1);
        assert!(ex1.exercise_contexts[0].piece.is_none());
        assert_eq!(ex1.exercise_contexts[0].latest_score, Some(5));
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
        let ctx = &ex1.exercise_contexts[0];
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
        assert_eq!(
            ex1.exercise_contexts.len(),
            1,
            "removed piece kept, not dropped"
        );
        let ctx = &ex1.exercise_contexts[0];
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

    /// Invariant 6: the derivation is identical whether sessions arrive via the
    /// online `SessionsLoaded` path or the local-first `SessionsStoreLoaded`
    /// persistence path — it's a pure projection over `model.sessions`.
    #[test]
    fn test_exercise_contexts_identical_in_both_modes() {
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

        // Online: local_first stays false, sessions arrive via SessionsLoaded.
        let mut online = Model::test_default();
        online.items = items.clone();
        let _ = app.update(
            Event::SessionsLoaded {
                sessions: sessions.clone(),
            },
            &mut online,
        );
        let online_ctx = app
            .view(&online)
            .items
            .into_iter()
            .find(|i| i.id == "ex-1")
            .unwrap()
            .exercise_contexts;

        // Local-first: sessions arrive via the persistence store.
        let mut local = Model::test_default();
        local.local_first = true;
        local.items = items;
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
            .exercise_contexts;

        assert_eq!(online_ctx.len(), 1);
        assert_eq!(online_ctx[0].piece.as_ref().unwrap().id, "P");
        assert_eq!(online_ctx[0].latest_score, Some(7));
        assert_eq!(
            online_ctx, local_ctx,
            "context derivation must not depend on load path (invariant 6)"
        );
    }

    #[test]
    fn test_reverse_index_drops_non_exercise_kind() {
        // Symmetric to the forward-path kind filter: if a linked id resolves to a
        // Piece (not an Exercise), both views must drop it — the forward
        // linked_exercises already does this; the reverse linked_from_pieces must too.
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

        // Reverse: item-b's linked_from_pieces must also be empty — it is not an
        // Exercise, so it should never appear as a link target's back-reference.
        let item_b = vm.items.iter().find(|i| i.id == "item-b").unwrap();
        assert!(
            item_b.linked_from_pieces.is_empty(),
            "reverse path must drop non-Exercise from linked_from_pieces (symmetric drop)"
        );
    }
    // ── Step ladder view derivation (#1083 C1) ─────────────────────────

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
            session_intention: None,
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
                score,
                intention: None,
                rep_target: None,
                rep_count: None,
                rep_target_reached: None,
                rep_history: None,
                planned_duration_secs: None,
                achieved_tempo: None,
                group_id: None,
                variant_id: variant_id.map(str::to_string),
            }],
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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
    fn view_exposes_live_steps_sorted_by_position_tombstones_excluded() {
        let vm = step_view_model(vec![]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert_eq!(
            ex.variants
                .iter()
                .map(|v| (v.id.as_str(), v.label.as_str(), v.position))
                .collect::<Vec<_>>(),
            vec![("v-c", "C", 0), ("v-f", "F", 1)],
            "live steps only, in ladder order"
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
            // A flat (unattributed) score never counts towards a step.
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
    fn view_marks_solid_steps_and_the_first_unsolid_as_current() {
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
        assert!(!c.is_current);
        let f = ex.variants.iter().find(|v| v.id == "v-f").unwrap();
        assert!(!f.is_solid, "7 of 10 is not yet solid");
        assert!(f.is_current, "the first unsolid step is current");
    }

    #[test]
    fn view_unrated_ladder_starts_current_at_the_first_step() {
        let vm = step_view_model(vec![]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(ex.variants[0].is_current);
        assert!(!ex.variants[1].is_current);
    }

    #[test]
    fn view_fully_solid_ladder_has_no_current_step() {
        let t0 = chrono::Utc::now();
        let vm = step_view_model(vec![
            step_session("s1", "ex-1", Some("v-c"), Some(8), t0),
            step_session(
                "s2",
                "ex-1",
                Some("v-f"),
                Some(10),
                t0 + chrono::Duration::days(1),
            ),
        ]);

        let ex = vm.items.iter().find(|i| i.id == "ex-1").unwrap();
        assert!(
            ex.variants.iter().all(|v| !v.is_current),
            "a finished ladder has no current step"
        );
    }

    #[test]
    fn view_session_entries_expose_variant_id() {
        let vm = step_view_model(vec![step_session(
            "s1",
            "ex-1",
            Some("v-c"),
            Some(6),
            chrono::Utc::now(),
        )]);

        assert_eq!(
            vm.sessions[0].entries[0].variant_id.as_deref(),
            Some("v-c"),
            "history entries carry their step through the view"
        );
    }
}

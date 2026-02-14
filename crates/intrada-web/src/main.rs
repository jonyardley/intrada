use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use chrono::Utc;
use crux_core::Core;
use leptos::ev;
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use intrada_core::app::{Effect, StorageEffect};
use intrada_core::domain::exercise::{Exercise, ExerciseEvent};
use intrada_core::domain::piece::{Piece, PieceEvent};
use intrada_core::domain::types::{
    CreateExercise, CreatePiece, Tempo, UpdateExercise, UpdatePiece,
};
use intrada_core::{Event, Intrada, LibraryItemView, ViewModel};

/// Wrapper around Core that is safe to use in Leptos reactive contexts (WASM is single-threaded).
type SharedCore = SendWrapper<Rc<RefCell<Core<Intrada>>>>;

// ---------------------------------------------------------------------------
// Phase 1 — ViewState enum (T001)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum ViewState {
    List,
    Detail(String),
    AddPiece,
    AddExercise,
    EditPiece(String),
    EditExercise(String),
}

// ---------------------------------------------------------------------------
// Phase 2 — Validation helpers (T006-T011)
// ---------------------------------------------------------------------------

/// Parse comma-separated tags string into Vec<String>.
/// Trims whitespace, filters empty entries.
fn parse_tags(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse tempo marking + BPM string into Option<Tempo>.
/// Returns None if both are empty.
fn parse_tempo(marking: &str, bpm_str: &str) -> Option<Tempo> {
    let marking = marking.trim();
    let bpm_str = bpm_str.trim();

    if marking.is_empty() && bpm_str.is_empty() {
        return None;
    }

    let marking_opt = if marking.is_empty() {
        None
    } else {
        Some(marking.to_string())
    };

    let bpm_opt = if bpm_str.is_empty() {
        None
    } else {
        bpm_str.parse::<u16>().ok()
    };

    Some(Tempo {
        marking: marking_opt,
        bpm: bpm_opt,
    })
}

/// Validate add/edit piece form fields. Returns map of field_name -> error message.
fn validate_piece_form(
    title: &str,
    composer: &str,
    notes: &str,
    bpm_str: &str,
    tempo_marking: &str,
    tags_str: &str,
) -> HashMap<String, String> {
    let mut errors = HashMap::new();

    // Title: required, 1-500 chars
    let title = title.trim();
    if title.is_empty() {
        errors.insert("title".to_string(), "Title is required".to_string());
    } else if title.len() > 500 {
        errors.insert(
            "title".to_string(),
            "Title must be between 1 and 500 characters".to_string(),
        );
    }

    // Composer: required for pieces, 1-200 chars
    let composer = composer.trim();
    if composer.is_empty() {
        errors.insert("composer".to_string(), "Composer is required".to_string());
    } else if composer.len() > 200 {
        errors.insert(
            "composer".to_string(),
            "Composer must be between 1 and 200 characters".to_string(),
        );
    }

    // Notes: optional, max 5000
    let notes = notes.trim();
    if !notes.is_empty() && notes.len() > 5000 {
        errors.insert(
            "notes".to_string(),
            "Notes must not exceed 5000 characters".to_string(),
        );
    }

    // BPM: optional, 1-400
    let bpm_str = bpm_str.trim();
    if !bpm_str.is_empty() {
        match bpm_str.parse::<u16>() {
            Ok(bpm) if !(1..=400).contains(&bpm) => {
                errors.insert(
                    "bpm".to_string(),
                    "BPM must be between 1 and 400".to_string(),
                );
            }
            Err(_) => {
                errors.insert(
                    "bpm".to_string(),
                    "BPM must be between 1 and 400".to_string(),
                );
            }
            _ => {}
        }
    }

    // Tempo marking: optional, max 100
    let tempo_marking = tempo_marking.trim();
    if !tempo_marking.is_empty() && tempo_marking.len() > 100 {
        errors.insert(
            "tempo_marking".to_string(),
            "Tempo marking must not exceed 100 characters".to_string(),
        );
    }

    // Tempo: if one part is set, at least one must be valid
    if (!tempo_marking.is_empty() || !bpm_str.is_empty())
        && tempo_marking.is_empty()
        && bpm_str.is_empty()
    {
        // This case can't actually occur, but defensive
        errors.insert(
            "tempo".to_string(),
            "Tempo must have at least a marking or BPM value".to_string(),
        );
    }

    // Tags: each 1-100 chars
    let tags = parse_tags(tags_str);
    for tag in &tags {
        if tag.len() > 100 {
            errors.insert(
                "tags".to_string(),
                "Each tag must be between 1 and 100 characters".to_string(),
            );
            break;
        }
    }

    errors
}

/// Validate add/edit exercise form fields. Returns map of field_name -> error message.
fn validate_exercise_form(
    title: &str,
    composer: &str,
    category: &str,
    notes: &str,
    bpm_str: &str,
    tempo_marking: &str,
    tags_str: &str,
) -> HashMap<String, String> {
    let mut errors = HashMap::new();

    // Title: required, 1-500 chars
    let title = title.trim();
    if title.is_empty() {
        errors.insert("title".to_string(), "Title is required".to_string());
    } else if title.len() > 500 {
        errors.insert(
            "title".to_string(),
            "Title must be between 1 and 500 characters".to_string(),
        );
    }

    // Composer: optional for exercises, max 200 if present
    let composer = composer.trim();
    if !composer.is_empty() && composer.len() > 200 {
        errors.insert(
            "composer".to_string(),
            "Composer must be between 1 and 200 characters".to_string(),
        );
    }

    // Category: optional, max 100
    let category = category.trim();
    if !category.is_empty() && category.len() > 100 {
        errors.insert(
            "category".to_string(),
            "Category must be between 1 and 100 characters".to_string(),
        );
    }

    // Notes: optional, max 5000
    let notes = notes.trim();
    if !notes.is_empty() && notes.len() > 5000 {
        errors.insert(
            "notes".to_string(),
            "Notes must not exceed 5000 characters".to_string(),
        );
    }

    // BPM: optional, 1-400
    let bpm_str = bpm_str.trim();
    if !bpm_str.is_empty() {
        match bpm_str.parse::<u16>() {
            Ok(bpm) if !(1..=400).contains(&bpm) => {
                errors.insert(
                    "bpm".to_string(),
                    "BPM must be between 1 and 400".to_string(),
                );
            }
            Err(_) => {
                errors.insert(
                    "bpm".to_string(),
                    "BPM must be between 1 and 400".to_string(),
                );
            }
            _ => {}
        }
    }

    // Tempo marking: optional, max 100
    let tempo_marking = tempo_marking.trim();
    if !tempo_marking.is_empty() && tempo_marking.len() > 100 {
        errors.insert(
            "tempo_marking".to_string(),
            "Tempo marking must not exceed 100 characters".to_string(),
        );
    }

    // Tags: each 1-100 chars
    let tags = parse_tags(tags_str);
    for tag in &tags {
        if tag.len() > 100 {
            errors.insert(
                "tags".to_string(),
                "Each tag must be between 1 and 100 characters".to_string(),
            );
            break;
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Create the stub data per data-model.md
fn create_stub_data() -> (Vec<Piece>, Vec<Exercise>) {
    let now = Utc::now();
    let pieces = vec![Piece {
        id: ulid::Ulid::new().to_string(),
        title: "Clair de Lune".to_string(),
        composer: "Claude Debussy".to_string(),
        key: Some("Db Major".to_string()),
        tempo: Some(Tempo {
            marking: Some("Andante tr\u{00e8}s expressif".to_string()),
            bpm: Some(66),
        }),
        notes: Some("Third movement of Suite bergamasque".to_string()),
        tags: vec!["impressionist".to_string(), "piano".to_string()],
        created_at: now,
        updated_at: now,
    }];
    let exercises = vec![Exercise {
        id: ulid::Ulid::new().to_string(),
        title: "Hanon No. 1".to_string(),
        composer: Some("Charles-Louis Hanon".to_string()),
        category: Some("Technique".to_string()),
        key: Some("C Major".to_string()),
        tempo: Some(Tempo {
            marking: Some("Moderato".to_string()),
            bpm: Some(108),
        }),
        notes: Some("The Virtuoso Pianist \u{2014} Exercise 1".to_string()),
        tags: vec!["technique".to_string(), "warm-up".to_string()],
        created_at: now,
        updated_at: now,
    }];
    (pieces, exercises)
}

/// Process effects returned by the Crux core.
fn process_effects(core: &Core<Intrada>, effects: Vec<Effect>, view_model: &RwSignal<ViewModel>) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {}
            Effect::Storage(boxed_request) => match &boxed_request.operation {
                StorageEffect::LoadAll => {
                    let (pieces, exercises) = create_stub_data();
                    let inner_effects = core.process_event(Event::DataLoaded { pieces, exercises });
                    process_effects(core, inner_effects, view_model);
                }
                StorageEffect::SavePiece(_)
                | StorageEffect::SaveExercise(_)
                | StorageEffect::UpdatePiece(_)
                | StorageEffect::UpdateExercise(_)
                | StorageEffect::DeleteItem { .. } => {}
            },
        }
    }
    view_model.set(core.view());
}

/// Sample piece names for the "Add Sample Item" button
const SAMPLE_PIECES: &[(&str, &str)] = &[
    ("Moonlight Sonata", "Ludwig van Beethoven"),
    ("Nocturne Op. 9 No. 2", "Fr\u{00e9}d\u{00e9}ric Chopin"),
    ("Gymnop\u{00e9}die No. 1", "Erik Satie"),
    ("Prelude in C Major", "Johann Sebastian Bach"),
    ("Liebestr\u{00e4}ume No. 3", "Franz Liszt"),
];

// ---------------------------------------------------------------------------
// Phase 1 — App component with ViewState routing (T002-T005)
// ---------------------------------------------------------------------------

#[component]
fn App() -> impl IntoView {
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());
    let view_state = RwSignal::new(ViewState::List);
    let sample_counter = RwSignal::new(0_usize);

    // Initialize: load stub data on mount
    {
        let core_ref = core.borrow();
        let (pieces, exercises) = create_stub_data();
        let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
        process_effects(&core_ref, effects, &view_model);
    }

    let core_for_view = core.clone();

    view! {
        <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-100 text-slate-800">
            // Header
            <header class="bg-white shadow-sm border-b border-slate-200" role="banner">
                <div class="max-w-4xl mx-auto px-6 py-5 flex items-center justify-between">
                    <div>
                        <h1 class="text-3xl font-bold tracking-tight text-slate-900">"Intrada"</h1>
                        <p class="text-sm text-slate-500 mt-0.5">"Your music practice companion"</p>
                    </div>
                    <span
                        class="inline-flex items-center rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-800"
                        aria-label="Application version"
                    >
                        "v0.1.0"
                    </span>
                </div>
            </header>

            // Main content — routed by ViewState
            <main class="max-w-4xl mx-auto px-6 py-10" role="main">
                {move || {
                    let vs = view_state.get();
                    let core = core_for_view.clone();
                    match vs {
                        ViewState::List => {
                            view! {
                                <LibraryListView
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                    sample_counter=sample_counter
                                />
                            }.into_any()
                        }
                        ViewState::Detail(id) => {
                            view! {
                                <DetailView
                                    id=id.clone()
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::AddPiece => {
                            view! {
                                <AddPieceForm
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::AddExercise => {
                            view! {
                                <AddExerciseForm
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::EditPiece(id) => {
                            view! {
                                <EditPieceForm
                                    id=id.clone()
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::EditExercise(id) => {
                            view! {
                                <EditExerciseForm
                                    id=id.clone()
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                    }
                }}
            </main>

            // Footer
            <footer class="max-w-4xl mx-auto px-6 py-6 border-t border-slate-200" role="contentinfo">
                <p class="text-xs text-slate-400 text-center">
                    "Built with Rust, Leptos & Crux \u{2014} Page reload resets to stub data"
                </p>
            </footer>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Phase 2 — FormFieldError component (T011)
// ---------------------------------------------------------------------------

/// Displays an inline validation error for a named form field.
#[component]
fn FormFieldError(field: String, errors: RwSignal<HashMap<String, String>>) -> impl IntoView {
    view! {
        {move || {
            errors.get().get(&field).cloned().map(|msg| {
                view! {
                    <p class="mt-1 text-sm text-red-600" role="alert">{msg}</p>
                }
            })
        }}
    }
}

// ---------------------------------------------------------------------------
// Library List View (extracted from App, Phase 3/4)
// ---------------------------------------------------------------------------

#[component]
fn LibraryListView(
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
    sample_counter: RwSignal<usize>,
) -> impl IntoView {
    let show_add_menu = RwSignal::new(false);
    let core_for_sample = core.clone();

    view! {
        // Hero section
        <section class="mb-10" aria-labelledby="welcome-heading">
            <h2 id="welcome-heading" class="text-2xl font-semibold text-slate-800 mb-3">
                "Welcome to Intrada"
            </h2>
            <p class="text-slate-600 leading-relaxed max-w-2xl">
                "Organize your music library, track your practice pieces and exercises, "
                "and build better practice habits. Intrada helps musicians stay focused "
                "on what matters \u{2014} making music."
            </p>
        </section>

        // Error banner
        {move || {
            view_model.get().error.map(|err| {
                view! {
                    <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                        <p class="text-sm text-red-800">
                            <span class="font-medium">"Error: "</span>{err}
                        </p>
                    </div>
                }
            })
        }}

        // Status message
        {move || {
            view_model.get().status.map(|status| {
                view! {
                    <div class="mb-6 rounded-lg bg-blue-50 border border-blue-200 p-4" role="status">
                        <p class="text-sm text-blue-800">{status}</p>
                    </div>
                }
            })
        }}

        // Library section header
        <section class="mb-10" aria-labelledby="library-heading">
            <div class="flex items-center justify-between mb-4">
                <h2 id="library-heading" class="text-lg font-semibold text-slate-700">"Library"</h2>
                <div class="flex items-center gap-3">
                    <span class="text-sm text-slate-500">
                        {move || {
                            let count = view_model.get().item_count;
                            format!("{count} item(s)")
                        }}
                    </span>

                    // Add dropdown (FR-001)
                    <div class="relative">
                        <button
                            class="inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3.5 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 transition-colors"
                            aria-label="Add a new item"
                            aria-expanded=move || show_add_menu.get().to_string()
                            on:click=move |_| { show_add_menu.set(!show_add_menu.get()); }
                        >
                            <span aria-hidden="true">"+"</span>
                            " Add"
                        </button>
                        {move || {
                            if show_add_menu.get() {
                                Some(view! {
                                    <div class="absolute right-0 mt-2 w-48 rounded-lg bg-white shadow-lg border border-slate-200 z-10">
                                        <button
                                            class="w-full text-left px-4 py-2.5 text-sm text-slate-700 hover:bg-slate-50 rounded-t-lg"
                                            on:click=move |_| {
                                                show_add_menu.set(false);
                                                view_state.set(ViewState::AddPiece);
                                            }
                                        >
                                            "Piece"
                                        </button>
                                        <button
                                            class="w-full text-left px-4 py-2.5 text-sm text-slate-700 hover:bg-slate-50 rounded-b-lg"
                                            on:click=move |_| {
                                                show_add_menu.set(false);
                                                view_state.set(ViewState::AddExercise);
                                            }
                                        >
                                            "Exercise"
                                        </button>
                                    </div>
                                })
                            } else {
                                None
                            }
                        }}
                    </div>

                    // "Add Sample Item" button (retained from MVP)
                    <button
                        class="inline-flex items-center gap-1.5 rounded-lg bg-slate-200 px-3.5 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-300 transition-colors"
                        aria-label="Add a sample piece to the library"
                        on:click=move |_| {
                            let idx = sample_counter.get() % SAMPLE_PIECES.len();
                            let (title, composer) = SAMPLE_PIECES[idx];
                            sample_counter.set(sample_counter.get() + 1);

                            let event = Event::Piece(PieceEvent::Add(CreatePiece {
                                title: title.to_string(),
                                composer: composer.to_string(),
                                key: None,
                                tempo: None,
                                notes: None,
                                tags: vec!["sample".to_string()],
                            }));

                            let core_ref = core_for_sample.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model);
                        }
                    >
                        "Add Sample"
                    </button>
                </div>
            </div>

            // Items list (FR-006: clickable items)
            <div class="space-y-3">
                {move || {
                    let vm = view_model.get();
                    if vm.items.is_empty() {
                        view! {
                            <div class="bg-white rounded-xl border border-slate-200 p-8 text-center">
                                <p class="text-slate-400">"No items in your library yet."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="space-y-3" role="list" aria-label="Library items">
                                {vm.items.into_iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let vs = view_state;
                                    view! {
                                        <LibraryItemCard
                                            item=item
                                            on_click=move |_: ev::MouseEvent| {
                                                vs.set(ViewState::Detail(item_id.clone()));
                                            }
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// LibraryItemCard — now clickable (Phase 4, T025)
// ---------------------------------------------------------------------------

#[component]
fn LibraryItemCard<F>(item: LibraryItemView, on_click: F) -> impl IntoView
where
    F: Fn(ev::MouseEvent) + 'static,
{
    let LibraryItemView {
        title,
        subtitle,
        item_type,
        key,
        tempo,
        tags,
        ..
    } = item;

    let badge_classes = if item_type == "piece" {
        "inline-flex items-center rounded-full bg-violet-100 px-2.5 py-0.5 text-xs font-medium text-violet-800"
    } else {
        "inline-flex items-center rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-medium text-emerald-800"
    };

    let has_subtitle = !subtitle.is_empty();
    let has_tags = !tags.is_empty();

    view! {
        <li
            class="bg-white rounded-xl shadow-sm border border-slate-200 p-5 hover:shadow-md transition-shadow cursor-pointer"
            tabindex="0"
            role="button"
            on:click=on_click
            on:keydown=move |ev: ev::KeyboardEvent| {
                if ev.key() == "Enter" || ev.key() == " " {
                    ev.prevent_default();
                    // Trigger click on the parent li
                    if let Some(target) = ev.target() {
                        use wasm_bindgen::JsCast;
                        if let Some(el) = target.dyn_ref::<leptos::web_sys::HtmlElement>() {
                            el.click();
                        }
                    }
                }
            }
        >
            <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                    <h3 class="text-base font-semibold text-slate-900 truncate">{title}</h3>
                    {if has_subtitle {
                        Some(view! {
                            <p class="text-sm text-slate-500 mt-0.5 truncate">{subtitle}</p>
                        })
                    } else {
                        None
                    }}
                    <div class="flex flex-wrap items-center gap-x-4 gap-y-1 mt-2 text-xs text-slate-400">
                        {key.map(|k| {
                            view! {
                                <span class="flex items-center gap-1">
                                    <span aria-hidden="true">"\u{266F}"</span>{k}
                                </span>
                            }
                        })}
                        {tempo.map(|t| {
                            view! {
                                <span class="flex items-center gap-1">
                                    <span aria-hidden="true">"\u{2669}"</span>{t}
                                </span>
                            }
                        })}
                    </div>
                    {if has_tags {
                        Some(view! {
                            <div class="flex flex-wrap gap-1.5 mt-2">
                                {tags.into_iter().map(|tag| {
                                    view! {
                                        <span class="inline-flex items-center rounded-md bg-slate-100 px-2 py-0.5 text-xs text-slate-600">
                                            {tag}
                                        </span>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        })
                    } else {
                        None
                    }}
                </div>
                <span class=badge_classes>{item_type}</span>
            </div>
        </li>
    }
}

// ---------------------------------------------------------------------------
// Phase 4 — Detail View (T023-T032)
// ---------------------------------------------------------------------------

#[component]
fn DetailView(
    id: String,
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let show_delete_confirm = RwSignal::new(false);

    // Find the item in the current ViewModel
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        // Item not found — navigate back to list (handles deleted-item edge case)
        view_state.set(ViewState::List);
        return view! { <p>"Item not found."</p> }.into_any();
    };

    let item_id = item.id.clone();
    let item_type = item.item_type.clone();
    let is_piece = item_type == "piece";

    // Clone fields for display
    let title = item.title.clone();
    let subtitle = item.subtitle.clone();
    let category = item.category.clone();
    let key = item.key.clone();
    let tempo = item.tempo.clone();
    let notes = item.notes.clone();
    let tags = item.tags.clone();
    let created_at = item.created_at.clone();
    let updated_at = item.updated_at.clone();

    // Clone IDs for closures
    let id_for_edit = item_id.clone();
    let id_for_delete = item_id.clone();
    let type_for_edit = item_type.clone();

    view! {
        <div>
            // Back button
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click=move |_| { view_state.set(ViewState::List); }
            >
                "\u{2190} Back to Library"
            </button>

            // Delete confirmation banner (FR-011)
            {move || {
                if show_delete_confirm.get() {
                    let id_del = id_for_delete.clone();
                    let core_del = core.clone();
                    let item_type_del = item_type.clone();
                    Some(view! {
                        <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                            <p class="text-sm text-red-800 mb-3">
                                "Are you sure you want to delete this item? This action cannot be undone."
                            </p>
                            <div class="flex gap-3">
                                <button
                                    class="rounded-lg bg-red-600 px-3.5 py-2 text-sm font-medium text-white hover:bg-red-500 transition-colors"
                                    on:click=move |_| {
                                        let event = if item_type_del == "piece" {
                                            Event::Piece(PieceEvent::Delete { id: id_del.clone() })
                                        } else {
                                            Event::Exercise(ExerciseEvent::Delete { id: id_del.clone() })
                                        };
                                        let core_ref = core_del.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model);
                                        view_state.set(ViewState::List);
                                    }
                                >
                                    "Confirm Delete"
                                </button>
                                <button
                                    class="rounded-lg bg-white px-3.5 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                                    on:click=move |_| { show_delete_confirm.set(false); }
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Detail card
            <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
                // Header: title + type badge
                <div class="flex items-start justify-between gap-3 mb-6">
                    <div>
                        <h2 class="text-2xl font-bold text-slate-900">{title}</h2>
                        {if !subtitle.is_empty() {
                            Some(view! {
                                <p class="text-lg text-slate-500 mt-1">{subtitle.clone()}</p>
                            })
                        } else {
                            None
                        }}
                    </div>
                    <span class={if is_piece {
                        "inline-flex items-center rounded-full bg-violet-100 px-3 py-1 text-sm font-medium text-violet-800"
                    } else {
                        "inline-flex items-center rounded-full bg-emerald-100 px-3 py-1 text-sm font-medium text-emerald-800"
                    }}>
                        {if is_piece { "Piece" } else { "Exercise" }}
                    </span>
                </div>

                // Fields grid (FR-007, FR-008: omit empty optional fields)
                <dl class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4 mb-6">
                    {category.map(|cat| {
                        view! {
                            <div>
                                <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">"Category"</dt>
                                <dd class="mt-1 text-sm text-slate-700">{cat}</dd>
                            </div>
                        }
                    })}
                    {key.map(|k| {
                        view! {
                            <div>
                                <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">"Key"</dt>
                                <dd class="mt-1 text-sm text-slate-700">{k}</dd>
                            </div>
                        }
                    })}
                    {tempo.map(|t| {
                        view! {
                            <div>
                                <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">"Tempo"</dt>
                                <dd class="mt-1 text-sm text-slate-700">{t}</dd>
                            </div>
                        }
                    })}
                </dl>

                // Notes
                {notes.map(|n| {
                    view! {
                        <div class="mb-6">
                            <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider mb-1">"Notes"</dt>
                            <dd class="text-sm text-slate-700 whitespace-pre-wrap">{n}</dd>
                        </div>
                    }
                })}

                // Tags
                {if !tags.is_empty() {
                    Some(view! {
                        <div class="mb-6">
                            <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider mb-2">"Tags"</dt>
                            <dd class="flex flex-wrap gap-1.5">
                                {tags.into_iter().map(|tag| {
                                    view! {
                                        <span class="inline-flex items-center rounded-md bg-slate-100 px-2.5 py-1 text-xs text-slate-600">
                                            {tag}
                                        </span>
                                    }
                                }).collect::<Vec<_>>()}
                            </dd>
                        </div>
                    })
                } else {
                    None
                }}

                // Timestamps
                <div class="border-t border-slate-100 pt-4 grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs text-slate-400">
                    <div>
                        <span class="font-medium">"Created: "</span>{created_at}
                    </div>
                    <div>
                        <span class="font-medium">"Updated: "</span>{updated_at}
                    </div>
                </div>
            </div>

            // Action buttons (FR-009, FR-011)
            <div class="mt-6 flex gap-3">
                <button
                    class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    on:click=move |_| {
                        if type_for_edit == "piece" {
                            view_state.set(ViewState::EditPiece(id_for_edit.clone()));
                        } else {
                            view_state.set(ViewState::EditExercise(id_for_edit.clone()));
                        }
                    }
                >
                    "Edit"
                </button>
                <button
                    class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-red-600 border border-red-300 hover:bg-red-50 transition-colors"
                    on:click=move |_| { show_delete_confirm.set(true); }
                >
                    "Delete"
                </button>
            </div>
        </div>
    }.into_any()
}

// ---------------------------------------------------------------------------
// Phase 3 — Add Piece Form (T012-T017)
// ---------------------------------------------------------------------------

#[component]
fn AddPieceForm(
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let composer = RwSignal::new(String::new());
    let key_sig = RwSignal::new(String::new());
    let tempo_marking = RwSignal::new(String::new());
    let bpm = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let tags_input = RwSignal::new(String::new());
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click=move |_| { view_state.set(ViewState::List); }
            >
                "\u{2190} Cancel"
            </button>

            <h2 class="text-2xl font-bold text-slate-900 mb-6">"Add Piece"</h2>

            <form
                class="bg-white rounded-xl shadow-sm border border-slate-200 p-6 space-y-5"
                on:submit=move |ev: ev::SubmitEvent| {
                    ev.prevent_default();

                    let validation_errors = validate_piece_form(
                        &title.get(),
                        &composer.get(),
                        &notes.get(),
                        &bpm.get(),
                        &tempo_marking.get(),
                        &tags_input.get(),
                    );

                    if !validation_errors.is_empty() {
                        errors.set(validation_errors);
                        return;
                    }
                    errors.set(HashMap::new());

                    let title_val = title.get().trim().to_string();
                    let composer_val = composer.get().trim().to_string();
                    let key_val = {
                        let k = key_sig.get().trim().to_string();
                        if k.is_empty() { None } else { Some(k) }
                    };
                    let tempo_val = parse_tempo(&tempo_marking.get(), &bpm.get());
                    let notes_val = {
                        let n = notes.get().trim().to_string();
                        if n.is_empty() { None } else { Some(n) }
                    };
                    let tags_val = parse_tags(&tags_input.get());

                    let event = Event::Piece(PieceEvent::Add(CreatePiece {
                        title: title_val,
                        composer: composer_val,
                        key: key_val,
                        tempo: tempo_val,
                        notes: notes_val,
                        tags: tags_val,
                    }));

                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(event);
                    process_effects(&core_ref, effects, &view_model);
                    view_state.set(ViewState::List);
                }
            >
                // Title (required)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-title">"Title *"</label>
                    <input
                        id="piece-title"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || title.get()
                        on:input=move |ev| { title.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="title".to_string() errors=errors />
                </div>

                // Composer (required for pieces)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-composer">"Composer *"</label>
                    <input
                        id="piece-composer"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || composer.get()
                        on:input=move |ev| { composer.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="composer".to_string() errors=errors />
                </div>

                // Key (optional)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-key">"Key"</label>
                    <input
                        id="piece-key"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. C Major, Db Minor"
                        prop:value=move || key_sig.get()
                        on:input=move |ev| { key_sig.set(event_target_value(&ev)); }
                    />
                </div>

                // Tempo: marking + BPM on one row
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-tempo-marking">"Tempo Marking"</label>
                        <input
                            id="piece-tempo-marking"
                            type="text"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="e.g. Allegro"
                            prop:value=move || tempo_marking.get()
                            on:input=move |ev| { tempo_marking.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="tempo_marking".to_string() errors=errors />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-bpm">"BPM"</label>
                        <input
                            id="piece-bpm"
                            type="number"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="1-400"
                            prop:value=move || bpm.get()
                            on:input=move |ev| { bpm.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="bpm".to_string() errors=errors />
                    </div>
                </div>

                // Notes (optional)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-notes">"Notes"</label>
                    <textarea
                        id="piece-notes"
                        rows="3"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || notes.get()
                        on:input=move |ev| { notes.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="notes".to_string() errors=errors />
                </div>

                // Tags (comma-separated)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="piece-tags">"Tags"</label>
                    <input
                        id="piece-tags"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="Comma-separated, e.g. classical, piano"
                        prop:value=move || tags_input.get()
                        on:input=move |ev| { tags_input.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="tags".to_string() errors=errors />
                </div>

                // Buttons
                <div class="flex gap-3 pt-2">
                    <button
                        type="submit"
                        class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    >
                        "Save"
                    </button>
                    <button
                        type="button"
                        class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                        on:click=move |_| { view_state.set(ViewState::List); }
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Phase 3 — Add Exercise Form (T018-T022)
// ---------------------------------------------------------------------------

#[component]
fn AddExerciseForm(
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let composer = RwSignal::new(String::new());
    let category = RwSignal::new(String::new());
    let key_sig = RwSignal::new(String::new());
    let tempo_marking = RwSignal::new(String::new());
    let bpm = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let tags_input = RwSignal::new(String::new());
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click=move |_| { view_state.set(ViewState::List); }
            >
                "\u{2190} Cancel"
            </button>

            <h2 class="text-2xl font-bold text-slate-900 mb-6">"Add Exercise"</h2>

            <form
                class="bg-white rounded-xl shadow-sm border border-slate-200 p-6 space-y-5"
                on:submit=move |ev: ev::SubmitEvent| {
                    ev.prevent_default();

                    let validation_errors = validate_exercise_form(
                        &title.get(),
                        &composer.get(),
                        &category.get(),
                        &notes.get(),
                        &bpm.get(),
                        &tempo_marking.get(),
                        &tags_input.get(),
                    );

                    if !validation_errors.is_empty() {
                        errors.set(validation_errors);
                        return;
                    }
                    errors.set(HashMap::new());

                    let title_val = title.get().trim().to_string();
                    let composer_val = {
                        let c = composer.get().trim().to_string();
                        if c.is_empty() { None } else { Some(c) }
                    };
                    let category_val = {
                        let c = category.get().trim().to_string();
                        if c.is_empty() { None } else { Some(c) }
                    };
                    let key_val = {
                        let k = key_sig.get().trim().to_string();
                        if k.is_empty() { None } else { Some(k) }
                    };
                    let tempo_val = parse_tempo(&tempo_marking.get(), &bpm.get());
                    let notes_val = {
                        let n = notes.get().trim().to_string();
                        if n.is_empty() { None } else { Some(n) }
                    };
                    let tags_val = parse_tags(&tags_input.get());

                    let event = Event::Exercise(ExerciseEvent::Add(CreateExercise {
                        title: title_val,
                        composer: composer_val,
                        category: category_val,
                        key: key_val,
                        tempo: tempo_val,
                        notes: notes_val,
                        tags: tags_val,
                    }));

                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(event);
                    process_effects(&core_ref, effects, &view_model);
                    view_state.set(ViewState::List);
                }
            >
                // Title (required)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-title">"Title *"</label>
                    <input
                        id="exercise-title"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || title.get()
                        on:input=move |ev| { title.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="title".to_string() errors=errors />
                </div>

                // Composer (optional for exercises)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-composer">"Composer"</label>
                    <input
                        id="exercise-composer"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || composer.get()
                        on:input=move |ev| { composer.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="composer".to_string() errors=errors />
                </div>

                // Category (optional, exercises only)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-category">"Category"</label>
                    <input
                        id="exercise-category"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. Technique, Scales"
                        prop:value=move || category.get()
                        on:input=move |ev| { category.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="category".to_string() errors=errors />
                </div>

                // Key
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-key">"Key"</label>
                    <input
                        id="exercise-key"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. C Major"
                        prop:value=move || key_sig.get()
                        on:input=move |ev| { key_sig.set(event_target_value(&ev)); }
                    />
                </div>

                // Tempo row
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-tempo-marking">"Tempo Marking"</label>
                        <input
                            id="exercise-tempo-marking"
                            type="text"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="e.g. Moderato"
                            prop:value=move || tempo_marking.get()
                            on:input=move |ev| { tempo_marking.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="tempo_marking".to_string() errors=errors />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-bpm">"BPM"</label>
                        <input
                            id="exercise-bpm"
                            type="number"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="1-400"
                            prop:value=move || bpm.get()
                            on:input=move |ev| { bpm.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="bpm".to_string() errors=errors />
                    </div>
                </div>

                // Notes
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-notes">"Notes"</label>
                    <textarea
                        id="exercise-notes"
                        rows="3"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || notes.get()
                        on:input=move |ev| { notes.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="notes".to_string() errors=errors />
                </div>

                // Tags
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-tags">"Tags"</label>
                    <input
                        id="exercise-tags"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="Comma-separated, e.g. technique, warm-up"
                        prop:value=move || tags_input.get()
                        on:input=move |ev| { tags_input.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="tags".to_string() errors=errors />
                </div>

                // Buttons
                <div class="flex gap-3 pt-2">
                    <button
                        type="submit"
                        class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    >
                        "Save"
                    </button>
                    <button
                        type="button"
                        class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                        on:click=move |_| { view_state.set(ViewState::List); }
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Phase 5 — Edit Piece Form (T033-T037)
// ---------------------------------------------------------------------------

#[component]
fn EditPieceForm(
    id: String,
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    // Find item to pre-populate
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        view_state.set(ViewState::List);
        return view! { <p>"Item not found."</p> }.into_any();
    };

    let item_id = item.id.clone();

    // Pre-populate signals from ViewModel
    // For pieces: subtitle = composer directly
    let title = RwSignal::new(item.title.clone());
    let composer = RwSignal::new(item.subtitle.clone());
    let key_sig = RwSignal::new(item.key.clone().unwrap_or_default());
    // Parse tempo string back into marking + BPM
    let (initial_marking, initial_bpm) = parse_tempo_display(&item.tempo);
    let tempo_marking = RwSignal::new(initial_marking);
    let bpm = RwSignal::new(initial_bpm);
    let notes = RwSignal::new(item.notes.clone().unwrap_or_default());
    let tags_input = RwSignal::new(item.tags.join(", "));
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click={
                    let id_back = item_id.clone();
                    move |_| { view_state.set(ViewState::Detail(id_back.clone())); }
                }
            >
                "\u{2190} Cancel"
            </button>

            <h2 class="text-2xl font-bold text-slate-900 mb-6">"Edit Piece"</h2>

            <form
                class="bg-white rounded-xl shadow-sm border border-slate-200 p-6 space-y-5"
                on:submit={
                    let item_id = item_id.clone();
                    move |ev: ev::SubmitEvent| {
                        ev.prevent_default();

                        let validation_errors = validate_piece_form(
                            &title.get(),
                            &composer.get(),
                            &notes.get(),
                            &bpm.get(),
                            &tempo_marking.get(),
                            &tags_input.get(),
                        );

                        if !validation_errors.is_empty() {
                            errors.set(validation_errors);
                            return;
                        }
                        errors.set(HashMap::new());

                        let title_val = title.get().trim().to_string();
                        let composer_val = composer.get().trim().to_string();
                        let key_val = {
                            let k = key_sig.get().trim().to_string();
                            if k.is_empty() { Some(None) } else { Some(Some(k)) }
                        };
                        let tempo_val = {
                            let t = parse_tempo(&tempo_marking.get(), &bpm.get());
                            match t {
                                None => Some(None),
                                Some(v) => Some(Some(v)),
                            }
                        };
                        let notes_val = {
                            let n = notes.get().trim().to_string();
                            if n.is_empty() { Some(None) } else { Some(Some(n)) }
                        };
                        let tags_val = parse_tags(&tags_input.get());

                        let input = UpdatePiece {
                            title: Some(title_val),
                            composer: Some(composer_val),
                            key: key_val,
                            tempo: tempo_val,
                            notes: notes_val,
                            tags: Some(tags_val),
                        };

                        let event = Event::Piece(PieceEvent::Update {
                            id: item_id.clone(),
                            input,
                        });

                        let core_ref = core.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model);
                        view_state.set(ViewState::Detail(item_id.clone()));
                    }
                }
            >
                // Title
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-title">"Title *"</label>
                    <input
                        id="edit-piece-title"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || title.get()
                        on:input=move |ev| { title.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="title".to_string() errors=errors />
                </div>

                // Composer
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-composer">"Composer *"</label>
                    <input
                        id="edit-piece-composer"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || composer.get()
                        on:input=move |ev| { composer.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="composer".to_string() errors=errors />
                </div>

                // Key
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-key">"Key"</label>
                    <input
                        id="edit-piece-key"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. C Major, Db Minor"
                        prop:value=move || key_sig.get()
                        on:input=move |ev| { key_sig.set(event_target_value(&ev)); }
                    />
                </div>

                // Tempo row
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-tempo-marking">"Tempo Marking"</label>
                        <input
                            id="edit-piece-tempo-marking"
                            type="text"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="e.g. Allegro"
                            prop:value=move || tempo_marking.get()
                            on:input=move |ev| { tempo_marking.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="tempo_marking".to_string() errors=errors />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-bpm">"BPM"</label>
                        <input
                            id="edit-piece-bpm"
                            type="number"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="1-400"
                            prop:value=move || bpm.get()
                            on:input=move |ev| { bpm.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="bpm".to_string() errors=errors />
                    </div>
                </div>

                // Notes
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-notes">"Notes"</label>
                    <textarea
                        id="edit-piece-notes"
                        rows="3"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || notes.get()
                        on:input=move |ev| { notes.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="notes".to_string() errors=errors />
                </div>

                // Tags
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-piece-tags">"Tags"</label>
                    <input
                        id="edit-piece-tags"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="Comma-separated"
                        prop:value=move || tags_input.get()
                        on:input=move |ev| { tags_input.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="tags".to_string() errors=errors />
                </div>

                // Buttons
                <div class="flex gap-3 pt-2">
                    <button
                        type="submit"
                        class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    >
                        "Save"
                    </button>
                    <button
                        type="button"
                        class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                        on:click={
                            let id_cancel = item_id.clone();
                            move |_| { view_state.set(ViewState::Detail(id_cancel.clone())); }
                        }
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }.into_any()
}

// ---------------------------------------------------------------------------
// Phase 5 — Edit Exercise Form (T038-T042)
// ---------------------------------------------------------------------------

#[component]
fn EditExerciseForm(
    id: String,
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        view_state.set(ViewState::List);
        return view! { <p>"Item not found."</p> }.into_any();
    };

    let item_id = item.id.clone();

    // For exercises: subtitle = category.or(composer), so we read category and need
    // to figure out composer. We use the category field directly from ViewModel.
    // The subtitle may be category OR composer. We check: if category is Some, subtitle = category;
    // otherwise subtitle = composer. But we don't have a separate composer field in ViewModel.
    // Strategy: Use subtitle as composer IF category is None. If category is Some, we don't
    // know the composer from ViewModel alone. For editing, we'll use subtitle as best-effort.
    // The Crux core's view() builds subtitle as: category.or(composer).unwrap_or_default()
    // So if category is set, subtitle = category; composer is hidden.
    // We pre-populate category from item.category, and leave composer empty if category was used
    // as subtitle. This is the documented limitation (U2 note).
    let composer_initial = if item.category.is_some() {
        // Subtitle is category, not composer — we can't recover composer from ViewModel
        String::new()
    } else {
        // No category, subtitle is composer (or empty)
        item.subtitle.clone()
    };

    let title = RwSignal::new(item.title.clone());
    let composer = RwSignal::new(composer_initial);
    let category = RwSignal::new(item.category.clone().unwrap_or_default());
    let key_sig = RwSignal::new(item.key.clone().unwrap_or_default());
    let (initial_marking, initial_bpm) = parse_tempo_display(&item.tempo);
    let tempo_marking = RwSignal::new(initial_marking);
    let bpm = RwSignal::new(initial_bpm);
    let notes = RwSignal::new(item.notes.clone().unwrap_or_default());
    let tags_input = RwSignal::new(item.tags.join(", "));
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click={
                    let id_back = item_id.clone();
                    move |_| { view_state.set(ViewState::Detail(id_back.clone())); }
                }
            >
                "\u{2190} Cancel"
            </button>

            <h2 class="text-2xl font-bold text-slate-900 mb-6">"Edit Exercise"</h2>

            <form
                class="bg-white rounded-xl shadow-sm border border-slate-200 p-6 space-y-5"
                on:submit={
                    let item_id = item_id.clone();
                    move |ev: ev::SubmitEvent| {
                        ev.prevent_default();

                        let validation_errors = validate_exercise_form(
                            &title.get(),
                            &composer.get(),
                            &category.get(),
                            &notes.get(),
                            &bpm.get(),
                            &tempo_marking.get(),
                            &tags_input.get(),
                        );

                        if !validation_errors.is_empty() {
                            errors.set(validation_errors);
                            return;
                        }
                        errors.set(HashMap::new());

                        let title_val = title.get().trim().to_string();
                        let composer_val = {
                            let c = composer.get().trim().to_string();
                            if c.is_empty() { Some(None) } else { Some(Some(c)) }
                        };
                        let category_val = {
                            let c = category.get().trim().to_string();
                            if c.is_empty() { Some(None) } else { Some(Some(c)) }
                        };
                        let key_val = {
                            let k = key_sig.get().trim().to_string();
                            if k.is_empty() { Some(None) } else { Some(Some(k)) }
                        };
                        let tempo_val = {
                            let t = parse_tempo(&tempo_marking.get(), &bpm.get());
                            match t {
                                None => Some(None),
                                Some(v) => Some(Some(v)),
                            }
                        };
                        let notes_val = {
                            let n = notes.get().trim().to_string();
                            if n.is_empty() { Some(None) } else { Some(Some(n)) }
                        };
                        let tags_val = parse_tags(&tags_input.get());

                        let input = UpdateExercise {
                            title: Some(title_val),
                            composer: composer_val,
                            category: category_val,
                            key: key_val,
                            tempo: tempo_val,
                            notes: notes_val,
                            tags: Some(tags_val),
                        };

                        let event = Event::Exercise(ExerciseEvent::Update {
                            id: item_id.clone(),
                            input,
                        });

                        let core_ref = core.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model);
                        view_state.set(ViewState::Detail(item_id.clone()));
                    }
                }
            >
                // Title
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-title">"Title *"</label>
                    <input
                        id="edit-exercise-title"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || title.get()
                        on:input=move |ev| { title.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="title".to_string() errors=errors />
                </div>

                // Composer (optional)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-composer">"Composer"</label>
                    <input
                        id="edit-exercise-composer"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || composer.get()
                        on:input=move |ev| { composer.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="composer".to_string() errors=errors />
                </div>

                // Category
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-category">"Category"</label>
                    <input
                        id="edit-exercise-category"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. Technique, Scales"
                        prop:value=move || category.get()
                        on:input=move |ev| { category.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="category".to_string() errors=errors />
                </div>

                // Key
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-key">"Key"</label>
                    <input
                        id="edit-exercise-key"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. C Major"
                        prop:value=move || key_sig.get()
                        on:input=move |ev| { key_sig.set(event_target_value(&ev)); }
                    />
                </div>

                // Tempo row
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-tempo-marking">"Tempo Marking"</label>
                        <input
                            id="edit-exercise-tempo-marking"
                            type="text"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="e.g. Moderato"
                            prop:value=move || tempo_marking.get()
                            on:input=move |ev| { tempo_marking.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="tempo_marking".to_string() errors=errors />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-bpm">"BPM"</label>
                        <input
                            id="edit-exercise-bpm"
                            type="number"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="1-400"
                            prop:value=move || bpm.get()
                            on:input=move |ev| { bpm.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="bpm".to_string() errors=errors />
                    </div>
                </div>

                // Notes
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-notes">"Notes"</label>
                    <textarea
                        id="edit-exercise-notes"
                        rows="3"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || notes.get()
                        on:input=move |ev| { notes.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="notes".to_string() errors=errors />
                </div>

                // Tags
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-tags">"Tags"</label>
                    <input
                        id="edit-exercise-tags"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="Comma-separated"
                        prop:value=move || tags_input.get()
                        on:input=move |ev| { tags_input.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="tags".to_string() errors=errors />
                </div>

                // Buttons
                <div class="flex gap-3 pt-2">
                    <button
                        type="submit"
                        class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    >
                        "Save"
                    </button>
                    <button
                        type="button"
                        class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                        on:click={
                            let id_cancel = item_id.clone();
                            move |_| { view_state.set(ViewState::Detail(id_cancel.clone())); }
                        }
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }.into_any()
}

// ---------------------------------------------------------------------------
// Helpers for edit form pre-population
// ---------------------------------------------------------------------------

/// Parse a formatted tempo display string back into (marking, bpm_str) for edit form pre-population.
/// Handles: "Allegro (132 BPM)", "Allegro", "132 BPM", None
fn parse_tempo_display(tempo: &Option<String>) -> (String, String) {
    let Some(t) = tempo else {
        return (String::new(), String::new());
    };

    // Pattern: "Marking (BPM_NUMBER BPM)"
    if let Some(paren_start) = t.rfind('(') {
        let marking = t[..paren_start].trim().to_string();
        let bpm_part = &t[paren_start + 1..];
        let bpm_str = bpm_part
            .trim_end_matches(')')
            .trim()
            .trim_end_matches("BPM")
            .trim()
            .to_string();
        return (marking, bpm_str);
    }

    // Pattern: "NUMBER BPM"
    if t.ends_with("BPM") {
        let bpm_str = t.trim_end_matches("BPM").trim().to_string();
        return (String::new(), bpm_str);
    }

    // Just a marking
    (t.clone(), String::new())
}

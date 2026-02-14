use std::cell::RefCell;
use std::rc::Rc;

use chrono::Utc;
use crux_core::Core;
use leptos::prelude::*;

use intrada_core::app::{Effect, StorageEffect};
use intrada_core::domain::exercise::Exercise;
use intrada_core::domain::piece::Piece;
use intrada_core::domain::types::{CreatePiece, Tempo};
use intrada_core::{Event, Intrada, LibraryItemView, ViewModel};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Create the stub data per data-model.md:
/// - Piece: "Clair de Lune" by Debussy, Db Major, Andante très expressif 66 BPM
/// - Exercise: "Hanon No. 1" by Hanon, Technique category, C Major, Moderato 108 BPM
fn create_stub_data() -> (Vec<Piece>, Vec<Exercise>) {
    let now = Utc::now();
    let pieces = vec![Piece {
        id: ulid::Ulid::new().to_string(),
        title: "Clair de Lune".to_string(),
        composer: "Claude Debussy".to_string(),
        key: Some("Db Major".to_string()),
        tempo: Some(Tempo {
            marking: Some("Andante très expressif".to_string()),
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
        notes: Some("The Virtuoso Pianist — Exercise 1".to_string()),
        tags: vec!["technique".to_string(), "warm-up".to_string()],
        created_at: now,
        updated_at: now,
    }];
    (pieces, exercises)
}

/// Process effects returned by the Crux core.
///
/// - `Render(_)`: fire-and-forget (view will be read after all effects)
/// - `Storage(req)`: stub handler — LoadAll returns stub data via DataLoaded event,
///   Save/Update/Delete are no-ops.
///
/// IMPORTANT: Do NOT call `core.resolve()` on notify_shell effects (RequestHandle::Never).
fn process_effects(core: &Core<Intrada>, effects: Vec<Effect>, view_model: &RwSignal<ViewModel>) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {
                // Fire-and-forget — update the reactive signal from core.view()
            }
            Effect::Storage(boxed_request) => {
                match &boxed_request.operation {
                    StorageEffect::LoadAll => {
                        // Return stub data via DataLoaded event
                        let (pieces, exercises) = create_stub_data();
                        let inner_effects =
                            core.process_event(Event::DataLoaded { pieces, exercises });
                        // Recursively process effects from DataLoaded
                        // (will produce a Render effect)
                        process_effects(core, inner_effects, view_model);
                        return; // We've already updated the view inside the recursive call
                    }
                    StorageEffect::SavePiece(_)
                    | StorageEffect::SaveExercise(_)
                    | StorageEffect::UpdatePiece(_)
                    | StorageEffect::UpdateExercise(_)
                    | StorageEffect::DeleteItem { .. } => {
                        // No-op for stub web shell — no persistence
                    }
                }
            }
        }
    }
    // After processing all effects, update the reactive ViewModel signal
    view_model.set(core.view());
}

/// Sample piece names for the "Add Sample Item" button
const SAMPLE_PIECES: &[(&str, &str)] = &[
    ("Moonlight Sonata", "Ludwig van Beethoven"),
    ("Nocturne Op. 9 No. 2", "Frédéric Chopin"),
    ("Gymnopédie No. 1", "Erik Satie"),
    ("Prelude in C Major", "Johann Sebastian Bach"),
    ("Liebesträume No. 3", "Franz Liszt"),
];

#[component]
fn App() -> impl IntoView {
    // Create the Crux core instance, wrapped in Rc<RefCell> for shared access
    let core = Rc::new(RefCell::new(Core::<Intrada>::new()));

    // Create reactive ViewModel signal
    let view_model = RwSignal::new(ViewModel::default());

    // Track sample counter for "Add Sample Item" button
    let sample_counter = RwSignal::new(0_usize);

    // Initialize: load stub data on mount
    {
        let core_ref = core.borrow();
        let (pieces, exercises) = create_stub_data();
        let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
        process_effects(&core_ref, effects, &view_model);
    }

    // Clone for the button handler
    let core_for_button = Rc::clone(&core);

    view! {
        <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-100 text-slate-800">
            // Header with Intrada branding (FR-002)
            <header class="bg-white shadow-sm border-b border-slate-200" role="banner">
                <div class="max-w-4xl mx-auto px-6 py-5 flex items-center justify-between">
                    <div>
                        <h1 class="text-3xl font-bold tracking-tight text-slate-900">
                            "Intrada"
                        </h1>
                        <p class="text-sm text-slate-500 mt-0.5">
                            "Your music practice companion"
                        </p>
                    </div>
                    <span
                        class="inline-flex items-center rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-800"
                        aria-label="Application version"
                    >
                        "v0.1.0"
                    </span>
                </div>
            </header>

            // Main content area
            <main class="max-w-4xl mx-auto px-6 py-10" role="main">
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

                // Error banner (if present) (T022)
                {move || {
                    view_model.get().error.map(|err| {
                        view! {
                            <div
                                class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4"
                                role="alert"
                            >
                                <p class="text-sm text-red-800">
                                    <span class="font-medium">"Error: "</span>
                                    {err}
                                </p>
                            </div>
                        }
                    })
                }}

                // Status message (if present) (T022)
                {move || {
                    view_model.get().status.map(|status| {
                        view! {
                            <div
                                class="mb-6 rounded-lg bg-blue-50 border border-blue-200 p-4"
                                role="status"
                            >
                                <p class="text-sm text-blue-800">{status}</p>
                            </div>
                        }
                    })
                }}

                // Library section with item count + add button (T019, T020, T021)
                <section class="mb-10" aria-labelledby="library-heading">
                    <div class="flex items-center justify-between mb-4">
                        <h2 id="library-heading" class="text-lg font-semibold text-slate-700">
                            "Library"
                        </h2>
                        <div class="flex items-center gap-3">
                            <span class="text-sm text-slate-500">
                                {move || {
                                    let count = view_model.get().item_count;
                                    format!("{count} item(s)")
                                }}
                            </span>
                            // "Add Sample Item" button (T021)
                            <button
                                class="inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3.5 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 transition-colors"
                                aria-label="Add a sample piece to the library"
                                on:click=move |_| {
                                    let idx = sample_counter.get() % SAMPLE_PIECES.len();
                                    let (title, composer) = SAMPLE_PIECES[idx];
                                    sample_counter.set(sample_counter.get() + 1);

                                    let event = Event::Piece(
                                        intrada_core::domain::piece::PieceEvent::Add(CreatePiece {
                                            title: title.to_string(),
                                            composer: composer.to_string(),
                                            key: None,
                                            tempo: None,
                                            notes: None,
                                            tags: vec!["sample".to_string()],
                                        }),
                                    );

                                    let core_ref = core_for_button.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model);
                                }
                            >
                                <span aria-hidden="true">"+"</span>
                                " Add Sample Item"
                            </button>
                        </div>
                    </div>

                    // Library items list (T019, T020)
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
                                            view! { <LibraryItemCard item=item /> }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        }}
                    </div>
                </section>
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

/// A styled card component for a single library item (T019, T020)
#[component]
fn LibraryItemCard(item: LibraryItemView) -> impl IntoView {
    let badge_classes = if item.item_type == "piece" {
        "inline-flex items-center rounded-full bg-violet-100 px-2.5 py-0.5 text-xs font-medium text-violet-800"
    } else {
        "inline-flex items-center rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-medium text-emerald-800"
    };

    // Clone owned values for the view macro (Leptos requires owned Strings)
    let title = item.title.clone();
    let subtitle = item.subtitle.clone();
    let item_type = item.item_type.clone();
    let has_subtitle = !subtitle.is_empty();
    let key = item.key.clone();
    let tempo = item.tempo.clone();
    let tags = item.tags.clone();
    let has_tags = !tags.is_empty();

    view! {
        <li class="bg-white rounded-xl shadow-sm border border-slate-200 p-5 hover:shadow-md transition-shadow">
            <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                    // Title and subtitle
                    <h3 class="text-base font-semibold text-slate-900 truncate">
                        {title}
                    </h3>
                    {if has_subtitle {
                        Some(view! {
                            <p class="text-sm text-slate-500 mt-0.5 truncate">{subtitle}</p>
                        })
                    } else {
                        None
                    }}

                    // Metadata row: key, tempo
                    <div class="flex flex-wrap items-center gap-x-4 gap-y-1 mt-2 text-xs text-slate-400">
                        {key.map(|k| {
                            view! {
                                <span class="flex items-center gap-1">
                                    <span aria-hidden="true">"\u{266F}"</span>
                                    {k}
                                </span>
                            }
                        })}
                        {tempo.map(|t| {
                            view! {
                                <span class="flex items-center gap-1">
                                    <span aria-hidden="true">"\u{2669}"</span>
                                    {t}
                                </span>
                            }
                        })}
                    </div>

                    // Tags
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

                // Type badge
                <span class=badge_classes>
                    {item_type}
                </span>
            </div>
        </li>
    }
}

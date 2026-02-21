use std::collections::HashMap;

use leptos::prelude::*;

use intrada_core::analytics::DailyPracticeTotal;
use intrada_core::{LibraryItemView, SetlistEntryView};

use crate::components::{
    Autocomplete, AutocompleteTextField, BackLink, Button, ButtonVariant, Card, DropIndicator,
    FieldLabel, FormFieldError, LibraryItemCard, LineChart, PageHeading, RoutineSaveForm,
    SetlistEntryRow, StatCard, TagInput, TextArea, TextField, TypeBadge, TypeTabs,
};
use intrada_web::types::ItemType;

/// Dev-only design catalogue at `/design`.
///
/// Renders every UI component in isolation with sample data so designers and
/// developers can see the full design system in one place.
#[component]
pub fn DesignCatalogue() -> impl IntoView {
    // ── Sample data ────────────────────────────────────────────────────

    let type_tab_active = RwSignal::new(ItemType::Piece);
    let sample_text = RwSignal::new(String::new());
    let sample_text_filled = RwSignal::new("Clair de Lune".to_string());
    let sample_area = RwSignal::new(String::new());
    let sample_area_filled = RwSignal::new(
        "Focus on smooth legato phrasing in the arpeggiated section. Watch dynamics in the climax."
            .to_string(),
    );
    let sample_area_error_val = RwSignal::new(String::new());

    let empty_errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let sample_errors: RwSignal<HashMap<String, String>> = RwSignal::new({
        let mut m = HashMap::new();
        m.insert("title".to_string(), "Title is required".to_string());
        m
    });
    let area_errors: RwSignal<HashMap<String, String>> = RwSignal::new({
        let mut m = HashMap::new();
        m.insert(
            "notes".to_string(),
            "Notes cannot exceed 500 characters".to_string(),
        );
        m
    });

    let sample_piece = LibraryItemView {
        id: "sample-1".to_string(),
        item_type: "piece".to_string(),
        title: "Clair de Lune".to_string(),
        subtitle: "Claude Debussy".to_string(),
        category: Some("Romantic".to_string()),
        key: Some("Db Major".to_string()),
        tempo: Some("66 bpm".to_string()),
        notes: None,
        tags: vec!["recital".to_string(), "impressionist".to_string()],
        created_at: "2025-01-15".to_string(),
        updated_at: "2025-02-01".to_string(),
        practice: None,
    };

    let sample_exercise = LibraryItemView {
        id: "sample-2".to_string(),
        item_type: "exercise".to_string(),
        title: "Hanon No. 1".to_string(),
        subtitle: "C Major scale pattern".to_string(),
        category: Some("Technical".to_string()),
        key: Some("C Major".to_string()),
        tempo: Some("120 bpm".to_string()),
        notes: None,
        tags: vec!["warm-up".to_string()],
        created_at: "2025-01-10".to_string(),
        updated_at: "2025-01-20".to_string(),
        practice: None,
    };

    let sample_minimal = LibraryItemView {
        id: "sample-3".to_string(),
        item_type: "piece".to_string(),
        title: "Prelude in C Major".to_string(),
        subtitle: String::new(),
        category: None,
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: "2025-03-01".to_string(),
        updated_at: "2025-03-01".to_string(),
        practice: None,
    };

    let sample_long_title = LibraryItemView {
        id: "sample-4".to_string(),
        item_type: "exercise".to_string(),
        title:
            "Scales and Arpeggios in All Major and Minor Keys — Two Octaves with Contrary Motion"
                .to_string(),
        subtitle: "ABRSM Grade 5 Syllabus 2024-2025".to_string(),
        category: Some("Technical".to_string()),
        key: None,
        tempo: Some("80 bpm".to_string()),
        notes: None,
        tags: vec![
            "exam".to_string(),
            "abrsm".to_string(),
            "grade-5".to_string(),
            "scales".to_string(),
        ],
        created_at: "2025-02-20".to_string(),
        updated_at: "2025-02-20".to_string(),
        practice: None,
    };

    let chart_data: Vec<DailyPracticeTotal> = (1..=28)
        .map(|day| DailyPracticeTotal {
            date: format!("2025-02-{:02}", day),
            minutes: match day % 7 {
                0 => 0,
                1 => 30,
                2 => 45,
                3 => 20,
                4 => 60,
                5 => 35,
                _ => 15,
            },
        })
        .collect();

    // Autocomplete sample data
    let autocomplete_value = RwSignal::new(String::new());
    let autocomplete_field_value = RwSignal::new(String::new());
    let composers: Signal<Vec<String>> = Signal::derive(|| {
        vec![
            "Johann Sebastian Bach".to_string(),
            "Ludwig van Beethoven".to_string(),
            "Frédéric Chopin".to_string(),
            "Claude Debussy".to_string(),
            "George Gershwin".to_string(),
            "Franz Liszt".to_string(),
            "Wolfgang Amadeus Mozart".to_string(),
            "Sergei Rachmaninoff".to_string(),
            "Franz Schubert".to_string(),
            "Pyotr Ilyich Tchaikovsky".to_string(),
        ]
    });

    // TagInput sample data
    let sample_tags = RwSignal::new(vec!["warm-up".to_string(), "recital".to_string()]);
    let available_tags: Signal<Vec<String>> = Signal::derive(|| {
        vec![
            "warm-up".to_string(),
            "recital".to_string(),
            "exam".to_string(),
            "sight-reading".to_string(),
            "improvisation".to_string(),
            "classical".to_string(),
            "jazz".to_string(),
            "scales".to_string(),
            "arpeggios".to_string(),
        ]
    });
    let tag_errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    // SetlistEntryRow sample data
    let entry_full = SetlistEntryView {
        id: "entry-1".to_string(),
        item_id: "item-1".to_string(),
        item_title: "Clair de Lune".to_string(),
        item_type: "piece".to_string(),
        position: 0,
        duration_display: "5m 32s".to_string(),
        status: "completed".to_string(),
        notes: None,
        score: Some(4),
    };
    let entry_display = SetlistEntryView {
        id: "entry-2".to_string(),
        item_id: "item-2".to_string(),
        item_title: "Hanon No. 1".to_string(),
        item_type: "exercise".to_string(),
        position: 1,
        duration_display: "3m 10s".to_string(),
        status: "pending".to_string(),
        notes: None,
        score: None,
    };
    let entry_drag = SetlistEntryView {
        id: "entry-3".to_string(),
        item_id: "item-3".to_string(),
        item_title: "Chromatic Scales".to_string(),
        item_type: "exercise".to_string(),
        position: 2,
        duration_display: "2m 05s".to_string(),
        status: "pending".to_string(),
        notes: None,
        score: None,
    };

    view! {
        <div class="space-y-12">
            <PageHeading text="Design System Catalogue" />
            <p class="text-sm text-gray-400 -mt-4 mb-8">
                "Dev-only reference of all UI components and design tokens. "
                "See " <code class="text-xs bg-white/10 rounded px-1 py-0.5">"specs/design-system.md"</code> " for full documentation."
            </p>

            // ── Table of Contents ─────────────────────────────────────
            <nav class="glass-card p-4 sm:p-6" aria-label="Catalogue navigation">
                <h3 class="text-sm font-semibold text-white mb-3">"Contents"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-1">
                    <div>
                        <p class="text-xs font-medium text-gray-400 uppercase mb-1">"Tokens"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#colours" class="text-indigo-300 hover:text-white">"Colours"</a></li>
                            <li><a href="#typography" class="text-indigo-300 hover:text-white">"Typography"</a></li>
                            <li><a href="#badges-tokens" class="text-indigo-300 hover:text-white">"Badge Colours"</a></li>
                            <li><a href="#radii" class="text-indigo-300 hover:text-white">"Radii"</a></li>
                            <li><a href="#utilities" class="text-indigo-300 hover:text-white">"Composite Utilities"</a></li>
                        </ul>
                    </div>
                    <div>
                        <p class="text-xs font-medium text-gray-400 uppercase mb-1">"Components"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#glass-card" class="text-indigo-300 hover:text-white">"Glass Card"</a></li>
                            <li><a href="#stat-card" class="text-indigo-300 hover:text-white">"Stat Card"</a></li>
                            <li><a href="#library-item-card" class="text-indigo-300 hover:text-white">"Library Item Card"</a></li>
                            <li><a href="#buttons" class="text-indigo-300 hover:text-white">"Buttons"</a></li>
                            <li><a href="#type-badge" class="text-indigo-300 hover:text-white">"Type Badge"</a></li>
                            <li><a href="#type-tabs" class="text-indigo-300 hover:text-white">"Type Tabs"</a></li>
                            <li><a href="#error-banner" class="text-indigo-300 hover:text-white">"Error Banner"</a></li>
                        </ul>
                    </div>
                    <div>
                        <p class="text-xs font-medium text-gray-400 uppercase mb-1">"Forms & Session"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#text-field" class="text-indigo-300 hover:text-white">"Text Field"</a></li>
                            <li><a href="#text-area" class="text-indigo-300 hover:text-white">"Text Area"</a></li>
                            <li><a href="#autocomplete" class="text-indigo-300 hover:text-white">"Autocomplete"</a></li>
                            <li><a href="#tag-input" class="text-indigo-300 hover:text-white">"Tag Input"</a></li>
                            <li><a href="#field-label" class="text-indigo-300 hover:text-white">"Field Label"</a></li>
                            <li><a href="#navigation" class="text-indigo-300 hover:text-white">"Navigation"</a></li>
                            <li><a href="#line-chart" class="text-indigo-300 hover:text-white">"Line Chart"</a></li>
                            <li><a href="#setlist-entry" class="text-indigo-300 hover:text-white">"Setlist Entry"</a></li>
                            <li><a href="#drag-drop" class="text-indigo-300 hover:text-white">"Drag & Drop"</a></li>
                            <li><a href="#routine-save" class="text-indigo-300 hover:text-white">"Routine Save Form"</a></li>
                            <li><a href="#shell" class="text-indigo-300 hover:text-white">"Shell Components"</a></li>
                        </ul>
                    </div>
                </div>
            </nav>

            // ══════════════════════════════════════════════════════════
            // TOKENS
            // ══════════════════════════════════════════════════════════

            // ── Colour Palette ────────────────────────────────────────
            <section id="colours">
                <h3 class="text-lg font-semibold text-white mb-4">"Colour Palette"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    // Surfaces
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-gray-400 uppercase">"Surfaces"</p>
                        <div class="h-12 rounded-lg bg-surface-primary border border-border-card"></div>
                        <p class="text-xs text-gray-500">"surface-primary"</p>
                        <div class="h-12 rounded-lg bg-surface-secondary border border-border-card"></div>
                        <p class="text-xs text-gray-500">"surface-secondary"</p>
                        <div class="h-12 rounded-lg bg-surface-chrome border border-border-card"></div>
                        <p class="text-xs text-gray-500">"surface-chrome"</p>
                        <div class="h-12 rounded-lg bg-surface-fallback border border-border-card"></div>
                        <p class="text-xs text-gray-500">"surface-fallback"</p>
                        <div class="h-12 rounded-lg bg-surface-hover border border-border-card"></div>
                        <p class="text-xs text-gray-500">"surface-hover"</p>
                        <div class="h-12 rounded-lg bg-surface-input border border-border-card"></div>
                        <p class="text-xs text-gray-500">"surface-input"</p>
                    </div>

                    // Accent
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-gray-400 uppercase">"Accent"</p>
                        <div class="h-12 rounded-lg bg-accent"></div>
                        <p class="text-xs text-gray-500">"accent"</p>
                        <div class="h-12 rounded-lg bg-accent-hover"></div>
                        <p class="text-xs text-gray-500">"accent-hover"</p>
                        <div class="h-12 rounded-lg bg-accent-text"></div>
                        <p class="text-xs text-gray-500">"accent-text"</p>
                        <div class="h-12 rounded-lg bg-accent-focus"></div>
                        <p class="text-xs text-gray-500">"accent-focus"</p>
                    </div>

                    // Danger
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-gray-400 uppercase">"Danger"</p>
                        <div class="h-12 rounded-lg bg-danger"></div>
                        <p class="text-xs text-gray-500">"danger"</p>
                        <div class="h-12 rounded-lg bg-danger-hover"></div>
                        <p class="text-xs text-gray-500">"danger-hover"</p>
                        <div class="h-12 rounded-lg bg-danger-text"></div>
                        <p class="text-xs text-gray-500">"danger-text"</p>
                        <div class="h-12 rounded-lg bg-danger-surface"></div>
                        <p class="text-xs text-gray-500">"danger-surface"</p>
                    </div>

                    // Borders & Text
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-gray-400 uppercase">"Borders"</p>
                        <div class="h-12 rounded-lg border-2 border-border-default"></div>
                        <p class="text-xs text-gray-500">"border-default"</p>
                        <div class="h-12 rounded-lg border-2 border-border-card"></div>
                        <p class="text-xs text-gray-500">"border-card"</p>
                        <div class="h-12 rounded-lg border-2 border-border-input"></div>
                        <p class="text-xs text-gray-500">"border-input"</p>
                    </div>
                </div>

                // Text colour samples
                <div class="mt-6 space-y-1">
                    <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Text Colours"</p>
                    <p class="text-text-primary">"text-primary — Headings and titles"</p>
                    <p class="text-text-secondary">"text-secondary — Body text"</p>
                    <p class="text-text-label">"text-label — Form labels"</p>
                    <p class="text-text-muted">"text-muted — Hints and metadata"</p>
                    <p class="text-text-faint">"text-faint — Timestamps, tertiary info"</p>
                    <p class="text-accent-text">"accent-text — Active navigation"</p>
                    <p class="text-danger-text">"danger-text — Error text"</p>
                </div>
            </section>

            // ── Typography ────────────────────────────────────────────
            <section id="typography">
                <h3 class="text-lg font-semibold text-white mb-4">"Typography"</h3>
                <Card>
                    <div class="space-y-3">
                        <h1 class="text-3xl font-bold text-white">"Heading 1 — 3xl bold"</h1>
                        <h2 class="text-2xl font-bold text-white">"Heading 2 — 2xl bold"</h2>
                        <h3 class="text-lg font-semibold text-white">"Heading 3 — lg semibold"</h3>
                        <p class="text-base text-gray-300">"Body text — base / gray-300"</p>
                        <p class="text-sm text-gray-400">"Small text — sm / gray-400"</p>
                        <p class="text-xs text-gray-500">"Extra small — xs / gray-500"</p>
                        <p class="text-xs font-medium text-gray-400 uppercase tracking-wider">"Label style — xs medium uppercase tracking-wider"</p>
                        <p class="text-4xl sm:text-6xl font-mono text-white">"12:34"<span class="text-sm text-gray-400 ml-2">"Mono large — timer display"</span></p>
                    </div>
                </Card>
            </section>

            // ── Badge Colours ─────────────────────────────────────────
            <section id="badges-tokens">
                <h3 class="text-lg font-semibold text-white mb-4">"Badge Colours"</h3>
                <Card>
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg bg-badge-piece-bg border border-border-card"></div>
                            <p class="text-xs text-gray-500">"badge-piece-bg"</p>
                        </div>
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg flex items-center justify-center">
                                <span class="text-badge-piece-text font-medium">"Piece Text"</span>
                            </div>
                            <p class="text-xs text-gray-500">"badge-piece-text"</p>
                        </div>
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg bg-badge-exercise-bg border border-border-card"></div>
                            <p class="text-xs text-gray-500">"badge-exercise-bg"</p>
                        </div>
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg flex items-center justify-center">
                                <span class="text-badge-exercise-text font-medium">"Exercise Text"</span>
                            </div>
                            <p class="text-xs text-gray-500">"badge-exercise-text"</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Radius Tokens ─────────────────────────────────────────
            <section id="radii">
                <h3 class="text-lg font-semibold text-white mb-4">"Radius Tokens"</h3>
                <Card>
                    <div class="flex flex-wrap gap-6 items-end">
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-card"></div>
                            <p class="text-xs text-gray-500">"radius-card"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-button"></div>
                            <p class="text-xs text-gray-500">"radius-button"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-input"></div>
                            <p class="text-xs text-gray-500">"radius-input"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-badge"></div>
                            <p class="text-xs text-gray-500">"radius-badge"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-pill"></div>
                            <p class="text-xs text-gray-500">"radius-pill"</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Composite Utilities ───────────────────────────────────
            <section id="utilities">
                <h3 class="text-lg font-semibold text-white mb-4">"Composite Utilities"</h3>
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div class="space-y-2">
                        <div class="glass-card p-4">
                            <p class="text-sm text-gray-300">"Content inside glass-card"</p>
                        </div>
                        <p class="text-xs text-gray-500 text-center">"glass-card"</p>
                        <p class="text-xs text-gray-600 text-center">"Glassmorphism + fallback + border + shadow"</p>
                    </div>
                    <div class="space-y-2">
                        <div class="glass-chrome border border-border-default p-4">
                            <p class="text-sm text-gray-300">"Content inside glass-chrome"</p>
                        </div>
                        <p class="text-xs text-gray-500 text-center">"glass-chrome"</p>
                        <p class="text-xs text-gray-600 text-center">"Neutral chrome for nav bars"</p>
                    </div>
                    <div class="space-y-2">
                        <input
                            type="text"
                            class="input-base"
                            placeholder="Content inside input-base"
                            readonly
                        />
                        <p class="text-xs text-gray-500 text-center">"input-base"</p>
                        <p class="text-xs text-gray-600 text-center">"Border + bg + focus ring + sizing"</p>
                    </div>
                </div>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Containers
            // ══════════════════════════════════════════════════════════

            // ── Glass Card ────────────────────────────────────────────
            <section id="glass-card">
                <h3 class="text-lg font-semibold text-white mb-4">"Glass Card"</h3>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <Card>
                        <p class="text-sm text-gray-300">"Default glass-card with standard padding."</p>
                    </Card>
                    <Card>
                        <h3 class="text-lg font-semibold text-white mb-2">"With heading"</h3>
                        <p class="text-sm text-gray-400">"Card content with heading and body text."</p>
                    </Card>
                    <Card>
                        <h3 class="text-lg font-semibold text-white mb-3">"With divider"</h3>
                        <div class="border-b border-border-default mb-3"></div>
                        <p class="text-sm text-gray-300">"Content below a horizontal divider."</p>
                        <div class="border-b border-border-default my-3"></div>
                        <p class="text-xs text-gray-500">"Footer-style content in the card."</p>
                    </Card>
                    <Card>
                        <h3 class="text-lg font-semibold text-white mb-3">"Nested content"</h3>
                        <div class="space-y-3">
                            <div class="bg-white/5 rounded-lg p-3">
                                <p class="text-sm text-gray-300">"Nested inner container"</p>
                            </div>
                            <div class="flex gap-2">
                                <Button variant=ButtonVariant::Primary>"Action"</Button>
                                <Button variant=ButtonVariant::Secondary>"Cancel"</Button>
                            </div>
                        </div>
                    </Card>
                </div>
            </section>

            // ── Stat Cards ────────────────────────────────────────────
            <section id="stat-card">
                <h3 class="text-lg font-semibold text-white mb-4">"Stat Card"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    <StatCard title="Current Streak" value="7 days".to_string() subtitle="Best: 14 days" />
                    <StatCard title="This Week" value="3h 45m".to_string() />
                    <StatCard title="Sessions" value="12".to_string() subtitle="This month" />
                    <StatCard title="Avg Score" value="3.8".to_string() subtitle="Out of 5" />
                </div>
            </section>

            // ── Library Item Cards ────────────────────────────────────
            <section id="library-item-card">
                <h3 class="text-lg font-semibold text-white mb-4">"Library Item Card"</h3>
                <div class="space-y-3">
                    <p class="text-xs font-medium text-gray-400 uppercase">"Full metadata (piece)"</p>
                    <LibraryItemCard item=sample_piece />

                    <p class="text-xs font-medium text-gray-400 uppercase mt-6">"Full metadata (exercise)"</p>
                    <LibraryItemCard item=sample_exercise />

                    <p class="text-xs font-medium text-gray-400 uppercase mt-6">"Minimal (no subtitle, tags, key, or tempo)"</p>
                    <LibraryItemCard item=sample_minimal />

                    <p class="text-xs font-medium text-gray-400 uppercase mt-6">"Long title + many tags"</p>
                    <LibraryItemCard item=sample_long_title />
                </div>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Display
            // ══════════════════════════════════════════════════════════

            // ── Buttons ───────────────────────────────────────────────
            <section id="buttons">
                <h3 class="text-lg font-semibold text-white mb-4">"Buttons"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Normal"</p>
                            <div class="flex flex-wrap gap-3">
                                <Button variant=ButtonVariant::Primary>"Primary"</Button>
                                <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                                <Button variant=ButtonVariant::Danger>"Danger"</Button>
                                <Button variant=ButtonVariant::DangerOutline>"Danger Outline"</Button>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Disabled"</p>
                            <div class="flex flex-wrap gap-3">
                                <Button variant=ButtonVariant::Primary disabled=Signal::derive(|| true)>"Primary"</Button>
                                <Button variant=ButtonVariant::Secondary disabled=Signal::derive(|| true)>"Secondary"</Button>
                                <Button variant=ButtonVariant::Danger disabled=Signal::derive(|| true)>"Danger"</Button>
                                <Button variant=ButtonVariant::DangerOutline disabled=Signal::derive(|| true)>"Danger Outline"</Button>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Loading"</p>
                            <div class="flex flex-wrap gap-3">
                                <Button variant=ButtonVariant::Primary loading=Signal::derive(|| true)>"Saving..."</Button>
                                <Button variant=ButtonVariant::Secondary loading=Signal::derive(|| true)>"Loading..."</Button>
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Type Badge ────────────────────────────────────────────
            <section id="type-badge">
                <h3 class="text-lg font-semibold text-white mb-4">"Type Badge"</h3>
                <Card>
                    <div class="flex flex-wrap gap-3">
                        <TypeBadge item_type="piece".to_string() />
                        <TypeBadge item_type="exercise".to_string() />
                        <TypeBadge item_type="unknown".to_string() />
                    </div>
                </Card>
            </section>

            // ── Type Tabs ─────────────────────────────────────────────
            <section id="type-tabs">
                <h3 class="text-lg font-semibold text-white mb-4">"Type Tabs"</h3>
                <Card>
                    <div class="space-y-3">
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Interactive"</p>
                            <TypeTabs
                                active=Signal::derive(move || type_tab_active.get())
                                on_change=Callback::new(move |t| type_tab_active.set(t))
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Display-only (Piece)"</p>
                            <TypeTabs active=Signal::derive(|| ItemType::Piece) />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Display-only (Exercise)"</p>
                            <TypeTabs active=Signal::derive(|| ItemType::Exercise) />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Error Banner ──────────────────────────────────────────
            <section id="error-banner">
                <h3 class="text-lg font-semibold text-white mb-4">"Error Banner"</h3>
                <p class="text-xs text-gray-500 mb-3">"Static preview — the real component reads from ViewModel context."</p>
                // Static mockup of ErrorBanner (can't use the component without context)
                <div class="mb-6 rounded-lg bg-red-500/10 border border-red-400/20 p-4" role="alert">
                    <div class="flex items-start justify-between gap-3">
                        <p class="text-sm text-red-300">
                            <span class="font-medium">"Error: "</span>"Failed to save session. Please check your connection and try again."
                        </p>
                        <button class="shrink-0 text-red-400 hover:text-red-300 text-xs font-medium">
                            "Dismiss"
                        </button>
                    </div>
                </div>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Forms
            // ══════════════════════════════════════════════════════════

            // ── Form Inputs ───────────────────────────────────────────
            <section id="text-field">
                <h3 class="text-lg font-semibold text-white mb-4">"Text Field"</h3>
                <Card>
                    <div class="space-y-4">
                        <TextField
                            id="demo-text"
                            label="Empty with placeholder"
                            value=sample_text
                            field_name="demo"
                            errors=empty_errors
                            placeholder="Enter some text..."
                        />
                        <TextField
                            id="demo-text-filled"
                            label="Pre-filled value"
                            value=sample_text_filled
                            field_name="title"
                            errors=empty_errors
                        />
                        <TextField
                            id="demo-text-hint"
                            label="With hint text"
                            value=sample_text
                            field_name="subtitle"
                            errors=empty_errors
                            placeholder="e.g. Claude Debussy"
                            hint="The composer or source of the piece"
                        />
                        <TextField
                            id="demo-text-required"
                            label="Required field"
                            value=sample_text
                            field_name="title_req"
                            errors=empty_errors
                            required=true
                            placeholder="Required..."
                        />
                        <TextField
                            id="demo-text-error"
                            label="With validation error"
                            value=sample_text
                            field_name="title"
                            errors=sample_errors
                            required=true
                        />
                    </div>
                </Card>
            </section>

            <section id="text-area">
                <h3 class="text-lg font-semibold text-white mb-4">"Text Area"</h3>
                <Card>
                    <div class="space-y-4">
                        <TextArea
                            id="demo-area"
                            label="Empty with hint"
                            value=sample_area
                            field_name="notes"
                            errors=empty_errors
                            hint="Optional hint text below the label"
                        />
                        <TextArea
                            id="demo-area-filled"
                            label="Pre-filled content"
                            value=sample_area_filled
                            field_name="notes_filled"
                            errors=empty_errors
                        />
                        <TextArea
                            id="demo-area-error"
                            label="With validation error"
                            value=sample_area_error_val
                            field_name="notes"
                            errors=area_errors
                        />
                    </div>
                </Card>
            </section>

            // ── Autocomplete ──────────────────────────────────────────
            <section id="autocomplete">
                <h3 class="text-lg font-semibold text-white mb-4">"Autocomplete"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Standalone autocomplete"</p>
                            <p class="text-xs text-gray-500 mb-2">"Type 2+ characters to see suggestions (try \"ba\" or \"ch\")"</p>
                            <Autocomplete
                                id="demo-autocomplete"
                                suggestions=composers
                                value=autocomplete_value
                                on_select=Callback::new(move |s: String| autocomplete_value.set(s))
                                placeholder="Search composers..."
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"AutocompleteTextField (with label + error)"</p>
                            <AutocompleteTextField
                                id="demo-autocomplete-field"
                                label="Composer"
                                value=autocomplete_field_value
                                suggestions=composers
                                placeholder="Start typing a composer name..."
                                field_name="composer"
                                errors=empty_errors
                            />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Tag Input ─────────────────────────────────────────────
            <section id="tag-input">
                <h3 class="text-lg font-semibold text-white mb-4">"Tag Input"</h3>
                <Card>
                    <p class="text-xs text-gray-500 mb-3">"Pre-populated with sample tags. Type to add more, click × to remove."</p>
                    <TagInput
                        id="demo-tags"
                        tags=sample_tags
                        available_tags=available_tags
                        field_name="tags"
                        errors=tag_errors
                    />
                </Card>
            </section>

            // ── Field Label ───────────────────────────────────────────
            <section id="field-label">
                <h3 class="text-lg font-semibold text-white mb-4">"Field Label"</h3>
                <Card>
                    <dl class="space-y-2">
                        <FieldLabel text="Key Signature" />
                        <dd class="text-white">"Db Major"</dd>
                        <FieldLabel text="Tempo" />
                        <dd class="text-white">"66 bpm"</dd>
                        <FieldLabel text="Category" />
                        <dd class="text-white">"Romantic"</dd>
                    </dl>
                </Card>
            </section>

            // ── Form Field Error (standalone) ─────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Form Field Error"</h3>
                <Card>
                    <FormFieldError field="title" errors=sample_errors />
                </Card>
            </section>

            // ── Navigation ────────────────────────────────────────────
            <section id="navigation">
                <h3 class="text-lg font-semibold text-white mb-4">"Navigation"</h3>
                <Card>
                    <div class="space-y-3">
                        <BackLink label="Back to Library" href="/".to_string() />
                        <PageHeading text="Sample Page Heading" />
                    </div>
                </Card>
            </section>

            // ── Line Chart ────────────────────────────────────────────
            <section id="line-chart">
                <h3 class="text-lg font-semibold text-white mb-4">"Line Chart"</h3>
                <Card>
                    <LineChart data=chart_data />
                </Card>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Session
            // ══════════════════════════════════════════════════════════

            // ── Setlist Entry Row ─────────────────────────────────────
            <section id="setlist-entry">
                <h3 class="text-lg font-semibold text-white mb-4">"Setlist Entry Row"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"With controls (remove, move up/down)"</p>
                            <SetlistEntryRow
                                entry=entry_full
                                on_remove=Some(Callback::new(|_: String| {}))
                                on_move_up=Some(Callback::new(|_: String| {}))
                                on_move_down=Some(Callback::new(|_: String| {}))
                                show_controls=true
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Display-only (no controls)"</p>
                            <SetlistEntryRow
                                entry=entry_display
                                show_controls=false
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Drag-active state"</p>
                            <SetlistEntryRow
                                entry=entry_drag
                                show_controls=false
                                is_dragging_this=Signal::derive(|| true)
                            />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Drag Handle + Drop Indicator ──────────────────────────
            <section id="drag-drop">
                <h3 class="text-lg font-semibold text-white mb-4">"Drag & Drop Primitives"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Drag Handle"</p>
                            <p class="text-xs text-gray-500 mb-2">"Six-dot grip icon, 44×44px touch target. Used inside SetlistEntryRow."</p>
                            <div class="flex items-center gap-3 rounded-lg bg-white/5 px-4 py-3">
                                // Static preview of the grip icon (can't use DragHandle without a real callback)
                                <div class="flex items-center justify-center w-11 h-11 min-w-[44px] min-h-[44px] cursor-grab text-gray-500">
                                    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                                        <circle cx="5" cy="3" r="1.5" />
                                        <circle cx="11" cy="3" r="1.5" />
                                        <circle cx="5" cy="8" r="1.5" />
                                        <circle cx="11" cy="8" r="1.5" />
                                        <circle cx="5" cy="13" r="1.5" />
                                        <circle cx="11" cy="13" r="1.5" />
                                    </svg>
                                </div>
                                <span class="text-sm text-gray-300">"Drag me to reorder"</span>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Drop Indicator"</p>
                            <p class="text-xs text-gray-500 mb-2">"Indigo-400 line between entries during drag. Always occupies layout space."</p>
                            <div class="space-y-2">
                                <div class="flex items-center gap-3 rounded-lg bg-white/5 px-4 py-3">
                                    <span class="text-sm text-gray-300">"Entry above"</span>
                                </div>
                                <DropIndicator visible=Signal::derive(|| true) />
                                <div class="flex items-center gap-3 rounded-lg bg-white/5 px-4 py-3">
                                    <span class="text-sm text-gray-300">"Entry below"</span>
                                </div>
                                <DropIndicator visible=Signal::derive(|| false) />
                                <div class="flex items-center gap-3 rounded-lg bg-white/5 px-4 py-3">
                                    <span class="text-sm text-gray-300">"No indicator visible here"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Routine Save Form ─────────────────────────────────────
            <section id="routine-save">
                <h3 class="text-lg font-semibold text-white mb-4">"Routine Save Form"</h3>
                <p class="text-xs text-gray-500 mb-3">"Click the dashed button to expand the form. Interactive — try saving without a name."</p>
                <RoutineSaveForm on_save=Callback::new(|_name: String| {}) />
            </section>

            // ── Shell Components ──────────────────────────────────────
            <section id="shell">
                <h3 class="text-lg font-semibold text-white mb-4">"Shell Components"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"App Header"</p>
                            <p class="text-xs text-gray-500">"Visible at the top of this page. Uses "<code class="bg-white/10 rounded px-1">"glass-chrome"</code>" utility, "<code class="bg-white/10 rounded px-1">"border-border-default"</code>" bottom border. Desktop-only nav links with "<code class="bg-white/10 rounded px-1">"text-accent-text"</code>" active state."</p>
                        </div>
                        <div class="border-b border-border-default"></div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"Bottom Tab Bar"</p>
                            <p class="text-xs text-gray-500">"Visible on mobile (below 640px). Fixed bottom, "<code class="bg-white/10 rounded px-1">"glass-chrome"</code>" + "<code class="bg-white/10 rounded px-1">"pb-safe"</code>" for iOS safe area. 4 tabs with SVG icons, 44px min touch target."</p>
                        </div>
                        <div class="border-b border-border-default"></div>
                        <div>
                            <p class="text-xs font-medium text-gray-400 uppercase mb-2">"App Footer"</p>
                            <p class="text-xs text-gray-500">"Visible at the bottom of this page. "<code class="bg-white/10 rounded px-1">"border-white/10"</code>" top border, "<code class="bg-white/10 rounded px-1">"text-xs text-gray-500"</code>" centered attribution text."</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Session Components (context-dependent) ────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Session Components"</h3>
                <Card>
                    <p class="text-sm text-gray-400 mb-3">"These components require app context (ViewModel, Core) and cannot be rendered in isolation. They are composed from the primitives shown above."</p>
                    <div class="space-y-2 text-sm text-gray-500">
                        <div class="flex items-center gap-2">
                            <span class="text-indigo-400">"→"</span>
                            <span>"SessionTimer — Card + TypeBadge + SetlistEntryRow + Button (timer, next/finish/skip)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-indigo-400">"→"</span>
                            <span>"SessionSummary — Card + Button + RoutineSaveForm (scoring, notes, save)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-indigo-400">"→"</span>
                            <span>"SetlistBuilder — Card + SetlistEntryRow + DragHandle + DropIndicator + RoutineLoader"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-indigo-400">"→"</span>
                            <span>"RoutineLoader — Card (routine list with load buttons)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-indigo-400">"→"</span>
                            <span>"ErrorBanner — shown above as static preview"</span>
                        </div>
                    </div>
                </Card>
            </section>
        </div>
    }
}

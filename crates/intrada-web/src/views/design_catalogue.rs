use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use leptos::prelude::*;

use intrada_core::analytics::DailyPracticeTotal;
use intrada_core::{EntryStatus, ItemKind, LibraryItemView, SetlistEntryView, TempoHistoryEntry};

use crate::components::{
    AccentBar, AccentRow, Autocomplete, AutocompleteTextField, BackLink, BottomSheet, Button,
    ButtonSize, ButtonVariant, Card, CircularButton, CircularButtonSize, CircularButtonVariant,
    ContextMenu, ContextMenuAction, DayCell, DetailGroup, DetailRow, DropIndicator, EmptyState,
    FieldLabel, FormFieldError, IconName, InlineTypeIndicator, LibraryItemCard, LineChart,
    PageHeading, ProgressRing, RoutineSaveForm, SectionLabel, SetlistEntryRow, SkeletonBlock,
    SkeletonCardList, SkeletonItemCard, SkeletonLine, StatCard, StatTone, SwipeActions, TagInput,
    TempoProgressChart, TextArea, TextField, Toast, ToastVariant, TransitionPrompt, TypeBadge,
    TypeTabs, WeekStrip,
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
    let sample_text_hint = RwSignal::new(String::new());
    let sample_text_required = RwSignal::new(String::new());
    let sample_text_error = RwSignal::new(String::new());
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
        item_type: ItemKind::Piece,
        title: "Clair de Lune".to_string(),
        subtitle: "Claude Debussy".to_string(),
        key: Some("Db Major".to_string()),
        tempo: Some("66 bpm".to_string()),
        notes: None,
        tags: vec!["recital".to_string(), "impressionist".to_string()],
        created_at: "2025-01-15".to_string(),
        updated_at: "2025-02-01".to_string(),
        practice: None,
        latest_achieved_tempo: None,
    };

    let sample_exercise = LibraryItemView {
        id: "sample-2".to_string(),
        item_type: ItemKind::Exercise,
        title: "Hanon No. 1".to_string(),
        subtitle: "C Major scale pattern".to_string(),
        key: Some("C Major".to_string()),
        tempo: Some("120 bpm".to_string()),
        notes: None,
        tags: vec!["warm-up".to_string()],
        created_at: "2025-01-10".to_string(),
        updated_at: "2025-01-20".to_string(),
        practice: None,
        latest_achieved_tempo: None,
    };

    let sample_minimal = LibraryItemView {
        id: "sample-3".to_string(),
        item_type: ItemKind::Piece,
        title: "Prelude in C Major".to_string(),
        subtitle: String::new(),
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: "2025-03-01".to_string(),
        updated_at: "2025-03-01".to_string(),
        practice: None,
        latest_achieved_tempo: None,
    };

    let sample_long_title = LibraryItemView {
        id: "sample-4".to_string(),
        item_type: ItemKind::Exercise,
        title:
            "Scales and Arpeggios in All Major and Minor Keys — Two Octaves with Contrary Motion"
                .to_string(),
        subtitle: "ABRSM Grade 5 Syllabus 2024-2025".to_string(),
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
        latest_achieved_tempo: None,
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

    // Tempo progress chart sample data — upward trend toward 120 BPM target
    let tempo_entries: Vec<TempoHistoryEntry> = vec![
        TempoHistoryEntry {
            session_date: "2026-01-05T10:00:00Z".to_string(),
            tempo: 60,
            session_id: "s1".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-01-12T10:00:00Z".to_string(),
            tempo: 68,
            session_id: "s2".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-01-19T10:00:00Z".to_string(),
            tempo: 72,
            session_id: "s3".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-01-26T10:00:00Z".to_string(),
            tempo: 78,
            session_id: "s4".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-02-02T10:00:00Z".to_string(),
            tempo: 85,
            session_id: "s5".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-02-09T10:00:00Z".to_string(),
            tempo: 82,
            session_id: "s6".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-02-16T10:00:00Z".to_string(),
            tempo: 90,
            session_id: "s7".to_string(),
        },
        TempoHistoryEntry {
            session_date: "2026-02-23T10:00:00Z".to_string(),
            tempo: 95,
            session_id: "s8".to_string(),
        },
    ];
    let tempo_target: Option<u16> = Some(120);
    let tempo_latest: Option<u16> = Some(95);

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
        item_type: ItemKind::Piece,
        position: 0,
        duration_display: "5m 32s".to_string(),
        status: EntryStatus::Completed,
        notes: None,
        score: Some(4),
        intention: None,
        rep_target: None,
        rep_count: None,
        rep_target_reached: None,
        rep_history: None,
        planned_duration_secs: None,
        planned_duration_display: None,
        achieved_tempo: None,
    };
    let entry_display = SetlistEntryView {
        id: "entry-2".to_string(),
        item_id: "item-2".to_string(),
        item_title: "Hanon No. 1".to_string(),
        item_type: ItemKind::Exercise,
        position: 1,
        duration_display: "3m 10s".to_string(),
        status: EntryStatus::NotAttempted,
        notes: None,
        score: None,
        intention: None,
        rep_target: None,
        rep_count: None,
        rep_target_reached: None,
        rep_history: None,
        planned_duration_secs: None,
        planned_duration_display: None,
        achieved_tempo: None,
    };
    let entry_drag = SetlistEntryView {
        id: "entry-3".to_string(),
        item_id: "item-3".to_string(),
        item_title: "Chromatic Scales".to_string(),
        item_type: ItemKind::Exercise,
        position: 2,
        duration_display: "2m 05s".to_string(),
        status: EntryStatus::NotAttempted,
        notes: None,
        score: None,
        intention: None,
        rep_target: None,
        rep_count: None,
        rep_target_reached: None,
        rep_history: None,
        planned_duration_secs: None,
        planned_duration_display: None,
        achieved_tempo: None,
    };

    view! {
        <div class="space-y-section">
            <PageHeading text="Design System Catalogue" />
            <p class="text-sm text-muted -mt-4 mb-8">
                "Dev-only reference of all UI components and design tokens. "
                "See " <code class="text-xs bg-surface-input rounded px-1 py-0.5">"specs/design-system.md"</code> " for full documentation."
            </p>

            // ── Table of Contents ─────────────────────────────────────
            <nav class="glass-card p-4 sm:p-6" aria-label="Catalogue navigation">
                <h3 class="text-sm font-semibold text-primary mb-3">"Contents"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-x-6 gap-y-1">
                    <div>
                        <p class="text-xs font-medium text-muted uppercase mb-1">"Tokens"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#colours" class="text-accent-text hover:text-primary">"Colours"</a></li>
                            <li><a href="#typography" class="text-accent-text hover:text-primary">"Typography"</a></li>
                            <li><a href="#spacing" class="text-accent-text hover:text-primary">"Spacing"</a></li>
                            <li><a href="#badges-tokens" class="text-accent-text hover:text-primary">"Badge Colours"</a></li>
                            <li><a href="#radii" class="text-accent-text hover:text-primary">"Radii"</a></li>
                            <li><a href="#utilities" class="text-accent-text hover:text-primary">"Composite Utilities"</a></li>
                        </ul>
                    </div>
                    <div>
                        <p class="text-xs font-medium text-muted uppercase mb-1">"Components"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#glass-card" class="text-accent-text hover:text-primary">"Glass Card"</a></li>
                            <li><a href="#stat-card" class="text-accent-text hover:text-primary">"Stat Card"</a></li>
                            <li><a href="#library-item-card" class="text-accent-text hover:text-primary">"Library Item Card"</a></li>
                            <li><a href="#buttons" class="text-accent-text hover:text-primary">"Buttons"</a></li>
                            <li><a href="#type-badge" class="text-accent-text hover:text-primary">"Type Badge"</a></li>
                            <li><a href="#type-tabs" class="text-accent-text hover:text-primary">"Type Tabs"</a></li>
                            <li><a href="#toast" class="text-accent-text hover:text-primary">"Toast"</a></li>
                            <li><a href="#error-banner" class="text-accent-text hover:text-primary">"Error Banner"</a></li>
                            <li><a href="#progress" class="text-accent-text hover:text-primary">"Progress Bar"</a></li>
                        </ul>
                    </div>
                    <div>
                        <p class="text-xs font-medium text-muted uppercase mb-1">"Forms & Data"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#text-field" class="text-accent-text hover:text-primary">"Text Field"</a></li>
                            <li><a href="#text-area" class="text-accent-text hover:text-primary">"Text Area"</a></li>
                            <li><a href="#form-states" class="text-accent-text hover:text-primary">"Form Validation States"</a></li>
                            <li><a href="#autocomplete" class="text-accent-text hover:text-primary">"Autocomplete"</a></li>
                            <li><a href="#tag-input" class="text-accent-text hover:text-primary">"Tag Input"</a></li>
                            <li><a href="#field-label" class="text-accent-text hover:text-primary">"Field Label"</a></li>
                            <li><a href="#form-field-error" class="text-accent-text hover:text-primary">"Form Field Error"</a></li>
                            <li><a href="#line-chart" class="text-accent-text hover:text-primary">"Line Chart"</a></li>
                            <li><a href="#tempo-chart" class="text-accent-text hover:text-primary">"Tempo Progress Chart"</a></li>
                        </ul>
                    </div>
                    <div>
                        <p class="text-xs font-medium text-muted uppercase mb-1">"Practice & Shell"</p>
                        <ul class="space-y-0.5 text-sm">
                            <li><a href="#navigation" class="text-accent-text hover:text-primary">"Navigation"</a></li>
                            <li><a href="#setlist-entry" class="text-accent-text hover:text-primary">"Setlist Entry"</a></li>
                            <li><a href="#drag-drop" class="text-accent-text hover:text-primary">"Drag & Drop"</a></li>
                            <li><a href="#routine-save" class="text-accent-text hover:text-primary">"Routine Save Form"</a></li>
                            <li><a href="#loading" class="text-accent-text hover:text-primary">"Loading States"</a></li>
                            <li><a href="#skeletons" class="text-accent-text hover:text-primary">"Skeletons"</a></li>
                            <li><a href="#week-strip" class="text-accent-text hover:text-primary">"Week Strip"</a></li>
                            <li><a href="#shell" class="text-accent-text hover:text-primary">"Shell Components"</a></li>
                            <li><a href="#accessibility" class="text-accent-text hover:text-primary">"Accessibility"</a></li>
                        </ul>
                    </div>
                </div>
            </nav>

            // ══════════════════════════════════════════════════════════
            // TOKENS
            // ══════════════════════════════════════════════════════════

            // ── Colour Palette ────────────────────────────────────────
            <section id="colours">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Colour Palette"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    // Surfaces
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-muted uppercase">"Surfaces"</p>
                        <div class="h-12 rounded-lg bg-surface-primary border border-border-card"></div>
                        <p class="text-xs text-faint">"surface-primary"</p>
                        <div class="h-12 rounded-lg bg-surface-secondary border border-border-card"></div>
                        <p class="text-xs text-faint">"surface-secondary"</p>
                        <div class="h-12 rounded-lg bg-surface-chrome border border-border-card"></div>
                        <p class="text-xs text-faint">"surface-chrome"</p>
                        <div class="h-12 rounded-lg bg-surface-fallback border border-border-card"></div>
                        <p class="text-xs text-faint">"surface-fallback"</p>
                        <div class="h-12 rounded-lg bg-surface-hover border border-border-card"></div>
                        <p class="text-xs text-faint">"surface-hover"</p>
                        <div class="h-12 rounded-lg bg-surface-input border border-border-card"></div>
                        <p class="text-xs text-faint">"surface-input"</p>
                    </div>

                    // Accent
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-muted uppercase">"Accent"</p>
                        <div class="h-12 rounded-lg bg-accent"></div>
                        <p class="text-xs text-faint">"accent"</p>
                        <div class="h-12 rounded-lg bg-accent-hover"></div>
                        <p class="text-xs text-faint">"accent-hover"</p>
                        <div class="h-12 rounded-lg bg-accent-text"></div>
                        <p class="text-xs text-faint">"accent-text"</p>
                        <div class="h-12 rounded-lg bg-accent-focus"></div>
                        <p class="text-xs text-faint">"accent-focus"</p>

                        <p class="text-xs font-medium text-muted uppercase pt-2">"Warm Accent"</p>
                        <div class="h-12 rounded-lg bg-warm-accent"></div>
                        <p class="text-xs text-faint">"warm-accent"</p>
                        <div class="h-12 rounded-lg bg-warm-accent-hover"></div>
                        <p class="text-xs text-faint">"warm-accent-hover"</p>
                        <div class="h-12 rounded-lg bg-warm-accent-text"></div>
                        <p class="text-xs text-faint">"warm-accent-text"</p>
                        <div class="h-12 rounded-lg bg-warm-accent-surface border border-border-card"></div>
                        <p class="text-xs text-faint">"warm-accent-surface"</p>
                    </div>

                    // Semantic colours
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-muted uppercase">"Success"</p>
                        <div class="h-12 rounded-lg bg-success"></div>
                        <p class="text-xs text-faint">"success"</p>
                        <div class="h-12 rounded-lg bg-success-hover"></div>
                        <p class="text-xs text-faint">"success-hover"</p>
                        <div class="h-12 rounded-lg bg-success-text"></div>
                        <p class="text-xs text-faint">"success-text"</p>
                        <div class="h-12 rounded-lg bg-success-surface border border-border-card"></div>
                        <p class="text-xs text-faint">"success-surface"</p>

                        <p class="text-xs font-medium text-muted uppercase pt-2">"Warning"</p>
                        <div class="h-12 rounded-lg bg-warning"></div>
                        <p class="text-xs text-faint">"warning"</p>
                        <div class="h-12 rounded-lg bg-warning-text"></div>
                        <p class="text-xs text-faint">"warning-text"</p>
                        <div class="h-12 rounded-lg bg-warning-surface border border-border-card"></div>
                        <p class="text-xs text-faint">"warning-surface"</p>

                        <p class="text-xs font-medium text-muted uppercase pt-2">"Info"</p>
                        <div class="h-12 rounded-lg bg-info"></div>
                        <p class="text-xs text-faint">"info"</p>
                        <div class="h-12 rounded-lg bg-info-text"></div>
                        <p class="text-xs text-faint">"info-text"</p>
                        <div class="h-12 rounded-lg bg-info-surface border border-border-card"></div>
                        <p class="text-xs text-faint">"info-surface"</p>
                    </div>

                    // Danger + Borders + Progress
                    <div class="space-y-2">
                        <p class="text-xs font-medium text-muted uppercase">"Danger"</p>
                        <div class="h-12 rounded-lg bg-danger"></div>
                        <p class="text-xs text-faint">"danger (coral)"</p>
                        <div class="h-12 rounded-lg bg-danger-hover"></div>
                        <p class="text-xs text-faint">"danger-hover"</p>
                        <div class="h-12 rounded-lg bg-danger-text"></div>
                        <p class="text-xs text-faint">"danger-text"</p>
                        <div class="h-12 rounded-lg bg-danger-surface border border-border-card"></div>
                        <p class="text-xs text-faint">"danger-surface"</p>

                        <p class="text-xs font-medium text-muted uppercase pt-2">"Borders"</p>
                        <div class="h-12 rounded-lg border-2 border-border-default"></div>
                        <p class="text-xs text-faint">"border-default"</p>
                        <div class="h-12 rounded-lg border-2 border-border-card"></div>
                        <p class="text-xs text-faint">"border-card"</p>
                        <div class="h-12 rounded-lg border-2 border-border-input"></div>
                        <p class="text-xs text-faint">"border-input"</p>

                        <p class="text-xs font-medium text-muted uppercase pt-2">"Progress"</p>
                        <div class="h-12 rounded-lg bg-progress-track border border-border-card"></div>
                        <p class="text-xs text-faint">"progress-track"</p>
                        <div class="h-12 rounded-lg bg-progress-fill"></div>
                        <p class="text-xs text-faint">"progress-fill"</p>
                        <div class="h-12 rounded-lg bg-progress-complete"></div>
                        <p class="text-xs text-faint">"progress-complete"</p>
                    </div>
                </div>

                // Text colour samples
                <div class="mt-6 space-y-1">
                    <p class="text-xs font-medium text-muted uppercase mb-2">"Text Colours"</p>
                    <p class="text-text-primary">"text-primary — Headings and titles"</p>
                    <p class="text-text-secondary">"text-secondary — Body text"</p>
                    <p class="text-text-label">"text-label — Form labels"</p>
                    <p class="text-text-muted">"text-muted — Hints and metadata"</p>
                    <p class="text-text-faint">"text-faint — Timestamps, tertiary info"</p>
                    <p class="text-accent-text">"accent-text — Active navigation"</p>
                    <p class="text-warm-accent-text">"warm-accent-text — Achievements, milestones"</p>
                    <p class="text-success-text">"success-text — Positive feedback"</p>
                    <p class="text-warning-text">"warning-text — Caution alerts"</p>
                    <p class="text-info-text">"info-text — Informational messages"</p>
                    <p class="text-danger-text">"danger-text — Error text"</p>
                </div>
            </section>

            // ── Typography ────────────────────────────────────────────
            <section id="typography">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Typography"</h3>
                <Card>
                    <div class="space-y-3">
                        <h1 class="text-3xl font-bold text-primary font-heading">"Heading 1 — 3xl bold serif"</h1>
                        <h2 class="text-2xl font-bold text-primary font-heading">"Heading 2 — 2xl bold serif"</h2>
                        <h3 class="text-lg font-semibold text-primary">"Heading 3 — lg semibold sans"</h3>
                        <p class="text-base text-secondary">"Body text — base / gray-300"</p>
                        <p class="text-sm text-muted">"Small text — sm / gray-400"</p>
                        <p class="text-xs text-faint">"Extra small — xs / gray-500"</p>
                        <p class="text-xs font-medium text-muted uppercase tracking-wider">"Label style — xs medium uppercase tracking-wider"</p>
                        <div class="pt-2 border-t border-border-default">
                            <p class="text-xs font-medium text-muted uppercase tracking-wider mb-1">"Time display — mono tabular (session timer, metronome)"</p>
                            <p class="text-4xl sm:text-6xl font-mono text-primary">"12:34"</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Spacing Tokens ────────────────────────────────────────
            <section id="spacing">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Spacing Tokens"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Semantic spacing scale mapped to the 4px base grid. Use these for component padding and gaps."</p>
                    <div class="space-y-3">
                        <div class="flex items-center gap-3">
                            <div class="w-[0.75rem] h-4 bg-accent-focus rounded-sm shrink-0"></div>
                            <code class="text-xs text-muted w-40">"card-compact (12px)"</code>
                            <span class="text-xs text-faint">"Stat cards, small elements"</span>
                        </div>
                        <div class="flex items-center gap-3">
                            <div class="w-[1rem] h-4 bg-accent-focus rounded-sm shrink-0"></div>
                            <code class="text-xs text-muted w-40">"card (16px)"</code>
                            <span class="text-xs text-faint">"Standard card padding"</span>
                        </div>
                        <div class="flex items-center gap-3">
                            <div class="w-[1.5rem] h-4 bg-accent-focus rounded-sm shrink-0"></div>
                            <code class="text-xs text-muted w-40">"card-comfortable (24px)"</code>
                            <span class="text-xs text-faint">"sm+ breakpoint card padding"</span>
                        </div>
                        <div class="flex items-center gap-3">
                            <div class="w-[3rem] h-4 bg-accent-focus rounded-sm shrink-0"></div>
                            <code class="text-xs text-muted w-40">"section (48px)"</code>
                            <span class="text-xs text-faint">"Between catalogue sections"</span>
                        </div>
                        <div class="flex items-center gap-3">
                            <div class="w-[4rem] h-4 bg-accent-focus rounded-sm shrink-0"></div>
                            <code class="text-xs text-muted w-40">"section-lg (64px)"</code>
                            <span class="text-xs text-faint">"Major section breaks"</span>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Badge Colours ─────────────────────────────────────────
            <section id="badges-tokens">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Badge Colours"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Rebalanced for equal visual weight. Piece uses accent-derived tones, exercise uses warm-accent-derived tones."</p>
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg bg-badge-piece-bg border border-border-card"></div>
                            <p class="text-xs text-faint">"badge-piece-bg"</p>
                        </div>
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg flex items-center justify-center">
                                <span class="text-badge-piece-text font-medium">"Piece Text"</span>
                            </div>
                            <p class="text-xs text-faint">"badge-piece-text"</p>
                        </div>
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg bg-badge-exercise-bg border border-border-card"></div>
                            <p class="text-xs text-faint">"badge-exercise-bg"</p>
                        </div>
                        <div class="space-y-2 text-center">
                            <div class="h-12 rounded-lg flex items-center justify-center">
                                <span class="text-badge-exercise-text font-medium">"Exercise Text"</span>
                            </div>
                            <p class="text-xs text-faint">"badge-exercise-text"</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Radius Tokens ─────────────────────────────────────────
            <section id="radii">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Radius Tokens"</h3>
                <Card>
                    <div class="flex flex-wrap gap-6 items-end">
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-card"></div>
                            <p class="text-xs text-faint">"radius-card"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-button"></div>
                            <p class="text-xs text-faint">"radius-button"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-input"></div>
                            <p class="text-xs text-faint">"radius-input"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-badge"></div>
                            <p class="text-xs text-faint">"radius-badge"</p>
                        </div>
                        <div class="text-center space-y-2">
                            <div class="w-20 h-20 bg-surface-hover border border-border-card rounded-pill"></div>
                            <p class="text-xs text-faint">"radius-pill"</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Composite Utilities ───────────────────────────────────
            <section id="utilities">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Composite Utilities"</h3>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div class="space-y-2">
                        <div class="glass-card p-4">
                            <p class="text-sm text-secondary">"Content inside glass-card"</p>
                        </div>
                        <p class="text-xs text-faint text-center">"glass-card"</p>
                        <p class="text-xs text-faint text-center">"Glassmorphism + fallback + border + shadow"</p>
                    </div>
                    <div class="space-y-2">
                        <div class="glass-card-active p-4">
                            <p class="text-sm text-secondary">"Active card — currently practicing"</p>
                        </div>
                        <p class="text-xs text-faint text-center">"glass-card-active"</p>
                        <p class="text-xs text-faint text-center">"Accent border + glow for active practice item"</p>
                    </div>
                    <div class="space-y-2">
                        <div class="glass-chrome border border-border-default p-4">
                            <p class="text-sm text-secondary">"Content inside glass-chrome"</p>
                        </div>
                        <p class="text-xs text-faint text-center">"glass-chrome"</p>
                        <p class="text-xs text-faint text-center">"Neutral chrome for nav bars"</p>
                    </div>
                    <div class="space-y-2">
                        <input
                            type="text"
                            class="input-base"
                            placeholder="Content inside input-base"
                            readonly
                        />
                        <p class="text-xs text-faint text-center">"input-base"</p>
                        <p class="text-xs text-faint text-center">"Border + bg + focus ring + sizing"</p>
                    </div>
                </div>
            </section>

            // ══════════════════════════════════════════════════════════
            // 2026 REFRESH — New primitives (signature design language)
            // ══════════════════════════════════════════════════════════

            // ── Section Label ─────────────────────────────────────────
            <section id="section-label">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Section Label"</h3>
                <p class="text-xs text-faint mb-3">"Uppercase 11px label that anchors grouped content. Place above any list, card group, or chart row."</p>
                <Card>
                    <SectionLabel text="Recent Activity" />
                    <p class="text-sm text-secondary mt-2">"…content lives below the label."</p>
                </Card>
            </section>

            // ── Inline Type Indicator ─────────────────────────────────
            <section id="inline-type-indicator">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Inline Type Indicator"</h3>
                <p class="text-xs text-faint mb-3">"Dot + label combo for in-row use. The boxed TypeBadge above is for surfaces where type is the primary content (form mode toggles); use this inline variant inside list rows where the boxed badge would compete with the row's accent bar."</p>
                <Card>
                    <div class="flex flex-col gap-3">
                        <InlineTypeIndicator item_type=ItemType::Piece />
                        <InlineTypeIndicator item_type=ItemType::Exercise />
                    </div>
                </Card>
            </section>

            // ── Accent Row ────────────────────────────────────────────
            <section id="accent-row">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Accent Row"</h3>
                <p class="text-xs text-faint mb-3">"List-row primitive with optional 4px gradient bar on the left. Use sparingly — bars earn their keep on mixed-type lists (library, setlists). For uniform lists, pass `bar=AccentBar::None` so they don't flatten into noise."</p>
                <div class="space-y-2">
                    <AccentRow bar=AccentBar::Gold>
                        <div class="flex flex-col flex-1 gap-0.5">
                            <span class="text-sm font-semibold text-primary">"Clair de Lune"</span>
                            <span class="text-xs text-muted">"Debussy"</span>
                        </div>
                        <InlineTypeIndicator item_type=ItemType::Piece />
                    </AccentRow>
                    <AccentRow bar=AccentBar::Blue>
                        <div class="flex flex-col flex-1 gap-0.5">
                            <span class="text-sm font-semibold text-primary">"Hanon Exercise No.1"</span>
                            <span class="text-xs text-muted">"Hanon"</span>
                        </div>
                        <InlineTypeIndicator item_type=ItemType::Exercise />
                    </AccentRow>
                    <AccentRow bar=AccentBar::None>
                        <div class="flex flex-col flex-1 gap-0.5">
                            <span class="text-sm font-semibold text-primary">"No-bar variant"</span>
                            <span class="text-xs text-muted">"For uniform-type lists"</span>
                        </div>
                    </AccentRow>
                </div>
            </section>

            // ── Detail Group ──────────────────────────────────────────
            <section id="detail-group">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Detail Group"</h3>
                <p class="text-xs text-faint mb-3">"Card containing a section label + grouped rows + the inset 4px bar. The signature container of the 2026 refresh — used for DETAILS / RECENT SESSIONS / NOTES on the Piece Detail page. DetailRow renders standard label/value pairs."</p>
                <div class="space-y-3">
                    <DetailGroup label="Details" bar=AccentBar::Gold>
                        <DetailRow label="Difficulty">"Intermediate"</DetailRow>
                        <DetailRow label="Key">"D♭ Major"</DetailRow>
                        <DetailRow label="Time Signature">"9/8"</DetailRow>
                        <DetailRow label="Added">"12 Apr 2026"</DetailRow>
                    </DetailGroup>
                    <DetailGroup label="Notes" bar=AccentBar::Blue>
                        <p class="text-sm text-secondary leading-relaxed">"Focus on the arpeggiated left hand in the opening section. Keep dynamics very soft, pp throughout the first page."</p>
                    </DetailGroup>
                </div>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Containers
            // ══════════════════════════════════════════════════════════

            // ── Glass Card ────────────────────────────────────────────
            <section id="glass-card">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Glass Card"</h3>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <Card>
                        <p class="text-sm text-secondary">"Default glass-card with standard padding."</p>
                    </Card>
                    <Card>
                        <h3 class="text-lg font-semibold text-primary mb-2">"With heading"</h3>
                        <p class="text-sm text-muted">"Card content with heading and body text."</p>
                    </Card>
                    <Card>
                        <h3 class="text-lg font-semibold text-primary mb-3">"With divider"</h3>
                        <div class="border-b border-border-default mb-3"></div>
                        <p class="text-sm text-secondary">"Content below a horizontal divider."</p>
                        <div class="border-b border-border-default my-3"></div>
                        <p class="text-xs text-faint">"Footer-style content in the card."</p>
                    </Card>
                    <div class="glass-card-active p-4 sm:p-6">
                        <h3 class="text-lg font-semibold text-primary mb-2">"Active card"</h3>
                        <p class="text-sm text-muted">"Currently practicing this item. Accent border + subtle glow."</p>
                    </div>
                </div>
            </section>

            // ── Stat Cards ────────────────────────────────────────────
            <section id="stat-card">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Stat Card"</h3>
                <p class="text-xs font-medium text-muted uppercase mb-2">"Classic — glass-card chrome"</p>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                    <StatCard title="Current Streak" value="7 days".to_string() subtitle="Best: 14 days" />
                    <StatCard title="This Week" value="3h 45m".to_string() />
                    <StatCard title="Sessions" value="12".to_string() subtitle="This month" />
                    <StatCard title="Avg Score" value="3.8".to_string() subtitle="Out of 5" />
                </div>
                <p class="text-xs font-medium text-muted uppercase mb-2">"2026 refresh \u{2014} inset accent bar + tone"</p>
                <p class="text-xs text-faint mb-3">"Whisper-soft surface, gradient bar inset on the left, value text in the matching tone signals the stat's category at a glance."</p>
                <div class="grid grid-cols-3 gap-3">
                    <StatCard title="Day Streak" value="12".to_string() bar=AccentBar::Gold tone=StatTone::Accent />
                    <StatCard title="Hrs This Week" value="8.5".to_string() bar=AccentBar::Blue tone=StatTone::WarmAccent />
                    <StatCard title="Pieces Learned" value="23".to_string() bar=AccentBar::Gold />
                </div>
            </section>

            // ── Library Item Cards ────────────────────────────────────
            <section id="library-item-card">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Library Item Card"</h3>
                <p class="text-xs text-faint mb-3">"Content grouped semantically: [title + composer] — [key/tempo] — [tags]."</p>
                <ul class="space-y-3">
                    <p class="text-xs font-medium text-muted uppercase">"Full metadata (piece)"</p>
                    <LibraryItemCard item=sample_piece />

                    <p class="text-xs font-medium text-muted uppercase mt-6">"Full metadata (exercise)"</p>
                    <LibraryItemCard item=sample_exercise />

                    <p class="text-xs font-medium text-muted uppercase mt-6">"Minimal (no subtitle, tags, key, or tempo)"</p>
                    <LibraryItemCard item=sample_minimal />

                    <p class="text-xs font-medium text-muted uppercase mt-6">"Long title + many tags"</p>
                    <LibraryItemCard item=sample_long_title />
                </ul>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Display
            // ══════════════════════════════════════════════════════════

            // ── Buttons ───────────────────────────────────────────────
            <section id="buttons">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Buttons"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Normal"</p>
                            <div class="flex flex-wrap gap-3">
                                <Button variant=ButtonVariant::Primary>"Primary"</Button>
                                <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                                <Button variant=ButtonVariant::Success>"Success"</Button>
                                <Button variant=ButtonVariant::Danger>"Danger"</Button>
                                <Button variant=ButtonVariant::DangerOutline>"Danger Outline"</Button>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Disabled"</p>
                            <div class="flex flex-wrap gap-3">
                                <Button variant=ButtonVariant::Primary disabled=Signal::derive(|| true)>"Primary"</Button>
                                <Button variant=ButtonVariant::Secondary disabled=Signal::derive(|| true)>"Secondary"</Button>
                                <Button variant=ButtonVariant::Success disabled=Signal::derive(|| true)>"Success"</Button>
                                <Button variant=ButtonVariant::Danger disabled=Signal::derive(|| true)>"Danger"</Button>
                                <Button variant=ButtonVariant::DangerOutline disabled=Signal::derive(|| true)>"Danger Outline"</Button>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Loading"</p>
                            <div class="flex flex-wrap gap-3">
                                <Button variant=ButtonVariant::Primary loading=Signal::derive(|| true)>"Saving..."</Button>
                                <Button variant=ButtonVariant::Success loading=Signal::derive(|| true)>"Completing..."</Button>
                                <Button variant=ButtonVariant::Secondary loading=Signal::derive(|| true)>"Loading..."</Button>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Hero (2026 refresh \u{2014} full-width CTA)"</p>
                            <p class="text-xs text-faint mb-2">"Larger 48px / text-base / weight-600 sizing for the primary action on a screen (Add to Library, Start Practice). Default size stays Small for inline use."</p>
                            <div class="space-y-3">
                                <Button variant=ButtonVariant::Primary size=ButtonSize::Hero attr:class="w-full">"Add to Library"</Button>
                                <Button variant=ButtonVariant::Primary size=ButtonSize::Hero attr:class="w-full">"Start Practice"</Button>
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Circular Button ───────────────────────────────────────
            <section id="circular-button">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Circular Button"</h3>
                <p class="text-xs text-faint mb-3">"Round icon-only action used for player controls in the 2026 refresh. 56px primary anchors the row; 44px secondary sits beside it. Light haptic on press, scale feedback on :active."</p>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Player controls (medium primary + small secondary)"</p>
                            <div class="flex items-center justify-center gap-6">
                                <CircularButton icon=IconName::Play aria_label="Play" />
                                <CircularButton icon=IconName::RotateCcw aria_label="Reset" size=CircularButtonSize::Small variant=CircularButtonVariant::Secondary />
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"All variants"</p>
                            <div class="flex items-center justify-center gap-4">
                                <CircularButton icon=IconName::Play aria_label="Primary medium" />
                                <CircularButton icon=IconName::Pause aria_label="Primary small" size=CircularButtonSize::Small />
                                <CircularButton icon=IconName::RotateCcw aria_label="Secondary medium" variant=CircularButtonVariant::Secondary />
                                <CircularButton icon=IconName::RotateCcw aria_label="Secondary small" size=CircularButtonSize::Small variant=CircularButtonVariant::Secondary />
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Disabled"</p>
                            <div class="flex items-center justify-center gap-4">
                                <CircularButton icon=IconName::Play aria_label="Disabled primary" disabled=Signal::derive(|| true) />
                                <CircularButton icon=IconName::RotateCcw aria_label="Disabled secondary" variant=CircularButtonVariant::Secondary disabled=Signal::derive(|| true) />
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Type Badge ────────────────────────────────────────────
            <section id="type-badge">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Type Badge"</h3>
                <Card>
                    <p class="text-xs text-faint mb-3">"Equal visual weight — accent-derived for Piece, warm-accent-derived for Exercise."</p>
                    <div class="flex flex-wrap gap-3">
                        <TypeBadge item_type=ItemKind::Piece />
                        <TypeBadge item_type=ItemKind::Exercise />
                    </div>
                </Card>
            </section>

            // ── Type Tabs ─────────────────────────────────────────────
            <section id="type-tabs">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Type Tabs"</h3>
                <Card>
                    <div class="space-y-3">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Interactive"</p>
                            <TypeTabs
                                active=Signal::derive(move || type_tab_active.get())
                                on_change=Callback::new(move |t| type_tab_active.set(t))
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Display-only (Piece)"</p>
                            <TypeTabs active=Signal::derive(|| ItemType::Piece) />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Display-only (Exercise)"</p>
                            <TypeTabs active=Signal::derive(|| ItemType::Exercise) />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Toast Notifications ──────────────────────────────────
            <section id="toast">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Toast Notifications"</h3>
                <div class="space-y-3">
                    <Toast variant=ToastVariant::Info>"Session auto-saved"</Toast>
                    <Toast variant=ToastVariant::Success>"5 correct in a row!"</Toast>
                    <Toast variant=ToastVariant::Warning>"Session timer paused — are you still there?"</Toast>
                    <Toast variant=ToastVariant::Danger>"Failed to save session. Please check your connection."</Toast>
                </div>
            </section>

            // ── Error Banner ──────────────────────────────────────────
            <section id="error-banner">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Error Banner"</h3>
                <p class="text-xs text-faint mb-3">"Static preview — the real component reads from ViewModel context. Uses softened coral danger tokens."</p>
                <div class="mb-6 rounded-lg bg-danger-surface border border-danger-text/20 p-4" role="alert">
                    <div class="flex items-start justify-between gap-3">
                        <p class="text-sm text-danger-text">
                            <span class="font-medium">"Error: "</span>"Failed to save session. Please check your connection and try again."
                        </p>
                        <button class="shrink-0 text-danger-text hover:text-danger-hover text-xs font-medium">
                            "Dismiss"
                        </button>
                    </div>
                </div>
            </section>

            // ── Progress Bar ──────────────────────────────────────────
            <section id="progress">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Progress Bar"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Tokenised progress colours: track, fill (in-progress), and complete."</p>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"In progress (60%)"</p>
                            <div class="h-2 rounded-full bg-progress-track">
                                <div class="h-full rounded-full bg-progress-fill" style="width: 60%"></div>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Complete (100%)"</p>
                            <div class="h-2 rounded-full bg-progress-track">
                                <div class="h-full rounded-full bg-progress-complete" style="width: 100%"></div>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Empty (0%)"</p>
                            <div class="h-2 rounded-full bg-progress-track">
                                <div class="h-full rounded-full bg-progress-fill" style="width: 0%"></div>
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Progress Ring ────────────────────────────────────────
            <section id="progress-ring">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Progress Ring"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"SVG circular progress indicator for timed practice items. Ring fills clockwise; digital timer centred inside. Changes to completion colour when elapsed exceeds planned."</p>
                    <div class="space-y-6">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"In progress (~40%)"</p>
                            <ProgressRing
                                elapsed_secs=RwSignal::new(120u32)
                                planned_duration_secs=300u32
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Complete (elapsed exceeds planned)"</p>
                            <ProgressRing
                                elapsed_secs=RwSignal::new(330u32)
                                planned_duration_secs=300u32
                            />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Transition Prompt ────────────────────────────────────
            <section id="transition-prompt">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Transition Prompt"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Non-blocking prompt shown when an item\u{2019}s planned duration elapses. Shows next item or practice completion message."</p>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"With next item"</p>
                            <TransitionPrompt next_item_title=Some("Clair de Lune".to_string()) />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Last item (practice complete)"</p>
                            <TransitionPrompt next_item_title=None />
                        </div>
                    </div>
                </Card>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — iOS Gesture Primitives
            // ══════════════════════════════════════════════════════════

            // ── BottomSheet ───────────────────────────────────────────
            <section id="bottom-sheet">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Bottom Sheet"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"iOS-style modal sheet (UISheetPresentationController feel). Slides up from the bottom over a dimmed backdrop, ~92vh tall. Drag handle, swipe-down to dismiss with elastic resistance, light haptic on cross-threshold, Cancel button + backdrop tap + Escape all dismiss. Renders into <body> via Portal so positioning is viewport-anchored. Used in production for the library Add Item and Edit forms."</p>
                    <BottomSheetDemo />
                </Card>
            </section>

            // ── EmptyState ────────────────────────────────────────────
            <section id="empty-state">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Empty State"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Centred icon + title + body + optional CTA, used wherever a list/section can legitimately have no rows. iOS scales the glyph up to ~80pt (SF-Symbol size) and tightens typography; web stays at the smaller default."</p>
                    <div class="space-y-6">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"With CTA — Library"</p>
                            <EmptyState
                                icon=IconName::Music
                                title="No items in your library yet"
                                body="Add a piece or exercise to get started."
                            >
                                <button type="button" class="cta-link">"Add Item"</button>
                            </EmptyState>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"With CTA — Routines"</p>
                            <EmptyState
                                icon=IconName::ListChecks
                                title="No saved routines yet"
                                body="Save a setlist as a routine when building a session."
                            >
                                <button type="button" class="cta-link">"New Session"</button>
                            </EmptyState>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"With CTA — Sessions"</p>
                            <EmptyState
                                icon=IconName::CalendarDays
                                title="No sessions on this day"
                                body="Start a practice session to see it here."
                            >
                                <button type="button" class="cta-link">"New Session"</button>
                            </EmptyState>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"No CTA — Analytics"</p>
                            <EmptyState
                                icon=IconName::BarChart
                                title="No session data yet"
                                body="Complete some sessions to see your analytics."
                            />
                        </div>
                    </div>
                </Card>
            </section>

            // ── SwipeActions ──────────────────────────────────────────
            <section id="swipe-actions">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Swipe Actions"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"iOS-style swipe-to-reveal trailing action (UISwipeActionsConfiguration feel). Touch-only; gesture is hidden on non-iOS. Direction discrimination ensures vertical scrolls fall through. Half-open snap reveals the action button; full-swipe past 200px commits without a button tap (light haptic on threshold). Used in production for library and routine row Delete."</p>
                    <p class="text-xs text-faint mb-4">"On iOS device: swipe the row left."</p>
                    <SwipeActionsDemo />
                </Card>
            </section>

            // ── ContextMenu ───────────────────────────────────────────
            <section id="context-menu">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Context Menu"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"iOS-style long-press context menu (UIContextMenuInteraction feel). ~500ms hold without significant movement triggers; medium haptic on activation. Menu floats anchored to the touch point, clamped to viewport edges, with backdrop blur + dim. Tap outside / Escape / select an action dismisses. Used in production for library and routine row Edit / Delete shortcuts."</p>
                    <p class="text-xs text-faint mb-4">"On iOS device: long-press the row below."</p>
                    <ContextMenuDemo />
                </Card>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Forms
            // ══════════════════════════════════════════════════════════

            // ── Form Inputs ───────────────────────────────────────────
            <section id="text-field">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Text Field"</h3>
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
                            value=sample_text_hint
                            field_name="subtitle"
                            errors=empty_errors
                            placeholder="e.g. Claude Debussy"
                            hint="The composer or source of the piece"
                        />
                        <TextField
                            id="demo-text-required"
                            label="Required field"
                            value=sample_text_required
                            field_name="title_req"
                            errors=empty_errors
                            required=true
                            placeholder="Required..."
                        />
                        <TextField
                            id="demo-text-error"
                            label="With validation error"
                            value=sample_text_error
                            field_name="title"
                            errors=sample_errors
                            required=true
                        />
                    </div>
                </Card>
            </section>

            <section id="text-area">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Text Area"</h3>
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

            // ── Form Validation States ───────────────────────────────
            <section id="form-states">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Form Validation States"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Static preview of all input validation states. Connected via aria-describedby in real usage."</p>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-text-label mb-1">"Default"</label>
                            <input type="text" class="input-base" value="Normal input" readonly />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-text-label mb-1">"Focused"</label>
                            <input type="text" class="input-base ring-1 ring-accent-focus border-accent-focus" value="Focused input" readonly />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-text-label mb-1">"Error"</label>
                            <input type="text" class="input-base input-error" value="" readonly />
                            <p class="text-xs text-danger-text mt-1">"Title is required"</p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-text-label mb-1">"Success"</label>
                            <input type="text" class="input-base input-success" value="Clair de Lune" readonly />
                            <p class="text-xs text-success-text mt-1">"Looks good!"</p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-text-label mb-1">"Disabled"</label>
                            <input type="text" class="input-base opacity-50 cursor-not-allowed" value="Disabled field" readonly disabled />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Autocomplete ──────────────────────────────────────────
            <section id="autocomplete">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Autocomplete"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Standalone autocomplete"</p>
                            <p class="text-xs text-faint mb-2">"Type 2+ characters to see suggestions (try \"ba\" or \"ch\")"</p>
                            <Autocomplete
                                id="demo-autocomplete"
                                suggestions=composers
                                value=autocomplete_value
                                on_select=Callback::new(move |s: String| autocomplete_value.set(s))
                                placeholder="Search composers..."
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"AutocompleteTextField (with label + error)"</p>
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
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Tag Input"</h3>
                <Card>
                    <p class="text-xs text-faint mb-3">"Pre-populated with sample tags. Type to add more, click × to remove."</p>
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
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Field Label"</h3>
                <Card>
                    <dl class="space-y-2">
                        <FieldLabel text="Key Signature" />
                        <dd class="text-primary">"Db Major"</dd>
                        <FieldLabel text="Tempo" />
                        <dd class="text-primary">"66 bpm"</dd>
                        <FieldLabel text="Category" />
                        <dd class="text-primary">"Romantic"</dd>
                    </dl>
                </Card>
            </section>

            // ── Form Field Error (standalone) ─────────────────────────
            <section id="form-field-error">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Form Field Error"</h3>
                <Card>
                    <FormFieldError field="title" errors=sample_errors />
                </Card>
            </section>

            // ── Navigation ────────────────────────────────────────────
            <section id="navigation">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Navigation"</h3>
                <Card>
                    <div class="space-y-3">
                        <BackLink label="Back to Library" href="/".to_string() />
                        <PageHeading text="Sample Page Heading" />
                    </div>
                </Card>
            </section>

            // ── Line Chart ────────────────────────────────────────────
            <section id="line-chart">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Line Chart"</h3>
                <Card>
                    <p class="text-xs text-faint mb-3">"Uses tokenised chart colours: chart-line, chart-area, chart-grid, chart-label."</p>
                    <LineChart data=chart_data />
                </Card>
            </section>

            // ── Tempo Progress Chart ──────────────────────────────────
            <section id="tempo-chart">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Tempo Progress Chart"</h3>
                <Card>
                    <div class="space-y-6">
                        <div>
                            <p class="text-xs text-faint mb-3">"SVG line chart for tempo data with target reference line, progress percentage, and tooltips. Uses chart-line, chart-secondary (target), chart-grid, chart-label tokens."</p>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"With target + progress (8 data points, target 120 BPM)"</p>
                            <TempoProgressChart
                                entries=tempo_entries.clone()
                                target_bpm=tempo_target
                                latest_tempo=tempo_latest
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Without target (no reference line or progress)"</p>
                            <TempoProgressChart
                                entries=tempo_entries.clone()
                                target_bpm=None
                                latest_tempo=tempo_latest
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Single data point"</p>
                            <TempoProgressChart
                                entries=vec![TempoHistoryEntry {
                                    session_date: "2026-02-23T10:00:00Z".to_string(),
                                    tempo: 80,
                                    session_id: "s1".to_string(),
                                }]
                                target_bpm=Some(120)
                                latest_tempo=Some(80)
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Empty state (no data)"</p>
                            <TempoProgressChart
                                entries=vec![]
                                target_bpm=Some(120)
                                latest_tempo=None
                            />
                        </div>
                    </div>
                </Card>
            </section>

            // ══════════════════════════════════════════════════════════
            // COMPONENTS — Practice
            // ══════════════════════════════════════════════════════════

            // ── Setlist Entry Row ─────────────────────────────────────
            <section id="setlist-entry">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Setlist Entry Row"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"With controls (remove, move up/down)"</p>
                            <SetlistEntryRow
                                entry=entry_full
                                on_remove=Some(Callback::new(|_: String| {}))
                                on_move_up=Some(Callback::new(|_: String| {}))
                                on_move_down=Some(Callback::new(|_: String| {}))
                                show_controls=true
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Display-only (no controls)"</p>
                            <SetlistEntryRow
                                entry=entry_display
                                show_controls=false
                            />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Drag-active state"</p>
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
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Drag & Drop Primitives"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Drag Handle"</p>
                            <p class="text-xs text-faint mb-2">"Six-dot grip icon, 44×44px touch target. Used inside SetlistEntryRow."</p>
                            <div class="flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3">
                                <div class="flex items-center justify-center w-11 h-11 min-w-[44px] min-h-[44px] cursor-grab text-faint">
                                    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                                        <circle cx="5" cy="3" r="1.5" />
                                        <circle cx="11" cy="3" r="1.5" />
                                        <circle cx="5" cy="8" r="1.5" />
                                        <circle cx="11" cy="8" r="1.5" />
                                        <circle cx="5" cy="13" r="1.5" />
                                        <circle cx="11" cy="13" r="1.5" />
                                    </svg>
                                </div>
                                <span class="text-sm text-secondary">"Drag me to reorder"</span>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Drop Indicator"</p>
                            <p class="text-xs text-faint mb-2">"Accent line between entries during drag. Always occupies layout space."</p>
                            <div class="space-y-2">
                                <div class="flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3">
                                    <span class="text-sm text-secondary">"Entry above"</span>
                                </div>
                                <DropIndicator visible=Signal::derive(|| true) />
                                <div class="flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3">
                                    <span class="text-sm text-secondary">"Entry below"</span>
                                </div>
                                <DropIndicator visible=Signal::derive(|| false) />
                                <div class="flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3">
                                    <span class="text-sm text-secondary">"No indicator visible here"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Routine Save Form ─────────────────────────────────────
            <section id="routine-save">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Routine Save Form"</h3>
                <p class="text-xs text-faint mb-3">"Click the dashed button to expand the form. Interactive — try saving without a name."</p>
                <RoutineSaveForm on_save=Callback::new(|_name: String| {}) />
            </section>

            // ── Loading States ────────────────────────────────────────
            <section id="loading">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Loading States"</h3>
                <Card>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-3">"Generic spinner"</p>
                            <p class="text-xs text-faint mb-3">"For non-practice utility contexts."</p>
                            <div class="flex items-center gap-3">
                                <span class="animate-spin rounded-full h-6 w-6 border-2 border-accent-focus border-t-transparent"></span>
                                <span class="text-sm text-muted">"Loading..."</span>
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-3">"Metronome (practice context)"</p>
                            <p class="text-xs text-faint mb-3">"Musical loading for practice flows."</p>
                            <div class="flex items-center gap-3">
                                <div class="w-8 h-8 flex items-center justify-center">
                                    <svg width="24" height="32" viewBox="0 0 24 32" class="metronome-loading text-warm-accent-text">
                                        <line x1="12" y1="4" x2="12" y2="28" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                                        <circle cx="12" cy="6" r="3" fill="currentColor" />
                                        <rect x="8" y="26" width="8" height="4" rx="1" fill="currentColor" opacity="0.5" />
                                    </svg>
                                </div>
                                <span class="text-sm text-muted">"Preparing session..."</span>
                            </div>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Skeleton Components ──────────────────────────────────
            <section id="skeletons">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Skeleton Components"</h3>
                <Card>
                    <div class="space-y-8">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-3">"SkeletonLine"</p>
                            <p class="text-xs text-faint mb-3">"Text placeholders at various widths."</p>
                            <div class="space-y-2">
                                <SkeletonLine />
                                <SkeletonLine width="w-1/2" />
                                <SkeletonLine width="w-1/4" height="h-3" />
                            </div>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-3">"SkeletonBlock"</p>
                            <p class="text-xs text-faint mb-3">"Card/chart placeholder blocks."</p>
                            <SkeletonBlock height="h-24" />
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-3">"SkeletonItemCard"</p>
                            <p class="text-xs text-faint mb-3">"Matches LibraryItemCard layout."</p>
                            <ul class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <SkeletonItemCard />
                                <SkeletonItemCard />
                            </ul>
                        </div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-3">"SkeletonCardList"</p>
                            <p class="text-xs text-faint mb-3">"Generic list page skeleton for sessions, routines."</p>
                            <SkeletonCardList count=3 />
                        </div>
                    </div>
                </Card>
            </section>

            // ── Chart Empty/Loading States ────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Chart States"</h3>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <Card>
                        <p class="text-xs font-medium text-muted uppercase mb-3">"Empty state"</p>
                        <div class="flex flex-col items-center justify-center py-8 text-center">
                            <svg width="48" height="48" viewBox="0 0 48 48" class="text-faint mb-3">
                                <rect x="6" y="32" width="6" height="8" rx="1" fill="currentColor" opacity="0.3" />
                                <rect x="16" y="24" width="6" height="16" rx="1" fill="currentColor" opacity="0.3" />
                                <rect x="26" y="28" width="6" height="12" rx="1" fill="currentColor" opacity="0.3" />
                                <rect x="36" y="20" width="6" height="20" rx="1" fill="currentColor" opacity="0.3" />
                                <line x1="4" y1="42" x2="44" y2="42" stroke="currentColor" stroke-width="1.5" opacity="0.3" />
                            </svg>
                            <p class="text-sm text-muted">"Practice this week to see your progress here"</p>
                        </div>
                    </Card>
                    <Card>
                        <p class="text-xs font-medium text-muted uppercase mb-3">"Loading state"</p>
                        <div class="flex flex-col items-center justify-center py-8 text-center">
                            <div class="w-full h-32 rounded-lg bg-surface-secondary animate-pulse mb-3"></div>
                            <span class="text-xs text-faint">"Loading practice data..."</span>
                        </div>
                    </Card>
                </div>
            </section>

            // ── Week Strip ───────────────────────────────────────────
            <section id="week-strip">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Week Strip"</h3>
                <div class="space-y-6">
                    // Full WeekStrip with practice dots on 3 days
                    <Card>
                        <p class="text-xs font-medium text-muted uppercase mb-3">"WeekStrip — with practices on 3 days (selected: Wednesday)"</p>
                        <WeekStrip
                            week_start=Signal::derive(|| NaiveDate::from_ymd_opt(2026, 3, 2).unwrap())
                            selected_date=Signal::derive(|| Some(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap()))
                            session_dates=Signal::derive(|| {
                                let mut s = HashSet::new();
                                s.insert(NaiveDate::from_ymd_opt(2026, 3, 2).unwrap());
                                s.insert(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap());
                                s.insert(NaiveDate::from_ymd_opt(2026, 3, 6).unwrap());
                                s
                            })
                            on_day_click=Callback::new(|_| {})
                            on_prev_week=Callback::new(|_| {})
                            on_next_week=Callback::new(|_| {})
                            on_today=Callback::new(|_| {})
                            is_current_week=Signal::derive(|| true)
                        />
                    </Card>

                    // Empty week — no practices
                    <Card>
                        <p class="text-xs font-medium text-muted uppercase mb-3">"WeekStrip — empty week (no practices, no selection)"</p>
                        <WeekStrip
                            week_start=Signal::derive(|| NaiveDate::from_ymd_opt(2026, 2, 23).unwrap())
                            selected_date=Signal::derive(|| None)
                            session_dates=Signal::derive(HashSet::new)
                            on_day_click=Callback::new(|_| {})
                            on_prev_week=Callback::new(|_| {})
                            on_next_week=Callback::new(|_| {})
                            on_today=Callback::new(|_| {})
                            is_current_week=Signal::derive(|| false)
                        />
                    </Card>

                    // Dual-month label (week spanning two months)
                    <Card>
                        <p class="text-xs font-medium text-muted uppercase mb-3">"WeekStrip — dual-month label (Feb – Mar 2026)"</p>
                        <WeekStrip
                            week_start=Signal::derive(|| NaiveDate::from_ymd_opt(2026, 2, 23).unwrap())
                            selected_date=Signal::derive(|| Some(NaiveDate::from_ymd_opt(2026, 2, 25).unwrap()))
                            session_dates=Signal::derive(|| {
                                let mut s = HashSet::new();
                                s.insert(NaiveDate::from_ymd_opt(2026, 2, 24).unwrap());
                                s.insert(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
                                s
                            })
                            on_day_click=Callback::new(|_| {})
                            on_prev_week=Callback::new(|_| {})
                            on_next_week=Callback::new(|_| {})
                            on_today=Callback::new(|_| {})
                            is_current_week=Signal::derive(|| false)
                        />
                    </Card>

                    // Individual DayCell states
                    <Card>
                        <p class="text-xs font-medium text-muted uppercase mb-3">"DayCell — individual states"</p>
                        <div class="grid grid-cols-4 gap-4">
                            <div class="text-center">
                                <p class="text-xs text-faint mb-2">"Default"</p>
                                <DayCell
                                    date=NaiveDate::from_ymd_opt(2026, 3, 2).unwrap()
                                    day_abbrev="M"
                                    is_selected=false
                                    has_sessions=false
                                    on_click=Callback::new(|_| {})
                                />
                            </div>
                            <div class="text-center">
                                <p class="text-xs text-faint mb-2">"With practice"</p>
                                <DayCell
                                    date=NaiveDate::from_ymd_opt(2026, 3, 3).unwrap()
                                    day_abbrev="T"
                                    is_selected=false
                                    has_sessions=true
                                    on_click=Callback::new(|_| {})
                                />
                            </div>
                            <div class="text-center">
                                <p class="text-xs text-faint mb-2">"Selected"</p>
                                <DayCell
                                    date=NaiveDate::from_ymd_opt(2026, 3, 4).unwrap()
                                    day_abbrev="W"
                                    is_selected=true
                                    has_sessions=false
                                    on_click=Callback::new(|_| {})
                                />
                            </div>
                            <div class="text-center">
                                <p class="text-xs text-faint mb-2">"Selected + practice"</p>
                                <DayCell
                                    date=NaiveDate::from_ymd_opt(2026, 3, 5).unwrap()
                                    day_abbrev="T"
                                    is_selected=true
                                    has_sessions=true
                                    on_click=Callback::new(|_| {})
                                />
                            </div>
                        </div>
                    </Card>
                </div>
            </section>

            // ── Shell Components ──────────────────────────────────────
            <section id="shell">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Shell Components"</h3>
                <Card>
                    <div class="space-y-4">
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"App Header"</p>
                            <p class="text-xs text-faint">"Visible at the top of this page. Uses "<code class="bg-surface-input rounded px-1">"glass-chrome"</code>" utility, "<code class="bg-surface-input rounded px-1">"border-border-default"</code>" bottom border. Desktop-only nav links with "<code class="bg-surface-input rounded px-1">"text-accent-text"</code>" active state."</p>
                        </div>
                        <div class="border-b border-border-default"></div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"Bottom Tab Bar"</p>
                            <p class="text-xs text-faint">"Visible on mobile (below 640px). Fixed bottom, "<code class="bg-surface-input rounded px-1">"glass-chrome"</code>" + "<code class="bg-surface-input rounded px-1">"pb-safe"</code>" for iOS safe area. 4 tabs with SVG icons, 44px min touch target."</p>
                        </div>
                        <div class="border-b border-border-default"></div>
                        <div>
                            <p class="text-xs font-medium text-muted uppercase mb-2">"App Footer"</p>
                            <p class="text-xs text-faint">"Visible at the bottom of this page. "<code class="bg-surface-input rounded px-1">"border-white/10"</code>" top border, "<code class="bg-surface-input rounded px-1">"text-xs text-faint"</code>" centered attribution text."</p>
                        </div>
                    </div>
                </Card>
            </section>

            // ── Accessibility ─────────────────────────────────────────
            <section id="accessibility">
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Accessibility — WCAG Contrast"</h3>
                <Card>
                    <p class="text-xs text-faint mb-4">"Contrast ratios for text tokens against common surface backgrounds. WCAG AA requires 4.5:1 for normal text, 3:1 for large text."</p>
                    <div class="overflow-x-auto">
                        <table class="w-full text-xs">
                            <thead>
                                <tr class="border-b border-border-default">
                                    <th class="text-left py-2 pr-4 text-muted font-medium">"Text Token"</th>
                                    <th class="text-left py-2 pr-4 text-muted font-medium">"On Background"</th>
                                    <th class="text-left py-2 text-muted font-medium">"Status"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border-default">
                                <tr>
                                    <td class="py-2 pr-4 text-text-primary">"text-primary (white)"</td>
                                    <td class="py-2 pr-4 text-faint">"gradient bg"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-text-secondary">"text-secondary"</td>
                                    <td class="py-2 pr-4 text-faint">"gradient bg"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-text-label">"text-label"</td>
                                    <td class="py-2 pr-4 text-faint">"surface-primary card"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-text-muted">"text-muted"</td>
                                    <td class="py-2 pr-4 text-faint">"gradient bg"</td>
                                    <td class="py-2 text-warning-text">"AA Large only"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-text-faint">"text-faint"</td>
                                    <td class="py-2 pr-4 text-faint">"gradient bg"</td>
                                    <td class="py-2 text-danger-text">"Below AA"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-accent-text">"accent-text"</td>
                                    <td class="py-2 pr-4 text-faint">"gradient bg"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-danger-text">"danger-text"</td>
                                    <td class="py-2 pr-4 text-faint">"danger-surface"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-success-text">"success-text"</td>
                                    <td class="py-2 pr-4 text-faint">"success-surface"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                                <tr>
                                    <td class="py-2 pr-4 text-warning-text">"warning-text"</td>
                                    <td class="py-2 pr-4 text-faint">"warning-surface"</td>
                                    <td class="py-2 text-success-text">"AA Pass"</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                    <p class="text-xs text-faint mt-4">"Note: text-faint is intentionally below AA — it is used only for decorative/tertiary info (timestamps, footer text) where readability impact is minimal. text-muted passes AA at large text sizes (18px+)."</p>
                </Card>
            </section>

            // ── Practice Components (context-dependent) ───────────────
            <section>
                <h3 class="text-lg font-semibold text-primary mb-4 font-heading">"Practice Components"</h3>
                <Card>
                    <p class="text-sm text-muted mb-3">"These components require app context (ViewModel, Core) and cannot be rendered in isolation. They are composed from the primitives shown above."</p>
                    <div class="space-y-2 text-sm text-faint">
                        <div class="flex items-center gap-2">
                            <span class="text-accent-text">"→"</span>
                            <span>"SessionTimer — Card + TypeBadge + SetlistEntryRow + Button (timer, next/finish/skip)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-accent-text">"→"</span>
                            <span>"SessionSummary — Card + Button + RoutineSaveForm (scoring, notes, save)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-accent-text">"→"</span>
                            <span>"SetlistBuilder — Card + SetlistEntryRow + DragHandle + DropIndicator + RoutineLoader"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-accent-text">"→"</span>
                            <span>"RoutineLoader — Card (routine list with load buttons)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-accent-text">"→"</span>
                            <span>"ErrorBanner — shown above as static preview"</span>
                        </div>
                    </div>
                </Card>
            </section>
        </div>
    }
}

/// Catalogue demo: BottomSheet open/close trigger.
#[component]
fn BottomSheetDemo() -> impl IntoView {
    let open = RwSignal::new(false);
    let close = Callback::new(move |_| open.set(false));
    view! {
        <div class="space-y-3">
            <Button
                variant=ButtonVariant::Primary
                on_click=Callback::new(move |_| open.set(true))
            >
                "Open Sheet"
            </Button>
            <BottomSheet
                open=open
                on_close=close
                nav_title="Demo Sheet".to_string()
            >
                <div class="space-y-3">
                    <p class="text-sm text-secondary">
                        "Tap Cancel in the nav bar, drag the handle down, or tap outside to dismiss."
                    </p>
                    <p class="text-sm text-muted">
                        "On iOS the swipe gesture also fires a light haptic when crossing the dismiss threshold."
                    </p>
                </div>
            </BottomSheet>
        </div>
    }
}

/// Catalogue demo: SwipeActions wrapping a fake row.
#[component]
fn SwipeActionsDemo() -> impl IntoView {
    let deleted = RwSignal::new(false);
    let on_delete = Callback::new(move |_| deleted.set(true));
    view! {
        <div class="border border-border-default rounded-lg overflow-hidden">
            <Show
                when=move || !deleted.get()
                fallback=move || view! {
                    <div class="p-card text-sm text-faint text-center">
                        "Row deleted. Refresh the page to reset."
                    </div>
                }
            >
                <SwipeActions on_delete=on_delete>
                    <div class="p-card flex items-center justify-between">
                        <div>
                            <p class="text-sm font-medium text-primary">"Sample Row"</p>
                            <p class="text-xs text-muted">"Swipe me left on iOS"</p>
                        </div>
                        <span class="text-xs text-faint">"›"</span>
                    </div>
                </SwipeActions>
            </Show>
        </div>
    }
}

/// Catalogue demo: ContextMenu wrapping a fake row.
#[component]
fn ContextMenuDemo() -> impl IntoView {
    let last_action = RwSignal::new(String::new());
    let actions = vec![
        ContextMenuAction {
            label: "Edit".to_string(),
            destructive: false,
            on_select: Callback::new(move |_| last_action.set("Edit selected".to_string())),
        },
        ContextMenuAction {
            label: "Duplicate".to_string(),
            destructive: false,
            on_select: Callback::new(move |_| last_action.set("Duplicate selected".to_string())),
        },
        ContextMenuAction {
            label: "Delete".to_string(),
            destructive: true,
            on_select: Callback::new(move |_| last_action.set("Delete selected".to_string())),
        },
    ];
    view! {
        <div class="space-y-3">
            <ContextMenu actions=actions>
                <div class="p-card border border-border-default rounded-lg flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-primary">"Sample Row"</p>
                        <p class="text-xs text-muted">"Long-press me on iOS"</p>
                    </div>
                    <span class="text-xs text-faint">"›"</span>
                </div>
            </ContextMenu>
            {move || {
                let action = last_action.get();
                if action.is_empty() {
                    None
                } else {
                    Some(view! {
                        <p class="text-xs text-accent-text">{action}</p>
                    })
                }
            }}
        </div>
    }
}

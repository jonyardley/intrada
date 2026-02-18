use std::collections::HashMap;

use leptos::prelude::*;

use intrada_core::analytics::DailyPracticeTotal;
use intrada_core::LibraryItemView;

use crate::components::{
    BackLink, Button, ButtonVariant, Card, FieldLabel, FormFieldError, LibraryItemCard, LineChart,
    PageHeading, StatCard, TextArea, TextField, TypeBadge, TypeTabs,
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
    let sample_area = RwSignal::new(String::new());

    let empty_errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let sample_errors: RwSignal<HashMap<String, String>> = RwSignal::new({
        let mut m = HashMap::new();
        m.insert("title".to_string(), "Title is required".to_string());
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

    view! {
        <div class="space-y-12">
            <PageHeading text="Design System Catalogue" />
            <p class="text-sm text-gray-400 -mt-4 mb-8">
                "Dev-only reference of all UI components and design tokens. "
                "See " <code class="text-xs bg-white/10 rounded px-1 py-0.5">"specs/design-system.md"</code> " for full documentation."
            </p>

            // ── Colour Palette ──────────────────────────────────────────
            <section>
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

            // ── Typography ──────────────────────────────────────────────
            <section>
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
                    </div>
                </Card>
            </section>

            // ── Glass Card ──────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Glass Card"</h3>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <Card>
                        <p class="text-sm text-gray-300">"Default glass-card with standard padding."</p>
                    </Card>
                    <Card>
                        <h3 class="text-lg font-semibold text-white mb-2">"With heading"</h3>
                        <p class="text-sm text-gray-400">"Card content with heading and body text."</p>
                    </Card>
                </div>
            </section>

            // ── Stat Cards ──────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Stat Card"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    <StatCard title="Current Streak" value="7 days".to_string() subtitle="Best: 14 days" />
                    <StatCard title="This Week" value="3h 45m".to_string() />
                    <StatCard title="Sessions" value="12".to_string() subtitle="This month" />
                    <StatCard title="Avg Score" value="3.8".to_string() subtitle="Out of 5" />
                </div>
            </section>

            // ── Library Item Cards ──────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Library Item Card"</h3>
                <ul class="space-y-3">
                    <LibraryItemCard item=sample_piece />
                    <LibraryItemCard item=sample_exercise />
                </ul>
            </section>

            // ── Buttons ─────────────────────────────────────────────────
            <section>
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

            // ── Type Badge ──────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Type Badge"</h3>
                <Card>
                    <div class="flex flex-wrap gap-3">
                        <TypeBadge item_type="piece".to_string() />
                        <TypeBadge item_type="exercise".to_string() />
                        <TypeBadge item_type="unknown".to_string() />
                    </div>
                </Card>
            </section>

            // ── Type Tabs ───────────────────────────────────────────────
            <section>
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

            // ── Form Inputs ─────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Form Inputs"</h3>
                <Card>
                    <div class="space-y-4">
                        <TextField
                            id="demo-text"
                            label="Text Field"
                            value=sample_text
                            field_name="title"
                            errors=empty_errors
                            placeholder="Enter some text..."
                            hint="This is a hint below the label"
                        />
                        <TextField
                            id="demo-text-error"
                            label="With Validation Error"
                            value=sample_text
                            field_name="title"
                            errors=sample_errors
                            required=true
                        />
                        <TextArea
                            id="demo-area"
                            label="Text Area"
                            value=sample_area
                            field_name="notes"
                            errors=empty_errors
                            hint="Optional hint text below the label"
                        />
                    </div>
                </Card>
            </section>

            // ── Field Label ─────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Field Label"</h3>
                <Card>
                    <dl class="space-y-2">
                        <FieldLabel text="Key Signature" />
                        <dd class="text-white">"Db Major"</dd>
                        <FieldLabel text="Tempo" />
                        <dd class="text-white">"66 bpm"</dd>
                    </dl>
                </Card>
            </section>

            // ── Navigation ──────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Navigation"</h3>
                <Card>
                    <div class="space-y-3">
                        <BackLink label="Back to Library" href="/".to_string() />
                        <PageHeading text="Sample Page Heading" />
                    </div>
                </Card>
            </section>

            // ── Line Chart ──────────────────────────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Line Chart"</h3>
                <Card>
                    <LineChart data=chart_data />
                </Card>
            </section>

            // ── Form Field Error (standalone) ───────────────────────────
            <section>
                <h3 class="text-lg font-semibold text-white mb-4">"Form Field Error"</h3>
                <Card>
                    <FormFieldError field="title" errors=sample_errors />
                </Card>
            </section>
        </div>
    }
}

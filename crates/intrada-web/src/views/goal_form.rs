use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::domain::goal::{GoalEvent, GoalKind};
use intrada_core::domain::types::CreateGoal;
use intrada_core::{Event, ViewModel};

use crate::components::{
    BackLink, Button, ButtonVariant, Card, Icon, IconName, PageHeading, TextField,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// The four goal types available for creation.
#[derive(Debug, Clone, Copy, PartialEq)]
enum GoalType {
    Frequency,
    Time,
    Mastery,
    Milestone,
}

impl GoalType {
    fn label(&self) -> &'static str {
        match self {
            GoalType::Frequency => "Practice Frequency",
            GoalType::Time => "Practice Time",
            GoalType::Mastery => "Item Mastery",
            GoalType::Milestone => "Milestone",
        }
    }

    fn icon(&self) -> IconName {
        match self {
            GoalType::Frequency => IconName::Calendar,
            GoalType::Time => IconName::Clock,
            GoalType::Mastery => IconName::Star,
            GoalType::Milestone => IconName::Target,
        }
    }

    fn hint(&self) -> &'static str {
        match self {
            GoalType::Frequency => "How many days per week do you want to practise?",
            GoalType::Time => "How many minutes per week do you want to practise?",
            GoalType::Mastery => "Pick a library item and a target score to aim for.",
            GoalType::Milestone => "Describe a personal milestone you want to achieve.",
        }
    }
}

/// Goal creation form with type selector and dynamic fields.
#[component]
pub fn GoalFormView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    // Form state
    let selected_type = RwSignal::new(GoalType::Frequency);
    let title = RwSignal::new(String::new());
    let title_manually_edited = RwSignal::new(false);

    // Type-specific fields
    let target_days = RwSignal::new("5".to_string());
    let target_minutes = RwSignal::new("60".to_string());
    let selected_item_id = RwSignal::new(String::new());
    let target_score = RwSignal::new("4".to_string());
    let milestone_description = RwSignal::new(String::new());

    // Validation errors
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    // Auto-generate title when type or target changes (unless manually edited)
    Effect::new(move || {
        if title_manually_edited.get() {
            return;
        }
        let auto_title = match selected_type.get() {
            GoalType::Frequency => {
                let days = target_days.get();
                format!("Practise {} days per week", days)
            }
            GoalType::Time => {
                let mins = target_minutes.get();
                format!("Practise {} minutes per week", mins)
            }
            GoalType::Mastery => {
                let vm = view_model.get();
                let item_id = selected_item_id.get();
                let item_name = vm
                    .items
                    .iter()
                    .find(|i| i.id == item_id)
                    .map(|i| i.title.as_str())
                    .unwrap_or("an item");
                format!("Master {}", item_name)
            }
            GoalType::Milestone => {
                let desc = milestone_description.get();
                if desc.is_empty() {
                    "Achieve a milestone".to_string()
                } else if desc.chars().count() > 50 {
                    let truncated: String = desc.chars().take(50).collect();
                    format!("{truncated}...")
                } else {
                    desc
                }
            }
        };
        title.set(auto_title);
    });

    view! {
        <div class="sm:max-w-2xl sm:mx-auto">
            <BackLink label="Back to Goals" href="/goals".to_string() />

            <PageHeading text="Set a Goal" />

            <Card>
                <form
                    class="space-y-6"
                    on:submit=move |ev: ev::SubmitEvent| {
                        ev.prevent_default();

                        let mut validation_errors = HashMap::new();
                        let title_val = title.get().trim().to_string();

                        if title_val.is_empty() {
                            validation_errors.insert("title".to_string(), "Title is required".to_string());
                        }
                        if title_val.len() > 200 {
                            validation_errors.insert("title".to_string(), "Title must be 200 characters or fewer".to_string());
                        }

                        let kind = match selected_type.get() {
                            GoalType::Frequency => {
                                match target_days.get().parse::<u8>() {
                                    Ok(d) if (1..=7).contains(&d) => GoalKind::SessionFrequency { target_days_per_week: d },
                                    _ => {
                                        validation_errors.insert("target_days".to_string(), "Enter a number between 1 and 7".to_string());
                                        GoalKind::SessionFrequency { target_days_per_week: 1 }
                                    }
                                }
                            }
                            GoalType::Time => {
                                match target_minutes.get().parse::<u32>() {
                                    Ok(m) if (1..=10080).contains(&m) => GoalKind::PracticeTime { target_minutes_per_week: m },
                                    _ => {
                                        validation_errors.insert("target_minutes".to_string(), "Enter a number between 1 and 10080".to_string());
                                        GoalKind::PracticeTime { target_minutes_per_week: 1 }
                                    }
                                }
                            }
                            GoalType::Mastery => {
                                let item_id = selected_item_id.get().trim().to_string();
                                if item_id.is_empty() {
                                    validation_errors.insert("item_id".to_string(), "Select a library item".to_string());
                                }
                                match target_score.get().parse::<u8>() {
                                    Ok(s) if (1..=5).contains(&s) => GoalKind::ItemMastery { item_id, target_score: s },
                                    _ => {
                                        validation_errors.insert("target_score".to_string(), "Enter a score between 1 and 5".to_string());
                                        GoalKind::ItemMastery { item_id, target_score: 1 }
                                    }
                                }
                            }
                            GoalType::Milestone => {
                                let desc = milestone_description.get().trim().to_string();
                                if desc.is_empty() {
                                    validation_errors.insert("milestone_description".to_string(), "Description is required".to_string());
                                }
                                if desc.len() > 1000 {
                                    validation_errors.insert("milestone_description".to_string(), "Description must be 1000 characters or fewer".to_string());
                                }
                                GoalKind::Milestone { description: desc }
                            }
                        };

                        if !validation_errors.is_empty() {
                            errors.set(validation_errors);
                            return;
                        }
                        errors.set(HashMap::new());

                        let event = Event::Goal(GoalEvent::Add(CreateGoal {
                            title: title_val,
                            kind,
                            deadline: None,
                        }));

                        let core_ref = core.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        navigate("/goals", NavigateOptions { replace: true, ..Default::default() });
                    }
                >
                    // Goal type selector
                    <div>
                        <p class="form-label">"Goal Type"</p>
                        <div class="grid grid-cols-2 gap-2 mt-1">
                            {[GoalType::Frequency, GoalType::Time, GoalType::Mastery, GoalType::Milestone]
                                .into_iter()
                                .map(|gt| {
                                    let is_selected = Signal::derive(move || selected_type.get() == gt);
                                    view! {
                                        <button
                                            type="button"
                                            class=move || {
                                                if is_selected.get() {
                                                    "flex items-center gap-2 p-3 rounded-lg border-2 border-accent-focus bg-accent-focus/10 text-primary text-sm font-medium transition-all"
                                                } else {
                                                    "flex items-center gap-2 p-3 rounded-lg border border-border-default bg-surface-secondary hover:bg-surface-hover text-secondary text-sm transition-all"
                                                }
                                            }
                                            on:click=move |_| {
                                                selected_type.set(gt);
                                                title_manually_edited.set(false);
                                                errors.set(HashMap::new());
                                            }
                                        >
                                            <Icon name=gt.icon() class="w-4 h-4" />
                                            <span>{gt.label()}</span>
                                        </button>
                                    }
                                })
                                .collect::<Vec<_>>()
                            }
                        </div>
                        <p class="hint-text mt-2">{move || selected_type.get().hint()}</p>
                    </div>

                    // Dynamic fields per type
                    {move || match selected_type.get() {
                        GoalType::Frequency => view! {
                            <TextField
                                id="target-days"
                                label="Days per week"
                                value=target_days
                                required=true
                                input_type="number"
                                hint="1-7"
                                placeholder="5"
                                field_name="target_days"
                                errors=errors
                            />
                        }.into_any(),
                        GoalType::Time => view! {
                            <TextField
                                id="target-minutes"
                                label="Minutes per week"
                                value=target_minutes
                                required=true
                                input_type="number"
                                hint="1-10080"
                                placeholder="60"
                                field_name="target_minutes"
                                errors=errors
                            />
                        }.into_any(),
                        GoalType::Mastery => {
                            let vm = view_model.get();
                            let items = vm.items.clone();
                            if items.is_empty() {
                                view! {
                                    <div class="text-center py-4">
                                        <p class="text-sm text-muted">"Add items to your library first."</p>
                                        <a href="/" class="text-xs text-accent-text hover:text-accent-hover font-medium mt-1 inline-block">"Go to Library"</a>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="space-y-4">
                                        <div>
                                            <label class="form-label" for="item-picker">"Library Item"</label>
                                            <select
                                                id="item-picker"
                                                class="input-base"
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    selected_item_id.set(val);
                                                    title_manually_edited.set(false);
                                                }
                                                prop:value=move || selected_item_id.get()
                                            >
                                                <option value="">"Select an item\u{2026}"</option>
                                                {items.into_iter().map(|item| {
                                                    let id = item.id.clone();
                                                    let display = format!("{} ({})", item.title, item.item_type);
                                                    view! {
                                                        <option value=id>{display}</option>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </select>
                                            {move || errors.get().get("item_id").map(|e| view! {
                                                <p class="text-xs text-danger-text mt-1">{e.clone()}</p>
                                            })}
                                        </div>
                                        <TextField
                                            id="target-score"
                                            label="Target Score"
                                            value=target_score
                                            required=true
                                            input_type="number"
                                            hint="1-5"
                                            placeholder="4"
                                            field_name="target_score"
                                            errors=errors
                                        />
                                    </div>
                                }.into_any()
                            }
                        }
                        GoalType::Milestone => view! {
                            <div>
                                <label class="form-label" for="milestone-desc">"Description"</label>
                                <p class="hint-text">"What do you want to achieve?"</p>
                                <textarea
                                    id="milestone-desc"
                                    class="input-base min-h-[80px]"
                                    placeholder="e.g. Perform at the spring recital"
                                    bind:value=milestone_description
                                    required=true
                                />
                                {move || errors.get().get("milestone_description").map(|e| view! {
                                    <p class="text-xs text-danger-text mt-1">{e.clone()}</p>
                                })}
                            </div>
                        }.into_any(),
                    }}

                    // Title (auto-generated, editable)
                    <div>
                        <label class="form-label" for="goal-title">"Title"</label>
                        <p class="hint-text">"Auto-generated from your selections. Edit if you prefer something different."</p>
                        <input
                            id="goal-title"
                            type="text"
                            class="input-base"
                            bind:value=title
                            required=true
                            on:input=move |_| {
                                title_manually_edited.set(true);
                            }
                        />
                        {move || errors.get().get("title").map(|e| view! {
                            <p class="text-xs text-danger-text mt-1">{e.clone()}</p>
                        })}
                    </div>

                    // Submit
                    <div class="flex gap-3 pt-2">
                        <Button
                            variant=ButtonVariant::Primary
                            button_type="submit"
                            loading=Signal::derive(move || is_submitting.get())
                        >
                            {move || if is_submitting.get() { "Creating\u{2026}" } else { "Create Goal" }}
                        </Button>
                    </div>
                </form>
            </Card>
        </div>
    }
}

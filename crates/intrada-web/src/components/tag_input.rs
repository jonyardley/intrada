use std::collections::HashMap;

use leptos::prelude::*;

use crate::components::{Autocomplete, FormFieldError};

/// Chip-based multi-tag input with integrated autocomplete.
///
/// Renders each tag as a chip/badge with a remove (×) button.
/// After chips, renders an inline text input for new tag entry.
/// Typing triggers Autocomplete suggestions (excluding already-selected tags).
/// Selecting a suggestion or pressing comma/Enter adds tag to `tags` and clears input.
/// Pasting comma-separated text parses and adds all tags directly.
#[component]
pub fn TagInput(
    id: &'static str,
    tags: RwSignal<Vec<String>>,
    available_tags: Signal<Vec<String>>,
    field_name: &'static str,
    errors: RwSignal<HashMap<String, String>>,
) -> impl IntoView {
    let input_value = RwSignal::new(String::new());

    // Derive exclude signal from current tags (so autocomplete doesn't suggest already-selected)
    let exclude = Signal::derive(move || tags.get());

    // Add a tag if it's non-empty and not already present (case-insensitive check)
    let add_tag = move |tag: String| {
        let trimmed = tag.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        let current = tags.get();
        let already_exists = current
            .iter()
            .any(|t| t.to_lowercase() == trimmed.to_lowercase());
        if !already_exists {
            tags.update(|t| t.push(trimmed));
        }
        input_value.set(String::new());
    };

    // Handle autocomplete selection — add the selected tag
    let on_select = Callback::new(move |selected: String| {
        add_tag(selected);
    });

    // Handle commit (Enter/comma) — add free-text tag
    let on_commit = Callback::new(move |val: String| {
        add_tag(val);
    });

    // Handle paste — parse comma-separated values and add each
    let on_paste = Callback::new(move |text: String| {
        let new_tags: Vec<String> = text
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for tag in new_tags {
            add_tag(tag);
        }
    });

    let error_id = format!("{id}-error");

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-200 mb-1" for=id>
                "Tags"
            </label>
            <div class="w-full rounded-lg border border-white/20 bg-white/10 px-2 py-1.5 flex flex-wrap items-center gap-1.5 focus-within:border-indigo-400 focus-within:ring-1 focus-within:ring-indigo-400">
                // Render tag chips
                {move || {
                    tags.get().into_iter().map(|tag| {
                        let tag_display = tag.clone();
                        let tag_label = tag.clone();
                        view! {
                            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-indigo-600/30 text-indigo-200 text-xs">
                                {tag_display}
                                <button
                                    type="button"
                                    class="text-indigo-300 hover:text-white focus:outline-none"
                                    aria-label=format!("Remove tag {tag_label}")
                                    on:click={
                                        let tag_val = tag.clone();
                                        move |_| {
                                            tags.update(|t| t.retain(|existing| existing != &tag_val));
                                        }
                                    }
                                >
                                    "\u{00d7}"
                                </button>
                            </span>
                        }
                    }).collect::<Vec<_>>()
                }}
                // Inline autocomplete input
                <div class="flex-1 min-w-[120px] relative">
                    <Autocomplete
                        id=id
                        suggestions=available_tags
                        value=input_value
                        on_select=on_select
                        placeholder="Add tag..."
                        exclude=exclude
                        on_commit=on_commit
                        on_paste=on_paste
                    />
                </div>
            </div>
            <FormFieldError field=field_name errors=errors error_id=error_id />
        </div>
    }
}

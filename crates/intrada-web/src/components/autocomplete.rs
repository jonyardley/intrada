use std::collections::HashMap;

use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

use crate::components::FormFieldError;
use intrada_web::helpers::filter_suggestions;

/// Reusable autocomplete dropdown component.
///
/// Filters `suggestions` by `value` text (case-insensitive, prefix-first ranking).
/// Shows dropdown when filtered list is non-empty and input length ≥ `min_chars`.
/// Click on a suggestion fires `on_select`. Dropdown closes on selection, Escape,
/// or focus leaving the component.
#[component]
pub fn Autocomplete(
    id: &'static str,
    suggestions: Signal<Vec<String>>,
    value: RwSignal<String>,
    on_select: Callback<String>,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(default = 2)] min_chars: usize,
    #[prop(default = 8)] max_suggestions: usize,
    #[prop(optional)] exclude: Option<Signal<Vec<String>>>,
    /// Called when Enter or comma is pressed with no highlighted suggestion.
    /// Used by TagInput for free-text tag creation.
    #[prop(optional)]
    on_commit: Option<Callback<String>>,
    /// Called on paste events with the raw pasted text.
    #[prop(optional)]
    on_paste: Option<Callback<String>>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let highlight_index = RwSignal::new(Option::<usize>::None);

    // Default exclude to empty vec if not provided
    let exclude = exclude.unwrap_or_else(|| Signal::derive(Vec::new));

    // Derive filtered suggestions list
    let filtered = Memo::new(move |_| {
        let val = value.get();
        if val.len() < min_chars {
            return Vec::new();
        }
        let all = suggestions.get();
        let excl = exclude.get();
        filter_suggestions(&all, &val, &excl, max_suggestions)
    });

    // Reset highlight when filtered list changes
    Effect::new(move || {
        let _ = filtered.get();
        highlight_index.set(None);
    });

    // Should dropdown be visible?
    let show_dropdown = move || is_open.get() && !filtered.get().is_empty();

    // Generate unique IDs for ARIA
    let listbox_id = format!("{id}-listbox");
    let listbox_id_clone = listbox_id.clone();

    // Handle focusout with delay to allow click events to fire first
    let on_focusout = move |_ev: web_sys::FocusEvent| {
        let handle = wasm_bindgen::closure::Closure::once(move || {
            is_open.set(false);
        });
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                handle.as_ref().unchecked_ref(),
                150,
            );
        handle.forget();
    };

    // Handle keyboard navigation
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let items = filtered.get();
        let len = items.len();

        match key.as_str() {
            "ArrowDown" if len > 0 => {
                ev.prevent_default();
                if !is_open.get() {
                    is_open.set(true);
                }
                let next = match highlight_index.get() {
                    None => 0,
                    Some(i) if i + 1 >= len => 0,
                    Some(i) => i + 1,
                };
                highlight_index.set(Some(next));
            }
            "ArrowUp" if len > 0 => {
                ev.prevent_default();
                if !is_open.get() {
                    is_open.set(true);
                }
                let next = match highlight_index.get() {
                    None | Some(0) => len.saturating_sub(1),
                    Some(i) => i - 1,
                };
                highlight_index.set(Some(next));
            }
            "Enter" | "Tab" => {
                // If a suggestion is highlighted and dropdown is open, select it
                if let Some(idx) = highlight_index.get() {
                    if is_open.get() {
                        if let Some(selected) = items.get(idx) {
                            ev.prevent_default();
                            on_select.run(selected.clone());
                            is_open.set(false);
                            highlight_index.set(None);
                            return;
                        }
                    }
                }
                // No highlighted selection — delegate to on_commit for free-text entry
                if key == "Enter" {
                    if let Some(commit) = &on_commit {
                        ev.prevent_default();
                        let val = value.get();
                        if !val.trim().is_empty() {
                            commit.run(val);
                        }
                    }
                }
            }
            "," => {
                // Comma commits for tag creation
                if let Some(commit) = &on_commit {
                    ev.prevent_default();
                    let val = value.get();
                    if !val.trim().is_empty() {
                        commit.run(val);
                    }
                }
            }
            "Escape" => {
                is_open.set(false);
                highlight_index.set(None);
            }
            _ => {}
        }
    };

    // Handle paste events
    let handle_paste = move |ev: web_sys::ClipboardEvent| {
        if let Some(on_paste_cb) = &on_paste {
            if let Some(data) = ev.clipboard_data() {
                if let Ok(text) = data.get_data("text/plain") {
                    ev.prevent_default();
                    on_paste_cb.run(text);
                }
            }
        }
    };

    view! {
        <div class="relative" on:focusout=on_focusout>
            <input
                id=id
                type="text"
                class="w-full rounded-lg border border-white/20 bg-white/10 px-3 py-2.5 text-sm text-white placeholder-gray-400 focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400"
                placeholder=placeholder.unwrap_or("")
                prop:value=move || value.get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    value.set(val);
                    is_open.set(true);
                    highlight_index.set(None);
                }
                on:keydown=on_keydown
                on:paste=handle_paste
                role="combobox"
                aria-autocomplete="list"
                aria-expanded=move || if show_dropdown() { "true" } else { "false" }
                aria-controls=listbox_id.clone()
                aria-activedescendant=move || {
                    highlight_index.get().map(|idx| format!("{id}-option-{idx}")).unwrap_or_default()
                }
            />
            <Show when=show_dropdown>
                <ul
                    id=listbox_id_clone.clone()
                    role="listbox"
                    class="absolute z-50 mt-1 w-full max-h-60 overflow-auto bg-gray-800/90 backdrop-blur-sm border border-white/10 rounded-lg shadow-lg"
                >
                    {move || {
                        filtered.get().into_iter().enumerate().map(|(idx, item)| {
                            let option_id = format!("{id}-option-{idx}");
                            let item_clone = item.clone();
                            let is_highlighted = move || highlight_index.get() == Some(idx);
                            view! {
                                <li
                                    id=option_id
                                    role="option"
                                    aria-selected=move || if is_highlighted() { "true" } else { "false" }
                                    class=move || {
                                        if is_highlighted() {
                                            "px-3 py-2 text-sm text-gray-200 cursor-pointer bg-indigo-600/50"
                                        } else {
                                            "px-3 py-2 text-sm text-gray-200 cursor-pointer hover:bg-white/10"
                                        }
                                    }
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                    }
                                    on:click={
                                        let item_val = item_clone.clone();
                                        move |_| {
                                            on_select.run(item_val.clone());
                                            is_open.set(false);
                                            highlight_index.set(None);
                                        }
                                    }
                                    on:mouseenter=move |_| {
                                        highlight_index.set(Some(idx));
                                    }
                                >
                                    {item}
                                </li>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </ul>
            </Show>
        </div>
    }
}

/// A thin wrapper combining `Autocomplete` behaviour with `TextField`-like props.
/// Renders a labelled input with autocomplete suggestions and error display.
/// Used for the composer field.
#[allow(unused_variables)]
#[component]
pub fn AutocompleteTextField(
    id: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    suggestions: Signal<Vec<String>>,
    #[prop(default = false)] required: bool,
    #[prop(optional)] placeholder: Option<&'static str>,
    field_name: &'static str,
    errors: RwSignal<HashMap<String, String>>,
) -> impl IntoView {
    let error_id = format!("{id}-error");
    let has_error = move || errors.get().contains_key(field_name);

    let on_select = Callback::new(move |selected: String| {
        value.set(selected);
    });

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-200 mb-1" for=id>
                {label}
            </label>
            <div
                aria-invalid=move || if has_error() { "true" } else { "false" }
                aria-describedby=error_id.clone()
            >
                <Autocomplete
                    id=id
                    suggestions=suggestions
                    value=value
                    on_select=on_select
                    placeholder=placeholder.unwrap_or("")
                />
            </div>
            <FormFieldError field=field_name errors=errors error_id=error_id />
        </div>
    }
}

use leptos::ev;
use leptos::prelude::*;

/// Visual variants for the shared Button component.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    DangerOutline,
}

impl ButtonVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::Primary => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-accent px-3.5 py-2.5 text-sm font-medium text-white shadow-sm hover:bg-accent-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-focus motion-safe:transition-colors min-h-[44px]",
            Self::Secondary => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-surface-secondary px-3.5 py-2.5 text-sm font-medium text-label border border-border-default hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-focus motion-safe:transition-colors min-h-[44px]",
            Self::Success => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-success px-3.5 py-2.5 text-sm font-medium text-white shadow-sm hover:bg-success-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-success-text motion-safe:transition-colors min-h-[44px]",
            Self::Danger => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-danger px-3.5 py-2.5 text-sm font-medium text-white hover:bg-danger-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-danger-text motion-safe:transition-colors min-h-[44px]",
            Self::DangerOutline => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-danger-surface px-3.5 py-2.5 text-sm font-medium text-danger-text border border-danger-text/30 hover:bg-danger-surface hover:brightness-150 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-danger-text motion-safe:transition-colors min-h-[44px]",
        }
    }
}

/// Shared button component with consistent styling per variant.
///
/// When `disabled` is true, the button is visually dimmed, shows a
/// `not-allowed` cursor, and ignores click events.
/// When `loading` is true, a small spinner is prepended to the label
/// and the button is also treated as disabled.
#[component]
pub fn Button(
    variant: ButtonVariant,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(default = "button")] button_type: &'static str,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] loading: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let is_disabled = Signal::derive(move || disabled.get() || loading.get());

    view! {
        <button
            type=button_type
            class=move || {
                let base = variant.classes();
                if is_disabled.get() {
                    format!("{base} opacity-50 cursor-not-allowed")
                } else {
                    base.to_string()
                }
            }
            disabled=is_disabled
            on:click=move |ev| {
                if !is_disabled.get() {
                    if let Some(cb) = &on_click {
                        cb.run(ev);
                    }
                }
            }
        >
            {move || {
                if loading.get() {
                    Some(view! {
                        <span class="animate-spin rounded-full h-4 w-4 border-2 border-current border-t-transparent" aria-hidden="true"></span>
                    })
                } else {
                    None
                }
            }}
            {children()}
        </button>
    }
}

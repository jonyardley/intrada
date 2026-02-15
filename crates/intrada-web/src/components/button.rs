use leptos::ev;
use leptos::prelude::*;

/// Visual variants for the shared Button component.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    DangerOutline,
}

impl ButtonVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::Primary => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-indigo-500 px-3.5 py-2.5 text-sm font-semibold text-white shadow-md shadow-indigo-500/25 hover:bg-indigo-400 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-400 motion-safe:transition-colors min-h-[44px]",
            Self::Secondary => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-white/10 px-3.5 py-2.5 text-sm font-medium text-gray-200 border border-white/20 hover:bg-white/20 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white/40 motion-safe:transition-colors min-h-[44px]",
            Self::Danger => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-red-600 px-3.5 py-2.5 text-sm font-medium text-white hover:bg-red-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-400 motion-safe:transition-colors min-h-[44px]",
            Self::DangerOutline => "inline-flex items-center justify-center gap-1.5 rounded-lg bg-red-500/10 px-3.5 py-2.5 text-sm font-medium text-red-400 border border-red-400/30 hover:bg-red-500/20 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-400 motion-safe:transition-colors min-h-[44px]",
        }
    }
}

/// Shared button component with consistent styling per variant.
#[component]
pub fn Button(
    variant: ButtonVariant,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(default = "button")] button_type: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type=button_type
            class=variant.classes()
            on:click=move |ev| {
                if let Some(cb) = &on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

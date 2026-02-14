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
            Self::Primary => "inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3.5 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 transition-colors",
            Self::Secondary => "inline-flex items-center gap-1.5 rounded-lg bg-white px-3.5 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-400 transition-colors",
            Self::Danger => "inline-flex items-center gap-1.5 rounded-lg bg-red-600 px-3.5 py-2 text-sm font-medium text-white hover:bg-red-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600 transition-colors",
            Self::DangerOutline => "inline-flex items-center gap-1.5 rounded-lg bg-white px-3.5 py-2 text-sm font-medium text-red-600 border border-red-300 hover:bg-red-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-400 transition-colors",
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

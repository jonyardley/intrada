use leptos::prelude::*;

/// Available icon names for the `Icon` component.
///
/// Lucide-inspired stroke icons: 24×24 viewBox, `stroke="currentColor"`,
/// `stroke-width="2"`, `fill="none"`. Size and colour are inherited from
/// the parent via the `class` prop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
    ArrowLeft,
    ArrowRight,
    Ban,
    Calendar,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    Clock,
    Minus,
    RotateCcw,
    Star,
    Target,
    X,
}

/// Inline SVG icon component.
///
/// Renders a Lucide-style stroke icon at the size of its container.
/// Pass Tailwind sizing classes (e.g. `"w-5 h-5"`) via `class` to control
/// dimensions. The icon inherits text colour via `currentColor`.
#[component]
pub fn Icon(
    name: IconName,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] aria_label: Option<String>,
) -> impl IntoView {
    let aria_hidden = if aria_label.is_some() {
        "false"
    } else {
        "true"
    };

    let paths = match name {
        IconName::ArrowLeft => view! {
            <path d="M19 12H5" />
            <path d="m12 19-7-7 7-7" />
        }
        .into_any(),
        IconName::ArrowRight => view! {
            <path d="M5 12h14" />
            <path d="m12 5 7 7-7 7" />
        }
        .into_any(),
        IconName::Ban => view! {
            <circle cx="12" cy="12" r="10" />
            <path d="m4.9 4.9 14.2 14.2" />
        }
        .into_any(),
        IconName::Calendar => view! {
            <path d="M8 2v4" />
            <path d="M16 2v4" />
            <rect width="18" height="18" x="3" y="4" rx="2" />
            <path d="M3 10h18" />
        }
        .into_any(),
        IconName::Check => view! {
            <path d="M20 6 9 17l-5-5" />
        }
        .into_any(),
        IconName::ChevronDown => view! {
            <path d="m6 9 6 6 6-6" />
        }
        .into_any(),
        IconName::ChevronLeft => view! {
            <path d="m15 18-6-6 6-6" />
        }
        .into_any(),
        IconName::ChevronRight => view! {
            <path d="m9 18 6-6-6-6" />
        }
        .into_any(),
        IconName::ChevronUp => view! {
            <path d="m18 15-6-6-6 6" />
        }
        .into_any(),
        IconName::Clock => view! {
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
        }
        .into_any(),
        IconName::Minus => view! {
            <path d="M5 12h14" />
        }
        .into_any(),
        IconName::RotateCcw => view! {
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
        }
        .into_any(),
        IconName::Star => view! {
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
        }
        .into_any(),
        IconName::Target => view! {
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="12" r="6" />
            <circle cx="12" cy="12" r="2" />
        }
        .into_any(),
        IconName::X => view! {
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
        }
        .into_any(),
    };

    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden=aria_hidden
            aria-label=aria_label
        >
            {paths}
        </svg>
    }
}

use leptos::prelude::*;

/// Severity variants for the Toast notification component.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Info,
    Success,
    Warning,
    Danger,
}

impl ToastVariant {
    fn border_class(self) -> &'static str {
        match self {
            Self::Info => "border-l-4 border-l-info",
            Self::Success => "border-l-4 border-l-success",
            Self::Warning => "border-l-4 border-l-warning",
            Self::Danger => "border-l-4 border-l-danger",
        }
    }

    fn bg_class(self) -> &'static str {
        match self {
            Self::Info => "bg-info-surface",
            Self::Success => "bg-success-surface",
            Self::Warning => "bg-warning-surface",
            Self::Danger => "bg-danger-surface",
        }
    }

    fn text_class(self) -> &'static str {
        match self {
            Self::Info => "text-info-text",
            Self::Success => "text-success-text",
            Self::Warning => "text-warning-text",
            Self::Danger => "text-danger-text",
        }
    }

    fn icon_svg(self) -> &'static str {
        match self {
            Self::Info => {
                r#"<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" fill="none"/><line x1="12" y1="16" x2="12" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><circle cx="12" cy="8" r="0.75" fill="currentColor"/>"#
            }
            Self::Success => {
                r#"<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" fill="none"/><path d="M8 12l3 3 5-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>"#
            }
            Self::Warning => {
                r#"<path d="M12 3L2 21h20L12 3z" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linejoin="round"/><line x1="12" y1="10" x2="12" y2="14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><circle cx="12" cy="17" r="0.75" fill="currentColor"/>"#
            }
            Self::Danger => {
                r#"<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" fill="none"/><line x1="15" y1="9" x2="9" y2="15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><line x1="9" y1="9" x2="15" y2="15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#
            }
        }
    }
}

/// Toast notification component with four severity variants (audit #18).
///
/// Each variant has a distinct left-border colour, background tint,
/// text colour, and icon. Designed for practice context:
/// - Info: "Practice auto-saved"
/// - Success: "5 correct in a row!"
/// - Warning: "Metronome battery low"
/// - Danger: "Failed to save practice"
#[component]
pub fn Toast(variant: ToastVariant, children: Children) -> impl IntoView {
    let container_class = format!(
        "flex items-start gap-3 rounded-lg p-4 {} {}",
        variant.border_class(),
        variant.bg_class()
    );
    let text_class = format!("text-sm {}", variant.text_class());
    let icon_class = format!("shrink-0 w-5 h-5 mt-0.5 {}", variant.text_class());
    let icon_svg = variant.icon_svg();

    view! {
        <div class=container_class role="alert">
            <svg class=icon_class viewBox="0 0 24 24" aria-hidden="true" inner_html=icon_svg></svg>
            <p class=text_class>{children()}</p>
        </div>
    }
}

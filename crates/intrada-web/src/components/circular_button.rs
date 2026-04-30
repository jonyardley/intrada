use intrada_web::haptics;
use leptos::ev;
use leptos::prelude::*;

use crate::components::{Icon, IconName};

/// Size of the circular button. Medium (56px) is the play/pause primary
/// action; Small (44px) is for secondary actions sitting next to it
/// (reset, skip).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CircularButtonSize {
    #[default]
    Medium,
    Small,
}

/// Visual variant. Primary is the accent-filled action (play/pause);
/// Secondary uses the muted surface treatment so the primary anchors
/// the row visually.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CircularButtonVariant {
    #[default]
    Primary,
    Secondary,
}

/// Round icon-only button used for player controls in the 2026 refresh.
///
/// Two sizes (56px primary / 44px secondary) and two visual variants.
/// All interactivity comes for free — light haptic on press, scale
/// feedback via the `:active` CSS rule.
#[component]
pub fn CircularButton(
    /// The icon to render in the centre of the button.
    icon: IconName,
    /// Accessible label — required because the button has no visible text.
    #[prop(into)]
    aria_label: String,
    #[prop(default = CircularButtonSize::Medium)] size: CircularButtonSize,
    #[prop(default = CircularButtonVariant::Primary)] variant: CircularButtonVariant,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let icon_size = match size {
        CircularButtonSize::Medium => "w-6 h-6",
        CircularButtonSize::Small => "w-5 h-5",
    };
    let class = format!(
        "circular-button{}{}",
        match size {
            CircularButtonSize::Medium => "",
            CircularButtonSize::Small => " circular-button--small",
        },
        match variant {
            CircularButtonVariant::Primary => "",
            CircularButtonVariant::Secondary => " circular-button--secondary",
        }
    );

    view! {
        <button
            type="button"
            class=class
            aria-label=aria_label
            disabled=move || disabled.get()
            on:click=move |ev| {
                if !disabled.get() {
                    haptics::haptic_light();
                    if let Some(cb) = &on_click {
                        cb.run(ev);
                    }
                }
            }
        >
            <Icon name=icon class=icon_size />
        </button>
    }
}

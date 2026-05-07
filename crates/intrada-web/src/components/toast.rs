use std::time::Duration;

use leptos::prelude::*;

use crate::components::{Icon, IconName};
use intrada_web::haptics;

/// Auto-dismiss delay for a toast. Long enough for the eye to land on
/// it, short enough not to linger.
const TOAST_DURATION: Duration = Duration::from_millis(2200);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToastKind {
    Success,
}

#[derive(Clone, Debug)]
pub struct ToastEntry {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
}

/// Channel for showing transient "Saved" / status feedback. Provided
/// once at the app root via [`provide_toast`] and consumed anywhere
/// downstream via [`use_toast`].
#[derive(Clone, Copy)]
pub struct ToastChannel {
    items: RwSignal<Vec<ToastEntry>>,
    next_id: StoredValue<u64>,
}

impl ToastChannel {
    /// Fire a success toast with the given message. Pairs with a
    /// success haptic on iOS.
    pub fn show(&self, message: impl Into<String>) {
        self.show_with(message, ToastKind::Success);
    }

    /// Fire a toast of the specified kind.
    pub fn show_with(&self, message: impl Into<String>, kind: ToastKind) {
        let id = self.with_next_id();
        self.items.update(|v| {
            v.push(ToastEntry {
                id,
                message: message.into(),
                kind,
            });
        });

        if matches!(kind, ToastKind::Success) {
            haptics::haptic_success();
        }

        let items = self.items;
        set_timeout(
            move || {
                items.update(|v| v.retain(|t| t.id != id));
            },
            TOAST_DURATION,
        );
    }

    fn with_next_id(&self) -> u64 {
        // `update_value` mutates atomically — two synchronous show()
        // calls cannot observe the same n before either writes back,
        // unlike a separate read + set_value pair.
        self.next_id.update_value(|n| *n += 1);
        self.next_id.get_value()
    }
}

/// Install the toast channel as a Leptos context. Call once from the
/// app root before mounting any view that fires toasts.
pub fn provide_toast() {
    let channel = ToastChannel {
        items: RwSignal::new(Vec::new()),
        next_id: StoredValue::new(0),
    };
    provide_context(channel);
}

/// Look up the current toast channel. Panics if [`provide_toast`]
/// wasn't called above this component in the tree.
pub fn use_toast() -> ToastChannel {
    expect_context::<ToastChannel>()
}

/// Global stack of active toasts. Mount once near the app root —
/// floats above content via fixed positioning.
#[component]
pub fn ToastStack() -> impl IntoView {
    let channel = use_toast();
    view! {
        <div class="toast-stack" aria-live="polite" role="status">
            <For
                each=move || channel.items.get()
                key=|t| t.id
                let:toast
            >
                <ToastItem entry=toast />
            </For>
        </div>
    }
}

#[component]
fn ToastItem(entry: ToastEntry) -> impl IntoView {
    let icon = match entry.kind {
        ToastKind::Success => IconName::Check,
    };
    let kind_attr = match entry.kind {
        ToastKind::Success => "success",
    };
    let message = entry.message.clone();
    view! {
        <div class="toast" data-toast-kind=kind_attr>
            <Icon name=icon class="w-4 h-4" />
            <span>{message}</span>
        </div>
    }
}

use intrada_core::domain::FeatureFlags;
use intrada_core::ViewModel;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::components::SkeletonCardList;

type FlagSelector = fn(&FeatureFlags) -> bool;

/// Wraps a route view so it only renders when a server feature flag is
/// enabled. See `crates/intrada-api/src/routes/features.rs` for the
/// server side.
#[component]
pub fn FeatureGate(select: FlagSelector, children: ChildrenFn) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let children = StoredValue::new(children);

    let state = Signal::derive(move || view_model.with(|vm| vm.features.as_ref().map(&select)));

    // Navigate in an Effect, not the render path — avoids a brief
    // render-then-redirect flash for non-allowlisted users.
    Effect::new(move |_| {
        if state.get() == Some(false) {
            let navigate = use_navigate();
            navigate(
                "/library",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    view! {
        {move || match state.get() {
            Some(true) => children.with_value(|c| c()).into_any(),
            Some(false) => view! { <div></div> }.into_any(),
            None => view! { <SkeletonCardList /> }.into_any(),
        }}
    }
}

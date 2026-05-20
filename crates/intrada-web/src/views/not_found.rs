use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::Icon;

#[component]
pub fn NotFoundView() -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <h2 class="page-title mb-4">"Page Not Found"</h2>
            <p class="text-secondary mb-6">
                "The page you're looking for doesn't exist or may have been moved."
            </p>
            <A href="/library" attr:class="inline-flex items-center gap-2 text-accent-text hover:text-accent-hover font-medium">
                <Icon icon=icondata::LuArrowLeft class="w-4 h-4" />
                "Back to Library"
            </A>
        </div>
    }
}

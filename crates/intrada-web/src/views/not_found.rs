use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::Card;

#[component]
pub fn NotFoundView() -> impl IntoView {
    view! {
        <Card>
            <div class="text-center py-8">
                <h2 class="text-2xl font-bold text-slate-800 mb-4">"Page Not Found"</h2>
                <p class="text-slate-600 mb-6">
                    "The page you're looking for doesn't exist or may have been moved."
                </p>
                <A href="/" attr:class="inline-flex items-center gap-2 text-indigo-600 hover:text-indigo-800 font-medium">
                    "← Back to Library"
                </A>
            </div>
        </Card>
    }
}

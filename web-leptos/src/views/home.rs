use crate::core;
use crate::GlobalState;
use leptos::prelude::*;
use reactive_stores::Store;

#[component]
pub fn Home() -> impl IntoView {
    let state = expect_context::<Store<GlobalState>>();

    let core = state.get().core.clone();
    let (view, render) = signal(core.view());
    let (event, set_event) = signal(shared::Event::Reset);

    Effect::new(move |_| {
        core::update(&core, event.get(), render);
    });

    view! {
        <section>
            <h1>"Home"</h1>
                <p class="is-size-5">{move || view.get().count}</p>
            <div>

                <button class="btn btn-error mr-4"
                    on:click=move |_| set_event.update(|value| *value = shared::Event::Reset)
                >
                    {"Reset"}
                </button>

                <button class="btn btn-success mr-4"
                    on:click=move |_| set_event.update(|value| *value = shared::Event::Increment)
                >
                    {"Increment"}
                </button>

                <button class="btn btn-warning"
                    on:click=move |_| set_event.update(|value| *value = shared::Event::Decrement)
                >
                    {"Decrement"}
                </button>
            </div>
        </section>
    }
}

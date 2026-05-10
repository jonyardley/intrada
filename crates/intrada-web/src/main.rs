mod app;
mod components;
mod views;

use app::App;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
    dismiss_splash();
}

fn dismiss_splash() {
    use leptos::web_sys;
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(el) = document.get_element_by_id("app-splash") else {
        return;
    };
    // Hold the splash for a minimum duration so it reads as intentional
    // branding rather than a flicker. The fade-out starts after the hold,
    // then the element is removed after the CSS transition completes.
    let fade = wasm_bindgen::closure::Closure::once(move || {
        let el_remove = el.clone();
        let _ = el
            .unchecked_ref::<web_sys::HtmlElement>()
            .style()
            .set_property("opacity", "0");
        let remove_cb = wasm_bindgen::closure::Closure::once(move || {
            el_remove.remove();
        });
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                remove_cb.as_ref().unchecked_ref(),
                400,
            );
        }
        remove_cb.forget();
    });
    let _ = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(fade.as_ref().unchecked_ref(), 600);
    fade.forget();
}

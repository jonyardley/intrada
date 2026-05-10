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
    let el_clone = el.clone();
    let _ = el
        .unchecked_ref::<web_sys::HtmlElement>()
        .style()
        .set_property("opacity", "0");
    let cb = wasm_bindgen::closure::Closure::once(move || {
        el_clone.remove();
    });
    let _ = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 350);
    cb.forget();
}

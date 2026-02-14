mod app;
mod components;
mod core_bridge;
mod data;
mod helpers;
mod types;
mod validation;
mod views;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

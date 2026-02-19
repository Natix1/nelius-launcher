#![windows_subsystem = "windows"]

use dioxus::prelude::*;

mod launcher;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx!(
        h1 { "Hello, dioxus" }
    )
}

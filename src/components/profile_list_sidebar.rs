use dioxus::prelude::*;

#[component]
pub fn ProfileListSidebar() -> Element {
    rsx! {
        div {
            class: "w-1/6 px-3 ring-1 ring-white/15 py-4 bg-white/5 rounded-xl h-full space-y-5",
            p {
                class: "opacity-50 font-ligh text-sm text-center",
                "Profiles"
            }
            ul {
                class: "flex flex-col gap-2 w-full items-center"
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn ProfileLogsBox() -> Element {
    rsx! {
        div {
            class: "w-full flex justify-center h-full",
            div {
                class: "w-4/6 h-full bg-white/10 rounded-lg flex flex-col",
                div {
                    class: "flex-1 overflow-y-auto p-4 custom-scrollbar",
                    p {
                        class: "whitespace-pre-wrap break-words w-full h-full",
                    }
                }
            }
        }
    }
}

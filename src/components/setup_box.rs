use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct SetupBoxProps {
    section_title: String,
    children: Element,
}

#[component]
pub fn SetupBox(props: SetupBoxProps) -> Element {
    rsx! {
        div {
            class: "w-3/4 outline-solid p-4 rounded-2xl rounded-lg",
            h1 { class: "text-2xl text-center font-semibold",  "// " {props.section_title} " \\\\" },
            {props.children}
        }
    }
}

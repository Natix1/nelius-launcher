use dioxus::prelude::*;

#[derive(Copy, Clone, PartialEq)]
pub enum NeliusButtonStyle {
    Safe,
    Danger,
    #[allow(dead_code)]
    Warning,
    Disabled,
}

impl NeliusButtonStyle {
    pub fn to_tailwind(&self) -> &'static str {
        match self {
            Self::Safe => "bg-emerald-600 hover:bg-emerald-500 cursor-pointer",
            Self::Danger => "bg-rose-600 hover:bg-rose-500 cursor-pointer",
            Self::Warning => "bg-yellow-700 hover:bg-yellow-600 cursor-pointer",
            Self::Disabled => "bg-white/10",
        }
    }
}

#[component]
pub fn NeliusButton(
    text: ReadSignal<String>,
    style: NeliusButtonStyle,
    icon: Asset,
    disabled: ReadSignal<bool>,
    onclick: EventHandler<Event<MouseData>>,
) -> Element {
    let real_style = use_memo(move || if disabled() { NeliusButtonStyle::Disabled } else { style });

    rsx! {
        button {
            class: format!("h-full rounded-lg {}", real_style().to_tailwind()),
            onclick: move |e| {
                onclick.call(e)
            },
            disabled: disabled(),
            div {
                class: "p-3 flex flex-row gap-1 justify-center items-center",
                img {
                    src: icon,
                    width: 30
                }
                p {
                    class: "",
                    {text}
                }
            }
            rect {
                class: "h-full w-1/6",
            }
        }
    }
}

use dioxus::prelude::*;

const MAX_LOGS_CHARS: usize = 5000;
pub static LAUNCHER_LOGS: GlobalSignal<String> = Signal::global(|| String::with_capacity(MAX_LOGS_CHARS));

pub fn log(message: impl Into<String>) {
    let message = message.into();
    println!("{}", &message);

    LAUNCHER_LOGS.with_mut(|logs| {
        logs.push_str(&message);
        logs.push('\n');

        if logs.len() > MAX_LOGS_CHARS {
            let truncate_to = logs.len() - MAX_LOGS_CHARS;
            logs.drain(..truncate_to);
        }
    });
}

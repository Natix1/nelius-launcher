use std::sync::OnceLock;

use reqwest::Client;

static REQWEST_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_reqwest_client() -> &'static Client {
    REQWEST_CLIENT
        .get_or_init(|| Client::builder().user_agent("nelius-mc/1.0").build().expect("the client to be built"))
}

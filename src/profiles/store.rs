use dioxus::prelude::*;
use std::collections::HashMap;

use crate::profiles::Profile;

#[derive(Debug, Clone)]
pub struct ProfileStore {
    pub profiles: Signal<HashMap<String, Signal<Profile>>>,
}

impl ProfileStore {
    pub fn new() -> Self {
        ProfileStore { profiles: Signal::new(HashMap::new()) }
    }

    fn read(&self, version_id: String) -> Option<Signal<Profile>> {
        self.profiles.peek().get(&version_id).copied()
    }

    pub fn write(&self, version_id: String, mutate: impl FnOnce(&mut Profile)) {
        let signal = self.profiles.peek().get(&version_id).copied();
        if let Some(mut signal) = signal {
            mutate(&mut signal.write());
            // save here
        }
    }
}

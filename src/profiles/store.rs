use anyhow::bail;
use dioxus::prelude::*;
use serde_json::Value;
use std::{collections::HashMap, fs};

use crate::{launcher::directories, profiles::Profile};

#[derive(Debug, Clone, Copy)]
pub struct ProfileStore {
    pub selected_profile_name: Signal<Option<String>>,
    pub profiles: Signal<HashMap<String, Signal<Profile>>>,
}

impl Default for ProfileStore {
    fn default() -> Self {
        ProfileStore {
            profiles: Signal::new_in_scope(HashMap::new(), ScopeId::ROOT),
            selected_profile_name: Signal::new_in_scope(None, ScopeId::ROOT),
        }
    }
}

impl ProfileStore {
    pub fn load() -> Self {
        let config_path = &directories::get_directories().config_file;
        let contents = match fs::read(config_path) {
            Ok(v) => v,
            Err(_) => {
                return Self::default();
            }
        };

        let data: Value = match serde_json::from_slice(contents.as_slice()) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };

        let profiles_object = match data.as_object() {
            Some(v) => v,
            None => return Self::default(),
        };
        let mut profiles = HashMap::new();

        for (name, data) in profiles_object {
            match serde_json::from_value::<Profile>(data.clone()) {
                Ok(profile) => profiles.insert(name.to_owned(), Signal::new_in_scope(profile, ScopeId::ROOT)),
                Err(_) => return Self::default(),
            };
        }

        ProfileStore { profiles: Signal::new_in_scope(profiles, ScopeId::ROOT), ..Default::default() }
    }

    pub fn peek(&self, profile_name: String) -> Option<Signal<Profile>> {
        self.profiles.peek().get(&profile_name).copied()
    }

    pub fn write(&self, profile_name: String, mutate: impl FnOnce(&mut Profile)) {
        let signal = self.profiles.read().get(&profile_name).copied();
        if let Some(mut signal) = signal {
            mutate(&mut signal.write());
            // save here
        }
    }

    pub fn exists_with_name(&self, profile_name: &String) -> bool {
        self.profiles.read().contains_key(profile_name)
    }

    pub fn add(&mut self, profile: Profile) -> anyhow::Result<()> {
        self.profiles.write().insert(profile.profile_name.clone(), Signal::new_in_scope(profile, ScopeId::ROOT));
        Ok(())
    }

    pub fn remove(&mut self, profile_name: String) -> anyhow::Result<()> {
        let selected = self.selected_profile_name.peek().to_owned();
        if let Some(selected) = selected {
            if selected == profile_name {
                self.selected_profile_name.set(None);
            }
        }

        if let Some(_) = self.profiles.write().remove(&profile_name) {
            Ok(())
        } else {
            bail!("This profile did not exist.")
        }
    }
}

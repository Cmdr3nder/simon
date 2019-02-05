use std::cmp::Ordering;
use std::collections::HashMap;
use config::{Config, File};

#[derive(Debug, Deserialize, Eq)]
pub struct TabSettings<'a> {
    pub name: &'a str,
    pub kind: &'a str,
    pub priority: usize,
    pub media_dirs: Option<Vec<&'a str>>,
    pub media_types: Option<Vec<&'a str>>,
    pub subs_dirs: Option<Vec<&'a str>>,
    pub subs_types: Option<Vec<&'a str>>,
}

impl<'a> Ord for TabSettings<'a> {
    fn cmp(&self, other: &TabSettings) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl<'a> PartialOrd for TabSettings<'a> {
    fn partial_cmp(&self, other: &TabSettings) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for TabSettings<'a> {
    fn eq(&self, other: &TabSettings) -> bool {
        self.priority == other.priority
    }
}

pub fn settings_from_file(settings_file: &str) -> Vec<&TabSettings> {
    let mut settings = Config::default();
    settings.merge(File::with_name(settings_file));
    let settings = settings.try_into::<HashMap<String, TabSettings>>().unwrap();

    let mut settings: Vec<&TabSettings> = settings.values().collect();
    settings.sort();

    settings
}

use config::{Config, File};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Eq)]
pub struct TabSettings {
    pub name: String,
    pub kind: String,
    pub priority: usize,
    pub media_dirs: Option<Vec<String>>,
    pub media_types: Option<Vec<String>>,
    pub subs_dirs: Option<Vec<String>>,
    pub subs_types: Option<Vec<String>>,
}

impl Ord for TabSettings {
    fn cmp(&self, other: &TabSettings) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for TabSettings {
    fn partial_cmp(&self, other: &TabSettings) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TabSettings {
    fn eq(&self, other: &TabSettings) -> bool {
        self.priority == other.priority
    }
}

pub fn settings_from_file(settings_file: &str) -> Vec<TabSettings> {
    let mut settings = Config::default();
    settings.merge(File::with_name(settings_file)).unwrap();
    let settings = settings.try_into::<HashMap<String, TabSettings>>().unwrap();

    let mut settings: Vec<TabSettings> = settings.values().map(|r| r.clone()).collect();
    settings.sort();

    settings
}

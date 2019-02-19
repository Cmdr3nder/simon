use config::{Config, File};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::Into;
use tui::style::Color;

#[derive(Clone, Debug, Deserialize, Eq)]
pub struct TabSettings {
    pub name: String,
    pub kind: String,
    pub priority: usize,
    pub media_dirs: Option<Vec<String>>,
    pub media_types: Option<Vec<String>>,
    pub subs_dirs: Option<Vec<String>>,
    pub subs_types: Option<Vec<String>>,
    pub base_color: Option<ColorSetting>,
    pub highlight_color: Option<ColorSetting>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum ColorSetting {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
}

impl Into<Color> for ColorSetting {
    fn into(self) -> Color {
        match self {
            ColorSetting::Reset => Color::Reset,
            ColorSetting::Black => Color::Black,
            ColorSetting::Red => Color::Red,
            ColorSetting::Green => Color::Green,
            ColorSetting::Yellow => Color::Yellow,
            ColorSetting::Blue => Color::Blue,
            ColorSetting::Magenta => Color::Magenta,
            ColorSetting::Cyan => Color::Cyan,
            ColorSetting::Gray => Color::Gray,
            ColorSetting::DarkGray => Color::DarkGray,
            ColorSetting::LightRed => Color::LightRed,
            ColorSetting::LightGreen => Color::LightGreen,
            ColorSetting::LightYellow => Color::LightYellow,
            ColorSetting::LightBlue => Color::LightBlue,
            ColorSetting::LightMagenta => Color::LightMagenta,
            ColorSetting::LightCyan => Color::LightCyan,
            ColorSetting::White => Color::White,
            ColorSetting::Rgb(r, g, b) => Color::Rgb(r, g, b),
        }
    }
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

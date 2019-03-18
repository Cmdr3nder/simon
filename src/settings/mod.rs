use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::Into;
use std::path::Path;

use directories::ProjectDirs;
use serde::Deserialize;
use tui::style::Color;

#[derive(Debug)]
pub enum Error {
    Find,
    Read(config::ConfigError),
    Content(config::ConfigError),
}

#[derive(Clone, Debug, Deserialize, Eq)]
pub struct TabSettings {
    pub name: String,
    pub kind: String,
    pub priority: usize,
    pub media_dirs: Option<Vec<String>>,
    pub media_types: Option<Vec<String>>,
    pub subs_dirs: Option<Vec<String>>,
    pub subs_types: Option<Vec<String>>,
    pub command: Option<CommandSetting>,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CommandSetting {
    pub program: String,
    pub args: Vec<String>,
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
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => self.name.cmp(&other.name),
            x => x,
        }
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

pub fn get_settings() -> Result<Vec<TabSettings>, Error> {
    match ProjectDirs::from("com", "agamecoder", "simon") {
        Some(dirs) => {
            let mut config = dirs.config_dir().to_path_buf();
            config.push("simon.config.toml");

            if config.exists() {
                read_config(config.as_path())
            } else {
                Err(Error::Find)
            }
        }
        None => Err(Error::Find),
    }
}

fn read_underlying_config(file: &Path) -> Result<HashMap<String, TabSettings>, Error> {
    let mut settings = config::Config::default();
    match settings.merge(config::File::from(file)) {
        Ok(_) => settings
            .try_into::<HashMap<String, TabSettings>>()
            .map_err(|err| Error::Content(err)),
        Err(err) => Err(Error::Read(err)),
    }
}

fn read_config(file: &Path) -> Result<Vec<TabSettings>, Error> {
    let config = read_underlying_config(file)?;

    let mut settings: Vec<TabSettings> = config.values().map(|r| r.clone()).collect();
    settings.sort();

    Ok(settings)
}

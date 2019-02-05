#![feature(try_from)]

extern crate config;

#[macro_use]
extern crate serde_derive;

mod util;

use std::io;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use config::{Config, File};
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{
    Block, Borders, SelectableList, Tabs, Widget,
};
use tui::{Frame, Terminal};

use crate::util::event::{Event, Events};

#[derive(Debug, Deserialize)]
pub struct TabSettings {
    name: String,
    kind: String,
    priority: usize,
    media_dirs: Option<Vec<String>>,
    media_types: Option<Vec<String>>,
    subs_dirs: Option<Vec<String>>,
    subs_types: Option<Vec<String>>,
}

pub struct Tab {
    settings: TabSettings,
    media: Option<SelectLoop<PathBuf>>,
    subs: Option<SelectLoop<PathBuf>>,
}

struct App {
    tabs: SelectLoop<Tab>,
}

struct SelectLoop<T> {
    items: Vec<T>,
    selected: usize,
}

impl<T> SelectLoop<T> {
    pub fn next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn get(&self) {
        self.items[self.selected]
    }
}

fn main() -> Result<(), failure::Error> {
    let mut settings = Config::default();
    settings
        .merge(File::with_name("conf/simon.config.toml")).unwrap();

    let settings = settings.try_into::<HashMap<String, TabSettings>>().unwrap();

    let mut app = build_app(settings);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size()?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let events = Events::new();

    /*let video_path = Path::new("/mnt/Data/Videos");
    let video = find_files(&video_path, &|entry| {
        match entry.path().extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "mp4" => true,
                "mkv" => true,
                "wmv" => true,
                _ => false
            },
            None => false
        }
    });
    let video: Vec<MediaItem> = video.iter().map(|path| MediaItem {
        name: path.file_name().unwrap().to_str().unwrap(),
        path: path.to_str().unwrap(),
    }).collect();*/

    loop {
        // Draw UI
        terminal.draw(|mut f| {
            let tab_titles: Vec<&str> = app.tabs.items.iter().map(|tab| tab.settings.name.as_str()).collect();

            let chunks = Layout::default()
                .constraints([Constraint::Length(u16::try_from(tab_titles.len()).unwrap()), Constraint::Min(0)].as_ref())
                .split(size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Simon"))
                .titles(&tab_titles)
                .style(Style::default().fg(Color::Green))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.tabs.selected)
                .render(&mut f, chunks[0]);
            draw_media_page(&mut f, &app, chunks[1]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Esc => {
                    break;
                }
                Key::Char('q') => {
                    break;
                }
                Key::Up => {
                }
                Key::Down => {
                }
                Key::Left => {
                    app.tabs.previous();
                }
                Key::Right => {
                    app.tabs.next();
                }
                _ => {}
            },
            Event::Tick => {
            }
        }
    }
    Ok(())
}

fn build_app(settings: HashMap<String, TabSettings>) -> App {
    App {
        tabs: SelectLoop {
            items: vec![],
            selected: 0,
        }
    }
}

fn visit_files<F>(dir: &Path, cb: &mut F) -> io::Result<()>
where
    F: FnMut(&DirEntry)
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_files(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }

    Ok(())
}

fn find_files(dir: &Path, filter: &Fn(&DirEntry) -> bool) -> Vec<PathBuf> {
    let mut result = Vec::new();

    match visit_files(dir, &mut |file_entry| {
        if filter(file_entry) {
            result.push(file_entry.path());
        }
    }) {
        Ok(_) => result,
        Err(_) => vec![] // TODO: Log this error.
    }
}

fn increment_wrap(current: usize, length: usize) -> usize {
    if current == length - 1 {
        return 0;
    }

    current + 1
}

fn decrement_wrap(current: usize, length: usize) -> usize {
    if current == 0 {
        return length - 1;
    }

    current - 1
}

fn draw_media_page<B>(f: &mut Frame<B>, tab: &Tab, area: Rect)
where
    B: Backend,
{
    let names: Vec<&str> = tab.media.items.iter();

    let selected = match app.tabs.index {
        0 => Some(app.selected_video),
        1 => Some(app.selected_audio),
        _ => None,
    };

    let title = match app.tabs.index {
        0 => "Video Files",
        1 => "Audio Files",
        _ => "Unknown",
    };

    SelectableList::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .items(&names)
        .select(selected)
        .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::Bold))
        .highlight_symbol(">")
        .render(f, area);
}


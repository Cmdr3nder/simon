#![feature(try_from)]

extern crate config;

#[macro_use]
extern crate serde_derive;

mod util;
mod settings;

use std::io;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{
    Block, Borders, Paragraph, SelectableList, Tabs, Text, Widget,
};
use tui::{Frame, Terminal};

use crate::settings::{TabSettings, settings_from_file};
use crate::util::SelectLoop;
use crate::util::event::{Event, Events};

#[derive(Debug)]
enum TabType {
    Media(MediaTab),
    Unknown
}

#[derive(Debug)]
struct Tab {
    name: String,
    tab_type: TabType,
}

#[derive(Debug)]
struct MediaTab {
    media: SelectLoop<PathBuf>,
    subs: Option<SelectLoop<PathBuf>>,
}

#[derive(Debug)]
struct App {
    tabs: SelectLoop<Tab>,
}

fn main() -> Result<(), failure::Error> {
    let settings = settings_from_file("conf/simon.config.toml");
    let mut app = build_app(settings);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size: Rect = terminal.size()?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let events = Events::new();

    loop {
        // Draw UI
        terminal.draw(|mut f| {
            let tab_titles: Vec<&str> = app.tabs.items.iter().map(|tab| tab.name.as_str()).collect();

            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Simon"))
                .titles(&tab_titles)
                .style(Style::default().fg(Color::Green))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.tabs.index)
                .render(&mut f, chunks[0]);

            match &app.tabs.current().tab_type {
                TabType::Media(media_tab) => draw_media_page(&mut f, &app.tabs.current(), &media_tab, chunks[1]),
                _ => draw_blank_page(&mut f, chunks[1])
            };
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

    terminal.clear()?;

    Ok(())
}

fn build_app(settings: Vec<TabSettings>) -> App {
    App {
        tabs: SelectLoop {
            items: settings.iter().map(|settings| {
                Tab {
                    name: settings.name.clone(),
                    tab_type: match settings.kind.as_str() {
                        "media" => TabType::Media(build_media(settings)),
                        _ => TabType::Unknown,
                    }
                }
            }).collect(),
            index: 0,
        }
    }
}

fn build_media(settings: &TabSettings) -> MediaTab {
    let mut media: Vec<PathBuf> = Vec::new();
    let media_types: &Vec<String> = match &settings.media_types {
        Some(types) => types,
        None => panic!("Configuration error for {}, you must provide media_types for a type=\"media\" tab", settings.name)
    };

    match &settings.media_dirs {
        Some(dirs) => {
            for dir in dirs {
                let mut files = find_files(Path::new(dir), &|entry| {
                    match entry.path().extension() {
                        Some(ext) => match ext.to_str() {
                            Some(ext) => media_types.contains(&String::from(ext)),
                            None => false,
                        },
                        None => false,
                    }
                });

                media.append(&mut files);
            }
        },
        None => panic!("Configuration error for {}, you must provide media_dirs for a type=\"media\" tab", settings.name)
    };

    MediaTab {
        media: SelectLoop::new(media),
        subs: None
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

fn draw_media_page<B>(f: &mut Frame<B>, tab: &Tab, media_tab: &MediaTab, area: Rect)
where
    B: Backend,
{
    let media_names: Vec<&str> = media_tab.media.items.iter().map(|path_buf| path_buf.to_str().unwrap()).collect(); // TODO: Remove unwrap and panic if we can't render
    let mut zone = area;

    match &media_tab.subs {
        Some(subs) => {
            let mut subs_names: Vec<&str> = subs.items.iter().map(|path_buf| path_buf.to_str().unwrap()).collect(); // TODO: Remove unwrap and panic if we can't render

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(area);

            zone = chunks[0];

            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Subtitles"))
                .items(&subs_names)
                .select(Some(subs.index))
                .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::Bold))
                .highlight_symbol(">")
                .render(f, chunks[1]);
        },
        None => {},
    };

    SelectableList::default()
        .block(Block::default().borders(Borders::ALL).title("Media"))
        .items(&media_names)
        .select(Some(media_tab.media.index))
        .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::Bold))
        .highlight_symbol(">")
        .render(f, area);
}

fn draw_blank_page<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = [
        Text::raw("Wait how did you do that?\n"),
    ];

    Paragraph::new(text.iter())
        .block(Block::default().borders(Borders::ALL))
        .render(f, area);
}


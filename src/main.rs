#![feature(try_from)]

extern crate config;

#[macro_use]
extern crate serde_derive;

mod util;
mod settings;

use std::io;
use std::convert::TryFrom;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Layout, Rect};
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
    Download
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
    //let stdout = AlternateScreen::from(stdout);
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
                    tab_type: TabType::Download
                }
            }).collect(),
            index: 0,
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

fn draw_media_page<B>(f: &mut Frame<B>, tab: &Tab, media_tab: &MediaTab, area: Rect)
where
    B: Backend,
{
    let media_names: Vec<&str> = media_tab.media.items.iter().map(|path_buf| path_buf.to_str().unwrap()).collect();

    match &media_tab.subs {
        Some(subs) => {
            let subs_names: Vec<&str> = subs.items.iter().map(|path_buf| path_buf.to_str().unwrap()).collect();
        },
        None => {},
    };

    SelectableList::default()
        .block(Block::default().borders(Borders::ALL).title(&tab.name))
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


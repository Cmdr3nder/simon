extern crate config;

#[macro_use]
extern crate serde_derive;

mod settings;
mod util;

use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Tabs, Text, Widget};
use tui::{Frame, Terminal};

use crate::settings::{settings_from_file, CommandSetting, TabSettings};
use crate::util::event::{Event, Events};
use crate::util::SelectLoop;

#[derive(Debug)]
enum TabType {
    Media(MediaTab),
    Unknown,
}

#[derive(Debug)]
struct Tab {
    base_color: Color,
    hightlight_color: Color,
    name: String,
    tab_type: TabType,
}

#[derive(Debug)]
enum MediaCursor {
    MediaListOut,
    MediaListIn,
    SubsListOut,
    SubsListIn,
}

#[derive(Debug)]
struct MediaTab {
    command: CommandSetting,
    cursor: MediaCursor,
    media: SelectLoop<PathBuf>,
    subs: Option<SelectLoop<PathBuf>>,
}

#[derive(Debug)]
enum AppCursor {
    TabList,
    TabContents,
}

#[derive(Debug)]
struct App {
    cursor: AppCursor,
    tabs: SelectLoop<Tab>,
}

enum ProgramStatus {
    Quit,
    Resume,
    Refresh,
}

fn main() -> Result<(), failure::Error> {
    let settings = settings_from_file("conf/simon.config.toml");
    let mut app = build_app(settings);

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut size: Rect = terminal.size()?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let mut events = Events::new();

    loop {
        let new_size = terminal.size()?;

        if new_size != size {
            terminal.resize(new_size)?;
            size = new_size;
        }

        // Draw UI
        terminal.draw(|mut f| {
            let tab_titles: Vec<&str> =
                app.tabs.items.iter().map(|tab| tab.name.as_str()).collect();

            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let border_color = match app.cursor {
                AppCursor::TabList => app.tabs.current().hightlight_color,
                _ => app.tabs.current().base_color,
            };

            Tabs::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .title_style(Style::default().fg(border_color))
                        .title("Simon"),
                )
                .titles(&tab_titles)
                .style(Style::default().fg(app.tabs.current().base_color))
                .highlight_style(Style::default().fg(app.tabs.current().hightlight_color))
                .select(app.tabs.index)
                .render(&mut f, chunks[0]);

            let contents_have_focus = match app.cursor {
                AppCursor::TabContents => true,
                _ => false,
            };

            match &app.tabs.current().tab_type {
                TabType::Media(media_tab) => draw_media_page(
                    &mut f,
                    &app.tabs.current(),
                    &media_tab,
                    contents_have_focus,
                    chunks[1],
                ),
                _ => draw_blank_page(&mut f, chunks[1]),
            };
        })?;

        match handle_input(&mut app, events.next()?) {
            ProgramStatus::Quit => {
                break;
            }
            ProgramStatus::Refresh => {
                events.stop();
                events = Events::new();
                terminal.resize(terminal.size()?)?;
                terminal.flush()?;
            }
            _ => {}
        }
    }

    terminal.clear()?;

    Ok(())
}

fn handle_input(app: &mut App, event: Event<Key>) -> ProgramStatus {
    match app.cursor {
        AppCursor::TabList => match event {
            Event::Input(input) => match input {
                Key::Esc => ProgramStatus::Quit,
                Key::Char('q') => ProgramStatus::Quit,
                Key::Left => {
                    app.tabs.previous();
                    ProgramStatus::Resume
                }
                Key::Right => {
                    app.tabs.next();
                    ProgramStatus::Resume
                }
                Key::Down => {
                    app.cursor = AppCursor::TabContents;
                    ProgramStatus::Resume
                }
                _ => ProgramStatus::Resume,
            },
            _ => ProgramStatus::Resume,
        },
        AppCursor::TabContents => match event {
            Event::Input(input) => match input {
                Key::Esc => ProgramStatus::Quit,
                Key::Char('q') => ProgramStatus::Quit,
                x => match handle_tab_input(app.tabs.current_mut(), x) {
                    Some(input) => match input {
                        Key::Up => {
                            app.cursor = AppCursor::TabList;
                            ProgramStatus::Resume
                        }
                        Key::Char('r') => ProgramStatus::Refresh,
                        _ => ProgramStatus::Resume,
                    },
                    None => ProgramStatus::Resume,
                },
            },
            _ => ProgramStatus::Resume,
        },
    }
}

fn handle_tab_input(tab: &mut Tab, key: Key) -> Option<Key> {
    match &mut tab.tab_type {
        TabType::Media(media_tab) => handle_media_tab_input(media_tab, key),
        _ => Some(key),
    }
}

fn handle_media_tab_input(media_tab: &mut MediaTab, key: Key) -> Option<Key> {
    match media_tab.cursor {
        MediaCursor::MediaListOut => match key {
            Key::Up => Some(key),
            Key::Char('\n') => {
                media_tab.cursor = MediaCursor::MediaListIn;
                None
            }
            Key::Char('p') => play_media(media_tab),
            _ => None,
        },
        MediaCursor::MediaListIn => match key {
            Key::Up => {
                media_tab.media.previous();
                None
            }
            Key::Down => {
                media_tab.media.next();
                None
            }
            Key::Char('p') => play_media(media_tab),
            Key::Char('\n') => {
                media_tab.cursor = MediaCursor::MediaListOut;
                None
            }
            _ => None,
        },
        _ => None,
    }
}

fn play_media(media_tab: &MediaTab) -> Option<Key> {
    let args: Vec<&str> = media_tab
        .command
        .args
        .iter()
        .map(|arg| match arg.as_str() {
            "{0}" => media_tab
                .media
                .current()
                .to_str()
                .expect("Path to media should be valid UTF-8"),
            s => s,
        })
        .collect();

    Command::new(media_tab.command.program.as_str())
        .args(&args)
        .spawn()
        .expect("Spawn media process correctly")
        .wait()
        .expect("Should wait on media process correctly");

    Some(Key::Char('r'))
}

fn build_app(settings: Vec<TabSettings>) -> App {
    App {
        cursor: AppCursor::TabList,
        tabs: SelectLoop {
            items: settings
                .iter()
                .map(|settings| {
                    let base_color = match settings.base_color {
                        Some(color) => color.into(),
                        None => Color::White,
                    };

                    let hightlight_color = match settings.highlight_color {
                        Some(color) => color.into(),
                        None => Color::Yellow,
                    };

                    Tab {
                        base_color,
                        hightlight_color,
                        name: settings.name.clone(),
                        tab_type: match settings.kind.as_str() {
                            "media" => TabType::Media(build_media(settings)),
                            _ => TabType::Unknown,
                        },
                    }
                })
                .collect(),
            index: 0,
        },
    }
}

fn build_media(settings: &TabSettings) -> MediaTab {
    let mut media: Vec<PathBuf> = Vec::new();
    let media_types: &Vec<String> = match &settings.media_types {
        Some(types) => types,
        None => panic!(
            "Configuration error for {}, you must provide media_types for a type=\"media\" tab",
            settings.name
        ),
    };

    match &settings.media_dirs {
        Some(dirs) => {
            for dir in dirs {
                let mut files =
                    find_files(Path::new(dir), &|entry| match entry.path().extension() {
                        Some(ext) => match ext.to_str() {
                            Some(ext) => media_types.contains(&String::from(ext)),
                            None => false,
                        },
                        None => false,
                    });

                media.append(&mut files);
            }
        }
        None => panic!(
            "Configuration error for {}, you must provide media_dirs for a type=\"media\" tab",
            settings.name
        ),
    };

    let command = match &settings.command {
        Some(command) => command.clone(),
        None => panic!(
            "Configuration error for {}, you must provide command for a type=\"media\" tab",
            settings.name
        ),
    };

    MediaTab {
        command,
        cursor: MediaCursor::MediaListOut,
        media: SelectLoop::new(media),
        subs: None,
    }
}

fn visit_files<F>(dir: &Path, cb: &mut F) -> io::Result<()>
where
    F: FnMut(&DirEntry),
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
        Err(_) => vec![], // TODO: Log this error.
    }
}

fn draw_media_page<B>(
    f: &mut Frame<B>,
    tab: &Tab,
    media_tab: &MediaTab,
    has_focus: bool,
    area: Rect,
) where
    B: Backend,
{
    let media_names: Vec<&str> = media_tab
        .media
        .items
        .iter()
        .map(|path_buf| path_buf.file_name().unwrap().to_str().unwrap())
        .collect(); // TODO: Remove unwrap and panic if we can't render
    let mut zone = area;

    match &media_tab.subs {
        Some(subs) => {
            let subs_names: Vec<&str> = subs
                .items
                .iter()
                .map(|path_buf| path_buf.file_name().unwrap().to_str().unwrap())
                .collect(); // TODO: Remove unwrap and panic if we can't render

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

            zone = chunks[0];

            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Subtitles"))
                .items(&subs_names)
                .select(Some(subs.index))
                .style(Style::default().fg(tab.base_color))
                .highlight_style(
                    Style::default()
                        .fg(tab.hightlight_color)
                        .modifier(Modifier::Bold),
                )
                .highlight_symbol(">")
                .render(f, chunks[1]);
        }
        None => {}
    };

    let border_color = match has_focus {
        true => tab.hightlight_color,
        false => tab.base_color,
    };

    let highlight_color = match media_tab.cursor {
        MediaCursor::MediaListIn => tab.hightlight_color,
        _ => tab.base_color,
    };

    SelectableList::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title_style(Style::default().fg(highlight_color))
                .title("Media"),
        )
        .items(&media_names)
        .select(Some(media_tab.media.index))
        .style(Style::default().fg(tab.base_color))
        .highlight_style(
            Style::default()
                .fg(highlight_color)
                .modifier(Modifier::Bold),
        )
        .highlight_symbol(">")
        .render(f, zone);
}

fn draw_blank_page<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = [Text::raw("Wait how did you do that?\n")];

    Paragraph::new(text.iter())
        .block(Block::default().borders(Borders::ALL))
        .render(f, area);
}

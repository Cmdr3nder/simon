#[allow(dead_code)]
mod util;

use std::io;

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

use crate::util::event::{Event, Events};
use crate::util::TabsState;

struct Server<'a> {
    name: &'a str,
    location: &'a str,
    coords: (f64, f64),
    status: &'a str,
}

struct MediaItem<'a> {
    name: &'a str,
    path: &'a str,
}

struct App<'a> {
    tabs: TabsState<'a>,
    video: Vec<MediaItem<'a>>,
    selected_video: usize,
    audio: Vec<MediaItem<'a>>,
    selected_audio: usize,
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size()?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let events = Events::new();

    let mut app = App {
        tabs: TabsState::new(vec!["Video", "Audio"]),
        video: vec![
            MediaItem {
                name: "Good Eats 01-01 'Crabby Talk'",
                path: "/home/andrew/media/video/good-eats/Good Eats 01-01 'Crabby Talk'.mp4"
            },
            MediaItem {
                name: "Good Eats 01-02 'Lobster Talk'",
                path: "/home/andrew/media/video/good-eats/Good Eats 01-02 'Lobster Talk'.mp4"
            },
            MediaItem {
                name: "Good Eats 01-03 'Ice Cream You Scream'",
                path: "/home/andrew/media/video/good-eats/Good Eats 01-02 'Lobster Talk'.mp4"
            },
        ],
        selected_video: 0,
        audio: vec![
            MediaItem {
                name: "TwentySided_035",
                path: "/home/andrew/media/audio/TwentySided_035.ogg"
            },
            MediaItem {
                name: "Lords_Of_The_Ring",
                path: "/home/andrew/media/audio/Lords_Of_The_Ring.ogg"
            },
            MediaItem {
                name: "Ten_Thousand",
                path: "/home/andrew/media/audio/Ten_Thousand_Fists.ogg"
            },
        ],
        selected_audio: 0,
    };

    loop {
        // Draw UI
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Simon"))
                .titles(&app.tabs.titles)
                .style(Style::default().fg(Color::Green))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.tabs.index)
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
                    match app.tabs.index {
                        0 => app.selected_video = decrement_wrap(app.selected_video, app.video.len()),
                        1 => app.selected_audio = decrement_wrap(app.selected_audio, app.audio.len()),
                        _ => {}
                    };
                }
                Key::Down => {
                    match app.tabs.index {
                        0 => app.selected_video = increment_wrap(app.selected_video, app.video.len()),
                        1 => app.selected_audio = increment_wrap(app.selected_audio, app.audio.len()),
                        _ => {}
                    };
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

fn increment_wrap(current: usize, length: usize) -> usize {
    if (current == length - 1) {
        return 0;
    }

    current + 1
}

fn decrement_wrap(current: usize, length: usize) -> usize {
    if (current == 0) {
        return length - 1;
    }

    current - 1
}

fn draw_media_page<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let names: Vec<&str> = match app.tabs.index {
        0 => app.video.iter().map(|x| x.name).collect(),
        1 => app.audio.iter().map(|x| x.name).collect(),
        _ => vec![],
    };

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

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = [
        Text::raw("This is a paragraph with several lines. You can change style your text the way you want.\n\nFox example: "),
        Text::styled("under", Style::default().fg(Color::Red)),
        Text::raw(" "),
        Text::styled("the", Style::default().fg(Color::Green)),
        Text::raw(" "),
        Text::styled("rainbow", Style::default().fg(Color::Blue)),
        Text::raw(".\nOh and if you didn't "),
        Text::styled("notice", Style::default().modifier(Modifier::Italic)),
        Text::raw(" you can "),
        Text::styled("automatically", Style::default().modifier(Modifier::Bold)),
        Text::raw(" "),
        Text::styled("wrap", Style::default().modifier(Modifier::Invert)),
        Text::raw(" your "),
        Text::styled("text", Style::default().modifier(Modifier::Underline)),
        Text::raw(".\nOne more thing is that it should display unicode characters: 10€")
    ];
    Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Footer")
                .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::Bold)),
        )
        .wrap(true)
        .render(f, area);
}

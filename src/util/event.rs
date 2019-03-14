use std::io;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {}

impl Events {
    pub fn new() -> Events {
        Events {}
    }

    pub fn next(&self) -> Result<Event<Key>, io::Error> {
        let key = io::stdin().keys().next();

        match key {
            Some(k) => k.map(|key| Event::Input(key)),
            None => Ok(Event::Tick),
        }
    }

    pub fn stop(self) {}
}

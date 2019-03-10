use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    input_ctrl_tx: mpsc::Sender<bool>,
    tick_handle: thread::JoinHandle<()>,
    tick_ctrl_tx: mpsc::Sender<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let (input_ctrl_tx, input_ctrl_rx) = mpsc::channel();
        let (tick_ctrl_tx, tick_ctrl_rx) = mpsc::channel();

        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                let mut keys = stdin.keys();

                while input_ctrl_rx.try_recv().is_err() {
                    let key = keys.next();

                    match key {
                        Some(evt) => match evt {
                            Ok(key) => {
                                if let Err(_) = tx.send(Event::Input(key)) {
                                    return;
                                }
                                if key == config.exit_key {
                                    return;
                                }
                            }
                            Err(_) => {}
                        },
                        None => {}
                    }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    if tick_ctrl_rx.try_recv().is_err() {
                        tx.send(Event::Tick).unwrap();
                        thread::sleep(config.tick_rate);
                    } else {
                        return;
                    }
                }
            })
        };
        Events {
            rx,
            input_handle,
            input_ctrl_tx,
            tick_handle,
            tick_ctrl_tx,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn stop(self) {
        match self.input_ctrl_tx.send(true) {
            Ok(_) => self
                .input_handle
                .join()
                .expect("Couldn't join on input_handle thread"),
            Err(_) => {}, //panic!("Couldn't stop the input_handle thread → {:?}", err),
        }

        match self.tick_ctrl_tx.send(true) {
            Ok(_) => self
                .tick_handle
                .join()
                .expect("Couldn't join on tick_handle thread"),
            Err(_) => {}, //panic!("Couldn't stop the tick_handle thread → {:?}", err),
        }
    }
}

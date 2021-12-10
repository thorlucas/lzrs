use std::{sync::mpsc::{Receiver, self, Sender},io::{self, Result}, thread};

use termion::{input::TermRead, event::Key};
use tracing::error;

pub enum Event {
    Tick,
    Key(Key),
    LoadDictBuffer {
        buf: &'static [u8],
        head: usize,
    }
}

pub fn start_event_loop(tx: Sender<Event>) {
    let keys_tx = tx.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
            if let Ok(evt) = evt {
                if let Err(err) = keys_tx.send(Event::Key(evt)) {
                    error!("{}", err);
                    return;
                }
            }
        }
    });
    let tick_tx = tx;
    thread::spawn(move || {
        loop {
            #[allow(deprecated)]
            thread::sleep_ms(250);
            tick_tx.send(Event::Tick).unwrap();
        }
    });
}

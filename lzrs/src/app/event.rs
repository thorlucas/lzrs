use std::{sync::mpsc::{Receiver, self},io::{self, Result}, thread};

use termion::{input::TermRead, event::Key};
use tracing::error;

pub fn start_event_loop() -> Result<Receiver<Key>> {
    let (tx, rx) = mpsc::channel();
    let keys_tx = tx.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
            if let Ok(evt) = evt {
                if let Err(err) = keys_tx.send(evt) {
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
            tick_tx.send(Key::Null).unwrap();
        }
    });
    Ok(rx)
}

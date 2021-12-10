use std::{sync::{mpsc::{Receiver, self}, Arc, Mutex}, thread, io::{self, Result, stdout}};

use termion::{input::TermRead, event::Key, screen::AlternateScreen, raw::IntoRawMode};
use tracing::{error, debug, info};

use tui::{backend::TermionBackend, Terminal};

mod app;
mod draw;
mod log;

pub use app::App;
pub use log::AppWriter;

use crate::ui::draw::draw;

pub fn start(mut app: App) -> Result<()> {
    let event_rx = events()?;

    //let stdout = stdout();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        match event_rx.recv().unwrap() {
            Key::Char('q') => app.should_quit = true,
            Key::Char(' ') => app.step_tx.send(()).unwrap(),
            _ => (),
        }

        if app.should_quit {
            info!("Got quit signal!");
            return Ok(());
        }
    }
}

fn events() -> Result<Receiver<Key>> {
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
            thread::sleep_ms(250);
            tick_tx.send(Key::Null).unwrap();
        }
    });
    Ok(rx)
}

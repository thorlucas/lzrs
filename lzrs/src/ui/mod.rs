use std::{sync::mpsc::{Receiver, self}, thread, io::{self, Result}};

use termion::{input::TermRead, event::Key, screen::AlternateScreen, raw::IntoRawMode};
use tracing::{error, debug, info};
use tui::{backend::TermionBackend, Terminal};

mod app;
mod draw;

pub use app::{App, Config};

use crate::ui::draw::draw;

pub fn start<F>(config: app::Config, init: F) -> Result<()>
    where
        F: FnOnce(Receiver<()>) -> ()
{
    let step_tx = {
        let (step_tx, step_rx) = mpsc::channel();
        init(step_rx);
        step_tx
    };

    let event_rx = events()?;

    println!("initted");

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(config);

    loop {
        terminal.draw(|f| draw(f, &mut app))?;
        debug!("Waiting for input...");

        match event_rx.recv().unwrap() {
            Key::Char('q') => app.should_quit = true,
            Key::Char(' ') => step_tx.send(()).unwrap(),
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
    thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
            if let Ok(evt) = evt {
                if let Err(err) = tx.send(evt) {
                    error!("{}", err);
                    return;
                }
            }
        }
    });
    Ok(rx)
}

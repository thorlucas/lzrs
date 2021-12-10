use std::{io::{Write, Result, self}, sync::mpsc, thread};
use termion::{raw::IntoRawMode, screen::AlternateScreen, input::TermRead, event::Key};
use tracing::{dispatcher, Dispatch, Level, info, debug, error};
use tracing_subscriber::prelude::*;

use lzrs_lib::prelude::{Writer, Config};
use lzrs::{ui::*, step::StepSubscriber};
use tui::{backend::TermionBackend, Terminal};

fn main() -> Result<()> {
    let (step_tx, step_rx) = mpsc::channel();

    tracing_subscriber::registry()
        .with(UISubscriber::new())
        .with(StepSubscriber::new(Some(step_rx)))
        .init();

    let fmt_subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .compact()
        .with_ansi(true)
        .finish();

    let fmt_dispatch = Dispatch::new(fmt_subscriber);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    thread::spawn(move || {
        let to: Vec<u8> = Vec::new();
        let mut comp = Writer::new(to, Config {
            dict_size: 0x80,
        });

        write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
    });

    // This is the events thread, which sends events to the main thread
    let event_dispatch = fmt_dispatch.clone();
    let (event_tx, event_rx) = mpsc::channel();
    thread::spawn(move || {
        dispatcher::with_default(&event_dispatch, move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(evt) = evt {
                    if let Err(err) = event_tx.send(evt) {
                        error!("{}", err);
                        return;
                    }
                }
            }
        });
    });

    // This is going to be our UI thread where the dispatch won't block it.
    dispatcher::with_default(&fmt_dispatch, move || {
        let mut should_quit = false;
        loop {
            terminal.clear().unwrap();
            terminal.set_cursor(0, 0).unwrap();

            debug!("Waiting for input...");
            match event_rx.recv().unwrap() {
                Key::Char('q') => should_quit = true,
                Key::Char(' ') => step_tx.send(()).unwrap(),
                _ => (),
            }

            if should_quit {
                info!("Got quit signal!");
                return;
            }
        }
    });

    Ok(())
}

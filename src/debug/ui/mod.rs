use std::{thread, io::{Result, stdout, stdin}, sync::{mpsc, Mutex}, cell::RefCell};
use termion::{raw::IntoRawMode, input::TermRead, screen::AlternateScreen, event::Key};
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer, prelude::*};
use tui::{backend::TermionBackend, Terminal};

pub struct UILayer {
    pub step_tx: Mutex<mpsc::Sender<()>>,
}

impl UILayer {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel();
        (
            Self {
                step_tx: Mutex::new(tx),
            },
            rx
        )
    }
}

impl<S> Layer<S> for UILayer
    where
        S: Subscriber + for<'lookup> LookupSpan<'lookup>
{

}

pub fn spawn_ui() -> Result<()> {
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let (step_tx, step_rx) = mpsc::channel();

    thread::spawn(move || {
        let stdin = stdin();
        for evt in stdin.keys() {
            if let Ok(key) = evt {
                match key {
                    Key::Char('q') => break,
                    _ => step_tx.send(()).unwrap(),
                }
                terminal.draw(|_f| {}).unwrap();
            }
        }
    });

    Ok(())
}

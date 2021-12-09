use std::{thread, io::{Result, stdout, stdin}, sync::mpsc};
use termion::{raw::IntoRawMode, input::TermRead, screen::AlternateScreen, event::Key};
use tui::{backend::TermionBackend, Terminal};
use crate::debug::step;

struct UI;

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

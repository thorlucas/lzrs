use std::{thread, io::{Result, stdout, stdin}, sync::mpsc};
use termion::{raw::IntoRawMode, input::TermRead, screen::AlternateScreen, event::Key};
use tui::{backend::TermionBackend, Terminal};

use crate::Compressor;

pub fn spawn_ui<W>(compressor: &mut Compressor<W>) -> Result<()> {
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let (tick_tx, tick_rx) = mpsc::channel();
    compressor.set_tick_receiver(tick_rx);

    thread::spawn(move || {
        let stdin = stdin();
        for evt in stdin.keys() {
            if let Ok(key) = evt {
                match key {
                    Key::Char('q') => break,
                    _ => tick_tx.send(()).unwrap(),
                }
                terminal.draw(|_f| {}).unwrap();
            }
        }
    });

    Ok(())
}



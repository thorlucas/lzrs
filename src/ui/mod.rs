use std::{thread, io::{Result, stdout, stdin}, sync::mpsc};
use termion::{raw::IntoRawMode, input::TermRead, screen::AlternateScreen, event::Key};
use tui::{backend::TermionBackend, Terminal};
use crate::debug::{set_instance, Instance, Command};

pub fn spawn_ui() -> Result<()> {
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let (command_tx, command_rx) = mpsc::channel();
    let (info_tx, _info_rx) = mpsc::channel();

    set_instance(Instance {
        command_rx: Some(command_rx),
        info_tx,
    });

    thread::spawn(move || {
        let stdin = stdin();
        for evt in stdin.keys() {
            if let Ok(key) = evt {
                match key {
                    Key::Char('q') => break,
                    _ => command_tx.send(Command::Step).unwrap(),
                }
                terminal.draw(|_f| {}).unwrap();
            }
        }
    });

    Ok(())
}

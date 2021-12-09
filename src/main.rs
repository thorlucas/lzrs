use std::io::{Write, Result};
use std::sync::mpsc;
use lzrs::{Compressor, Config, ascii_buf};

const DICT_SIZE: usize = 0x80;

#[cfg(feature = "ui")]
fn spawn_ui<W>(compressor: &mut Compressor<W>) -> Result<()> {
    use std::{thread, io::{stdout, stdin}};
    use termion::{raw::IntoRawMode, input::TermRead, screen::AlternateScreen, event::Key};
    use tui::{backend::TermionBackend, Terminal};

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

fn main() -> Result<()> {
    let to: Vec<u8> = Vec::new();

    let mut comp = Compressor::new(to, Config {
        dict_size: DICT_SIZE,
    });

    #[cfg(feature = "ui")]
    spawn_ui(&mut comp)?;

    write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
    let out = comp.finish().unwrap();

    println!("{}", ascii_buf(out.iter()));

    Ok(())
}

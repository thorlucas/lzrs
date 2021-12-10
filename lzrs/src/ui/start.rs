use std::io::{self, Result};
use tui::{backend::{TermionBackend, Backend}, Terminal};
use termion::{screen::AlternateScreen, raw::IntoRawMode};

use crate::app::App;
use super::draw;

pub fn start() -> Result<Terminal<impl Backend>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    
    Ok(terminal)
}

pub fn draw_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
     terminal.draw(|f| draw(f, app))?;
     Ok(())
}

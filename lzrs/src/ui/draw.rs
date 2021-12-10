use std::{io::{Result, Write}, intrinsics::write_bytes};
use ansi_to_tui::ansi_to_text;
use tracing::info;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(f.size());

    let log_chunk = chunks[1];

    let text = {
        app.log_buffer.flush().unwrap();
        let buf: Vec<u8> = app.log_buffer.buf.lock().unwrap().drain(..).collect();
        ansi_to_text(buf).unwrap()
    };
    app.log.extend(text);

    let chunk_height = (log_chunk.height as usize) - 2;
    let lines = app.log.lines.len();
    if lines > chunk_height {
        app.log.lines = app.log.lines[(lines - chunk_height)..].to_vec();
    }

    let log_widget = Paragraph::new(app.log.clone())
        .block(Block::default().title("Log").borders(Borders::ALL))
        .wrap(tui::widgets::Wrap { trim: false });

    f.render_widget(log_widget, log_chunk);
}

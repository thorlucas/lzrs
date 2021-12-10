use ansi_to_tui::ansi_to_text;
use std::io::Write;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(f.size());

    let log_chunk = chunks[1];

    let text = {
        app.ui.log_buffer.flush().unwrap();
        let buf: Vec<u8> = app.ui.log_buffer.buf.lock().unwrap().drain(..).collect();
        ansi_to_text(buf).unwrap()
    };
    app.ui.log.extend(text);

    let chunk_height = (log_chunk.height as usize) - 2;
    let lines = app.ui.log.lines.len();
    if lines > chunk_height {
        app.ui.log.lines = app.ui.log.lines[(lines - chunk_height)..].to_vec();
    }

    let log_widget = Paragraph::new(app.ui.log.clone())
        .block(Block::default().title("Log").borders(Borders::ALL))
        .wrap(tui::widgets::Wrap { trim: false });

    f.render_widget(log_widget, log_chunk);
}

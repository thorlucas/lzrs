use std::io::Result;
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
        .margin(0)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());

    let text = {
        use tui::text::Text;
        Text::raw("Hello, world!")
    };

    let log_widget = Paragraph::new(text)
        .block(Block::default().title("Log").borders(Borders::ALL))
        .wrap(tui::widgets::Wrap { trim: false });
    f.render_widget(log_widget, chunks[1]);
}

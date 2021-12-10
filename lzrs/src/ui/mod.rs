use tui::text::Text;

mod draw;
mod writer;
mod start;

pub use writer::UIWriter;
pub use draw::draw;
pub use start::{start, draw_loop};

pub struct UI<'a> {
    pub log_buffer: UIWriter,
    pub log: Text<'a>,
}

impl<'a> UI<'a> {
    pub fn new() -> Self {
        Self {
            log_buffer: UIWriter::new(),
            log: vec![].into(),
        }
    }

    pub fn log_writer(&self) -> UIWriter {
        self.log_buffer.clone()
    }
}

mod event;
mod run;

pub use run::run;
pub use event::start_event_loop;

use crate::ui::{UI, UIWriter};
use crate::trace::Trace;

pub struct App<'a> {
    pub should_quit: bool,
    pub ui: UI<'a>,
    pub trace: Trace<UIWriter>,
}

impl App<'_> {
    pub fn new() -> Self {
        let ui = UI::new();
        let writer = ui.log_writer();

        let trace = Trace::new(writer);

        Self {
            should_quit: false,
            ui,
            trace,
        }
    }
}

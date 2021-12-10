mod event;
mod run;

use std::sync::mpsc::Sender;

pub use run::run;
pub use event::start_event_loop;

use crate::ui::{UI, UIWriter};
use crate::trace::Trace;

pub struct App<'a> {
    pub should_quit: bool,
    pub ui: UI<'a>,
    pub trace: Trace<UIWriter>,
    pub step: Sender<()>,
}

impl App<'_> {
    pub fn new() -> Self {
        let ui = UI::new();
        let writer = ui.log_writer();
        let mut trace = Trace::new(writer);
        let step = trace.take_step_tx().unwrap();

        Self {
            should_quit: false,
            ui,
            trace,
            step,
        }
    }
}

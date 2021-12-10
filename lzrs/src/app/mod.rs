mod event;
mod run;

use std::sync::mpsc::{Sender, Receiver, self};

pub use run::run;
pub use event::{start_event_loop, Event};

use crate::ui::{UI, UIWriter};
use crate::trace::Trace;

pub struct App<'a> {
    pub should_quit: bool,
    pub ui: UI<'a>,
    pub trace: Trace<UIWriter>,
    pub step: Sender<()>,

    pub event_rx: Receiver<Event>,
    pub event_tx: Option<Sender<Event>>,

    pub dict: Option<(&'static [u8], usize)>,
}

impl App<'_> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let ui = UI::new();
        let writer = ui.log_writer();

        let mut trace = Trace::new(writer);

        trace.subscribe_event_tx(tx.clone());
        let step = trace.take_step_tx().unwrap();

        Self {
            should_quit: false,
            ui,
            trace,
            step,

            event_rx: rx,
            event_tx: Some(tx),

            dict: None,
        }
    }
}

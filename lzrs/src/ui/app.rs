use std::{sync::{Mutex, Arc, mpsc::{self, Receiver, Sender}}, cell::RefCell, borrow::BorrowMut};

use tracing_subscriber::fmt::{MakeWriter, writer::MakeWriterExt};
use tui::text::{Text, Spans};

use super::log::AppWriter;

pub struct App<'a> {
    pub should_quit: bool,
    pub log_buffer: AppWriter,
    pub step_tx: Sender<()>,
    step_rx: Option<Receiver<()>>,
    pub log: Text<'a>,
}

impl App<'_> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            should_quit: false,
            log_buffer: AppWriter::new(),
            step_tx: tx,
            step_rx: Some(rx),
            log: vec![].into(),
        }
    }

    pub fn subscribe_step(&mut self) -> Option<Receiver<()>> {
        self.step_rx.take()
    }
}

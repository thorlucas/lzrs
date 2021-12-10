use std::sync::{mpsc::{Receiver, Sender, self}, Mutex};

use tracing::{Subscriber, span};
use tracing_subscriber::{registry::LookupSpan, Layer};

pub struct StepLayer {
    pub(super) tx: Mutex<Option<Sender<()>>>,
    rx: Mutex<Receiver<()>>,
}

impl StepLayer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx: Mutex::new(Some(tx)),
            rx: Mutex::new(rx)
        }
    }
}

impl<S> Layer<S> for StepLayer 
    where
        S: Subscriber + for<'a> LookupSpan<'a>
{
    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(_span) = ctx.span(id) {
            self.rx.lock().unwrap().recv().unwrap();
        }
    }
}

use std::sync::{mpsc::Receiver, Mutex};

use tracing::{Subscriber, span, Level};
use tracing_subscriber::{registry::LookupSpan, Layer};

pub struct StepSubscriber {
    step_rx: Mutex<Option<Receiver<()>>>,
}

impl StepSubscriber {
    pub fn new(step_rx: Option<Receiver<()>>) -> Self {
       Self {
            step_rx: Mutex::new(step_rx),
       }
    }
}

impl<S> Layer<S> for StepSubscriber 
    where
        S: Subscriber + for<'a> LookupSpan<'a>
{
    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            println!("Blocking on span {}", span.name());
            if let Some(rx) = &*self.step_rx.lock().unwrap() {
                rx.recv().unwrap();
            }
            println!("Unblocking");
        }
    }
}

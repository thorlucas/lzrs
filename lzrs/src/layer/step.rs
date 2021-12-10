use std::sync::{mpsc::Receiver, Mutex, Arc};

use tracing::{Subscriber, span, subscriber::Interest};
use tracing_subscriber::{registry::LookupSpan, Layer};

pub struct StepLayer {}

impl StepLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for StepLayer 
    where
        S: Subscriber + for<'a> LookupSpan<'a>
{
    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            //println!("Blocking on span {}", span.name());
            //self.step_rx.recv().unwrap();
            //println!("Unblocking");
        }
    }
}

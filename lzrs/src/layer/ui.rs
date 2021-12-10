use std::sync::mpsc::Receiver;

use tracing::{Subscriber, span, subscriber::Interest};
use tracing_subscriber::{registry::LookupSpan, Layer};

pub struct UILayer;

impl UILayer {
    pub fn new() -> Self {
       Self
    }
}

/// The UISubscriber is the main default subscriber that sends messages to the UI threads.
/// It should only be responsible for decoding the logged data and sending it to the UI thread. A
/// separate subscriber is responsible for blocking the thread when needed.
impl<S> Layer<S> for UILayer 
    where
        S: Subscriber + for<'a> LookupSpan<'a>
{
    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            //println!("TARGET: {} - UI got span {}", span.metadata().target(), span.name());
        }
    }
}

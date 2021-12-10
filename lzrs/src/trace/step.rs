use tracing::{Subscriber, span};
use tracing_subscriber::{registry::LookupSpan, Layer};

#[derive(Copy, Clone)]
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
        if let Some(_span) = ctx.span(id) {
            //println!("Blocking on span {}", span.name());
            //self.step_rx.recv().unwrap();
            //println!("Unblocking");
        }
    }
}

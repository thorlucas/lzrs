use tracing::{Subscriber, span, info, subscriber::Interest, Metadata, debug, field::{Visit, Field}};
use tracing_subscriber::{registry::LookupSpan, Layer, prelude::*};

#[derive(Copy, Clone)]
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
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        info!("New span");
        let span = ctx.span(id).unwrap();        
        if let Some(field) = span.fields().field("dict.ptr") {
            info!(field.name = field.name(), span.name = span.name(), "Found a dict field!");

            let mut v: DictVisitor = Default::default();
            attrs.record(&mut v);
            
            info!(dict.ptr = v.ptr.unwrap() as usize, dict.len = v.len, dict.head = v.head, "Got fields!");
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        info!("Enter");
    }

    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        info!("Event");
    }

    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.span(id).unwrap();        
        if let Some(field) = span.fields().field("dict.ptr") {
            info!(field.name = field.name(), span.name = span.name(), "Found a dict field!");

            let mut v: DictVisitor = if let Some(old) = span.extensions().get() {
                *old
            } else {
                DictVisitor::default()
            };

            values.record(&mut v);

            if v.is_done() {
                let v = v.finish();
                let lower = (v.1 as isize - 5).max(0) as usize;
                let upper = (v.1 + 5).min(v.0.len());
                let b = &(v.0)[lower..upper];
                let s = unsafe { std::str::from_utf8_unchecked(b) };
                info!(dict.ptr = s, dict.len = v.0.len(), dict.head = v.1, "Finished dictionary buffer!");
                span.extensions_mut().replace(v);
            } else {
                span.extensions_mut().replace(v);
            }
        }
    }
}

#[derive(Copy, Clone, Default)]
struct DictVisitor {
    pub ptr: Option<usize>,
    pub len: Option<usize>,
    pub head: Option<usize>,
}

impl DictVisitor {
    pub fn is_done(&self) -> bool {
        self.ptr.is_some() && self.len.is_some() && self.head.is_some()
    }

    pub fn finish(mut self) -> (&'static [u8], usize) {
        let ptr: *const u8 = unsafe { std::mem::transmute(self.ptr.unwrap()) };
        let buf: &'static [u8] = unsafe { std::slice::from_raw_parts(ptr, self.len.unwrap()) };
        let head: usize = self.head.unwrap();
        (buf, head)
    }
}

impl DictVisitor {
    const DICT_PTR: &'static str = "dict.ptr";
    const DICT_LEN: &'static str = "dict.len";
    const DICT_HEAD: &'static str = "dict.head";
}

impl Visit for DictVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {}

    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            DictVisitor::DICT_PTR => self.ptr = Some(value as usize),
            DictVisitor::DICT_LEN => self.len = Some(value as usize),
            DictVisitor::DICT_HEAD => self.head = Some(value as usize),
            _ => (),
        }
    }
}

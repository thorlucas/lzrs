use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt::MakeWriter, filter::filter_fn};
use tracing_subscriber::prelude::*;

use super::Trace;

pub fn start<W>(trace: &Trace<W>)
    where
        W: for<'w> MakeWriter<'w> + 'static + Send + Sync + Clone
{ 
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_writer(trace.writer.clone());
        
    let sub = tracing_subscriber::registry()
        .with(
            fmt_layer
            .with_filter(filter_fn(|meta| {
                match meta.target() {
                    //t if t.starts_with("lzrs::") => *meta.level() <= Level::TRACE,
                    //t if t.starts_with("lzrs_lib::") => *meta.level() <= Level::INFO,
                    _ => true,
                }
            }))
        )
        .with(
            trace.ui_layer.clone()
            .with_filter(filter_fn(|meta| {
                !meta.target().starts_with("lzrs::")
            }))
        )
        .with(
            trace.step_layer.clone()
            .with_filter(filter_fn(|meta| {
                !meta.target().starts_with("lzrs::")
            }))
        )
        .init();

    sub
}

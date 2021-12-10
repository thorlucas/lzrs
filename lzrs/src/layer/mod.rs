pub mod step;
pub mod ui;

use std::{sync::{mpsc::{Receiver}, Arc, Mutex}, io::Write};

use tracing::Level;
use tracing_subscriber::{filter::filter_fn, Layer, prelude::*, fmt::MakeWriter};
pub use ui::UILayer;
pub use step::StepLayer;

use crate::ui::AppWriter;

pub struct Config {
    pub writer: AppWriter,
}


pub fn start(config: Config) { 
    //let (make_writer, _guard) = tracing_appender::non_blocking(
        //config.writer
    //);

    let ui_layer = UILayer::new();
    let step_layer = StepLayer::new();
    let fmt_layer = tracing_subscriber::fmt::layer()
        //.compact()
        //.pretty()    
        .with_ansi(true)
        .with_writer(config.writer);
        
    tracing_subscriber::registry()
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
            ui_layer
            .with_filter(filter_fn(|meta| {
                !meta.target().starts_with("lzrs::")
            }))
        )
        .with(
            step_layer
            .with_filter(filter_fn(|meta| {
                !meta.target().starts_with("lzrs::")
            }))
        )
        .init();
}

pub mod step;
pub mod ui;

use std::sync::mpsc::Receiver;

use tracing::{Level, Subscriber};
use tracing_subscriber::{filter::filter_fn, Layer, prelude::*};
pub use ui::UILayer;
pub use step::StepLayer;

pub struct Config {
    pub step_rx: Receiver<()>,
}

pub fn make_subscriber(config: Config) -> impl Subscriber {
    let ui_layer = UILayer::new()
        .with_filter(filter_fn(|meta| {
            !meta.target().starts_with("lzrs::")
        }));

    let step_layer = StepLayer::new(Some(config.step_rx))
        .with_filter(filter_fn(|meta| {
            !meta.target().starts_with("lzrs::")
        }));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .pretty()    
        .with_ansi(true)
        //.with_writer(app.log_buf)
        .with_filter(filter_fn(|meta| {
            match meta.target() {
                t if t.starts_with("lzrs::") => *meta.level() <= Level::TRACE,
                t if t.starts_with("lzrs_lib::") => *meta.level() <= Level::INFO,
                _ => true,
            }
        }));
        
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(ui_layer)
        .with(step_layer)
}

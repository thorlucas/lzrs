use tracing_subscriber::fmt::MakeWriter;

use self::{ui::UILayer, step::StepLayer};

mod step;
mod ui;
mod start;

pub use start::start;

pub struct Trace<W> {
    ui_layer: UILayer,
    step_layer: StepLayer,
    writer: W,
}

impl<W> Trace<W>
    where
        W: for<'w> MakeWriter<'w>
{
    pub fn new(writer: W) -> Self {
        Self {
            ui_layer: UILayer::new(),
            step_layer: StepLayer::new(),
            writer,
        }
    }
}

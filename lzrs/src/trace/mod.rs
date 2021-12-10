use std::sync::mpsc::Sender;
use tracing_subscriber::fmt::MakeWriter;
use self::{ui::UILayer, step::StepLayer};

mod step;
mod ui;
mod start;

pub use start::start;

pub struct Trace<W> {
    ui_layer: Option<UILayer>,
    step_layer: Option<StepLayer>,
    writer: W,
}

impl<W> Trace<W>
    where
        W: for<'w> MakeWriter<'w>
{
    pub fn new(writer: W) -> Self {
        Self {
            ui_layer: Some(UILayer::new()),
            step_layer: Some(StepLayer::new()),
            writer,
        }
    }

    pub fn take_step_tx(&mut self) -> Option<Sender<()>> {
        if let Some(step_layer) = &mut self.step_layer {
            step_layer.tx.lock().unwrap().take() 
        } else {
            None
        }
    }
}

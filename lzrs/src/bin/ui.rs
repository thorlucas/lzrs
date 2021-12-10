use std::{io::{Write, Result}, sync::mpsc};
use tracing_subscriber::prelude::*;

use lzrs_lib::prelude::{Writer, Config};
use lzrs::{ui::*, step::StepSubscriber};

fn main() -> Result<()> {
    let (step_tx, step_rx) = mpsc::channel();

    tracing_subscriber::registry()
        .with(UISubscriber::new())
        .with(StepSubscriber::new(Some(step_rx)))
        .init();

    let to: Vec<u8> = Vec::new();

    let mut comp = Writer::new(to, Config {
        dict_size: 0x80,
    });

    write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();

    Ok(())
}

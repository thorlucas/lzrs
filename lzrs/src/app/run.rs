use std::io::Result;
use termion::event::Key;
use tracing::info;

use super::{start_event_loop, App};
use crate::{ui::{self, draw_loop}, trace};

pub fn run<F>(mut app: App, init: F) -> Result<()>
where
    F: FnOnce() -> (),
{
    let event_rx = start_event_loop()?; 
    let mut terminal = ui::start()?;
    trace::start(&app.trace);

    init(); 

    loop {
        draw_loop(&mut terminal, &mut app)?;

        match event_rx.recv().unwrap() {
            Key::Char('q') => app.should_quit = true,
            //Key::Char(' ') => app.step_tx.send(()).unwrap(),
            _ => (),
        }

        if app.should_quit {
            info!("Got quit signal!");
            return Ok(());
        }
    }
}

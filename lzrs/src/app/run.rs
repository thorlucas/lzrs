use std::io::Result;
use termion::event::Key;
use tracing::info;

use super::{start_event_loop, App, event::Event};
use crate::{ui::{self, draw_loop}, trace};

pub fn run<F>(mut app: App, init: F) -> Result<()>
where
    F: FnOnce() -> (),
{
    start_event_loop(app.event_tx.take().unwrap());
    let mut terminal = ui::start()?;
    trace::start(&mut app.trace);

    init(); 

    loop {
        draw_loop(&mut terminal, &mut app)?;

        match app.event_rx.recv().unwrap() {
            Event::Tick => (),
            Event::Key(key) => match key {
                Key::Char('q') => app.should_quit = true,
                Key::Char(' ') => app.step.send(()).unwrap(),
                _ => (),
            },
            Event::LoadDictBuffer { buf, head } => {
                app.dict = Some((buf, head));
                info!("Updated event!");
            },
        }

        if app.should_quit {
            info!("Got quit signal!");
            return Ok(());
        }
    }
}

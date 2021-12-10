use std::{thread, io::{Result, stdout, stdin}, sync::{mpsc, Mutex}, cell::RefCell};
use termion::{raw::IntoRawMode, input::TermRead, screen::AlternateScreen, event::Key};
use tracing::{Subscriber, span};
use tracing_subscriber::{registry::LookupSpan, Layer, prelude::*};
use tui::{backend::TermionBackend, Terminal};

use crate::debug::step::step;

pub struct UILayer;

impl UILayer {
    pub fn new() -> Result<(Self, mpsc::Receiver<()>)> {
        let stdout = stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;

        let (step_tx, step_rx) = mpsc::channel();

        thread::spawn(move || {
            let stdin = stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    match key {
                        Key::Char('q') => break,
                        _ => step_tx.send(()).unwrap(),
                    }
                    terminal.draw(|_f| {}).unwrap();
                }
            }
        });

        Ok((
            Self,
            step_rx
        ))
    }
}

impl<S> Layer<S> for UILayer
    where
        S: Subscriber + for<'lookup> LookupSpan<'lookup>
{
    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let name = ctx.span(id).unwrap().metadata().name();
        println!("Entering span {}, blocking", name);
        step();
        println!("unblocked");
    }
}

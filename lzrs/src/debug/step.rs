use std::sync::{Mutex, mpsc::{Sender, Receiver}};
use lazy_static::lazy_static;

lazy_static! {
    static ref STEP: Mutex<Option<Receiver<()>>> = Mutex::new(None);
}

pub fn set_step(step: Receiver<()>) {
    *STEP.lock().unwrap() = Some(step);
}

pub fn free_step() -> Option<Receiver<()>> {
    STEP.lock().unwrap().take()
}

pub fn step() {
    if let Some(step) = &*STEP.lock().unwrap() {
        step.recv().unwrap();
    }
}

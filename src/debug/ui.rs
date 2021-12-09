use std::sync::{Mutex, mpsc};
use lazy_static::lazy_static;

lazy_static! {
    static ref UI_INSTANCE: Mutex<Option<UIDebugger>> = Mutex::new(None);
}

pub fn set_ui(ui: UIDebugger) {
    *UI_INSTANCE.lock().unwrap() = Some(ui);
}

pub fn tick() {
    if let Some(ui) = &mut *UI_INSTANCE.lock().unwrap() {
        ui.tick.recv().unwrap();
    }
}

pub struct UIDebugger {
    pub tick: mpsc::Receiver<()>,
}


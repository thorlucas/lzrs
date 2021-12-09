use std::sync::{Mutex, mpsc::{Sender, Receiver}};

use lazy_static::lazy_static;

lazy_static! {
    static ref CURRENT_INSTANCE: Mutex<Option<Instance>> = Mutex::new(None);
}

#[derive(Copy, Clone)]
pub enum Command {
    Step
}

#[derive(Copy, Clone)]
pub enum Info {

}

pub struct Instance {
    pub command_rx: Option<Receiver<Command>>,
    pub info_tx: Sender<Info>,
}

pub fn set_instance(instance: Instance) {
    *CURRENT_INSTANCE.lock().unwrap() = Some(instance);
}

fn with_instance<F>(cb: F)
    where
        F: FnOnce(&mut Instance) -> ()
{
    if let Some(instance) = &mut *CURRENT_INSTANCE.lock().unwrap() {
        cb(instance);
    }
}

pub fn tick() {
    with_instance(|instance| {
        if let Some(command) = &instance.command_rx {
            loop {
                match command.recv().unwrap() {
                    Command::Step => break, 
                }
            }
        }
    });
}

use std::{io::{Write, Read}, sync::{Mutex, Arc}};

use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
pub struct UIWriter {
    pub buf: Arc<Mutex<Vec<u8>>>,
}

impl UIWriter {
    pub fn new() -> Self {
        Self {
            buf: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn available(&self) -> usize {
        self.buf.lock().unwrap().len() 
    }
}

impl Write for UIWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        //println!("{}", String::from_utf8(buf.to_vec()).unwrap());
        //println!("Writing {} bytes into", buf.len());
        self.buf.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        //println!("Flushing");
        self.buf.lock().unwrap().flush()
    }
}

impl Read for UIWriter {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        buf.write(&self.buf.lock().unwrap())
    }
}

impl MakeWriter<'_> for UIWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        self.clone()
    }
}

mod writer;
pub use writer::Writer;

pub struct Config {
    pub dict_size: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Token {
    Literal {
        byte: u8,
    },
    Rep {
        distance: usize,
        length: usize,
    }
}

pub mod prelude {
    pub use super::writer::Writer;
    pub use super::Config;
}

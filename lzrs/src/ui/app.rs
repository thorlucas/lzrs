pub struct Config {

}

pub struct App {
    pub should_quit: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            should_quit: false,
        }
    }
}

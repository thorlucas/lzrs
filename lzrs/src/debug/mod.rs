#[cfg(feature = "step")]
mod step;

#[cfg(feature = "ui")]
pub mod ui;

use tracing_subscriber::prelude::*;

#[cfg(not(feature = "ui"))]
pub fn init() -> std::io::Result<()> {
    use tracing_subscriber::fmt;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();

    Ok(())
}

#[cfg(feature = "ui")]
pub fn init() -> std::io::Result<()> {
    use crate::debug::step::set_step;

    use self::ui::UILayer;

    let (ui, step_rx) = UILayer::new()?;
    set_step(step_rx);

    tracing_subscriber::registry()
        .with(ui)
        .init();

    Ok(())
}
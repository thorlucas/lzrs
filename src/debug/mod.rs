#[cfg(feature = "ui")]
pub mod ui;

#[macro_export]
macro_rules! tick {
    () => {
        #[cfg(feature = "ui")]        
        crate::debug::ui::tick();
    };
}

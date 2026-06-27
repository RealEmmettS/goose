#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
pub struct RuntimeOptions {
    pub no_sound: bool,
    pub no_mouse_steal: bool,
    pub no_window_ride: bool,
}

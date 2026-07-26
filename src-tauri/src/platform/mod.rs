#[cfg(target_os = "macos")]
mod macos;

use thiserror::Error;

#[cfg(target_os = "macos")]
pub use macos::{configure_window, pin_window_to_screen_top, resign_window_focus, screen_geometry};

#[cfg(not(target_os = "macos"))]
pub fn configure_window(_window: &tauri::WebviewWindow) -> Result<(), PlatformError> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn resign_window_focus(_window: &tauri::WebviewWindow) -> Result<(), PlatformError> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn pin_window_to_screen_top(_window: &tauri::WebviewWindow) -> Result<(), PlatformError> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn screen_geometry(
    _window: &tauri::WebviewWindow,
) -> Result<crate::window::layout::ScreenGeometry, PlatformError> {
    Err(PlatformError::Unavailable)
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("platform information is unavailable")]
    Unavailable,
    #[error("operation must run on the main thread")]
    NotMainThread,
    #[error("native window handle is unavailable: {0}")]
    NativeHandle(String),
}

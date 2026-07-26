use crate::platform::PlatformError;
use objc2_app_kit::{NSColor, NSStatusWindowLevel, NSWindow, NSWindowCollectionBehavior};
use objc2_foundation::NSPoint;
use tauri::WebviewWindow;

pub fn configure_window(window: &WebviewWindow) -> Result<(), PlatformError> {
    let pointer = window
        .ns_window()
        .map_err(|error| PlatformError::NativeHandle(error.to_string()))?
        as usize;
    window
        .run_on_main_thread(move || {
            // SAFETY: Tauri owns this NSWindow for the full lifetime of the webview window.
            // The callback is explicitly dispatched to AppKit's main thread, and the pointer
            // is only borrowed for this callback; ownership is never transferred.
            let native = unsafe { &*(pointer as *mut NSWindow) };
            native.setOpaque(false);
            native.setHasShadow(false);
            let clear = NSColor::clearColor();
            native.setBackgroundColor(Some(&clear));
            native.setLevel(NSStatusWindowLevel + 1);
            native.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );
        })
        .map_err(|error| PlatformError::NativeHandle(error.to_string()))
}

pub fn resign_window_focus(window: &WebviewWindow) -> Result<(), PlatformError> {
    let pointer = window
        .ns_window()
        .map_err(|error| PlatformError::NativeHandle(error.to_string()))?
        as usize;
    window
        .run_on_main_thread(move || {
            // SAFETY: Tauri owns the NSWindow and guarantees that this main-thread callback
            // executes while the webview window exists. The reference does not escape.
            let native = unsafe { &*(pointer as *mut NSWindow) };
            native.resignKeyWindow();
        })
        .map_err(|error| PlatformError::NativeHandle(error.to_string()))
}

pub fn pin_window_to_screen_top(window: &WebviewWindow) -> Result<(), PlatformError> {
    let pointer = window
        .ns_window()
        .map_err(|error| PlatformError::NativeHandle(error.to_string()))?
        as usize;
    window
        .run_on_main_thread(move || {
            // SAFETY: Tauri owns this NSWindow for the webview lifetime. The callback is
            // dispatched to AppKit's main thread, borrows the pointer only temporarily,
            // and neither transfers ownership nor lets the reference escape.
            let native = unsafe { &*(pointer as *mut NSWindow) };
            if let Some(screen) = native.screen() {
                let screen_frame = screen.frame();
                let physical_top = screen_frame.origin.y + screen_frame.size.height;
                let current_x = native.frame().origin.x;
                native.setFrameTopLeftPoint(NSPoint::new(current_x, physical_top));
            }
        })
        .map_err(|error| PlatformError::NativeHandle(error.to_string()))
}

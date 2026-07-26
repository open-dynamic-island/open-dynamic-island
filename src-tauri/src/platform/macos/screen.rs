use crate::platform::PlatformError;
use crate::window::layout::ScreenGeometry;
use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;
use std::sync::{Mutex, OnceLock};
use tauri::WebviewWindow;

static SCREEN_GEOMETRY_CACHE: OnceLock<Mutex<ScreenGeometry>> = OnceLock::new();

pub fn screen_geometry(_window: &WebviewWindow) -> Result<ScreenGeometry, PlatformError> {
    let Some(mtm) = MainThreadMarker::new() else {
        return SCREEN_GEOMETRY_CACHE
            .get()
            .and_then(|cache| cache.lock().ok().map(|screen| screen.clone()))
            .ok_or(PlatformError::NotMainThread);
    };
    let main = NSScreen::mainScreen(mtm).ok_or(PlatformError::Unavailable)?;
    let frame = main.frame();
    let visible = main.visibleFrame();
    let safe = main.safeAreaInsets();
    let auxiliary_left = main.auxiliaryTopLeftArea();
    let auxiliary_right = main.auxiliaryTopRightArea();
    let scale = main.backingScaleFactor();

    // AppKit is bottom-left based. Relative to the main display, Tauri's top-left Y is
    // zero; converting with the main display's top also preserves negative Y origins.
    let top = frame.origin.y + frame.size.height;
    let visible_top = top - (visible.origin.y + visible.size.height);

    let geometry = ScreenGeometry {
        frame_x: frame.origin.x,
        frame_y: 0.0,
        frame_width: frame.size.width,
        frame_height: frame.size.height,
        visible_x: visible.origin.x,
        visible_y: visible_top,
        visible_width: visible.size.width,
        visible_height: visible.size.height,
        safe_top: safe.top,
        safe_left: safe.left,
        safe_right: safe.right,
        safe_bottom: safe.bottom,
        auxiliary_left_width: (auxiliary_left.size.width > 0.0)
            .then_some(auxiliary_left.size.width),
        auxiliary_right_width: (auxiliary_right.size.width > 0.0)
            .then_some(auxiliary_right.size.width),
        scale_factor: scale,
    };
    let cache = SCREEN_GEOMETRY_CACHE.get_or_init(|| Mutex::new(geometry.clone()));
    if let Ok(mut cached) = cache.lock() {
        *cached = geometry.clone();
    }
    Ok(geometry)
}

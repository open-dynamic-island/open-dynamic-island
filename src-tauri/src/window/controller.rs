use super::constants::{
    ATTENTION_HEIGHT, ATTENTION_WIDTH, COMPACT_HEIGHT, COMPACT_WIDTH, EXPANDED_HEIGHT,
    EXPANDED_WIDTH,
};
use super::layout::{calculate_island_frame, ScreenGeometry};
use crate::platform;
use island_model::IslandMode;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use thiserror::Error;

pub const ISLAND_WINDOW_LABEL: &str = "island";

#[derive(Clone)]
pub struct WindowController {
    app: AppHandle,
}

impl WindowController {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn create(app: &AppHandle) -> Result<WebviewWindow, WindowError> {
        let window = WebviewWindowBuilder::new(app, ISLAND_WINDOW_LABEL, WebviewUrl::default())
            .title("Open Island")
            .inner_size(COMPACT_WIDTH, COMPACT_HEIGHT)
            .min_inner_size(COMPACT_WIDTH, COMPACT_HEIGHT)
            .decorations(false)
            .resizable(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .focusable(false)
            .focused(false)
            .visible(false)
            .transparent(true)
            .shadow(false)
            .build()?;
        platform::configure_window(&window)?;
        Self::new(app.clone()).reposition_for(COMPACT_WIDTH, COMPACT_HEIGHT)?;
        let controller = Self::new(app.clone());
        window.on_window_event(move |event| {
            if matches!(event, tauri::WindowEvent::ScaleFactorChanged { .. }) {
                let _ = controller.reposition();
            }
        });
        Ok(window)
    }

    pub fn show_compact(&self) -> Result<(), WindowError> {
        self.resize_and_position(COMPACT_WIDTH, COMPACT_HEIGHT)?;
        self.window()?.show()?;
        Ok(())
    }

    pub fn show_attention(&self) -> Result<(), WindowError> {
        self.resize_and_position(ATTENTION_WIDTH, ATTENTION_HEIGHT)?;
        self.window()?.show()?;
        Ok(())
    }

    pub fn prepare_expanded(&self) -> Result<(), WindowError> {
        self.resize_and_position(EXPANDED_WIDTH, EXPANDED_HEIGHT)?;
        self.window()?.set_focusable(true)?;
        Ok(())
    }

    pub fn finalize_compact(&self) -> Result<(), WindowError> {
        self.resign_focus()?;
        self.window()?.set_focusable(false)?;
        self.resize_and_position(COMPACT_WIDTH, COMPACT_HEIGHT)
    }

    pub fn hide(&self) -> Result<(), WindowError> {
        self.window()?.hide()?;
        Ok(())
    }

    pub fn focus_expanded(&self) -> Result<(), WindowError> {
        let window = self.window()?;
        window.set_focusable(true)?;
        window.set_focus()?;
        Ok(())
    }

    pub fn resign_focus(&self) -> Result<(), WindowError> {
        platform::resign_window_focus(&self.window()?)?;
        Ok(())
    }

    pub fn reposition(&self) -> Result<(), WindowError> {
        let window = self.window()?;
        let size = window.inner_size()?;
        let scale = window.scale_factor()?;
        self.reposition_for(size.width as f64 / scale, size.height as f64 / scale)
    }

    pub fn apply_mode(&self, mode: IslandMode) -> Result<(), WindowError> {
        match mode {
            IslandMode::Hidden => self.hide(),
            IslandMode::Compact => self.show_compact(),
            IslandMode::Attention => self.show_attention(),
            IslandMode::Expanded => self.prepare_expanded(),
        }
    }

    fn resize_and_position(&self, width: f64, height: f64) -> Result<(), WindowError> {
        self.reposition_for(width, height)
    }

    fn reposition_for(&self, width: f64, height: f64) -> Result<(), WindowError> {
        let window = self.window()?;
        let screen =
            platform::screen_geometry(&window).or_else(|_| fallback_screen_geometry(&window))?;
        let frame = calculate_island_frame(&screen, width, height);
        window.set_size(tauri::LogicalSize::new(frame.width, frame.height))?;
        window.set_position(tauri::LogicalPosition::new(frame.x, frame.y))?;
        platform::pin_window_to_screen_top(&window)?;
        #[cfg(debug_assertions)]
        eprintln!("Open Island frame: screen={screen:?}, calculated={frame:?}");
        Ok(())
    }

    fn window(&self) -> Result<WebviewWindow, WindowError> {
        self.app
            .get_webview_window(ISLAND_WINDOW_LABEL)
            .ok_or(WindowError::MissingWindow)
    }
}

fn fallback_screen_geometry(window: &WebviewWindow) -> Result<ScreenGeometry, WindowError> {
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or(WindowError::MissingMonitor)?;
    let scale = monitor.scale_factor();
    let position = monitor.position();
    let size = monitor.size();
    Ok(ScreenGeometry {
        frame_x: position.x as f64 / scale,
        frame_y: position.y as f64 / scale,
        frame_width: size.width as f64 / scale,
        frame_height: size.height as f64 / scale,
        visible_x: position.x as f64 / scale,
        visible_y: position.y as f64 / scale,
        visible_width: size.width as f64 / scale,
        visible_height: size.height as f64 / scale,
        safe_top: 0.0,
        safe_left: 0.0,
        safe_right: 0.0,
        safe_bottom: 0.0,
        auxiliary_left_width: None,
        auxiliary_right_width: None,
        scale_factor: scale,
    })
}

#[derive(Debug, Error)]
pub enum WindowError {
    #[error("island window is not available")]
    MissingWindow,
    #[error("no display is available")]
    MissingMonitor,
    #[error("native window operation failed: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("platform window operation failed: {0}")]
    Platform(#[from] platform::PlatformError),
}

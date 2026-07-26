mod activity;
mod commands;
mod events;
mod platform;
mod tray;
mod window;

use activity::manager::ActivityManager;
use commands::{
    animation_completed, dismiss_activity, first_frame_ready, frontend_ready,
    invoke_activity_action, run_demo, toggle_expansion,
};
use island_model::IslandDisplayContext;
use tauri::Manager;
use window::controller::WindowController;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(ActivityManager::default())
        .invoke_handler(tauri::generate_handler![
            frontend_ready,
            first_frame_ready,
            toggle_expansion,
            dismiss_activity,
            invoke_activity_action,
            animation_completed,
            run_demo
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            let window = WindowController::create(app.handle())?;
            if let Ok(screen) = platform::screen_geometry(&window) {
                let context = IslandDisplayContext {
                    has_notch: screen.has_notch(),
                    center_exclusion_width: screen.estimated_notch_width(),
                };
                if let Err(error) = app.state::<ActivityManager>().set_display_context(context) {
                    eprintln!("Open Island could not store display context: {error}");
                }
            }
            tray::menu::build(app)?;

            Ok(())
        });

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("Open Island failed to start: {error}");
    }
}

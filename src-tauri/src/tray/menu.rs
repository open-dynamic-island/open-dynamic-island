use crate::commands::{hide_all, start_demo};
use island_model::ActivityKind;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

pub fn build(app: &tauri::App) -> tauri::Result<()> {
    let timer = MenuItem::with_id(app, "demo_timer", "Show Timer Demo", true, None::<&str>)?;
    let recording = MenuItem::with_id(
        app,
        "demo_recording",
        "Show Recording Demo",
        true,
        None::<&str>,
    )?;
    let download = MenuItem::with_id(
        app,
        "demo_download",
        "Show Download Demo",
        true,
        None::<&str>,
    )?;
    let notification = MenuItem::with_id(
        app,
        "demo_notification",
        "Show Notification Demo",
        true,
        None::<&str>,
    )?;
    let failure = MenuItem::with_id(app, "demo_failure", "Show Failure Demo", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Island", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Open Island", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &timer,
            &recording,
            &download,
            &notification,
            &failure,
            &hide,
            &quit,
        ],
    )?;
    let icon = tauri::image::Image::from_bytes(include_bytes!("../../icons/tray-icon.png"))?;

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Open Island")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let result = match event.id.as_ref() {
                "demo_timer" => start_demo(app, ActivityKind::Timer, false),
                "demo_recording" => start_demo(app, ActivityKind::Recording, false),
                "demo_download" => start_demo(app, ActivityKind::Download, false),
                "demo_notification" => start_demo(app, ActivityKind::Notification, false),
                "demo_failure" => start_demo(app, ActivityKind::Notification, true),
                "hide" => hide_all(app),
                "quit" => {
                    app.exit(0);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                eprintln!("Open Island tray action failed: {error}");
            }
        })
        .build(app)?;
    Ok(())
}

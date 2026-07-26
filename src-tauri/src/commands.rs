use crate::activity::demo;
use crate::activity::manager::ActivityManager;
use crate::activity::reducer::{IslandAction, IslandEffect};
use crate::events::{SNAPSHOT_EVENT, WINDOW_READY_EVENT};
use crate::window::controller::WindowController;
use island_model::{ActivityKind, ActivityStatus, IslandMode, IslandSnapshot};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn frontend_ready(manager: State<'_, ActivityManager>) -> Result<IslandSnapshot, String> {
    manager.snapshot().map_err(sanitized_activity_error)
}

#[tauri::command]
pub fn first_frame_ready(
    app: AppHandle,
    manager: State<'_, ActivityManager>,
) -> Result<(), String> {
    let snapshot = manager.snapshot().map_err(sanitized_activity_error)?;
    if snapshot.mode != IslandMode::Hidden {
        WindowController::new(app.clone())
            .apply_mode(snapshot.mode)
            .map_err(sanitized_window_error)?;
    }
    app.emit(WINDOW_READY_EVENT, ())
        .map_err(|_| "could not announce window readiness".to_string())
}

#[tauri::command]
pub fn toggle_expansion(app: AppHandle) -> Result<(), String> {
    dispatch(&app, IslandAction::ToggleExpanded)
}

#[tauri::command]
pub fn dismiss_activity(app: AppHandle, activity_id: String) -> Result<(), String> {
    dispatch(&app, IslandAction::Dismiss(activity_id))
}

#[tauri::command]
pub fn invoke_activity_action(
    app: AppHandle,
    activity_id: String,
    action_id: String,
) -> Result<(), String> {
    let retry = action_id == "retry";
    let kind = if retry {
        app.state::<ActivityManager>()
            .activity(&activity_id)
            .map_err(sanitized_activity_error)?
            .map(|activity| activity.kind)
    } else {
        None
    };
    if let Some(kind) = kind {
        dispatch(&app, IslandAction::Dismiss(activity_id))?;
        return start_demo(&app, kind, false);
    }
    dispatch(
        &app,
        IslandAction::InvokeAction {
            activity_id,
            action_id,
        },
    )?;
    Ok(())
}

#[tauri::command]
pub fn animation_completed(
    app: AppHandle,
    transition_id: u64,
    final_mode: IslandMode,
) -> Result<(), String> {
    dispatch(
        &app,
        IslandAction::AnimationCompleted {
            transition_id,
            final_mode,
        },
    )
}

#[tauri::command]
pub fn run_demo(app: AppHandle, kind: ActivityKind, failed: Option<bool>) -> Result<(), String> {
    start_demo(&app, kind, failed.unwrap_or(false))
}

pub fn start_demo(app: &AppHandle, kind: ActivityKind, failed: bool) -> Result<(), String> {
    let activity = demo::create(kind, failed);
    let id = activity.id.clone();
    dispatch(app, IslandAction::AddActivity(activity))?;

    if matches!(
        kind,
        ActivityKind::Timer | ActivityKind::Recording | ActivityKind::Download
    ) {
        let app = app.clone();
        std::thread::spawn(move || {
            for tick in 1..=60_u64 {
                std::thread::sleep(Duration::from_secs(1));
                let manager = app.state::<ActivityManager>();
                let Ok(Some(mut activity)) = manager.activity(&id) else {
                    break;
                };
                if activity.status == ActivityStatus::Paused {
                    continue;
                }
                if activity.status != ActivityStatus::Running {
                    break;
                }

                match kind {
                    ActivityKind::Timer => {
                        activity.elapsed_ms = Some(tick * 1_000);
                        activity.remaining_ms = Some(60_000_u64.saturating_sub(tick * 1_000));
                        activity.progress = Some((tick as f64 / 60.0).clamp(0.0, 1.0));
                        if tick == 60 {
                            activity.status = ActivityStatus::Completed;
                        }
                    }
                    ActivityKind::Recording => {
                        activity.elapsed_ms = Some(tick * 1_000);
                    }
                    ActivityKind::Download => {
                        let download_tick = tick.min(10);
                        activity.elapsed_ms = Some(download_tick * 1_000);
                        activity.remaining_ms =
                            Some(10_000_u64.saturating_sub(download_tick * 1_000));
                        activity.progress = Some((download_tick as f64 / 10.0).clamp(0.0, 1.0));
                        if download_tick == 10 {
                            activity.status = ActivityStatus::Completed;
                        }
                    }
                    _ => {}
                }

                if dispatch(&app, IslandAction::UpdateActivity(activity)).is_err()
                    || (kind == ActivityKind::Download && tick >= 10)
                {
                    break;
                }
            }
        });
    }
    Ok(())
}

pub fn hide_all(app: &AppHandle) -> Result<(), String> {
    dispatch(app, IslandAction::HideAll)
}

fn dispatch(app: &AppHandle, action: IslandAction) -> Result<(), String> {
    let manager = app.state::<ActivityManager>();
    let (snapshot, effects) = manager.dispatch(action).map_err(sanitized_activity_error)?;
    execute_effects(app, &snapshot, effects)
}

fn execute_effects(
    app: &AppHandle,
    snapshot: &IslandSnapshot,
    effects: Vec<IslandEffect>,
) -> Result<(), String> {
    let controller = WindowController::new(app.clone());
    for effect in effects {
        match effect {
            IslandEffect::PublishSnapshot => app
                .emit(SNAPSHOT_EVENT, snapshot)
                .map_err(|_| "could not publish activity state".to_string())?,
            IslandEffect::ShowWindow => controller
                .apply_mode(snapshot.mode)
                .map_err(sanitized_window_error)?,
            IslandEffect::HideWindow => controller.hide().map_err(sanitized_window_error)?,
            IslandEffect::PrepareExpandedWindow => controller
                .prepare_expanded()
                .map_err(sanitized_window_error)?,
            IslandEffect::PrepareCompactWindow => {
                if snapshot.mode == IslandMode::Attention {
                    controller
                        .show_attention()
                        .map_err(sanitized_window_error)?;
                } else {
                    controller
                        .finalize_compact()
                        .map_err(sanitized_window_error)?;
                }
            }
            IslandEffect::FocusWindow => controller
                .focus_expanded()
                .map_err(sanitized_window_error)?,
            IslandEffect::ResignWindowFocus => {
                controller.resign_focus().map_err(sanitized_window_error)?
            }
            IslandEffect::ScheduleAutoDismiss {
                activity_id,
                after_ms,
            } => {
                let app = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(after_ms));
                    let _ = dispatch(&app, IslandAction::AutoDismissElapsed { activity_id });
                });
            }
        }
    }
    Ok(())
}

fn sanitized_activity_error(_error: crate::activity::manager::ActivityError) -> String {
    #[cfg(debug_assertions)]
    eprintln!("Open Island activity error: {_error:?}");
    "activity operation could not be completed".into()
}

fn sanitized_window_error(_error: crate::window::controller::WindowError) -> String {
    #[cfg(debug_assertions)]
    eprintln!("Open Island window error: {_error:?}");
    "window operation could not be completed".into()
}

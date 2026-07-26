use super::actions::ActivityActions;
use super::activity_icon::ActivityIcon;
use super::compact::compact_time;
use super::progress::Progress;
use crate::bridge;
use island_model::IslandActivity;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn ExpandedIsland(
    activity: IslandActivity,
    queue_count: usize,
    has_notch: bool,
) -> impl IntoView {
    let id = activity.id.clone();
    let dismiss_id = StoredValue::new(id);
    let dismissible = activity.dismissible;
    let title = activity.title.clone();
    let subtitle = activity.subtitle.clone().unwrap_or_default();
    let kind = activity.kind;
    let status = activity.status;
    let progress = activity.progress;
    let time = compact_time(&activity);
    let actions_activity = activity.clone();

    view! {
        <div class="expanded-content">
            <header class="expanded-content__header">
                <ActivityIcon kind=kind status=status/>
                <Show when=move || has_notch>
                    <span class="notch-exclusion" aria-hidden="true"></span>
                </Show>
                <Show when=move || dismissible>
                    <button
                        type="button"
                        class="dismiss"
                        aria-label="Dismiss activity"
                        on:click=move |event| {
                            event.stop_propagation();
                            let id = dismiss_id.get_value();
                            spawn_local(async move {
                                if let Err(error) = bridge::dismiss_activity(&id).await {
                                    bridge::log_error(&error);
                                }
                            });
                        }
                    >
                        "×"
                    </button>
                </Show>
            </header>
            <div class="expanded-content__heading">
                <strong>{title}</strong>
                <span>{subtitle}</span>
            </div>
            <div class="expanded-content__status">
                <Progress value=progress label="Activity progress".into()/>
                <span>{time}</span>
            </div>
            <footer class="expanded-content__footer">
                <Show when=move || queue_count != 0>
                    <span class="queue-count">{format!("+{queue_count} queued")}</span>
                </Show>
                <ActivityActions activity=actions_activity.clone()/>
            </footer>
        </div>
    }
}

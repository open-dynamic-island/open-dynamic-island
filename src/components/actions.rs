use crate::bridge;
use island_model::{ActivityAction, IslandActivity};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn ActivityActions(activity: IslandActivity) -> impl IntoView {
    let activity_id = activity.id.clone();
    view! {
        <div class="actions" aria-label="Activity actions">
            {activity.actions.into_iter().map(move |action| {
                action_button(activity_id.clone(), action)
            }).collect_view()}
        </div>
    }
}

fn action_button(activity_id: String, action: ActivityAction) -> impl IntoView {
    let label = action.label.clone();
    let class = if action.destructive {
        "action action--destructive"
    } else {
        "action"
    };
    view! {
        <button
            type="button"
            class=class
            on:click=move |event| {
                event.stop_propagation();
                let activity_id = activity_id.clone();
                let action_id = action.id.clone();
                spawn_local(async move {
                    if let Err(error) =
                        bridge::invoke_activity_action(&activity_id, &action_id).await
                    {
                        bridge::log_error(&error);
                    }
                });
            }
        >
            {label}
        </button>
    }
}

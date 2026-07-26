use island_model::{ActivityKind, ActivityStatus};
use leptos::prelude::*;

#[component]
pub fn ActivityIcon(kind: ActivityKind, status: ActivityStatus) -> impl IntoView {
    let icon = match (kind, status) {
        (_, ActivityStatus::Failed) => "!",
        (ActivityKind::Timer, _) => "◷",
        (ActivityKind::Recording, _) => "●",
        (ActivityKind::Download, _) => "↓",
        (ActivityKind::Meeting, _) => "⌁",
        (ActivityKind::Notification, _) => "◆",
    };
    let class = if status == ActivityStatus::Failed {
        "activity-icon is-failed"
    } else if kind == ActivityKind::Recording && status == ActivityStatus::Running {
        "activity-icon is-recording"
    } else {
        "activity-icon"
    };
    view! { <span class=class aria-hidden="true">{icon}</span> }
}

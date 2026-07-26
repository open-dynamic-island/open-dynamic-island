use super::activity_icon::ActivityIcon;
use super::progress::Progress;
use island_model::{ActivityKind, IslandActivity};
use leptos::prelude::*;

#[component]
pub fn CompactIsland(activity: IslandActivity, has_notch: bool) -> impl IntoView {
    let time = compact_time(&activity);
    let kind = activity.kind;
    let status = activity.status;
    let progress = activity.progress;
    let title = activity.title.clone();
    view! {
        <div class="compact-content">
            <div class="compact-content__leading">
                <ActivityIcon kind=kind status=status/>
                <span class="compact-content__title">{title}</span>
            </div>
            <Show when=move || has_notch>
                <span
                    class="notch-exclusion"
                    aria-hidden="true"
                ></span>
            </Show>
            <div class="compact-content__trailing">
                <Show
                    when=move || matches!(kind, ActivityKind::Download | ActivityKind::Timer)
                    fallback=move || view! { <span class="status-dot"></span> }
                >
                    <Progress value=progress label="Activity progress".into()/>
                </Show>
                <span class="compact-content__time">{time}</span>
            </div>
        </div>
    }
}

pub fn compact_time(activity: &IslandActivity) -> String {
    if let Some(remaining) = activity.remaining_ms {
        return format_duration(remaining);
    }
    activity.elapsed_ms.map(format_duration).unwrap_or_default()
}

pub fn format_duration(milliseconds: u64) -> String {
    let seconds = milliseconds / 1_000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

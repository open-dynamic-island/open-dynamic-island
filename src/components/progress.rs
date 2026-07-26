use leptos::prelude::*;

#[component]
pub fn Progress(value: Option<f64>, label: String) -> impl IntoView {
    let progress = value.unwrap_or(0.0).clamp(0.0, 1.0);
    let percentage = (progress * 100.0).round() as u32;
    view! {
        <div
            class="progress"
            role="progressbar"
            aria-label=label
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=percentage
        >
            <span class="progress__fill" style=format!("--progress: {percentage}%")></span>
        </div>
    }
}

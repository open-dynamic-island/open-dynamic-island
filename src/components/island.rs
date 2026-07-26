use super::compact::CompactIsland;
use super::expanded::ExpandedIsland;
use crate::app::{AnimationState, AppState, BridgeStatus};
use crate::bridge;
use island_model::IslandMode;
use leptos::ev::{KeyboardEvent, TransitionEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn IslandRoot() -> impl IntoView {
    let state = expect_context::<AppState>();

    let activate = move || {
        spawn_local(async move {
            if let Err(error) = bridge::toggle_expansion().await {
                bridge::log_error(&error);
            }
        });
    };
    let on_click = move |_| activate();
    let on_keydown = move |event: KeyboardEvent| match event.key().as_str() {
        "Enter" | " " => {
            event.prevent_default();
            activate();
        }
        "Escape" if state.snapshot.get_untracked().mode == IslandMode::Expanded => activate(),
        _ => {}
    };
    let on_transition_end = move |event: TransitionEvent| {
        let animation = state.animation.get_untracked();
        let completion = match animation {
            AnimationState::Expanding { transition_id } if event.property_name() == "height" => {
                Some((
                    transition_id,
                    IslandMode::Expanded,
                    AnimationState::Expanded,
                ))
            }
            AnimationState::Collapsing { transition_id } if event.property_name() == "width" => {
                Some((transition_id, IslandMode::Compact, AnimationState::Compact))
            }
            AnimationState::Dismissing { transition_id }
                if event.property_name() == "transform" =>
            {
                Some((transition_id, IslandMode::Hidden, AnimationState::Hidden))
            }
            _ => None,
        };
        if let Some((transition_id, mode, stable)) = completion {
            state.animation.set(stable);
            spawn_local(async move {
                if let Err(error) = bridge::animation_completed(transition_id, mode).await {
                    bridge::log_error(&error);
                }
            });
        }
    };

    let class = move || {
        let snapshot = state.snapshot.get();
        let animation = state.animation.get();
        let state_class = match animation {
            AnimationState::Hidden => "is-hidden",
            AnimationState::Compact => "is-compact",
            AnimationState::Expanding { .. } => "is-expanded is-transitioning",
            AnimationState::Expanded => "is-expanded",
            AnimationState::Collapsing { .. } => "is-compact is-transitioning",
            AnimationState::Dismissing { .. } => "is-dismissing",
        };
        let mode_class = if snapshot.mode == IslandMode::Attention {
            " is-attention"
        } else {
            ""
        };
        let notch_class = if snapshot.display_context.has_notch {
            " has-notch"
        } else {
            ""
        };
        format!("island {state_class}{mode_class}{notch_class}")
    };
    let style = move || {
        let context = state.snapshot.get().display_context;
        let compact_width = if context.has_notch {
            (context.center_exclusion_width + 240.0).clamp(236.0, 400.0)
        } else {
            236.0
        };
        format!(
            "--island-live-compact-width: {compact_width}px; --notch-exclusion-width: {}px",
            context.center_exclusion_width.clamp(110.0, 180.0)
        )
    };

    view! {
        <main class="island-stage">
            <Show when=move || state.bridge_status.get() == BridgeStatus::Failed>
                <div class="bridge-error" role="alert">"Open Island could not connect."</div>
            </Show>
            <article
                class=class
                style=style
                role="button"
                aria-label=move || {
                    if state.snapshot.get().mode == IslandMode::Expanded {
                        "Collapse Open Island"
                    } else {
                        "Expand Open Island"
                    }
                }
                tabindex=move || {
                    if state.snapshot.get().mode == IslandMode::Expanded { "0" } else { "-1" }
                }
                on:click=on_click
                on:keydown=on_keydown
                on:transitionend=on_transition_end
            >
                {move || {
                    let snapshot = state.snapshot.get();
                    snapshot.primary_activity.map(|activity| {
                        if snapshot.mode == IslandMode::Expanded {
                            view! {
                                <ExpandedIsland
                                    activity=activity
                                    queue_count=snapshot.queued_activity_count
                                    has_notch=snapshot.display_context.has_notch
                                />
                            }.into_any()
                        } else {
                            view! {
                                <CompactIsland
                                    activity=activity
                                    has_notch=snapshot.display_context.has_notch
                                />
                            }.into_any()
                        }
                    })
                }}
            </article>
        </main>
    }
}

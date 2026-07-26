use crate::bridge;
use crate::components::island::IslandRoot;
use island_model::{IslandMode, IslandSnapshot};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStatus {
    Connecting,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Hidden,
    Compact,
    Expanding { transition_id: u64 },
    Expanded,
    Collapsing { transition_id: u64 },
    Dismissing { transition_id: u64 },
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub snapshot: RwSignal<IslandSnapshot>,
    pub animation: RwSignal<AnimationState>,
    pub bridge_status: RwSignal<BridgeStatus>,
}

#[component]
pub fn App() -> impl IntoView {
    let state = AppState {
        snapshot: RwSignal::new(IslandSnapshot::default()),
        animation: RwSignal::new(AnimationState::Hidden),
        bridge_status: RwSignal::new(BridgeStatus::Connecting),
    };
    provide_context(state);

    let subscription = StoredValue::new_local(None::<bridge::EventSubscription>);
    on_cleanup(move || subscription.update_value(|value| drop(value.take())));

    spawn_local(async move {
        let snapshot_signal = state.snapshot;
        match bridge::subscribe_to_snapshots(move |next| {
            let previous = snapshot_signal.get_untracked();
            state.animation.set(animation_for(&previous, &next));
            snapshot_signal.set(next);
        })
        .await
        {
            Ok(handle) => subscription.set_value(Some(handle)),
            Err(error) => {
                bridge::log_error(&error);
                state.bridge_status.set(BridgeStatus::Failed);
                return;
            }
        }

        match bridge::frontend_ready().await {
            Ok(snapshot) => {
                state.animation.set(stable_animation(snapshot.mode));
                state.snapshot.set(snapshot);
                state.bridge_status.set(BridgeStatus::Ready);
                if let Err(error) = bridge::first_frame_ready().await {
                    bridge::log_error(&error);
                }
            }
            Err(error) => {
                bridge::log_error(&error);
                state.bridge_status.set(BridgeStatus::Failed);
            }
        }
    });

    view! {
        <IslandRoot/>
    }
}

fn animation_for(previous: &IslandSnapshot, next: &IslandSnapshot) -> AnimationState {
    match (previous.mode, next.mode, next.transition_id) {
        (_, IslandMode::Hidden, Some(transition_id)) => {
            AnimationState::Dismissing { transition_id }
        }
        (
            IslandMode::Compact | IslandMode::Attention,
            IslandMode::Expanded,
            Some(transition_id),
        ) => AnimationState::Expanding { transition_id },
        (IslandMode::Expanded, IslandMode::Compact, Some(transition_id)) => {
            AnimationState::Collapsing { transition_id }
        }
        _ => stable_animation(next.mode),
    }
}

fn stable_animation(mode: IslandMode) -> AnimationState {
    match mode {
        IslandMode::Hidden => AnimationState::Hidden,
        IslandMode::Compact | IslandMode::Attention => AnimationState::Compact,
        IslandMode::Expanded => AnimationState::Expanded,
    }
}

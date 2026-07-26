use island_model::{
    ActivityKind, ActivityStatus, IslandActivity, IslandDisplayContext, IslandMode, IslandSnapshot,
};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ActivityRecord {
    pub activity: IslandActivity,
    pub created_order: u64,
}

#[derive(Debug)]
pub struct ActivityState {
    pub activities: BTreeMap<String, ActivityRecord>,
    pub mode: IslandMode,
    pub revision: u64,
    pub active_transition_id: Option<u64>,
    pub next_transition_id: u64,
    pub next_creation_order: u64,
    pub auto_dismiss_deadline_ms: Option<u64>,
    pub last_user_interaction_ms: u64,
    pub display_context: IslandDisplayContext,
}

impl Default for ActivityState {
    fn default() -> Self {
        Self {
            activities: BTreeMap::new(),
            mode: IslandMode::Hidden,
            revision: 0,
            active_transition_id: None,
            next_transition_id: 1,
            next_creation_order: 1,
            auto_dismiss_deadline_ms: None,
            last_user_interaction_ms: now_ms(),
            display_context: IslandDisplayContext::default(),
        }
    }
}

impl ActivityState {
    pub fn snapshot(&self) -> IslandSnapshot {
        let primary = select_primary(&self.activities);
        IslandSnapshot {
            revision: self.revision,
            mode: self.mode,
            primary_activity: primary.map(|record| record.activity.clone()),
            queued_activity_count: self
                .activities
                .len()
                .saturating_sub(usize::from(primary.is_some())),
            transition_id: self.active_transition_id,
            display_context: self.display_context.clone(),
        }
    }

    fn begin_transition(&mut self) -> u64 {
        let id = self.next_transition_id;
        self.next_transition_id = self.next_transition_id.saturating_add(1);
        self.active_transition_id = Some(id);
        id
    }

    fn changed(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IslandAction {
    AddActivity(IslandActivity),
    UpdateActivity(IslandActivity),
    RemoveActivity(String),
    HideAll,
    ToggleExpanded,
    Collapse,
    Dismiss(String),
    InvokeAction {
        activity_id: String,
        action_id: String,
    },
    AutoDismissElapsed {
        activity_id: String,
    },
    AnimationCompleted {
        transition_id: u64,
        final_mode: IslandMode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IslandEffect {
    PublishSnapshot,
    ShowWindow,
    HideWindow,
    PrepareExpandedWindow,
    PrepareCompactWindow,
    FocusWindow,
    ResignWindowFocus,
    ScheduleAutoDismiss { activity_id: String, after_ms: u64 },
}

pub fn reduce(state: &mut ActivityState, action: IslandAction) -> Vec<IslandEffect> {
    match action {
        IslandAction::AddActivity(activity) => {
            if activity.validate().is_err() {
                return vec![];
            }
            let was_hidden = state.mode == IslandMode::Hidden;
            let id = activity.id.clone();
            let created_order = state.next_creation_order;
            state.next_creation_order = state.next_creation_order.saturating_add(1);
            state.activities.insert(
                id,
                ActivityRecord {
                    activity,
                    created_order,
                },
            );
            if was_hidden {
                state.mode = IslandMode::Compact;
            }
            state.changed();
            let mut effects = vec![];
            if was_hidden {
                effects.extend([IslandEffect::PrepareCompactWindow, IslandEffect::ShowWindow]);
            }
            effects.push(IslandEffect::PublishSnapshot);
            effects
        }
        IslandAction::UpdateActivity(activity) => {
            if activity.validate().is_err() {
                return vec![];
            }
            let Some(record) = state.activities.get_mut(&activity.id) else {
                return vec![];
            };
            let completed_download = record.activity.kind == ActivityKind::Download
                && record.activity.status != ActivityStatus::Completed
                && activity.status == ActivityStatus::Completed;
            record.activity = activity;
            let mut effects = vec![];
            if completed_download && state.mode != IslandMode::Expanded {
                state.mode = IslandMode::Attention;
                state.auto_dismiss_deadline_ms = Some(now_ms().saturating_add(4_000));
                effects.push(IslandEffect::PrepareCompactWindow);
                effects.push(IslandEffect::ScheduleAutoDismiss {
                    activity_id: record.activity.id.clone(),
                    after_ms: 4_000,
                });
            }
            state.changed();
            effects.push(IslandEffect::PublishSnapshot);
            effects
        }
        IslandAction::RemoveActivity(id) | IslandAction::Dismiss(id) => {
            if state.activities.remove(&id).is_none() {
                return vec![];
            }
            state.last_user_interaction_ms = now_ms();
            if state.activities.is_empty() {
                state.mode = IslandMode::Hidden;
                state.begin_transition();
            } else if state.mode == IslandMode::Attention {
                state.mode = IslandMode::Compact;
            }
            state.changed();
            vec![IslandEffect::PublishSnapshot]
        }
        IslandAction::HideAll => {
            if state.activities.is_empty() && state.mode == IslandMode::Hidden {
                return vec![];
            }
            state.activities.clear();
            state.mode = IslandMode::Hidden;
            state.begin_transition();
            state.changed();
            vec![IslandEffect::PublishSnapshot]
        }
        IslandAction::ToggleExpanded => {
            state.last_user_interaction_ms = now_ms();
            match state.mode {
                IslandMode::Compact | IslandMode::Attention => {
                    state.mode = IslandMode::Expanded;
                    state.begin_transition();
                    state.changed();
                    vec![
                        IslandEffect::PrepareExpandedWindow,
                        IslandEffect::FocusWindow,
                        IslandEffect::PublishSnapshot,
                    ]
                }
                IslandMode::Expanded => collapse(state),
                IslandMode::Hidden => vec![],
            }
        }
        IslandAction::Collapse => collapse(state),
        IslandAction::InvokeAction {
            activity_id,
            action_id,
        } => {
            let Some(record) = state.activities.get(&activity_id).cloned() else {
                return vec![];
            };
            match action_id.as_str() {
                "dismiss" | "stop" => reduce(state, IslandAction::Dismiss(activity_id)),
                "pause" => {
                    let mut activity = record.activity;
                    activity.status = ActivityStatus::Paused;
                    if let Some(action) = activity
                        .actions
                        .iter_mut()
                        .find(|action| action.id == "pause")
                    {
                        action.id = "resume".into();
                        action.label = "Resume".into();
                    }
                    reduce(state, IslandAction::UpdateActivity(activity))
                }
                "resume" | "retry" => {
                    let mut activity = record.activity;
                    activity.status = ActivityStatus::Running;
                    activity.progress =
                        activity
                            .progress
                            .map(|value| if action_id == "retry" { 0.0 } else { value });
                    if let Some(action) = activity
                        .actions
                        .iter_mut()
                        .find(|action| action.id == "resume")
                    {
                        action.id = "pause".into();
                        action.label = "Pause".into();
                    }
                    reduce(state, IslandAction::UpdateActivity(activity))
                }
                _ => vec![],
            }
        }
        IslandAction::AutoDismissElapsed { activity_id } => {
            if state.mode == IslandMode::Expanded {
                return vec![];
            }
            reduce(state, IslandAction::Dismiss(activity_id))
        }
        IslandAction::AnimationCompleted {
            transition_id,
            final_mode,
        } => {
            if state.active_transition_id != Some(transition_id) || state.mode != final_mode {
                return vec![];
            }
            state.active_transition_id = None;
            match final_mode {
                IslandMode::Hidden => {
                    vec![IslandEffect::ResignWindowFocus, IslandEffect::HideWindow]
                }
                IslandMode::Compact | IslandMode::Attention => vec![
                    IslandEffect::ResignWindowFocus,
                    IslandEffect::PrepareCompactWindow,
                ],
                IslandMode::Expanded => vec![],
            }
        }
    }
}

fn collapse(state: &mut ActivityState) -> Vec<IslandEffect> {
    if state.mode != IslandMode::Expanded {
        return vec![];
    }
    state.mode = IslandMode::Compact;
    state.begin_transition();
    state.changed();
    vec![IslandEffect::PublishSnapshot]
}

pub fn select_primary(activities: &BTreeMap<String, ActivityRecord>) -> Option<&ActivityRecord> {
    activities.values().min_by_key(|record| {
        (
            category_rank(&record.activity),
            std::cmp::Reverse(record.activity.priority),
            record.created_order,
        )
    })
}

fn category_rank(activity: &IslandActivity) -> u8 {
    if activity.status == ActivityStatus::Failed {
        return 0;
    }
    match (activity.kind, activity.status) {
        (ActivityKind::Recording, _) => 1,
        (ActivityKind::Meeting, _) => 2,
        (ActivityKind::Timer, ActivityStatus::Running) => 3,
        (ActivityKind::Download, _) => 4,
        (ActivityKind::Notification, _) => 5,
        (ActivityKind::Timer, _) => 6,
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use island_model::ActivityAction;

    fn demo(id: &str, kind: ActivityKind, status: ActivityStatus) -> IslandActivity {
        IslandActivity {
            id: id.into(),
            kind,
            title: id.into(),
            subtitle: None,
            status,
            progress: None,
            elapsed_ms: None,
            remaining_ms: None,
            priority: 1,
            dismissible: true,
            actions: vec![ActivityAction {
                id: "dismiss".into(),
                label: "Dismiss".into(),
                destructive: false,
            }],
        }
    }

    #[test]
    fn adding_first_activity_shows_compact() {
        let mut state = ActivityState::default();
        let effects = reduce(
            &mut state,
            IslandAction::AddActivity(demo("timer", ActivityKind::Timer, ActivityStatus::Running)),
        );
        assert_eq!(state.mode, IslandMode::Compact);
        assert!(effects.contains(&IslandEffect::ShowWindow));
    }

    #[test]
    fn higher_category_priority_becomes_primary() {
        let mut state = ActivityState::default();
        reduce(
            &mut state,
            IslandAction::AddActivity(demo(
                "download",
                ActivityKind::Download,
                ActivityStatus::Running,
            )),
        );
        reduce(
            &mut state,
            IslandAction::AddActivity(demo(
                "recording",
                ActivityKind::Recording,
                ActivityStatus::Running,
            )),
        );
        assert_eq!(state.snapshot().primary_activity.unwrap().id, "recording");
    }

    #[test]
    fn stale_animation_completion_is_ignored() {
        let mut state = ActivityState::default();
        reduce(
            &mut state,
            IslandAction::AddActivity(demo("timer", ActivityKind::Timer, ActivityStatus::Running)),
        );
        reduce(&mut state, IslandAction::ToggleExpanded);
        let effects = reduce(
            &mut state,
            IslandAction::AnimationCompleted {
                transition_id: 999,
                final_mode: IslandMode::Expanded,
            },
        );
        assert!(effects.is_empty());
        assert!(state.active_transition_id.is_some());
    }

    #[test]
    fn completed_download_enters_attention() {
        let mut state = ActivityState::default();
        let mut download = demo("download", ActivityKind::Download, ActivityStatus::Running);
        download.progress = Some(0.9);
        reduce(&mut state, IslandAction::AddActivity(download.clone()));
        download.progress = Some(1.0);
        download.status = ActivityStatus::Completed;
        reduce(&mut state, IslandAction::UpdateActivity(download));
        assert_eq!(state.mode, IslandMode::Attention);
    }

    #[test]
    fn expanded_activity_does_not_auto_dismiss() {
        let mut state = ActivityState::default();
        reduce(
            &mut state,
            IslandAction::AddActivity(demo(
                "notice",
                ActivityKind::Notification,
                ActivityStatus::Completed,
            )),
        );
        reduce(&mut state, IslandAction::ToggleExpanded);
        reduce(
            &mut state,
            IslandAction::AutoDismissElapsed {
                activity_id: "notice".into(),
            },
        );
        assert!(state.activities.contains_key("notice"));
    }
}

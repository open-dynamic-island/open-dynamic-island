use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    Timer,
    Recording,
    Download,
    Meeting,
    Notification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IslandMode {
    Hidden,
    Compact,
    Expanded,
    Attention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityAction {
    pub id: String,
    pub label: String,
    pub destructive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IslandActivity {
    pub id: String,
    pub kind: ActivityKind,
    pub title: String,
    pub subtitle: Option<String>,
    pub status: ActivityStatus,
    pub progress: Option<f64>,
    pub elapsed_ms: Option<u64>,
    pub remaining_ms: Option<u64>,
    pub priority: u8,
    pub dismissible: bool,
    pub actions: Vec<ActivityAction>,
}

impl IslandActivity {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::EmptyActivityId);
        }
        if self
            .progress
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(ModelError::InvalidProgress);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IslandSnapshot {
    pub revision: u64,
    pub mode: IslandMode,
    pub primary_activity: Option<IslandActivity>,
    pub queued_activity_count: usize,
    pub transition_id: Option<u64>,
    pub display_context: IslandDisplayContext,
}

impl Default for IslandSnapshot {
    fn default() -> Self {
        Self {
            revision: 0,
            mode: IslandMode::Hidden,
            primary_activity: None,
            queued_activity_count: 0,
            transition_id: None,
            display_context: IslandDisplayContext::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IslandDisplayContext {
    pub has_notch: bool,
    pub center_exclusion_width: f64,
}

impl Default for IslandDisplayContext {
    fn default() -> Self {
        Self {
            has_notch: false,
            center_exclusion_width: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    #[error("activity ID cannot be empty")]
    EmptyActivityId,
    #[error("progress must be a finite number between 0 and 1")]
    InvalidProgress,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(progress: Option<f64>) -> IslandActivity {
        IslandActivity {
            id: "stable-demo-id".into(),
            kind: ActivityKind::Download,
            title: "Download".into(),
            subtitle: None,
            status: ActivityStatus::Running,
            progress,
            elapsed_ms: None,
            remaining_ms: None,
            priority: 1,
            dismissible: true,
            actions: vec![],
        }
    }

    #[test]
    fn enum_representation_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&ActivityKind::Notification).unwrap(),
            "\"notification\""
        );
    }

    #[test]
    fn snapshot_round_trips() {
        let snapshot = IslandSnapshot {
            primary_activity: Some(activity(Some(0.25))),
            ..IslandSnapshot::default()
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert_eq!(
            serde_json::from_str::<IslandSnapshot>(&json).unwrap(),
            snapshot
        );
    }

    #[test]
    fn progress_is_validated() {
        assert!(activity(Some(0.0)).validate().is_ok());
        assert!(activity(Some(1.0)).validate().is_ok());
        assert_eq!(
            activity(Some(1.01)).validate(),
            Err(ModelError::InvalidProgress)
        );
        assert_eq!(
            activity(Some(f64::NAN)).validate(),
            Err(ModelError::InvalidProgress)
        );
    }

    #[test]
    fn activity_id_is_stable_across_serialization() {
        let before = activity(None);
        let after: IslandActivity =
            serde_json::from_str(&serde_json::to_string(&before).unwrap()).unwrap();
        assert_eq!(after.id, "stable-demo-id");
    }
}

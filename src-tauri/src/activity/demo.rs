use island_model::{ActivityAction, ActivityKind, ActivityStatus, IslandActivity};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DEMO_ID: AtomicU64 = AtomicU64::new(1);

pub fn create(kind: ActivityKind, failed: bool) -> IslandActivity {
    let sequence = NEXT_DEMO_ID.fetch_add(1, Ordering::Relaxed);
    let id = format!("demo-{}-{sequence}", kind_name(kind));
    let dismiss = ActivityAction {
        id: "dismiss".into(),
        label: "Dismiss".into(),
        destructive: false,
    };
    match (kind, failed) {
        (ActivityKind::Timer, _) => IslandActivity {
            id,
            kind,
            title: "Focus timer".into(),
            subtitle: Some("One minute demo".into()),
            status: ActivityStatus::Running,
            progress: Some(0.0),
            elapsed_ms: Some(0),
            remaining_ms: Some(60_000),
            priority: 40,
            dismissible: true,
            actions: vec![
                ActivityAction {
                    id: "pause".into(),
                    label: "Pause".into(),
                    destructive: false,
                },
                dismiss,
            ],
        },
        (ActivityKind::Recording, _) => IslandActivity {
            id,
            kind,
            title: "Recording".into(),
            subtitle: Some("Demo only — microphone is not used".into()),
            status: ActivityStatus::Running,
            progress: None,
            elapsed_ms: Some(0),
            remaining_ms: None,
            priority: 90,
            dismissible: false,
            actions: vec![ActivityAction {
                id: "stop".into(),
                label: "Stop".into(),
                destructive: true,
            }],
        },
        (ActivityKind::Download, _) => IslandActivity {
            id,
            kind,
            title: "Downloading update".into(),
            subtitle: Some("Open Island demo package".into()),
            status: ActivityStatus::Running,
            progress: Some(0.0),
            elapsed_ms: Some(0),
            remaining_ms: Some(10_000),
            priority: 30,
            dismissible: true,
            actions: vec![dismiss],
        },
        (ActivityKind::Meeting, _) => IslandActivity {
            id,
            kind,
            title: "Design review".into(),
            subtitle: Some("Starts in 5 minutes".into()),
            status: ActivityStatus::Pending,
            progress: None,
            elapsed_ms: None,
            remaining_ms: Some(300_000),
            priority: 60,
            dismissible: true,
            actions: vec![dismiss],
        },
        (ActivityKind::Notification, true) => IslandActivity {
            id,
            kind,
            title: "Demo failed".into(),
            subtitle: Some("The simulated operation could not finish.".into()),
            status: ActivityStatus::Failed,
            progress: None,
            elapsed_ms: None,
            remaining_ms: None,
            priority: 100,
            dismissible: true,
            actions: vec![
                ActivityAction {
                    id: "retry".into(),
                    label: "Retry".into(),
                    destructive: false,
                },
                dismiss,
            ],
        },
        (ActivityKind::Notification, false) => IslandActivity {
            id,
            kind,
            title: "Open Island is ready".into(),
            subtitle: Some("This is a local demo notification.".into()),
            status: ActivityStatus::Completed,
            progress: None,
            elapsed_ms: None,
            remaining_ms: None,
            priority: 10,
            dismissible: true,
            actions: vec![dismiss],
        },
    }
}

fn kind_name(kind: ActivityKind) -> &'static str {
    match kind {
        ActivityKind::Timer => "timer",
        ActivityKind::Recording => "recording",
        ActivityKind::Download => "download",
        ActivityKind::Meeting => "meeting",
        ActivityKind::Notification => "notification",
    }
}

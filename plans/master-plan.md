# Codex Implementation Directive: Open Island for macOS

## Context

You are working inside an existing Tauri + Leptos template project named:

```text
open-island
```

The project already runs on macOS.

Build an initial production-quality implementation of a Dynamic Island-style macOS overlay. The application should display a small pill-shaped interface at the top center of the screen, adjacent to the MacBook notch when one exists.

The first implementation should provide:

* A compact island.
* An expanded island.
* Smooth transitions between states.
* A demo activity system.
* Native macOS window positioning.
* Notch-aware placement.
* Always-on-top behavior.
* Visibility across macOS Spaces.
* No normal application window.
* No Dock icon during normal operation.
* No focus stealing while compact.
* A minimal menu-bar item for demo actions and quitting.

Do not attempt to integrate with private macOS system events such as AirPods pairing, Face ID, camera usage, system calls, or other applications’ private state.

---

# 1. Development principles

Follow these rules throughout the implementation.

## 1.1 Inspect before modifying

Before making changes:

1. Inspect the root `Cargo.toml`.
2. Inspect `src-tauri/Cargo.toml`.
3. Inspect `src-tauri/tauri.conf.json` or the equivalent Tauri configuration.
4. Inspect `Trunk.toml`.
5. Inspect `index.html`.
6. Inspect the existing Leptos entry point and component structure.
7. Run the existing application once with its current development command.
8. Record the detected versions of:

   * Tauri
   * Tauri CLI
   * Leptos
   * Trunk
   * Rust edition
   * macOS minimum deployment target

Do not blindly replace the template structure.

Do not upgrade dependencies unless an API required by this implementation is unavailable in the installed version.

Adapt all code to the APIs available in the project’s installed versions.

## 1.2 Preserve build commands

The following workflows must continue working:

```bash
cargo fmt
cargo check
cargo test
cargo tauri dev
cargo tauri build
```

Use the project’s existing equivalent commands if the template uses a wrapper such as `npm`, `pnpm`, `just`, or `cargo make`.

## 1.3 Keep frontend and native responsibilities separate

The Leptos frontend owns:

* Rendering.
* Visual state.
* CSS animations.
* Pointer and keyboard interactions.
* Accessible labels.
* Displaying activities.

The Tauri Rust backend owns:

* The authoritative activity state.
* Window creation.
* Window positioning.
* Window sizing.
* Focus behavior.
* macOS monitor detection.
* Notch detection.
* Spaces and full-screen behavior.
* Menu-bar integration.
* Application lifecycle.

The frontend must not calculate macOS screen coordinates.

The frontend must not directly manipulate the Tauri window through JavaScript window APIs. Expose a small set of typed Tauri commands instead.

## 1.4 Keep platform-specific code isolated

All AppKit or macOS-specific code must live behind a module such as:

```text
src-tauri/src/platform/macos/
```

The rest of the application must not directly import AppKit types.

Add non-macOS fallback implementations where needed so `cargo check` remains structurally clean, even though the product is currently macOS-only.

---

# 2. Architecture

Use this approximate structure, adapting it to the existing template:

```text
open-island/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── crates/
│   └── island-model/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── bridge.rs
│   ├── state.rs
│   ├── components/
│   │   ├── mod.rs
│   │   ├── island.rs
│   │   ├── compact.rs
│   │   ├── expanded.rs
│   │   ├── activity_icon.rs
│   │   ├── progress.rs
│   │   └── actions.rs
│   └── styles/
│       ├── reset.css
│       ├── island.css
│       └── animations.css
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── commands.rs
        ├── events.rs
        ├── activity/
        │   ├── mod.rs
        │   ├── manager.rs
        │   ├── reducer.rs
        │   └── demo.rs
        ├── window/
        │   ├── mod.rs
        │   ├── controller.rs
        │   ├── layout.rs
        │   └── constants.rs
        ├── tray/
        │   ├── mod.rs
        │   └── menu.rs
        └── platform/
            ├── mod.rs
            └── macos/
                ├── mod.rs
                ├── appkit.rs
                ├── screen.rs
                ├── panel.rs
                └── notifications.rs
```

Do not create empty abstraction files merely to match this tree. Use the structure where it clarifies responsibility.

---

# 3. Shared model crate

Create a small shared crate named `island-model` that can compile for:

* Native Rust.
* `wasm32-unknown-unknown`.

Keep it free of Tauri, Leptos, AppKit, filesystem, and operating-system dependencies.

Use only lightweight dependencies such as `serde`.

Avoid native-only timestamp libraries in the shared model. Represent timestamps as integer Unix milliseconds or use duration values.

## 3.1 Core types

Create types approximately equivalent to:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    Timer,
    Recording,
    Download,
    Meeting,
    Notification,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IslandMode {
    Hidden,
    Compact,
    Expanded,
    Attention,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityAction {
    pub id: String,
    pub label: String,
    pub destructive: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IslandSnapshot {
    pub revision: u64,
    pub mode: IslandMode,
    pub primary_activity: Option<IslandActivity>,
    pub queued_activity_count: usize,
}
```

Validate progress values in backend logic:

```text
0.0 <= progress <= 1.0
```

Do not rely on floating-point equality for rendering decisions.

## 3.2 Authoritative state

The Tauri backend is the authoritative source of `IslandSnapshot`.

The Leptos frontend may hold a local copy of the latest snapshot, but it must not independently decide persistent activity state.

The frontend may maintain transient animation state such as:

* Entering.
* Expanding.
* Expanded.
* Collapsing.
* Exiting.

---

# 4. Tauri and Leptos bridge

Use the existing Leptos/Trunk integration.

Confirm that Tauri’s global API bridge is enabled when required by the installed template:

```json
{
  "app": {
    "withGlobalTauri": true
  }
}
```

Tauri’s Leptos integration uses a static/CSR frontend rather than a server-backed Leptos configuration. Do not add Leptos SSR.

Create one frontend bridge module:

```text
src/bridge.rs
```

This must be the only frontend module that knows about JavaScript interop or `window.__TAURI__`.

Expose typed Rust functions approximately like:

```rust
pub async fn frontend_ready() -> Result<IslandSnapshot, BridgeError>;
pub async fn toggle_expansion() -> Result<(), BridgeError>;
pub async fn dismiss_activity(activity_id: &str) -> Result<(), BridgeError>;
pub async fn invoke_activity_action(
    activity_id: &str,
    action_id: &str,
) -> Result<(), BridgeError>;
pub async fn animation_completed(
    transition_id: u64,
    final_mode: IslandMode,
) -> Result<(), BridgeError>;
pub async fn run_demo(kind: ActivityKind) -> Result<(), BridgeError>;
```

Expose a subscription API:

```rust
pub fn subscribe_to_snapshots(
    callback: impl Fn(IslandSnapshot) + 'static,
) -> Result<EventSubscription, BridgeError>;
```

The subscription handle must unregister its event listener when dropped, where practical.

## 4.1 Event names

Centralize event names instead of scattering string literals.

Use names similar to:

```text
open-island://snapshot
open-island://request-animation
open-island://window-ready
```

Do not expose low-level AppKit events to the frontend.

## 4.2 Initial handshake

Use this startup sequence:

1. Create the native window hidden.
2. Load the Leptos application.
3. Initialize frontend signals.
4. Register backend event listeners.
5. Frontend invokes `frontend_ready`.
6. Backend returns the current snapshot.
7. Frontend renders the snapshot.
8. Frontend reports that the first frame is ready.
9. Backend positions the native window.
10. Backend shows the window only when there is visible activity.

This must avoid:

* A white flash.
* A rectangular flash.
* The window appearing in the wrong position.
* A stale event being lost before listeners are registered.

---

# 5. Backend activity manager

Create an `ActivityManager` stored as Tauri-managed application state.

A suitable shape is:

```rust
pub struct ActivityManager {
    inner: std::sync::Mutex<ActivityState>,
}
```

Use an async lock only if the project already uses an async execution model that requires it. Do not introduce Tokio solely for an in-memory mutex.

## 5.1 Activity state

Maintain:

* Activities keyed by ID.
* Current primary activity.
* Current island mode.
* Monotonic revision number.
* Active transition ID.
* Optional auto-dismiss deadline.
* Last user interaction time.

## 5.2 Activity priority

Select the primary activity using deterministic rules:

1. Failed activities requiring attention.
2. Recording activities.
3. Meetings.
4. Running timers.
5. Downloads.
6. Informational notifications.
7. Higher explicit priority.
8. Earlier creation time as a stable tiebreaker.

Put priority logic in a pure function.

## 5.3 Reducer

Implement activity behavior through explicit actions instead of arbitrary state mutation.

Example backend actions:

```rust
pub enum IslandAction {
    AddActivity(IslandActivity),
    UpdateActivity(IslandActivity),
    RemoveActivity(String),
    ToggleExpanded,
    Expand,
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
```

Use a reducer-like function:

```rust
pub fn reduce(
    state: &mut ActivityState,
    action: IslandAction,
) -> Vec<IslandEffect>;
```

Effects may include:

```rust
pub enum IslandEffect {
    PublishSnapshot,
    ShowWindow,
    HideWindow,
    PrepareExpandedWindow,
    PrepareCompactWindow,
    FocusWindow,
    ResignWindowFocus,
    ScheduleAutoDismiss {
        activity_id: String,
        after_ms: u64,
    },
}
```

Keep state decisions separate from Tauri and AppKit side effects.

## 5.4 Revisions and transition IDs

Increment `revision` every time the externally visible snapshot changes.

Use a separate transition ID for asynchronous animation completion.

Ignore animation completion messages whose transition ID is no longer current. This prevents rapid clicks from letting an old timer collapse a newly expanded island.

---

# 6. Demo activities

Implement built-in demo activities so the UI can be tested without external integrations.

Support:

* Timer.
* Recording.
* Download.
* Meeting.
* Notification.
* Failure notification.

## 6.1 Timer demo

Create a 60-second countdown.

Update the displayed remaining time no more frequently than needed for the UI. A one-second interval is sufficient.

## 6.2 Download demo

Increase progress from `0.0` to `1.0` over approximately 10 seconds.

When complete:

1. Set status to `Completed`.
2. Show the island in attention mode.
3. Auto-dismiss after several seconds unless expanded.

## 6.3 Recording demo

Display:

* Recording indicator.
* Elapsed duration.
* Stop action.

The recording demo does not need to access the microphone.

## 6.4 Failure demo

Display a failed activity with:

* Error title.
* Short detail.
* Retry action.
* Dismiss action.

Retry can restart the demo.

---

# 7. Native window creation

Do not rely on the template’s default visible `main` window.

Prefer creating the island window programmatically during Tauri setup.

Use the label:

```text
island
```

Initial properties should be equivalent to:

```rust
WebviewWindowBuilder::new(app, "island", WebviewUrl::default())
    .title("Open Island")
    .inner_size(COMPACT_WIDTH, COMPACT_HEIGHT)
    .min_inner_size(COMPACT_WIDTH, COMPACT_HEIGHT)
    .decorations(false)
    .resizable(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .focusable(false)
    .focused(false)
    .visible(false)
    .transparent(true)
    .shadow(false)
```

Adapt method names to the installed Tauri version.

Tauri currently provides builder-level support for always-on-top, focusability, visibility across workspaces, decorations, visibility, and transparency.

Do not depend on `skip_taskbar` for macOS behavior because that builder option is not supported on macOS. Use the application activation policy instead.

During setup on macOS, set:

```rust
app.handle()
    .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
```

This is the supported Tauri API for an accessory-style macOS application.

## 7.1 Transparency distribution constraint

For this first implementation, direct distribution is acceptable.

If the project requires Tauri’s `macos-private-api` feature for full transparent-window behavior, enable it only in the macOS configuration and document the consequence in `README.md`.

Tauri warns that its private macOS API mode prevents Mac App Store acceptance.

Do not claim that this MVP is Mac App Store-ready.

## 7.2 Window constants

Start with constants similar to:

```rust
pub const COMPACT_WIDTH: f64 = 220.0;
pub const COMPACT_HEIGHT: f64 = 40.0;

pub const EXPANDED_WIDTH: f64 = 420.0;
pub const EXPANDED_HEIGHT: f64 = 176.0;

pub const ATTENTION_WIDTH: f64 = 300.0;
pub const ATTENTION_HEIGHT: f64 = 52.0;

pub const NON_NOTCH_TOP_MARGIN: f64 = 6.0;
pub const WINDOW_RESIZE_ANIMATION_MS: u64 = 240;
```

Keep them centralized.

Do not scatter dimensions across Rust and CSS. Expose them as CSS custom properties or keep matching values documented in a single frontend constants module.

---

# 8. Window controller

Create a `WindowController` responsible for translating an island mode into a native layout.

Example interface:

```rust
pub struct WindowController {
    app: tauri::AppHandle,
}

impl WindowController {
    pub fn initialize(&self) -> Result<(), WindowError>;
    pub fn show_compact(&self) -> Result<(), WindowError>;
    pub fn show_attention(&self) -> Result<(), WindowError>;
    pub fn prepare_expanded(&self) -> Result<(), WindowError>;
    pub fn finalize_compact(&self) -> Result<(), WindowError>;
    pub fn hide(&self) -> Result<(), WindowError>;
    pub fn focus_expanded(&self) -> Result<(), WindowError>;
    pub fn resign_focus(&self) -> Result<(), WindowError>;
    pub fn reposition(&self) -> Result<(), WindowError>;
}
```

All window mutations must happen on the appropriate Tauri/AppKit main thread.

Do not call AppKit window methods from an arbitrary background thread.

## 8.1 Top-center anchor

All island layouts must preserve the top-center anchor.

When width changes:

```text
new_x = screen_center_x - new_width / 2
```

When height changes, the top edge must stay fixed.

The island should expand downward and outward, not jump vertically.

## 8.2 Expand sequence

Use this order:

1. User clicks the compact island.
2. Frontend invokes `toggle_expansion`.
3. Backend creates a new transition ID.
4. Backend enlarges and repositions the native window while preserving the top-center anchor.
5. Backend enables interactive focus behavior.
6. Backend emits an expanded snapshot or animation request.
7. Frontend waits for the next animation frame.
8. Frontend applies the expanded CSS class.
9. Frontend reports animation completion.

The native window must become large enough before the visual island expands, otherwise the content will be clipped.

## 8.3 Collapse sequence

Use this order:

1. Backend emits a collapsing state with transition ID.
2. Frontend animates the island from expanded to compact.
3. Frontend reports animation completion.
4. Backend verifies that the transition ID is current.
5. Backend resigns key-window status.
6. Backend disables focusability.
7. Backend shrinks and repositions the native window.
8. Backend publishes the stable compact snapshot.

Do not make a focused window non-focusable without first resigning focus.

Tauri documents a macOS limitation where making an already focused window non-focusable does not itself unfocus it.

## 8.4 Hiding

When there are no activities:

1. Animate the visual island out.
2. Wait for animation completion.
3. Hide the native window.
4. Keep the WebView alive.
5. Do not destroy and recreate the WebView for every notification.

---

# 9. macOS screen and notch detection

Use AppKit through `objc2-app-kit` in the macOS platform module.

Add only the required crate features.

Do not add a broad Cocoa dependency if `objc2-app-kit` can provide the needed APIs.

`objc2-app-kit` exposes:

* `NSScreen.safeAreaInsets`
* `NSScreen.auxiliaryTopLeftArea`
* `NSScreen.auxiliaryTopRightArea`

## 9.1 Internal screen model

Convert native screen data into a platform-neutral structure:

```rust
pub struct ScreenGeometry {
    pub frame_x: f64,
    pub frame_y: f64,
    pub frame_width: f64,
    pub frame_height: f64,

    pub visible_x: f64,
    pub visible_y: f64,
    pub visible_width: f64,
    pub visible_height: f64,

    pub safe_top: f64,
    pub safe_left: f64,
    pub safe_right: f64,
    pub safe_bottom: f64,

    pub auxiliary_left_width: Option<f64>,
    pub auxiliary_right_width: Option<f64>,
    pub scale_factor: f64,
}
```

Do not expose `NSScreen`, `NSRect`, or `NSEdgeInsets` outside the macOS module.

## 9.2 Notch detection

Treat a screen as notch-bearing when:

* The top safe-area inset is meaningfully greater than zero.
* The top auxiliary regions indicate a gap around the center camera housing.

Do not hardcode Mac model identifiers.

Calculate an estimated notch width from the auxiliary regions when reliable.

Apply reasonable clamping:

```text
collapsed_width = max(default_compact_width, estimated_notch_width)
```

Do not let abnormal screen metrics produce an excessively large island.

## 9.3 Positioning rules

For a notch-bearing display:

* Center the island horizontally on the physical screen.
* Place the top edge flush with the top screen edge.
* Let the visual pill extend downward from the notch area.
* Do not place meaningful text underneath the physical camera housing.
* Keep critical compact content on the leading and trailing sides.

For a display without a notch:

* Center the island horizontally.
* Place it below the menu bar or visible-frame top.
* Apply a small top margin.

Create a pure function:

```rust
pub fn calculate_island_frame(
    screen: &ScreenGeometry,
    width: f64,
    height: f64,
) -> IslandFrame;
```

Unit-test this function with:

* Notched built-in display.
* Non-notched external display.
* Left-positioned secondary monitor.
* Monitor with a negative X origin.
* Monitor with a negative Y origin.
* Retina scale factor.
* Very narrow display.

## 9.4 Coordinate systems

AppKit and Tauri may expose screen coordinates using different origins or logical/physical units.

Centralize all coordinate conversion in one module.

Do not fix coordinate bugs by adding arbitrary offsets in UI code.

Add debug logging in development builds for:

```text
selected screen
screen frame
visible frame
safe-area insets
auxiliary areas
scale factor
calculated island frame
```

## 9.5 Active display policy

For the MVP:

* Use the screen containing the island window when already visible.
* Otherwise use the primary/main screen.
* Keep the island on that screen while its current activity remains active.

Do not make the island jump between displays every time the mouse moves.

Later, this policy can become configurable.

## 9.6 Display changes

Reposition the island when:

* Display arrangement changes.
* A display is connected or disconnected.
* Resolution changes.
* Scaling changes.
* The selected display disappears.

Use an AppKit screen-parameter notification where appropriate, or the closest stable event available through the installed libraries.

Debounce repeated display-change events.

---

# 10. AppKit window behavior

Start with a normal Tauri `WebviewWindow`.

Configure the underlying macOS window through public AppKit APIs where safely available.

Desired behavior:

* Transparent background.
* Opaque flag disabled.
* No native title bar.
* No native shadow, unless visual testing shows a subtle shadow is beneficial.
* Always above normal application windows.
* Visible across Spaces.
* Suitable for full-screen auxiliary presentation.
* Stationary during Mission Control where supported.

Apple exposes window collection behaviors including:

* `canJoinAllSpaces`
* `stationary`
* `fullScreenAuxiliary`
* Newer Stage Manager/full-screen overlay behavior such as `canJoinAllApplications`

Do not combine mutually exclusive Stage Manager/full-screen roles. Apple documents that primary, auxiliary, and can-join-all-applications roles are mutually exclusive.

Prefer conservative, broadly supported behavior for the MVP:

```text
canJoinAllSpaces
stationary
fullScreenAuxiliary
```

Use availability checks for APIs that require newer macOS versions.

## 10.1 Do not require NSPanel in the first milestone

Do not introduce a third-party NSPanel dependency until the standard Tauri window satisfies the following:

* Correct rendering.
* Correct notch positioning.
* Correct resizing.
* Correct focus behavior.
* Correct all-Spaces behavior.
* Stable animations.

After the standard-window milestone, evaluate converting it to a true `NSPanel`.

A current community project, `tauri-nspanel`, supports creating or converting Tauri windows into macOS panels, but it is not part of Tauri itself. Pin any eventual use to an explicit compatible revision rather than an unbounded Git branch.

Keep the platform adapter designed so the implementation can later change from:

```text
StandardTauriWindowAdapter
```

to:

```text
NsPanelWindowAdapter
```

without changing activity or frontend logic.

---

# 11. Leptos frontend state

Create one top-level application state structure.

A suitable shape is:

```rust
#[derive(Clone)]
pub struct AppState {
    pub snapshot: RwSignal<IslandSnapshot>,
    pub animation: RwSignal<AnimationState>,
    pub bridge_status: RwSignal<BridgeStatus>,
}
```

Possible animation state:

```rust
pub enum AnimationState {
    Hidden,
    Appearing { transition_id: u64 },
    Compact,
    Expanding { transition_id: u64 },
    Expanded,
    Collapsing { transition_id: u64 },
    Dismissing { transition_id: u64 },
}
```

Avoid having individual components maintain contradictory copies of island mode.

## 11.1 Main component hierarchy

Use approximately:

```text
App
└── IslandRoot
    ├── CompactIsland
    ├── ExpandedIsland
    ├── AttentionIsland
    └── ActivityActions
```

Keep both compact and expanded content simple.

## 11.2 Compact island

Display:

* Activity icon.
* Short title or status.
* Progress ring/bar or elapsed time.
* A trailing status indicator.

For a notched display, support a layout where content sits to the left and right of a center exclusion zone.

The frontend can receive an optional notch-layout hint from the backend, such as:

```rust
pub struct IslandDisplayContext {
    pub has_notch: bool,
    pub center_exclusion_width: f64,
}
```

Do not let the frontend calculate this from browser dimensions.

## 11.3 Expanded island

Display:

* Activity title.
* Subtitle.
* Progress.
* Elapsed or remaining time.
* Available actions.
* Dismiss control when allowed.
* Queue count when more activities exist.

Keep the expanded interface compact enough to remain a floating utility rather than becoming a normal application window.

## 11.4 Interaction rules

Compact:

* Click expands.
* Enter or Space expands when keyboard focus is intentionally enabled.
* Hover may produce a subtle emphasis.
* Do not implement hover-to-expand in the initial milestone.

Expanded:

* Click outside the content or press Escape to collapse.
* Clicking an action invokes the backend.
* Destructive actions require visually distinct treatment.
* Avoid confirmation dialogs for demo actions.

---

# 12. Styling and animation

Use plain CSS unless the existing project already has a styling system.

Do not add a large UI framework.

## 12.1 Root document

Set:

```css
html,
body,
#root {
    width: 100%;
    height: 100%;
    margin: 0;
    background: transparent;
    overflow: hidden;
}
```

Also disable:

* Browser scrollbars.
* Default body padding.
* Accidental text selection on non-text controls.
* Default focus outlines only when replaced with an accessible custom outline.

## 12.2 Island surface

The visual island should have:

* Near-black background.
* Fully rounded compact state.
* Larger rounded rectangle in expanded state.
* Subtle inner highlight.
* Optional subtle shadow.
* High-contrast text.
* Proper clipping for internal content.
* No visible native WebView rectangle.

Use CSS custom properties:

```css
:root {
    --island-compact-width: 220px;
    --island-compact-height: 40px;
    --island-expanded-width: 420px;
    --island-expanded-height: 176px;
    --island-animation-duration: 240ms;
}
```

## 12.3 Animation properties

Prefer animating:

* `transform`
* `opacity`
* `border-radius`
* Internal content opacity and translation

Avoid repeatedly animating expensive layout properties inside the WebView.

The native window resize and the WebView animation must be explicitly sequenced.

Use an easing curve with a responsive spring-like feel, but do not add an animation framework for the first milestone.

## 12.4 Reduced motion

Respect:

```css
@media (prefers-reduced-motion: reduce)
```

When reduced motion is enabled:

* Remove scaling.
* Shorten or remove transitions.
* Preserve all functionality.

## 12.5 Continuous indicators

Use CSS or SVG for:

* Progress rings.
* Recording pulse.
* Download progress.

Do not use Canvas unless there is a demonstrated need.

Do not implement a high-frequency audio waveform in the MVP.

---

# 13. Focus and activation behavior

The compact island must not steal keyboard focus from the active application.

The expanded island may temporarily accept input.

Implement focus behavior through the native window adapter.

## 13.1 Compact state

* Window is visible.
* Window remains non-key.
* Window is non-focusable.
* Clicking it may trigger expansion through mouse handling without disrupting the currently active application more than necessary.

## 13.2 Expanded state

Before showing interactive controls:

1. Make the window focusable.
2. Bring it forward.
3. Make it key only when required.
4. Focus the first meaningful control only when appropriate.

Do not automatically focus a destructive action.

## 13.3 Returning to compact

Before setting the window to non-focusable:

1. Resign key-window status.
2. Collapse visual content.
3. Disable focusability.
4. Restore compact frame.

Ensure the previously active application can continue receiving keyboard input.

Add manual testing with a text editor actively receiving typed input.

---

# 14. Menu-bar item

Because the application uses accessory activation policy and may have no Dock icon, add a minimal menu-bar item.

Use Tauri’s tray/menu functionality supported by the installed version.

Menu entries:

```text
Show Timer Demo
Show Recording Demo
Show Download Demo
Show Notification Demo
Show Failure Demo
Hide Island
Quit Open Island
```

The menu-bar icon may be a simple monochrome template icon.

Do not build a settings window in this milestone.

The menu-bar item should remain functional when the island is hidden.

---

# 15. Error handling and logging

Create explicit error enums for:

* Bridge errors.
* Activity errors.
* Window errors.
* Platform errors.

Do not use `unwrap()` or `expect()` in runtime paths unless failure is genuinely unrecoverable during startup.

For command errors:

* Return sanitized error messages to the frontend.
* Log detailed native errors in the backend.
* Do not expose raw AppKit pointers or internal paths.

Use development logging for:

* Activity transitions.
* Snapshot revisions.
* Native frame changes.
* Screen selection.
* Focus transitions.
* Bridge subscription readiness.

Avoid logging every one-second timer tick at normal log levels.

---

# 16. Security and Tauri capabilities

Keep capabilities minimal.

Because window mutations occur in backend commands, avoid exposing broad core window permissions to frontend JavaScript.

Review:

```text
src-tauri/capabilities/default.json
```

Allow only:

* Required custom commands.
* Required event functionality.
* Required tray behavior.

Do not enable:

* Filesystem access.
* Shell execution.
* Arbitrary process launching.
* Network access.
* Clipboard access.

unless already needed by the existing template.

Tauri capabilities control which windows and webviews can access core and plugin commands.

Restrict the capability to the `island` window label.

---

# 17. Tests

Add focused tests rather than a large test framework.

## 17.1 Shared-model tests

Test:

* Serialization and deserialization.
* Enum representation.
* Progress validation.
* Stable activity IDs.

## 17.2 Reducer tests

Test:

* Adding the first activity shows compact mode.
* Higher-priority activity becomes primary.
* Expanding creates a transition.
* Stale animation completion is ignored.
* Completing a download enters attention mode.
* Dismissing the final activity eventually hides the island.
* Expanded activity does not auto-dismiss unexpectedly.

## 17.3 Layout tests

Test:

* Top-center anchoring.
* Compact-to-expanded width changes.
* Negative monitor origins.
* Notched screen placement.
* External display placement.
* Invalid notch metrics are clamped.
* Frame remains inside a usable screen region.

## 17.4 No AppKit unit tests required

Do not attempt to instantiate actual `NSScreen` or `NSWindow` objects in normal unit tests.

Keep native calls behind an adapter so layout logic can be tested with plain structs.

---

# 18. Manual verification checklist

Create:

```text
docs/manual-test-checklist.md
```

Include the following checks.

## Startup

* No normal rectangular window appears.
* No white flash appears.
* No Dock icon remains visible.
* Menu-bar item appears.
* Island remains hidden until an activity exists.

## Compact mode

* Timer demo appears at top center.
* Compact island does not steal text input from another application.
* Progress updates.
* UI remains visually centered.
* No transparent rectangular region blocks unrelated clicks.

## Expanded mode

* Clicking compact expands the native window first.
* Animation is not clipped.
* Top edge remains anchored.
* Expanded controls are clickable.
* Escape collapses.
* Focus returns to the previously active application.

## Completion

* Download reaches 100%.
* Completion attention state appears.
* Completion auto-dismiss works.
* Expanding prevents premature auto-dismiss.

## Displays

* Built-in notched MacBook display.
* Built-in display without a notch, if available.
* External monitor.
* External monitor arranged left of built-in display.
* External monitor arranged above built-in display.
* Monitor disconnected while island is visible.
* Display scaling changed.

## macOS environments

* Multiple Spaces.
* Mission Control.
* Stage Manager.
* Native full-screen application.
* Menu bar set to auto-hide.
* Light appearance.
* Dark appearance.
* Reduced-motion setting.

Document any OS-level cases that remain unreliable.

---

# 19. Implementation milestones

Complete the implementation in this order.

## Milestone 1: Repository audit

Deliver:

* Existing project still runs.
* Dependency versions documented.
* No behavior changes.

## Milestone 2: Static Leptos island

Deliver:

* Compact and expanded components.
* CSS animations.
* Local temporary mock state.
* No native behavior yet.

## Milestone 3: Shared model and backend state

Deliver:

* Shared model crate.
* Activity manager.
* Reducer.
* Demo activities.
* Snapshot events.
* Mock frontend state removed.

## Milestone 4: Native window lifecycle

Deliver:

* Programmatically created hidden island window.
* Transparent undecorated presentation.
* Accessory activation policy.
* Show, hide, resize, and top-center anchoring.
* Correct transition sequencing.

## Milestone 5: macOS screen support

Deliver:

* AppKit screen adapter.
* Safe-area and notch detection.
* Multi-monitor geometry handling.
* Repositioning after screen changes.

## Milestone 6: Focus behavior

Deliver:

* Compact state does not steal focus.
* Expanded state accepts interaction.
* Collapse resigns focus safely.
* Escape behavior works.

## Milestone 7: Menu-bar controls

Deliver:

* Demo menu actions.
* Hide action.
* Quit action.

## Milestone 8: Hardening

Deliver:

* Unit tests.
* Manual test checklist.
* Error handling.
* Development logging.
* Release build.
* Documentation of limitations.

Do not start NSPanel conversion until Milestones 1–8 work with a standard Tauri window.

---

# 20. Acceptance criteria

The task is complete when all of the following are true:

1. `cargo tauri dev` starts Open Island successfully.
2. The application does not open a normal desktop window.
3. The application runs as an accessory/menu-bar application.
4. Triggering a timer demo displays a compact island.
5. The island is horizontally centered on the selected display.
6. On a notched MacBook display, placement accounts for the notch.
7. The island stays above normal application windows.
8. It appears across Spaces.
9. Clicking it expands smoothly.
10. Expansion is not clipped by the native window bounds.
11. The top-center anchor remains stable during expansion.
12. Compact mode does not steal keyboard focus.
13. Expanded controls are interactive.
14. Escape collapses the island.
15. The previously active app can continue receiving keyboard input afterward.
16. Demo activities update in real time.
17. Download completion produces an attention state and auto-dismiss.
18. Menu-bar demo actions work while the island is hidden.
19. Menu-bar Quit terminates the application cleanly.
20. `cargo fmt`, `cargo check`, and `cargo test` pass.
21. `cargo tauri build` produces a macOS build.
22. The README accurately documents the transparent-window and App Store limitation.

---

# 21. Explicit non-goals

Do not implement any of the following in this task:

* iPhone ActivityKit integration.
* A WidgetKit extension.
* System notification interception.
* AirPods connection monitoring.
* Camera or microphone privacy-indicator replacement.
* Face ID simulation.
* Phone-call interception.
* Media control scraping from unrelated apps.
* Accessibility API permissions.
* Screen recording permissions.
* Auto-update.
* Login-at-startup.
* Cloud synchronization.
* Analytics or telemetry.
* User accounts.
* A settings window.
* Plugin marketplace.
* Multiple islands.
* Drag-and-drop repositioning.
* Hover-to-expand.
* High-frequency audio waveform rendering.
* Automatic use of a third-party NSPanel crate.

---

# 22. Final deliverables

At completion, provide:

1. All implementation code.
2. Updated `README.md`.
3. `docs/architecture.md`.
4. `docs/manual-test-checklist.md`.
5. A concise list of files changed.
6. Commands used to validate the implementation.
7. Test results.
8. Known limitations.
9. Any AppKit `unsafe` blocks, each accompanied by:

   * A safety comment.
   * The main-thread assumption.
   * Ownership/lifetime reasoning.
10. A recommendation on whether a true NSPanel conversion is still needed after testing the standard Tauri window.

Do not merely provide sample snippets or a future plan. Implement each milestone in the repository and leave the project in a buildable state.

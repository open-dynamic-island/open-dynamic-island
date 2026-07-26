# Architecture

## Responsibilities

The `island-model` crate contains the serializable activity, mode, action, display-context, and snapshot types. It has no Tauri, Leptos, filesystem, or operating-system dependency and builds for native Rust and WebAssembly.

The Tauri backend is authoritative. `ActivityManager` protects `ActivityState` with a standard mutex. The pure reducer applies explicit `IslandAction` values, increments revisions, creates transition IDs, selects the primary activity deterministically, and returns `IslandEffect` values. Command handlers execute those effects through the window controller and publish snapshots.

The Leptos frontend keeps only the latest backend snapshot plus transient animation and bridge status. `src/bridge.rs` is the sole JavaScript interop boundary. Components render the compact or expanded activity, and report CSS transition completion with the backend transition ID. Stale completions are ignored by the reducer.

## Startup and transitions

At startup, Tauri creates the `island` webview hidden, transparent, undecorated, always on top, non-focusable, and present across workspaces. The frontend registers its snapshot listener before invoking `frontend_ready`. After applying that snapshot it invokes `first_frame_ready`; the backend only shows the window if state is visible.

Expansion first enlarges and top-center repositions the native window, then enables focus and publishes the expanded snapshot. Collapse publishes the compact snapshot first; after the CSS transition reports completion, the backend resigns AppKit key-window status, disables focusability, and restores the compact frame. Hiding likewise waits for the visual dismissal before hiding while keeping the webview alive.

## macOS adapter

All AppKit types are isolated under `src-tauri/src/platform/macos`. `screen.rs` translates `NSScreen` frame, visible frame, safe-area, auxiliary top regions, and scale into `ScreenGeometry`. Pure layout code centers and clamps the window and is covered by unit tests.

All display types anchor at the physical top edge, so the compact surface occupies menu-bar level and expands downward. `appkit.rs` contains the only native window pointer casts. Each unsafe block:

- runs in a callback explicitly dispatched through Tauri to the AppKit main thread;
- borrows the `NSWindow` pointer owned by Tauri only for the callback;
- never transfers ownership or lets the reference escape.

The adapter applies `canJoinAllSpaces`, `stationary`, and `fullScreenAuxiliary`, disables opacity and shadow, and resigns key-window status before compact mode becomes non-focusable.

Tauri scale-factor events trigger repositioning. Frame calculations also reselect the window's current monitor whenever a native layout changes, so expansion, collapse, resolution changes, and display removal do not reuse stale coordinates.

## Security

The capability applies only to the `island` label and grants core defaults plus frontend event listening. Window mutations and demo control remain custom Rust commands. No filesystem, shell, network, clipboard, opener, or process plugin is enabled.

## NSPanel recommendation

Keep the standard Tauri window unless manual tests demonstrate a specific unresolved problem with focus restoration, non-activating clicks, full-screen auxiliary presentation, or transparent hit testing. If one remains, the platform adapter is the appropriate seam for an explicitly pinned NSPanel implementation; activity and frontend code should not change.

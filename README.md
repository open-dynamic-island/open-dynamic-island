# Open Island

Open Island is a macOS menu-bar utility built with Tauri and Leptos. It presents local demo activities in a compact, top-center overlay that can expand for details and actions.

The MVP intentionally uses demo data only. It does not intercept notifications, inspect other applications, use the microphone, or depend on private system events.

## Requirements

- macOS 11 or newer
- Rust with the `wasm32-unknown-unknown` target
- Trunk
- Tauri CLI 2
- Xcode Command Line Tools

The repository audit was performed with:

| Component | Detected version |
| --- | --- |
| Tauri | 2.11.5 |
| Tauri CLI | 2.11.4 |
| Leptos | 0.8.20 |
| Trunk | 0.21.14 |
| Rust edition | 2021 |
| Minimum macOS target | 11.0 |

## Development

```bash
rustup target add wasm32-unknown-unknown
cargo tauri dev
```

If the shell exports `NO_COLOR=1`, Trunk 0.21.14 rejects that value because it expects a boolean string. Run:

```bash
NO_COLOR=false cargo tauri dev
```

Use the Open Island menu-bar icon to start timer, recording, download, notification, or failure demos. The island starts hidden.

Validation commands:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo tauri build
```

The default bundle target is the macOS `.app`. A DMG can be requested explicitly in an interactive Finder session with `cargo tauri build --bundles dmg`.

## Distribution note

Transparent macOS webview rendering uses Tauri's `macos-private-api` feature (`app.macOSPrivateApi` in the Tauri configuration). Tauri documents that builds using this mode are not eligible for the Mac App Store. This MVP is intended for direct distribution and is not claimed to be App Store-ready.

## Current limitations

- The product is macOS-first; non-macOS builds use fallback screen/window adapters.
- Notch geometry uses the selected AppKit main screen when the app initializes. Later off-main-thread repositioning uses the window's current Tauri monitor; scale-factor changes trigger a fresh layout.
- Display-change behavior needs manual validation across unusual arrangements and Stage Manager.
- A standard Tauri `NSWindow` is used. A true `NSPanel` should only be considered if manual testing finds focus, click-through, or full-screen Space behavior that the standard window cannot satisfy.

See [architecture](docs/architecture.md) and the [manual test checklist](docs/manual-test-checklist.md).

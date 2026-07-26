# Manual test checklist

Record the macOS version, Mac model, monitor arrangement, and result for every failure.

## Startup

- [ ] No normal rectangular window appears.
- [ ] No white or incorrectly positioned flash appears.
- [ ] No Dock icon remains visible.
- [ ] The menu-bar item appears.
- [ ] The island remains hidden until an activity exists.

## Compact mode

- [ ] Timer demo appears at the top center.
- [ ] Compact mode does not interrupt typing in another application.
- [ ] Timer/progress updates once per second.
- [ ] The pill remains visually centered.
- [ ] Transparent window area does not block unrelated clicks.

## Expanded mode

- [ ] Clicking compact expands the native window before content animation.
- [ ] Expansion content is not clipped.
- [ ] The top edge and horizontal center remain anchored.
- [ ] Expanded controls are clickable.
- [ ] Escape collapses the island.
- [ ] Focus returns to the previously active application.

## Completion

- [ ] Download reaches 100%.
- [ ] Download completion enters attention mode.
- [ ] Completion auto-dismisses after several seconds.
- [ ] Expanding prevents premature auto-dismiss.
- [ ] Failure Retry creates a fresh running notification.

## Displays

- [ ] Built-in notched MacBook display.
- [ ] Built-in display without a notch, if available.
- [ ] External monitor.
- [ ] External monitor arranged left of the built-in display.
- [ ] External monitor arranged above the built-in display.
- [ ] Monitor disconnected while the island is visible.
- [ ] Display resolution and scaling changed.

## macOS environments

- [ ] Multiple Spaces.
- [ ] Mission Control.
- [ ] Stage Manager.
- [ ] Native full-screen application.
- [ ] Auto-hidden menu bar.
- [ ] Light appearance.
- [ ] Dark appearance.
- [ ] Reduced-motion accessibility setting.

## Known OS-level risk areas

- Standard `NSWindow` behavior can vary around full-screen Spaces and Stage Manager; record whether an NSPanel is actually needed.
- Verify notch exclusion on each hardware shape because AppKit reports safe/auxiliary regions rather than a direct notch rectangle.
- Verify clicks near transparent expanded bounds; a standard webview window still owns its rectangular native hit-test region.

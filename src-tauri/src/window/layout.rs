use super::constants::{MAX_NOTCH_COMPACT_WIDTH, MAX_NOTCH_WIDTH, NOTCH_CONTENT_WINGS};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IslandFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ScreenGeometry {
    pub fn has_notch(&self) -> bool {
        self.safe_top > 1.0
            && self.auxiliary_left_width.is_some_and(|value| value > 0.0)
            && self.auxiliary_right_width.is_some_and(|value| value > 0.0)
    }

    pub fn estimated_notch_width(&self) -> f64 {
        if !self.has_notch() {
            return 0.0;
        }
        let left = self.auxiliary_left_width.unwrap_or_default();
        let right = self.auxiliary_right_width.unwrap_or_default();
        (self.frame_width - left - right).clamp(0.0, MAX_NOTCH_WIDTH)
    }
}

pub fn calculate_island_frame(
    screen: &ScreenGeometry,
    requested_width: f64,
    height: f64,
) -> IslandFrame {
    let max_width = screen.frame_width.max(1.0);
    let notch_aware_width = if screen.has_notch() && requested_width < MAX_NOTCH_COMPACT_WIDTH {
        requested_width.max(
            (screen.estimated_notch_width() + NOTCH_CONTENT_WINGS).min(MAX_NOTCH_COMPACT_WIDTH),
        )
    } else {
        requested_width
    };
    let width = notch_aware_width.min(max_width);
    let x = screen.frame_x + ((screen.frame_width - width) / 2.0);
    // Anchor to the physical top edge even on a non-notched display. This lets
    // the compact surface occupy menu-bar level and expand downward like a notch.
    let y = screen.frame_y;
    IslandFrame {
        x,
        y,
        width,
        height: height.min(screen.frame_height.max(1.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(x: f64, y: f64, notched: bool) -> ScreenGeometry {
        ScreenGeometry {
            frame_x: x,
            frame_y: y,
            frame_width: 1512.0,
            frame_height: 982.0,
            visible_x: x,
            visible_y: y + if notched { 38.0 } else { 24.0 },
            visible_width: 1512.0,
            visible_height: 944.0,
            safe_top: if notched { 38.0 } else { 0.0 },
            safe_left: 0.0,
            safe_right: 0.0,
            safe_bottom: 0.0,
            auxiliary_left_width: notched.then_some(692.0),
            auxiliary_right_width: notched.then_some(692.0),
            scale_factor: 2.0,
        }
    }

    #[test]
    fn anchors_width_changes_at_top_center() {
        let screen = screen(0.0, 0.0, true);
        let compact = calculate_island_frame(&screen, 220.0, 40.0);
        let expanded = calculate_island_frame(&screen, 420.0, 176.0);
        assert!(((compact.x + compact.width / 2.0) - 756.0).abs() < 0.01);
        assert!(((expanded.x + expanded.width / 2.0) - 756.0).abs() < 0.01);
        assert_eq!(compact.y, expanded.y);
    }

    #[test]
    fn supports_negative_monitor_origins() {
        let frame = calculate_island_frame(&screen(-1512.0, -982.0, true), 220.0, 40.0);
        assert_eq!(frame.x, -940.0);
        assert_eq!(frame.y, -982.0);
    }

    #[test]
    fn external_display_anchors_at_physical_top() {
        let frame = calculate_island_frame(&screen(0.0, 0.0, false), 220.0, 40.0);
        assert_eq!(frame.y, 0.0);
    }

    #[test]
    fn invalid_notch_metrics_are_clamped() {
        let mut geometry = screen(0.0, 0.0, true);
        geometry.auxiliary_left_width = Some(1.0);
        geometry.auxiliary_right_width = Some(1.0);
        assert_eq!(geometry.estimated_notch_width(), MAX_NOTCH_WIDTH);
        let frame = calculate_island_frame(&geometry, 220.0, 40.0);
        assert_eq!(frame.width, MAX_NOTCH_COMPACT_WIDTH);
    }

    #[test]
    fn narrow_display_keeps_frame_inside_screen() {
        let mut geometry = screen(-400.0, 50.0, false);
        geometry.frame_width = 180.0;
        geometry.visible_width = 180.0;
        let frame = calculate_island_frame(&geometry, 420.0, 176.0);
        assert_eq!(frame.x, -400.0);
        assert_eq!(frame.width, 180.0);
    }
}

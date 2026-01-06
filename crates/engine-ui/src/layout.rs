// UI layout and positioning

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Rectangle in screen space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            x: min.x,
            y: min.y,
            width: max.x - min.x,
            height: max.y - min.y,
        }
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    pub fn shrink(&self, amount: f32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - amount * 2.0).max(0.0),
            height: (self.height - amount * 2.0).max(0.0),
        }
    }

    pub fn expand(&self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: self.width + amount * 2.0,
            height: self.height + amount * 2.0,
        }
    }
}

/// Padding around a widget
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Padding {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn uniform(amount: f32) -> Self {
        Self {
            left: amount,
            right: amount,
            top: amount,
            bottom: amount,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    pub fn apply(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.left,
            y: rect.y + self.top,
            width: (rect.width - self.left - self.right).max(0.0),
            height: (rect.height - self.top - self.bottom).max(0.0),
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::uniform(0.0)
    }
}

/// Anchor point for positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    pub fn offset(&self, container: Rect, widget_size: Vec2) -> Vec2 {
        match self {
            Anchor::TopLeft => Vec2::new(container.x, container.y),
            Anchor::TopCenter => Vec2::new(
                container.x + (container.width - widget_size.x) * 0.5,
                container.y,
            ),
            Anchor::TopRight => Vec2::new(
                container.x + container.width - widget_size.x,
                container.y,
            ),
            Anchor::CenterLeft => Vec2::new(
                container.x,
                container.y + (container.height - widget_size.y) * 0.5,
            ),
            Anchor::Center => Vec2::new(
                container.x + (container.width - widget_size.x) * 0.5,
                container.y + (container.height - widget_size.y) * 0.5,
            ),
            Anchor::CenterRight => Vec2::new(
                container.x + container.width - widget_size.x,
                container.y + (container.height - widget_size.y) * 0.5,
            ),
            Anchor::BottomLeft => Vec2::new(
                container.x,
                container.y + container.height - widget_size.y,
            ),
            Anchor::BottomCenter => Vec2::new(
                container.x + (container.width - widget_size.x) * 0.5,
                container.y + container.height - widget_size.y,
            ),
            Anchor::BottomRight => Vec2::new(
                container.x + container.width - widget_size.x,
                container.y + container.height - widget_size.y,
            ),
        }
    }
}

/// Alignment for content within a container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Start,
    Center,
    End,
}

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// Layout configuration for arranging widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub direction: LayoutDirection,
    pub spacing: f32,
    pub padding: Padding,
    pub alignment: Alignment,
}

impl Layout {
    pub fn horizontal() -> Self {
        Self {
            direction: LayoutDirection::Horizontal,
            spacing: 8.0,
            padding: Padding::default(),
            alignment: Alignment::Start,
        }
    }

    pub fn vertical() -> Self {
        Self {
            direction: LayoutDirection::Vertical,
            spacing: 8.0,
            padding: Padding::default(),
            alignment: Alignment::Start,
        }
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Calculate positions for child widgets
    pub fn arrange(&self, container: Rect, widget_sizes: &[Vec2]) -> Vec<Rect> {
        let content_area = self.padding.apply(container);
        let mut rects = Vec::new();

        match self.direction {
            LayoutDirection::Horizontal => {
                let total_width: f32 = widget_sizes.iter().map(|s| s.x).sum();
                let total_spacing = self.spacing * (widget_sizes.len().saturating_sub(1)) as f32;
                let available_width = content_area.width - total_spacing;

                let mut x = content_area.x;

                // Adjust starting position based on alignment
                match self.alignment {
                    Alignment::Center => x += (available_width - total_width) * 0.5,
                    Alignment::End => x += available_width - total_width,
                    _ => {}
                }

                for size in widget_sizes {
                    let y = content_area.y
                        + match self.alignment {
                            Alignment::Center => (content_area.height - size.y) * 0.5,
                            Alignment::End => content_area.height - size.y,
                            _ => 0.0,
                        };

                    rects.push(Rect::new(x, y, size.x, size.y));
                    x += size.x + self.spacing;
                }
            }
            LayoutDirection::Vertical => {
                let total_height: f32 = widget_sizes.iter().map(|s| s.y).sum();
                let total_spacing = self.spacing * (widget_sizes.len().saturating_sub(1)) as f32;
                let available_height = content_area.height - total_spacing;

                let mut y = content_area.y;

                // Adjust starting position based on alignment
                match self.alignment {
                    Alignment::Center => y += (available_height - total_height) * 0.5,
                    Alignment::End => y += available_height - total_height,
                    _ => {}
                }

                for size in widget_sizes {
                    let x = content_area.x
                        + match self.alignment {
                            Alignment::Center => (content_area.width - size.x) * 0.5,
                            Alignment::End => content_area.width - size.x,
                            _ => 0.0,
                        };

                    rects.push(Rect::new(x, y, size.x, size.y));
                    y += size.y + self.spacing;
                }
            }
        }

        rects
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::vertical()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(rect.contains_point(Vec2::new(50.0, 50.0)));
        assert!(!rect.contains_point(Vec2::new(150.0, 50.0)));
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(rect.center(), Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_padding_apply() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let padding = Padding::uniform(10.0);
        let padded = padding.apply(rect);
        assert_eq!(padded.x, 10.0);
        assert_eq!(padded.y, 10.0);
        assert_eq!(padded.width, 80.0);
        assert_eq!(padded.height, 80.0);
    }

    #[test]
    fn test_layout_horizontal() {
        let container = Rect::new(0.0, 0.0, 300.0, 100.0);
        let layout = Layout::horizontal().with_spacing(10.0);
        let sizes = vec![Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0)];
        let rects = layout.arrange(container, &sizes);

        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[1].x, 60.0); // 50 + 10 spacing
    }
}

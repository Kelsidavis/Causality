// UI drawing canvas

use glam::Vec2;
use crate::{Color, Rect};

/// Drawing command for rendering
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    Rect {
        rect: Rect,
        color: Color,
        corner_radius: f32,
    },
    /// Draw a rectangle outline
    RectOutline {
        rect: Rect,
        color: Color,
        thickness: f32,
        corner_radius: f32,
    },
    /// Draw text
    Text {
        position: Vec2,
        text: String,
        font_size: f32,
        color: Color,
    },
    /// Draw an image/texture
    Image {
        rect: Rect,
        texture_id: u64,
        tint: Color,
    },
    /// Draw a line
    Line {
        start: Vec2,
        end: Vec2,
        color: Color,
        thickness: f32,
    },
    /// Draw a circle
    Circle {
        center: Vec2,
        radius: f32,
        color: Color,
    },
}

/// Canvas for collecting draw commands
pub struct Canvas {
    commands: Vec<DrawCommand>,
    clip_stack: Vec<Rect>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            clip_stack: Vec::new(),
        }
    }

    /// Push a clipping region
    pub fn push_clip(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    /// Pop the current clipping region
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Draw a filled rectangle
    pub fn rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        if let Some(clipped) = self.clip_rect(rect) {
            self.commands.push(DrawCommand::Rect {
                rect: clipped,
                color,
                corner_radius,
            });
        }
    }

    /// Draw a rectangle outline
    pub fn rect_outline(&mut self, rect: Rect, color: Color, thickness: f32, corner_radius: f32) {
        if let Some(clipped) = self.clip_rect(rect) {
            self.commands.push(DrawCommand::RectOutline {
                rect: clipped,
                color,
                thickness,
                corner_radius,
            });
        }
    }

    /// Draw text
    pub fn text(&mut self, position: Vec2, text: impl Into<String>, font_size: f32, color: Color) {
        if self.is_point_visible(position) {
            self.commands.push(DrawCommand::Text {
                position,
                text: text.into(),
                font_size,
                color,
            });
        }
    }

    /// Draw an image
    pub fn image(&mut self, rect: Rect, texture_id: u64, tint: Color) {
        if let Some(clipped) = self.clip_rect(rect) {
            self.commands.push(DrawCommand::Image {
                rect: clipped,
                texture_id,
                tint,
            });
        }
    }

    /// Draw a line
    pub fn line(&mut self, start: Vec2, end: Vec2, color: Color, thickness: f32) {
        if self.is_point_visible(start) || self.is_point_visible(end) {
            self.commands.push(DrawCommand::Line {
                start,
                end,
                color,
                thickness,
            });
        }
    }

    /// Draw a circle
    pub fn circle(&mut self, center: Vec2, radius: f32, color: Color) {
        if self.is_point_visible(center) {
            self.commands.push(DrawCommand::Circle {
                center,
                radius,
                color,
            });
        }
    }

    /// Get all draw commands
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
        self.clip_stack.clear();
    }

    /// Check if a point is visible in the current clip region
    fn is_point_visible(&self, point: Vec2) -> bool {
        if let Some(clip) = self.clip_stack.last() {
            clip.contains_point(point)
        } else {
            true
        }
    }

    /// Clip a rectangle to the current clip region
    fn clip_rect(&self, rect: Rect) -> Option<Rect> {
        if let Some(clip) = self.clip_stack.last() {
            // Simple intersection check
            let x1 = rect.x.max(clip.x);
            let y1 = rect.y.max(clip.y);
            let x2 = (rect.x + rect.width).min(clip.x + clip.width);
            let y2 = (rect.y + rect.height).min(clip.y + clip.height);

            if x2 > x1 && y2 > y1 {
                Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
            } else {
                None
            }
        } else {
            Some(rect)
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_draw_rect() {
        let mut canvas = Canvas::new();
        canvas.rect(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED, 0.0);
        assert_eq!(canvas.commands().len(), 1);
    }

    #[test]
    fn test_canvas_clipping() {
        let mut canvas = Canvas::new();
        canvas.push_clip(Rect::new(0.0, 0.0, 50.0, 50.0));

        // This rectangle should be clipped
        canvas.rect(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED, 0.0);

        assert_eq!(canvas.commands().len(), 1);
        if let DrawCommand::Rect { rect, .. } = &canvas.commands()[0] {
            assert_eq!(rect.width, 50.0);
            assert_eq!(rect.height, 50.0);
        } else {
            panic!("Expected Rect command");
        }
    }

    #[test]
    fn test_canvas_clear() {
        let mut canvas = Canvas::new();
        canvas.rect(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED, 0.0);
        canvas.clear();
        assert_eq!(canvas.commands().len(), 0);
    }
}

// UI styling and theming

use serde::{Deserialize, Serialize};

/// RGBA color
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as f32 / 255.0,
            g: ((hex >> 8) & 0xFF) as f32 / 255.0,
            b: (hex & 0xFF) as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha;
        self
    }

    pub fn lerp(self, other: Color, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Font style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontStyle {
    pub size: f32,
    pub color: Color,
    pub align: TextAlign,
}

impl FontStyle {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            color: Color::WHITE,
            align: TextAlign::Left,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::new(16.0)
    }
}

/// UI widget style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    /// Background color
    pub background: Color,
    /// Border color
    pub border_color: Color,
    /// Border width
    pub border_width: f32,
    /// Corner radius for rounded corners
    pub corner_radius: f32,
    /// Font style for text
    pub font: FontStyle,
}

impl Style {
    pub fn new() -> Self {
        Self {
            background: Color::rgba(0.2, 0.2, 0.2, 0.9),
            border_color: Color::rgba(1.0, 1.0, 1.0, 0.5),
            border_width: 1.0,
            corner_radius: 4.0,
            font: FontStyle::default(),
        }
    }

    pub fn button() -> Self {
        Self {
            background: Color::rgba(0.3, 0.3, 0.8, 0.9),
            border_color: Color::rgba(0.5, 0.5, 1.0, 1.0),
            border_width: 2.0,
            corner_radius: 8.0,
            font: FontStyle::new(18.0).with_color(Color::WHITE),
        }
    }

    pub fn panel() -> Self {
        Self {
            background: Color::rgba(0.1, 0.1, 0.1, 0.85),
            border_color: Color::rgba(0.5, 0.5, 0.5, 0.8),
            border_width: 2.0,
            corner_radius: 12.0,
            font: FontStyle::default(),
        }
    }

    pub fn health_bar() -> Self {
        Self {
            background: Color::rgba(0.2, 0.0, 0.0, 0.8),
            border_color: Color::BLACK,
            border_width: 2.0,
            corner_radius: 4.0,
            font: FontStyle::default(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex(0xFF0000); // Red
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_lerp() {
        let start = Color::BLACK;
        let end = Color::WHITE;
        let mid = start.lerp(end, 0.5);
        assert_eq!(mid.r, 0.5);
        assert_eq!(mid.g, 0.5);
        assert_eq!(mid.b, 0.5);
    }

    #[test]
    fn test_color_alpha() {
        let color = Color::RED.with_alpha(0.5);
        assert_eq!(color.a, 0.5);
    }
}

// UI widgets - buttons, labels, health bars, etc.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{Canvas, Color, FontStyle, Padding, Rect, Style, TextAlign};

/// Unique widget identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetId(pub u64);

/// Base widget trait
pub trait Widget {
    /// Get widget ID
    fn id(&self) -> WidgetId;

    /// Get widget bounds
    fn bounds(&self) -> Rect;

    /// Set widget bounds
    fn set_bounds(&mut self, bounds: Rect);

    /// Draw the widget
    fn draw(&self, canvas: &mut Canvas);

    /// Handle mouse interaction (returns true if clicked)
    fn handle_mouse(&mut self, mouse_pos: Vec2, mouse_down: bool) -> bool {
        let _ = (mouse_pos, mouse_down);
        false
    }

    /// Update widget state
    fn update(&mut self, _delta_time: f32) {}
}

/// Text label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: WidgetId,
    pub bounds: Rect,
    pub text: String,
    pub style: FontStyle,
}

impl Label {
    pub fn new(id: WidgetId, text: impl Into<String>) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 100.0, 30.0),
            text: text.into(),
            style: FontStyle::default(),
        }
    }

    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Label {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        let pos = match self.style.align {
            TextAlign::Left => Vec2::new(self.bounds.x, self.bounds.y + self.bounds.height * 0.5),
            TextAlign::Center => Vec2::new(
                self.bounds.x + self.bounds.width * 0.5,
                self.bounds.y + self.bounds.height * 0.5,
            ),
            TextAlign::Right => Vec2::new(
                self.bounds.x + self.bounds.width,
                self.bounds.y + self.bounds.height * 0.5,
            ),
        };

        canvas.text(pos, &self.text, self.style.size, self.style.color);
    }
}

/// Button widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    pub id: WidgetId,
    pub bounds: Rect,
    pub text: String,
    pub style: Style,
    pub hovered: bool,
    pub pressed: bool,
}

impl Button {
    pub fn new(id: WidgetId, text: impl Into<String>) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 120.0, 40.0),
            text: text.into(),
            style: Style::button(),
            hovered: false,
            pressed: false,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        // Background
        let mut bg_color = self.style.background;
        if self.pressed {
            bg_color = bg_color.lerp(Color::BLACK, 0.3);
        } else if self.hovered {
            bg_color = bg_color.lerp(Color::WHITE, 0.2);
        }

        canvas.rect(self.bounds, bg_color, self.style.corner_radius);

        // Border
        canvas.rect_outline(
            self.bounds,
            self.style.border_color,
            self.style.border_width,
            self.style.corner_radius,
        );

        // Text
        let text_pos = self.bounds.center();
        canvas.text(text_pos, &self.text, self.style.font.size, self.style.font.color);
    }

    fn handle_mouse(&mut self, mouse_pos: Vec2, mouse_down: bool) -> bool {
        let was_pressed = self.pressed;
        self.hovered = self.bounds.contains_point(mouse_pos);
        self.pressed = self.hovered && mouse_down;

        // Return true on button release while hovered
        was_pressed && !mouse_down && self.hovered
    }
}

/// Progress bar / health bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBar {
    pub id: WidgetId,
    pub bounds: Rect,
    pub value: f32, // 0.0 to 1.0
    pub style: Style,
    pub fill_color: Color,
}

impl HealthBar {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 200.0, 30.0),
            value: 1.0,
            style: Style::health_bar(),
            fill_color: Color::GREEN,
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for HealthBar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        // Background
        canvas.rect(
            self.bounds,
            self.style.background,
            self.style.corner_radius,
        );

        // Fill (health)
        let fill_width = self.bounds.width * self.value;
        if fill_width > 0.0 {
            let fill_rect = Rect::new(
                self.bounds.x,
                self.bounds.y,
                fill_width,
                self.bounds.height,
            );

            // Color transitions from green to red based on health
            let color = if self.value > 0.5 {
                Color::GREEN
            } else if self.value > 0.25 {
                Color::YELLOW
            } else {
                Color::RED
            };

            canvas.rect(fill_rect, color, self.style.corner_radius);
        }

        // Border
        canvas.rect_outline(
            self.bounds,
            self.style.border_color,
            self.style.border_width,
            self.style.corner_radius,
        );
    }
}

/// Progress bar (generic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressBar {
    pub id: WidgetId,
    pub bounds: Rect,
    pub value: f32, // 0.0 to 1.0
    pub style: Style,
    pub fill_color: Color,
}

impl ProgressBar {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 200.0, 20.0),
            value: 0.0,
            style: Style::default(),
            fill_color: Color::BLUE,
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for ProgressBar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        // Background
        canvas.rect(
            self.bounds,
            self.style.background,
            self.style.corner_radius,
        );

        // Fill
        let fill_width = self.bounds.width * self.value;
        if fill_width > 0.0 {
            let fill_rect = Rect::new(
                self.bounds.x,
                self.bounds.y,
                fill_width,
                self.bounds.height,
            );
            canvas.rect(fill_rect, self.fill_color, self.style.corner_radius);
        }

        // Border
        canvas.rect_outline(
            self.bounds,
            self.style.border_color,
            self.style.border_width,
            self.style.corner_radius,
        );
    }
}

/// Panel container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub id: WidgetId,
    pub bounds: Rect,
    pub style: Style,
    pub padding: Padding,
}

impl Panel {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 300.0, 200.0),
            style: Style::panel(),
            padding: Padding::uniform(10.0),
        }
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn content_bounds(&self) -> Rect {
        self.padding.apply(self.bounds)
    }
}

impl Widget for Panel {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        // Background
        canvas.rect(
            self.bounds,
            self.style.background,
            self.style.corner_radius,
        );

        // Border
        canvas.rect_outline(
            self.bounds,
            self.style.border_color,
            self.style.border_width,
            self.style.corner_radius,
        );
    }
}

/// Image widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: WidgetId,
    pub bounds: Rect,
    pub texture_id: u64,
    pub tint: Color,
}

impl Image {
    pub fn new(id: WidgetId, texture_id: u64) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
            texture_id,
            tint: Color::WHITE,
        }
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
}

impl Widget for Image {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.image(self.bounds, self.texture_id, self.tint);
    }
}

/// Slider widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slider {
    pub id: WidgetId,
    pub bounds: Rect,
    pub value: f32, // 0.0 to 1.0
    pub style: Style,
    pub dragging: bool,
}

impl Slider {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 200.0, 20.0),
            value: 0.5,
            style: Style::default(),
            dragging: false,
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for Slider {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        // Track
        canvas.rect(
            self.bounds,
            self.style.background,
            self.style.corner_radius,
        );

        // Handle
        let handle_size = 10.0;
        let handle_x = self.bounds.x + (self.bounds.width - handle_size) * self.value;
        let handle_rect = Rect::new(
            handle_x,
            self.bounds.y - (handle_size - self.bounds.height) * 0.5,
            handle_size,
            handle_size,
        );

        canvas.rect(handle_rect, Color::WHITE, handle_size * 0.5);
        canvas.rect_outline(handle_rect, self.style.border_color, 2.0, handle_size * 0.5);
    }

    fn handle_mouse(&mut self, mouse_pos: Vec2, mouse_down: bool) -> bool {
        if mouse_down && self.bounds.contains_point(mouse_pos) {
            self.dragging = true;
        }

        if !mouse_down {
            self.dragging = false;
        }

        if self.dragging {
            let relative_x = (mouse_pos.x - self.bounds.x) / self.bounds.width;
            self.value = relative_x.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }
}

/// Text input widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInput {
    pub id: WidgetId,
    pub bounds: Rect,
    pub text: String,
    pub placeholder: String,
    pub style: Style,
    pub focused: bool,
}

impl TextInput {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::new(0.0, 0.0, 200.0, 30.0),
            text: String::new(),
            placeholder: String::new(),
            style: Style::default(),
            focused: false,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

impl Widget for TextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, canvas: &mut Canvas) {
        // Background
        let bg_color = if self.focused {
            self.style.background.lerp(Color::WHITE, 0.1)
        } else {
            self.style.background
        };

        canvas.rect(self.bounds, bg_color, self.style.corner_radius);

        // Text or placeholder
        let display_text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };

        let text_color = if self.text.is_empty() {
            self.style.font.color.with_alpha(0.5)
        } else {
            self.style.font.color
        };

        let text_pos = Vec2::new(
            self.bounds.x + 8.0,
            self.bounds.y + self.bounds.height * 0.5,
        );

        canvas.text(text_pos, display_text, self.style.font.size, text_color);

        // Border
        let border_color = if self.focused {
            Color::BLUE
        } else {
            self.style.border_color
        };

        canvas.rect_outline(
            self.bounds,
            border_color,
            self.style.border_width,
            self.style.corner_radius,
        );
    }

    fn handle_mouse(&mut self, mouse_pos: Vec2, mouse_down: bool) -> bool {
        if mouse_down {
            self.focused = self.bounds.contains_point(mouse_pos);
            self.focused
        } else {
            false
        }
    }
}

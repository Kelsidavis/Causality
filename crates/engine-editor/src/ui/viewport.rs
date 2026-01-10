// Viewport controls - camera manipulation

use engine_render::camera::Camera;
use glam::Vec3;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};

pub struct ViewportControls {
    pub orbit_active: bool,
    pub pan_active: bool,
    pub brush_active: bool,  // Left mouse for brush (just pressed)
    pub brush_held: bool,    // Left mouse being held (for continuous sculpting)
    pub last_mouse_pos: Option<(f32, f32)>,
    pub current_mouse_pos: (f32, f32),
    pub orbit_distance: f32,
    pub orbit_pitch: f32,
    pub orbit_yaw: f32,
    pub pan_offset: Vec3,
    /// Last position where terrain was modified (to throttle updates)
    pub last_terrain_sculpt_pos: Option<(f32, f32)>,
}

impl ViewportControls {
    pub fn new() -> Self {
        Self {
            orbit_active: false,
            pan_active: false,
            brush_active: false,
            brush_held: false,
            last_mouse_pos: None,
            current_mouse_pos: (0.0, 0.0),
            orbit_distance: 15.0,
            orbit_pitch: 30.0_f32.to_radians(),
            orbit_yaw: 45.0_f32.to_radians(),
            pan_offset: Vec3::ZERO,
            last_terrain_sculpt_pos: None,
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left => {
                let was_held = self.brush_held;
                self.brush_held = state == ElementState::Pressed;
                // brush_active is true only on the frame when button is first pressed
                self.brush_active = state == ElementState::Pressed && !was_held;
                // Clear last sculpt position when button is released
                if state == ElementState::Released {
                    self.last_terrain_sculpt_pos = None;
                }
            }
            MouseButton::Right => {
                self.orbit_active = state == ElementState::Pressed;
                if state == ElementState::Released {
                    self.last_mouse_pos = None;
                }
            }
            MouseButton::Middle => {
                self.pan_active = state == ElementState::Pressed;
                if state == ElementState::Released {
                    self.last_mouse_pos = None;
                }
            }
            _ => {}
        }
    }

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        self.current_mouse_pos = (x, y);
        if let Some((last_x, last_y)) = self.last_mouse_pos {
            let delta_x = x - last_x;
            let delta_y = y - last_y;

            if self.orbit_active {
                // Orbit camera
                self.orbit_yaw -= delta_x * 0.005;
                self.orbit_pitch -= delta_y * 0.005;
                self.orbit_pitch = self.orbit_pitch.clamp(-1.5, 1.5);
            } else if self.pan_active {
                // Pan camera
                let pan_speed = 0.01;
                self.pan_offset.x -= delta_x * pan_speed;
                self.pan_offset.y += delta_y * pan_speed;
            }
        }
        self.last_mouse_pos = Some((x, y));
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let zoom_amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
        };

        self.orbit_distance -= zoom_amount;
        self.orbit_distance = self.orbit_distance.clamp(1.0, 100.0);
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        // Calculate camera position based on orbit parameters
        let x = self.orbit_distance * self.orbit_pitch.cos() * self.orbit_yaw.sin();
        let y = self.orbit_distance * self.orbit_pitch.sin();
        let z = self.orbit_distance * self.orbit_pitch.cos() * self.orbit_yaw.cos();

        camera.position = Vec3::new(x, y, z) + self.pan_offset;
        camera.target = self.pan_offset;
    }
}

impl Default for ViewportControls {
    fn default() -> Self {
        Self::new()
    }
}

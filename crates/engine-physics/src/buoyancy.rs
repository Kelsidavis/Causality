// Buoyancy and water physics

use engine_scene::Scene;
use glam::Vec3;
use rapier3d::prelude::*;

pub struct WaterVolume {
    pub center: Vec3,
    pub size: Vec3,
    pub water_level: f32,
    pub density: f32, // Water density (typically 1000 kg/m³)
    pub drag_coefficient: f32,
}

impl Default for WaterVolume {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            size: Vec3::new(100.0, 10.0, 100.0),
            water_level: 0.0,
            density: 1000.0,
            drag_coefficient: 0.5,
        }
    }
}

impl WaterVolume {
    pub fn new(center: Vec3, size: Vec3, water_level: f32) -> Self {
        Self {
            center,
            size,
            water_level,
            ..Default::default()
        }
    }

    /// Check if a point is underwater
    pub fn is_underwater(&self, position: Vec3) -> bool {
        // Check if point is within water volume bounds
        let min = self.center - self.size * 0.5;
        let max = self.center + self.size * 0.5;

        position.x >= min.x
            && position.x <= max.x
            && position.z >= min.z
            && position.z <= max.z
            && position.y < self.water_level
    }

    /// Get submersion depth (0 = at surface, positive = underwater)
    pub fn get_submersion_depth(&self, position: Vec3) -> f32 {
        if !self.is_underwater(position) {
            return 0.0;
        }
        (self.water_level - position.y).max(0.0)
    }

    /// Calculate buoyancy force for a submerged object
    pub fn calculate_buoyancy_force(
        &self,
        position: Vec3,
        volume: f32, // Object volume in m³
    ) -> Vec3 {
        let submersion_depth = self.get_submersion_depth(position);
        if submersion_depth <= 0.0 {
            return Vec3::ZERO;
        }

        // Archimedes' principle: F = ρ * V * g
        // where ρ is fluid density, V is displaced volume, g is gravity
        let gravity = 9.81;
        let buoyancy_magnitude = self.density * volume * gravity;

        // Buoyancy force acts upward
        Vec3::new(0.0, buoyancy_magnitude, 0.0)
    }

    /// Calculate drag force for an object moving through water
    pub fn calculate_drag_force(&self, position: Vec3, velocity: Vec3, cross_section_area: f32) -> Vec3 {
        if !self.is_underwater(position) {
            return Vec3::ZERO;
        }

        // Drag force: F = 0.5 * ρ * v² * Cd * A
        // where ρ is fluid density, v is velocity, Cd is drag coefficient, A is cross-sectional area
        let velocity_magnitude = velocity.length();
        if velocity_magnitude < 0.001 {
            return Vec3::ZERO;
        }

        let drag_magnitude =
            0.5 * self.density * velocity_magnitude * velocity_magnitude * self.drag_coefficient * cross_section_area;

        // Drag opposes motion
        -velocity.normalize() * drag_magnitude
    }
}

/// System to apply buoyancy and water physics to rigid bodies
pub struct BuoyancySystem {
    pub water_volumes: Vec<WaterVolume>,
}

impl BuoyancySystem {
    pub fn new() -> Self {
        Self {
            water_volumes: Vec::new(),
        }
    }

    pub fn add_water_volume(&mut self, volume: WaterVolume) {
        self.water_volumes.push(volume);
    }

    /// Apply buoyancy and drag forces to all rigid bodies in the scene
    pub fn update(
        &self,
        rigid_body_set: &mut RigidBodySet,
        scene: &Scene,
    ) {
        for entity in scene.entities() {
            if let Some(rigid_body_component) = entity.get_component::<crate::components::RigidBody>() {
                // Get rapier rigid body handle from entity
                // Note: This requires a mapping from entity ID to rapier handle
                // For now, we'll just demonstrate the physics calculation

                let position = entity.transform.position;

                // Estimate object volume from collider (simplified)
                let estimated_volume = 1.0; // TODO: Calculate from actual collider
                let cross_section_area = 1.0; // TODO: Calculate from actual collider

                for water_volume in &self.water_volumes {
                    if water_volume.is_underwater(position) {
                        // Calculate buoyancy force
                        let buoyancy_force = water_volume.calculate_buoyancy_force(position, estimated_volume);

                        // Calculate drag force (requires velocity)
                        let velocity = Vec3::from(rigid_body_component.linear_velocity);
                        let drag_force = water_volume.calculate_drag_force(position, velocity, cross_section_area);

                        // Apply forces to rigid body
                        // Note: This would require access to the rapier rigid body handle
                        // For now, this is just the calculation framework
                        log::trace!(
                            "Entity {} underwater: buoyancy={:?}, drag={:?}",
                            entity.name,
                            buoyancy_force,
                            drag_force
                        );
                    }
                }
            }
        }
    }
}

impl Default for BuoyancySystem {
    fn default() -> Self {
        Self::new()
    }
}

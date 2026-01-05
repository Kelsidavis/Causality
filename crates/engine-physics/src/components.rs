// Physics components

use engine_scene::entity::Component;
use engine_scene::impl_component;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Rigid body type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    /// Dynamic body - affected by forces and gravity
    Dynamic,
    /// Kinematic body - can be moved programmatically but not affected by forces
    Kinematic,
    /// Static body - never moves
    Static,
}

/// Rigid body component - adds physics simulation to an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub can_sleep: bool,
    pub ccd_enabled: bool, // Continuous collision detection
}

impl RigidBody {
    pub fn dynamic(mass: f32) -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            can_sleep: true,
            ccd_enabled: false,
        }
    }

    pub fn kinematic() -> Self {
        Self {
            body_type: RigidBodyType::Kinematic,
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            can_sleep: false,
            ccd_enabled: false,
        }
    }

    pub fn static_body() -> Self {
        Self {
            body_type: RigidBodyType::Static,
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            can_sleep: false,
            ccd_enabled: false,
        }
    }

    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.linear_velocity = velocity;
        self
    }

    pub fn with_damping(mut self, linear: f32, angular: f32) -> Self {
        self.linear_damping = linear;
        self.angular_damping = angular;
        self
    }

    pub fn with_ccd(mut self, enabled: bool) -> Self {
        self.ccd_enabled = enabled;
        self
    }
}

impl_component!(RigidBody);

/// Collider shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape {
    /// Box collider with half-extents
    Box { half_extents: Vec3 },
    /// Sphere collider with radius
    Sphere { radius: f32 },
    /// Capsule collider (cylinder with hemispheres on ends)
    Capsule { half_height: f32, radius: f32 },
    /// Cylinder collider
    Cylinder { half_height: f32, radius: f32 },
}

/// Collider component - defines collision shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32, // Bounciness (0 = no bounce, 1 = perfect bounce)
    pub density: f32,
    pub is_sensor: bool, // Sensor colliders detect collisions but don't generate contact forces
}

impl Collider {
    pub fn box_collider(half_extents: Vec3) -> Self {
        Self {
            shape: ColliderShape::Box { half_extents },
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        }
    }

    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        }
    }

    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule { half_height, radius },
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        }
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn as_sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }
}

impl_component!(Collider);

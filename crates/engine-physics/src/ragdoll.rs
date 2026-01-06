// Ragdoll physics system - hierarchical rigidbody chains with joints

use glam::{Quat, Vec3};
use rapier3d::prelude::*;
use engine_scene::entity::EntityId;

use crate::components::ColliderShape;
use crate::joints::{JointConfig, JointHandle};

/// Ragdoll body part definition
#[derive(Debug, Clone)]
pub struct RagdollPart {
    /// Name of the part (e.g., "head", "torso", "left_upper_arm")
    pub name: String,
    /// Collider shape for this part
    pub shape: ColliderShape,
    /// Mass of this part
    pub mass: f32,
    /// Local position relative to parent
    pub local_position: Vec3,
    /// Local rotation relative to parent
    pub local_rotation: Quat,
    /// Index of parent part (None for root)
    pub parent_index: Option<usize>,
    /// Joint configuration connecting to parent
    pub joint_config: Option<JointConfig>,
}

impl RagdollPart {
    /// Create a new ragdoll part
    pub fn new(name: impl Into<String>, shape: ColliderShape, mass: f32) -> Self {
        Self {
            name: name.into(),
            shape,
            mass,
            local_position: Vec3::ZERO,
            local_rotation: Quat::IDENTITY,
            parent_index: None,
            joint_config: None,
        }
    }

    /// Set local position
    pub fn with_position(mut self, position: Vec3) -> Self {
        self.local_position = position;
        self
    }

    /// Set local rotation
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.local_rotation = rotation;
        self
    }

    /// Set parent index
    pub fn with_parent(mut self, parent_index: usize) -> Self {
        self.parent_index = Some(parent_index);
        self
    }

    /// Set joint configuration
    pub fn with_joint(mut self, joint_config: JointConfig) -> Self {
        self.joint_config = Some(joint_config);
        self
    }
}

/// Complete ragdoll configuration
#[derive(Debug, Clone)]
pub struct RagdollConfig {
    /// All body parts in the ragdoll
    pub parts: Vec<RagdollPart>,
    /// Global damping for stability
    pub linear_damping: f32,
    pub angular_damping: f32,
    /// Whether to start active (dynamic) or kinematic
    pub start_active: bool,
}

impl RagdollConfig {
    /// Create a new empty ragdoll configuration
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            linear_damping: 0.5,
            angular_damping: 0.8,
            start_active: false,
        }
    }

    /// Add a part to the ragdoll
    pub fn add_part(mut self, part: RagdollPart) -> Self {
        self.parts.push(part);
        self
    }

    /// Set damping values
    pub fn with_damping(mut self, linear: f32, angular: f32) -> Self {
        self.linear_damping = linear;
        self.angular_damping = angular;
        self
    }

    /// Start the ragdoll in active (dynamic) mode
    pub fn start_active(mut self, active: bool) -> Self {
        self.start_active = active;
        self
    }

    /// Create a simple humanoid ragdoll configuration
    pub fn humanoid() -> Self {
        let mut config = Self::new();

        // Torso (root)
        config = config.add_part(
            RagdollPart::new("torso", ColliderShape::Capsule { radius: 0.2, half_height: 0.3 }, 15.0)
        );

        // Head (connected to torso with ball-socket)
        config = config.add_part(
            RagdollPart::new("head", ColliderShape::Sphere { radius: 0.15 }, 5.0)
                .with_position(Vec3::new(0.0, 0.45, 0.0))
                .with_parent(0)
                .with_joint(JointConfig::spherical(
                    Vec3::new(0.0, 0.3, 0.0),  // Anchor on torso (top)
                    Vec3::new(0.0, -0.15, 0.0), // Anchor on head (bottom)
                ))
        );

        // Pelvis (connected to torso with ball-socket)
        config = config.add_part(
            RagdollPart::new("pelvis", ColliderShape::Capsule { radius: 0.15, half_height: 0.15 }, 10.0)
                .with_position(Vec3::new(0.0, -0.45, 0.0))
                .with_parent(0)
                .with_joint(JointConfig::spherical(
                    Vec3::new(0.0, -0.3, 0.0), // Anchor on torso (bottom)
                    Vec3::new(0.0, 0.15, 0.0),  // Anchor on pelvis (top)
                ))
        );

        // Left upper arm
        config = config.add_part(
            RagdollPart::new("left_upper_arm", ColliderShape::Capsule { radius: 0.05, half_height: 0.15 }, 2.0)
                .with_position(Vec3::new(0.3, 0.15, 0.0))
                .with_parent(0)
                .with_joint(JointConfig::spherical(
                    Vec3::new(0.2, 0.2, 0.0),  // Anchor on torso (shoulder)
                    Vec3::new(0.0, 0.15, 0.0),  // Anchor on arm (top)
                ))
        );

        // Left lower arm
        config = config.add_part(
            RagdollPart::new("left_lower_arm", ColliderShape::Capsule { radius: 0.04, half_height: 0.125 }, 1.5)
                .with_position(Vec3::new(0.3, -0.2, 0.0))
                .with_parent(3)
                .with_joint(
                    JointConfig::revolute(
                        Vec3::new(0.0, -0.15, 0.0), // Anchor on upper arm (bottom)
                        Vec3::new(0.0, 0.125, 0.0),  // Anchor on lower arm (top)
                        Vec3::Z, // Hinge axis
                    ).with_limits(0.0, 2.5) // Elbow can only bend one way
                )
        );

        // Right upper arm (mirror of left)
        config = config.add_part(
            RagdollPart::new("right_upper_arm", ColliderShape::Capsule { radius: 0.05, half_height: 0.15 }, 2.0)
                .with_position(Vec3::new(-0.3, 0.15, 0.0))
                .with_parent(0)
                .with_joint(JointConfig::spherical(
                    Vec3::new(-0.2, 0.2, 0.0),
                    Vec3::new(0.0, 0.15, 0.0),
                ))
        );

        // Right lower arm
        config = config.add_part(
            RagdollPart::new("right_lower_arm", ColliderShape::Capsule { radius: 0.04, half_height: 0.125 }, 1.5)
                .with_position(Vec3::new(-0.3, -0.2, 0.0))
                .with_parent(5)
                .with_joint(
                    JointConfig::revolute(
                        Vec3::new(0.0, -0.15, 0.0),
                        Vec3::new(0.0, 0.125, 0.0),
                        Vec3::Z,
                    ).with_limits(0.0, 2.5)
                )
        );

        // Left upper leg
        config = config.add_part(
            RagdollPart::new("left_upper_leg", ColliderShape::Capsule { radius: 0.08, half_height: 0.2 }, 5.0)
                .with_position(Vec3::new(0.1, -0.85, 0.0))
                .with_parent(2)
                .with_joint(JointConfig::spherical(
                    Vec3::new(0.1, -0.15, 0.0),  // Anchor on pelvis
                    Vec3::new(0.0, 0.2, 0.0),     // Anchor on upper leg (top)
                ))
        );

        // Left lower leg
        config = config.add_part(
            RagdollPart::new("left_lower_leg", ColliderShape::Capsule { radius: 0.06, half_height: 0.2 }, 3.0)
                .with_position(Vec3::new(0.1, -1.45, 0.0))
                .with_parent(7)
                .with_joint(
                    JointConfig::revolute(
                        Vec3::new(0.0, -0.2, 0.0), // Anchor on upper leg (bottom)
                        Vec3::new(0.0, 0.2, 0.0),   // Anchor on lower leg (top)
                        Vec3::X, // Hinge axis
                    ).with_limits(-2.5, 0.0) // Knee can only bend backward
                )
        );

        // Right upper leg
        config = config.add_part(
            RagdollPart::new("right_upper_leg", ColliderShape::Capsule { radius: 0.08, half_height: 0.2 }, 5.0)
                .with_position(Vec3::new(-0.1, -0.85, 0.0))
                .with_parent(2)
                .with_joint(JointConfig::spherical(
                    Vec3::new(-0.1, -0.15, 0.0),
                    Vec3::new(0.0, 0.2, 0.0),
                ))
        );

        // Right lower leg
        config = config.add_part(
            RagdollPart::new("right_lower_leg", ColliderShape::Capsule { radius: 0.06, half_height: 0.2 }, 3.0)
                .with_position(Vec3::new(-0.1, -1.45, 0.0))
                .with_parent(9)
                .with_joint(
                    JointConfig::revolute(
                        Vec3::new(0.0, -0.2, 0.0),
                        Vec3::new(0.0, 0.2, 0.0),
                        Vec3::X,
                    ).with_limits(-2.5, 0.0)
                )
        );

        config
    }
}

impl Default for RagdollConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Active ragdoll instance
pub struct Ragdoll {
    /// Entity IDs for each body part
    pub part_entities: Vec<EntityId>,
    /// Rigidbody handles for each part
    pub part_bodies: Vec<RigidBodyHandle>,
    /// Joint handles connecting parts
    pub joint_handles: Vec<JointHandle>,
    /// Whether the ragdoll is currently active (dynamic)
    pub is_active: bool,
}

impl Ragdoll {
    /// Create a new ragdoll instance
    pub fn new() -> Self {
        Self {
            part_entities: Vec::new(),
            part_bodies: Vec::new(),
            joint_handles: Vec::new(),
            is_active: false,
        }
    }

    /// Activate ragdoll (switch all parts to dynamic)
    pub fn activate(&mut self, rigid_body_set: &mut RigidBodySet) {
        if self.is_active {
            return;
        }

        for handle in &self.part_bodies {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                body.set_body_type(rapier3d::prelude::RigidBodyType::Dynamic, true);
            }
        }

        self.is_active = true;
    }

    /// Deactivate ragdoll (switch all parts to kinematic)
    pub fn deactivate(&mut self, rigid_body_set: &mut RigidBodySet) {
        if !self.is_active {
            return;
        }

        for handle in &self.part_bodies {
            if let Some(body) = rigid_body_set.get_mut(*handle) {
                body.set_body_type(rapier3d::prelude::RigidBodyType::KinematicPositionBased, true);
            }
        }

        self.is_active = false;
    }

    /// Apply impulse to a specific body part
    pub fn apply_impulse(
        &self,
        rigid_body_set: &mut RigidBodySet,
        part_index: usize,
        impulse: Vec3,
    ) -> bool {
        if part_index >= self.part_bodies.len() {
            return false;
        }

        if let Some(body) = rigid_body_set.get_mut(self.part_bodies[part_index]) {
            body.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
            return true;
        }

        false
    }

    /// Get position of a specific body part
    pub fn get_part_position(
        &self,
        rigid_body_set: &RigidBodySet,
        part_index: usize,
    ) -> Option<Vec3> {
        if part_index >= self.part_bodies.len() {
            return None;
        }

        rigid_body_set.get(self.part_bodies[part_index]).map(|body| {
            let pos = body.translation();
            Vec3::new(pos.x, pos.y, pos.z)
        })
    }

    /// Get rotation of a specific body part
    pub fn get_part_rotation(
        &self,
        rigid_body_set: &RigidBodySet,
        part_index: usize,
    ) -> Option<Quat> {
        if part_index >= self.part_bodies.len() {
            return None;
        }

        rigid_body_set.get(self.part_bodies[part_index]).map(|body| {
            let rot = body.rotation();
            Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w)
        })
    }

    /// Get the number of parts in this ragdoll
    pub fn part_count(&self) -> usize {
        self.part_entities.len()
    }
}

impl Default for Ragdoll {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ragdoll_part() {
        let part = RagdollPart::new(
            "test_part",
            ColliderShape::Capsule { radius: 0.1, half_height: 0.25 },
            2.0,
        )
        .with_position(Vec3::new(1.0, 2.0, 3.0))
        .with_rotation(Quat::IDENTITY)
        .with_parent(0);

        assert_eq!(part.name, "test_part");
        assert_eq!(part.mass, 2.0);
        assert_eq!(part.local_position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(part.parent_index, Some(0));
    }

    #[test]
    fn test_ragdoll_config() {
        let config = RagdollConfig::new()
            .with_damping(0.5, 0.8)
            .start_active(true);

        assert_eq!(config.linear_damping, 0.5);
        assert_eq!(config.angular_damping, 0.8);
        assert!(config.start_active);
    }

    #[test]
    fn test_humanoid_ragdoll() {
        let config = RagdollConfig::humanoid();

        // Should have torso, head, pelvis, 4 arms, 4 legs = 11 parts
        assert_eq!(config.parts.len(), 11);

        // Root (torso) should have no parent
        assert!(config.parts[0].parent_index.is_none());

        // All other parts should have parents
        for part in &config.parts[1..] {
            assert!(part.parent_index.is_some());
            assert!(part.joint_config.is_some());
        }
    }

    #[test]
    fn test_ragdoll_instance() {
        let ragdoll = Ragdoll::new();
        assert_eq!(ragdoll.part_count(), 0);
        assert!(!ragdoll.is_active);
    }
}

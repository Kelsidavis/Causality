// Physics joints and constraints - connects rigidbodies together

use rapier3d::prelude::*;
use glam::Vec3;

/// Joint type enumeration
#[derive(Debug, Clone, Copy)]
pub enum JointType {
    /// Fixed joint - welds two bodies together
    Fixed,
    /// Revolute joint - allows rotation around one axis (hinge)
    Revolute { axis: Vec3 },
    /// Spherical joint - allows rotation in all directions (ball-socket)
    Spherical,
    /// Prismatic joint - allows sliding along one axis
    Prismatic { axis: Vec3 },
}

/// Joint configuration
#[derive(Debug, Clone)]
pub struct JointConfig {
    /// Type of joint
    pub joint_type: JointType,
    /// First body anchor point (local space)
    pub anchor1: Vec3,
    /// Second body anchor point (local space)
    pub anchor2: Vec3,
    /// Joint limits (min, max) for revolute/prismatic
    pub limits: Option<(f32, f32)>,
    /// Motor settings (target velocity, max force)
    pub motor: Option<(f32, f32)>,
}

impl Default for JointConfig {
    fn default() -> Self {
        Self {
            joint_type: JointType::Fixed,
            anchor1: Vec3::ZERO,
            anchor2: Vec3::ZERO,
            limits: None,
            motor: None,
        }
    }
}

/// Joint handle wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JointHandle(pub ImpulseJointHandle);

impl JointConfig {
    /// Create a fixed joint configuration
    pub fn fixed(anchor1: Vec3, anchor2: Vec3) -> Self {
        Self {
            joint_type: JointType::Fixed,
            anchor1,
            anchor2,
            limits: None,
            motor: None,
        }
    }

    /// Create a revolute joint (hinge) configuration
    pub fn revolute(anchor1: Vec3, anchor2: Vec3, axis: Vec3) -> Self {
        Self {
            joint_type: JointType::Revolute { axis },
            anchor1,
            anchor2,
            limits: None,
            motor: None,
        }
    }

    /// Create a spherical joint (ball-socket) configuration
    pub fn spherical(anchor1: Vec3, anchor2: Vec3) -> Self {
        Self {
            joint_type: JointType::Spherical,
            anchor1,
            anchor2,
            limits: None,
            motor: None,
        }
    }

    /// Create a prismatic joint (slider) configuration
    pub fn prismatic(anchor1: Vec3, anchor2: Vec3, axis: Vec3) -> Self {
        Self {
            joint_type: JointType::Prismatic { axis },
            anchor1,
            anchor2,
            limits: None,
            motor: None,
        }
    }

    /// Set joint limits
    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.limits = Some((min, max));
        self
    }

    /// Set motor parameters
    pub fn with_motor(mut self, target_velocity: f32, max_force: f32) -> Self {
        self.motor = Some((target_velocity, max_force));
        self
    }

    /// Build Rapier joint from configuration
    pub fn build(&self) -> GenericJoint {
        let anchor1 = point![self.anchor1.x, self.anchor1.y, self.anchor1.z];
        let anchor2 = point![self.anchor2.x, self.anchor2.y, self.anchor2.z];

        match self.joint_type {
            JointType::Fixed => {
                GenericJointBuilder::new(JointAxesMask::LOCKED_FIXED_AXES)
                    .local_anchor1(anchor1)
                    .local_anchor2(anchor2)
                    .build()
            }
            JointType::Revolute { axis } => {
                let axis = UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]);
                let mut builder = RevoluteJointBuilder::new(axis)
                    .local_anchor1(anchor1)
                    .local_anchor2(anchor2);

                // Apply limits if specified
                if let Some((min, max)) = self.limits {
                    builder = builder.limits([min, max]);
                }

                // Apply motor if specified
                if let Some((target_vel, max_force)) = self.motor {
                    builder = builder.motor_velocity(target_vel, max_force);
                }

                builder.build().into()
            }
            JointType::Spherical => {
                SphericalJointBuilder::new()
                    .local_anchor1(anchor1)
                    .local_anchor2(anchor2)
                    .build()
                    .into()
            }
            JointType::Prismatic { axis } => {
                let axis = UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]);
                let mut builder = PrismaticJointBuilder::new(axis)
                    .local_anchor1(anchor1)
                    .local_anchor2(anchor2);

                // Apply limits if specified
                if let Some((min, max)) = self.limits {
                    builder = builder.limits([min, max]);
                }

                // Apply motor if specified
                if let Some((target_vel, max_force)) = self.motor {
                    builder = builder.motor_velocity(target_vel, max_force);
                }

                builder.build().into()
            }
        }
    }
}

/// Joint manager for physics world
pub struct JointManager {
    joints: Vec<(JointHandle, JointConfig)>,
}

impl JointManager {
    /// Create a new joint manager
    pub fn new() -> Self {
        Self {
            joints: Vec::new(),
        }
    }

    /// Add a joint to the physics world
    pub fn add_joint(
        &mut self,
        impulse_joint_set: &mut ImpulseJointSet,
        body1: RigidBodyHandle,
        body2: RigidBodyHandle,
        config: JointConfig,
    ) -> JointHandle {
        let joint = config.build();
        let handle = impulse_joint_set.insert(body1, body2, joint, true);
        let joint_handle = JointHandle(handle);
        self.joints.push((joint_handle, config.clone()));
        joint_handle
    }

    /// Remove a joint from the physics world
    pub fn remove_joint(
        &mut self,
        impulse_joint_set: &mut ImpulseJointSet,
        joint_handle: JointHandle,
    ) -> bool {
        if impulse_joint_set.remove(joint_handle.0, true).is_some() {
            self.joints.retain(|(h, _)| h != &joint_handle);
            true
        } else {
            false
        }
    }

    /// Get joint configuration
    pub fn get_joint_config(&self, joint_handle: JointHandle) -> Option<&JointConfig> {
        self.joints
            .iter()
            .find(|(h, _)| h == &joint_handle)
            .map(|(_, config)| config)
    }

    /// Update joint motor parameters
    pub fn set_motor(
        &mut self,
        impulse_joint_set: &mut ImpulseJointSet,
        joint_handle: JointHandle,
        target_velocity: f32,
        max_force: f32,
    ) -> bool {
        if let Some(joint) = impulse_joint_set.get_mut(joint_handle.0) {
            // Find joint config
            if let Some((_, config)) = self.joints.iter_mut().find(|(h, _)| h == &joint_handle) {
                config.motor = Some((target_velocity, max_force));

                // Update the actual joint
                match config.joint_type {
                    JointType::Revolute { .. } => {
                        joint.data.set_motor(JointAxis::AngX, target_velocity, max_force, 1.0, 1.0);
                    }
                    JointType::Prismatic { .. } => {
                        joint.data.set_motor(JointAxis::LinX, target_velocity, max_force, 1.0, 1.0);
                    }
                    _ => return false,
                }
                return true;
            }
        }
        false
    }

    /// Get number of joints
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// Clear all joints
    pub fn clear(&mut self) {
        self.joints.clear();
    }
}

impl Default for JointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_config_builder() {
        let config = JointConfig::revolute(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::Y,
        )
        .with_limits(-1.57, 1.57)
        .with_motor(0.0, 10.0);

        assert!(matches!(config.joint_type, JointType::Revolute { .. }));
        assert_eq!(config.limits, Some((-1.57, 1.57)));
        assert_eq!(config.motor, Some((0.0, 10.0)));
    }

    #[test]
    fn test_joint_manager() {
        let mut manager = JointManager::new();
        assert_eq!(manager.joint_count(), 0);

        // Note: We can't fully test without a physics world,
        // but we can verify the manager structure
        manager.clear();
        assert_eq!(manager.joint_count(), 0);
    }
}

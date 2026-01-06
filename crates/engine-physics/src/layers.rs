// Collision layers and filtering

/// Collision layer groups for filtering physics interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionGroups {
    /// Which groups this collider belongs to (bitmask)
    pub memberships: u32,
    /// Which groups this collider interacts with (bitmask)
    pub filter: u32,
}

impl CollisionGroups {
    /// Create new collision groups
    pub fn new(memberships: u32, filter: u32) -> Self {
        Self {
            memberships,
            filter,
        }
    }

    /// Default groups - belongs to group 0, interacts with all groups
    pub fn all() -> Self {
        Self {
            memberships: 0xFFFF_FFFF,
            filter: 0xFFFF_FFFF,
        }
    }

    /// Create groups from a single layer (1-32)
    pub fn from_layer(layer: u32) -> Self {
        assert!(layer > 0 && layer <= 32, "Layer must be between 1 and 32");
        let bit = 1 << (layer - 1);
        Self {
            memberships: bit,
            filter: 0xFFFF_FFFF, // Interact with all by default
        }
    }

    /// Set which layers this collider belongs to
    pub fn with_memberships(mut self, layers: &[u32]) -> Self {
        self.memberships = 0;
        for &layer in layers {
            assert!(layer > 0 && layer <= 32, "Layer must be between 1 and 32");
            self.memberships |= 1 << (layer - 1);
        }
        self
    }

    /// Set which layers this collider interacts with
    pub fn with_filter(mut self, layers: &[u32]) -> Self {
        self.filter = 0;
        for &layer in layers {
            assert!(layer > 0 && layer <= 32, "Layer must be between 1 and 32");
            self.filter |= 1 << (layer - 1);
        }
        self
    }

    /// Convert to Rapier's InteractionGroups
    pub fn to_rapier(&self) -> rapier3d::prelude::InteractionGroups {
        rapier3d::prelude::InteractionGroups::new(
            rapier3d::prelude::Group::from_bits_truncate(self.memberships),
            rapier3d::prelude::Group::from_bits_truncate(self.filter),
        )
    }
}

impl Default for CollisionGroups {
    fn default() -> Self {
        Self::all()
    }
}

/// Predefined collision layers for common use cases
pub mod layers {
    /// Default layer for world geometry
    pub const WORLD: u32 = 1;
    /// Player character layer
    pub const PLAYER: u32 = 2;
    /// Enemy characters layer
    pub const ENEMY: u32 = 3;
    /// Projectiles layer
    pub const PROJECTILE: u32 = 4;
    /// Triggers/sensors layer
    pub const TRIGGER: u32 = 5;
    /// Items/pickups layer
    pub const ITEM: u32 = 6;
    /// Ragdoll bodies layer
    pub const RAGDOLL: u32 = 7;
    /// Debris/props layer
    pub const DEBRIS: u32 = 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_groups() {
        let groups = CollisionGroups::from_layer(layers::PLAYER);
        assert_eq!(groups.memberships, 1 << (layers::PLAYER - 1));
        assert_eq!(groups.filter, 0xFFFF_FFFF);
    }

    #[test]
    fn test_multiple_layers() {
        let groups = CollisionGroups::all()
            .with_memberships(&[layers::PLAYER, layers::RAGDOLL])
            .with_filter(&[layers::WORLD, layers::ENEMY]);

        let player_bit = 1 << (layers::PLAYER - 1);
        let ragdoll_bit = 1 << (layers::RAGDOLL - 1);
        assert_eq!(groups.memberships, player_bit | ragdoll_bit);

        let world_bit = 1 << (layers::WORLD - 1);
        let enemy_bit = 1 << (layers::ENEMY - 1);
        assert_eq!(groups.filter, world_bit | enemy_bit);
    }

    #[test]
    #[should_panic]
    fn test_invalid_layer() {
        CollisionGroups::from_layer(33); // Should panic
    }
}

// Frustum culling - skip rendering objects outside camera view

use glam::{Mat4, Vec3, Vec4};

/// Frustum plane
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector pointing inward
    pub normal: Vec3,
    /// Distance from origin
    pub distance: f32,
}

impl Plane {
    /// Create a plane from a normal and distance
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create a plane from a 4D vector (normal + distance)
    pub fn from_vec4(v: Vec4) -> Self {
        let normal = Vec3::new(v.x, v.y, v.z);
        let length = normal.length();
        Self {
            normal: normal / length,
            distance: v.w / length,
        }
    }

    /// Distance from point to plane (positive = in front, negative = behind)
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// View frustum with 6 planes
#[derive(Debug, Clone)]
pub struct Frustum {
    /// Left plane
    pub left: Plane,
    /// Right plane
    pub right: Plane,
    /// Bottom plane
    pub bottom: Plane,
    /// Top plane
    pub top: Plane,
    /// Near plane
    pub near: Plane,
    /// Far plane
    pub far: Plane,
}

impl Frustum {
    /// Extract frustum planes from view-projection matrix
    pub fn from_view_projection(view_proj: Mat4) -> Self {
        // Extract planes using Gribb-Hartmann method
        let m = view_proj.to_cols_array_2d();

        // Left plane: row4 + row1
        let left = Plane::from_vec4(Vec4::new(
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        ));

        // Right plane: row4 - row1
        let right = Plane::from_vec4(Vec4::new(
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        ));

        // Bottom plane: row4 + row2
        let bottom = Plane::from_vec4(Vec4::new(
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        ));

        // Top plane: row4 - row2
        let top = Plane::from_vec4(Vec4::new(
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        ));

        // Near plane: row4 + row3
        let near = Plane::from_vec4(Vec4::new(
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        ));

        // Far plane: row4 - row3
        let far = Plane::from_vec4(Vec4::new(
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        ));

        Self {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    /// Test if a sphere is inside or intersecting the frustum
    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        // Check against all 6 planes
        for plane in [&self.left, &self.right, &self.bottom, &self.top, &self.near, &self.far] {
            let distance = plane.distance_to_point(center);
            if distance < -radius {
                // Sphere is completely outside this plane
                return false;
            }
        }
        true
    }

    /// Test if an axis-aligned bounding box (AABB) is inside or intersecting the frustum
    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        // Check against all 6 planes
        for plane in [&self.left, &self.right, &self.bottom, &self.top, &self.near, &self.far] {
            // Get positive and negative vertices
            let mut p = min;
            if plane.normal.x >= 0.0 {
                p.x = max.x;
            }
            if plane.normal.y >= 0.0 {
                p.y = max.y;
            }
            if plane.normal.z >= 0.0 {
                p.z = max.z;
            }

            // If positive vertex is outside, AABB is outside
            if plane.distance_to_point(p) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a point is inside the frustum
    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in [&self.left, &self.right, &self.bottom, &self.top, &self.near, &self.far] {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    /// Minimum corner
    pub min: Vec3,
    /// Maximum corner
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a center and half-extents
    pub fn from_center_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Get center point
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get half-extents (distance from center to edge)
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Get bounding sphere radius
    pub fn bounding_sphere_radius(&self) -> f32 {
        self.half_extents().length()
    }

    /// Transform AABB by a matrix
    pub fn transform(&self, matrix: Mat4) -> Self {
        // Transform all 8 corners and find new min/max
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for corner in corners {
            let transformed = matrix.transform_point3(corner);
            min = min.min(transformed);
            max = max.max(transformed);
        }

        Self { min, max }
    }

    /// Merge with another AABB
    pub fn merge(&self, other: &AABB) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Check if AABB contains a point
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if AABB intersects another AABB
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Ray-AABB intersection test using slab method
    /// Returns Some(t) where t is the distance along the ray to the intersection point,
    /// or None if there's no intersection
    pub fn ray_intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / ray_direction.x,
            1.0 / ray_direction.y,
            1.0 / ray_direction.z,
        );

        let t1 = (self.min.x - ray_origin.x) * inv_dir.x;
        let t2 = (self.max.x - ray_origin.x) * inv_dir.x;
        let t3 = (self.min.y - ray_origin.y) * inv_dir.y;
        let t4 = (self.max.y - ray_origin.y) * inv_dir.y;
        let t5 = (self.min.z - ray_origin.z) * inv_dir.z;
        let t6 = (self.max.z - ray_origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        // If tmax < 0, ray is intersecting AABB but behind origin
        // If tmin > tmax, ray doesn't intersect AABB
        if tmax < 0.0 || tmin > tmax {
            return None;
        }

        // Return the closest intersection point (tmin if positive, otherwise tmax)
        if tmin >= 0.0 {
            Some(tmin)
        } else {
            Some(tmax)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_center() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.half_extents(), Vec3::ONE);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains_point(Vec3::ZERO));
        assert!(aabb.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!aabb.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_aabb_intersects() {
        let aabb1 = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let aabb2 = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        let aabb3 = AABB::new(Vec3::new(3.0, 3.0, 3.0), Vec3::new(4.0, 4.0, 4.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_frustum_contains_sphere() {
        // Create a simple orthographic-like frustum
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);

        // Sphere at origin should be visible
        assert!(frustum.contains_sphere(Vec3::ZERO, 1.0));

        // Sphere far away should not be visible
        assert!(!frustum.contains_sphere(Vec3::new(0.0, 0.0, -200.0), 1.0));
    }

    #[test]
    fn test_frustum_contains_aabb() {
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);

        // AABB at origin should be visible
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(frustum.contains_aabb(aabb.min, aabb.max));

        // AABB far away should not be visible
        let far_aabb = AABB::new(Vec3::new(-1.0, -1.0, -200.0), Vec3::new(1.0, 1.0, -198.0));
        assert!(!frustum.contains_aabb(far_aabb.min, far_aabb.max));
    }
}

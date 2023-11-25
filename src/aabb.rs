use bevy::prelude::*;

/// Axis-aligned bounding box in 2d.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}
impl Aabb2 {
    /// Clamp a 2d vector to the bounding box.
    pub fn clamp2(&self, vec: &mut Vec2) {
        vec.x = vec.x.clamp(self.min.x, self.max.x);
        vec.y = vec.y.clamp(self.min.y, self.max.y);
    }
    /// Clamp a 3d vector (ignoring Z) to the bounding box.
    pub fn clamp3(&self, vec: &mut Vec3) {
        vec.x = vec.x.clamp(self.min.x, self.max.x);
        vec.y = vec.y.clamp(self.min.y, self.max.y);
    }
}

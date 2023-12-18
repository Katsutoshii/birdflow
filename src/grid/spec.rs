use crate::prelude::*;
use bevy::prelude::*;

/// Specification describing how large the grid is.
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct GridSpec {
    pub rows: u16,
    pub cols: u16,
    pub width: f32,
    pub visualize: bool,
}
impl Default for GridSpec {
    fn default() -> Self {
        Self {
            rows: 10,
            cols: 10,
            width: 10.0,
            visualize: true,
        }
    }
}
impl GridSpec {
    pub fn discretize(&self, value: f32) -> u16 {
        (value / self.width) as u16
    }
    // Covert row, col to a single index.
    pub fn index(&self, row: u16, col: u16) -> usize {
        row as usize * self.cols as usize + col as usize
    }

    /// Returns (row, col) from a position in world space.
    pub fn to_rowcol(&self, mut position: Vec2) -> (u16, u16) {
        position += self.offset();
        (self.discretize(position.y), self.discretize(position.x))
    }

    /// Returns the world position of the cell coordinate.
    pub fn to_world_position(&self, cell: (u16, u16)) -> Vec2 {
        let (row, col) = cell;
        Vec2 {
            x: (col as f32 + 0.5) * self.width,
            y: (row as f32 + 0.5) * self.width,
        } - self.offset()
    }

    /// Compute the offset vector for this grid spec.
    pub fn offset(&self) -> Vec2 {
        Vec2 {
            x: self.width * self.cols as f32 / 2.,
            y: self.width * self.rows as f32 / 2.,
        }
    }

    /// Compute the (min, max) position for the grid.
    pub fn world2d_bounds(&self) -> Aabb2 {
        Aabb2 {
            min: -self.offset(),
            max: self.offset(),
        }
    }

    pub fn scale(&self) -> Vec2 {
        Vec2 {
            x: self.width * self.cols as f32,
            y: self.width * self.rows as f32,
        }
    }
}

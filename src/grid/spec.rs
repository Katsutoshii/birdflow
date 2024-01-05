use crate::prelude::*;
use bevy::prelude::*;

use super::RowCol;

/// Specification describing how large the grid is.
#[derive(Resource, Reflect, Clone, Debug)]
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
    pub fn flat_index(&self, rowcol: RowCol) -> usize {
        let (row, col) = rowcol;
        row as usize * self.cols as usize + col as usize
    }

    /// Returns (row, col) from a position in world space.
    pub fn to_rowcol(&self, mut position: Vec2) -> RowCol {
        position += self.offset();
        (self.discretize(position.y), self.discretize(position.x))
    }

    /// Returns the world position of the cell coordinate.
    pub fn to_world_position(&self, rowcol: RowCol) -> Vec2 {
        let (row, col) = rowcol;
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

    /// Returns true iff the rowcol is on the boundary of the grid.
    pub fn is_boundary(&self, rowcol: RowCol) -> bool {
        let (row, col) = rowcol;
        if row == 0 || row == self.rows - 1 {
            return true;
        }
        if col == 0 || col == self.cols - 1 {
            return true;
        }
        false
    }
}

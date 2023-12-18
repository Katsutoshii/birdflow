use std::ops::{Index, IndexMut, RangeInclusive};

use bevy::prelude::*;

mod spec;
pub use spec::GridSpec;
mod fog;
pub use fog::FogPlugin;
mod visualizer;
pub use visualizer::{GridShaderMaterial, GridVisualizer};
mod entity;
pub use entity::{EntityGrid, EntityGridEvent, GridEntity};
mod obstacles;
pub use obstacles::{Obstacle, ObstaclesGrid, ObstaclesPlugin};

use crate::{Aabb2, SystemStage};

use self::visualizer::GridVisualizerPlugin;

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .add_event::<EntityGridEvent>()
            .add_plugins(GridVisualizerPlugin)
            .add_plugins(ObstaclesPlugin)
            .add_plugins(FogPlugin)
            .insert_resource(EntityGrid::default())
            .add_systems(
                FixedUpdate,
                (
                    GridEntity::update.in_set(SystemStage::PostApply),
                    EntityGrid::resize_on_change,
                ),
            );
    }
}

/// 2D Grid containing arbitrary data.
#[derive(Clone, Default)]
pub struct Grid2<T: Sized + Default + Clone> {
    pub spec: GridSpec,
    cells: Vec<T>,
}
impl<T: Sized + Default + Clone> Index<(u16, u16)> for Grid2<T> {
    type Output = T;
    fn index(&self, i: (u16, u16)) -> &Self::Output {
        let (row, col) = i;
        &self.cells[self.spec.index(row, col)]
    }
}
impl<T: Sized + Default + Clone> IndexMut<(u16, u16)> for Grid2<T> {
    fn index_mut(&mut self, i: (u16, u16)) -> &mut T {
        let (row, col) = i;
        &mut self.cells[self.spec.index(row, col)]
    }
}
impl<T: Sized + Default + Clone> Grid2<T> {
    /// Resize the grid to match the given spec.
    pub fn resize_with(&mut self, spec: GridSpec) {
        self.spec = spec;
        self.resize();
    }
    /// Resize the grid.
    pub fn resize(&mut self) {
        let num_cells = self.spec.rows as usize * self.spec.cols as usize;
        self.cells.resize(num_cells, T::default());
    }

    /// Get all entities in a given bounding box.
    pub fn get_in_aabb(&self, aabb: &Aabb2) -> Vec<(u16, u16)> {
        let mut results = Vec::default();

        let (min_row, min_col) = self.spec.to_rowcol(aabb.min);
        let (max_row, max_col) = self.spec.to_rowcol(aabb.max);
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                results.push((row, col))
            }
        }
        results
    }

    /// Get in radius.
    pub fn get_in_radius(&self, position: Vec2, radius: f32) -> Vec<(u16, u16)> {
        self.get_in_radius_discrete(self.spec.to_rowcol(position), self.spec.discretize(radius))
    }

    /// Get in radius, with discrete cell position inputs.
    pub fn get_in_radius_discrete(&self, cell: (u16, u16), radius: u16) -> Vec<(u16, u16)> {
        let (row, col) = cell;

        let mut results = Vec::default();
        for other_row in Self::cell_range(row, radius) {
            for other_col in Self::cell_range(col, radius) {
                let other_cell = (other_row, other_col);
                if !Self::in_radius(cell, other_cell, radius) {
                    continue;
                }
                results.push(other_cell)
            }
        }
        results
    }

    /// Returns true if a cell is within the given radius to another cell.
    pub fn in_radius(cell: (u16, u16), other_cell: (u16, u16), radius: u16) -> bool {
        let (row, col) = cell;
        let (other_row, other_col) = other_cell;
        let row_dist = other_row as i16 - row as i16;
        let col_dist = other_col as i16 - col as i16;
        row_dist * row_dist + col_dist * col_dist < radius as i16 * radius as i16
    }

    /// Returns a range starting at `center - radius` ending at `center + radius`.
    fn cell_range(center: u16, radius: u16) -> RangeInclusive<u16> {
        let (min, max) = (
            (center as i16 - radius as i16).max(0) as u16,
            center + radius,
        );
        min..=max
    }

    /// Get the set of entities at the current position.
    pub fn get(&self, row: u16, col: u16) -> Option<&T> {
        let index = self.spec.index(row, col);
        self.cells.get(index)
    }

    /// Get the mutable set of entities at the current position.
    pub fn get_mut(&mut self, row: u16, col: u16) -> Option<&mut T> {
        let index = self.spec.index(row, col);
        self.cells.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::Grid2;

    #[test]
    fn grid_radius() {
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (2, 2);
            let radius = 2;
            assert!(Grid2::<()>::in_radius(
                (row, col),
                (other_row, other_col),
                radius
            ));
        }
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (4, 4);
            let radius = 2;
            assert!(!Grid2::<()>::in_radius(
                (row, col),
                (other_row, other_col),
                radius
            ));
        }
    }
}

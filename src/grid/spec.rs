use crate::{
    grid::fog::{FogAssets, FogPlane, FogShaderMaterial},
    prelude::*,
};
use bevy::prelude::*;

use super::{CellVisualizer, EntityGrid, GridAssets, GridShaderMaterial};

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

    /// When the spec changes, update the grid spec and resize.
    pub fn resize_on_change(spec: Res<GridSpec>, mut grid: ResMut<EntityGrid>) {
        if !spec.is_changed() {
            return;
        }

        grid.spec = spec.clone();
        grid.resize();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn visualize_on_change(
        spec: Res<Self>,
        grid_assets: Res<GridAssets>,
        fog_assets: Res<FogAssets>,

        query: Query<Entity, With<CellVisualizer>>,
        fog_query: Query<Entity, With<FogPlane>>,

        mut grid_shader_assets: ResMut<Assets<GridShaderMaterial>>,
        mut fog_shader_assets: ResMut<Assets<FogShaderMaterial>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for entity in &query {
            commands.entity(entity).despawn();
        }
        for entity in &fog_query {
            commands.entity(entity).despawn();
        }

        // Initialize the grid visualization shader.
        {
            let material = grid_shader_assets
                .get_mut(&grid_assets.shader_material)
                .unwrap();
            material.resize(&spec);
        }

        // Initialize the fog shader, which also uses the grid spec.
        {
            let material = fog_shader_assets
                .get_mut(&fog_assets.shader_material)
                .unwrap();
            material.resize(&spec);
        }

        commands.spawn(CellVisualizer { active: true }.bundle(&spec, &grid_assets));
        commands.spawn(FogPlane.bundle(&spec, &fog_assets));
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

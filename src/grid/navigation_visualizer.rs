use crate::prelude::*;
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use super::{
    navigation::NavigationCostEvent,
    shader_plane::{ShaderPlaneAssets, ShaderPlanePlugin},
    GridShaderMaterial,
};

pub struct NavigationVisualizerPlugin;
impl Plugin for NavigationVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<NavigationShaderMaterial>::default())
            .add_systems(FixedUpdate, (NavigationShaderMaterial::update,));
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct NavigationShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    size: GridSize,
    #[storage(2, read_only)]
    grid: Vec<f32>,
}
impl Default for NavigationShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::ORANGE_RED,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl GridShaderMaterial for NavigationShaderMaterial {
    fn zindex() -> f32 {
        zindex::NAVIGATION_LAYER
    }
    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid
            .resize(spec.rows as usize * spec.cols as usize, 0.);
    }
}
impl NavigationShaderMaterial {
    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        mut events: EventReader<NavigationCostEvent>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut input_actions: EventReader<InputActionEvent>,
    ) {
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        for &InputActionEvent {
            action,
            position: _,
        } in input_actions.read()
        {
            if action == InputAction::StartMove {
                material.grid = vec![0.; material.grid.len()];
            }
        }
        for &NavigationCostEvent {
            entity: _,
            rowcol,
            cost,
        } in events.read()
        {
            material.grid[grid_spec.flat_index(rowcol)] = cost * 0.002;
        }
    }
}
impl Material2d for NavigationShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/navigation_shader.wgsl".into()
    }
}

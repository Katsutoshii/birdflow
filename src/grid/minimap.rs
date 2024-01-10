use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
    window::PrimaryWindow,
};

use crate::prelude::*;

use super::{
    shader_plane::{ShaderPlaneAssets, ShaderPlanePlugin},
    ShaderPlaneMaterial,
};

/// Plugin for visualizing the grid.
/// This plugin reads events from the entity grid and updates the shader's input buffer
/// to light up the cells that have entities.
pub struct MinimapPlugin;
impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<MinimapShaderMaterial>::default())
            .add_systems(
                FixedUpdate,
                MinimapShaderMaterial::update.after(GridEntity::update),
            );
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct MinimapShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    size: GridSize,
    #[storage(2, read_only)]
    grid: Vec<u32>,
    #[uniform(3)]
    offset: Vec2,
    #[uniform(4)]
    viewport: Vec2,
}
impl Default for MinimapShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::GRAY,
            size: GridSize::default(),
            grid: Vec::default(),
            offset: Vec2::default(),
            viewport: Vec2::default(),
        }
    }
}
impl ShaderPlaneMaterial for MinimapShaderMaterial {
    fn scale(window: &Window, _spec: &GridSpec) -> Vec3 {
        let viewport_size = Vec2 {
            x: window.physical_width() as f32,
            y: window.physical_height() as f32,
        } / window.scale_factor() as f32;
        let quad_size = viewport_size.xx() / 8.;
        (quad_size * Vec2 { x: 1., y: -1. }).extend(1.)
    }
    fn translation(window: &Window, _spec: &GridSpec) -> Vec3 {
        let viewport_size = Vec2 {
            x: window.physical_width() as f32,
            y: window.physical_height() as f32,
        } / window.scale_factor() as f32;
        let quad_size = viewport_size.xx() / 8.;

        let mut translation = Vec2::ZERO;
        translation += Vec2 {
            x: viewport_size.x,
            y: -viewport_size.y,
        } / 2.;
        translation -= Vec2 {
            x: quad_size.x,
            y: -quad_size.y,
        } / 2.;
        dbg!(translation);
        translation.extend(zindex::MINIMAP)
    }

    fn resize(&mut self, spec: &GridSpec) {
        // quad_size = (rows / subsample) * width
        // width = quad_size / (rows)
        self.size.rows = spec.rows as u32 / Self::SUBSAMPLE as u32;
        self.size.cols = spec.cols as u32 / Self::SUBSAMPLE as u32;
        self.size.width = spec.width * Self::SUBSAMPLE as f32;
        self.grid
            .resize(self.size.rows as usize * self.size.cols as usize, 0);
    }
    fn parent_camera() -> bool {
        true
    }
}
impl MinimapShaderMaterial {
    const SUBSAMPLE: u16 = 8;
    /// Update the grid shader material.
    pub fn update(
        configs: Res<Configs>,
        grid_spec: Res<GridSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        window: Query<&Window, With<PrimaryWindow>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut grid_events: EventReader<EntityGridEvent>,
    ) {
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();

        let mut spec = grid_spec.clone();
        spec.rows /= Self::SUBSAMPLE;
        spec.cols /= Self::SUBSAMPLE;

        if configs.is_changed() {
            let window = window.single();
            let viewport_size = Vec2 {
                x: window.physical_width() as f32,
                y: window.physical_height() as f32,
            };
            dbg!(viewport_size);
            material.viewport = viewport_size;
            material.offset =
                (Self::translation(window, &spec) * window.scale_factor() as f32).xy();
        }

        for &EntityGridEvent {
            entity: _,
            prev_cell,
            prev_cell_empty,
            cell,
        } in grid_events.read()
        {
            if let Some(prev_cell) = prev_cell {
                if prev_cell_empty {
                    let resized_prev_cell =
                        (prev_cell.0 / Self::SUBSAMPLE, prev_cell.1 / Self::SUBSAMPLE);
                    material.grid[spec.flat_index(resized_prev_cell)] = 0;
                }
            }
            let resized_cell = (cell.0 / Self::SUBSAMPLE, cell.1 / Self::SUBSAMPLE);
            material.grid[spec.flat_index(resized_cell)] = 1;
        }
    }
}
impl Material2d for MinimapShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/minimap.wgsl".into()
    }
}

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::zindex;

use super::{EntityGridEvent, GridEntity, GridSpec};

/// Plugin for visualizing the grid.
/// This plugin reads events from the entity grid and updates the shader's input buffer
/// to light up the cells that have entities.
pub struct GridVisualizerPlugin;
impl Plugin for GridVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<GridShaderMaterial>::default())
            .init_resource::<GridAssets>()
            .add_systems(
                FixedUpdate,
                (
                    GridShaderMaterial::update
                        .after(GridEntity::update)
                        .run_if(should_visualize_grid),
                    GridVisualizer::resize_on_change,
                ),
            );
    }
}

/// Returns true if the grid should be visualized.
fn should_visualize_grid(spec: Res<GridSpec>) -> bool {
    spec.visualize
}

/// Component to visualize the grid.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct GridVisualizer {
    pub active: bool,
}
impl GridVisualizer {
    pub fn bundle(self, spec: &GridSpec, assets: &GridAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<GridShaderMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(spec.scale().extend(1.))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: zindex::SHADER_BACKGROUND,
                    }),
                material: assets.shader_material.clone(),
                ..default()
            },
            Name::new("GridVis"),
            self,
        )
    }

    /// When the spec is changed, respawn the visualizer entity with the new size.
    pub fn resize_on_change(
        spec: Res<GridSpec>,
        grid_assets: Res<GridAssets>,
        query: Query<Entity, With<Self>>,
        mut shader_assets: ResMut<Assets<GridShaderMaterial>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for entity in &query {
            commands.entity(entity).despawn();
        }

        let material = shader_assets.get_mut(&grid_assets.shader_material).unwrap();
        material.resize(&spec);

        commands.spawn(GridVisualizer { active: true }.bundle(&spec, &grid_assets));
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GridShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    width: f32,
    #[uniform(2)]
    rows: u32,
    #[uniform(3)]
    cols: u32,
    #[storage(4, read_only)]
    grid: Vec<u32>,
}
impl Default for GridShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            width: 100.,
            rows: 50,
            cols: 100,
            grid: Vec::default(),
        }
    }
}
impl GridShaderMaterial {
    pub fn resize(&mut self, spec: &GridSpec) {
        self.width = spec.width;
        self.rows = spec.rows.into();
        self.cols = spec.cols.into();
        self.grid.resize(spec.rows as usize * spec.cols as usize, 0);
    }
    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        assets: Res<GridAssets>,
        mut shader_assets: ResMut<Assets<GridShaderMaterial>>,
        mut grid_events: EventReader<EntityGridEvent>,
    ) {
        let material: &mut GridShaderMaterial =
            shader_assets.get_mut(&assets.shader_material).unwrap();
        for &EntityGridEvent {
            entity: _,
            prev_cell,
            prev_cell_empty,
            cell,
        } in grid_events.read()
        {
            if let Some(prev_cell) = prev_cell {
                let (prev_row, prev_col) = prev_cell;
                if prev_cell_empty {
                    material.grid[grid_spec.index(prev_row, prev_col)] = 0;
                }
            }
            let (row, col) = cell;
            material.grid[grid_spec.index(row, col)] = 1;
        }
    }
}
impl Material2d for GridShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid_background.wgsl".into()
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct GridAssets {
    pub mesh: Handle<Mesh>,
    pub gray_material: Handle<ColorMaterial>,
    pub dark_gray_material: Handle<ColorMaterial>,
    pub blue_material: Handle<ColorMaterial>,
    pub dark_blue_material: Handle<ColorMaterial>,
    pub shader_material: Handle<GridShaderMaterial>,
}
impl FromWorld for GridAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            // Unit square
            meshes.add(Mesh::from(shape::Box {
                min_x: -0.5,
                max_x: 0.5,
                min_y: -0.5,
                max_y: 0.5,
                min_z: 0.0,
                max_z: 0.0,
            }))
        };
        let shader_material = {
            let mut materials = world
                .get_resource_mut::<Assets<GridShaderMaterial>>()
                .unwrap();
            materials.add(GridShaderMaterial::default())
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            shader_material,
            gray_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.15))),
            dark_gray_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.3))),
            blue_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.1))),

            dark_blue_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.1))),
        }
    }
}

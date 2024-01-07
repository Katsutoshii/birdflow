use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::prelude::*;

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
    size: GridSize,
    #[storage(2, read_only)]
    grid: Vec<u32>,
}
impl Default for GridShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl GridShaderMaterial {
    pub fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
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
                if prev_cell_empty {
                    material.grid[grid_spec.flat_index(prev_cell)] = 0;
                }
            }
            material.grid[grid_spec.flat_index(cell)] = 1;
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
    pub shader_material: Handle<GridShaderMaterial>,
}
impl FromWorld for GridAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(meshes::UNIT_SQUARE))
        };
        let shader_material = {
            let mut materials = world
                .get_resource_mut::<Assets<GridShaderMaterial>>()
                .unwrap();
            materials.add(GridShaderMaterial::default())
        };
        Self {
            mesh,
            shader_material,
        }
    }
}

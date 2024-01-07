use crate::prelude::*;
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use super::navigation::NavigationCostEvent;

pub struct NavigationVisualizerPlugin;
impl Plugin for NavigationVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<NavigationShaderMaterial>::default())
            .init_resource::<NavigationAssets>()
            .add_systems(
                FixedUpdate,
                (
                    NavigationShaderMaterial::update,
                    NavigationVisualizer::resize_on_change,
                ),
            );
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct NavigationShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    width: f32,
    #[uniform(2)]
    rows: u32,
    #[uniform(3)]
    cols: u32,
    #[storage(4, read_only)]
    grid: Vec<f32>,
}
impl Default for NavigationShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::ORANGE_RED,
            width: 100.,
            rows: 50,
            cols: 100,
            grid: Vec::default(),
        }
    }
}
impl NavigationShaderMaterial {
    pub fn resize(&mut self, spec: &GridSpec) {
        self.width = spec.width;
        self.rows = spec.rows.into();
        self.cols = spec.cols.into();
        self.grid
            .resize(spec.rows as usize * spec.cols as usize, 0.);
    }
    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        mut events: EventReader<NavigationCostEvent>,
        assets: Res<NavigationAssets>,
        mut shader_assets: ResMut<Assets<NavigationShaderMaterial>>,
        mut input_actions: EventReader<InputActionEvent>,
    ) {
        let material: &mut NavigationShaderMaterial =
            shader_assets.get_mut(&assets.shader_material).unwrap();
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
            material.grid[grid_spec.flat_index(rowcol)] = cost * 0.005;
        }
    }
}
impl Material2d for NavigationShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/navigation_shader.wgsl".into()
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct NavigationAssets {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<NavigationShaderMaterial>,
}
impl FromWorld for NavigationAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(meshes::UNIT_SQUARE))
        };
        let shader_material = {
            let mut materials = world
                .get_resource_mut::<Assets<NavigationShaderMaterial>>()
                .unwrap();
            materials.add(NavigationShaderMaterial::default())
        };
        Self {
            mesh,
            shader_material,
        }
    }
}

/// Component to visualize the grid.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct NavigationVisualizer {
    pub active: bool,
}
impl NavigationVisualizer {
    pub fn bundle(self, spec: &GridSpec, assets: &NavigationAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<NavigationShaderMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(spec.scale().extend(1.))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: zindex::NAVIGATION_LAYER,
                    }),
                material: assets.shader_material.clone(),
                ..default()
            },
            Name::new("NavVis"),
            self,
        )
    }

    /// When the spec is changed, respawn the visualizer entity with the new size.
    pub fn resize_on_change(
        spec: Res<GridSpec>,
        nav_assets: Res<NavigationAssets>,
        query: Query<Entity, With<Self>>,
        mut shader_assets: ResMut<Assets<NavigationShaderMaterial>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for entity in &query {
            commands.entity(entity).despawn();
        }

        let material = shader_assets.get_mut(&nav_assets.shader_material).unwrap();
        material.resize(&spec);

        commands.spawn(NavigationVisualizer { active: true }.bundle(&spec, &nav_assets));
    }
}

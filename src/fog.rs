use crate::{grid::EntityGridSpec, objects::Team, prelude::*};
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct FogPlugin;
impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FogShaderMaterial>::default())
            .init_resource::<FogAssets>();
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct FogAssets {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<FogShaderMaterial>,
}
impl FromWorld for FogAssets {
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
                .get_resource_mut::<Assets<FogShaderMaterial>>()
                .unwrap();
            materials.add(FogShaderMaterial::default())
        };
        Self {
            mesh,
            shader_material,
        }
    }
}

/// Component to visualize a cell.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct FogPlane {
    pub team: Team,
}
impl FogPlane {
    pub fn bundle(self, spec: &EntityGridSpec, assets: &FogAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<FogShaderMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(spec.scale().extend(1.))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: zindex::FOG_OF_WAR,
                    }),
                material: assets.shader_material.clone(),
                ..default()
            },
            Name::new("GridVis"),
            self,
        )
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FogShaderMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(1)]
    pub width: f32,
    #[uniform(2)]
    pub rows: u32,
    #[uniform(3)]
    pub cols: u32,
    #[storage(4, read_only)]
    pub grid: Vec<f32>,
}
impl Default for FogShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 100.,
            rows: 50,
            cols: 100,
            grid: Vec::default(),
        }
    }
}
impl FogShaderMaterial {
    pub fn resize(&mut self, spec: &EntityGridSpec) {
        self.width = spec.width;
        self.rows = spec.rows.into();
        self.cols = spec.cols.into();
        self.grid
            .resize(spec.rows as usize * spec.cols as usize, 1.);
    }
}
impl Material2d for FogShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fog_of_war.wgsl".into()
    }
}

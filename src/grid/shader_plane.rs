use crate::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use std::marker::PhantomData;

/// Plugin for a 2D plane with a shader material.
#[derive(Default)]
pub struct ShaderPlanePlugin<M: GridShaderMaterial>(PhantomData<M>);
impl<M: GridShaderMaterial> Plugin for ShaderPlanePlugin<M>
where
    Material2dPlugin<M>: Plugin,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<M>::default())
            .init_resource::<ShaderPlaneAssets<M>>()
            .add_systems(FixedUpdate, M::resize_on_change);
    }
}

/// Trait must be implemented by all Plane shaders.
pub trait GridShaderMaterial: Material2d + Default {
    /// Return the zindex for the position of the grid.
    fn zindex() -> f32;

    fn resize(&mut self, spec: &GridSpec);

    /// When the spec is changed, respawn the visualizer entity with the new size.
    fn resize_on_change(
        spec: Res<GridSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        query: Query<Entity, With<ShaderPlane<Self>>>,
        mut shader_assets: ResMut<Assets<Self>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for entity in &query {
            commands.entity(entity).despawn();
        }

        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        material.resize(&spec);

        commands.spawn(ShaderPlane::<Self>::default().bundle(&spec, &assets));
    }
}

/// Component that marks an entity as a shader plane.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct ShaderPlane<M: GridShaderMaterial>(PhantomData<M>);
impl<M: GridShaderMaterial> ShaderPlane<M> {
    pub fn bundle(self, spec: &GridSpec, assets: &ShaderPlaneAssets<M>) -> impl Bundle {
        (
            MaterialMesh2dBundle::<M> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(spec.scale().extend(1.))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: M::zindex(),
                    }),
                material: assets.shader_material.clone(),
                ..default()
            },
            Name::new("ShaderPlane"),
            self,
        )
    }
}

/// Handles to shader plane assets.
#[derive(Resource)]
pub struct ShaderPlaneAssets<M: Material2d> {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<M>,
}
impl<M: Material2d + Default> FromWorld for ShaderPlaneAssets<M> {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(meshes::UNIT_SQUARE))
        };
        let shader_material = {
            let mut materials = world.get_resource_mut::<Assets<M>>().unwrap();
            materials.add(M::default())
        };
        Self {
            mesh,
            shader_material,
        }
    }
}

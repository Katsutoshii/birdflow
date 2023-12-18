use crate::{
    objects::{Configs, Team},
    prelude::*,
};
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use super::{EntityGridEvent, Grid2, GridEntity, GridSpec};

/// Plugin for fog of war.
pub struct FogPlugin;
impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FogShaderMaterial>::default())
            .insert_resource(VisibilityGrid::default())
            .init_resource::<FogAssets>()
            .add_systems(
                FixedUpdate,
                (
                    VisibilityGrid::update.after(grid::GridEntity::update),
                    VisibilityGrid::resize_on_change,
                    VisibilityGrid::update_visibility.after(VisibilityGrid::update),
                ),
            );
    }
}

/// Stores visibility per team.
#[derive(Clone, Default)]
pub struct TeamVisibility {
    teams: [u32; Team::count()],
}
impl TeamVisibility {
    pub fn get(&self, team: Team) -> u32 {
        self.teams[team as usize]
    }

    pub fn get_mut(&mut self, team: Team) -> &mut u32 {
        &mut self.teams[team as usize]
    }
}

/// Handles to common fog assets.
#[derive(Resource, Default, Deref, DerefMut)]
struct VisibilityGrid(pub Grid2<TeamVisibility>);
impl VisibilityGrid {
    pub fn resize_on_change(
        mut grid: ResMut<Self>,
        spec: Res<GridSpec>,
        assets: Res<FogAssets>,
        query: Query<Entity, With<FogPlane>>,
        mut shader_assets: ResMut<Assets<FogShaderMaterial>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }
        for entity in &query {
            commands.entity(entity).despawn();
        }

        grid.0.resize_with(spec.clone());

        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        material.resize(&spec);

        commands.spawn(FogPlane.bundle(&spec, &assets));
    }

    pub fn update_visibility(
        mut query: Query<(&GridEntity, &mut Visibility)>,
        grid: ResMut<Self>,
        configs: Res<Configs>,
    ) {
        for (grid_entity, mut visibility) in &mut query {
            if let Some(cell) = grid_entity.cell {
                *visibility = grid.get_visibility(cell, configs.player_team)
            }
        }
    }

    pub fn update(
        mut grid: ResMut<Self>,
        configs: Res<Configs>,
        assets: Res<FogAssets>,
        teams: Query<&Team>,
        mut shader_assets: ResMut<Assets<FogShaderMaterial>>,
        mut grid_events: EventReader<EntityGridEvent>,
    ) {
        let material: &mut FogShaderMaterial =
            shader_assets.get_mut(&assets.shader_material).unwrap();
        for &EntityGridEvent {
            entity,
            prev_cell,
            prev_cell_empty: _,
            cell,
        } in grid_events.read()
        {
            let team = *teams.get(entity).unwrap();
            if let Some(prev_cell) = prev_cell {
                grid.remove_visibility(prev_cell, team, &configs, &mut material.grid)
            }
            grid.add_visibility(cell, team, &configs, &mut material.grid);
        }
    }

    fn remove_visibility(
        &mut self,
        cell: (u16, u16),
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for (other_row, other_col) in self.0.get_in_radius_discrete(cell, radius) {
            if let Some(grid_visibility) = self.0.get_mut(other_row, other_col) {
                if grid_visibility.get(team) > 0 {
                    *grid_visibility.get_mut(team) -= 1;
                    if team == configs.player_team && grid_visibility.get(team) == 0 {
                        visibility[self.0.spec.index(other_row, other_col)] = 0.5;
                    }
                }
            }
        }
    }

    /// Return the visibility status at the cell corresponding to position for the given team.
    pub fn get_visibility(&self, cell: (u16, u16), team: Team) -> Visibility {
        let (row, col) = cell;
        if let Some(visibility) = self.0.get(row, col) {
            if visibility.get(team) > 0 {
                return Visibility::Visible;
            }
        }
        Visibility::Hidden
    }

    fn add_visibility(
        &mut self,
        cell: (u16, u16),
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for (other_row, other_col) in self.0.get_in_radius_discrete(cell, radius) {
            if let Some(grid_visibility) = self.0.get_mut(other_row, other_col) {
                *grid_visibility.get_mut(team) += 1;
                if team == configs.player_team
                    && Grid2::<()>::in_radius(cell, (other_row, other_col), configs.fog_radius)
                {
                    visibility[self.0.spec.index(other_row, other_col)] = 0.
                }
            }
        }
    }
}

/// Handles to common fog assets.
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

/// Fog plane between the world and the camera.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct FogPlane;
impl FogPlane {
    pub fn bundle(self, spec: &GridSpec, assets: &FogAssets) -> impl Bundle {
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
    pub fn resize(&mut self, spec: &GridSpec) {
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

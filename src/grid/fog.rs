use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use crate::prelude::*;

use super::{
    shader_plane::{ShaderPlaneAssets, ShaderPlanePlugin},
    GridShaderMaterial,
};

/// Plugin for fog of war.
pub struct FogPlugin;
impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<FogShaderMaterial>::default())
            .add_plugins(Grid2Plugin::<TeamVisibility>::default())
            .add_systems(
                FixedUpdate,
                (
                    Grid2::<TeamVisibility>::update.after(GridEntity::update),
                    Grid2::<TeamVisibility>::update_visibility
                        .after(Grid2::<TeamVisibility>::update),
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

impl Grid2<TeamVisibility> {
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
        assets: Res<ShaderPlaneAssets<FogShaderMaterial>>,
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
        rowcol: RowCol,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for other_rowcol in self.get_in_radius_discrete(rowcol, radius) {
            if let Some(grid_visibility) = self.get_mut(other_rowcol) {
                if grid_visibility.get(team) > 0 {
                    *grid_visibility.get_mut(team) -= 1;
                    if team == configs.player_team && grid_visibility.get(team) == 0 {
                        let index = self.flat_index(other_rowcol);
                        visibility[index] = 0.5;
                    }
                }
            }
        }
    }

    /// Return the visibility status at the cell corresponding to position for the given team.
    pub fn get_visibility(&self, rowcol: RowCol, team: Team) -> Visibility {
        if let Some(visibility) = self.get(rowcol) {
            if visibility.get(team) > 0 {
                return Visibility::Visible;
            }
        }
        Visibility::Hidden
    }

    fn add_visibility(
        &mut self,
        cell: RowCol,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for other_rowcol in self.get_in_radius_discrete(cell, radius) {
            if let Some(grid_visibility) = self.get_mut(other_rowcol) {
                *grid_visibility.get_mut(team) += 1;
                if team == configs.player_team
                    && Grid2::<()>::in_radius(cell, other_rowcol, configs.fog_radius)
                {
                    visibility[self.flat_index(other_rowcol)] = 0.
                }
            }
        }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FogShaderMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(1)]
    pub size: GridSize,
    #[storage(2, read_only)]
    pub grid: Vec<f32>,
}
impl Default for FogShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl GridShaderMaterial for FogShaderMaterial {
    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid
            .resize(spec.rows as usize * spec.cols as usize, 1.);
    }
    fn translation(_window: &Window, _spec: &GridSpec) -> Vec3 {
        Vec2::ZERO.extend(zindex::FOG_OF_WAR)
    }
}
impl Material2d for FogShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fog_of_war.wgsl".into()
    }
}

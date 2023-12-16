use std::ops::RangeInclusive;

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    utils::{HashMap, HashSet},
};

mod spec;
pub use spec::GridSpec;
mod fog;
pub use fog::FogPlugin;

use crate::{
    grid::fog::{FogAssets, FogShaderMaterial},
    objects::{Configs, Team},
    zindex, Aabb2, SystemStage,
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .add_plugins(Material2dPlugin::<GridShaderMaterial>::default())
            .add_plugins(FogPlugin)
            .init_resource::<GridAssets>()
            .insert_resource(EntityGrid::default())
            .add_systems(
                FixedUpdate,
                (
                    GridEntity::update.in_set(SystemStage::PostApply),
                    GridSpec::visualize_on_change,
                    GridSpec::resize_on_change,
                ),
            );
    }
}

/// Component to track an entity in the grid.
/// This also tracks visibility.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GridEntity;
impl GridEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut query: Query<(Entity, &Transform, &Team, &mut Visibility), With<Self>>,
        mut grid: ResMut<EntityGrid>,
        grid_assets: Res<GridAssets>,
        mut grid_shader_assets: ResMut<Assets<GridShaderMaterial>>,
        fog_assets: Res<FogAssets>,
        mut fog_shader_assets: ResMut<Assets<FogShaderMaterial>>,
        spec: Res<GridSpec>,
        configs: Res<Configs>,
    ) {
        // Initialize the shader if not yet initialized.
        let grid_material: &mut GridShaderMaterial = grid_shader_assets
            .get_mut(&grid_assets.shader_material)
            .unwrap();
        let fog_material = fog_shader_assets
            .get_mut(&fog_assets.shader_material)
            .unwrap();
        for (entity, transform, team, mut visibility) in &mut query {
            let grid_material = if spec.visualize {
                Some(grid_material.grid.as_mut_slice())
            } else {
                None
            };
            grid.update_entity(
                entity,
                transform.translation.xy(),
                *team,
                &configs,
                &mut fog_material.grid,
                grid_material,
            );
            *visibility = grid.get_visibility(transform.translation.xy(), configs.player_team)
        }
    }
}

/// A grid of cells that keep track of what entities are contained within them.
#[derive(Resource, Default)]
pub struct EntityGrid {
    pub spec: GridSpec,
    pub entities: Vec<HashSet<Entity>>,
    pub team_visibility: Vec<Vec<u32>>,
    pub entity_to_rowcol: HashMap<Entity, (u16, u16)>,
}
impl EntityGrid {
    pub fn resize(&mut self) {
        let num_cells = self.spec.rows as usize * self.spec.cols as usize;
        self.entities.resize(num_cells, HashSet::default());
        self.team_visibility
            .resize(num_cells, vec![0; Team::count()])
    }

    /// Update an entity's position in the grid.
    pub fn update_entity(
        &mut self,
        entity: Entity,
        position: Vec2,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
        mut grid: Option<&mut [u32]>,
    ) {
        let (row, col) = self.to_rowcol(position);

        // Remove this entity's old position if it was different.
        if let Some(&(old_row, old_col)) = self.entity_to_rowcol.get(&entity) {
            // If in same position, do nothing.
            if (old_row, old_col) == (row, col) {
                return;
            }

            if let Some(entities) = self.get_mut(old_row, old_col) {
                entities.remove(&entity);
                if let Some(grid) = grid.as_deref_mut() {
                    if entities.is_empty() {
                        grid[self.index(old_row, old_col)] = 0;
                    }
                }
                self.remove_visibility(old_row, old_col, team, configs, visibility);
            }
        }

        if let Some(entities) = self.get_mut(row, col) {
            entities.insert(entity);
            self.entity_to_rowcol.insert(entity, (row, col));
            if let Some(grid) = grid {
                grid[self.index(row, col)] = 1;
            }
            self.add_visibility(row, col, team, configs, visibility);
        }
    }

    fn in_radius(row: u16, col: u16, other_row: u16, other_col: u16, radius: u16) -> bool {
        let row_dist = other_row as i16 - row as i16;
        let col_dist = other_col as i16 - col as i16;
        row_dist * row_dist + col_dist * col_dist < radius as i16 * radius as i16
    }

    fn cell_range(center: u16, radius: u16) -> RangeInclusive<u16> {
        let (min, max) = (
            (center as i16 - radius as i16).max(0) as u16,
            center + radius,
        );
        min..=max
    }

    fn remove_visibility(
        &mut self,
        row: u16,
        col: u16,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for other_row in Self::cell_range(row, radius) {
            for other_col in Self::cell_range(col, radius) {
                if !Self::in_radius(row, col, other_row, other_col, radius) {
                    continue;
                }

                let i = self.index(other_row, other_col);
                if let Some(grid_visibility) = self.team_visibility.get_mut(i) {
                    if grid_visibility[team as usize] > 0 {
                        grid_visibility[team as usize] -= 1;
                        if team == configs.player_team && grid_visibility[team as usize] == 0 {
                            visibility[i] = 0.5;
                        }
                    }
                }
            }
        }
    }

    fn add_visibility(
        &mut self,
        row: u16,
        col: u16,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for other_row in Self::cell_range(row, radius) {
            for other_col in Self::cell_range(col, radius) {
                if !Self::in_radius(row, col, other_row, other_col, radius) {
                    continue;
                }

                let i = self.index(other_row, other_col);
                if let Some(grid_visibility) = self.team_visibility.get_mut(i) {
                    grid_visibility[team as usize] += 1;
                    if team == configs.player_team
                        && Self::in_radius(row, col, other_row, other_col, configs.fog_radius)
                    {
                        visibility[i] = 0.
                    }
                }
            }
        }
    }

    /// Remove an entity from the grid entirely.
    #[allow(dead_code)]
    pub fn remove(&mut self, entity: Entity) {
        if let Some(&(row, col)) = self.entity_to_rowcol.get(&entity) {
            if let Some(cell) = self.get_mut(row, col) {
                cell.remove(&entity);
            } else {
                error!("No cell at {:?}.", (row, col))
            }
        } else {
            error!("No row col for {:?}", entity)
        }
    }

    /// Get the set of entities at the current position.
    pub fn get(&self, row: u16, col: u16) -> Option<&HashSet<Entity>> {
        let index = self.index(row, col);
        self.entities.get(index)
    }

    /// Return the visibility status at the cell corresponding to position for the given team.
    pub fn get_visibility(&self, position: Vec2, team: Team) -> Visibility {
        let (row, col) = self.to_rowcol(position);
        let i = self.index(row, col);
        if self.team_visibility[i][team as usize] > 0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    }

    /// Get the mutable set of entities at the current position.
    pub fn get_mut(&mut self, row: u16, col: u16) -> Option<&mut HashSet<Entity>> {
        let index = self.index(row, col);
        self.entities.get_mut(index)
    }

    /// Get all entities in a given bounding box.
    pub fn get_in_aabb(&self, aabb: &Aabb2) -> Vec<Entity> {
        let mut result = HashSet::default();

        let (min_row, min_col) = self.to_rowcol(aabb.min);
        let (max_row, max_col) = self.to_rowcol(aabb.max);
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                if let Some(set) = self.get(row, col) {
                    result.extend(set.iter());
                }
            }
        }
        result.into_iter().collect()
    }

    /// Get all entities in radius.
    pub fn get_in_radius(&self, position: Vec2, radius: f32) -> Vec<Entity> {
        self.get_in_aabb(&Aabb2 {
            min: position + (Vec2::NEG_ONE * radius),
            max: position + (Vec2::ONE * radius),
        })
    }

    /// Returns (row, col) from a position in world space.
    fn to_rowcol(&self, mut position: Vec2) -> (u16, u16) {
        position += self.spec.offset();
        (
            (position.y / self.spec.width) as u16,
            (position.x / self.spec.width) as u16,
        )
    }

    // Covert row, col to a single index.
    fn index(&self, row: u16, col: u16) -> usize {
        row as usize * self.spec.cols as usize + col as usize
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

/// Component to visualize a cell.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct CellVisualizer {
    pub active: bool,
}
impl CellVisualizer {
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
    fn resize(&mut self, spec: &GridSpec) {
        self.width = spec.width;
        self.rows = spec.rows.into();
        self.cols = spec.cols.into();
        self.grid.resize(spec.rows as usize * spec.cols as usize, 0);
    }
}
impl Material2d for GridShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid_background.wgsl".into()
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::GridSpec;

    use super::EntityGrid;
    use bevy::{prelude::*, utils::HashMap};

    #[test]
    fn test_update() {
        let mut grid = EntityGrid {
            spec: GridSpec {
                rows: 10,
                cols: 10,
                width: 10.0,
                visualize: false,
            },
            entities: Vec::default(),
            entity_to_rowcol: HashMap::default(),
            team_visibility: Vec::default(),
        };
        grid.resize();
        assert_eq!(grid.spec.offset(), Vec2 { x: 50.0, y: 50.0 });
        let rowcol = grid.to_rowcol(Vec2 { x: 0., y: 0. });
        assert_eq!(rowcol, (5, 5));

        assert!(grid.get_mut(5, 5).is_some());
        assert!(grid.get(5, 5).is_some());
    }

    #[test]
    fn grid_radius() {
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (2, 2);
            let radius = 2;
            assert!(EntityGrid::in_radius(
                row, col, other_row, other_col, radius
            ));
        }
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (4, 4);
            let radius = 2;
            assert!(!EntityGrid::in_radius(
                row, col, other_row, other_col, radius
            ));
        }
    }
}
